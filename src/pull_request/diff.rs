//! Unified diff hunk parsing and line-level addition/deletion tracking.

use crate::pull_request::change::{FileChange, FileChangeStatus};
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

/// Available previous and next change targets for one canonical caret position.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ChangeNavigation {
    /// Previous changed block, or None at the beginning.
    pub previous: Option<usize>,
    /// Next changed block, or None at the end.
    pub next: Option<usize>,
}

impl FileLineChanges {
    /// Returns the same bounded targets used by controls and command dispatch.
    #[must_use]
    pub fn navigation_at(&self, line: usize, revision: ReviewRevision) -> ChangeNavigation {
        ChangeNavigation {
            previous: self.previous_change_line(line, revision),
            next: self.next_change_line(line, revision),
        }
    }

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
        self.navigation_blocks(revision)
            .into_iter()
            .map(|range| range.start)
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

    /// Returns the line number of the previous changed hunk block before `current_line`.
    #[must_use]
    pub fn previous_change_line(
        &self,
        current_line: usize,
        revision: ReviewRevision,
    ) -> Option<usize> {
        self.navigation_blocks(revision)
            .into_iter()
            .take_while(|range| range.end <= current_line)
            .last()
            .map(|range| range.start)
    }

    /// Returns the line number of the next changed hunk block after `current_line`.
    #[must_use]
    pub fn next_change_line(&self, current_line: usize, revision: ReviewRevision) -> Option<usize> {
        self.change_anchors(revision)
            .into_iter()
            .find(|&line| line > current_line)
    }

    fn navigation_blocks(&self, revision: ReviewRevision) -> Vec<Range<usize>> {
        self.hunks
            .iter()
            .map(|hunk| {
                let range = match revision {
                    ReviewRevision::Before => hunk.removed_lines(),
                    ReviewRevision::After => hunk.added_lines(),
                };
                if range.is_empty() {
                    let anchor = (range.start + 1).max(1);
                    anchor..anchor + 1
                } else {
                    range
                }
            })
            .collect()
    }

