//! Sparse coordinate conversion over immutable, host-owned UTF-8 source.

/// Units used by a source-coordinate producer for its zero-based columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnEncoding {
    /// UTF-8 bytes.
    Utf8,
    /// UTF-16 code units.
    Utf16,
    /// Unicode scalar values (UTF-32 code units).
    Utf32,
}

impl ColumnEncoding {
    fn width(self, ch: char) -> usize {
        match self {
            Self::Utf8 => ch.len_utf8(),
            Self::Utf16 => ch.len_utf16(),
            Self::Utf32 => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Checkpoint {
    byte: usize,
    utf16: usize,
    utf32: usize,
}

impl Checkpoint {
    fn units(self, encoding: ColumnEncoding) -> usize {
        match encoding {
            ColumnEncoding::Utf8 => self.byte,
            ColumnEncoding::Utf16 => self.utf16,
            ColumnEncoding::Utf32 => self.utf32,
        }
    }
}

const CHECKPOINT_BYTES: usize = 4096;

/// Converts source bytes and encoded offsets with bounded suffix scans.
///
/// Offsets count from the beginning of the source, including line endings. Hosts
/// can subtract the units at a line start to obtain a line-relative column.
/// Construction scans once and stores three counters approximately every 4 KB,
/// plus EOF. Lookups binary-search these checkpoints and scan at most 4099 bytes.
/// The index does not retain or copy source text.
///
/// Every lookup must receive the same immutable text used at construction.
/// Rebuild the index when that text changes.
///
/// # Examples
///
/// ```rust
/// use reader_text::{ColumnEncoding, CoordinateIndex};
///
/// let source = "first\r\n😀é";
/// let index = CoordinateIndex::new(source);
/// let line_start = index.units_at_byte(source, 7, ColumnEncoding::Utf16).unwrap();
/// assert_eq!(index.byte_at_units(source, line_start + 2, ColumnEncoding::Utf16), Some(11));
/// assert_eq!(index.byte_at_units(source, line_start + 1, ColumnEncoding::Utf16), None);
/// ```
#[derive(Debug, Clone)]
pub struct CoordinateIndex {
    checkpoints: Vec<Checkpoint>,
}

impl CoordinateIndex {
    /// Indexes the original text without normalizing its line endings.
    #[must_use]
    pub fn new(source: &str) -> Self {
        let mut checkpoints = vec![Checkpoint::default()];
        let mut position = Checkpoint::default();
        for ch in source.chars() {
            position.byte += ch.len_utf8();
            position.utf16 += ch.len_utf16();
            position.utf32 += 1;
            if position.byte - checkpoints.last().unwrap().byte >= CHECKPOINT_BYTES {
                checkpoints.push(position);
            }
        }
        if checkpoints.last().unwrap().byte != position.byte {
            checkpoints.push(position);
        }
        Self { checkpoints }
    }

    /// Returns the encoded offset at an exact UTF-8 scalar boundary.
    ///
    /// Returns `None` beyond EOF or inside a scalar. EOF itself is valid.
    #[must_use]
    pub fn units_at_byte(
        &self,
        source: &str,
        byte: usize,
        encoding: ColumnEncoding,
    ) -> Option<usize> {
        if !source.is_char_boundary(byte) {
            return None;
        }
        if encoding == ColumnEncoding::Utf8 {
            return Some(byte);
        }
        let index = self.checkpoints.partition_point(|point| point.byte <= byte) - 1;
        let point = self.checkpoints[index];
        Some(
            point.units(encoding)
                + source
                    .get(point.byte..byte)?
                    .chars()
                    .map(|ch| encoding.width(ch))
                    .sum::<usize>(),
        )
    }

    /// Returns the exact UTF-8 byte at an encoded offset from the source start.
    ///
    /// Returns `None` beyond EOF, inside a UTF-8 scalar, or between the UTF-16
    /// surrogate units of a scalar. EOF itself is valid.
    #[must_use]
    pub fn byte_at_units(
        &self,
        source: &str,
        units: usize,
        encoding: ColumnEncoding,
    ) -> Option<usize> {
        if encoding == ColumnEncoding::Utf8 {
            return source.is_char_boundary(units).then_some(units);
        }
        let index = self
            .checkpoints
            .partition_point(|point| point.units(encoding) <= units)
            - 1;
        let point = self.checkpoints[index];
        if point.units(encoding) == units {
            return Some(point.byte);
        }
        let end = self.checkpoints.get(index + 1)?.byte;
        let mut current = point.units(encoding);
        for (byte, ch) in source.get(point.byte..end)?.char_indices() {
            if current == units {
                return Some(point.byte + byte);
            }
            current += encoding.width(ch);
            if current > units {
                return None;
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_coordinates_match_independent_encoding_at_every_byte() {
        for source in [
            String::new(),
            "ascii\r\nlast\r".into(),
            "aé🦀漢字e\u{301}\r\n".repeat(900),
        ] {
            let index = CoordinateIndex::new(&source);
            for encoding in [
                ColumnEncoding::Utf8,
                ColumnEncoding::Utf16,
                ColumnEncoding::Utf32,
            ] {
                let mut expected = vec![None; source.len() * 2 + 2];
                for byte in 0..=source.len() + 1 {
                    let units = source.get(..byte).map(|prefix| match encoding {
                        ColumnEncoding::Utf8 => prefix.len(),
                        ColumnEncoding::Utf16 => prefix.encode_utf16().count(),
                        ColumnEncoding::Utf32 => prefix.chars().count(),
                    });
                    assert_eq!(index.units_at_byte(&source, byte, encoding), units);
                    if let Some(units) = units {
                        expected[units] = Some(byte);
                    }
                }
                for (units, byte) in expected.into_iter().enumerate() {
                    assert_eq!(index.byte_at_units(&source, units, encoding), byte);
                }
                assert_eq!(index.byte_at_units(&source, usize::MAX, encoding), None);
                assert_eq!(index.units_at_byte(&source, usize::MAX, encoding), None);
            }
        }
    }

    #[test]
    fn multi_megabyte_lookup_scans_are_bounded_and_index_is_sparse() {
        let source = "abcé🦀漢字 ".repeat(400_000);
        let index = CoordinateIndex::new(&source);
        assert!(source.len() > 6_000_000);
        assert!(index.checkpoints.len() <= source.len() / CHECKPOINT_BYTES + 2);
        for pair in index.checkpoints.windows(2) {
            assert!(pair[1].byte - pair[0].byte <= CHECKPOINT_BYTES + 3);
            assert!(source.is_char_boundary(pair[1].byte));
        }
        for encoding in [ColumnEncoding::Utf16, ColumnEncoding::Utf32] {
            let units = index
                .units_at_byte(&source, source.len(), encoding)
                .unwrap();
            let suffix = "漢字 ";
            let suffix_units = suffix.chars().map(|ch| encoding.width(ch)).sum::<usize>();
            assert_eq!(
                index.byte_at_units(&source, units - suffix_units, encoding),
                Some(source.len() - suffix.len())
            );
            assert_eq!(index.byte_at_units(&source, units + 1, encoding), None);
        }
    }
}
