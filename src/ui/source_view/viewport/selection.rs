//! Canonical pointer-selection units and bounded word-range work.

use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SelectionUnit {
    Character,
    Word,
    Line,
}

pub(super) struct SelectionGesture {
    pub unit: SelectionUnit,
    pub origin: Range<usize>,
    pub clicked: usize,
    pub clicks: u8,
    pub modifiers: Modifiers,
}

impl SelectionGesture {
    fn extend(&self, range: Range<usize>) -> ByteSelection {
        if range.start < self.origin.start {
            ByteSelection {
                anchor: self.origin.end,
                focus: range.start,
            }
        } else {
            ByteSelection {
                anchor: self.origin.start,
                focus: range.end,
            }
        }
    }
}

impl SourceViewport {
    pub(super) fn pointer_selection_range(
        &mut self,
        buffer: &SourceBuffer,
        byte: usize,
        unit: SelectionUnit,
    ) -> Option<Range<usize>> {
        if unit != SelectionUnit::Word {
            self.word_selection = None;
            return Some(if unit == SelectionUnit::Line {
                self.pointer_line_range(buffer, byte)
            } else {
                byte..byte
            });
        }
        if self
            .word_selection
            .as_ref()
            .is_none_or(|(previous, _)| *previous != byte)
        {
            self.word_selection = Some((
                byte,
                reader_text::WordRangeCursor::new(&buffer.text, &self.fragments, byte),
            ));
        }
        let started = std::time::Instant::now();
        let (_, cursor) = self.word_selection.as_mut()?;
        loop {
            if let Some(range) = cursor.advance(&buffer.text, &self.fragments) {
                self.word_selection = None;
                return Some(range);
            }
            if started.elapsed() >= std::time::Duration::from_millis(2) {
                return None;
            }
        }
    }

    fn pointer_line_range(&self, buffer: &SourceBuffer, byte: usize) -> Range<usize> {
        let index = self
            .fragments
            .partition_point(|part| part.bytes.start <= byte)
            .saturating_sub(1);
        let Some(part) = self.fragments.get(index) else {
            return byte..byte;
        };
        let first = self.fragments[..=index]
            .iter()
            .rposition(|part| part.first_in_line)
            .unwrap_or(0);
        let following = self.fragments[index + 1..]
            .iter()
            .position(|part| part.first_in_line)
            .map_or(self.fragments.len(), |next| index + 1 + next);
        let end = self.fragments[following - 1].bytes.end;
        let line = buffer.line_at_byte(end);
        let end = buffer
            .line_span_byte_range(line..line + 1)
            .map_or(end, |range| range.end);
        self.fragments[first].bytes.start..end.max(part.bytes.end)
    }

    pub(super) fn finish_pointer_selection(
        &mut self,
        content: &mut CanvasContent<'_>,
        range: Range<usize>,
        click: bool,
    ) -> Option<SourceViewAction> {
        let byte = range.start;
        let caret = self.extend_pointer_selection(range)?;
        self.dragging = false;
        let original = self.selection_gesture.take()?;
        let selected = self
            .selection
            .is_some_and(|selection| !selection.range().is_empty());
        let primary = original.modifiers.ctrl || original.modifiers.command;
        if !click {
            self.occurrence_clicks = None;
        }
        let moved = move_caret(content, caret);
        Some(match original.clicks {
            2 if original.unit == SelectionUnit::Word && !primary => {
                SourceViewAction::PinAtByte(original.clicked)
            }
            1 if click && !selected && !primary => SourceViewAction::SelectOccurrences(byte),
            _ => moved,
        })
    }

    pub(super) fn extend_pointer_selection(&mut self, range: Range<usize>) -> Option<usize> {
        let gesture = self.selection_gesture.as_ref()?;
        if gesture.unit == SelectionUnit::Character {
            self.selection.as_mut()?.focus = range.start;
        } else {
            self.selection = Some(gesture.extend(range));
        }
        Some(self.selection?.focus)
    }
}
