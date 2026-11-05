//! Ordered keyboard batches shared by asynchronous navigation pickers.

use super::ModalAction;
use crate::{
    background::LatestRead,
    theme::AppTheme,
    ui::{commands, text_fit},
};
use egui::{Context, Event, Key};
use std::collections::VecDeque;

#[derive(Default)]
pub(super) struct PickerInput {
    pub(super) events: VecDeque<Event>,
    pub(super) navigation: VecDeque<Key>,
}

impl PickerInput {
    pub(super) fn begin<T>(&mut self, ctx: &Context, worker: &mut LatestRead<T>) -> Option<Key> {
        if self.take(ctx) {
            worker.stop();
            self.clear();
            Some(Key::Escape)
        } else {
            self.prepare(ctx)
        }
    }
    /// Captures exclusive input; Escape cancels even behind a pending Return.
    pub(super) fn take(&mut self, ctx: &Context) -> bool {
        self.events.extend(ctx.input_mut(|input| {
            commands::take_input_actions(&mut input.events, |event| {
                commands::keyboard_event(event).then(|| event.clone())
            })
        }));
        self.events
            .iter()
            .any(|event| control(event) == Some(Key::Escape))
    }

    /// Delivers text before its next navigation key. Return waits for publication.
    pub(super) fn prepare(&mut self, ctx: &Context) -> Option<Key> {
        if self.navigation.contains(&Key::Enter) {
            return None;
        }
        while let Some(event) = self.events.pop_front() {
            if let Some(key) = control(&event) {
                return Some(key);
            }
            ctx.input_mut(|input| input.events.push(event));
        }
        None
    }

    pub(super) fn clear(&mut self) {
        self.events.clear();
        self.navigation.clear();
    }

    pub(super) fn route(
        &mut self,
        ctx: &Context,
        key: Option<Key>,
        query: &egui::Response,
        enabled: bool,
    ) {
        let Some(key) = key else {
            return;
        };
        if key == Key::Enter && !query.has_focus() && !query.lost_focus() {
            ctx.input_mut(|input| {
                input.events.push(Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                })
            });
        } else if enabled {
            self.navigation.push_back(key);
        }
    }

    /// Retires closing work and keeps queued batches moving while the picker lives.
    pub(super) fn finish<T>(
        &mut self,
        ctx: &Context,
        worker: &mut LatestRead<T>,
        action: Option<ModalAction>,
    ) -> Option<ModalAction> {
        if action.is_some() {
            worker.stop();
            self.clear();
        } else if !self.events.is_empty() {
            ctx.request_repaint();
        }
        action
    }
}

pub(super) fn window(title: &str, owner: uuid::Uuid) -> egui::Window<'_> {
    egui::Window::new(title)
        .id(egui::Id::new(owner))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 100.0))
        .fixed_size(egui::vec2(620.0, 430.0))
}

pub(super) fn query(
    ui: &mut egui::Ui,
    focused: &mut bool,
    query: &mut String,
    icon: super::super::icons::Icon,
    hint: &str,
    color: egui::Color32,
) -> egui::Response {
    ui.ctx().move_to_top(ui.layer_id());
    ui.horizontal(|ui| {
        icon.ui(ui, 14.0, color);
        let id = ui.make_persistent_id("query");
        if !*focused {
            ui.memory_mut(|memory| memory.request_focus(id));
        }
        let response = ui.add(
            egui::TextEdit::singleline(query)
                .id(id)
                .hint_text(hint)
                .desired_width(ui.available_width()),
        );
        *focused |= response.enabled() && response.has_focus();
        response
    })
    .inner
}

pub(super) fn body(ui: &mut egui::Ui, render: impl FnOnce(&mut egui::Ui)) {
    ui.separator();
    let footer = ui.spacing().interact_size.y + ui.spacing().item_spacing.y + 8.0;
    let height = (ui.available_height() - footer).max(40.0);
    ui.allocate_ui(egui::vec2(ui.available_width(), height), |ui| {
        ui.set_min_height(height);
        render(ui);
    });
}

pub(super) fn footer(
    ui: &mut egui::Ui,
    theme: &AppTheme,
    label: &str,
    verb: &str,
    selected: Option<ModalAction>,
    query: egui::Id,
) -> Option<ModalAction> {
    ui.separator();
    let mut action = None;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).small().color(theme.text_muted));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let enabled = selected.is_some();
            let primary = ui.add_enabled(enabled, egui::Button::new(verb));
            if primary.clicked() {
                action = selected;
            }
            let cancel = ui.button("Cancel");
            if cancel.clicked() {
                action = Some(ModalAction::Close);
            }
            focus_tabs(ui, query, enabled.then_some(primary.id), cancel.id);
        });
    });
    action
}

