//! Ephemeral reading anchors, independent of exact-caret navigation history.
use super::{document_memory::DocumentMemory, SourceBuffer};

#[derive(Debug, Default)]
pub(crate) struct ReadingPositions {
    documents: DocumentMemory<Option<usize>>,
    byte: Option<usize>,
    top: Option<usize>,
}

impl ReadingPositions {
    /// Resolves a successfully accepted destination against exact document memory.
    /// Explicit navigation always wins; a restore reports whether an anchor existed.
    pub(crate) fn activate(
        &mut self,
        buffer: &SourceBuffer,
        requested: usize,
        restore: bool,
    ) -> (usize, bool) {
        self.documents.activate(buffer, &mut self.byte);
        let saved = self.byte.filter(|_| restore);
        let byte = buffer.clamp_to_char_boundary(saved.unwrap_or(requested).min(buffer.byte_len()));
        self.byte = Some(byte);
        // Initial layout is programmatic, not a new passive scrolling report.
        self.top = None;
        (byte, saved.is_some())
    }

    /// Records explicit caret/restoration reports or a changed passive viewport.
    /// First paint retains the accepted anchor rather than rounding away its column.
    pub(crate) fn observe(
        &mut self,
        buffer: &SourceBuffer,
        top: Option<usize>,
        position: Option<usize>,
    ) {
        let reported =
            position.or_else(|| top.filter(|byte| self.top.is_some() && self.top != Some(*byte)));
        if let Some(byte) = reported {
            self.byte = Some(buffer.clamp_to_char_boundary(byte.min(buffer.byte_len())));
        }
        self.top = top;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::ReviewRevision;

    #[test]
    fn reports_keep_columns_until_passive_scroll_and_explicit_targets_win() {
        let a = SourceBuffer::from_str("a.rs", "first\nsecond café\nthird\n");
        let b = SourceBuffer::from_str("b.rs", "other\n");
        let mut positions = ReadingPositions::default();
        positions.activate(&a, 0, true);
        positions.observe(&a, Some(0), Some(9));
        positions.activate(&b, 0, false);
        assert_eq!(positions.activate(&a, 0, true), (9, true));
        positions.observe(&a, Some(6), None); // Initial layout must not erase column 3.
        positions.activate(&b, 0, false);
        assert_eq!(positions.activate(&a, 0, true), (9, true));
        positions.observe(&a, Some(6), None);
        positions.observe(&a, Some(19), None); // Later passive scroll becomes the reading anchor.
        positions.activate(&b, 0, false);
        assert_eq!(positions.activate(&a, 0, true), (19, true));
        assert_eq!(positions.activate(&a, 2, false), (2, false));
    }

    #[test]
    fn exact_content_revision_and_utf8_boundaries_qualify_saved_positions() {
        let mut after = SourceBuffer::from_str("a.rs", "é🦀 end\n");
        after.revision = Some(ReviewRevision::After);
        let mut before = SourceBuffer::from_str("a.rs", "é🦀 end\n");
        before.revision = Some(ReviewRevision::Before);
        let mut positions = ReadingPositions::default();
        positions.activate(&after, 0, true);
        positions.observe(&after, Some(0), Some(4));
        assert_eq!(positions.activate(&before, 0, true), (0, false));
        positions.observe(&before, Some(0), Some(7));
        assert_eq!(positions.activate(&after, 0, true), (2, true));
        let mut edited = SourceBuffer::from_str("a.rs", "new\n");
        edited.revision = Some(ReviewRevision::After);
        assert_eq!(positions.activate(&edited, 0, true), (0, false));
        assert_eq!(positions.activate(&before, 0, true), (7, true));
        assert_eq!(
            positions.activate(&before, usize::MAX, false),
            (before.byte_len(), false)
        );
    }
}
