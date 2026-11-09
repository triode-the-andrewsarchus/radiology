//! Persistent source bookmark model with anchor-based relocation.

use crate::source::buffer::SourceBuffer;
use crate::source::location::{ReviewRevision, SourceLocation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A persistent bookmark tied to an exact canonical location or relocated line anchor.
///
/// When a file is modified, [`Self::resolve`] uses the preserved line anchor and surrounding
/// context lines to find the new line number rather than jumping to outdated byte offsets.
///
/// # Examples
///
/// ```rust
/// use radiology::bookmarks::bookmark::SourceBookmark;
/// use radiology::source::SourceBuffer;
///
/// let buf = SourceBuffer::from_str("src/main.rs", "fn start() {}\nfn process() {\n}\n");
/// let bookmark = SourceBookmark::new(&buf, 15, "process function");
///
/// assert_eq!(bookmark.line, 2);
/// assert!(bookmark.anchor.is_some());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceBookmark {
    /// Unique identifier for this bookmark.
    pub id: Uuid,
    /// Repository-relative path of the bookmarked document.
    pub path: String,
    /// Pull request revision if created during a review.
    pub revision: Option<ReviewRevision>,
    /// Byte offset within the file at the time the bookmark was recorded.
    pub byte_offset: usize,
    /// 1-based line number.
    pub line: usize,
    /// 0-based character column.
    pub column: usize,
    /// User-visible label or path tag.
    pub label: String,
    /// Hash digest of file bytes when recorded.
    pub digest: String,
    /// Original bounded line text used for relocation; absent for blank/oversized lines.
    pub anchor: Option<String>,
    /// Text of the preceding line for disambiguation.
    pub preceding_line: Option<String>,
    /// Text of the following line for disambiguation.
    pub following_line: Option<String>,
}

impl SourceBookmark {
    /// Creates a new [`SourceBookmark`] capturing bounded context for relocation.
    ///
    /// Lines larger than [`SourceBuffer::DISPLAY_LINE_LIMIT`] are omitted from the
    /// anchor and context. Exact-byte navigation remains available for unchanged source.
    pub fn new(buffer: &SourceBuffer, byte_offset: usize, label: impl Into<String>) -> Self {
        let byte = buffer.clamp_to_char_boundary(byte_offset.min(buffer.data.len()));
        let line = buffer.line_at_byte(byte);
        let column = buffer.column_at_byte(byte);

        let anchor = bounded_line(buffer, line)
            .filter(|text| !text.trim().is_empty())
            .map(str::to_owned);
        let preceding_line = bounded_line(buffer, line.saturating_sub(1)).map(str::to_owned);
        let following_line = bounded_line(buffer, line + 1).map(str::to_owned);

        let digest = format!("{:x}", buffer.content_identity());

        Self {
            id: Uuid::new_v4(),
            path: buffer.path.clone(),
            revision: buffer.revision,
            byte_offset: byte,
            line,
            column,
            label: label.into(),
            digest,
            anchor,
            preceding_line,
            following_line,
        }
    }

    /// Returns the initial [`SourceLocation`] of this bookmark.
    #[must_use]
    pub fn location(&self) -> SourceLocation {
        SourceLocation::with_revision(&self.path, self.byte_offset, self.revision)
    }

