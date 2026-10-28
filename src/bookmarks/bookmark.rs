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
    /// Trimmed text of the bookmarked line used for relocation.
    pub anchor: Option<String>,
    /// Text of the preceding line for disambiguation.
    pub preceding_line: Option<String>,
    /// Text of the following line for disambiguation.
    pub following_line: Option<String>,
}

impl SourceBookmark {
    /// Creates a new [`SourceBookmark`] capturing context lines for future relocation.
    pub fn new(buffer: &SourceBuffer, byte_offset: usize, label: impl Into<String>) -> Self {
        let byte = buffer.clamp_to_char_boundary(byte_offset.min(buffer.data.len()));
        let line = buffer.line_at_byte(byte);
        let column = buffer.column_at_byte(byte);

        let anchor = if line <= buffer.line_count() {
            let t = buffer.line_text(line);
            if !t.trim().is_empty() {
                Some(t.to_string())
            } else {
                None
            }
        } else {
            None
        };

        let preceding_line = if line > 1 {
            Some(buffer.line_text(line - 1).to_string())
        } else {
            None
        };

        let following_line = if line < buffer.line_count() {
            Some(buffer.line_text(line + 1).to_string())
        } else {
            None
        };

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
    /// was deleted or matches ambiguously.
    pub fn resolve(&self, buffer: &SourceBuffer) -> Result<SourceLocation, &'static str> {
        if buffer.path != self.path || buffer.revision != self.revision {
            return Err("Bookmark belongs to another document");
        }

        let curr_digest = format!("{:x}", buffer.content_identity());

        if curr_digest == self.digest {
            return Ok(buffer.location_at_byte(self.byte_offset));
        }

        let Some(anchor) = &self.anchor else {
            return Err("Bookmark anchor text missing");
        };

        let mut matching_lines = Vec::new();
        for line_num in 1..=buffer.line_count() {
            if buffer.line_text(line_num) == anchor {
                matching_lines.push(line_num);
            }
        }

        if matching_lines.len() > 1 {
            matching_lines.retain(|&line_num| {
                let prev = if line_num > 1 {
                    Some(buffer.line_text(line_num - 1))
                } else {
                    None
                };
                let next = if line_num < buffer.line_count() {
                    Some(buffer.line_text(line_num + 1))
                } else {
                    None
                };
                prev == self.preceding_line.as_deref() && next == self.following_line.as_deref()
            });
        }

        if matching_lines.len() == 1 {
            let line = matching_lines[0];
            let byte = buffer.byte_offset(line, self.column);
            Ok(buffer.location_at_byte(byte))
        } else {
            Err("Bookmark location is ambiguous or was deleted")
        }
    }
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
}
