//! Outline preview/commit binding to the exact displayed source.

use super::*;
use crate::ui::outline_list::{OutlineAction, OutlineList};

impl RadiologyApp {
    pub(super) fn outline_available(&self) -> bool {
        self.source_location_ready()
            && self.under_caret.is_none()
            && !self.under_caret_job.is_running()
            && self.under_caret_error.is_none()
    }

    pub(super) fn sync_outline(&mut self) {
        if let Some(buffer) = &self.open_buffer {
            self.outline_list.sync(
                buffer,
                &self.outline_symbols,
                &self.test_code,
                buffer.byte_offset(self.caret_line, self.caret_col),
                self.line_changes.as_ref().filter(|_| self.review.is_some()),
            );
        } else {
            self.outline_list = Default::default();
        }
    }

    pub(super) fn handle_outline_keys(&mut self, ctx: &Context, input: &mut egui::RawInput) {
        if !self.outline_available() {
            return;
        }
        self.sync_outline();
        for key in crate::ui::commands::take_focused_keys(
            ctx,
            input,
            OutlineList::focus_id(),
            OutlineList::KEYS,
        ) {
            if let Some(action) = self.outline_list.key(key) {
                self.handle_outline_action(action);
            }
        }
    }

    pub(super) fn handle_outline_action(&mut self, action: OutlineAction) {
        if !self.outline_available() {
            return;
        }
        self.sync_outline();
        let (OutlineAction::Preview(target)
        | OutlineAction::Commit(target)
        | OutlineAction::Activate(target)) = &action;
        let Some(buffer) = &self.open_buffer else {
            return;
        };
        let Some(byte) = self.outline_list.destination(buffer, target) else {
            return;
        };
        match action {
            OutlineAction::Preview(_) => {
                self.fold_store
                    .reveal_line(buffer.line_at_byte(byte), &self.fold_ranges);
                self.source_viewport.preview_byte(buffer, byte);
            }
            OutlineAction::Commit(_) | OutlineAction::Activate(_) => self.navigate_target(
                buffer.location_at_byte(byte),
                None,
                source::NavigationIntent {
                    record_history: true,
                    reveal: true,
                    flash_destination: matches!(action, OutlineAction::Activate(_)),
                    ..Default::default()
                },
            ),
        }
    }
}