    /// Resolves the bookmark within the current `buffer`, relocating using anchors if bytes changed.
    ///
    /// # Errors
    ///
    /// Returns an error string if the bookmark belongs to a different document or if the anchor
    /// was omitted, deleted, matches ambiguously, or has an invalid saved column.
    pub fn resolve(&self, buffer: &SourceBuffer) -> Result<SourceLocation, &'static str> {
        self.resolve_cancellable(buffer, || false)
    }

    pub(crate) fn resolve_cancellable(
        &self,
        buffer: &SourceBuffer,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<SourceLocation, &'static str> {
        if cancelled() {
            return Err("Bookmark relocation cancelled");
        }
        if buffer.path != self.path || buffer.revision != self.revision {
            return Err("Bookmark belongs to another document");
        }
        if format!("{:x}", buffer.content_identity()) == self.digest {
            return Ok(buffer.location_at_byte(self.byte_offset));
        }
        let Some(anchor) = &self.anchor else {
            return Err("Bookmark anchor text missing");
        };
        let mut matching_lines = Vec::with_capacity(2);
        let mut contextual_lines = Vec::with_capacity(2);
        for line in 1..=buffer.line_count() {
            if line % 256 == 0 && cancelled() {
                return Err("Bookmark relocation cancelled");
            }
            if bounded_line(buffer, line) != Some(anchor.as_str()) {
                continue;
            }
            if matching_lines.len() < 2 {
                matching_lines.push(line);
            }
            if self.context_matches(buffer, line) {
                contextual_lines.push(line);
                if contextual_lines.len() == 2 {
                    return Err("Bookmark location is ambiguous or was deleted");
                }
            }
        }
        if cancelled() {
            return Err("Bookmark relocation cancelled");
        }
        let candidates = if matching_lines.len() == 1 {
            matching_lines
        } else {
            contextual_lines
        };
        let [line] = candidates.as_slice() else {
            return Err("Bookmark location is ambiguous or was deleted");
        };
        let byte = buffer
            .byte_offset_encoded(*line, self.column, crate::source::ColumnEncoding::Utf32)
            .ok_or("Bookmark column is not valid in this source")?;
        Ok(buffer.location_at_byte(byte))
    }

    fn context_matches(&self, buffer: &SourceBuffer, line: usize) -> bool {
        [
            (
                line.checked_sub(1).filter(|number| *number > 0),
                self.preceding_line.as_deref(),
            ),
            (
                (line < buffer.line_count()).then_some(line + 1),
                self.following_line.as_deref(),
            ),
        ]
        .into_iter()
        .all(|(number, expected)| match number {
            Some(number) => bounded_line(buffer, number).is_some_and(|text| Some(text) == expected),
            None => expected.is_none(),
        })
    }
}

