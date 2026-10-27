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
    /// Text or regular expression query string.
    pub query: String,
    /// Whether `query` should be treated as a regular expression.
    pub is_regex: bool,
    /// Whether searches should match case exactly.
    pub case_sensitive: bool,
    /// Optional file inclusion glob pattern (e.g. `"*.rs"`).
    pub include_glob: String,
    /// Optional file exclusion glob pattern (e.g. `"*.min.js"`).
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

        if !self.include_glob.trim().is_empty() {
            args.push("--glob".to_string());
            args.push(self.include_glob.trim().to_string());
        }

        if !self.exclude_glob.trim().is_empty() {
            args.push("--glob".to_string());
            args.push(format!("!{}", self.exclude_glob.trim()));
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
    /// Returns [`SearchError::NotFound`] if `rg` is not installed.
    /// Returns [`SearchError::Process`] if `rg` terminates with an unexpected error status.
    pub async fn search(
        root: &Path,
        options: &SearchOptions,
        cancel: Arc<AtomicBool>,
        mut on_batch: impl FnMut(Vec<SearchMatch>),
    ) -> Result<bool, SearchError> {
        if options.query.trim().is_empty() {
            return Ok(false);
        }

        let mut child = Command::new("rg")
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
            })?;

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

        while let Some(line) = reader.next_line().await? {
            if cancel.load(Ordering::Relaxed) {
                let _ = child.kill().await;
                let _ = stderr_task.await;
                return Ok(false);
            }

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

        if !limited
            && !cancel.load(Ordering::Relaxed)
            && !matches!(status.code(), Some(0) | Some(1))
        {
            let trimmed = stderr_output.trim();
            let msg = if trimmed.is_empty() {
                format!("ripgrep exited with code {:?}", status.code())
            } else {
                trimmed.to_string()
            };
            return Err(SearchError::Process(msg));
        }

        Ok(limited)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_options_args() {
        let opt = SearchOptions {
            query: "SourceBuffer".to_string(),
            case_sensitive: true,
            include_glob: "*.rs".to_string(),
            ..Default::default()
        };

        let args = opt.build_arguments();
        assert!(args.contains(&"--case-sensitive".to_string()));
        assert!(args.contains(&"--fixed-strings".to_string()));
        assert!(args.contains(&"*.rs".to_string()));
        assert!(args.contains(&"SourceBuffer".to_string()));
    }
}