    /// Maps a canonical line into the other revision using exact zero-context hunks.
    /// Lines replaced or deleted map to the nearest surviving destination line.
    #[must_use]
    pub fn map_line(&self, line: usize, from: ReviewRevision) -> usize {
        let mut delta = 0_isize;
        for hunk in &self.hunks {
            let (source, target) = match from {
                ReviewRevision::Before => (hunk.removed_lines(), hunk.added_lines()),
                ReviewRevision::After => (hunk.added_lines(), hunk.removed_lines()),
            };
            // Git places an empty side after its anchor line. That preceding
            // line survives unchanged; only following lines receive this delta.
            if line < source.start || (source.is_empty() && line == source.start) {
                break;
            }
            if source.contains(&line) {
                return (target.start + (line - source.start).min(target.len().saturating_sub(1)))
                    .max(1);
            }
            delta += target.len() as isize - source.len() as isize;
        }
        line.saturating_add_signed(delta).max(1)
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

    /// Parses a complete multi-file unified diff into a list of [`FileChange`] items.
    #[must_use]
    pub fn parse_file_changes(text: &str) -> Vec<FileChange> {
        let mut changes = Vec::new();
        let mut lines = text.lines().peekable();

        while let Some(line) = lines.next() {
            if !line.starts_with("diff --git ") {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            let mut old_path = parts
                .get(2)
                .and_then(|p| p.strip_prefix("a/"))
                .map(|p| p.to_string());
            let mut new_path = parts
                .get(3)
                .and_then(|p| p.strip_prefix("b/"))
                .map(|p| p.to_string());

            let mut status = FileChangeStatus::Modified;
            let mut is_binary = false;
            let mut additions = 0;
            let mut deletions = 0;

            while let Some(&sub_line) = lines.peek() {
                if sub_line.starts_with("diff --git ") {
                    break;
                }
                lines.next();

                if sub_line.starts_with("new file mode") {
                    status = FileChangeStatus::Added;
                } else if sub_line.starts_with("deleted file mode") {
                    status = FileChangeStatus::Deleted;
                } else if let Some(from) = sub_line.strip_prefix("rename from ") {
                    old_path = Some(from.to_string());
                    let sim = if let FileChangeStatus::Renamed { similarity } = status {
                        similarity
                    } else {
                        100
                    };
                    status = FileChangeStatus::Renamed { similarity: sim };
                } else if let Some(to) = sub_line.strip_prefix("rename to ") {
                    new_path = Some(to.to_string());
                } else if let Some(score_str) = sub_line.strip_prefix("similarity index ") {
                    if let Some(num_str) = score_str.strip_suffix('%') {
                        if let Ok(sim) = num_str.parse::<usize>() {
                            status = FileChangeStatus::Renamed { similarity: sim };
                        }
                    }
                } else if sub_line.starts_with("Binary files")
                    || sub_line.starts_with("GIT binary patch")
                {
                    is_binary = true;
                } else if sub_line.starts_with('+') && !sub_line.starts_with("+++") {
                    additions += 1;
                } else if sub_line.starts_with('-') && !sub_line.starts_with("---") {
                    deletions += 1;
                }
            }

            let final_path = if status == FileChangeStatus::Deleted {
                old_path.clone().unwrap_or_default()
            } else {
                new_path
                    .clone()
                    .unwrap_or_else(|| old_path.clone().unwrap_or_default())
            };

            if !final_path.is_empty() {
                changes.push(FileChange {
                    path: final_path,
                    old_path: if status == FileChangeStatus::Modified
                        || status == FileChangeStatus::Added
                    {
                        None
                    } else {
                        old_path
                    },
                    status,
                    before_blob: String::new(),
                    after_blob: String::new(),
                    additions,
                    deletions,
                    is_binary,
                });
            }
        }

        changes
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
    fn zero_length_hunks_preserve_anchor_lines_and_accumulate_prior_offsets() {
        for (diff, from, cases) in [
            ("@@ -2,2 +1,0 @@", ReviewRevision::After, [(1, 1), (2, 4)]),
            ("@@ -2,2 +1,0 @@", ReviewRevision::Before, [(1, 1), (4, 2)]),
            ("@@ -1,0 +2,2 @@", ReviewRevision::Before, [(1, 1), (2, 4)]),
            ("@@ -1,0 +2,2 @@", ReviewRevision::After, [(1, 1), (4, 2)]),
            ("@@ -0,0 +1,2 @@", ReviewRevision::Before, [(1, 3), (2, 4)]),
            (
                "@@ -1,0 +2,2 @@\n@@ -4,0 +7,1 @@",
                ReviewRevision::Before,
                [(4, 6), (5, 8)],
            ),
        ] {
            let changes = UnifiedDiffParser::parse(diff);
            for (line, expected) in cases {
                assert_eq!(
                    changes.map_line(line, from),
                    expected,
                    "{diff}, {from:?}, line {line}"
                );
            }
        }
    }

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

    #[test]
    fn test_parse_file_changes() {
        let diff = r#"
diff --git a/src/new.rs b/src/new.rs
new file mode 100644
index 0000000..abcdef1
--- /dev/null
+++ b/src/new.rs
@@ -0,0 +1,3 @@
+fn added() {}
+fn two() {}
+fn three() {}
diff --git a/src/old.rs b/src/renamed.rs
similarity index 95%
rename from src/old.rs
rename to src/renamed.rs
index 1111111..2222222 100644
--- a/src/old.rs
+++ b/src/renamed.rs
@@ -1,2 +1,2 @@
-let a = 1;
+let a = 2;
"#;
        let files = UnifiedDiffParser::parse_file_changes(diff);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "src/new.rs");
        assert_eq!(files[0].status, FileChangeStatus::Added);
        assert_eq!(files[0].additions, 3);
        assert_eq!(files[0].deletions, 0);

        assert_eq!(files[1].path, "src/renamed.rs");
        assert_eq!(files[1].old_path, Some("src/old.rs".to_string()));
        assert_eq!(
            files[1].status,
            FileChangeStatus::Renamed { similarity: 95 }
        );
        assert_eq!(files[1].additions, 1);
        assert_eq!(files[1].deletions, 1);
    }

    #[test]
    fn test_change_block_navigation() {
        let diff = r#"
@@ -5,2 +5,2 @@
-a
+b
@@ -20,1 +20,3 @@
-x
+y
+z
+w
"#;
        let changes = UnifiedDiffParser::parse(diff);
        assert_eq!(
            changes.previous_change_line(10, ReviewRevision::After),
            Some(5)
        );
        assert_eq!(
            changes.next_change_line(10, ReviewRevision::After),
            Some(20)
        );
        assert_eq!(changes.previous_change_line(5, ReviewRevision::After), None);
        assert_eq!(changes.next_change_line(25, ReviewRevision::After), None);
    }
}
