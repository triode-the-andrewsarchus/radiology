//! Independent reference selection, disclosure, focus and variable-height geometry.

use crate::{
    semantic::disclosure::ReferenceDisclosure,
    semantic::reference_results::{ReferenceHit, ReferenceResults},
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
    /// Expands resource limits and queries more indexed sites.
    More,
    /// Expands the caller trace budget for one disclosed reference.
    MoreCallers(usize),
    /// Opens one validated caller site in a disclosed reference tree.
    CommitCaller(usize, usize),
}

/// Measured reference-list geometry, reusable without application navigation state.
#[derive(Default)]
pub struct ReferenceViewport {
    rows: RowHeights,
    expanded: BTreeMap<usize, (usize, usize)>,
    geometry: Option<(usize, f32, f32)>,
    groups: BTreeMap<usize, usize>,
    reveal: bool,
    focus_after_registration: bool,
    caller_controls: Vec<(Id, ReferenceAction)>,
    reveal_caller: Option<(usize, usize)>,
    pending_caller_focus: Option<Id>,
}

impl ReferenceViewport {
    pub(crate) fn show_coverage(
        &mut self,
        ui: &mut Ui,
        theme: &AppTheme,
        results: &ReferenceResults,
        loading: bool,
    ) -> Option<ReferenceAction> {
        if results.reference_count == 0 && !loading {
            ui.colored_label(theme.text_muted, "No references shown.");
        }
        if results.unmapped > 0 || !results.unreadable.is_empty() {
            ui.colored_label(theme.text_muted, format!("{} unmapped · {} unreadable files", results.unmapped, results.unreadable.len()))
            .on_hover_ui(|ui| {
                ui.label("Unmapped sites have stale source identities, unsupported encodings, or coordinates that do not fit the source. Reindex to refresh.");
                for file in results.unreadable.iter().take(16) { ui.label(format!("{} ({} sites): {}", file.path, file.sites, file.message)); }
            });
        }
        let mut action = None;
        if results.omitted > 0 {
            ui.horizontal(|ui| {
                ui.colored_label(
                    theme.text_muted,
                    format!("{} sites not loaded", results.omitted),
                );
                if ui
                    .add_enabled(!loading, egui::Button::new("Load More").small())
                    .clicked()
                {
                    self.focus_after_registration = true;
                    action = Some(ReferenceAction::More);
                }
            });
        }
        action
    }

    pub(crate) fn focus_id() -> Id {
        Id::new("reference_list_focus")
    }

    pub(crate) fn has_focus(&self, ctx: &Context) -> bool {
        ctx.memory(|memory| {
            memory.has_focus(Self::focus_id())
                || memory
                    .focused()
                    .is_some_and(|id| self.caller_controls.iter().any(|control| control.0 == id))
        })
    }

    pub(crate) fn reveal_selection(&mut self) {
        self.reveal = true;
    }

    pub(crate) fn focus_callers(&mut self, reference: usize) -> bool {
        let id = super::reference_callers::control_id(reference, Some(0));
        if self.caller_controls.iter().any(|control| control.0 == id) {
            self.pending_caller_focus = Some(id);
            self.reveal_caller = Some((reference, 0));
            true
        } else {
            false
        }
    }

