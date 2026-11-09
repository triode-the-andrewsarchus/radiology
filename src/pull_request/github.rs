//! GitHub CLI (`gh`) integration for pull request summaries and diff retrieval.

use super::control::ReviewControl;
use crate::process::{self, ProcessCommand};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

const SUMMARY_FIELDS: &str = "number,title,state,baseRefName,baseRefOid,headRefName,headRefOid,isCrossRepository,updatedAt,mergeCommit,author";

/// GitHub account associated with a pull-request summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullRequestAuthor {
    /// GitHub account login, including bot account suffixes when present.
    pub login: String,
}

/// Merge-commit metadata returned by GitHub.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullRequestMergeCommit {
    /// Object ID reported for the merge commit.
    pub oid: String,
}

/// Summary metadata for a single GitHub pull request.
///
/// Missing author/merge metadata reads as `None`; an omitted cross-repository
/// flag reads as `false`, preserving older serialized summaries.
///
/// # Examples
///
/// ```
/// use radiology::pull_request::PullRequestSummary;
/// let summary: PullRequestSummary = serde_json::from_str(
///     r#"{"number":7,"title":"Review","state":"OPEN","author":{"login":"reviewer"},"isCrossRepository":true}"#,
/// )?;
/// assert_eq!(summary.author.as_ref().map(|author| author.login.as_str()), Some("reviewer"));
/// assert!(summary.is_cross_repository);
/// assert!(summary.merge_commit.is_none());
/// # Ok::<(), serde_json::Error>(())
/// ```
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
    /// Whether the pull-request branch belongs to another repository.
    #[serde(rename = "isCrossRepository", default)]
    pub is_cross_repository: bool,
    /// Merge commit, when GitHub reports one.
    #[serde(rename = "mergeCommit", default)]
    pub merge_commit: Option<PullRequestMergeCommit>,
    /// Author account, when GitHub reports one.
    #[serde(default)]
    pub author: Option<PullRequestAuthor>,
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
                SUMMARY_FIELDS,
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
            &["pr", "view", &number.to_string(), "--json", SUMMARY_FIELDS],
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

#[cfg(test)]
mod tests {
    use super::*;

