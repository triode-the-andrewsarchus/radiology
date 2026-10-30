//! Independent reference selection, disclosure, focus and variable-height geometry.

use crate::{
    semantic::caller_tree::{caller_pill_color, ReferenceCallerNode},
    source::preview::SourcePreviewLoader,
    theme::AppTheme,
};
use egui::{Context, Id, Key, Modifiers, RichText, Sense, Ui};
use reader_text::RowHeights;
use std::collections::BTreeMap;

/// Reference interactions never move the reader until an explicit commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceAction {
    /// Selects a reference row without navigating.
    Select(usize),
    /// Shows or hides source context for one row.
    SetExpanded(usize, bool),
    /// Selects the next or previous row, clamping at the list ends.
    MoveSelection(bool),
    /// Shows or hides source context for the selected row.
    SetSelectionExpanded(bool),
    /// Opens the given row, or the selected row when no index is supplied.
    Commit(Option<usize>),
    /// Clears list selection without moving the source caret.
    ClearSelection,
}

/// Measured reference-list geometry, reusable without application navigation state.
#[derive(Default)]
pub struct ReferenceViewport {
    rows: RowHeights,
    expanded: BTreeMap<usize, usize>,
    geometry: Option<(f32, f32)>,
    reveal: bool,
}

impl ReferenceViewport {
    pub(crate) fn focus_id() -> Id {
        Id::new("reference_list_focus")
    }

    pub(crate) fn reveal_selection(&mut self) {
        self.reveal = true;
    }

    pub(crate) fn take_actions(ctx: &Context) -> Vec<ReferenceAction> {
        if !ctx.memory(|memory| memory.has_focus(Self::focus_id())) {
            return Vec::new();
        }
        super::commands::take_key_actions(ctx, |key, modifiers| {
            if modifiers != Modifiers::NONE {
                return None;
            }
            Some(match key {
                Key::ArrowDown => ReferenceAction::MoveSelection(true),
                Key::ArrowUp => ReferenceAction::MoveSelection(false),
                Key::ArrowRight => ReferenceAction::SetSelectionExpanded(true),
                Key::ArrowLeft => ReferenceAction::SetSelectionExpanded(false),
                Key::Enter => ReferenceAction::Commit(None),
                Key::Escape => ReferenceAction::ClearSelection,
                _ => return None,
            })
        })
    }

    pub(crate) fn show(
        &mut self,
        ui: &mut Ui,
        theme: &AppTheme,
        nodes: &[ReferenceCallerNode],
        selected: Option<usize>,
        previews: &BTreeMap<usize, SourcePreviewLoader>,
    ) -> Option<ReferenceAction> {
        ui.interact(
            ui.available_rect_before_wrap(),
            Self::focus_id(),
            Sense::focusable_noninteractive(),
        );
        ui.memory_mut(|memory| {
            memory.set_focus_lock_filter(
                Self::focus_id(),
                egui::EventFilter {
                    horizontal_arrows: true,
                    vertical_arrows: true,
                    escape: true,
                    ..Default::default()
                },
            )
        });
        let geometry = (ui.available_width(), ui.ctx().pixels_per_point());
        self.rows.reset(nodes.len(), geometry.0, 24.0, geometry.1);
        if self.geometry != Some(geometry) {
            self.expanded.clear();
            self.geometry = Some(geometry);
        }
        for &index in self
            .expanded
            .keys()
            .filter(|index| !previews.contains_key(index))
        {
            if index < nodes.len() {
                self.rows.set_height(index, 24.0);
            }
        }
        self.expanded
            .retain(|index, _| previews.contains_key(index));
        for (&index, preview) in previews {
            let signature = match preview.phase() {
                crate::source::preview::PreviewPhase::None => 0,
                crate::source::preview::PreviewPhase::Loading => 1,
                crate::source::preview::PreviewPhase::Unavailable(_) => 2,
                crate::source::preview::PreviewPhase::Loaded(preview) => {
                    3 + preview.lines.len() * 2 + usize::from(preview.truncated)
                }
            };
            if index < nodes.len() && self.expanded.get(&index) != Some(&signature) {
                self.rows
                    .set_height(index, 24.0 + super::source_preview::height(preview.phase()));
            }
            self.expanded.insert(index, signature);
        }
        let mut scroll = egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .id_source("declaration_references");
        if self.reveal {
            if let Some(row) = selected {
                scroll = scroll.vertical_scroll_offset(self.rows.offset(row));
            }
            self.reveal = false;
        }
        let mut action = None;
        scroll.show_viewport(ui, |ui, visible| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.set_min_height(self.rows.total_height());
            let first = self.rows.row_at(visible.top());
            let last = (self.rows.row_at(visible.bottom()) + 1).min(nodes.len());
            ui.add_space(self.rows.offset(first));
            for (index, node) in nodes.iter().enumerate().take(last).skip(first) {
                let top = ui.cursor().top();
                if let Some(next) = render_row(
                    ui,
                    theme,
                    node,
                    index,
                    selected == Some(index),
                    previews.contains_key(&index),
                ) {
                    action = Some(next);
                }
                if let Some(preview) = previews.get(&index) {
                    super::source_preview::show(ui, theme, preview.phase());
                }
                self.rows
                    .set_height(index, (ui.cursor().top() - top).max(24.0));
            }
        });
        action
    }
}

