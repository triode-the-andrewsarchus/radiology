//! Asynchronous ripgrep process execution and options management.

use crate::search::match_item::SearchMatch;
use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::Command;

/// Maximum number of search matches collected per query to prevent runaway resource consumption.
pub const SEARCH_RESULT_LIMIT: usize = 5000;

/// Describes errors encountered during search execution.
#[derive(Error, Debug)]
pub enum SearchError {
    /// The caller cancelled the request before it completed.
    #[error("Search cancelled")]
    Cancelled,
    /// The `rg` executable was not found in `PATH`.
    #[error("ripgrep (rg) was not found in PATH")]
    NotFound,
    /// Ripgrep terminated with an unexpected non-zero exit code.
    #[error("Search execution failed: {0}")]
    Process(String),
    /// An I/O error occurred while interacting with the process pipes.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Options and filter parameters for a repository search.
///
/// # Examples
///
/// ```rust
/// use radiology::search::ripgrep::SearchOptions;
///
/// let mut opts = SearchOptions::default();
/// opts.query = "SourceBuffer".to_string();
/// opts.case_sensitive = true;
///
/// let args = opts.build_arguments();
/// assert!(args.contains(&"--case-sensitive".to_string()));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SearchOptions {
    /// Text or regular expression query string; nonempty whitespace is significant.
    pub query: String,
    /// Whether `query` should be treated as a regular expression.
    pub is_regex: bool,
    /// Whether searches should match case exactly.
    pub case_sensitive: bool,
    /// Optional file inclusion glob pattern (e.g. `"*.rs"`), passed without trimming.
    pub include_glob: String,
    /// Optional file exclusion glob pattern (e.g. `"*.min.js"`), passed without trimming.
    pub exclude_glob: String,
}

impl SearchOptions {
    /// Constructs the command-line argument list passed to `rg`.
    #[must_use]
    pub fn build_arguments(&self) -> Vec<String> {
        let mut args = vec![
            "--json".to_string(),
            "--line-number".to_string(),
            "--column".to_string(),
            "--hidden".to_string(),
            "--glob".to_string(),
            "!.git/**".to_string(),
            "--glob".to_string(),
            "!.build/**".to_string(),
            "--glob".to_string(),
            "!build/**".to_string(),
            "--glob".to_string(),
            "!dist/**".to_string(),
            "--glob".to_string(),
            "!node_modules/**".to_string(),
            "--glob".to_string(),
            "!vendor/**".to_string(),
            "--glob".to_string(),
            "!target/**".to_string(),
            "--glob".to_string(),
            "!DerivedData/**".to_string(),
        ];

        if !self.is_regex {
            args.push("--fixed-strings".to_string());
        }

        if self.case_sensitive {
            args.push("--case-sensitive".to_string());
        } else {
            args.push("--ignore-case".to_string());
        }

        if !self.include_glob.is_empty() {
            args.push("--glob".to_string());
            args.push(self.include_glob.clone());
        }

        if !self.exclude_glob.is_empty() {
            args.push("--glob".to_string());
            args.push(format!("!{}", self.exclude_glob));
        }

        args.push("--".to_string());
        args.push(self.query.clone());
        args.push(".".to_string());

        args
    }
}

/// Runs asynchronous ripgrep streaming searches.
pub struct SearchRunner;

impl SearchRunner {
    /// Executes a search within `root`, streaming batches of [`SearchMatch`] into `on_batch`.
    ///
    /// Stderr is drained asynchronously to prevent pipe deadlocks.
    /// Returns `true` if search was capped at [`SEARCH_RESULT_LIMIT`].
    ///
    /// # Errors
    ///
    /// Returns [`SearchError::Cancelled`] if the cancellation flag is set before
    /// launch or during execution. Already delivered batches remain owned by the caller.
    /// Returns [`SearchError::NotFound`] if `rg` is not installed.
    /// Returns [`SearchError::Process`] if `rg` terminates with an unexpected error status.
    pub async fn search(
        root: &Path,
        options: &SearchOptions,
        cancel: Arc<AtomicBool>,
        on_batch: impl FnMut(Vec<SearchMatch>),
    ) -> Result<bool, SearchError> {
        Self::search_cancellable(root, options, &cancel, on_batch).await
    }

