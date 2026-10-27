use serde::{Deserialize, Serialize};
use crate::source::location::ReviewRevision;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileChangeStatus {
    Added,
    Modified,
    Deleted,
    Renamed { similarity: usize },
    Copied,
    Other(String),
}

impl FileChangeStatus {
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

/// A file touched by a pull request or git diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub old_path: Option<String>,
    pub status: FileChangeStatus,
    pub before_blob: String,
    pub after_blob: String,
    pub additions: usize,
    pub deletions: usize,
    pub is_binary: bool,
}

impl FileChange {
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
