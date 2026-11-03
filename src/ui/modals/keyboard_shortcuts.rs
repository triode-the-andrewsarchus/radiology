//! Bundled reader help using the existing modal input and focus boundary.
use super::ModalAction;
use egui::{Context, Key, Modifiers};

const DOCUMENT: &str = include_str!("../../../plans/keyboard-shortcuts-linux.md");

/// Scrollable, selectable keyboard reference for standalone and embedded hosts.
#[derive(Default)]
pub struct KeyboardShortcutsModal {
    focused: bool,
    scroll: f32,
}

impl KeyboardShortcutsModal {
    /// Renders the bundled reference; hosts wrap this in [`super::ModalInput`].
    pub fn show(&mut self, ctx: &Context) -> Option<ModalAction> {
        let mut action = None;
        let screen = ctx.screen_rect();
        let height = (screen.height() - 100.0).clamp(200.0, 560.0);
        egui::Window::new("Keyboard Shortcuts")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .fixed_size(egui::vec2(
                (screen.width() - 60.0).clamp(280.0, 640.0),
                height,
            ))
            .show(ctx, |ui| {
                ctx.move_to_top(ui.layer_id());
                let jump_to_end = self.scroll_keys(ctx, height - 60.0);
                let output = egui::ScrollArea::vertical()
                    .id_source("keyboard_shortcuts_body")
                    .animated(false)
                    .max_height(height - 60.0)
                    .vertical_scroll_offset(self.scroll)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        for paragraph in DOCUMENT.split("\n\n").map(str::trim) {
                            if paragraph.starts_with("# ") || paragraph.is_empty() {
                                continue;
                            }
                            if let Some(title) = paragraph.strip_prefix("## ") {
                                ui.add_space(ui.spacing().item_spacing.y);
                                ui.heading(title);
                            } else if let Some((keys, explanation)) = paragraph.split_once(" — ")
                            {
                                ui.label(egui::RichText::new(keys.trim_matches('*')).strong());
                                ui.add(egui::Label::new(explanation).wrap(true).selectable(true));
                            } else {
                                ui.add(egui::Label::new(paragraph).wrap(true).selectable(true));
                            }
                        }
                        if jump_to_end {
                            ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                        }
                    });
                self.scroll = output.state.offset.y;
                ui.separator();
                ui.horizontal(|ui| {
                    let copy = ui.button("Copy Reference");
                    if copy.clicked() {
                        ctx.output_mut(|output| output.copied_text = DOCUMENT.into());
                    }
                    let done = ui.button("Done");
                    if !self.focused {
                        done.request_focus();
                        self.focused = ui.is_visible();
                    }
                    if done.clicked() {
                        action = Some(ModalAction::Close);
                    }
                    super::picker::focus_ring(ui, &[copy.id, done.id]);
                });
                if ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape)) {
                    action = Some(ModalAction::Close);
                }
            });
        action
    }

    fn scroll_keys(&mut self, ctx: &Context, page: f32) -> bool {
        for (key, distance) in [
            (Key::ArrowUp, -40.0),
            (Key::ArrowDown, 40.0),
            (Key::PageUp, -page),
            (Key::PageDown, page),
        ] {
            if ctx.input_mut(|input| input.consume_key(Modifiers::NONE, key)) {
                self.scroll = (self.scroll + distance).max(0.0);
            }
        }
        if ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Home)) {
            self.scroll = 0.0;
        }
        ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::End))
    }
}
