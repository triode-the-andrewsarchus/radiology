//! UTF-8 source text buffer with line-start indexing and coordinate conversion.

use crate::source::location::{ColumnEncoding, ReviewRevision, SourceLocation};
use std::ops::Range;
use thiserror::Error;

/// Describes errors that can occur when loading or validating a [`SourceBuffer`].
#[derive(Error, Debug)]
pub enum SourceBufferError {
    /// The file contains null bytes or is not valid UTF-8.
    #[error("File is binary or not valid UTF-8 text")]
    BinaryOrInvalidUtf8,
    /// An I/O error occurred while reading from disk.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// The file exceeds [`SourceBuffer::MAX_FILE_SIZE`].
    #[error("File exceeds maximum allowed size")]
    TooLarge,
}

/// Stores validated UTF-8 text with indexed line starts and coordinate translations.
///
/// Byte offsets are canonical across all Radiology subsystems. [`SourceBuffer`]
/// translates between byte offsets, 1-based line numbers, and 0-based character columns.
///
/// # Examples
///
/// ```rust
/// use radiology::source::SourceBuffer;
///
/// let buf = SourceBuffer::from_str("src/lib.rs", "fn hello() {\n    world();\n}\n");
/// assert_eq!(buf.line_count(), 4);
/// assert_eq!(buf.line_at_byte(0), 1);
/// assert_eq!(buf.line_text(1), "fn hello() {");
/// ```
#[derive(Debug, Clone)]
pub struct SourceBuffer {
    /// Repository-relative file path.
    pub path: String,
    /// Full UTF-8 document text.
    pub text: String,
    /// Raw byte representation of [`Self::text`].
    pub data: Vec<u8>,
    /// Byte offsets of line beginnings; always contains at least index 0.
    pub line_starts: Vec<usize>,
    /// Pull request review revision, or `None` for standard checkout files.
    pub revision: Option<ReviewRevision>,
    content_identity: u64,
}

impl SourceBuffer {
    /// Reads bounded UTF-8 source without allocating an oversized file first.
    ///
    /// # Errors
    /// Returns an error for missing, oversized, binary, or invalid UTF-8 files.
    pub fn read(
        path: &std::path::Path,
        relative_path: &str,
        revision: Option<ReviewRevision>,
    ) -> Result<Self, SourceBufferError> {
        use std::io::Read;
        let file = std::fs::File::open(path)?;
        if file.metadata()?.len() > Self::MAX_FILE_SIZE as u64 {
            return Err(SourceBufferError::TooLarge);
        }
        let mut bytes = Vec::new();
        file.take(Self::MAX_FILE_SIZE as u64 + 1)
            .read_to_end(&mut bytes)?;
        Self::from_bytes(relative_path, bytes, revision)
    }
    /// Maximum file size permitted for reading (16 MB).
    pub const MAX_FILE_SIZE: usize = 16 * 1024 * 1024;
    /// Maximum line length rendered in inline bookmark anchors.
    pub const DISPLAY_LINE_LIMIT: usize = 2000;

    /// Creates a [`SourceBuffer`] from raw bytes, verifying UTF-8 and size limits.
    ///
    /// # Errors
    ///
    /// Returns [`SourceBufferError::TooLarge`] if `bytes` exceeds [`Self::MAX_FILE_SIZE`].
    /// Returns [`SourceBufferError::BinaryOrInvalidUtf8`] if `bytes` contains null bytes
    /// or fails UTF-8 decoding.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use radiology::source::SourceBuffer;
    ///
    /// let bytes = b"let x = 1;\n".to_vec();
    /// let buffer = SourceBuffer::from_bytes("test.rs", bytes, None).unwrap();
    /// assert_eq!(buffer.line_text(1), "let x = 1;");
    /// ```
    pub fn from_bytes(
        path: impl Into<String>,
        bytes: Vec<u8>,
        revision: Option<ReviewRevision>,
    ) -> Result<Self, SourceBufferError> {
        if bytes.len() > Self::MAX_FILE_SIZE {
            return Err(SourceBufferError::TooLarge);
        }
        if bytes.contains(&0) {
            return Err(SourceBufferError::BinaryOrInvalidUtf8);
        }
        let text = String::from_utf8(bytes).map_err(|_| SourceBufferError::BinaryOrInvalidUtf8)?;
        Ok(Self::from_text(path.into(), text, revision))
    }

    /// Creates a source buffer from valid UTF-8 text without applying file-size limits.
    #[must_use]
    pub fn from_str(path: impl Into<String>, text: &str) -> Self {
        Self::from_text(path.into(), text.to_string(), None)
    }

    fn from_text(path: String, text: String, revision: Option<ReviewRevision>) -> Self {
        use std::hash::{Hash, Hasher};
        let data = text.as_bytes().to_vec();
        let mut line_starts = vec![0];
        line_starts.extend(
            data.iter()
                .enumerate()
                .filter_map(|(index, &byte)| (byte == b'\n').then_some(index + 1)),
        );
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        data.hash(&mut hash);
        Self {
            path,
            text,
            data,
            line_starts,
            revision,
            content_identity: hash.finish(),
        }
    }

    /// Returns the immutable byte identity used by document presentation and bookmarks.
    #[must_use]
    pub fn content_identity(&self) -> u64 {
        self.content_identity
    }

