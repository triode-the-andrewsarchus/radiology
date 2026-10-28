//! GitHub CLI (`gh`) integration for pull request summaries and diff retrieval.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

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
pub struct GitHubClient;

impl GitHubClient {
    /// Checks whether the `gh` executable is available in `PATH`.
    #[must_use]
    pub fn is_gh_available() -> bool {
        Command::new("gh")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Lists up to `limit` recent pull requests in the repository at `root`.
    ///
    /// # Errors
    ///
    /// Returns an error message string if `gh` execution fails or returns invalid JSON.
    pub fn list_pull_requests(
        root: &Path,
        limit: usize,
    ) -> Result<Vec<PullRequestSummary>, String> {
        let output = Command::new("gh")
            .args([
                "pr",
                "list",
                "--state",
                "all",
                "--limit",
                &limit.to_string(),
                "--json",
                "number,title,state,baseRefName,headRefName,updatedAt",
            ])
            .current_dir(root)
            .output()
            .map_err(|e| format!("Failed to run gh CLI: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(stderr.trim().to_string());
        }

        serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("Failed to parse PR list JSON: {e}"))
    }

    /// Fetches details for a specific pull request number.
    ///
    /// # Errors
    ///
    /// Returns an error message string if `gh pr view` fails or cannot find the pull request.
    pub fn view_pull_request(root: &Path, number: usize) -> Result<PullRequestSummary, String> {
        let output = Command::new("gh")
            .args([
                "pr",
                "view",
                &number.to_string(),
                "--json",
                "number,title,state,baseRefName,baseRefOid,headRefName,headRefOid,updatedAt",
            ])
            .current_dir(root)
            .output()
            .map_err(|e| format!("Failed to run gh CLI: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(stderr.trim().to_string());
        }

        serde_json::from_slice(&output.stdout).map_err(|e| format!("Failed to parse PR JSON: {e}"))
    }

    /// Fetches the unified diff text for a given pull request number.
    ///
    /// # Errors
    ///
    /// Returns an error message string if `gh pr diff` fails.
    pub fn get_pr_diff(root: &Path, number: usize) -> Result<String, String> {
        let output = Command::new("gh")
            .args(["pr", "diff", &number.to_string()])
            .current_dir(root)
            .output()
            .map_err(|e| format!("Failed to run gh CLI: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(stderr.trim().to_string());
        }

        String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8 diff output: {e}"))
    }
}
