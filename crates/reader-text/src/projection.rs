//! Disjoint visible source spans joined across hidden newlines.

use std::ops::Range;

/// A bounded displayed fragment retaining exact canonical source spans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedFragment {
    /// Canonical envelope; hidden bytes inside this range are never laid out.
    pub bytes: Range<usize>,
    /// Original line owning this displayed row's first visible byte.
    pub line: usize,
    /// Whether this fragment owns the displayed row's gutter.
    pub first_in_line: bool,
    spans: Vec<Range<usize>>,
}

impl ProjectedFragment {
    /// Returns the visible canonical spans in display order.
    #[must_use]
    pub fn spans(&self) -> &[Range<usize>] {
        &self.spans
    }

    /// Counts visible UTF-8 bytes, excluding hidden contents of the envelope.
    #[must_use]
    pub fn visible_len(&self) -> usize {
        self.spans.iter().map(|span| span.len()).sum()
    }

    /// Maps a canonical byte to the preceding visible character boundary.
    #[must_use]
    pub fn char_index(&self, source: &str, byte: usize) -> usize {
        self.spans
            .iter()
            .map(|span| {
                let mut end = byte.clamp(span.start, span.end);
                while !source.is_char_boundary(end) {
                    end -= 1;
                }
                source[span.start..end].chars().count()
            })
            .sum()
    }

    /// Maps a displayed character boundary to its canonical source byte.
    /// At a hidden gap, the surviving character after the gap owns the boundary.
    #[must_use]
    pub fn byte_at_char(&self, source: &str, mut index: usize) -> usize {
        for span in &self.spans {
            let text = &source[span.clone()];
            if let Some((byte, _)) = text.char_indices().nth(index) {
                return span.start + byte;
            }
            index = index.saturating_sub(text.chars().count());
        }
        self.bytes.end
    }

    /// Returns whether a source byte survives projection in this fragment.
    #[must_use]
    pub fn contains_byte(&self, byte: usize) -> bool {
        self.spans.iter().any(|span| span.contains(&byte)) || byte == self.bytes.end
    }
}

/// Projects exact hidden intervals while preserving source coordinates and bounds.
///
/// Hidden intervals must end on UTF-8 boundaries and lie inside `source`.
/// Overlapping/adjacent intervals are unioned before projecting. Hidden newlines
/// join their surviving prefixes and suffixes; visible CRLF/LF separate rows.
///
/// # Panics
///
/// Panics if a nonempty hidden interval exceeds the source or splits a UTF-8 scalar.
#[must_use]
pub fn projected_fragments(
    source: &str,
    hidden: &[Range<usize>],
    max_bytes: usize,
) -> Vec<ProjectedFragment> {
    let mut hidden: Vec<_> = hidden
        .iter()
        .filter(|range| !range.is_empty())
        .cloned()
        .collect();
    hidden.sort_by_key(|range| (range.start, range.end));
    let mut union = Vec::<Range<usize>>::new();
    for range in hidden {
        assert!(
            range.end <= source.len()
                && source.is_char_boundary(range.start)
                && source.is_char_boundary(range.end)
        );
        if let Some(last) = union.last_mut().filter(|last| range.start <= last.end) {
            last.end = last.end.max(range.end);
        } else {
            union.push(range);
        }
    }
    let mut builder = FragmentBuilder::new(max_bytes.max(4));
    let mut start = 0;
    let mut line = 1;
    for range in union
        .into_iter()
        .chain(std::iter::once(source.len()..source.len()))
    {
        builder.append(source, start..range.start, &mut line);
        line += source[range.clone()]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count();
        start = range.end;
    }
    builder.finish_row(start);
    builder.output
}

struct FragmentBuilder {
    output: Vec<ProjectedFragment>,
    spans: Vec<Range<usize>>,
    length: usize,
    limit: usize,
    line: usize,
    first: bool,
}

impl FragmentBuilder {
    fn new(limit: usize) -> Self {
        Self {
            output: Vec::new(),
            spans: Vec::new(),
            length: 0,
            limit,
            line: 1,
            first: true,
        }
    }

