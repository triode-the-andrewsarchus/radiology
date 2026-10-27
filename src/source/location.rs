use serde::{Deserialize, Serialize};

/// Identifies whether a file location belongs to the merge base (`Before`)
/// or the pull request head (`After`) during PR review, or nil for standard repository checkout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReviewRevision {
    Before,
    After,
}

impl ReviewRevision {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Before => "Before",
            Self::After => "After",
        }
    }
}

/// A precise byte-level coordinate within a repository or PR revision.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceLocation {
    pub path: String,
    pub byte_offset: usize,
    pub revision: Option<ReviewRevision>,
}

impl SourceLocation {
    pub fn new(path: impl Into<String>, byte_offset: usize) -> Self {
        Self {
            path: path.into(),
            byte_offset,
            revision: None,
        }
    }

    pub fn with_revision(path: impl Into<String>, byte_offset: usize, revision: Option<ReviewRevision>) -> Self {
        Self {
            path: path.into(),
            byte_offset,
            revision,
        }
    }

    /// Key used for caching per-file restore positions and bookmarks.
    pub fn position_key(&self) -> String {
        Self::format_position_key(&self.path, self.revision)
    }

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
