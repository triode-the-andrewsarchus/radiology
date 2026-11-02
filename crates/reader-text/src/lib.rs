//! Source-byte selection and bounded geometry for independent reader hosts.
//!
//! This crate has no GUI framework, application configuration, or daemon dependency.
//! Source bytes remain owned by the host; wrapping and continuation rows never
//! change selection/copy coordinates.

mod coordinates;
mod find;
mod projection;
mod rows;
mod unwrapped;
mod words;
pub use coordinates::{ColumnEncoding, CoordinateIndex};
pub use find::{FindOptions, MatchSpool};
pub use projection::{projected_fragments, union_ranges, ProjectedFragment};
pub use rows::RowHeights;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;
pub use unwrapped::UnwrappedRows;
pub use words::WordCursor;

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
/// Graphemes that fit within the bound are kept together. Larger graphemes are
/// split at scalar boundaries to retain bounded layout work.
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
            let end = start + fragment_prefix_len(&text[start..line_end], max_bytes, max_bytes);
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

// Scan only a bounded window: a source line may contain millions of scalars in
// one grapheme. A partial fragment may return zero so its caller can flush it.
fn fragment_prefix_len(text: &str, budget: usize, limit: usize) -> usize {
    let mut cut = budget.min(text.len());
    while !text.is_char_boundary(cut) {
        cut -= 1;
    }
    if cut == text.len() {
        return cut;
    }
    // Four extra bytes ensure a truncated grapheme crossing `cut` is known to
    // exceed `limit`, even after backing up to a UTF-8 boundary.
    let mut end = budget
        .saturating_add(limit)
        .saturating_add(4)
        .min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    if text[..end].is_ascii() {
        return cut;
    }
    for (start, grapheme) in text[..end].grapheme_indices(true) {
        if start >= cut {
            break;
        }
        if start + grapheme.len() > cut {
            return if grapheme.len() <= limit { start } else { cut };
        }
    }
    cut
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
    fn fragment_edges_keep_fitting_graphemes_and_bound_oversized_sequences() {
        for cluster in ["e\u{301}", "1\u{fe0f}\u{20e3}", "👨‍👩‍👧‍👦", "👍🏽", "🇺🇸"]
        {
            for limit in [4, 7, 8, 25, 32, 4096] {
                for prefix in [0, 1, limit - 1, limit, limit + 1] {
                    let source = format!("{}{} tail\r\nnext", "x".repeat(prefix), cluster);
                    let parts = source_fragments(&source, limit);
                    let projected = projected_fragments(&source, &[], limit);
                    assert_eq!(parts.len(), projected.len());
                    for (part, projected) in parts.iter().zip(&projected) {
                        assert!(part.bytes.len() <= limit);
                        assert_eq!(part.bytes, projected.bytes);
                    }
                    if cluster.len() <= limit {
                        assert!(
                            parts.iter().any(|part| part.bytes.start <= prefix
                                && part.bytes.end >= prefix + cluster.len()),
                            "{cluster:?}, {limit}, {prefix}"
                        );
                    }
                    let visible: String = parts
                        .iter()
                        .map(|part| &source[part.bytes.clone()])
                        .collect();
                    assert_eq!(visible, source.replace("\r\n", ""));
                }
            }
        }
        let source = format!("a{}z", "\u{301}".repeat(100_000));
        let parts = source_fragments(&source, 32);
        assert!(parts
            .iter()
            .all(|part| !part.bytes.is_empty() && part.bytes.len() <= 32));
        assert_eq!(
            parts
                .iter()
                .map(|part| &source[part.bytes.clone()])
                .collect::<String>(),
            source
        );
        assert_eq!(fragment_prefix_len("a\u{301}b", 1, 4), 0);
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
