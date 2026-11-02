//! Modal action dispatch and the input ownership of a disappearing sheet.

use super::*;
use crate::ui::modals::ModalInput;

#[derive(Default)]
pub(super) struct ModalFocus {
    previous: Option<egui::Id>,
    input: ModalInput,
    surface_ready: bool,
}

impl ModalFocus {
    fn prepare_surface(&mut self, ctx: &Context) -> Option<egui::Id> {
        let backdrop = modal_backdrop(ctx, true);
        // Raise a reused dialog alone once, then raise the pair together.
        if !self.input.is_registered() || self.surface_ready {
            ctx.move_to_top(backdrop.layer_id);
        } else {
            self.surface_ready = true;
            ctx.request_repaint();
        }
        if !backdrop.contains_pointer() || !ctx.input(|input| input.pointer.any_pressed()) {
            return None;
        }
        ctx.memory(|memory| memory.focused()).filter(|id| {
            ctx.read_response(*id)
                .is_some_and(|response| response.layer_id.order == egui::Order::Middle)
        })
    }
}

impl RadiologyApp {
    pub(super) fn take_modal_input(&mut self, ctx: &Context, input: &mut egui::RawInput) {
        if matches!(self.modal, ActiveModal::None) {
            return;
        }
        let focus = self.modal_focus.get_or_insert_with(|| ModalFocus {
            previous: ctx.memory(|memory| memory.focused()),
            ..Default::default()
        });
        focus.input.capture_input(input);
    }

    /// Returns whether a sheet owned this frame, including its closing input.
    pub(super) fn show_modal(&mut self, ctx: &Context) -> bool {
        if matches!(self.modal, ActiveModal::None) {
            modal_backdrop(ctx, false);
            self.modal_focus = None;
            return false;
        }
        let code_path_busy = self.is_code_path_storage_busy();
        let code_path_error = self.code_path_storage_error().map(str::to_owned);
        let focus = self.modal_focus.get_or_insert_with(|| ModalFocus {
            previous: ctx.memory(|memory| memory.focused()),
            ..Default::default()
        });
        let outside_focus = focus.prepare_surface(ctx);
        let previous = focus.previous;
        let action = focus.input.show(ctx, |ctx| match &mut self.modal {
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
                    .as_ref()
                    .filter(|_| self.active_revision != Some(ReviewRevision::Before)),
                &self.outline_symbols,
            ),
            ActiveModal::PullRequest(modal) => modal.show(ctx, &self.theme),
            ActiveModal::CodePathImport(modal) => {
                modal.show(ctx, &self.theme, code_path_busy, code_path_error.as_deref())
            }
        });
        let Some(action) = action else {
            if let Some(id) = outside_focus {
                ctx.memory_mut(|memory| memory.request_focus(id));
            }
            return true;
        };
        if let ModalAction::ImportCodePath(text) = action {
            self.import_code_path_text(text);
            return true;
        }
        modal_backdrop(ctx, false);
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
            ModalAction::ImportCodePath(_) => unreachable!(),
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

fn modal_backdrop(ctx: &Context, active: bool) -> egui::Response {
    let screen = ctx.screen_rect();
    let backdrop = egui::Area::new(egui::Id::new("modal_backdrop"))
        .order(egui::Order::Middle)
        .fixed_pos(screen.min)
        .movable(false)
        .interactable(active)
        .sense(egui::Sense::hover())
        .show(ctx, |ui| {
            if active {
                ui.set_min_size(screen.size());
            } else {
                // egui 0.27 snapshots an area's old size before its contents.
                // Retire that hover rectangle now, so next-frame input is live.
                ui.interact(
                    egui::Rect::from_min_size(screen.min, egui::Vec2::ZERO),
                    ui.layer_id().id.with("move"),
                    egui::Sense::hover(),
                );
            }
        });
    backdrop.response
}
