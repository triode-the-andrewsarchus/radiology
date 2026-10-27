use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullRequestSummary {
    pub number: usize,
    pub title: String,
    pub state: String,
    #[serde(rename = "baseRefName", default)]
    pub base_ref_name: String,
    #[serde(rename = "headRefName", default)]
    pub head_ref_name: String,
    #[serde(rename = "headRefOid", default)]
    pub head_ref_oid: Option<String>,
    #[serde(rename = "updatedAt", default)]
    pub updated_at: String,
}

impl PullRequestSummary {
    pub fn is_open(&self) -> bool {
        self.state.eq_ignore_ascii_case("OPEN")
    }

    pub fn is_merged(&self) -> bool {
        self.state.eq_ignore_ascii_case("MERGED")
    }
}

pub struct GitHubClient;

impl GitHubClient {
    pub fn is_gh_available() -> bool {
        Command::new("gh")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn list_pull_requests(root: &Path, limit: usize) -> Result<Vec<PullRequestSummary>, String> {
        let output = Command::new("gh")
            .args([
                "pr", "list",
                "--limit", &limit.to_string(),
                "--json", "number,title,state,baseRefName,headRefName,updatedAt",
            ])
            .current_dir(root)
            .output()
            .map_err(|e| format!("Failed to run gh CLI: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(stderr.trim().to_string());
        }

        serde_json::from_slice(&output.stdout).map_err(|e| format!("Failed to parse PR list JSON: {e}"))
    }

    pub fn view_pull_request(root: &Path, number: usize) -> Result<PullRequestSummary, String> {
        let output = Command::new("gh")
            .args([
                "pr", "view", &number.to_string(),
                "--json", "number,title,state,baseRefName,headRefName,headRefOid,updatedAt",
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
