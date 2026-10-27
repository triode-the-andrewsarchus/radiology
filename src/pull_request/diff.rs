//! Unified diff hunk parsing and line-level addition/deletion tracking.

use crate::source::location::ReviewRevision;
use std::ops::Range;

/// Represents a single hunk parsed from unified diff output (`git diff -U0`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffHunk {
    /// 1-based start line in the Before revision.
    pub before_start: usize,
    /// Number of lines deleted from the Before revision.
    pub before_count: usize,
    /// 1-based start line in the After revision.
    pub after_start: usize,
    /// Number of lines added to the After revision.
    pub after_count: usize,
}

impl DiffHunk {
    /// Returns the 1-based half-open line range removed from the Before revision.
    #[must_use]
    pub fn removed_lines(&self) -> Range<usize> {
        self.before_start..(self.before_start + self.before_count)
    }

    /// Returns the 1-based half-open line range added to the After revision.
    #[must_use]
    pub fn added_lines(&self) -> Range<usize> {
        self.after_start..(self.after_start + self.after_count)
    }
}

/// Stores all parsed diff hunks for a single document.
///
/// # Examples
///
/// ```rust
/// use radiology::pull_request::diff::UnifiedDiffParser;
/// use radiology::source::location::ReviewRevision;
///
/// let diff_text = "@@ -10,2 +10,4 @@\n-old\n+new1\n+new2\n";
/// let changes = UnifiedDiffParser::parse(diff_text);
///
/// assert_eq!(changes.total_additions(), 4);
/// assert_eq!(changes.total_deletions(), 2);
/// assert_eq!(changes.changed_lines(ReviewRevision::After), vec![10..14]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileLineChanges {
    /// Hunks defining changes between Before and After revisions.
    pub hunks: Vec<DiffHunk>,
    /// Whether the file diff was detected as binary.
    pub is_binary: bool,
}

impl FileLineChanges {
    /// Returns 1-based line ranges corresponding to changed lines in `revision`.
    #[must_use]
    pub fn changed_lines(&self, revision: ReviewRevision) -> Vec<Range<usize>> {
        self.hunks
            .iter()
            .map(|h| match revision {
                ReviewRevision::After => h.added_lines(),
                ReviewRevision::Before => h.removed_lines(),
            })
            .filter(|r| !r.is_empty())
            .collect()
    }

    /// Returns anchor line numbers suitable for jumping directly between change blocks.
    #[must_use]
    pub fn change_anchors(&self, revision: ReviewRevision) -> Vec<usize> {
        self.hunks
            .iter()
            .map(|h| match revision {
                ReviewRevision::After => {
                    if h.after_count > 0 {
                        h.after_start
                    } else {
                        (h.after_start + 1).max(1)
                    }
                }
                ReviewRevision::Before => {
                    if h.before_count > 0 {
                        h.before_start
                    } else {
                        (h.before_start + 1).max(1)
                    }
                }
            })
            .collect()
    }

    /// Returns the total number of added lines across all hunks.
    #[must_use]
    pub fn total_additions(&self) -> usize {
        self.hunks.iter().map(|h| h.after_count).sum()
    }

    /// Returns the total number of deleted lines across all hunks.
    #[must_use]
    pub fn total_deletions(&self) -> usize {
        self.hunks.iter().map(|h| h.before_count).sum()
    }
}

/// Parses unified diff outputs into structured [`FileLineChanges`].
pub struct UnifiedDiffParser;

impl UnifiedDiffParser {
    /// Parses the raw text output of `git diff -U0` into a [`FileLineChanges`] instance.
    #[must_use]
    pub fn parse(text: &str) -> FileLineChanges {
        if text.contains("Binary files") || text.contains("GIT binary patch") {
            return FileLineChanges {
                hunks: Vec::new(),
                is_binary: true,
            };
        }

        let mut hunks = Vec::new();

        for line in text.lines() {
            if line.starts_with("@@ ") {
                if let Some(hunk) = Self::parse_hunk_header(line) {
                    hunks.push(hunk);
                }
            }
        }

        FileLineChanges {
            hunks,
            is_binary: false,
        }
    }

    fn parse_hunk_header(line: &str) -> Option<DiffHunk> {
        let mut parts = line.split_whitespace();
        let _at = parts.next()?;
        let before_part = parts.next()?;
        let after_part = parts.next()?;

        if !before_part.starts_with('-') || !after_part.starts_with('+') {
            return None;
        }

        let (before_start, before_count) = Self::parse_range(&before_part[1..])?;
        let (after_start, after_count) = Self::parse_range(&after_part[1..])?;

        Some(DiffHunk {
            before_start,
            before_count,
            after_start,
            after_count,
        })
    }

    fn parse_range(s: &str) -> Option<(usize, usize)> {
        if let Some((start_str, count_str)) = s.split_once(',') {
            let start = start_str.parse().ok()?;
            let count = count_str.parse().ok()?;
            Some((start, count))
        } else {
            let start = s.parse().ok()?;
            Some((start, 1))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_unified_diff() {
        let diff = r#"
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,3 +10,5 @@
 println!("hello");
-foo();
+bar();
+baz();
+qux();
"#;
        let changes = UnifiedDiffParser::parse(diff);
        assert_eq!(changes.hunks.len(), 1);
        let h = &changes.hunks[0];
        assert_eq!(h.before_start, 10);
        assert_eq!(h.before_count, 3);
        assert_eq!(h.after_start, 10);
        assert_eq!(h.after_count, 5);

        let after_ranges = changes.changed_lines(ReviewRevision::After);
        assert_eq!(after_ranges, vec![10..15]);
    }
}
