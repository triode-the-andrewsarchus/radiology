//! GitHub CLI (`gh`) integration for pull request summaries and diff retrieval.

use super::control::ReviewControl;
use crate::process::{self, ProcessCommand};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

/// Summary metadata for a single GitHub pull request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullRequestSummary {
    /// Pull request number.
    pub number: usize,
    /// Title of the pull request.
    pub title: String,
    /// State string (e.g. `"OPEN"`, `"MERGED"`, `"CLOSED"`).
    pub state: String,
    /// Name of the base branch receiving the changes.
    #[serde(rename = "baseRefName", default)]
    pub base_ref_name: String,
    /// Name of the feature branch containing the changes.
    #[serde(rename = "headRefName", default)]
    pub head_ref_name: String,
    /// Commit hash of the pull request head.
    #[serde(rename = "headRefOid", default)]
    pub head_ref_oid: Option<String>,
    /// Exact base branch commit used to prepare the review merge base.
    #[serde(rename = "baseRefOid", default)]
    pub base_ref_oid: Option<String>,
    /// Last update timestamp string.
    #[serde(rename = "updatedAt", default)]
    pub updated_at: String,
}

impl PullRequestSummary {
    /// Returns a reader-facing title for the GitHub state, preserving unknown states.
    #[must_use]
    pub fn state_title(&self) -> &str {
        match self.state.as_str() {
            "OPEN" => "Open",
            "MERGED" => "Merged",
            "CLOSED" => "Closed",
            _ => &self.state,
        }
    }

    /// Returns `true` if the pull request state is open.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.state.eq_ignore_ascii_case("OPEN")
    }

    /// Returns `true` if the pull request has been merged.
    #[must_use]
    pub fn is_merged(&self) -> bool {
        self.state.eq_ignore_ascii_case("MERGED")
    }
}

/// Read-only client executing GitHub operations via the `gh` command-line tool.
///
/// Requests have bounded output and a two-minute deadline. Availability probes
/// use five seconds. Interactive prompts are disabled in child processes.
pub struct GitHubClient;

impl GitHubClient {
    /// Checks whether the `gh` executable is available in `PATH`.
    #[must_use]
    pub fn is_gh_available() -> bool {
        let cancel = AtomicBool::new(false);
        let mut control = ReviewControl::new(&cancel);
        control.deadline = Instant::now() + std::time::Duration::from_secs(5);
        Self::request(Path::new("."), &["--version"], &control).is_ok()
    }

    /// Lists recent pull requests in the repository at `root`.
    ///
    /// The requested limit is clamped to at least one. Invalid or duplicate
    /// numbers and responses exceeding the requested limit are rejected.
    ///
    /// # Errors
    ///
    /// Returns an error message string if `gh` execution fails or returns invalid JSON.
    pub fn list_pull_requests(
        root: &Path,
        limit: usize,
    ) -> Result<Vec<PullRequestSummary>, String> {
        Self::list_cancellable(root, limit, &ReviewControl::new(&AtomicBool::new(false)))
    }

    pub(crate) fn list_cancellable(
        root: &Path,
        limit: usize,
        control: &ReviewControl<'_>,
    ) -> Result<Vec<PullRequestSummary>, String> {
        let limit = limit.max(1);
        let summaries: Vec<PullRequestSummary> = Self::query(
            root,
            &[
                "pr",
                "list",
                "--state",
                "all",
                "--limit",
                &limit.to_string(),
                "--json",
                "number,title,state,baseRefName,headRefName,updatedAt",
            ],
            control,
        )?;
        if summaries.len() > limit {
            return Err("GitHub returned more pull requests than requested".into());
        }
        let mut seen = std::collections::HashSet::new();
        for summary in &summaries {
            control.check()?;
            if summary.number == 0 || !seen.insert(summary.number) {
                return Err("GitHub returned invalid or duplicate pull request numbers".into());
            }
        }
        Ok(summaries)
    }

    /// Fetches details for a specific pull request number.
    ///
    /// # Errors
    ///
    /// Returns an error message string if `gh pr view` fails or cannot find the pull request.
    pub fn view_pull_request(root: &Path, number: usize) -> Result<PullRequestSummary, String> {
        Self::view_cancellable(root, number, &ReviewControl::new(&AtomicBool::new(false)))
    }

    pub(crate) fn view_cancellable(
        root: &Path,
        number: usize,
        control: &ReviewControl<'_>,
    ) -> Result<PullRequestSummary, String> {
        Self::query(
            root,
            &[
                "pr",
                "view",
                &number.to_string(),
                "--json",
                "number,title,state,baseRefName,baseRefOid,headRefName,headRefOid,updatedAt",
            ],
            control,
        )
    }

    /// Fetches the unified diff text for a given pull request number.
    ///
    /// # Errors
    ///
    /// Returns an error message string if `gh pr diff` fails.
    pub fn get_pr_diff(root: &Path, number: usize) -> Result<String, String> {
        String::from_utf8(Self::request(
            root,
            &["pr", "diff", &number.to_string()],
            &ReviewControl::new(&AtomicBool::new(false)),
        )?)
        .map_err(|error| format!("Invalid UTF-8 diff output: {error}"))
    }
    fn query<T: serde::de::DeserializeOwned>(
        root: &Path,
        arguments: &[&str],
        control: &ReviewControl<'_>,
    ) -> Result<T, String> {
        let value = serde_json::from_slice(&Self::request(root, arguments, control)?)
            .map_err(|error| format!("Invalid GitHub JSON: {error}"))?;
        control.check()?;
        Ok(value)
    }

    fn request(
        root: &Path,
        arguments: &[&str],
        control: &ReviewControl<'_>,
    ) -> Result<Vec<u8>, String> {
        control.check()?;
        let command = ProcessCommand {
            executable: control.gh.into(),
            directory: root.into(),
            arguments: arguments.iter().map(Into::into).collect(),
            environment: [
                ("GH_PROMPT_DISABLED".into(), "1".into()),
                ("GIT_TERMINAL_PROMPT".into(), "0".into()),
            ]
            .into(),
            resources: vec![],
        };
        let (output, bytes) = process::run_collect(
            &command,
            control.cancel,
            control.deadline.saturating_duration_since(Instant::now()),
            16 * 1024 * 1024,
        )
        .map_err(|error| match error.kind {
            process::RunFailureKind::Spawn {
                kind: std::io::ErrorKind::NotFound,
                ..
            } => "GitHub CLI (gh) was not found. Install it and run gh auth login.".into(),
            _ => format!("GitHub CLI: {error}"),
        })?;
        if !output.succeeded() {
            return Err(format!("GitHub CLI failed: {}", output.stderr.text.trim()));
        }
        control.check()?;
        Ok(bytes)
    }
}
