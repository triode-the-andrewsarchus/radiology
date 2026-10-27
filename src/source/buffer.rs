use std::ops::Range;
use crate::source::location::{ReviewRevision, SourceLocation};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SourceBufferError {
    #[error("File is binary or not valid UTF-8 text")]
    BinaryOrInvalidUtf8,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("File exceeds maximum allowed size")]
    TooLarge,
}

/// UTF-8 source text buffer with indexed line starts and coordinate translations.
/// Byte offsets are canonical throughout Radiology.
#[derive(Debug, Clone)]
pub struct SourceBuffer {
    pub path: String,
    pub text: String,
    pub data: Vec<u8>,
    pub line_starts: Vec<usize>,
    pub revision: Option<ReviewRevision>,
}

impl SourceBuffer {
    pub const MAX_FILE_SIZE: usize = 16 * 1024 * 1024; // 16 MB limit
    pub const DISPLAY_LINE_LIMIT: usize = 2000;

    pub fn from_bytes(path: impl Into<String>, bytes: Vec<u8>, revision: Option<ReviewRevision>) -> Result<Self, SourceBufferError> {
        if bytes.len() > Self::MAX_FILE_SIZE {
            return Err(SourceBufferError::TooLarge);
        }
        if bytes.contains(&0) {
            return Err(SourceBufferError::BinaryOrInvalidUtf8);
        }
        let text = String::from_utf8(bytes).map_err(|_| SourceBufferError::BinaryOrInvalidUtf8)?;
        let data = text.as_bytes().to_vec();

        let mut line_starts = vec![0];
        for (i, &b) in data.iter().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }

        Ok(Self {
            path: path.into(),
            text,
            data,
            line_starts,
            revision,
        })
    }

    pub fn from_str(path: impl Into<String>, text: &str) -> Self {
        let bytes = text.as_bytes().to_vec();
        let mut line_starts = vec![0];
        for (i, &b) in bytes.iter().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Self {
            path: path.into(),
            text: text.to_string(),
            data: bytes,
            line_starts,
            revision: None,
        }
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    pub fn byte_len(&self) -> usize {
        self.data.len()
    }

    pub fn location_at_byte(&self, byte: usize) -> SourceLocation {
        SourceLocation::with_revision(&self.path, self.clamp_to_char_boundary(byte), self.revision)
    }

    pub fn position_key(&self) -> String {
        SourceLocation::format_position_key(&self.path, self.revision)
    }

    /// Returns the 1-based line number containing the given byte offset.
    pub fn line_at_byte(&self, byte_offset: usize) -> usize {
        let clamped = byte_offset.min(self.data.len());
        match self.line_starts.binary_search(&clamped) {
            Ok(idx) => idx + 1,
            Err(idx) => idx, // line_starts[idx - 1] < clamped < line_starts[idx]
        }
    }

    /// Returns the 0-based column (in chars) on a 1-based line for the given byte offset.
    pub fn column_at_byte(&self, byte_offset: usize) -> usize {
        let line = self.line_at_byte(byte_offset);
        let start = self.line_starts[line - 1];
        let clamped = byte_offset.min(self.data.len());
        if clamped <= start {
            return 0;
        }
        let line_prefix = &self.text[start..clamped];
        line_prefix.chars().count()
    }

    /// Converts 1-based line and 0-based char column into a byte offset.
    pub fn byte_offset(&self, line: usize, column: usize) -> usize {
        if self.line_starts.is_empty() {
            return 0;
        }
        let line_idx = line.saturating_sub(1).min(self.line_starts.len() - 1);
        let line_start = self.line_starts[line_idx];
        let line_end = if line_idx + 1 < self.line_starts.len() {
            self.line_starts[line_idx + 1].saturating_sub(1)
        } else {
            self.data.len()
        };

        if column == 0 {
            return line_start;
        }

        let slice = &self.text[line_start..line_end];
        let mut char_count = 0;
        for (byte_idx, _) in slice.char_indices() {
            if char_count == column {
                return line_start + byte_idx;
            }
            char_count += 1;
        }
        line_end
    }

    /// Byte range for a 1-based line (excluding trailing newline).
    pub fn line_byte_range(&self, line: usize) -> Range<usize> {
        if line == 0 || line > self.line_starts.len() {
            return 0..0;
        }
        let start = self.line_starts[line - 1];
        let end = if line < self.line_starts.len() {
            let next_start = self.line_starts[line];
            if next_start > 0 && self.data.get(next_start - 1) == Some(&b'\n') {
                if next_start > 1 && self.data.get(next_start - 2) == Some(&b'\r') {
                    next_start - 2
                } else {
                    next_start - 1
                }
            } else {
                next_start
            }
        } else {
            self.data.len()
        };
        start..end
    }

    /// Returns the text slice of a 1-based line (trimmed of trailing newline/CR).
    pub fn line_text(&self, line: usize) -> &str {
        let range = self.line_byte_range(line);
        &self.text[range]
    }

    /// Clamps an arbitrary byte offset to a valid UTF-8 character boundary.
    pub fn clamp_to_char_boundary(&self, mut byte: usize) -> usize {
        if byte >= self.text.len() {
            return self.text.len();
        }
        while byte > 0 && !self.text.is_char_boundary(byte) {
            byte -= 1;
        }
        byte
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_buffer() {
        let buf = SourceBuffer::from_str("empty.txt", "");
        assert_eq!(buf.line_count(), 1);
        assert_eq!(buf.line_at_byte(0), 1);
        assert_eq!(buf.byte_offset(1, 0), 0);
        assert_eq!(buf.line_text(1), "");
    }

    #[test]
    fn test_lines_and_coordinates() {
        let text = "alpha\nbravo\r\ncharlie\n";
        let buf = SourceBuffer::from_str("test.txt", text);
        assert_eq!(buf.line_count(), 4); // ["alpha\n", "bravo\r\n", "charlie\n", ""]

        assert_eq!(buf.line_at_byte(0), 1);
        assert_eq!(buf.line_text(1), "alpha");

        let bravo_start = 6;
        assert_eq!(buf.line_at_byte(bravo_start), 2);
        assert_eq!(buf.line_text(2), "bravo");

        assert_eq!(buf.line_text(3), "charlie");
        assert_eq!(buf.line_text(4), "");

        assert_eq!(buf.byte_offset(2, 2), bravo_start + 2);
        assert_eq!(buf.column_at_byte(bravo_start + 2), 2);
    }

    #[test]
    fn test_multibyte_utf8() {
        let text = "hello 🦀 world\nsecond line\n";
        let buf = SourceBuffer::from_str("crab.txt", text);
        assert_eq!(buf.line_count(), 3);

        let crab_start = 6;
        assert_eq!(buf.clamp_to_char_boundary(crab_start + 1), crab_start);
        assert_eq!(buf.column_at_byte(crab_start), 6);
        // After the 4-byte crab:
        let after_crab = crab_start + 4;
        assert_eq!(buf.column_at_byte(after_crab), 7);
    }
}