fn focus_tabs(ui: &egui::Ui, query: egui::Id, primary: Option<egui::Id>, cancel: egui::Id) {
    let stops = [query, primary.unwrap_or(cancel), cancel];
    focus_ring(ui, &stops[..if primary.is_some() { 3 } else { 2 }]);
}

pub(super) fn focus_ring(ui: &egui::Ui, stops: &[egui::Id]) {
    // App-modal keys are replayed after egui's native focus pass. Direct
    // hosts that supplied Tab in RawInput already received that pass.
    let native_tab = ui.input(|input| {
        input.raw.events.iter().any(|event| {
            matches!(
                event,
                Event::Key {
                    key: Key::Tab,
                    pressed: true,
                    ..
                }
            )
        })
    });
    if !native_tab {
        let tabs = ui.input_mut(|input| {
            commands::take_input_actions(&mut input.events, |event| match event {
                Event::Key {
                    key: Key::Tab,
                    pressed: true,
                    modifiers,
                    ..
                } if !modifiers.alt && !modifiers.ctrl && !modifiers.command => {
                    Some(modifiers.shift)
                }
                _ => None,
            })
        });
        for reverse in tabs {
            let index = ui
                .memory(|memory| memory.focused())
                .and_then(|id| stops.iter().position(|stop| *stop == id))
                .unwrap_or(0);
            let next = (index + if reverse { stops.len() - 1 } else { 1 }) % stops.len();
            ui.memory_mut(|memory| memory.request_focus(stops[next]));
        }
    }
}

pub(super) fn scroll(
    ui: &egui::Ui,
    id: egui::Id,
    selected: usize,
    reveal: &mut bool,
) -> egui::ScrollArea {
    let mut scroll = egui::ScrollArea::vertical()
        .id_source(id)
        .auto_shrink([false, false]);
    if *reveal {
        scroll = commands::reveal_list_row(ui, scroll, id, Some(selected), PickerRow::height(ui));
        *reveal = false;
    }
    scroll
}

fn control(event: &Event) -> Option<Key> {
    match event {
        Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } if modifiers.is_none()
            && matches!(
                key,
                Key::ArrowDown | Key::ArrowUp | Key::Enter | Key::Escape
            ) =>
        {
            Some(*key)
        }
        _ => None,
    }
}

/// Fixed-height, measured two-line row shared by file and symbol pickers.
pub(super) struct PickerRow<'a> {
    pub(super) title: &'a str,
    pub(super) detail: &'a str,
    pub(super) metadata: &'a str,
}

impl PickerRow<'_> {
    pub(super) fn height(ui: &egui::Ui) -> f32 {
        ui.text_style_height(&egui::TextStyle::Body)
            + ui.text_style_height(&egui::TextStyle::Small)
            + 10.0
    }

    pub(super) fn show(
        &self,
        ui: &mut egui::Ui,
        theme: &AppTheme,
        selected: bool,
        height: f32,
    ) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), height),
            egui::Sense::click(),
        );
        if selected || response.hovered() {
            ui.painter().rect_filled(
                rect,
                4.0,
                theme
                    .accent
                    .linear_multiply(if selected { 0.2 } else { 0.1 }),
            );
        }
        let text_rect = rect.shrink2(egui::vec2(8.0, 4.0));
        let secondary = if selected {
            theme.text_primary
        } else {
            theme.text_muted
        };
        let meta = text_fit::middle(
            ui,
            self.metadata,
            egui::TextStyle::Small.resolve(ui.style()),
            secondary,
            text_rect.width() * 0.4,
        );
        let reserved = if self.metadata.is_empty() {
            0.0
        } else {
            meta.size().x + 8.0
        };
        let title = text_fit::middle(
            ui,
            self.title,
            egui::TextStyle::Body.resolve(ui.style()),
            theme.text_primary,
            text_rect.width() - reserved,
        );
        let detail = text_fit::middle(
            ui,
            self.detail,
            egui::TextStyle::Small.resolve(ui.style()),
            secondary,
            text_rect.width(),
        );
        let top = text_rect.left_top();
        let lower = top + egui::vec2(0.0, title.size().y + 2.0);
        ui.painter().galley(
            egui::pos2(text_rect.right() - meta.size().x, top.y),
            meta,
            secondary,
        );
        ui.painter().galley(top, title, theme.text_primary);
        ui.painter().galley(lower, detail, secondary);
        response
    }
}
