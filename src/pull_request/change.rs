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

/// Filtered presentation of pull request file changes honoring active source filters.
///
/// # Examples
///
/// ```rust
/// use radiology::pull_request::change::{FileChange, FileChangeStatus, ReviewChangesPresentation};
/// use radiology::repository::filters::SourceFilters;
///
/// let changes = vec![
///     FileChange {
///         path: "src/main.rs".to_string(),
///         old_path: None,
///         status: FileChangeStatus::Modified,
///         before_blob: String::new(),
///         after_blob: String::new(),
///         additions: 10,
///         deletions: 2,
///         is_binary: false,
///     },
///     FileChange {
///         path: "README.md".to_string(),
///         old_path: None,
///         status: FileChangeStatus::Modified,
///         before_blob: String::new(),
///         after_blob: String::new(),
///         additions: 5,
///         deletions: 0,
///         is_binary: false,
///     },
/// ];
///
/// let mut filters = SourceFilters::default();
/// filters.source_files_only = true;
/// let presentation = ReviewChangesPresentation::new(&changes, &filters);
/// assert_eq!(presentation.changes.len(), 1);
/// assert_eq!(presentation.changes[0].path, "src/main.rs");
/// assert_eq!(presentation.file_count_label(), "1 of 2 files");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewChangesPresentation {
    /// Total unfiltered change count.
    pub total_count: usize,
    /// Filtered list of visible file changes.
    pub changes: Vec<FileChange>,
}

impl ReviewChangesPresentation {
    /// Creates a new presentation filtering changes according to `filters`.
    #[must_use]
    pub fn new(
        changes: &[FileChange],
        filters: &crate::repository::filters::SourceFilters,
    ) -> Self {
        let total_count = changes.len();
        let filtered = changes
            .iter()
            .filter(|c| {
                let matches_after = c
                    .path_for_revision(ReviewRevision::After)
                    .map(|p| filters.includes_path(p))
                    .unwrap_or(false);
                let matches_before = c
                    .path_for_revision(ReviewRevision::Before)
                    .map(|p| filters.includes_path(p))
                    .unwrap_or(false);
                matches_after || matches_before
            })
            .cloned()
            .collect();

        Self {
            total_count,
            changes: filtered,
        }
    }

    /// Returns `true` if there are genuinely zero changed files in the pull request.
    #[must_use]
    pub fn is_genuinely_empty(&self) -> bool {
        self.total_count == 0
    }

    /// Returns `true` if the active filters hid all changed files.
    #[must_use]
    pub fn is_filtered_empty(&self) -> bool {
        self.total_count > 0 && self.changes.is_empty()
    }

    /// Returns the sum of added lines across visible changes.
    #[must_use]
    pub fn additions(&self) -> usize {
        self.changes.iter().map(|c| c.additions).sum()
    }

    /// Returns the sum of deleted lines across visible changes.
    #[must_use]
    pub fn deletions(&self) -> usize {
        self.changes.iter().map(|c| c.deletions).sum()
    }

    /// Returns the user-facing count label (e.g. `12 of 15 files`).
    #[must_use]
    pub fn file_count_label(&self) -> String {
        format!("{} of {} files", self.changes.len(), self.total_count)
    }

    /// Finds the file change path matching `path` in `revision`.
    #[must_use]
    pub fn selection_id(&self, path: &str, revision: ReviewRevision) -> Option<&str> {
        self.changes
            .iter()
            .find(|c| c.path_for_revision(revision) == Some(path))
            .map(|c| c.path.as_str())
    }
}
