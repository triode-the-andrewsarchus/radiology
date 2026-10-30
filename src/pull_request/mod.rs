//! Pull request review, GitHub CLI integration, and unified diff parsing.
//!
//! Provides [`GitHubClient`] for read-only pull request inspection via the `gh` CLI,
//! [`UnifiedDiffParser`] for parsing standard unified diffs (`git diff -U0`), and
//! [`FileChange`] for tracking file status, additions, and deletions.

pub mod change;
pub(crate) mod control;
pub mod diff;
pub mod github;
pub mod recent;
pub mod review;

pub use change::{FileChange, FileChangeStatus, ReviewChangesPresentation};
pub use diff::{DiffHunk, FileLineChanges, UnifiedDiffParser};
pub use github::{GitHubClient, PullRequestSummary};

#[cfg(all(test, unix))]
pub(crate) mod test_support;