    fn flush(&mut self, empty_byte: usize) {
        let bytes = self.spans.first().map_or(empty_byte, |span| span.start)
            ..self.spans.last().map_or(empty_byte, |span| span.end);
        self.output.push(ProjectedFragment {
            bytes,
            line: self.line,
            first_in_line: self.first,
            spans: std::mem::take(&mut self.spans),
        });
        self.first = false;
        self.length = 0;
    }

    fn finish_row(&mut self, empty_byte: usize) {
        // A full continuation fragment does not create an additional empty row.
        if self.first || !self.spans.is_empty() {
            self.flush(empty_byte);
        }
        self.first = true;
    }

    fn append(&mut self, source: &str, range: Range<usize>, line: &mut usize) {
        let mut start = range.start;
        for chunk in source[range.clone()].split_inclusive('\n') {
            let newline = chunk.ends_with('\n');
            let end = start + chunk.len() - usize::from(newline);
            let end = if newline && end > start && source.as_bytes().get(end - 1) == Some(&b'\r') {
                end - 1
            } else {
                end
            };
            if self.first && self.spans.is_empty() {
                self.line = *line;
            }
            self.append_span(source, start..end);
            if newline {
                self.finish_row(start);
                *line += 1;
                self.line = *line;
            }
            start += chunk.len();
        }
    }

    fn append_span(&mut self, source: &str, range: Range<usize>) {
        let mut start = range.start;
        while start < range.end {
            let mut end = (start + self.limit - self.length).min(range.end);
            while !source.is_char_boundary(end) {
                end -= 1;
            }
            if end == start {
                self.flush(start);
                continue;
            }
            self.spans.push(start..end);
            self.length += end - start;
            start = end;
            if self.length == self.limit {
                self.flush(start);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_hidden_newlines_join_delimiters_with_exact_unicode_coordinates() {
        let source = "fn café() {\r\n hidden 😀();\r\n} tail\r\nnext";
        let start = source.find('{').unwrap() + 1;
        let end = source.find('}').unwrap();
        let parts = projected_fragments(source, &[start..end, start + 2..end], 4096);
        assert_eq!(parts.len(), 2);
        let visible: String = parts[0]
            .spans()
            .iter()
            .map(|range| &source[range.clone()])
            .collect();
        assert_eq!(visible, "fn café() {} tail");
        let close = visible.find('}').unwrap();
        let chars = visible[..close].chars().count();
        assert_eq!(parts[0].byte_at_char(source, chars), end);
        assert_eq!(parts[0].char_index(source, end), chars);
        assert!(!parts[0].contains_byte(start));
        assert_eq!((parts[0].line, parts[1].line), (1, 4));
    }

    #[test]
    fn unhidden_projection_matches_bounded_source_fragments() {
        for source in ["", "\n", "a\r\n\r\nend\n", "ab😀é tail", "{}"] {
            for limit in [4, 5, 4096] {
                let old = crate::source_fragments(source, limit);
                let new = projected_fragments(source, &[], limit);
                assert_eq!(old.len(), new.len(), "{source:?} at {limit}");
                for (old, new) in old.iter().zip(new.iter()) {
                    assert_eq!(
                        (&old.bytes, old.line, old.first_in_line),
                        (&new.bytes, new.line, new.first_in_line)
                    );
                }
            }
        }
    }

    #[test]
    fn projected_bounds_count_visible_bytes_and_keep_empty_rows() {
        let source = format!("a😀{}é tail\n\nend\n", " hidden\n".repeat(1000));
        let end = source.find('é').unwrap();
        let hidden = 5..end;
        let parts = projected_fragments(&source, std::slice::from_ref(&hidden), 5);
        assert!(parts.iter().all(|part| part.visible_len() <= 5));
        let text: String = parts
            .iter()
            .flat_map(|part| part.spans())
            .map(|range| &source[range.clone()])
            .collect();
        assert_eq!(text, "a😀é tailend");
        assert_eq!(parts.iter().filter(|part| part.first_in_line).count(), 4);
        assert!(parts.iter().filter(|part| part.bytes.is_empty()).count() >= 2);
    }
}
