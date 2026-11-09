//! Unified diff hunk parsing and line-level addition/deletion tracking.

use crate::pull_request::change::{FileChange, FileChangeStatus};
use crate::source::{location::ReviewRevision, SourceBuffer};
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
/// let changes = UnifiedDiffParser::parse(diff_text).unwrap();
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

/// Available previous and next change targets in the queried coordinate space.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ChangeNavigation {
    /// Previous changed block, or None at the beginning.
    pub previous: Option<usize>,
    /// Next changed block, or None at the end.
    pub next: Option<usize>,
}

impl FileLineChanges {
    /// Returns bounded one-based line targets for a one-based caret line.
    #[must_use]
    pub fn navigation_at(&self, line: usize, revision: ReviewRevision) -> ChangeNavigation {
        ChangeNavigation {
            previous: self.previous_change_line(line, revision),
            next: self.next_change_line(line, revision),
        }
    }

    /// Returns bounded original-byte targets for the caret in `buffer`.
    ///
    /// Empty hunk sides remain insertion points, including EOF. An unterminated
    /// changed final line includes its EOF caret; a trailing empty line does not.
    /// Binary changes have no targets. The buffer's revision selects the hunk side.
    ///
    /// # Examples
    ///
    /// ```
    /// use radiology::{pull_request::diff::UnifiedDiffParser, source::SourceBuffer};
    /// let source = SourceBuffer::from_str("a.rs", "one\ntwo");
    /// let changes = UnifiedDiffParser::parse("@@ -3,2 +2,0 @@\n-three\n-four\n").unwrap();
    /// assert_eq!(changes.navigation_at_byte(&source, 0).next, Some(7));
    /// assert_eq!(changes.navigation_at_byte(&source, 7).next, None);
    /// ```
    #[must_use]
    pub fn navigation_at_byte(&self, buffer: &SourceBuffer, caret: usize) -> ChangeNavigation {
        let mut blocks: Vec<_> = self
            .navigation_blocks(buffer.revision.unwrap_or(ReviewRevision::After))
            .into_iter()
            .map(|range| buffer.byte_offset(range.start, 0)..buffer.byte_offset(range.end, 0))
            .collect();
        blocks.dedup_by_key(|range| range.start);
        let inside = blocks.iter().position(|range| {
            range.contains(&caret)
                || (range.is_empty() && range.start == caret)
                || (!range.is_empty()
                    && range.end == buffer.byte_len()
                    && caret == buffer.byte_len()
                    && buffer.data.last() != Some(&b'\n'))
        });
        if let Some(index) = inside {
            ChangeNavigation {
                previous: index.checked_sub(1).map(|index| blocks[index].start),
                next: blocks.get(index + 1).map(|range| range.start),
            }
        } else {
            ChangeNavigation {
                previous: blocks
                    .iter()
                    .rfind(|range| range.start < caret)
                    .map(|range| range.start),
                next: blocks
                    .iter()
                    .find(|range| range.start > caret)
                    .map(|range| range.start),
            }
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
            .take_while(|range| range.start < current_line && range.end <= current_line)
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
        if self.is_binary {
            return Vec::new();
        }
        self.hunks
            .iter()
            .map(|hunk| {
                let range = match revision {
                    ReviewRevision::Before => hunk.removed_lines(),
                    ReviewRevision::After => hunk.added_lines(),
                };
                if range.is_empty() {
                    let anchor = (range.start + 1).max(1);
                    anchor..anchor
                } else {
                    range
                }
            })
            .collect()
    }

    /// Maps a canonical line into the other revision using exact zero-context hunks.
    /// Lines inside a change map to its first destination line, or the line after
    /// an empty destination gap. Lines outside changes retain accumulated shifts.
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
                return if target.is_empty() {
                    target.start.saturating_add(1).max(1)
                } else {
                    target.start.max(1)
                };
            }
            delta += target.len() as isize - source.len() as isize;
        }
        line.saturating_add_signed(delta).max(1)
    }
}

/// Describes a malformed hunk in unified diff output.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Unrecognized diff hunk header: {header}")]
pub struct DiffParseError {
    /// Complete header that could not be interpreted safely.
    pub header: String,
}

/// Parses unified diff outputs into structured [`FileLineChanges`].
pub struct UnifiedDiffParser;

