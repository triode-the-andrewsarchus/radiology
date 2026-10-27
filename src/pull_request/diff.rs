use crate::source::location::ReviewRevision;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffHunk {
    pub before_start: usize,
    pub before_count: usize,
    pub after_start: usize,
    pub after_count: usize,
}

impl DiffHunk {
    pub fn removed_lines(&self) -> Range<usize> {
        self.before_start..(self.before_start + self.before_count)
    }

    pub fn added_lines(&self) -> Range<usize> {
        self.after_start..(self.after_start + self.after_count)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileLineChanges {
    pub hunks: Vec<DiffHunk>,
    pub is_binary: bool,
}

impl FileLineChanges {
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

    pub fn total_additions(&self) -> usize {
        self.hunks.iter().map(|h| h.after_count).sum()
    }

    pub fn total_deletions(&self) -> usize {
        self.hunks.iter().map(|h| h.before_count).sum()
    }
}

pub struct UnifiedDiffParser;

impl UnifiedDiffParser {
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
        // Example: @@ -10,4 +12,6 @@
        let mut parts = line.split_whitespace();
        let _at = parts.next()?; // @@
        let before_part = parts.next()?; // -10,4
        let after_part = parts.next()?; // +12,6

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
