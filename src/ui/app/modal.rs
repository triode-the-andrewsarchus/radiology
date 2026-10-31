//! Modal action dispatch and the keyboard ownership of a disappearing sheet.

use super::*;
use crate::ui::commands::keyboard_event;

pub(super) struct ModalFocus {
    previous: Option<egui::Id>,
    input: Vec<egui::Event>,
    registered: bool,
}

impl RadiologyApp {
    pub(super) fn take_modal_input(&mut self, ctx: &Context, input: &mut egui::RawInput) {
        if matches!(self.modal, ActiveModal::None) {
            return;
        }
        let focus = self.modal_focus.get_or_insert_with(|| ModalFocus {
            previous: ctx.memory(|memory| memory.focused()),
            input: Vec::new(),
            registered: false,
        });
        focus.input.extend(crate::ui::commands::take_input_actions(
            &mut input.events,
            |event| keyboard_event(event).then(|| event.clone()),
        ));
    }

    /// Returns whether a sheet owned this frame, including its closing input.
    pub(super) fn show_modal(&mut self, ctx: &Context) -> bool {
        if matches!(self.modal, ActiveModal::None) {
            self.modal_focus = None;
            return false;
        }
        let focus = self.modal_focus.get_or_insert_with(|| ModalFocus {
            previous: ctx.memory(|memory| memory.focused()),
            input: Vec::new(),
            registered: false,
        });
        let previous = focus.previous;
        if focus.registered {
            ctx.input_mut(|input| input.events.extend(std::mem::take(&mut focus.input)));
        } else {
            // The first window pass can be invisible while egui measures it.
            focus.input.extend(ctx.input_mut(|input| {
                crate::ui::commands::take_input_actions(&mut input.events, |event| {
                    keyboard_event(event).then(|| event.clone())
                })
            }));
            focus.registered = true;
            ctx.request_repaint();
        }
        let action = match &mut self.modal {
            ActiveModal::None => unreachable!(),
            ActiveModal::QuickFilePicker(modal) => modal.show(ctx, &self.theme, &self.all_files),
            ActiveModal::GoToLine(modal) => modal.show(
                ctx,
                &self.theme,
                self.open_buffer
                    .as_ref()
                    .map_or(1, |buffer| buffer.line_count()),
            ),
            ActiveModal::GoToSymbol(modal) => modal.show(
                ctx,
                &self.theme,
                self.semantic_index
                    .as_deref()
                    .filter(|_| self.active_revision != Some(ReviewRevision::Before)),
                &self.outline_symbols,
            ),
            ActiveModal::PullRequest(modal) => modal.show(ctx, &self.theme),
        };
        ctx.input_mut(|input| input.events.retain(|event| !keyboard_event(event)));
        let Some(action) = action else {
            return true;
        };
        let source_navigation = matches!(
            action,
            ModalAction::OpenFile(_)
                | ModalAction::JumpToLine(_)
                | ModalAction::JumpToSymbol(_)
                | ModalAction::JumpToIndexedSymbol(_)
        );
        match action {
            ModalAction::OpenFile(path) => self.open_file(&path),
            ModalAction::JumpToLine(line) => {
                if let Some(buffer) = &self.open_buffer {
                    self.jump_to_location(&buffer.path.clone(), line, buffer.byte_offset(line, 0));
                }
            }
            ModalAction::JumpToSymbol(byte) => {
                if let Some(buffer) = &self.open_buffer {
                    self.navigate(buffer.location_at_byte(byte), true);
                }
            }
            ModalAction::JumpToIndexedSymbol(id) => self.navigate_to_indexed_symbol(&id),
            ModalAction::OpenPullRequest(number) => self.open_pull_request(number),
            ModalAction::Close => {}
        }
        self.modal = ActiveModal::None;
        self.modal_focus = None;
        ctx.request_repaint();
        if source_navigation {
            self.source_viewport.request_focus(ctx);
        } else if let Some(previous) = previous.filter(|id| ctx.read_response(*id).is_some()) {
            ctx.memory_mut(|memory| memory.request_focus(previous));
        } else if let Some(focused) = ctx.memory(|memory| memory.focused()) {
            ctx.memory_mut(|memory| memory.surrender_focus(focused));
        }
        true
    }
}
