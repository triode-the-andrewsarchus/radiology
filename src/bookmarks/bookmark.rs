use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::source::buffer::SourceBuffer;
use crate::source::location::{ReviewRevision, SourceLocation};

/// A persistent bookmark tied to an exact canonical location or relocated line anchor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceBookmark {
    pub id: Uuid,
    pub path: String,
    pub revision: Option<ReviewRevision>,
    pub byte_offset: usize,
    pub line: usize,
    pub column: usize,
    pub label: String,
    pub digest: String,
    pub anchor: Option<String>,
    pub preceding_line: Option<String>,
    pub following_line: Option<String>,
}

impl SourceBookmark {
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

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        buffer.data.hash(&mut hasher);
        let digest = format!("{:x}", hasher.finish());

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

    pub fn location(&self) -> SourceLocation {
        SourceLocation::with_revision(&self.path, self.byte_offset, self.revision)
    }

    /// Resolves bookmark location within current buffer text, relocating if file content changed.
    pub fn resolve(&self, buffer: &SourceBuffer) -> Result<SourceLocation, &'static str> {
        if buffer.path != self.path || buffer.revision != self.revision {
            return Err("Bookmark belongs to another document");
        }

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        buffer.data.hash(&mut hasher);
        let curr_digest = format!("{:x}", hasher.finish());

        if curr_digest == self.digest {
            return Ok(buffer.location_at_byte(self.byte_offset));
        }

        let Some(anchor) = &self.anchor else {
            return Err("Bookmark anchor text missing");
        };

        // Search for matching anchor line
        let mut matching_lines = Vec::new();
        for line_num in 1..=buffer.line_count() {
            if buffer.line_text(line_num) == anchor {
                matching_lines.push(line_num);
            }
        }

        // If multiple matches exist, disambiguate using surrounding lines
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

        // Identical buffer resolves immediately
        let loc1 = bookmark.resolve(&buf1).unwrap();
        assert_eq!(loc1.byte_offset, target_byte);

        // Modified buffer with inserted header lines
        let code2 = "// new comment line 1\n// new comment line 2\nfn alpha() {}\nfn target() {\n    let x = 1;\n}\nfn omega() {}\n";
        let buf2 = SourceBuffer::from_str("src/test.rs", code2);
        let loc2 = bookmark.resolve(&buf2).unwrap();
        assert_eq!(buf2.line_at_byte(loc2.byte_offset), 4);
    }
}
