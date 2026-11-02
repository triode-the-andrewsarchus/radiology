//! Cooperative source-word navigation over bounded projected fragments.

use crate::ProjectedFragment;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

/// Resumable movement over Unicode identifier runs and atomic source symbols.
///
/// Letters, numbers and underscores form words, including their combining marks.
/// Whitespace and ASCII punctuation separate words. Other graphemes (such as
/// emoji) are individual stops; keycaps remain atomic even though they contain a
/// digit. Camel case and underscores do not introduce subword stops.
///
/// Hosts provide the same immutable source/projection on every call and retire
/// this cursor on navigation, projection or input-owner changes. Progress is
/// private until [`Self::advance`] returns a complete canonical destination.
/// Each call visits at most one bounded fragment; the host owns frame scheduling.
///
/// # Examples
///
/// ```
/// use reader_text::{projected_fragments, WordCursor};
/// let source = "long_identifier tail";
/// let parts = projected_fragments(source, &[], 4);
/// let mut cursor = WordCursor::new(&parts, 0, true);
/// let destination = loop {
///     if let Some(byte) = cursor.advance(source, &parts) { break byte; }
/// };
/// assert_eq!(destination, "long_identifier".len());
/// ```
#[derive(Debug)]
pub struct WordCursor {
    byte: usize,
    fragment: usize,
    forward: bool,
    reading: bool,
    done: bool,
    selection: Option<SelectionKind>,
}

impl WordCursor {
    /// Starts a word movement at a canonical UTF-8 character boundary.
    #[must_use]
    pub fn new(parts: &[ProjectedFragment], byte: usize, forward: bool) -> Self {
        let byte = parts
            .first()
            .zip(parts.last())
            .map_or(byte, |(first, last)| {
                byte.clamp(first.bytes.start, last.bytes.end)
            });
        Self {
            byte,
            fragment: parts
                .partition_point(|part| part.bytes.start <= byte)
                .saturating_sub(1),
            forward,
            reading: false,
            done: parts.is_empty(),
            selection: None,
        }
    }

    /// Visits one fragment, returning `None` when more bounded work is required.
    ///
    /// A completed cursor keeps returning its destination. Visible line breaks
    /// stop a word; hidden joins remain hard boundaries. Copy coordinates are
    /// canonical and may enclose hidden bytes. Fragment size limits stay host-owned.
    ///
    /// # Panics
    ///
    /// May panic if the host replaces the source/projection during a movement.
    pub fn advance(&mut self, source: &str, parts: &[ProjectedFragment]) -> Option<usize> {
        if self.done {
            return Some(self.byte);
        }
        let part = &parts[self.fragment];
        let ranges = graphemes(source, part);
        if self.forward {
            for (range, text) in ranges {
                if range.end > self.byte && self.visit(range, text) {
                    break;
                }
            }
        } else {
            for (range, text) in ranges.rev() {
                if range.start < self.byte && self.visit(range, text) {
                    break;
                }
            }
        }
        if self.done {
            return Some(self.byte);
        }
        self.next_fragment(parts)
    }

    fn next_fragment(&mut self, parts: &[ProjectedFragment]) -> Option<usize> {
        let part = &parts[self.fragment];
        let next = if self.forward {
            self.fragment
                .checked_add(1)
                .filter(|&index| index < parts.len())
        } else {
            self.fragment.checked_sub(1)
        };
        let Some(next) = next else {
            self.done = true;
            return Some(self.byte);
        };
        let newline = if self.forward {
            parts[next].first_in_line
        } else {
            part.first_in_line
        };
        if self.reading && newline {
            self.done = true;
            return Some(self.byte);
        }
        if !self.reading {
            self.byte = if self.forward {
                parts[next].bytes.start
            } else {
                parts[next].bytes.end
            };
        }
        self.fragment = next;
        None
    }

    fn visit(&mut self, range: Range<usize>, text: &str) -> bool {
        let before = if self.forward { range.start } else { range.end };
        let after = if self.forward { range.end } else { range.start };
        let gap = if self.forward {
            before > self.byte
        } else {
            before < self.byte
        };
        if gap && self.reading {
            // The surviving character after a hidden gap owns its boundary.
            if self.forward && self.selection.is_none() {
                self.byte = before;
            }
            self.done = true;
            return true;
        }
        if let Some(kind) = self.selection {
            if kind == selection_kind(text) {
                self.byte = after;
            } else {
                self.done = true;
            }
            return self.done;
        }
        let word = is_word(text);
        if self.reading && !word {
            self.byte = before;
            self.done = true;
        } else if !word
            && !text
                .chars()
                .all(|ch| ch.is_whitespace() || ch.is_ascii_punctuation() || ch.is_control())
        {
            self.byte = after;
            self.done = true;
        } else {
            self.byte = after;
            self.reading |= word;
        }
        self.done
    }
}