    pub(crate) fn take_actions(&mut self, ctx: &Context) -> Vec<ReferenceAction> {
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

    fn configure_caller_controls(
        &mut self,
        ctx: &Context,
        disclosures: &BTreeMap<usize, ReferenceDisclosure>,
    ) {
        let mut controls = Vec::new();
        for (&reference, disclosure) in disclosures {
            if let Some(tree) = disclosure.callers() {
                controls.extend((0..tree.rows.len()).map(|caller| {
                    (
                        super::reference_callers::control_id(reference, Some(caller)),
                        ReferenceAction::CommitCaller(reference, caller),
                    )
                }));
                if tree.has_more {
                    controls.push((
                        super::reference_callers::control_id(reference, None),
                        ReferenceAction::MoreCallers(reference),
                    ));
                }
            }
        }
        let focused = ctx.memory(|memory| memory.focused());
        if focused.is_some_and(|id| {
            self.caller_controls.iter().any(|control| control.0 == id)
                && !controls.iter().any(|control| control.0 == id)
        }) {
            self.focus_after_registration = true;
        }
        self.caller_controls = controls;
    }

    pub(crate) fn take_raw_actions(
        &mut self,
        ctx: &Context,
        input: &mut egui::RawInput,
    ) -> Vec<ReferenceAction> {
        let focused = ctx.memory(|memory| memory.focused());
        let Some(mut index) = self
            .caller_controls
            .iter()
            .position(|control| Some(control.0) == focused)
        else {
            return Vec::new();
        };
        let mut focus_list = false;
        let mut reveal = false;
        let actions = super::commands::take_event_actions(&mut input.events, |key, modifiers| {
            if modifiers != Modifiers::NONE {
                return None;
            }
            let reference = match self.caller_controls[index].1 {
                ReferenceAction::CommitCaller(reference, _)
                | ReferenceAction::MoreCallers(reference) => reference,
                _ => unreachable!(),
            };
            match key {
                Key::Enter | Key::Space => Some(self.caller_controls[index].1.clone()),
                Key::ArrowUp | Key::ArrowDown | Key::Home | Key::End => {
                    index = if key == Key::Home {
                        0
                    } else if key == Key::End {
                        self.caller_controls.len() - 1
                    } else if key == Key::ArrowDown {
                        (index + 1).min(self.caller_controls.len() - 1)
                    } else {
                        index.saturating_sub(1)
                    };
                    reveal = true;
                    Some(ReferenceAction::Select(reference))
                }
                Key::ArrowLeft => {
                    focus_list = true;
                    Some(ReferenceAction::SetExpanded(reference, false))
                }
                Key::Escape => {
                    focus_list = true;
                    Some(ReferenceAction::Select(reference))
                }
                Key::ArrowRight => Some(ReferenceAction::Select(reference)),
                _ => None,
            }
        });
        if focus_list {
            ctx.memory_mut(|memory| memory.request_focus(Self::focus_id()));
        } else if reveal {
            let (id, action) = &self.caller_controls[index];
            let (reference, caller) = match *action {
                ReferenceAction::CommitCaller(reference, caller) => (reference, caller),
                ReferenceAction::MoreCallers(reference) => (reference, usize::MAX),
                _ => unreachable!(),
            };
            self.reveal_caller = Some((reference, caller));
            ctx.memory_mut(|memory| memory.request_focus(*id));
        }
        actions
    }

    pub(crate) fn show(
        &mut self,
        ui: &mut Ui,
        theme: &AppTheme,
        results: &ReferenceResults,
        selected: Option<usize>,
        previews: &BTreeMap<usize, ReferenceDisclosure>,
    ) -> Option<ReferenceAction> {
        self.configure_caller_controls(ui.ctx(), previews);
        ui.interact(
            ui.available_rect_before_wrap(),
            Self::focus_id(),
            Sense::focusable_noninteractive(),
        );
        // A press and release may arrive together. Registering the list would
        // otherwise surrender focus requested by the coverage button above it.
        if std::mem::take(&mut self.focus_after_registration) {
            ui.memory_mut(|memory| memory.request_focus(Self::focus_id()));
        }
        if let Some(id) = self.pending_caller_focus.take() {
            ui.memory_mut(|memory| memory.request_focus(id));
            super::commands::retain_navigation_keys(ui.ctx(), id);
        }
        super::commands::retain_navigation_keys(ui.ctx(), Self::focus_id());
        self.configure_rows(ui, results, previews);
        let nodes = &results.hits;
        let mut scroll = egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .id_source("declaration_references");
        if self.reveal {
            if let Some(row) = selected {
                scroll = scroll.vertical_scroll_offset(self.rows.offset(row));
            }
            self.reveal = false;
        }
        if let Some((reference, caller)) = self.reveal_caller.take() {
            if let Some(disclosure) = previews.get(&reference) {
                let offset = self.rows.offset(reference)
                    + 24.0
                    + 24.0 * f32::from(self.groups.contains_key(&reference))
                    + super::source_preview::height(disclosure.phase())
                    + super::reference_callers::HEADER_HEIGHT
                    + caller.min(disclosure.callers().map_or(0, |tree| tree.rows.len())) as f32
                        * super::reference_callers::ROW_HEIGHT;
                scroll = scroll.vertical_scroll_offset(
                    (offset - super::reference_callers::ROW_HEIGHT).max(0.0),
                );
            }
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
                if let Some(&group) = self.groups.get(&index) {
                    render_group(ui, theme, &results.groups[group]);
                }
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
                    if let Some(next) = super::reference_callers::show(
                        ui,
                        theme,
                        index,
                        preview.callers(),
                        preview.callers_loading(),
                        preview.caller_error(),
                    ) {
                        action = Some(next);
                    }
                }
                self.rows
                    .set_height(index, (ui.cursor().top() - top).max(24.0));
            }
        });
        action
    }
    fn configure_rows(
        &mut self,
        ui: &Ui,
        results: &ReferenceResults,
        previews: &BTreeMap<usize, ReferenceDisclosure>,
    ) {
        let geometry = (
            results.hits.len(),
            ui.available_width(),
            ui.ctx().pixels_per_point(),
        );
        self.rows.reset(geometry.0, geometry.1, 24.0, geometry.2);
        let groups: BTreeMap<_, _> = results
            .groups
            .iter()
            .enumerate()
            .map(|(group, file)| (file.hits.start, group))
            .collect();
        if self.geometry != Some(geometry) || self.groups != groups {
            for &row in self.groups.keys() {
                if row < geometry.0 {
                    self.rows.set_height(row, 24.0);
                }
            }
            self.expanded.clear();
            self.geometry = Some(geometry);
            self.groups = groups;
            for &row in self.groups.keys() {
                self.rows.set_height(row, 48.0);
            }
        }
        for &row in self
            .expanded
            .keys()
            .filter(|row| !previews.contains_key(row))
        {
            if row < geometry.0 {
                self.rows
                    .set_height(row, 24.0 + 24.0 * f32::from(self.groups.contains_key(&row)));
            }
        }
        self.expanded.retain(|row, _| previews.contains_key(row));
        for (&row, preview) in previews {
            let signature = match preview.phase() {
                crate::source::preview::PreviewPhase::None => 0,
                crate::source::preview::PreviewPhase::Loading => 1,
                crate::source::preview::PreviewPhase::Unavailable(_) => 2,
                crate::source::preview::PreviewPhase::Loaded(preview) => {
                    3 + preview.lines.len() * 2 + usize::from(preview.truncated)
                }
            };
            let caller_height =
                super::reference_callers::height(preview.callers(), preview.caller_error());
            let signature = (signature, caller_height as usize);
            if row < geometry.0 && self.expanded.get(&row) != Some(&signature) {
                self.rows.set_height(
                    row,
                    24.0 + 24.0 * f32::from(self.groups.contains_key(&row))
                        + super::source_preview::height(preview.phase())
                        + caller_height,
                );
            }
            self.expanded.insert(row, signature);
        }
    }
}

