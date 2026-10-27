use crate::search::match_item::SearchMatch;
use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

pub const SEARCH_RESULT_LIMIT: usize = 5000;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("ripgrep (rg) was not found in PATH")]
    NotFound,
    #[error("Search execution failed: {0}")]
    Process(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SearchOptions {
    pub query: String,
    pub is_regex: bool,
    pub case_sensitive: bool,
    pub include_glob: String,
    pub exclude_glob: String,
}

impl SearchOptions {
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

pub struct SearchRunner;

impl SearchRunner {
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
        let mut reader = BufReader::new(stdout).lines();

        let mut batch = Vec::with_capacity(64);
        let mut total_count = 0;
        let mut limited = false;

        while let Some(line) = reader.next_line().await? {
            if cancel.load(Ordering::Relaxed) {
                let _ = child.kill().await;
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

            if batch.len() >= 50 {
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
        // Exit code 0 means matches found, 1 means no matches found (both normal for ripgrep)
        if !limited
            && !cancel.load(Ordering::Relaxed)
            && !matches!(status.code(), Some(0) | Some(1))
        {
            return Err(SearchError::Process(format!(
                "ripgrep exited with code {:?}",
                status.code()
            )));
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