fn render_row(
    ui: &mut Ui,
    theme: &AppTheme,
    node: &ReferenceCallerNode,
    index: usize,
    selected: bool,
    expanded: bool,
) -> Option<ReferenceAction> {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 24.0), Sense::click());
    if selected || response.hovered() {
        ui.painter().rect_filled(
            rect,
            3.0,
            theme
                .accent
                .linear_multiply(if selected { 0.18 } else { 0.08 }),
        );
    }
    let mut child = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
    let chevron = if expanded {
        super::icons::Icon::ChevronDown
    } else {
        super::icons::Icon::ChevronRight
    };
    let disclosure = chevron
        .button(&mut child, 10.0, theme.text_secondary)
        .on_hover_text("Right shows source; Left hides it. Return opens the reference.");
    let color = caller_pill_color(&node.name, &node.path);
    egui::Frame::none()
        .fill(color.linear_multiply(0.2))
        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
        .rounding(10.0)
        .show(&mut child, |ui| {
            ui.add(
                egui::Label::new(RichText::new(&node.name).color(color).size(11.0)).truncate(true),
            );
        });
    child.add(
        egui::Label::new(
            RichText::new(format!("{}:{}", node.path, node.line))
                .color(theme.text_secondary)
                .size(11.0),
        )
        .truncate(true),
    );
    let action = if disclosure.clicked() {
        Some(ReferenceAction::SetExpanded(index, !expanded))
    } else if response.double_clicked() {
        Some(ReferenceAction::Commit(Some(index)))
    } else if response.clicked() {
        Some(ReferenceAction::Select(index))
    } else {
        None
    };
    response.on_hover_text(format!(
        "{}:{} — select; double-click or Return to open",
        node.path, node.line
    ));
    if action.is_some() {
        ui.memory_mut(|memory| memory.request_focus(ReferenceViewport::focus_id()));
    }
    action
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Event;
    #[test]
    fn single_click_selects_and_double_click_commits() {
        let context = egui::Context::default();
        let range = crate::semantic::ScipRange::from_packed(&[1, 7, 13]).unwrap();
        let node = ReferenceCallerNode {
            name: "caller".into(),
            path: "b.ts".into(),
            line: 2,
            range: range.clone(),
            depth: 0,
            color_slot: 0,
            is_recursive: false,
            children: vec![],
        };
        let mut action = None;
        let mut position = egui::Pos2::ZERO;
        let frame = |events| egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(320.0, 300.0),
            )),
            events,
            ..Default::default()
        };
        let _ = context.run(frame(vec![]), |context| {
            egui::CentralPanel::default().show(context, |ui| {
                position = ui.next_widget_position() + egui::vec2(120.0, 12.0);
                action = render_row(ui, &AppTheme::dark(), &node, 0, false, false);
            });
        });
        for expected in [ReferenceAction::Select(0), ReferenceAction::Commit(Some(0))] {
            for pressed in [true, false] {
                let _ = context.run(
                    frame(vec![
                        egui::Event::PointerMoved(position),
                        egui::Event::PointerButton {
                            pos: position,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: Default::default(),
                        },
                    ]),
                    |context| {
                        egui::CentralPanel::default().show(context, |ui| {
                            action = render_row(ui, &AppTheme::dark(), &node, 0, false, false);
                        });
                    },
                );
            }
            assert_eq!(action, Some(expected));
        }
    }
    #[test]
    fn keyboard_reveal_virtualizes_a_large_reference_list() {
        let ctx = Context::default();
        let nodes: Vec<_> = (0..1000)
            .map(|index| ReferenceCallerNode {
                name: format!("caller{index}"),
                path: "a.rs".into(),
                line: index + 1,
                range: crate::semantic::ScipRange::from_packed(&[index as i32, 0, 1]).unwrap(),
                depth: 0,
                color_slot: 0,
                is_recursive: false,
                children: vec![],
            })
            .collect();
        let mut viewport = ReferenceViewport::default();
        let frame = || egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(320.0, 300.0),
            )),
            ..Default::default()
        };
        let render = |viewport: &mut ReferenceViewport| {
            ctx.run(frame(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    viewport.show(ui, &AppTheme::dark(), &nodes, Some(999), &BTreeMap::new());
                });
            })
        };
        let initial = render(&mut viewport);
        assert!(initial.shapes.len() < 200, "offscreen rows were painted");
        viewport.reveal_selection();
        let mut last = None;
        for _ in 0..3 {
            last = Some(render(&mut viewport));
        }
        let output = last.unwrap();
        assert!(output.shapes.len() < 200);
        assert!(output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == "caller999")), "keyboard selection stayed outside the visible viewport");
    }
    #[test]
    fn reference_focus_keeps_arrows_and_escape_across_frames() {
        let ctx = Context::default();
        let node = ReferenceCallerNode {
            name: "caller".into(),
            path: "a.rs".into(),
            line: 1,
            range: crate::semantic::ScipRange::from_packed(&[0, 0, 1]).unwrap(),
            depth: 0,
            color_slot: 0,
            is_recursive: false,
            children: vec![],
        };
        let mut viewport = ReferenceViewport::default();
        let mut frame = |key: Option<Key>| {
            let mut actions = Vec::new();
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(320.0, 300.0),
                    )),
                    events: key
                        .map(|key| Event::Key {
                            key,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: Modifiers::NONE,
                        })
                        .into_iter()
                        .collect(),
                    ..Default::default()
                },
                |ctx| {
                    actions = ReferenceViewport::take_actions(ctx);
                    egui::CentralPanel::default().show(ctx, |ui| {
                        viewport.show(
                            ui,
                            &AppTheme::dark(),
                            std::slice::from_ref(&node),
                            Some(0),
                            &BTreeMap::new(),
                        );
                    });
                },
            );
            actions
        };
        frame(None);
        ctx.memory_mut(|memory| memory.request_focus(ReferenceViewport::focus_id()));
        frame(None);
        for (key, expected) in [
            (Key::ArrowRight, ReferenceAction::SetSelectionExpanded(true)),
            (Key::ArrowLeft, ReferenceAction::SetSelectionExpanded(false)),
            (Key::ArrowDown, ReferenceAction::MoveSelection(true)),
            (Key::ArrowUp, ReferenceAction::MoveSelection(false)),
            (Key::Escape, ReferenceAction::ClearSelection),
        ] {
            assert_eq!(frame(Some(key)), [expected]);
            assert!(ctx.memory(|memory| memory.has_focus(ReferenceViewport::focus_id())));
        }
    }
}