    const SUMMARIES: &str = r#"[
        {"number":724,"title":"Own desktop runtime","state":"OPEN","baseRefName":"main","baseRefOid":"488d8768d55a4e88a772c673bada7d89168aed0f","headRefName":"codex/x","headRefOid":"c9a5b0502e8cad8f06440466c8f2124422ee6a23","isCrossRepository":false,"mergeCommit":null,"updatedAt":"2026-10-04T22:11:41Z","author":{"id":"opaque","is_bot":false,"login":"dadrian","name":"D"}},
        {"number":723,"title":"Build","state":"MERGED","baseRefName":"main","baseRefOid":"8368298660b3a5a15398a88767775a610779b90f","headRefName":"b","headRefOid":"f4082f9e3ab0f5b363ff5300acd7bf8ccf78a19e","isCrossRepository":true,"mergeCommit":{"oid":"6af21769e85adea14949be3105af4a65c43ad8b2"},"updatedAt":"2026-10-04T21:32:49Z","author":null}
    ]"#;

    #[test]
    fn summary_metadata_roundtrips_open_merged_and_legacy_records() {
        let summaries: Vec<PullRequestSummary> = serde_json::from_str(SUMMARIES).unwrap();
        assert_eq!(
            summaries.iter().map(|s| s.number).collect::<Vec<_>>(),
            [724, 723]
        );
        assert_eq!(summaries[0].state_title(), "Open");
        assert!(summaries[0].is_open() && summaries[1].is_merged());
        assert!(crate::pull_request::change::is_object_id(
            summaries[0].head_ref_oid.as_deref().unwrap()
        ));
        assert!(!crate::pull_request::change::is_object_id("c9a5b05"));
        let encoded = serde_json::to_value(&summaries).unwrap();
        assert_eq!(encoded[0]["author"]["login"], "dadrian");
        assert_eq!(encoded[0]["isCrossRepository"], false);
        assert!(encoded[0]["mergeCommit"].is_null());
        assert_eq!(encoded[1]["isCrossRepository"], true);
        assert_eq!(
            encoded[1]["mergeCommit"]["oid"],
            "6af21769e85adea14949be3105af4a65c43ad8b2"
        );
        assert!(encoded[1]["author"].is_null());
        assert_eq!(summaries[0].updated_at, "2026-10-04T22:11:41Z");
        assert_eq!(summaries[0].author.as_ref().unwrap().login, "dadrian");
        assert!(summaries[0].merge_commit.is_none() && !summaries[0].is_cross_repository);
        assert!(summaries[1].author.is_none() && summaries[1].is_cross_repository);
        for (field, malformed) in [
            ("author", serde_json::json!({"login":42})),
            ("author", serde_json::json!({})),
            ("mergeCommit", serde_json::json!({"oid":42})),
            ("mergeCommit", serde_json::json!({})),
            ("isCrossRepository", serde_json::json!("true")),
            ("isCrossRepository", serde_json::Value::Null),
        ] {
            let mut invalid = encoded[0].clone();
            invalid[field] = malformed;
            assert!(
                serde_json::from_value::<PullRequestSummary>(invalid).is_err(),
                "{field}"
            );
        }
        let decoded: Vec<PullRequestSummary> = serde_json::from_value(encoded).unwrap();
        assert_eq!(decoded, summaries);
        let legacy: PullRequestSummary =
            serde_json::from_str(r#"{"number":7,"title":"Legacy","state":"OPEN"}"#).unwrap();
        let encoded = serde_json::to_value(&legacy).unwrap();
        assert_eq!(encoded["isCrossRepository"], false);
        assert!(encoded["mergeCommit"].is_null() && encoded["author"].is_null());
    }
    #[cfg(unix)]
    #[test]
    fn list_and_view_request_complete_metadata_through_the_shared_child_runner() {
        use crate::repository::test_support::fake_git;
        use std::collections::BTreeSet;
        let owner = tempfile::tempdir().unwrap();
        let root = owner.path();
        std::fs::write(root.join("summary-list.json"), SUMMARIES).unwrap();
        let expected: Vec<PullRequestSummary> = serde_json::from_str(SUMMARIES).unwrap();
        for summary in &expected {
            std::fs::write(
                root.join(format!("view-{}.json", summary.number)),
                serde_json::to_vec(summary).unwrap(),
            )
            .unwrap();
        }
        let gh = fake_git(
            root,
            r#"test "$GH_PROMPT_DISABLED" = 1 || exit 8
printf '%s\n' "$@" > arguments
case "$2" in
 list) cat summary-list.json ;;
 view) cat "view-$3.json" ;;
 *) exit 9 ;;
esac"#,
        );
        let cancel = AtomicBool::new(false);
        let control = ReviewControl {
            gh: &gh,
            ..ReviewControl::new(&cancel)
        };
        for number in [None, Some(724), Some(723)] {
            let received = match number {
                None => GitHubClient::list_cancellable(root, 2, &control).unwrap(),
                Some(number) => {
                    vec![GitHubClient::view_cancellable(root, number, &control).unwrap()]
                }
            };
            let wanted: Vec<_> = expected
                .iter()
                .filter(|s| number.is_none_or(|n| n == s.number))
                .cloned()
                .collect();
            assert_eq!(received, wanted);
            let arguments = std::fs::read_to_string(root.join("arguments")).unwrap();
            let args: Vec<_> = arguments.lines().collect();
            assert_eq!(args[0], "pr");
            assert_eq!(args[1], if number.is_some() { "view" } else { "list" });
            if let Some(number) = number {
                assert_eq!(args[2], number.to_string());
            }
            let fields = args.iter().position(|arg| *arg == "--json").unwrap();
            assert_eq!(
                args[fields + 1].split(',').collect::<BTreeSet<_>>(),
                [
                    "number",
                    "title",
                    "state",
                    "baseRefName",
                    "baseRefOid",
                    "headRefName",
                    "headRefOid",
                    "isCrossRepository",
                    "updatedAt",
                    "mergeCommit",
                    "author"
                ]
                .into_iter()
                .collect()
            );
        }
    }
}
