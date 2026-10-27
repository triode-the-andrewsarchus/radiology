//! Pull request file change models and revision path resolution.

use crate::source::location::ReviewRevision;
use serde::{Deserialize, Serialize};

/// Status of a file modified within a pull request or commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileChangeStatus {
    /// Newly added file.
    Added,
    /// Modified file.
    Modified,
    /// Deleted file.
    Deleted,
    /// Renamed file with similarity percentage.
    Renamed {
        /// Similarity index from 0 to 100.
        similarity: usize,
    },
    /// Copied file.
    Copied,
    /// Other change type or typechange.
    Other(String),
}

impl FileChangeStatus {
    /// Returns the human-readable description of this change status.
    #[must_use]
    pub fn title(&self) -> &'static str {
        match self {
            Self::Added => "Added",
            Self::Modified => "Modified",
            Self::Deleted => "Deleted",
            Self::Renamed { similarity } => {
                if *similarity >= 100 {
                    "Renamed"
                } else {
                    "Renamed & Modified"
                }
            }
            Self::Copied => "Copied",
            Self::Other(_) => "Changed",
        }
    }

    /// Returns the single-letter badge abbreviation (`A`, `M`, `D`, `R`, etc.).
    #[must_use]
    pub fn badge_letter(&self) -> &'static str {
        match self {
            Self::Added => "A",
            Self::Modified => "M",
            Self::Deleted => "D",
            Self::Renamed { .. } => "R",
            Self::Copied => "C",
            Self::Other(_) => "?",
        }
    }
}

/// Represents a single file touched by a pull request.
///
/// # Examples
///
/// ```rust
/// use radiology::pull_request::change::{FileChange, FileChangeStatus};
/// use radiology::source::location::ReviewRevision;
///
/// let change = FileChange {
///     path: "src/new.rs".to_string(),
///     old_path: None,
///     status: FileChangeStatus::Added,
///     before_blob: String::new(),
///     after_blob: "abcdef".to_string(),
///     additions: 15,
///     deletions: 0,
///     is_binary: false,
/// };
/// assert_eq!(change.path_for_revision(ReviewRevision::After), Some("src/new.rs"));
/// assert_eq!(change.path_for_revision(ReviewRevision::Before), None);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileChange {
    /// Repository-relative path in the head revision.
    pub path: String,
    /// Path in the merge base if renamed or copied.
    pub old_path: Option<String>,
    /// Status classification of this change.
    pub status: FileChangeStatus,
    /// Git object blob ID in the merge base revision.
    pub before_blob: String,
    /// Git object blob ID in the head revision.
    pub after_blob: String,
    /// Number of lines added.
    pub additions: usize,
    /// Number of lines deleted.
    pub deletions: usize,
    /// Whether this file is binary.
    pub is_binary: bool,
}

impl FileChange {
    /// Returns the appropriate repository-relative path to read for the given [`ReviewRevision`].
    #[must_use]
    pub fn path_for_revision(&self, revision: ReviewRevision) -> Option<&str> {
        match revision {
            ReviewRevision::After => {
                if self.status != FileChangeStatus::Deleted {
                    Some(&self.path)
                } else {
                    None
                }
            }
            ReviewRevision::Before => {
                if self.status != FileChangeStatus::Added {
                    Some(self.old_path.as_deref().unwrap_or(&self.path))
                } else {
                    None
                }
            }
        }
    }
}