fn render_row(
    ui: &mut Ui,
    theme: &AppTheme,
    node: &ReferenceHit,
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
    child.label(
        RichText::new(format!("{:>3}", node.source_line.number))
            .monospace()
            .color(theme.text_muted),
    );
    let badge = if node.is_definition { 28.0 } else { 0.0 };
    let (text_rect, _) = child.allocate_exact_size(
        egui::vec2((child.available_width() - badge).max(1.0), 24.0),
        Sense::hover(),
    );
    paint_source_line(&child, theme, node, text_rect);
    if node.is_definition {
        child.colored_label(theme.text_muted, RichText::new("def").size(11.0));
    }
    let mut action = if disclosure.clicked() {
        Some(ReferenceAction::SetExpanded(index, !expanded))
    } else if response.double_clicked() {
        Some(ReferenceAction::Commit(Some(index)))
    } else if response.clicked() {
        Some(ReferenceAction::Select(index))
    } else {
        None
    };
    crate::ui::commands::context_menu(&response, |ui| {
        if ui.button("Go to Reference").clicked() {
            action = Some(ReferenceAction::Commit(Some(index)));
            ui.close_menu();
        }
    });
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::SelectableLabel,
            format!(
                "{} line {}{}: {}",
                node.path,
                node.source_line.number,
                if node.is_definition {
                    ", definition"
                } else {
                    ""
                },
                node.source_line.text
            ),
        )
    });
    response.on_hover_text(format!(
        "{}:{} — select; double-click or Return to open",
        node.path, node.source_line.number
    ));
    if action.is_some() {
        ui.memory_mut(|memory| memory.request_focus(ReferenceViewport::focus_id()));
    }
    action
}