fn bounded_line(buffer: &SourceBuffer, line: usize) -> Option<&str> {
    (line > 0
        && line <= buffer.line_count()
        && buffer.line_byte_range(line).len() <= SourceBuffer::DISPLAY_LINE_LIMIT)
        .then(|| buffer.line_text(line))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmark_creation_and_relocation() {
        let code1 = "fn alpha() {}\nfn target() {\n    let x = 1;\n}\nfn omega() {}\n";
        let buf1 = SourceBuffer::from_str("src/test.rs", code1);
        let target_byte = buf1.byte_offset(2, 4);

        let bookmark = SourceBookmark::new(&buf1, target_byte, "target bookmark");
        assert_eq!(bookmark.line, 2);
        assert_eq!(bookmark.anchor, Some("fn target() {".to_string()));

        let loc1 = bookmark.resolve(&buf1).unwrap();
        assert_eq!(loc1.byte_offset, target_byte);

        let code2 = "// new comment line 1\n// new comment line 2\nfn alpha() {}\nfn target() {\n    let x = 1;\n}\nfn omega() {}\n";
        let buf2 = SourceBuffer::from_str("src/test.rs", code2);
        let loc2 = bookmark.resolve(&buf2).unwrap();
        assert_eq!(buf2.line_at_byte(loc2.byte_offset), 4);
    }

    #[test]
    fn saved_anchors_are_bounded_by_original_utf8_bytes_without_losing_exact_navigation() {
        let limit = "é".repeat(SourceBuffer::DISPLAY_LINE_LIMIT / 2);
        let exact = SourceBuffer::from_str("a.rs", &limit);
        assert_eq!(
            SourceBookmark::new(&exact, 2, "at limit").anchor.as_deref(),
            Some(limit.as_str())
        );
        let oversized = format!("{limit}é");
        let buffer =
            SourceBuffer::from_str("a.rs", &format!("{oversized}\r\ncenter\r\n{oversized}\r\n"));
        let middle = SourceBookmark::new(&buffer, buffer.byte_offset(2, 0), "middle");
        assert_eq!(middle.anchor.as_deref(), Some("center"));
        assert!(middle.preceding_line.is_none() && middle.following_line.is_none());
        let long = SourceBookmark::new(&buffer, 2, "long");
        assert!(long.anchor.is_none());
        assert_eq!(long.following_line.as_deref(), Some("center"));
        assert_eq!(long.resolve(&buffer).unwrap().byte_offset, 2);
        assert!(serde_json::to_vec(&long).unwrap().len() < 1024);
        let changed = SourceBuffer::from_str("a.rs", &format!("{}tail\r\n", buffer.text));
        assert!(long.resolve(&changed).unwrap_err().contains("anchor"));
        let mut legacy = long;
        legacy.anchor = Some(oversized);
        assert!(
            legacy.resolve(&changed).is_err(),
            "legacy long anchors cannot bypass the relocation limit"
        );
    }

    #[test]
    fn relocation_uses_unique_anchor_or_bounded_context_and_rejects_invalid_columns() {
        let original = SourceBuffer::from_str("a.rs", "before\r\né🦀 target\r\nafter\r\n");
        let bookmark = SourceBookmark::new(&original, original.byte_offset(2, 1), "target");
        let unique = SourceBuffer::from_str("a.rs", "different\r\né🦀 target\r\nother\r\n");
        assert_eq!(
            bookmark.resolve(&unique).unwrap().byte_offset,
            unique.byte_offset(2, 1)
        );
        let contextual = SourceBuffer::from_str(
            "a.rs",
            "wrong\r\né🦀 target\r\nother\r\nbefore\r\né🦀 target\r\nafter\r\n",
        );
        assert_eq!(
            bookmark.resolve(&contextual).unwrap().byte_offset,
            contextual.byte_offset(5, 1)
        );
        let ambiguous = SourceBuffer::from_str("a.rs", &original.text.repeat(2));
        assert!(bookmark.resolve(&ambiguous).is_err());
        let edited = SourceBuffer::from_str("a.rs", &original.text.replace("target", "changed"));
        assert!(bookmark.resolve(&edited).is_err());
        let mut before = original.clone();
        before.revision = Some(ReviewRevision::Before);
        assert!(bookmark.resolve(&before).is_err());
        let mut invalid = bookmark.clone();
        invalid.column = usize::MAX;
        assert!(invalid.resolve(&unique).unwrap_err().contains("column"));
        let large = "x".repeat(SourceBuffer::DISPLAY_LINE_LIMIT + 1);
        let unbounded_context =
            SourceBuffer::from_str("a.rs", &format!("{large}\nneedle\nafter\n"));
        let bookmark = SourceBookmark::new(
            &unbounded_context,
            unbounded_context.byte_offset(2, 0),
            "needle",
        );
        let ambiguous = SourceBuffer::from_str("a.rs", &unbounded_context.text.repeat(2));
        assert!(
            bookmark.resolve(&ambiguous).is_err(),
            "omitted oversized context must not prove uniqueness"
        );
    }

    #[test]
    fn relocation_checks_retirement_before_exact_lookup_and_during_a_long_scan() {
        let original = SourceBuffer::from_str("a.rs", "needle\n");
        let bookmark = SourceBookmark::new(&original, 0, "needle");
        assert!(bookmark
            .resolve_cancellable(&original, || true)
            .unwrap_err()
            .contains("cancelled"));
        let changed =
            SourceBuffer::from_str("a.rs", &format!("{}needle\n", "other\n".repeat(50000)));
        let mut checks = 0;
        let result = bookmark.resolve_cancellable(&changed, || {
            checks += 1;
            checks >= 3
        });
        assert!(result.unwrap_err().contains("cancelled"));
        assert!(checks <= 4, "a retired scan must stop polling promptly");
        assert_eq!(
            bookmark.resolve(&changed).unwrap().byte_offset,
            changed.byte_offset(50001, 0)
        );
    }
}
