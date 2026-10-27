//! Source location coordinates and review revisions.

use serde::{Deserialize, Serialize};

/// Identifies whether a file location belongs to the merge base (`Before`)
/// or the pull request head (`After`) during review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReviewRevision {
    /// Merge base revision representing the file prior to PR changes.
    Before,
    /// Head revision of the pull request under review.
    After,
}

impl ReviewRevision {
    /// Returns the human-readable display title for this revision.
    #[must_use]
    pub fn title(&self) -> &'static str {
        match self {
            Self::Before => "Before",
            Self::After => "After",
        }
    }
}

/// Represents a precise byte-level coordinate within a repository or PR revision.
///
/// Byte offsets are canonical across all subsystems. The optional [`ReviewRevision`]
/// distinguishes between Before and After versions during pull request inspection.
///
/// # Examples
///
/// ```rust
/// use radiology::source::location::{ReviewRevision, SourceLocation};
///
/// let loc = SourceLocation::new("src/main.rs", 42);
/// assert_eq!(loc.position_key(), "src/main.rs");
///
/// let pr_loc = SourceLocation::with_revision("src/main.rs", 42, Some(ReviewRevision::Before));
/// assert_eq!(pr_loc.position_key(), "before:src/main.rs");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Repository-relative file path.
    pub path: String,
    /// Zero-based byte offset within the file's UTF-8 text.
    pub byte_offset: usize,
    /// Pull request review revision, or `None` for standard repository browsing.
    pub revision: Option<ReviewRevision>,
}

impl SourceLocation {
    /// Creates a new [`SourceLocation`] with no review revision.
    pub fn new(path: impl Into<String>, byte_offset: usize) -> Self {
        Self {
            path: path.into(),
            byte_offset,
            revision: None,
        }
    }

    /// Creates a new [`SourceLocation`] with an explicit review revision.
    pub fn with_revision(
        path: impl Into<String>,
        byte_offset: usize,
        revision: Option<ReviewRevision>,
    ) -> Self {
        Self {
            path: path.into(),
            byte_offset,
            revision,
        }
    }

    /// Returns the cache key used for storing per-document restore positions.
    #[must_use]
    pub fn position_key(&self) -> String {
        Self::format_position_key(&self.path, self.revision)
    }

    /// Formats a path and optional revision into a canonical position key string.
    #[must_use]
    pub fn format_position_key(path: &str, revision: Option<ReviewRevision>) -> String {
        match revision {
            Some(ReviewRevision::Before) => format!("before:{path}"),
            Some(ReviewRevision::After) => format!("after:{path}"),
            None => path.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_key() {
        let loc1 = SourceLocation::new("src/main.rs", 42);
        assert_eq!(loc1.position_key(), "src/main.rs");

        let loc2 = SourceLocation::with_revision("src/main.rs", 100, Some(ReviewRevision::Before));
        assert_eq!(loc2.position_key(), "before:src/main.rs");

        let loc3 = SourceLocation::with_revision("src/main.rs", 100, Some(ReviewRevision::After));
        assert_eq!(loc3.position_key(), "after:src/main.rs");
    }
}