fn paint_source_line(ui: &Ui, theme: &AppTheme, hit: &ReferenceHit, rect: egui::Rect) {
    let mut job = super::source_preview::line_job(&hit.source_line, theme);
    job.wrap.max_width = f32::INFINITY;
    job.break_on_newline = false;
    let galley = ui.fonts(|fonts| fonts.layout_job(job));
    let offset = hit.source_line.emphasis.as_ref().map_or(0.0, |emphasis| {
        let x = |byte| {
            let cursor = galley.from_ccursor(egui::text::CCursor::new(
                hit.source_line.text[..byte].chars().count(),
            ));
            galley.pos_from_cursor(&cursor).left()
        };
        if x(emphasis.end) <= rect.width() {
            0.0
        } else {
            (x(emphasis.start) - rect.width() * 0.25).max(0.0)
        }
    });
    let ellipsis =
        ui.painter()
            .layout_no_wrap("…".into(), egui::FontId::monospace(12.0), theme.text_muted);
    let left = if offset > 0.0 { ellipsis.size().x } else { 0.0 };
    let right = if galley.size().x - offset + left > rect.width() {
        ellipsis.size().x
    } else {
        0.0
    };
    let clip = egui::Rect::from_min_max(
        egui::pos2(rect.left() + left, rect.top()),
        egui::pos2(
            (rect.right() - right).max(rect.left() + left),
            rect.bottom(),
        ),
    );
    ui.painter()
        .with_clip_rect(clip.intersect(ui.clip_rect()))
        .galley(
            egui::pos2(
                rect.left() + left - offset,
                rect.center().y - galley.size().y / 2.0,
            ),
            galley,
            theme.text_primary,
        );
    let y = rect.center().y - ellipsis.size().y / 2.0;
    if left > 0.0 {
        ui.painter().galley(
            egui::pos2(rect.left(), y),
            ellipsis.clone(),
            theme.text_muted,
        );
    }
    if right > 0.0 {
        ui.painter().galley(
            egui::pos2(rect.right() - right, y),
            ellipsis,
            theme.text_muted,
        );
    }
}