impl UnifiedDiffParser {
    /// Parses the raw text output of `git diff -U0` into a [`FileLineChanges`] instance.
    ///
    /// # Errors
    ///
    /// Returns [`DiffParseError`] for a malformed hunk header or a range whose
    /// end or empty-side navigation anchor cannot be represented.
    ///
    /// # Examples
    ///
    /// ```
    /// use radiology::pull_request::diff::UnifiedDiffParser;
    /// let changes = UnifiedDiffParser::parse("@@ -1 +1,2 @@\n")?;
    /// assert_eq!(changes.total_additions(), 2);
    /// assert!(UnifiedDiffParser::parse("@@ -x +1 @@\n").is_err());
    /// # Ok::<(), radiology::pull_request::diff::DiffParseError>(())
    /// ```
    pub fn parse(text: &str) -> Result<FileLineChanges, DiffParseError> {
        let mut hunks = Vec::new();
        for line in text.lines() {
            if (line.starts_with("Binary files ") && line.ends_with(" differ"))
                || line == "GIT binary patch"
            {
                return Ok(FileLineChanges {
                    hunks: Vec::new(),
                    is_binary: true,
                });
            }
            if line.starts_with("@@ ") {
                hunks.push(Self::parse_hunk_header(line).ok_or_else(|| DiffParseError {
                    header: line.into(),
                })?);
            }
        }
        Ok(FileLineChanges {
            hunks,
            is_binary: false,
        })
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
        let (start, count) = if let Some((start_str, count_str)) = s.split_once(',') {
            (
                start_str.parse::<usize>().ok()?,
                count_str.parse::<usize>().ok()?,
            )
        } else {
            (s.parse::<usize>().ok()?, 1)
        };
        isize::try_from(start.checked_add(count.max(1))?).ok()?;
        Some((start, count))
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
            let changes = UnifiedDiffParser::parse(diff).unwrap();
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
    fn line_map_lands_on_changes_and_after_empty_gaps_in_both_revisions() {
        let changes =
            UnifiedDiffParser::parse("@@ -3,8 +2,0 @@\n@@ -20,0 +10,3 @@\n@@ -31,2 +26,5 @@\n")
                .unwrap();
        for (revision, cases) in [
            (
                ReviewRevision::Before,
                &[
                    (1, 1),
                    (2, 2),
                    (3, 3),
                    (10, 3),
                    (11, 3),
                    (20, 12),
                    (21, 16),
                    (31, 26),
                    (32, 26),
                    (33, 31),
                    (40, 38),
                ][..],
            ),
            (
                ReviewRevision::After,
                &[(3, 11), (11, 21), (25, 30), (26, 31), (29, 31), (100, 102)][..],
            ),
        ] {
            for &(line, expected) in cases {
                assert_eq!(
                    changes.map_line(line, revision),
                    expected,
                    "{revision:?} line {line}"
                );
            }
        }
    }

    #[test]
    fn malformed_hunk_headers_are_reported_instead_of_discarded() {
        for header in [
            "@@ -x +1 @@".into(),
            "@@ -1 +x @@".into(),
            "@@ -1 @@".into(),
            "@@ -1,,2 +1 @@".into(),
            "@@ --1 +1 @@".into(),
            "@@ -1,-1 +1 @@".into(),
            format!("@@ -0,{} +1 @@", isize::MAX as usize + 1),
            format!("@@ -{},2 +1 @@", usize::MAX),
            format!("@@ -1 +{},0 @@", usize::MAX),
        ] {
            let input = format!("@@ -1 +1 @@\n-old\n+new\n{header}\r\n");
            let error = UnifiedDiffParser::parse(&input).unwrap_err();
            assert_eq!(error.header, header);
            assert!(error.to_string().contains(&error.header));
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
        let changes = UnifiedDiffParser::parse(diff).unwrap();
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
    fn headers_counts_ranges_and_binary_markers_match_swift_fixtures() {
        let changes = UnifiedDiffParser::parse(
            "diff --git a/x.kt b/x.kt\nindex old..new 100644\n--- a/x.kt\n+++ b/x.kt\n\
             @@ -3,8 +2,0 @@ package inc.tk.os.engine\n-import java.io.InputStream\n\
             @@ -20,0 +10 @@ import kotlinx.coroutines.launch\n+import kotlinx.coroutines.coroutineScope\n\
             @@ -31,25 +20,21 @@ import kotlinx.serialization.json.int\n-/** old */\n+/** new */\n\\ No newline at end of file\n",
        ).unwrap();
        assert!(!changes.is_binary);
        assert_eq!(
            changes.hunks,
            [
                DiffHunk {
                    before_start: 3,
                    before_count: 8,
                    after_start: 2,
                    after_count: 0
                },
                DiffHunk {
                    before_start: 20,
                    before_count: 0,
                    after_start: 10,
                    after_count: 1
                },
                DiffHunk {
                    before_start: 31,
                    before_count: 25,
                    after_start: 20,
                    after_count: 21
                },
            ]
        );
        assert_eq!(
            changes.changed_lines(ReviewRevision::After),
            [10..11, 20..41]
        );
        assert_eq!(
            changes.changed_lines(ReviewRevision::Before),
            [3..11, 31..56]
        );
        assert_eq!(changes.change_anchors(ReviewRevision::After), [3, 10, 20]);
        assert_eq!(
            (changes.total_additions(), changes.total_deletions()),
            (22, 33)
        );
        for (text, expected) in [
            (
                "--- /dev/null\n+++ b/new.rs\n@@ -0,0 +1,3 @@\n+a\n+b\n+c\n",
                vec![(0, 0, 1, 3)],
            ),
            (
                "--- a/old.rs\n+++ /dev/null\n@@ -1,2 +0,0 @@\n-x\r\n-y\r\n",
                vec![(1, 2, 0, 0)],
            ),
            ("@@ -4 +4 @@\r\n-é\r\n+🚀\r\n", vec![(4, 1, 4, 1)]),
            (
                "@@ -2 +2 @@ line1\n-line2\r\n+line2 changed\r\n@@ -3,0 +4 @@ line3\n+line4\r\n",
                vec![(2, 1, 2, 1), (3, 0, 4, 1)],
            ),
        ] {
            let changes = UnifiedDiffParser::parse(text).unwrap();
            assert_eq!(
                changes
                    .hunks
                    .iter()
                    .map(|h| (h.before_start, h.before_count, h.after_start, h.after_count))
                    .collect::<Vec<_>>(),
                expected
            );
        }
        for marker in [
            "Binary files a/i.png and b/i.png differ\r\n",
            "GIT binary patch\n",
        ] {
            let changes = UnifiedDiffParser::parse(marker).unwrap();
            assert!(changes.is_binary);
            assert!(changes.hunks.is_empty());
        }
        assert_eq!(
            UnifiedDiffParser::parse("").unwrap(),
            FileLineChanges::default()
        );
        assert!(
            !UnifiedDiffParser::parse("+// Binary files a and b differ\n+GIT binary patch\n")
                .unwrap()
                .is_binary
        );
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
    fn byte_navigation_matches_inside_gap_and_unterminated_eof_boundaries() {
        let source = SourceBuffer::from_str(
            "a.rs",
            &(1..=30)
                .map(|n| format!("line{n}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        let changes =
            UnifiedDiffParser::parse("@@ -3,3 +3,3 @@\n@@ -12,2 +12,2 @@\n@@ -23 +23 @@\n")
                .unwrap();
        for (line, previous, next) in [
            (4, None, Some(12)),
            (13, Some(3), Some(23)),
            (9, Some(3), Some(12)),
            (1, None, Some(3)),
            (24, Some(23), None),
        ] {
            let expected = ChangeNavigation {
                previous: previous.map(|line| source.byte_offset(line, 0)),
                next: next.map(|line| source.byte_offset(line, 0)),
            };
            assert_eq!(
                changes.navigation_at_byte(&source, source.byte_offset(line, 1)),
                expected
            );
        }
        let last = UnifiedDiffParser::parse("@@ -1 +1 @@\n").unwrap();
        for (text, previous) in [("changed", None), ("changed\n", Some(0))] {
            let source = SourceBuffer::from_str("a.rs", text);
            assert_eq!(
                last.navigation_at_byte(&source, source.byte_len()),
                ChangeNavigation {
                    previous,
                    next: None
                }
            );
        }
    }

    #[test]
    fn empty_byte_targets_are_points_deduplicate_clamped_eof_and_ignore_binary_changes() {
        for (revision, patch) in [
            (ReviewRevision::After, "@@ -3,2 +2,0 @@\n@@ -8 +3,0 @@\n"),
            (ReviewRevision::Before, "@@ -2,0 +3,2 @@\n@@ -3,0 +8 @@\n"),
        ] {
            let mut source = SourceBuffer::from_str("a.rs", "one\ntwo");
            source.revision = Some(revision);
            let mut changes = UnifiedDiffParser::parse(patch).unwrap();
            assert!(changes.changed_lines(revision).is_empty());
            assert_eq!(
                changes.navigation_at_byte(&source, 0).next,
                Some(source.byte_len())
            );
            assert_eq!(
                changes.navigation_at_byte(&source, source.byte_len()),
                ChangeNavigation::default()
            );
            changes.is_binary = true;
            assert_eq!(
                changes.navigation_at_byte(&source, 0),
                ChangeNavigation::default()
            );
        }
        let source = SourceBuffer::from_str("a.rs", "one\ntwo\nthree\nfour\n");
        let changes = UnifiedDiffParser::parse("@@ -2,2 +1,0 @@\n@@ -6 +4 @@\n").unwrap();
        let point = source.byte_offset(2, 0);
        assert_eq!(changes.navigation_at_byte(&source, point).previous, None);
        assert_eq!(
            changes.navigation_at_byte(&source, point + 1),
            ChangeNavigation {
                previous: Some(point),
                next: Some(source.byte_offset(4, 0))
            }
        );
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
        let changes = UnifiedDiffParser::parse(diff).unwrap();
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
