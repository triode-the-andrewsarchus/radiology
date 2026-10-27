//! Pull request review, GitHub CLI integration, and unified diff parsing.
//!
//! Provides [`GitHubClient`] for read-only pull request inspection via the `gh` CLI,
//! [`UnifiedDiffParser`] for parsing standard unified diffs (`git diff -U0`), and
//! [`FileChange`] for tracking file status, additions, and deletions.

pub mod change;
pub mod diff;
pub mod github;

pub use change::{FileChange, FileChangeStatus};
pub use diff::{DiffHunk, FileLineChanges, UnifiedDiffParser};
pub use github::{GitHubClient, PullRequestSummary};