fn graphemes<'a>(
    source: &'a str,
    part: &'a ProjectedFragment,
) -> impl DoubleEndedIterator<Item = (Range<usize>, &'a str)> + 'a {
    part.spans().iter().flat_map(|span| {
        source[span.clone()]
            .grapheme_indices(true)
            .map(move |(offset, text)| {
                let start = span.start + offset;
                (start..start + text.len(), text)
            })
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectionKind {
    Word,
    Whitespace,
    Atomic,
}

fn is_word(text: &str) -> bool {
    !text.contains('\u{20e3}') && (text.contains('_') || text.unicode_words().next().is_some())
}

fn selection_kind(text: &str) -> SelectionKind {
    if is_word(text) {
        SelectionKind::Word
    } else if text.chars().all(char::is_whitespace) {
        SelectionKind::Whitespace
    } else {
        SelectionKind::Atomic
    }
}

/// Cooperatively selects a source word, whitespace run or atomic grapheme.
///
/// Uses the same identifier policy as [`WordCursor`]. Hidden joins and logical
/// newlines stop a range; punctuation and compound symbols select one grapheme.
/// Grapheme boundaries follow the projection: a cluster larger than its host
/// fragment limit can be split, preserving that limit rather than scanning beyond it.
/// Each advance visits at most one bounded fragment per boundary. Hosts retain
/// the same immutable source/projection and retire progress when either changes.
///
/// # Examples
///
/// ```
/// use reader_text::{projected_fragments, WordRangeCursor};
/// let source = "a long_identifier tail";
/// let parts = projected_fragments(source, &[], 4);
/// let mut cursor = WordRangeCursor::new(source, &parts, 7);
/// let range = loop {
///     if let Some(range) = cursor.advance(source, &parts) { break range; }
/// };
/// assert_eq!(&source[range], "long_identifier");
/// ```
#[derive(Debug)]
pub struct WordRangeCursor {
    before: WordCursor,
    after: WordCursor,
}

impl WordRangeCursor {
    /// Starts at the visible grapheme containing `byte`, or an empty boundary.
    ///
    /// # Panics
    ///
    /// Panics if `parts` do not describe valid spans of `source`.
    #[must_use]
    pub fn new(source: &str, parts: &[ProjectedFragment], byte: usize) -> Self {
        let index = parts
            .partition_point(|part| part.bytes.start <= byte)
            .saturating_sub(1);
        let seed = parts
            .get(index)
            .into_iter()
            .flat_map(|part| graphemes(source, part))
            .find(|(range, _)| range.contains(&byte));
        let (range, kind) = seed.map_or((byte..byte, SelectionKind::Atomic), |(range, text)| {
            (range, selection_kind(text))
        });
        let mut before = WordCursor::new(parts, range.start, false);
        let mut after = WordCursor::new(parts, range.end, true);
        for cursor in [&mut before, &mut after] {
            cursor.reading = true;
            cursor.selection = Some(kind);
            cursor.done |= kind == SelectionKind::Atomic;
        }
        Self { before, after }
    }

    /// Visits bounded fragments and returns the complete canonical range when ready.
    ///
    /// # Panics
    ///
    /// May panic if the host changes source/projection while this cursor is active.
    pub fn advance(&mut self, source: &str, parts: &[ProjectedFragment]) -> Option<Range<usize>> {
        let start = self.before.advance(source, parts);
        let end = self.after.advance(source, parts);
        start.zip(end).map(|(start, end)| start..end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projected_fragments;

    fn destination(
        source: &str,
        parts: &[ProjectedFragment],
        start: usize,
        forward: bool,
    ) -> usize {
        let mut cursor = WordCursor::new(parts, start, forward);
        for _ in 0..=parts.len() {
            if let Some(byte) = cursor.advance(source, parts) {
                assert_eq!(cursor.advance(source, parts), Some(byte));
                return byte;
            }
        }
        panic!("word movement did not finish");
    }

    #[test]
    fn unicode_identifier_symbols_and_separators_have_stable_source_stops() {
        let source = "foo_bar café e\u{301} 漢字 👨‍👩‍👧‍👦 1️⃣ 👍🏽 🇺🇸 ... tail";
        let parts = projected_fragments(source, &[], 4096);
        let tokens = [
            "foo_bar",
            "café",
            "e\u{301}",
            "漢字",
            "👨‍👩‍👧‍👦",
            "1️⃣",
            "👍🏽",
            "🇺🇸",
            "tail",
        ];
        let mut byte = 0;
        for token in tokens {
            let start = source.find(token).unwrap();
            let end = start + token.len();
            assert_eq!(destination(source, &parts, byte, true), end);
            assert_eq!(destination(source, &parts, end, false), start);
            byte = end;
        }
        assert_eq!(destination(source, &parts, 0, false), 0);
        assert_eq!(
            destination(source, &parts, source.len(), true),
            source.len()
        );
        for source in ["", "...   ", "\r\n", "\n\r\n"] {
            let parts = projected_fragments(source, &[], 4);
            assert_eq!(destination(source, &parts, 0, true), source.len());
            assert_eq!(destination(source, &parts, source.len(), false), 0);
        }
        let source = "one\r\n\r\n";
        let parts = projected_fragments(source, &[], 4);
        assert_eq!(destination(source, &parts, 3, true), source.len());
        assert_eq!(destination(source, &parts, source.len(), false), 0);
    }

    #[test]
    fn fragment_continuations_are_not_word_boundaries_but_visible_newlines_are() {
        let source = "long_identifier\r\nnext_word";
        for limit in [4, 7, 4096] {
            let parts = projected_fragments(source, &[], limit);
            assert_eq!(destination(source, &parts, 0, true), 15);
            assert_eq!(destination(source, &parts, source.len(), false), 17);
            assert_eq!(destination(source, &parts, 15, true), source.len());
            assert_eq!(destination(source, &parts, 17, false), 0);
        }
        let source = "aaaa hidden\r\nbbbb tail";
        let hidden = 4..source.find("bbbb").unwrap();
        for limit in [4, 4096] {
            let parts = projected_fragments(source, std::slice::from_ref(&hidden), limit);
            assert_eq!(destination(source, &parts, 0, true), hidden.end);
            assert_eq!(
                destination(source, &parts, hidden.end + 4, false),
                hidden.end
            );
        }
    }

    #[test]
    fn huge_words_and_separators_progress_cooperatively_in_both_directions() {
        for unit in ["é", "."] {
            let source = format!("{} tail", unit.repeat(100_000));
            let parts = projected_fragments(&source, &[], 4096);
            let mut cursor = WordCursor::new(&parts, 0, true);
            assert!(cursor.advance(&source, &parts).is_none());
            let expected = if unit == "é" { 200_000 } else { source.len() };
            let edge = unit.len() * 100_000;
            let mut reverse = WordCursor::new(&parts, edge, false);
            assert!(reverse.advance(&source, &parts).is_none());
            assert_eq!(destination(&source, &parts, edge, false), 0);
            assert_eq!(destination(&source, &parts, 0, true), expected);
            assert_eq!(
                destination(&source, &parts, expected, false),
                if unit == "é" { 0 } else { source.len() - 4 }
            );
        }
    }
    #[test]
    fn word_ranges_keep_identifier_whitespace_grapheme_and_hidden_boundaries() {
        let source = "a café_name  \t.👍🏽1️⃣ tail\r\nnext";
        for expected in ["a", "café_name", "  \t", ".", "👍🏽", "1️⃣", "tail", "next"] {
            let start = source.find(expected).unwrap();
            for (offset, _) in source[start..start + expected.len()].char_indices() {
                let parts = projected_fragments(source, &[], 16);
                let mut cursor = WordRangeCursor::new(source, &parts, start + offset);
                let range = loop {
                    if let Some(range) = cursor.advance(source, &parts) {
                        break range;
                    }
                };
                assert_eq!(
                    range,
                    start..start + expected.len(),
                    "{expected:?}/{offset}"
                );
            }
        }
        let source = "leftHIDDENright";
        let parts = projected_fragments(source, std::slice::from_ref(&(4..10)), 4);
        for (byte, expected) in [(2, 0..4), (12, 10..15)] {
            let mut cursor = WordRangeCursor::new(source, &parts, byte);
            let range = loop {
                if let Some(range) = cursor.advance(source, &parts) {
                    break range;
                }
            };
            assert_eq!(range, expected);
        }
    }

    #[test]
    fn large_word_ranges_progress_without_publishing_partial_endpoints() {
        let word = "é_name".repeat(20_000);
        let source = format!("{word} tail");
        let parts = projected_fragments(&source, &[], 64);
        let mut cursor = WordRangeCursor::new(&source, &parts, word.len() / 2);
        assert!(cursor.advance(&source, &parts).is_none());
        let mut advances = 1;
        let range = loop {
            advances += 1;
            if let Some(range) = cursor.advance(&source, &parts) {
                break range;
            }
        };
        assert!(advances > 1000);
        assert_eq!(range, 0..word.len());
        assert_eq!(cursor.advance(&source, &parts), Some(range));
    }
}