    /// Returns the total number of lines in the buffer.
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Returns the total byte length of the buffer.
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.data.len()
    }

    /// Returns the maximum line length in characters across all lines.
    ///
    /// Used for computing horizontal scroll extents when line wrapping is disabled.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use radiology::source::SourceBuffer;
    ///
    /// let buf = SourceBuffer::from_str("test.rs", "short\na very long line here\n");
    /// assert_eq!(buf.max_line_len(), 21);
    /// ```
    #[must_use]
    pub fn max_line_len(&self) -> usize {
        if self.line_starts.len() <= 1 {
            return self.text.chars().count();
        }
        let mut max_chars = 0;
        for i in 0..self.line_starts.len() - 1 {
            let start = self.line_starts[i];
            let end = self.line_starts[i + 1].saturating_sub(1); // omit newline
            let line_chars = self.text[start..end].chars().count();
            if line_chars > max_chars {
                max_chars = line_chars;
            }
        }
        let last_start = *self.line_starts.last().unwrap_or(&0);
        let last_chars = self.text[last_start..].chars().count();
        max_chars.max(last_chars)
    }

    /// Returns a [`SourceLocation`] pointing at the given byte offset, clamped to character bounds.
    #[must_use]
    pub fn location_at_byte(&self, byte: usize) -> SourceLocation {
        SourceLocation::with_revision(&self.path, self.clamp_to_char_boundary(byte), self.revision)
    }

    /// Returns the unique cache position key for this buffer.
    #[must_use]
    pub fn position_key(&self) -> String {
        SourceLocation::format_position_key(&self.path, self.revision)
    }

    /// Returns the 1-based line number containing the given byte offset.
    #[must_use]
    pub fn line_at_byte(&self, byte_offset: usize) -> usize {
        let clamped = self.clamp_to_char_boundary(byte_offset.min(self.data.len()));
        match self.line_starts.binary_search(&clamped) {
            Ok(idx) => idx + 1,
            Err(idx) => idx,
        }
    }

    /// Returns the 0-based character column on a 1-based line for the given byte offset.
    #[must_use]
    pub fn column_at_byte(&self, byte_offset: usize) -> usize {
        let line = self.line_at_byte(byte_offset);
        let start = self.line_starts[line - 1];
        let clamped = self.clamp_to_char_boundary(byte_offset.min(self.data.len()));
        if clamped <= start {
            return 0;
        }
        let line_prefix = &self.text[start..clamped];
        line_prefix.chars().count()
    }

    /// Converts a 1-based line number and 0-based character column into a canonical byte offset.
    #[must_use]
    pub fn byte_offset(&self, line: usize, column: usize) -> usize {
        if line > self.line_count() {
            return self.byte_len();
        }
        let line = line.max(1);
        self.byte_offset_encoded(line, column, ColumnEncoding::Utf32)
            .unwrap_or_else(|| self.line_byte_range(line).end)
    }

    /// Converts an encoded column on a 1-based line to an exact source byte.
    /// Returns `None` for invalid lines, split scalars/surrogates, or oversized columns.
    #[must_use]
    pub fn byte_offset_encoded(
        &self,
        line: usize,
        column: usize,
        encoding: ColumnEncoding,
    ) -> Option<usize> {
        let start = *self.line_starts.get(line.checked_sub(1)?)?;
        let text = self.line_text(line);
        if encoding == ColumnEncoding::Utf8 {
            return (column <= text.len() && text.is_char_boundary(column))
                .then_some(start + column);
        }
        let mut units = 0;
        for (byte, ch) in text.char_indices() {
            if units == column {
                return Some(start + byte);
            }
            units += if encoding == ColumnEncoding::Utf16 {
                ch.len_utf16()
            } else {
                1
            };
        }
        (units == column).then_some(start + text.len())
    }

    /// Returns the byte range of a 1-based line, excluding trailing `\n` or `\r\n`.
    #[must_use]
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

    /// Returns the text slice of a 1-based line, stripped of trailing newline characters.
    #[must_use]
    pub fn line_text(&self, line: usize) -> &str {
        let range = self.line_byte_range(line);
        &self.text[range]
    }

    /// Clamps an arbitrary byte offset to a valid UTF-8 character boundary.
    #[must_use]
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
        assert_eq!(buf.line_count(), 4);

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
        let text = "hello \u{20000} world\nsecond line\n";
        let buf = SourceBuffer::from_str("multibyte.txt", text);
        assert_eq!(buf.line_count(), 3);

        let char_start = 6;
        assert_eq!(buf.clamp_to_char_boundary(char_start + 1), char_start);
        assert_eq!(buf.column_at_byte(char_start), 6);
        let after_char = char_start + 4;
        assert_eq!(buf.column_at_byte(after_char), 7);
    }
    #[test]
    fn encoded_coordinates_reject_split_surrogates_and_utf8_scalars() {
        let buffer = SourceBuffer::from_str("unicode.rs", "😀éx\r\nlast");
        assert_eq!(
            buffer.byte_offset_encoded(1, 2, ColumnEncoding::Utf16),
            Some(4)
        );
        assert_eq!(
            buffer.byte_offset_encoded(1, 1, ColumnEncoding::Utf16),
            None
        );
        assert_eq!(buffer.byte_offset_encoded(1, 3, ColumnEncoding::Utf8), None);
        assert_eq!(
            buffer.byte_offset_encoded(1, 2, ColumnEncoding::Utf32),
            Some(6)
        );
        assert_eq!(buffer.column_at_byte(3), 0);
        assert_eq!(buffer.byte_offset(3, 0), buffer.byte_len());
    }
}
