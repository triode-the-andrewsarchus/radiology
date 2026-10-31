//! Source-byte selection and bounded geometry for independent reader hosts.
//!
//! This crate has no GUI framework, application configuration, or daemon dependency.
//! Source bytes remain owned by the host; wrapping and continuation rows never
//! change selection/copy coordinates.

mod find;
mod projection;
mod rows;
mod unwrapped;
pub use find::{FindOptions, MatchSpool};
pub use projection::{projected_fragments, union_ranges, ProjectedFragment};
pub use rows::RowHeights;
use std::ops::Range;
pub use unwrapped::UnwrappedRows;

/// One bounded display fragment of a canonical source line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFragment {
    /// Exact UTF-8 bytes, excluding line terminators.
    pub bytes: Range<usize>,
    /// One-based canonical source line, shared by its continuation fragments.
    pub line: usize,
    /// Whether this fragment owns the line number and fold disclosure.
    pub first_in_line: bool,
}

/// Splits source into UTF-8-safe fragments without normalizing the original bytes.
///
/// The bound is clamped to at least four bytes so a complete scalar always fits.
/// CRLF and LF remain in the original source for copy; neither is painted.
#[must_use]
pub fn source_fragments(text: &str, max_bytes: usize) -> Vec<SourceFragment> {
    let max_bytes = max_bytes.max(4);
    let mut fragments = Vec::new();
    let mut offset = 0;
    for (index, chunk) in text.split('\n').enumerate() {
        let line_end = offset + chunk.strip_suffix('\r').unwrap_or(chunk).len();
        let mut start = offset;
        loop {
            let mut end = (start + max_bytes).min(line_end);
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            fragments.push(SourceFragment {
                bytes: start..end,
                line: index + 1,
                first_in_line: start == offset,
            });
            if end == line_end {
                break;
            }
            start = end;
        }
        offset += chunk.len() + 1;
    }
    fragments
}

/// A selection whose endpoints remain canonical bytes when rows scroll or fold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteSelection {
    /// Fixed endpoint, retained when extending selection.
    pub anchor: usize,
    /// Moving endpoint.
    pub focus: usize,
}

impl ByteSelection {
    /// Returns the selected half-open range in source order.
    #[must_use]
    pub fn range(self) -> Range<usize> {
        self.anchor.min(self.focus)..self.anchor.max(self.focus)
    }

    /// Copies from original source, including hidden rows and exact line endings.
    #[must_use]
    pub fn text(self, source: &str) -> Option<&str> {
        source.get(self.range())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fragments_bound_unicode_and_preserve_canonical_crlf_coordinates() {
        let text = "ab😀é\r\n\nlast\n";
        let fragments = source_fragments(text, 4);
        assert_eq!(
            fragments
                .iter()
                .map(|fragment| &text[fragment.bytes.clone()])
                .collect::<Vec<_>>(),
            vec!["ab", "😀", "é", "", "last", ""]
        );
        assert!(fragments.iter().all(|fragment| fragment.bytes.len() <= 4));
        assert_eq!(fragments[3].line, 2);
        assert!(!fragments[1].first_in_line);
        let selection = ByteSelection {
            anchor: 0,
            focus: text.len(),
        };
        assert_eq!(selection.text(text), Some(text));
    }

    #[test]
    fn measured_rows_match_linear_geometry_across_block_boundaries() {
        let mut rows = RowHeights::default();
        rows.reset(1003, 80.0, 22.0, 1.0);
        let mut expected = vec![22.0; 1003];
        for index in (0..1003).step_by(7) {
            let height = 22.0 + (index % 13) as f32 * 9.0;
            expected[index] = height;
            rows.set_height(index, height);
        }
        let mut y = 0.0;
        for (index, height) in expected.iter().enumerate() {
            assert!((rows.offset(index) - y).abs() < 0.1);
            assert_eq!(rows.row_at(y + 0.25), index);
            y += height;
        }
        assert!((rows.total_height() - y).abs() < 0.1);
        rows.reset(1003, 120.0, 22.0, 1.0);
        assert_eq!(rows.total_height(), 1003.0 * 22.0);
    }
}