fn render_group(
    ui: &mut Ui,
    theme: &AppTheme,
    group: &crate::semantic::reference_results::ReferenceGroup,
) {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 24.0), Sense::hover());
    let text = ui.painter().layout_no_wrap(
        group.path.clone(),
        egui::FontId::proportional(12.0),
        theme.text_secondary,
    );
    let count = ui.painter().layout_no_wrap(
        group.hits.len().to_string(),
        egui::FontId::monospace(11.0),
        theme.text_muted,
    );
    let path_rect = egui::Rect::from_min_max(
        rect.min,
        egui::pos2(rect.right() - count.size().x - 8.0, rect.bottom()),
    );
    let offset = (text.size().x - path_rect.width()).max(0.0);
    ui.painter()
        .with_clip_rect(path_rect.intersect(ui.clip_rect()))
        .galley(
            egui::pos2(
                path_rect.left() - offset,
                rect.center().y - text.size().y / 2.0,
            ),
            text,
            theme.text_secondary,
        );
    ui.painter().galley(
        egui::pos2(
            rect.right() - count.size().x,
            rect.center().y - count.size().y / 2.0,
        ),
        count,
        theme.text_muted,
    );
    response.on_hover_text(format!(
        "{} — {} indexed sites, including definitions",
        group.path,
        group.hits.len()
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caller_keys_preserve_order_and_return_focus_to_the_parent_list() {
        let ctx = Context::default();
        let mut viewport = ReferenceViewport::default();
        let ids = [
            super::super::reference_callers::control_id(4, Some(0)),
            super::super::reference_callers::control_id(4, Some(1)),
            super::super::reference_callers::control_id(4, Some(2)),
            super::super::reference_callers::control_id(4, Some(3)),
        ];
        viewport.caller_controls = ids
            .iter()
            .enumerate()
            .map(|(row, id)| (*id, ReferenceAction::CommitCaller(4, row)))
            .collect();
        let mut frame = |keys: &[Key]| {
            let mut input = egui::RawInput {
                events: keys
                    .iter()
                    .map(|&key| Event::Key {
                        key,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: Modifiers::NONE,
                    })
                    .collect(),
                ..Default::default()
            };
            let actions = viewport.take_raw_actions(&ctx, &mut input);
            let _ = ctx.run(input, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.interact(
                        ui.available_rect_before_wrap(),
                        ReferenceViewport::focus_id(),
                        Sense::focusable_noninteractive(),
                    );
                    for id in ids {
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(200.0, 44.0), Sense::hover());
                        ui.interact(rect, id, Sense::click());
                        super::super::commands::retain_navigation_keys(ui.ctx(), id);
                    }
                });
            });
            actions
        };
        frame(&[]);
        ctx.memory_mut(|memory| memory.request_focus(ids[0]));
        frame(&[]);
        assert_eq!(frame(&[Key::ArrowDown]), [ReferenceAction::Select(4)]);
        assert!(ctx.memory(|memory| memory.has_focus(ids[1])));
        assert_eq!(
            frame(&[Key::ArrowDown, Key::Enter]),
            [
                ReferenceAction::Select(4),
                ReferenceAction::CommitCaller(4, 2)
            ]
        );
        assert!(ctx.memory(|memory| memory.has_focus(ids[2])));
        assert_eq!(
            frame(&[Key::ArrowLeft]),
            [ReferenceAction::SetExpanded(4, false)]
        );
        assert!(ctx.memory(|memory| memory.has_focus(ReferenceViewport::focus_id())));
    }
    use egui::Event;
    fn test_hit(index: usize) -> ReferenceHit {
        ReferenceHit {
            path: "a.rs".into(),
            range: crate::semantic::ScipRange::from_packed(&[index as i32, 0, 1]).unwrap(),
            byte_range: index..index + 1,
            source_line: crate::source::preview::PreviewLine {
                number: index + 1,
                text: format!("caller{index}"),
                emphasis: Some(0..1),
                clipped: false,
            },
            is_definition: false,
        }
    }
    fn test_results(hits: Vec<ReferenceHit>) -> ReferenceResults {
        ReferenceResults {
            groups: vec![crate::semantic::reference_results::ReferenceGroup {
                path: "a.rs".into(),
                hits: 0..hits.len(),
                content_digest: String::new(),
            }],
            reference_count: hits.len(),
            hits,
            ..Default::default()
        }
    }

    #[test]
    fn single_click_selects_and_double_click_commits() {
        let context = egui::Context::default();
        let range = crate::semantic::ScipRange::from_packed(&[1, 7, 13]).unwrap();
        let node = ReferenceHit {
            path: "b.ts".into(),
            range: range.clone(),
            byte_range: 0..6,
            source_line: crate::source::preview::PreviewLine {
                number: 2,
                text: "caller".into(),
                emphasis: Some(0..6),
                clipped: false,
            },
            is_definition: false,
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
        let nodes: Vec<_> = (0..1000).map(test_hit).collect();
        let results = test_results(nodes);
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
                    viewport.show(ui, &AppTheme::dark(), &results, Some(999), &BTreeMap::new());
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
        let results = test_results(vec![test_hit(0)]);
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
                    actions = viewport.take_actions(ctx);
                    egui::CentralPanel::default().show(ctx, |ui| {
                        viewport.show(ui, &AppTheme::dark(), &results, Some(0), &BTreeMap::new());
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
    #[test]
    fn narrow_source_rows_keep_the_indexed_match_visible_and_mark_cropped_text() {
        let ctx = Context::default();
        let mut hit = test_hit(0);
        hit.source_line.text = format!("{}target();{}", "prefix ".repeat(40), " tail".repeat(40));
        let start = hit.source_line.text.find("target").unwrap();
        hit.source_line.emphasis = Some(start..start + 6);
        hit.byte_range = start..start + 6;
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(220.0, 160.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    render_row(ui, &AppTheme::dark(), &hit, 0, false, false);
                });
            },
        );
        let source = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.job.text == hit.source_line.text => {
                    Some((shape.clip_rect, text))
                }
                _ => None,
            })
            .unwrap();
        for position in [start, start + 6] {
            let cursor = source
                .1
                .galley
                .from_ccursor(egui::text::CCursor::new(position));
            let point = source.1.pos
                + source
                    .1
                    .galley
                    .pos_from_cursor(&cursor)
                    .left_center()
                    .to_vec2();
            assert!(
                source.0.contains(point),
                "indexed match is outside the row clip"
            );
        }
        assert!(output.shapes.iter().any(
            |shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == "…")
        ));
    }

    #[test]
    fn load_more_returns_focus_to_the_reference_list() {
        let ctx = Context::default();
        let mut results = test_results(vec![test_hit(0)]);
        results.omitted = 2;
        let mut viewport = ReferenceViewport::default();
        let mut render = |events, complete: bool| {
            let mut results = results.clone();
            if complete {
                results.omitted = 0;
            }
            let mut action = None;
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(320.0, 300.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        action = viewport.show_coverage(ui, &AppTheme::dark(), &results, false);
                        viewport.show(ui, &AppTheme::dark(), &results, Some(0), &BTreeMap::new());
                    });
                },
            );
            (output, action)
        };
        let (initial, _) = render(vec![], false);
        let position = initial
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.job.text == "Load More" => {
                    Some(text.pos + egui::vec2(10.0, 7.0))
                }
                _ => None,
            })
            .unwrap();
        let (_, clicked) = render(
            vec![
                Event::PointerMoved(position),
                Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
                Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                },
            ],
            false,
        );
        assert_eq!(clicked, Some(ReferenceAction::More));
        for _ in 0..3 {
            render(vec![], true);
        }
        assert!(ctx.memory(|memory| memory.has_focus(ReferenceViewport::focus_id())));
    }
}