    fn spawn(root: &Path, options: &SearchOptions) -> Result<tokio::process::Child, SearchError> {
        Command::new("rg")
            .args(options.build_arguments())
            .current_dir(root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    SearchError::NotFound
                } else {
                    SearchError::Io(e)
                }
            })
    }

    pub(crate) async fn search_cancellable(
        root: &Path,
        options: &SearchOptions,
        cancel: &AtomicBool,
        mut on_batch: impl FnMut(Vec<SearchMatch>),
    ) -> Result<bool, SearchError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(SearchError::Cancelled);
        }
        if options.query.is_empty() {
            return Ok(false);
        }

        let mut child = Self::spawn(root, options)?;

        let stdout = child.stdout.take().expect("stdout piped");
        let mut stderr = child.stderr.take().expect("stderr piped");

        // Concurrently drain stderr to prevent pipe buffer deadlock
        let stderr_task = tokio::spawn(async move {
            let mut err_str = String::new();
            let _ = stderr.read_to_string(&mut err_str).await;
            err_str
        });

        let mut reader = BufReader::new(stdout).lines();

        let mut batch = Vec::with_capacity(64);
        let mut total_count = 0;
        let mut limited = false;

        loop {
            if cancel.load(Ordering::Relaxed) {
                let _ = child.kill().await;
                let _ = stderr_task.await;
                return Err(SearchError::Cancelled);
            }

            let line = tokio::select! {
                line = reader.next_line() => line?,
                _ = tokio::time::sleep(std::time::Duration::from_millis(50)) => continue,
            };
            let Some(line) = line else {
                break;
            };

            let matches = SearchMatch::decode_rg_json_line(&line);
            for m in matches {
                batch.push(m);
                total_count += 1;
                if total_count >= SEARCH_RESULT_LIMIT {
                    limited = true;
                    let _ = child.kill().await;
                    break;
                }
            }

            if batch.len() >= 40 {
                on_batch(std::mem::take(&mut batch));
            }

            if limited {
                break;
            }
        }

        if !batch.is_empty() {
            on_batch(batch);
        }

        let status = child.wait().await?;
        let stderr_output = stderr_task.await.unwrap_or_default();

        match (cancel.load(Ordering::Relaxed), limited, status.code()) {
            (true, _, _) => Err(SearchError::Cancelled),
            (_, true, _) | (_, _, Some(0 | 1)) => Ok(limited),
            _ => {
                let trimmed = stderr_output.trim();
                let message = if trimmed.is_empty() {
                    format!("ripgrep exited with code {:?}", status.code())
                } else {
                    trimmed.to_string()
                };
                Err(SearchError::Process(message))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_options_args() {
        for is_regex in [false, true] {
            for case_sensitive in [false, true] {
                let options = SearchOptions {
                    query: "--help".into(),
                    is_regex,
                    case_sensitive,
                    include_glob: "*.swift".into(),
                    exclude_glob: "Tests/**".into(),
                };
                let args = options.build_arguments();
                assert_eq!(&args[args.len() - 3..], ["--", "--help", "."]);
                assert_eq!(args.iter().any(|arg| arg == "--fixed-strings"), !is_regex);
                assert_eq!(
                    args.iter().any(|arg| arg == "--case-sensitive"),
                    case_sensitive
                );
                assert_eq!(
                    args.iter().any(|arg| arg == "--ignore-case"),
                    !case_sensitive
                );
                for glob in ["*.swift", "!Tests/**", "!.git/**", "!target/**"] {
                    assert!(args.windows(2).any(|pair| pair == ["--glob", glob]));
                }
            }
        }
    }

    #[tokio::test]
    async fn streaming_search_caps_submatches_and_reports_cancellation() {
        let owner = tempfile::tempdir().unwrap();
        // The first case crosses the limit inside one JSON record.
        // The cancellation cases cover launch, the final short batch and streaming.
        for (line, repeats, cancel_after, expected_count) in [
            ("needle needle needle\n", 2000, None, SEARCH_RESULT_LIMIT),
            ("needle\n", 0, Some(0), 0),
            ("needle\n", 10, Some(1), 10),
            ("needle\n", 10000, Some(1), 40),
        ] {
            std::fs::write(owner.path().join("large.txt"), line.repeat(repeats)).unwrap();
            let cancel = Arc::new(AtomicBool::new(cancel_after == Some(0)));
            let mut matches = Vec::new();
            let result = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                SearchRunner::search(
                    owner.path(),
                    &SearchOptions {
                        query: "needle".into(),
                        ..Default::default()
                    },
                    Arc::clone(&cancel),
                    |batch| {
                        matches.extend(batch);
                        if cancel_after.is_some_and(|count| matches.len() >= count) {
                            cancel.store(true, Ordering::Relaxed);
                        }
                    },
                ),
            )
            .await
            .expect("search did not finish");
            if cancel_after.is_some() {
                assert!(matches!(result, Err(SearchError::Cancelled)), "{result:?}");
            } else {
                assert!(result.unwrap());
            }
            assert_eq!(matches.len(), expected_count);
            let words_per_line = line.split_whitespace().count();
            for (index, hit) in matches.iter().enumerate() {
                assert_eq!(hit.path, "large.txt");
                assert_eq!(hit.line, index / words_per_line + 1);
                assert_eq!(hit.byte_offset, index * 7);
            }
        }
    }
}
