//! Binding reference focus to exact parsed declarations and immutable indexes.

use super::*;

impl RadiologyApp {
    /// Updates enclosing-declaration references without rebuilding on each caret move.
    pub fn refresh_symbol_context(&mut self) {
        let index = self
            .open_buffer
            .as_ref()
            .filter(|buffer| buffer.revision != Some(ReviewRevision::Before))
            .and(self.semantic_index.clone());
        let automatic =
            self.open_buffer
                .as_ref()
                .zip(index.as_ref())
                .and_then(|(buffer, index)| {
                    let byte = buffer.byte_offset(self.caret_line, self.caret_col);
                    let mut enclosing = Vec::new();
                    for symbol in &self.outline_symbols {
                        symbol.breadcrumbs(byte, &mut enclosing);
                    }
                    ReferenceTarget::declaration(index, buffer, enclosing.last()?)
                });
        self.reference_context
            .update(index, self.open_buffer.as_deref(), automatic);
    }

    pub(super) fn find_references_at_caret(&mut self) {
        if let Some(buffer) = &self.open_buffer {
            self.find_references_at_byte(buffer.byte_offset(self.caret_line, self.caret_col));
        }
    }

    pub(super) fn find_references_at_byte(&mut self, byte: usize) {
        let target = self
            .open_buffer
            .as_ref()
            .zip(self.semantic_index.as_ref())
            .and_then(|(buffer, index)| ReferenceTarget::at_byte(index, buffer, byte));
        let Some(target) = target else {
            self.error = Some("No exact indexed symbol at this source position.".into());
            return;
        };
        self.refresh_symbol_context();
        self.reference_context.show(target);
        self.show_inspector = true;
        self.error = None;
    }

    pub(super) fn follow_reference_caret(&mut self) {
        self.refresh_symbol_context();
        self.reference_context.follow_caret();
    }

    /// Returns whether the current reference context awaits background publication.
    #[must_use]
    pub fn is_reference_loading(&self) -> bool {
        self.reference_context.is_running()
    }
}
