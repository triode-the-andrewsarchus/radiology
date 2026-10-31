//! Measured source viewport. Coordinates always refer to original UTF-8 bytes.

use super::{for_each_match_rect, paint_match, SourceView, SourceViewAction};
use crate::{
    critical::lens::SourceMode,
    source::{selection::OccurrencePinStore, ReviewRevision, SourceBuffer},
    syntax::{
        folds::{FoldRange, FoldStore},
        highlighter::HighlightedSpan,
    },
    theme::AppTheme,
    ui::{icons::Icon, source_find::SourceFindPanel},
};
use egui::text::{LayoutJob, TextFormat};
use egui::{pos2, vec2, Color32, FontId, Id, Key, Modifiers, Rect, Sense, Stroke, Ui};
use reader_text::{
    projected_fragments, ByteSelection, ProjectedFragment as SourceFragment, RowHeights,
};
use std::{collections::HashSet, ops::Range, sync::Arc};

const FRAGMENT_BYTES: usize = 4096;
const TEXT_INSET: f32 = 12.0;

/// Per-view geometry and canonical selection, separate from immutable source.
#[derive(Default)]
pub struct SourceViewport {
    document: Option<(String, Option<ReviewRevision>, u64)>,
    fragments: Vec<SourceFragment>,
    visible: Vec<usize>,
    folded: HashSet<Range<usize>>,
    rows: RowHeights,
    scroll_y: f32,
    scroll_x: f32,
    anchor_after_reset: Option<usize>,
    selection: Option<ByteSelection>,
    dragging: bool,
    max_bytes: usize,
    test_generation: u64,
    hover: crate::critical::hover::CriticalHover,
    hover_popup: crate::ui::critical_hover::HoverPopup,
    hover_pointer: Option<usize>,
    hover_origin: egui::Pos2,
    preview: Option<(String, String, usize)>,
    restore: Option<(String, String, usize)>,
    destination: super::destination::DestinationFeedback,
    destination_frame: Option<super::destination::FeedbackFrame>,
    occurrence_clicks: Option<(egui::Pos2, f64, u8)>,
}

pub(super) struct CanvasContent<'a> {
    pub theme: &'a AppTheme,
    pub test_code: &'a crate::source::test_code::TestCodePresentation,
    pub mode: SourceMode,
    pub mark_context: Option<crate::critical::menu::MarkContext<'a>>,
    pub projection: Option<&'a crate::critical::SourceProjection>,
    pub annotations: Option<&'a crate::critical::annotations::SourceAnnotations>,
    pub hover_inputs: Option<Arc<crate::critical::worker::GraphInputs>>,
    pub buffer: &'a SourceBuffer,
    pub highlights: &'a [Vec<HighlightedSpan>],
    pub function_shade: Option<&'a Range<usize>>,
    pub caret_line: &'a mut usize,
    pub caret_col: &'a mut usize,
    pub folds: &'a [FoldRange],
    pub fold_store: &'a mut FoldStore,
    pub changes: &'a [Range<usize>],
    pub pins: &'a OccurrencePinStore,
    pub wrap: bool,
    pub font_size: f32,
    pub target: &'a mut Option<usize>,
    pub keyboard_enabled: bool,
    pub find: &'a mut SourceFindPanel,
}

impl CanvasContent<'_> {
    fn definition_at_byte(&self, byte: usize) -> Option<crate::semantic::index::SemanticSymbolID> {
        let index = self.mark_context?.index;
        let target =
            crate::semantic::references::ReferenceTarget::at_byte(index, self.buffer, byte)?;
        index.find_definition_for_id(&target.id)?;
        Some(target.id)
    }
}

struct CanvasGeometry {
    font: FontId,
    minimum_height: f32,
    width: f32,
    height: f32,
    surface: Rect,
    code_rect: Rect,
    tolerance: f32,
}

impl CanvasGeometry {
    fn new(ui: &Ui, content: &CanvasContent<'_>) -> Self {
        let font_size = if content.font_size.is_finite() {
            content.font_size.clamp(8.0, 32.0)
        } else {
            14.0
        };
        let font = FontId::monospace(font_size);
        let minimum_height = ui.fonts(|fonts| fonts.row_height(&font)) + 4.0;
        let height = (ui.available_height() - 28.0).max(40.0);
        let width = if content.wrap {
            (ui.available_width()
                - SourceView::GUTTER_WIDTH
                - TEXT_INSET * 2.0
                - ui.spacing().scroll.allocated_width())
            .max(font_size)
        } else {
            f32::INFINITY
        };
        let surface = Rect::from_min_size(
            ui.available_rect_before_wrap().min,
            vec2(ui.available_width(), height),
        );
        let code_rect = Rect::from_min_max(
            pos2(surface.left() + SourceView::GUTTER_WIDTH, surface.top()),
            surface.max,
        );
        Self {
            font,
            minimum_height,
            width,
            height,
            surface,
            code_rect,
            tolerance: 0.5 / ui.ctx().pixels_per_point(),
        }
    }
}

impl SourceViewport {
    fn focus_id() -> Id {
        Id::new("radiology_source_canvas")
    }

    /// Returns whether source, rather than another control, owns keyboard input.
    pub fn has_focus(&self, ctx: &egui::Context) -> bool {
        ctx.memory(|memory| memory.has_focus(Self::focus_id()))
    }

    /// Briefly highlights a successfully resolved destination without changing selection.
    /// The hold/fade clock begins when these exact source bytes are painted.
    pub fn flash_destination(&mut self, buffer: &SourceBuffer, byte: usize) {
        self.destination.request(buffer, byte);
        self.destination_frame = None;
    }

    /// Retires pending and visible landing feedback at a host navigation boundary.
    pub fn cancel_destination(&mut self) {
        self.restore = None;
        self.destination.cancel();
        self.destination_frame = None;
    }

    pub(crate) fn top_byte(&self, buffer: &SourceBuffer) -> Option<usize> {
        if self.document.as_ref()
            != Some(&(
                buffer.path.clone(),
                buffer.revision,
                buffer.content_identity(),
            ))
        {
            return None;
        }
        self.visible
            .get(self.rows.row_at(self.scroll_y))
            .map(|&index| self.fragments[index].bytes.start)
    }

    pub(crate) fn restore_top(&mut self, buffer: &SourceBuffer, byte: usize) {
        self.restore = Some((
            buffer.position_key(),
            buffer.content_digest().to_owned(),
            buffer.clamp_to_char_boundary(byte.min(buffer.data.len())),
        ));
    }

    /// Replaces the selection with original-byte endpoints (for local find).
    pub fn select(&mut self, range: Range<usize>) {
        self.selection = Some(ByteSelection {
            anchor: range.start,
            focus: range.end,
        });
    }

    /// Reveals exact source bytes without moving the caret or replacing selection.
    /// A queued reveal is discarded if the displayed document or bytes change.
    pub fn preview_byte(&mut self, buffer: &SourceBuffer, byte: usize) {
        self.cancel_destination();
        self.preview = Some((
            buffer.position_key(),
            buffer.content_digest().to_owned(),
            buffer.clamp_to_char_boundary(byte.min(buffer.data.len())),
        ));
        self.hover.dismiss();
    }

    fn sync(&mut self, content: &CanvasContent<'_>) {
        let source = content.buffer;
        let key = (
            source.path.clone(),
            source.revision,
            source.content_identity(),
        );
        let changed = self.document.as_ref() != Some(&key);
        let old_byte = self
            .visible
            .get(self.rows.row_at(self.scroll_y))
            .map_or(0, |&index| self.fragments[index].bytes.start);
        let restore = self.restore.take().filter(|(key, digest, _)| {
            key == &source.position_key() && digest == source.content_digest()
        });
        let projected = restore.is_some()
            || changed
            || self.test_generation != content.test_code.generation()
            || self.folded != content.fold_store.folded_ranges;
        if projected {
            let hidden: Vec<_> = content
                .folds
                .iter()
                .filter(|fold| content.fold_store.is_folded(&fold.byte_range))
                .map(|fold| fold.hidden_range(source))
                .chain(content.test_code.hidden_ranges().iter().cloned())
                .collect();
            self.fragments = projected_fragments(&source.text, &hidden, FRAGMENT_BYTES);
            self.max_bytes = self
                .fragments
                .iter()
                .map(SourceFragment::visible_len)
                .max()
                .unwrap_or(0);
            self.test_generation = content.test_code.generation();
            self.visible = (0..self.fragments.len()).collect();
            self.anchor_after_reset = Some(
                self.visible
                    .partition_point(|&index| {
                        self.fragments[index].bytes.start
                            <= restore
                                .as_ref()
                                .map_or(if changed { 0 } else { old_byte }, |(_, _, byte)| *byte)
                    })
                    .saturating_sub(1),
            );
            self.folded.clone_from(&content.fold_store.folded_ranges);
            self.rows = RowHeights::default();
        }
        if changed {
            self.occurrence_clicks = None;
            self.selection = None;
            self.dragging = false;
            self.scroll_y = 0.0;
            self.scroll_x = 0.0;
            self.document = Some(key);
        }
    }

    pub(super) fn show(
        &mut self,
        ui: &mut Ui,
        mut content: CanvasContent<'_>,
    ) -> Option<SourceViewAction> {
        let caret_before = (*content.caret_line, *content.caret_col);
        self.configure_hover(&content);
        self.hover_pointer = None;
        self.sync(&content);
        self.destination_frame = self.destination.frame(
            content.buffer,
            ui.input(|input| input.time),
            ui.style().animation_time > 0.0,
        );
        if let Some(frame) = self.destination_frame {
            ui.ctx().request_repaint_after(frame.next_frame);
        }
        let geometry = CanvasGeometry::new(ui, &content);
        if ui.input(|input| {
            input.pointer.primary_pressed()
                && input
                    .pointer
                    .interact_pos()
                    .is_none_or(|point| !geometry.code_rect.contains(point))
        }) {
            self.occurrence_clicks = None;
        }
        let (scroll_y, mut action) = self.prepare(ui, &mut content, &geometry);
        ui.interact(
            geometry.code_rect,
            Self::focus_id(),
            Sense::focusable_noninteractive(),
        );
        crate::ui::commands::retain_navigation_keys(ui.ctx(), Self::focus_id());
        let output = egui::ScrollArea::new([!content.wrap, true])
            .id_source((
                "source_scroll",
                &content.buffer.path,
                content.buffer.revision,
            ))
            .auto_shrink([false, false])
            .max_height(geometry.height)
            .vertical_scroll_offset(scroll_y)
            .horizontal_scroll_offset(if content.wrap { 0.0 } else { self.scroll_x })
            .drag_to_scroll(false)
            .show_viewport(ui, |ui, viewport| {
                if let Some(event) = self.render_rows(ui, viewport, &geometry, &mut content) {
                    action = Some(event);
                }
            });
        self.scroll_y = output.state.offset.y;
        self.scroll_x = output.state.offset.x;
        self.finish_drag(ui, geometry.code_rect);
        if let Some(event) = self.show_hover(ui, content.theme) {
            action = Some(event);
        }
        if caret_before != (*content.caret_line, *content.caret_col) {
            // The host derives breadcrumbs from the published caret next frame.
            ui.ctx().request_repaint();
        }
        action
    }

    fn configure_hover(&mut self, content: &CanvasContent<'_>) {
        let hover_inputs = content.hover_inputs.clone().filter(|inputs| {
            content.mode == SourceMode::Critical
                && content.keyboard_enabled
                && inputs.buffer.path == content.buffer.path
                && inputs.buffer.revision == content.buffer.revision
                && inputs.buffer.content_digest() == content.buffer.content_digest()
        });
        self.hover.configure(hover_inputs);
    }

    fn show_hover(&mut self, ui: &Ui, theme: &AppTheme) -> Option<SourceViewAction> {
        let popup = self
            .hover_popup
            .show(ui.ctx(), &self.hover, self.hover_origin, theme);
        let interrupted = ui.input(|input| {
            (input.pointer.any_pressed() && !popup.hovered)
                || input.raw_scroll_delta != egui::Vec2::ZERO
                || input.events.iter().any(|event| {
                    matches!(
                        event,
                        egui::Event::Key { pressed: true, .. } | egui::Event::Text(_)
                    )
                })
        });
        let now = ui.input(|input| input.time);
        if self
            .hover
            .update(self.hover_pointer, popup.hovered, interrupted, now)
        {
            ui.ctx().request_repaint();
        }
        if let Some(destination) = popup.destination {
            self.hover.dismiss();
            return Some(SourceViewAction::NavigateToDefinition(destination));
        }
        if let Some(delay) = self.hover.next_wake(now) {
            ui.ctx().request_repaint_after(delay);
        }
        None
    }

    fn reveal_target(&mut self, content: &mut CanvasContent<'_>) -> Option<(usize, bool)> {
        let preview = self.preview.take().filter(|(key, digest, _)| {
            key == &content.buffer.position_key() && digest == content.buffer.content_digest()
        });
        content
            .target
            .take()
            .map(|line| {
                let caret = content
                    .buffer
                    .byte_offset(*content.caret_line, *content.caret_col);
                let byte = if content.buffer.line_at_byte(caret) == line {
                    caret
                } else {
                    content.buffer.byte_offset(line, 0)
                };
                (byte, true)
            })
            .or_else(|| preview.map(|(_, _, byte)| (byte, false)))
    }

    fn prepare(
        &mut self,
        ui: &Ui,
        content: &mut CanvasContent<'_>,
        geometry: &CanvasGeometry,
    ) -> (f32, Option<SourceViewAction>) {
        let mut action = None;
        let tolerance = 0.5 / ui.ctx().pixels_per_point();
        let (anchor, inset) = self.anchor_after_reset.take().map_or_else(
            || {
                let anchor = self.rows.row_at(self.scroll_y + tolerance);
                (anchor, (self.scroll_y - self.rows.offset(anchor)).max(0.0))
            },
            |anchor| (anchor, 0.0),
        );
        self.rows.reset(
            self.visible.len(),
            geometry.width,
            geometry.minimum_height,
            ui.ctx().pixels_per_point(),
        );
        self.measure(
            ui,
            content,
            &geometry.font,
            geometry.width,
            geometry.minimum_height,
        );
        let mut scroll_y = self.rows.offset(anchor) + inset;
        let mut keyboard_reveal = false;
        let focused = self.has_focus(ui.ctx()) || !ui.ctx().wants_keyboard_input();
        if content.keyboard_enabled && focused {
            if let Some(next) =
                self.keyboard(ui, content, &geometry.font, geometry.width, geometry.height)
            {
                action = Some(next);
                keyboard_reveal = true;
            }
        }
        let target = self.reveal_target(content);
        if let Some((byte, committed)) = target {
            let rect = self.caret_rect(
                ui,
                content,
                byte,
                &geometry.font,
                geometry.width,
                geometry.minimum_height,
            );
            scroll_y = if keyboard_reveal {
                scroll_y.clamp(
                    (rect.bottom() - geometry.height).max(0.0),
                    rect.top().max(0.0),
                )
            } else {
                (rect.center().y - geometry.height / 2.0).max(0.0)
            };
            if !content.wrap {
                let available =
                    (ui.available_width() - SourceView::GUTTER_WIDTH - TEXT_INSET * 2.0).max(1.0);
                self.scroll_x = self
                    .scroll_x
                    .clamp((rect.right() - available).max(0.0), rect.left().max(0.0));
            }
            // A navigation destination retires an unrelated selection, including a
            // stale match. Find calls select after document synchronization below.
            let caret = content
                .buffer
                .byte_offset(*content.caret_line, *content.caret_col);
            if committed
                && self
                    .selection
                    .is_some_and(|selection| selection.focus != caret && selection.anchor != caret)
            {
                self.selection = None;
            }
        }
        if content.find.take_selection_request() {
            if let Some(range) = &content.find.active_range {
                self.select(range.clone());
            }
        }

        (scroll_y, action)
    }

    fn render_rows(
        &mut self,
        ui: &mut Ui,
        viewport: Rect,
        geometry: &CanvasGeometry,
        content: &mut CanvasContent<'_>,
    ) -> Option<SourceViewAction> {
        let mut action = None;
        let origin = ui.max_rect().min;
        let clip = ui.clip_rect().intersect(geometry.code_rect);
        let painter = ui.painter().with_clip_rect(clip);
        let gutter = ui
            .painter()
            .with_clip_rect(ui.clip_rect().intersect(geometry.surface));
        gutter.rect_filled(
            Rect::from_min_max(
                geometry.surface.min,
                pos2(geometry.code_rect.left(), geometry.surface.bottom()),
            ),
            0.0,
            content.theme.bg_gutter,
        );
        gutter.line_segment(
            [
                geometry.code_rect.left_top(),
                geometry.code_rect.left_bottom(),
            ],
            Stroke::new(1.0_f32, content.theme.border),
        );
        let mut row = self.rows.row_at(viewport.top() + geometry.tolerance);
        let mut y = self.rows.offset(row);
        let mut full_width = ui.available_width();
        while row < self.visible.len() && y <= viewport.bottom() {
            let fragment = self.fragments[self.visible[row]].clone();
            let galley = layout(ui, content, &fragment, &geometry.font, geometry.width);
            let row_height = measured_height(&galley, geometry.minimum_height, &fragment, content);
            self.rows.set_height(row, row_height);
            let text_origin = origin + vec2(SourceView::GUTTER_WIDTH + TEXT_INSET, y + 2.0);
            let rect = Rect::from_min_size(
                pos2(geometry.code_rect.left(), origin.y + y),
                vec2(geometry.code_rect.width(), row_height),
            );
            let code_bottom = origin.y + y + (galley.size().y + 4.0).max(geometry.minimum_height);
            let code_hit = Rect::from_min_max(rect.min, pos2(rect.right(), code_bottom));
            let response = ui.interact(
                code_hit,
                Id::new(("source_row", fragment.bytes.start)),
                Sense::click_and_drag(),
            );
            self.paint_row(&painter, &galley, text_origin, rect, &fragment, content);
            if fragment.first_in_line {
                if let Some(event) =
                    paint_gutter(ui, &gutter, geometry.surface, rect, &fragment, content)
                {
                    if let SourceViewAction::ToggleFold(fold) = &event {
                        content.fold_store.toggle(fold.clone());
                    }
                    action = Some(event);
                }
            }
            if let Some(event) =
                paint_test_label(ui, &painter, rect, text_origin, &fragment, content)
            {
                action = Some(event);
            }
            if let Some(pointer) = response
                .interact_pointer_pos()
                .or_else(|| ui.ctx().pointer_interact_pos())
            {
                let target_y = pointer
                    .y
                    .clamp(geometry.code_rect.top(), geometry.code_rect.bottom() - 0.1);
                if target_y >= rect.top() && target_y < code_bottom {
                    let byte = hit_byte(
                        &galley,
                        pointer - text_origin,
                        &fragment,
                        &content.buffer.text,
                    );
                    self.track_hover_geometry(
                        response.hovered().then_some((byte, pointer.x)),
                        &galley,
                        text_origin,
                        &fragment,
                        content.buffer,
                    );
                    if let Some(event) = self
                        .definition_pointer(ui, &response, byte, content)
                        .or_else(|| {
                            self.pointer_input(
                                ui,
                                &response,
                                byte,
                                geometry.code_rect.contains(pointer),
                                content,
                            )
                        })
                    {
                        action = Some(event);
                    }
                    action = self.occurrence_click(ui, &response, byte).or(action);
                }
            }
            if let Some(event) = self.context_menu(&response, content) {
                action = Some(event);
            }
            full_width =
                full_width.max(galley.size().x + SourceView::GUTTER_WIDTH + TEXT_INSET * 2.0);
            y += row_height;
            row += 1;
        }
        let document_width = if content.wrap {
            0.0
        } else {
            full_width.max(
                self.max_bytes as f32 * geometry.font.size * 0.7
                    + SourceView::GUTTER_WIDTH
                    + TEXT_INSET * 2.0,
            )
        };
        ui.expand_to_include_rect(Rect::from_min_size(
            origin,
            vec2(document_width, self.rows.total_height()),
        ));
        action
    }

    fn track_hover_geometry(
        &mut self,
        hit: Option<(usize, f32)>,
        galley: &egui::Galley,
        origin: egui::Pos2,
        part: &SourceFragment,
        buffer: &SourceBuffer,
    ) {
        let Some((byte, x)) = hit else {
            return;
        };
        self.hover_pointer = Some(byte);
        if self
            .hover
            .anchor_range()
            .is_some_and(|range| range.contains(&byte))
        {
            let chars = part.char_index(&buffer.text, byte);
            let cursor = galley.pos_from_ccursor(egui::text::CCursor::new(chars));
            self.hover_origin = pos2(x, origin.y + cursor.bottom());
        }
    }

    fn finish_drag(&mut self, ui: &Ui, code_rect: Rect) {
        if self.dragging {
            if let Some(pointer) = ui.ctx().pointer_interact_pos() {
                let outside = pointer.y - pointer.y.clamp(code_rect.top(), code_rect.bottom());
                if outside.abs() > 0.0 {
                    self.scroll_y = (self.scroll_y + outside.clamp(-30.0, 30.0)).max(0.0);
                    ui.ctx().request_repaint();
                }
            }
            if !ui.input(|input| input.pointer.primary_down()) {
                self.dragging = false;
            }
        }
    }

    fn context_menu(
        &self,
        response: &egui::Response,
        content: &CanvasContent<'_>,
    ) -> Option<SourceViewAction> {
        let mut action = None;
        crate::ui::commands::context_menu(response, |ui| {
            if ui
                .add_enabled(
                    self.selection
                        .is_some_and(|selection| !selection.range().is_empty()),
                    egui::Button::new("Copy Selection"),
                )
                .clicked()
            {
                self.copy(ui.ctx(), content.buffer);
                ui.close_menu();
            }
            if ui.button("Copy Line").clicked() {
                ui.output_mut(|output| {
                    output.copied_text = content.buffer.line_text(*content.caret_line).to_owned()
                });
                ui.close_menu();
            }
            let byte = content
                .buffer
                .byte_offset(*content.caret_line, *content.caret_col);
            let definition = content.definition_at_byte(byte);
            if ui
                .add_enabled(
                    definition.is_some(),
                    egui::Button::new("Go to Definition")
                        .shortcut_text("Ctrl+Alt+J / F12")
                        .wrap(false),
                )
                .clicked()
            {
                action = definition.map(SourceViewAction::NavigateToDefinition);
                ui.close_menu();
            }
            let reference_target = content.mark_context.and_then(|context| {
                crate::semantic::references::ReferenceTarget::at_byte(
                    context.index,
                    content.buffer,
                    byte,
                )
            });
            if ui
                .add_enabled(
                    reference_target.is_some(),
                    egui::Button::new("Find References")
                        .shortcut_text("Ctrl+Alt+R")
                        .wrap(false),
                )
                .clicked()
            {
                action = Some(SourceViewAction::FindReferences(byte));
                ui.close_menu();
            }
            if let Some(menu) = content
                .mark_context
                .and_then(|context| context.at_byte(content.buffer, byte))
            {
                if let Some(edit) = crate::ui::critical_menu::show(ui, &menu) {
                    action = Some(SourceViewAction::SetCriticality(edit));
                }
            }
        });
        action
    }

    fn definition_pointer(
        &mut self,
        ui: &Ui,
        response: &egui::Response,
        byte: usize,
        content: &CanvasContent<'_>,
    ) -> Option<SourceViewAction> {
        let modifiers = ui.input(|input| input.modifiers);
        if !content.keyboard_enabled
            || !response.hovered()
            || !(modifiers.ctrl || modifiers.command)
            || modifiers.alt
            || modifiers.shift
        {
            return None;
        }
        let target = content.definition_at_byte(byte)?;
        response
            .clone()
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        if !ui.input(|input| input.pointer.primary_pressed()) {
            return None;
        }
        self.dragging = false;
        self.occurrence_clicks = None;
        self.hover.dismiss();
        Some(SourceViewAction::NavigateToDefinition(target))
    }

    fn pointer_input(
        &mut self,
        ui: &Ui,
        response: &egui::Response,
        byte: usize,
        inside: bool,
        content: &mut CanvasContent<'_>,
    ) -> Option<SourceViewAction> {
        let pressed = inside
            && response.contains_pointer()
            && ui.input(|input| input.pointer.primary_pressed());
        let secondary = inside && response.secondary_clicked();
        if pressed || secondary {
            ui.memory_mut(|memory| memory.request_focus(Self::focus_id()));
            crate::ui::commands::retain_navigation_keys(ui.ctx(), Self::focus_id());
            let extending = ui.input(|input| input.modifiers.shift);
            let keep_selection = secondary
                && self
                    .selection
                    .is_some_and(|selection| selection.range().contains(&byte));
            if !keep_selection {
                let previous = content
                    .buffer
                    .byte_offset(*content.caret_line, *content.caret_col);
                let anchor = if extending {
                    self.selection
                        .map_or(previous, |selection| selection.anchor)
                } else {
                    byte
                };
                self.selection = Some(ByteSelection {
                    anchor,
                    focus: byte,
                });
            }
            self.dragging = pressed;
        } else if self.dragging
            && ui.input(|input| input.pointer.primary_down() || input.pointer.primary_released())
        {
            if let Some(selection) = &mut self.selection {
                selection.focus = byte;
            }
        } else {
            return None;
        }
        ui.ctx().request_repaint();
        Some(move_caret(content, byte))
    }

    fn occurrence_click(
        &mut self,
        ui: &Ui,
        response: &egui::Response,
        byte: usize,
    ) -> Option<SourceViewAction> {
        if self
            .selection
            .is_some_and(|selection| selection.anchor != selection.focus)
        {
            self.occurrence_clicks = None;
            return None;
        }
        if !response.clicked() {
            return None;
        }
        if ui.input(|input| input.modifiers != Modifiers::NONE) {
            self.occurrence_clicks = None;
            return None;
        }
        let point = response.interact_pointer_pos()?;
        let now = ui.input(|input| input.time);
        // egui 0.27's global double-click count checks time only. Keep the
        // source gesture local and use its same six-point / 300 ms tolerances.
        let count = self
            .occurrence_clicks
            .filter(|(previous, time, _)| {
                previous.distance(point) <= 6.0 && now >= *time && now - time < 0.3
            })
            .map_or(1, |(_, _, count)| count.saturating_add(1));
        self.occurrence_clicks = Some((point, now, count));
        match count {
            1 => Some(SourceViewAction::SelectOccurrences(byte)),
            2 => Some(SourceViewAction::PinAtByte(byte)),
            _ => None,
        }
    }

    fn row_for_byte(&self, byte: usize) -> Option<(usize, &SourceFragment)> {
        let row = self
            .visible
            .partition_point(|&index| self.fragments[index].bytes.start <= byte)
            .saturating_sub(1);
        self.visible
            .get(row)
            .map(|&index| (row, &self.fragments[index]))
    }

    fn caret_rect(
        &mut self,
        ui: &Ui,
        content: &CanvasContent<'_>,
        byte: usize,
        font: &FontId,
        width: f32,
        minimum_height: f32,
    ) -> Rect {
        let Some((row, part)) = self.row_for_byte(byte) else {
            return Rect::NOTHING;
        };
        let part = part.clone();
        let galley = layout(ui, content, &part, font, width);
        self.rows.set_height(
            row,
            measured_height(&galley, minimum_height, &part, content),
        );
        let chars = part.char_index(&content.buffer.text, byte);
        galley
            .pos_from_ccursor(egui::text::CCursor::new(chars))
            .translate(vec2(0.0, self.rows.offset(row) + 2.0))
    }

    fn measure(
        &mut self,
        ui: &Ui,
        content: &CanvasContent<'_>,
        font: &FontId,
        width: f32,
        height: f32,
    ) {
        if !content.wrap {
            if self.rows.next_measurement > 0 {
                return;
            }
            for (row, &index) in self.visible.iter().enumerate() {
                let part = &self.fragments[index];
                if part.first_in_line && content.test_code.header(part.line).is_some() {
                    self.rows.set_height(row, height + 18.0);
                }
            }
            self.rows.next_measurement = self.visible.len();
            return;
        }
        let started = std::time::Instant::now();
        for _ in 0..64 {
            let row = self.rows.next_measurement;
            if row >= self.visible.len() {
                break;
            }
            let part = &self.fragments[self.visible[row]];
            let galley = layout(ui, content, part, font, width);
            self.rows
                .set_height(row, measured_height(&galley, height, part, content));
            self.rows.next_measurement += 1;
            if started.elapsed() >= std::time::Duration::from_millis(2) {
                break;
            }
        }
        if self.rows.next_measurement < self.visible.len() {
            ui.ctx().request_repaint();
        }
    }

    fn copy(&self, ctx: &egui::Context, buffer: &SourceBuffer) {
        if let Some(text) = self
            .selection
            .and_then(|selection| selection.text(&buffer.text))
        {
            ctx.output_mut(|output| output.copied_text = text.to_owned());
        }
    }

    fn keyboard(
        &mut self,
        ui: &Ui,
        content: &mut CanvasContent<'_>,
        font: &FontId,
        width: f32,
        height: f32,
    ) -> Option<SourceViewAction> {
        let actions = ui.input_mut(|input| {
            crate::ui::commands::take_input_actions(&mut input.events, source_input)
        });
        let mut moved = None;
        for action in actions {
            match action {
                SourceInput::Copy => self.copy(ui.ctx(), content.buffer),
                SourceInput::SelectAll => {
                    self.select(0..content.buffer.byte_len());
                    ui.ctx().request_repaint();
                }
                SourceInput::Move(key, shift) => {
                    moved = Some(self.move_key(ui, content, font, width, height, (key, shift)));
                }
            }
        }
        moved
    }

    fn move_key(
        &mut self,
        ui: &Ui,
        content: &mut CanvasContent<'_>,
        font: &FontId,
        width: f32,
        height: f32,
        navigation: (Key, bool),
    ) -> SourceViewAction {
        let (key, shift) = navigation;
        let buffer = content.buffer;
        let byte = buffer.byte_offset(*content.caret_line, *content.caret_col);
        let target = if matches!(key, Key::ArrowUp | Key::ArrowDown) {
            self.vertical_target(ui, content, byte, key == Key::ArrowDown, font, width)
        } else {
            keyboard_target(
                key,
                byte,
                buffer,
                &self.fragments,
                &self.visible,
                (height / (font.size + 4.0)).max(1.0) as isize,
            )
        };
        if shift {
            self.selection
                .get_or_insert(ByteSelection {
                    anchor: byte,
                    focus: byte,
                })
                .focus = target;
        } else {
            self.selection = None;
        }
        *content.target = Some(buffer.line_at_byte(target));
        move_caret(content, target)
    }

    fn vertical_target(
        &self,
        ui: &Ui,
        content: &CanvasContent<'_>,
        byte: usize,
        down: bool,
        font: &FontId,
        width: f32,
    ) -> usize {
        let Some((row, part)) = self.row_for_byte(byte) else {
            return byte;
        };
        let galley = layout(ui, content, part, font, width);
        let cursor = galley.from_ccursor(egui::text::CCursor::new(
            part.char_index(&content.buffer.text, byte),
        ));
        let next = if down {
            galley.cursor_down_one_row(&cursor)
        } else {
            galley.cursor_up_one_row(&cursor)
        };
        let can_move = if down {
            cursor.rcursor.row + 1 < galley.rows.len()
        } else {
            cursor.rcursor.row > 0
        };
        if can_move {
            return part.byte_at_char(&content.buffer.text, next.ccursor.index);
        }
        let adjacent = if down {
            row.saturating_add(1)
        } else {
            row.saturating_sub(1)
        };
        let Some(&index) = self.visible.get(adjacent).filter(|_| adjacent != row) else {
            return byte;
        };
        let part = &self.fragments[index];
        let galley_next = layout(ui, content, part, font, width);
        let x = galley.pos_from_cursor(&cursor).min.x;
        hit_byte(
            &galley_next,
            vec2(x, if down { 0.0 } else { galley_next.size().y }),
            part,
            &content.buffer.text,
        )
    }

    fn paint_destination(
        &self,
        painter: &egui::Painter,
        galley: &egui::Galley,
        origin: egui::Pos2,
        rect: Rect,
        part: &SourceFragment,
        content: &CanvasContent<'_>,
    ) {
        let Some(frame) = self.destination_frame else {
            return;
        };
        if !self
            .row_for_byte(frame.byte)
            .is_some_and(|(_, owner)| owner == part && part.contains_byte(frame.byte))
        {
            return;
        }
        let chars = part.char_index(&content.buffer.text, frame.byte);
        let cursor = galley
            .pos_from_ccursor(egui::text::CCursor::new(chars))
            .translate(origin.to_vec2());
        let band = if cursor.height() > 0.0 {
            Rect::from_min_max(
                pos2(rect.left(), cursor.top()),
                pos2(rect.right(), cursor.bottom()),
            )
        } else {
            rect
        };
        painter.rect_filled(band, 0.0, content.theme.accent.gamma_multiply(frame.alpha));
    }

    fn paint_row_backgrounds(
        &self,
        painter: &egui::Painter,
        galley: &egui::Galley,
        origin: egui::Pos2,
        rect: Rect,
        part: &SourceFragment,
        content: &CanvasContent<'_>,
    ) {
        let theme = content.theme;
        let rect = if part.first_in_line && content.test_code.header(part.line).is_some() {
            Rect::from_min_max(rect.min, pos2(rect.right(), rect.bottom() - 18.0))
        } else {
            rect
        };
        if let Some(band) = content.function_shade.and_then(|range| {
            function_band(galley, origin, rect, part, range, &content.buffer.text)
        }) {
            painter.rect_filled(band, 0.0, theme.function_shade_color());
        }
        if content.changes.iter().any(|range| {
            part.spans()
                .iter()
                .any(|span| range.contains(&content.buffer.line_at_byte(span.start)))
        }) {
            painter.rect_filled(
                rect,
                0.0,
                if content.buffer.revision == Some(ReviewRevision::Before) {
                    theme.diff_del
                } else {
                    theme.diff_add
                },
            );
        }
        self.paint_destination(painter, galley, origin, rect, part, content);
        let caret_byte = content
            .buffer
            .byte_offset(*content.caret_line, *content.caret_col);
        if self
            .row_for_byte(caret_byte)
            .is_some_and(|(_, owner)| owner == part && part.contains_byte(caret_byte))
        {
            let chars = part.char_index(&content.buffer.text, caret_byte);
            painter.rect_filled(
                visual_line_band(galley, origin, rect, egui::text::CCursor::new(chars)),
                0.0,
                theme.accent.gamma_multiply(0.05),
            );
        }
    }

    fn paint_row(
        &self,
        painter: &egui::Painter,
        galley: &egui::Galley,
        origin: egui::Pos2,
        rect: Rect,
        part: &SourceFragment,
        content: &mut CanvasContent<'_>,
    ) {
        self.paint_row_backgrounds(painter, galley, origin, rect, part, content);
        let theme = content.theme;
        let mut matches: Vec<_> = part
            .spans()
            .iter()
            .flat_map(|span| content.find.matches(span.clone()))
            .collect();
        matches.dedup_by(|a, b| a.0 == b.0);
        for (range, active) in matches {
            paint_match(
                painter,
                galley,
                origin,
                part,
                &content.buffer.text,
                range,
                if active {
                    theme.accent.gamma_multiply(0.45)
                } else {
                    Color32::from_rgba_unmultiplied(220, 170, 40, 60)
                },
            );
        }
        if let Some(selection) = self
            .selection
            .filter(|selection| selection.anchor != selection.focus)
        {
            let range = selection.range();
            if range.start < part.bytes.end && range.end > part.bytes.start {
                paint_match(
                    painter,
                    galley,
                    origin,
                    part,
                    &content.buffer.text,
                    range,
                    theme.accent.gamma_multiply(0.35),
                );
            }
        }
        paint_occurrences(painter, galley, origin, part, content);
        painter.galley(origin, Arc::new(galley.clone()), theme.text_primary);
        let byte = content
            .buffer
            .byte_offset(*content.caret_line, *content.caret_col);
        if self
            .row_for_byte(byte)
            .is_some_and(|(_, owner)| owner == part && part.contains_byte(byte))
        {
            let chars = part.char_index(&content.buffer.text, byte);
            let cursor = galley.from_ccursor(egui::text::CCursor::new(chars));
            let caret = galley.pos_from_cursor(&cursor).translate(origin.to_vec2());
            painter.line_segment(
                [caret.left_top(), caret.left_bottom()],
                Stroke::new(1.5_f32, theme.accent),
            );
        }
    }
}

enum SourceInput {
    Copy,
    SelectAll,
    Move(Key, bool),
}

fn source_input(event: &egui::Event) -> Option<SourceInput> {
    if matches!(event, egui::Event::Copy) {
        return Some(SourceInput::Copy);
    }
    let egui::Event::Key {
        key,
        modifiers,
        pressed: true,
        ..
    } = event
    else {
        return None;
    };
    if modifiers.command || modifiers.ctrl {
        if modifiers.alt || modifiers.shift {
            return None;
        }
        return match key {
            Key::C => Some(SourceInput::Copy),
            Key::A => Some(SourceInput::SelectAll),
            _ => None,
        };
    }
    const NAVIGATION: [Key; 8] = [
        Key::ArrowUp,
        Key::ArrowDown,
        Key::ArrowLeft,
        Key::ArrowRight,
        Key::Home,
        Key::End,
        Key::PageUp,
        Key::PageDown,
    ];
    (!modifiers.alt && NAVIGATION.contains(key)).then_some(SourceInput::Move(*key, modifiers.shift))
}

fn move_caret(content: &mut CanvasContent<'_>, byte: usize) -> SourceViewAction {
    let line = content.buffer.line_at_byte(byte);
    let column = content.buffer.column_at_byte(byte);
    *content.caret_line = line;
    *content.caret_col = column;
    SourceViewAction::CaretMoved {
        line,
        column,
        byte_offset: byte,
    }
}

fn paint_occurrences(
    painter: &egui::Painter,
    galley: &egui::Galley,
    origin: egui::Pos2,
    part: &SourceFragment,
    content: &CanvasContent<'_>,
) {
    let theme = content.theme;
    let pins = content.pins.groups.iter().flat_map(|group| {
        let base = theme.occurrence_color(group.color);
        let fill = base.gamma_multiply(if theme.is_dark { 0.22 } else { 0.15 });
        let stroke = Stroke::new(
            1.4_f32,
            base.gamma_multiply(if theme.is_dark { 0.88 } else { 0.72 }),
        );
        group.ranges.iter().map(move |range| (range, fill, stroke))
    });
    let transient = content.pins.ephemeral.iter().flat_map(|selection| {
        selection.ranges.iter().map(move |range| {
            let active = *range == selection.active;
            let fill = theme
                .accent
                .gamma_multiply(if active { 0.28 } else { 0.16 });
            let stroke = if active {
                Stroke::new(1.0_f32, theme.accent.gamma_multiply(0.55))
            } else {
                Stroke::NONE
            };
            (range, fill, stroke)
        })
    });
    for (range, fill, stroke) in pins.chain(transient).filter(|(range, _, _)| {
        !range.is_empty() && range.start < part.bytes.end && range.end > part.bytes.start
    }) {
        for_each_match_rect(
            galley,
            origin,
            part,
            &content.buffer.text,
            range.clone(),
            |rect| {
                painter.rect(rect.expand2(vec2(1.5, 0.0)), 2.5, fill, stroke);
            },
        );
    }
}

// Extend to the fragment edges so adjacent bands include line leading, while
// wrapped boundaries still follow the shaped glyph rows. Never include test labels.
fn visual_line_band(
    galley: &egui::Galley,
    origin: egui::Pos2,
    rect: Rect,
    cursor: egui::text::CCursor,
) -> Rect {
    let row = galley.from_ccursor(cursor).rcursor.row;
    let top = if row == 0 {
        rect.top()
    } else {
        origin.y + galley.rows[row].min_y()
    };
    let bottom = galley
        .rows
        .get(row + 1)
        .map_or(rect.bottom(), |next| origin.y + next.min_y());
    Rect::from_min_max(pos2(rect.left(), top), pos2(rect.right(), bottom))
}

fn function_band(
    galley: &egui::Galley,
    origin: egui::Pos2,
    rect: Rect,
    part: &SourceFragment,
    range: &Range<usize>,
    source: &str,
) -> Option<Rect> {
    if part.bytes.is_empty() {
        return range
            .contains(&part.bytes.start)
            .then(|| visual_line_band(galley, origin, rect, egui::text::CCursor::new(0)));
    }
    let start = range.start.max(part.bytes.start);
    let end = range.end.min(part.bytes.end);
    if start >= end {
        return None;
    }
    let first = part.char_index(source, start);
    let end = part.char_index(source, end);
    if first >= end {
        return None;
    }
    let last = end - 1;
    let band = |index| {
        visual_line_band(
            galley,
            origin,
            rect,
            egui::text::CCursor {
                index,
                prefer_next_row: true,
            },
        )
    };
    Some(Rect::from_min_max(band(first).min, band(last).max))
}

fn hit_byte(
    galley: &egui::Galley,
    position: egui::Vec2,
    part: &SourceFragment,
    source: &str,
) -> usize {
    part.byte_at_char(source, galley.cursor_from_pos(position).ccursor.index)
}

fn keyboard_target(
    key: Key,
    byte: usize,
    buffer: &SourceBuffer,
    parts: &[SourceFragment],
    visible: &[usize],
    page: isize,
) -> usize {
    if visible.is_empty() {
        return byte;
    }
    let position = visible
        .partition_point(|&index| parts[index].bytes.start <= byte)
        .saturating_sub(1);
    let part = &parts[visible[position]];
    let chars = part.char_index(&buffer.text, byte);
    let total = part.char_index(&buffer.text, part.bytes.end);
    match key {
        Key::ArrowLeft if chars > 0 => part.byte_at_char(&buffer.text, chars - 1),
        Key::ArrowRight if chars < total => part.byte_at_char(&buffer.text, chars + 1),
        Key::ArrowLeft => parts[visible[position.saturating_sub(1)]].bytes.end,
        Key::ArrowRight => {
            parts[visible[(position + 1).min(visible.len() - 1)]]
                .bytes
                .start
        }
        Key::Home => {
            let mut first = position;
            while first > 0 && !parts[visible[first]].first_in_line {
                first -= 1;
            }
            parts[visible[first]].bytes.start
        }
        Key::End => {
            let mut last = position;
            while last + 1 < visible.len() && !parts[visible[last + 1]].first_in_line {
                last += 1;
            }
            parts[visible[last]].bytes.end
        }
        _ => {
            let step = if matches!(key, Key::ArrowUp | Key::PageUp) {
                -page
            } else {
                page
            };
            let next = position.saturating_add_signed(step).min(visible.len() - 1);
            parts[visible[next]].byte_at_char(&buffer.text, chars)
        }
    }
}

fn layout(
    ui: &Ui,
    content: &CanvasContent<'_>,
    part: &SourceFragment,
    font: &FontId,
    width: f32,
) -> Arc<egui::Galley> {
    let mut job = LayoutJob {
        break_on_newline: false,
        ..Default::default()
    };
    job.wrap.max_width = width;
    for range in part.spans() {
        let line = content.buffer.line_at_byte(range.start);
        let line_start = content.buffer.byte_offset(line, 0);
        let before = job.text.len();
        if let Some(spans) = content.highlights.get(line - 1) {
            let first = spans.partition_point(|span| line_start + span.bytes.end <= range.start);
            for span in &spans[first..] {
                let start = (line_start + span.bytes.start).max(range.start);
                let end = (line_start + span.bytes.end).min(range.end);
                if start < end {
                    let color = if content.test_code.header(line).is_some_and(|region| {
                        content.test_code.is_hidden(region.byte_range.start)
                            && region.byte_range.start <= start
                    }) {
                        span.color.gamma_multiply(0.55)
                    } else {
                        span.color
                    };
                    append_projected(&mut job, content, font, start..end, color);
                }
                if line_start + span.bytes.end >= range.end {
                    break;
                }
            }
        }
        if job.text.len() == before {
            let color = if content.test_code.header(line).is_some_and(|region| {
                content.test_code.is_hidden(region.byte_range.start)
                    && region.byte_range.start <= range.start
            }) {
                content.theme.text_muted
            } else {
                content.theme.text_primary
            };
            append_projected(&mut job, content, font, range.clone(), color);
        }
    }
    ui.fonts(|fonts| fonts.layout_job(job))
}

fn append_projected(
    job: &mut LayoutJob,
    content: &CanvasContent<'_>,
    font: &FontId,
    range: Range<usize>,
    color: Color32,
) {
    let projection = content.projection.filter(|projection| {
        content.mode == SourceMode::Critical && projection.matches(content.buffer)
    });
    let mut cursor = range.start;
    if let Some(projection) = projection {
        let first = projection
            .receded_byte_ranges
            .partition_point(|receded| receded.end <= range.start);
        for receded in &projection.receded_byte_ranges[first..] {
            if receded.start >= range.end {
                break;
            }
            if cursor < receded.start {
                job.append(
                    &content.buffer.text[cursor..receded.start],
                    0.0,
                    TextFormat::simple(font.clone(), color),
                );
            }
            let start = cursor.max(receded.start);
            let end = range.end.min(receded.end);
            let dimmed = color.gamma_multiply(if content.theme.is_dark { 0.38 } else { 0.30 });
            job.append(
                &content.buffer.text[start..end],
                0.0,
                TextFormat::simple(font.clone(), dimmed),
            );
            cursor = end;
        }
    }
    if cursor < range.end {
        job.append(
            &content.buffer.text[cursor..range.end],
            0.0,
            TextFormat::simple(font.clone(), color),
        );
    }
}

fn paint_gutter(
    ui: &mut Ui,
    painter: &egui::Painter,
    surface: Rect,
    row: Rect,
    part: &SourceFragment,
    content: &CanvasContent<'_>,
) -> Option<SourceViewAction> {
    let theme = content.theme;
    painter.text(
        pos2(row.left() - 10.0, row.top() + 2.0),
        egui::Align2::RIGHT_TOP,
        part.line.to_string(),
        FontId::monospace(12.0),
        if part.line == *content.caret_line {
            theme.accent
        } else {
            theme.text_muted
        },
    );
    if content
        .changes
        .iter()
        .any(|range| range.contains(&part.line))
    {
        painter.rect_filled(
            Rect::from_min_size(pos2(row.left() - 3.0, row.top()), vec2(3.0, row.height())),
            0.0,
            if content.buffer.revision == Some(ReviewRevision::Before) {
                Color32::from_rgb(255, 59, 48)
            } else {
                Color32::from_rgb(52, 199, 89)
            },
        );
    }
    if let Some(region) = content.test_code.header(part.line) {
        let rect = Rect::from_min_size(
            pos2(surface.left() + 4.0, row.top() + 3.0),
            vec2(28.0, 16.0),
        );
        let hidden = content.test_code.is_hidden(region.byte_range.start);
        let icon = if hidden {
            Icon::ChevronRight
        } else {
            Icon::ChevronDown
        };
        icon.paint(
            painter,
            Rect::from_min_size(rect.min, vec2(10.0, 12.0)),
            theme.text_muted,
        );
        let count = region.line_range.end.saturating_sub(region.header_line + 1);
        let response = ui
            .interact(
                rect,
                Id::new(("test_gutter", region.byte_range.start)),
                Sense::click(),
            )
            .on_hover_text(format!("Test code — {count} lines hidden; click to toggle"));
        return response
            .clicked()
            .then_some(SourceViewAction::ToggleTestRegion(region.byte_range.start));
    }
    let fold = content.folds.iter().find(|fold| {
        fold.header_line(content.buffer) == part.line
            && (fold.header_line(content.buffer) != fold.start_line
                || part.contains_byte(fold.byte_range.start))
    })?;
    let rect = Rect::from_min_size(
        pos2(surface.left() + 4.0, row.top() + 3.0),
        vec2(28.0, 16.0),
    );
    let collapsed = content.fold_store.is_folded(&fold.byte_range);
    let icon = if collapsed {
        Icon::ChevronRight
    } else {
        Icon::ChevronDown
    };
    icon.paint(
        painter,
        Rect::from_min_size(rect.min, vec2(10.0, 12.0)),
        theme.text_muted,
    );
    let hidden = fold
        .end_line
        .saturating_sub(fold.header_line(content.buffer));
    let annotation = (content.mode == SourceMode::Critical)
        .then_some(content.annotations)
        .flatten()
        .and_then(|annotations| annotations.at_range(content.buffer, &fold.byte_range));
    if collapsed {
        paint_fold_count(painter, surface, row, hidden, annotation, theme);
    }
    let tooltip = if collapsed {
        annotation.map_or_else(
            || format!("Collapsed block — {hidden} lines hidden; click to expand"),
            |annotation| {
                format!(
                    "Collapsed function — {hidden} lines hidden · {}",
                    annotation.detail
                )
            },
        )
    } else {
        "Click to collapse this block".into()
    };
    let hit = Rect::from_min_max(rect.min, pos2(surface.left() + 58.0, rect.bottom()));
    let response = ui.interact(
        hit,
        Id::new(("source_fold", fold.byte_range.start, fold.byte_range.end)),
        Sense::click(),
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, &tooltip));
    response
        .on_hover_text(tooltip)
        .clicked()
        .then_some(SourceViewAction::ToggleFold(fold.byte_range.clone()))
}

fn paint_fold_count(
    painter: &egui::Painter,
    surface: Rect,
    row: Rect,
    hidden: usize,
    annotation: Option<&crate::critical::annotations::FoldAnnotation>,
    theme: &AppTheme,
) {
    let rect = Rect::from_min_size(
        pos2(surface.left() + 26.0, row.top() + 2.0),
        vec2(30.0, 17.0),
    );
    let (label, color) = annotation.map_or_else(
        || (hidden.to_string(), theme.text_secondary),
        |annotation| {
            let emphasized = annotation.count.completeness
                == crate::critical::CountCompleteness::Partial
                || annotation.count.count > 0;
            (
                annotation.count.label(),
                if emphasized {
                    theme.accent
                } else {
                    theme.text_muted
                },
            )
        },
    );
    if annotation.is_some() {
        painter.rect_stroke(rect, 3.0_f32, Stroke::new(1.0_f32, color));
    }
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(10.0),
        color,
    );
}

fn measured_height(
    galley: &egui::Galley,
    minimum: f32,
    part: &SourceFragment,
    content: &CanvasContent<'_>,
) -> f32 {
    (galley.size().y + 4.0).max(minimum)
        + if part.first_in_line && content.test_code.header(part.line).is_some() {
            18.0
        } else {
            0.0
        }
}

fn paint_test_label(
    ui: &Ui,
    painter: &egui::Painter,
    row: Rect,
    origin: egui::Pos2,
    part: &SourceFragment,
    content: &CanvasContent<'_>,
) -> Option<SourceViewAction> {
    if !part.first_in_line {
        return None;
    }
    let region = content.test_code.header(part.line)?;
    let hidden = content.test_code.is_hidden(region.byte_range.start);
    let count = region.line_range.end.saturating_sub(region.header_line + 1);
    let label = if hidden {
        format!("Test code · {count} lines hidden")
    } else {
        "Hide test code".into()
    };
    let rect = Rect::from_min_size(pos2(origin.x, row.bottom() - 17.0), vec2(190.0, 16.0));
    painter.text(
        rect.left_center(),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(11.0),
        content.theme.text_muted,
    );
    ui.interact(
        rect,
        Id::new(("test_label", region.byte_range.start)),
        Sense::click(),
    )
    .on_hover_text("Toggle this test region")
    .clicked()
    .then_some(SourceViewAction::ToggleTestRegion(region.byte_range.start))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Reader {
        source: SourceBuffer,
        viewport: SourceViewport,
        folds: FoldStore,
        ranges: Vec<FoldRange>,
        find: SourceFindPanel,
        line: usize,
        column: usize,
        target: Option<usize>,
        wrap: bool,
        font_size: f32,
        test_code: crate::source::test_code::TestCodePresentation,
        mode: SourceMode,
        projection: Option<crate::critical::SourceProjection>,
        annotations: crate::critical::annotations::CriticalAnnotations,
        highlights: Vec<Vec<HighlightedSpan>>,
        hover_inputs: Option<Arc<crate::critical::worker::GraphInputs>>,
        now: Option<f64>,
        last_action: Option<SourceViewAction>,
        pins: OccurrencePinStore,
        function_shade: Option<Range<usize>>,
        theme: AppTheme,
        changes: Vec<Range<usize>>,
    }

    impl Reader {
        fn new(text: &str) -> Self {
            Self {
                source: SourceBuffer::from_str("fixture.rs", text),
                viewport: Default::default(),
                folds: Default::default(),
                ranges: Vec::new(),
                find: Default::default(),
                line: 1,
                column: 0,
                target: None,
                wrap: true,
                font_size: 14.0,
                test_code: Default::default(),
                mode: SourceMode::Normal,
                projection: None,
                annotations: Default::default(),
                highlights: Vec::new(),
                hover_inputs: None,
                now: None,
                last_action: None,
                pins: Default::default(),
                function_shade: None,
                theme: AppTheme::dark(),
                changes: Vec::new(),
            }
        }

        fn set_index(&mut self, index: crate::semantic::SemanticIndex) {
            let buffer = Arc::new(self.source.clone());
            self.hover_inputs = Some(Arc::new(crate::critical::worker::GraphInputs {
                source: crate::source::loader::SourceRead::Buffer(buffer.clone()),
                calls: Arc::new(crate::syntax::DocumentSyntax::analyze(&buffer).calls),
                buffer,
                index: Arc::new(index),
                overrides: Default::default(),
            }));
        }

        fn hover_ready_frame(&mut self, ctx: &egui::Context, now: f64) -> egui::FullOutput {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            while matches!(
                self.viewport.hover.presentation().map(|shown| &shown.phase),
                Some(crate::critical::hover::HoverPhase::Loading)
            ) {
                self.now = Some(now);
                self.frame(ctx, Vec::new());
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            self.frame(ctx, Vec::new())
        }

        fn frame(&mut self, ctx: &egui::Context, events: Vec<egui::Event>) -> egui::FullOutput {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(400.0, 360.0))),
                    modifiers: events
                        .iter()
                        .rev()
                        .find_map(|event| match event {
                            egui::Event::PointerButton { modifiers, .. }
                            | egui::Event::Key { modifiers, .. } => Some(*modifiers),
                            _ => None,
                        })
                        .unwrap_or_default(),
                    events,
                    time: self.now,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let event = self.viewport.show(
                            ui,
                            CanvasContent {
                                theme: &self.theme,
                                test_code: &self.test_code,
                                mode: self.mode,
                                mark_context: self.hover_inputs.as_deref().map(|inputs| {
                                    crate::critical::menu::MarkContext {
                                        index: &inputs.index,
                                        overrides: &inputs.overrides,
                                    }
                                }),
                                projection: self.projection.as_ref(),
                                annotations: self.annotations.current(&self.source),
                                hover_inputs: self.hover_inputs.clone(),
                                buffer: &self.source,
                                highlights: &self.highlights,
                                function_shade: self.function_shade.as_ref(),
                                caret_line: &mut self.line,
                                caret_col: &mut self.column,
                                folds: &self.ranges,
                                fold_store: &mut self.folds,
                                changes: &self.changes,
                                pins: &self.pins,
                                wrap: self.wrap,
                                font_size: self.font_size,
                                target: &mut self.target,
                                keyboard_enabled: true,
                                find: &mut self.find,
                            },
                        );
                        if let Some(SourceViewAction::ToggleTestRegion(id)) = &event {
                            self.test_code.toggle(&self.source, *id);
                        }
                        self.last_action = event;
                    });
                },
            )
        }
    }

    #[test]
    fn preview_reveals_exact_bytes_preserves_caret_selection_and_rejects_stale_source() {
        let ctx = egui::Context::default();
        let mut reader = Reader::new(&format!("origin café 😀\n{}target\n", "line\n".repeat(200)));
        reader.frame(&ctx, Vec::new());
        reader.viewport.select(0..6);
        let selected = reader.viewport.selection;
        let byte = reader.source.text.find("target").unwrap();
        reader.viewport.preview_byte(&reader.source, byte);
        reader.frame(&ctx, Vec::new());
        assert!(reader.viewport.scroll_y > 1000.0);
        assert_eq!((reader.line, reader.column), (1, 0));
        assert_eq!(reader.viewport.selection, selected);
        assert!(reader.last_action.is_none());
        let original = reader.source.clone();
        for source in [
            SourceBuffer::from_str("other.rs", &original.text),
            SourceBuffer::from_str(&original.path, &(original.text.clone() + "changed")),
            SourceBuffer::from_bytes(
                &original.path,
                original.data.clone(),
                Some(ReviewRevision::Before),
            )
            .unwrap(),
        ] {
            reader.viewport.preview_byte(&original, byte);
            reader.source = source;
            reader.frame(&ctx, Vec::new());
            assert_eq!(reader.viewport.scroll_y, 0.0);
        }
    }

    #[test]
    fn refreshed_viewport_restores_a_source_row_and_explicit_navigation_supersedes_it() {
        let ctx = egui::Context::default();
        let mut reader = Reader::new(&"line café 🦀\n".repeat(200));
        reader.frame(&ctx, vec![]);
        let byte = reader.source.byte_offset(80, 0);
        reader.viewport.restore_top(&reader.source, byte);
        reader.frame(&ctx, vec![]);
        assert_eq!(reader.viewport.top_byte(&reader.source), Some(byte));
        reader.source =
            SourceBuffer::from_str("renamed.rs", &format!("inserted\n{}", reader.source.text));
        let mapped = reader.source.byte_offset(81, 0);
        reader.viewport.restore_top(&reader.source, mapped);
        reader.frame(&ctx, vec![]);
        assert_eq!(reader.viewport.top_byte(&reader.source), Some(mapped));
        reader.viewport.restore_top(&reader.source, mapped);
        reader.viewport.cancel_destination();
        reader.line = 1;
        reader.target = Some(1);
        reader.frame(&ctx, vec![]);
        assert_eq!(reader.viewport.top_byte(&reader.source), Some(0));
    }

    #[test]
    fn ordinary_click_selects_occurrences_but_drag_selection_does_not() {
        let ctx = egui::Context::default();
        let mut reader = Reader::new("café 😀 value value\n");
        reader.now = Some(0.0);
        let output = reader.frame(&ctx, Vec::new());
        let (origin, galley) = text_shape(&output, "café 😀 value value");
        let point = origin
            + galley
                .pos_from_ccursor(egui::text::CCursor::new(8))
                .center()
                .to_vec2();
        reader.frame(&ctx, pointer(point, true));
        reader.frame(&ctx, pointer(point, false));
        assert!(matches!(
            reader.last_action,
            Some(SourceViewAction::SelectOccurrences(_))
        ));
        let other = point + vec2(55.0, 0.0);
        reader.now = Some(0.1);
        reader.frame(&ctx, pointer(other, true));
        reader.frame(&ctx, pointer(other, false));
        assert!(
            matches!(
                reader.last_action,
                Some(SourceViewAction::SelectOccurrences(_))
            ),
            "nearby-in-time clicks at different words are not a pin gesture"
        );
        reader.now = Some(0.2);
        reader.frame(&ctx, pointer(other, true));
        reader.frame(&ctx, pointer(other, false));
        assert!(matches!(
            reader.last_action,
            Some(SourceViewAction::PinAtByte(_))
        ));
        reader.now = Some(0.25);
        reader.frame(&ctx, pointer(other, true));
        reader.frame(&ctx, pointer(other, false));
        assert!(
            !matches!(
                reader.last_action,
                Some(SourceViewAction::PinAtByte(_)) | Some(SourceViewAction::SelectOccurrences(_))
            ),
            "triple-clicks do not toggle another pin"
        );
        reader.now = Some(1.0);
        reader.frame(&ctx, pointer(point, true));
        reader.frame(&ctx, pointer(point + vec2(35.0, 0.0), false));
        assert!(!matches!(
            reader.last_action,
            Some(SourceViewAction::SelectOccurrences(_))
        ));
        assert!(reader
            .viewport
            .selection
            .is_some_and(|selection| !selection.range().is_empty()));
        assert!(
            reader.viewport.occurrence_clicks.is_none(),
            "dragging ends the preceding click sequence"
        );
    }

    #[test]
    fn transient_occurrences_use_distinct_active_stroke_and_skip_folded_bytes() {
        for theme in [AppTheme::dark(), AppTheme::light()] {
            let ctx = egui::Context::default();
            let text = "fn f() { café();\n café();\n café();\n}\n";
            let mut reader = Reader::new(text);
            reader.theme = theme;
            let ranges: Vec<_> = text
                .match_indices("café")
                .map(|(start, word)| start..start + word.len())
                .collect();
            reader.pins.ephemeral = Some(crate::source::selection::OccurrenceSelection {
                symbol_name: "syntactic:f:café".into(),
                label: "café".into(),
                active: ranges[0].clone(),
                ranges,
            });
            reader.pins.toggle_labeled_pin(
                "pinned:café",
                "café",
                reader.pins.ephemeral.as_ref().unwrap().ranges.clone(),
                0,
            );
            let selected = reader.pins.ephemeral.clone();
            let output = reader.frame(&ctx, Vec::new());
            let pin_base = reader
                .theme
                .occurrence_color(crate::source::selection::OccurrenceColor::Teal);
            let pin_fill = pin_base.gamma_multiply(if reader.theme.is_dark { 0.22 } else { 0.15 });
            assert_eq!(filled_bands(&output, pin_fill).len(), 3);
            assert!(output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Rect(rect)
                if rect.fill == pin_fill && rect.stroke.width == 1.4_f32
                    && rect.stroke.color == pin_base.gamma_multiply(if reader.theme.is_dark { 0.88 } else { 0.72 }))));
            let active = filled_bands(&output, reader.theme.accent.gamma_multiply(0.28));
            let others = filled_bands(&output, reader.theme.accent.gamma_multiply(0.16));
            assert_eq!((active.len(), others.len()), (1, 2));
            assert!(output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Rect(rect)
                if rect.fill == reader.theme.accent.gamma_multiply(0.28) && rect.stroke.color == reader.theme.accent.gamma_multiply(0.55))));
            reader.ranges = crate::syntax::DocumentSyntax::analyze(&reader.source).folds;
            reader.folds.collapse_all_functions(&reader.ranges);
            let output = reader.frame(&ctx, Vec::new());
            assert!(filled_bands(&output, reader.theme.accent.gamma_multiply(0.28)).is_empty());
            assert!(filled_bands(&output, reader.theme.accent.gamma_multiply(0.16)).is_empty());
            assert_eq!(reader.pins.ephemeral, selected);
            assert!(reader.viewport.selection.is_none());
        }
    }

    #[test]
    fn folded_delimiters_and_unicode_tail_keep_exact_hit_arrow_and_copy_coordinates() {
        let ctx = egui::Context::default();
        let text = "fn café() {\r\n secret();\r\n} // 😀 tail\r\nnext();";
        let mut reader = Reader::new(text);
        reader.ranges = crate::syntax::DocumentSyntax::analyze(&reader.source).folds;
        reader.folds.collapse_all_functions(&reader.ranges);
        let output = reader.frame(&ctx, Vec::new());
        let (origin, galley) = text_shape(&output, "fn café() {} // 😀 tail");
        assert!(galley.job.text.len() < reader.source.text.len());
        let chars = galley.job.text.find("tail").unwrap();
        let chars = galley.job.text[..chars].chars().count();
        let point = origin
            + galley
                .pos_from_ccursor(egui::text::CCursor::new(chars))
                .center()
                .to_vec2();
        reader.frame(&ctx, pointer(point, true));
        reader.frame(&ctx, pointer(point, false));
        assert_eq!(
            reader.source.byte_offset(reader.line, reader.column),
            text.find("tail").unwrap()
        );
        reader.frame(&ctx, vec![key(Key::Home, Modifiers::NONE)]);
        assert_eq!((reader.line, reader.column), (1, 0));
        let opener = text.find('{').unwrap();
        reader.column = text[..opener].chars().count();
        reader.frame(&ctx, vec![key(Key::ArrowRight, Modifiers::NONE)]);
        assert_eq!(
            reader.source.byte_offset(reader.line, reader.column),
            text.find('}').unwrap()
        );
        reader.frame(&ctx, vec![key(Key::ArrowLeft, Modifiers::NONE)]);
        assert_eq!(
            reader.source.byte_offset(reader.line, reader.column),
            opener
        );
        reader.frame(&ctx, vec![key(Key::A, Modifiers::CTRL)]);
        assert_eq!(
            reader.viewport.selection.map(ByteSelection::range),
            Some(0..text.len())
        );
        let output = reader.frame(&ctx, vec![egui::Event::Copy]);
        assert_eq!(output.platform_output.copied_text, text);
    }

    #[test]
    fn indentation_disclosure_uses_the_preceding_header_and_reveals_whole_body() {
        let ctx = egui::Context::default();
        let mut reader = Reader::new("");
        reader.source = SourceBuffer::from_str("a.py", "def f():\n    one()\n    two()\nnext()\n");
        reader.ranges = crate::syntax::DocumentSyntax::analyze(&reader.source).folds;
        let fold = reader
            .ranges
            .iter()
            .find(|fold| fold.kind == crate::syntax::FoldKind::Callable)
            .unwrap()
            .clone();
        assert_eq!(fold.header_line(&reader.source), 1);
        assert_eq!(fold.hidden_range(&reader.source), fold.byte_range);
        reader.frame(&ctx, Vec::new());
        let point = pos2(20.0, 18.0);
        reader.frame(&ctx, pointer(point, true));
        reader.frame(&ctx, pointer(point, false));
        let output = reader.frame(&ctx, Vec::new());
        assert!(reader.folds.is_folded(&fold.byte_range));
        assert!(!output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text.contains("one()") || text.galley.job.text.contains("two()"))));
        reader
            .folds
            .reveal_byte(fold.byte_range.start, &reader.source, &reader.ranges);
        assert!(!reader.folds.is_folded(&fold.byte_range));
    }

    #[test]
    fn shared_header_gutter_toggles_outer_range_without_erasing_callable_defaults() {
        let ctx = egui::Context::default();
        let mut reader =
            Reader::new("impl Demo { fn first() {\n one();\n }\n fn second() {\n two();\n }\n}\n");
        reader.ranges = crate::syntax::DocumentSyntax::analyze(&reader.source).folds;
        reader.folds.collapse_all_functions(&reader.ranges);
        let outer = reader.ranges[0].byte_range.clone();
        assert_eq!(reader.ranges[0].kind, crate::syntax::FoldKind::Type);
        reader.frame(&ctx, Vec::new());
        assert!(!reader.folds.is_folded(&outer));
        let children = reader.folds.folded_ranges.clone();
        let point = pos2(20.0, 18.0);
        reader.frame(&ctx, pointer(point, true));
        reader.frame(&ctx, pointer(point, false));
        reader.frame(&ctx, Vec::new());
        assert!(reader.folds.is_folded(&outer));
        assert!(children.is_subset(&reader.folds.folded_ranges));
        assert!(!reader
            .viewport
            .visible
            .iter()
            .any(|index| reader.viewport.fragments[*index].line == 4));
        reader.frame(&ctx, pointer(point, true));
        reader.frame(&ctx, pointer(point, false));
        reader.frame(&ctx, Vec::new());
        assert_eq!(reader.folds.folded_ranges, children);
        assert!(reader
            .viewport
            .visible
            .iter()
            .any(|index| reader.viewport.fragments[*index].line == 4));
    }

    #[test]
    fn clicking_middle_of_empty_line_moves_caret_to_the_gap() {
        let ctx = egui::Context::default();
        let mut reader = Reader::new("fn first() {}\n\nfn second() {}\n");
        reader.frame(&ctx, Vec::new());
        let point = pos2(200.0, 8.0 + reader.viewport.rows.offset(1) + 10.0);
        reader.frame(&ctx, pointer(point, true));
        reader.frame(&ctx, pointer(point, false));
        assert_eq!((reader.line, reader.column), (2, 0));
    }

    fn filled_bands(output: &egui::FullOutput, color: Color32) -> Vec<Rect> {
        output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Rect(rect) if rect.fill == color => Some(rect.rect),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn callable_wash_tracks_nested_declarations_and_clears_between_methods() {
        let text = "struct T;\nimpl T {\n fn outer() {\n  fn inner() { café(); }\n\n }\n\n fn next() {}\n}\n";
        let mut reader = Reader::new(text);
        let syntax = crate::syntax::DocumentSyntax::analyze(&reader.source);
        let ctx = egui::Context::default();
        for (needle, expected) in [
            ("inner()", Some("inner")),
            ("outer()", Some("outer")),
            ("\n\n fn next", None),
            ("struct T", None),
        ] {
            let byte = text.find(needle).unwrap();
            let mut trail = Vec::new();
            for symbol in &syntax.symbols {
                symbol.breadcrumbs(byte, &mut trail);
            }
            let callable = trail.iter().rev().find(|symbol| symbol.is_callable());
            assert_eq!(callable.map(|symbol| symbol.name.as_str()), expected);
            reader.function_shade = callable.map(|symbol| symbol.range.clone());
            reader.line = reader.source.line_at_byte(byte);
            let output = reader.frame(&ctx, Vec::new());
            let bands = filled_bands(&output, reader.theme.function_shade_color());
            assert_eq!(!bands.is_empty(), expected.is_some());
            if expected == Some("outer") {
                // Signature, nested body, empty line and closing brace form one band.
                assert_eq!(bands.len(), 4);
                for pair in bands.windows(2) {
                    assert!((pair[0].bottom() - pair[1].top()).abs() < 0.01);
                }
            }
        }
    }

    #[test]
    fn callable_wash_clips_wrapped_unicode_ranges_and_keeps_layer_order() {
        for theme in [AppTheme::dark(), AppTheme::light()] {
            let ctx = egui::Context::default();
            let text = format!(
                "{}target{}",
                "é😀 word ".repeat(8),
                " continuation".repeat(12)
            );
            let mut reader = Reader::new(&text);
            reader.theme = theme;
            let start = text.find("target").unwrap();
            reader.function_shade = Some(start..start + 6);
            reader.column = text[..start].chars().count();
            reader.changes.push(1..2);
            reader.now = Some(1.0);
            reader.frame(&ctx, Vec::new());
            reader.viewport.flash_destination(&reader.source, start);
            let output = reader.frame(&ctx, Vec::new());
            let shade = filled_bands(&output, reader.theme.function_shade_color());
            let active = filled_bands(&output, reader.theme.accent.gamma_multiply(0.05));
            assert_eq!(shade.len(), 1);
            assert_eq!(active.len(), 1);
            assert!(shade[0].height() < 40.0);
            assert!(
                shade[0].top() > 20.0,
                "unrelated wrapped prefix must stay unshaded"
            );
            let colors: Vec<_> = output
                .shapes
                .iter()
                .filter_map(|shape| match &shape.shape {
                    egui::Shape::Rect(rect) => Some(rect.fill),
                    _ => None,
                })
                .collect();
            let index = |color| colors.iter().position(|fill| *fill == color).unwrap();
            assert!(index(reader.theme.function_shade_color()) < index(reader.theme.diff_add));
            assert!(index(reader.theme.diff_add) < index(reader.theme.accent.gamma_multiply(0.18)));
            assert!(
                index(reader.theme.accent.gamma_multiply(0.18))
                    < index(reader.theme.accent.gamma_multiply(0.05))
            );
        }
    }

    #[test]
    fn callable_wash_half_open_end_excludes_next_fragment_and_folded_body() {
        let ctx = egui::Context::default();
        let mut reader = Reader::new("fn example() {\n\n café 😀();\n}\nnext();\n");
        reader.function_shade = Some(0..reader.source.text.find("next").unwrap());
        let syntax = crate::syntax::DocumentSyntax::analyze(&reader.source);
        reader.ranges = syntax.folds;
        let output = reader.frame(&ctx, Vec::new());
        let bands = filled_bands(&output, reader.theme.function_shade_color());
        assert_eq!(bands.len(), 4);
        reader.folds.toggle_header(1, &reader.ranges);
        let output = reader.frame(&ctx, Vec::new());
        let folded = filled_bands(&output, reader.theme.function_shade_color());
        assert!(folded.len() < bands.len());
        assert!(!folded.is_empty());
    }

    fn feedback_bands(output: &egui::FullOutput, alpha: f32) -> Vec<Rect> {
        filled_bands(output, AppTheme::dark().accent.gamma_multiply(alpha))
    }

    #[test]
    fn landing_feedback_owns_one_wrapped_unicode_row_and_preserves_source_selection() {
        let ctx = egui::Context::default();
        let mut reader = Reader::new(&format!("{}\r\nend", "é😀 word ".repeat(1200)));
        reader.now = Some(1.0);
        reader.frame(&ctx, Vec::new());
        reader.viewport.select(0..2);
        let selection = reader.viewport.selection;
        let byte = reader.source.text.rfind("word").unwrap();
        reader.viewport.preview_byte(&reader.source, byte);
        reader.viewport.flash_destination(&reader.source, byte);
        reader.now = Some(1000.0);
        let output = reader.frame(&ctx, Vec::new());
        let bands = feedback_bands(&output, 0.18);
        assert_eq!(
            bands.len(),
            1,
            "continuation fragments must not duplicate feedback"
        );
        let (row, _) = reader.viewport.row_for_byte(byte).unwrap();
        assert!(bands[0].height() > 0.0);
        assert!(
            bands[0].height()
                < (reader.viewport.rows.offset(row + 1) - reader.viewport.rows.offset(row)) / 2.0,
            "only the destination's wrapped visual row should flash"
        );
        assert!(bands[0].width() > 200.0);
        assert_eq!((reader.line, reader.column), (1, 0));
        assert_eq!(reader.viewport.selection, selection);
        reader.now = Some(1001.005);
        assert_eq!(
            feedback_bands(&reader.frame(&ctx, Vec::new()), 0.09).len(),
            1
        );
        reader.now = Some(1001.2);
        let output = reader.frame(&ctx, Vec::new());
        assert!(reader.viewport.destination_frame.is_none());
        assert!(feedback_bands(&output, 0.18).is_empty());
        assert_eq!(reader.viewport.selection, selection);
        reader.viewport.flash_destination(&reader.source, byte);
        reader.viewport.preview_byte(&reader.source, 0);
        reader.now = Some(2000.0);
        assert!(feedback_bands(&reader.frame(&ctx, Vec::new()), 0.18).is_empty());
    }

    #[test]
    fn empty_and_crlf_destinations_have_a_visible_band_without_changing_the_caret() {
        for (text, byte) in [("", 0), ("café 😀\r\nsecond\r\n", 12)] {
            let ctx = egui::Context::default();
            let mut reader = Reader::new(text);
            reader.now = Some(1.0);
            reader.frame(&ctx, Vec::new());
            reader.viewport.flash_destination(&reader.source, byte);
            reader.now = Some(10.0);
            let bands = feedback_bands(&reader.frame(&ctx, Vec::new()), 0.18);
            assert_eq!(bands.len(), 1);
            assert!(bands[0].height() > 0.0);
            assert_eq!((reader.line, reader.column), (1, 0));
        }
    }

    fn key(key: Key, modifiers: Modifiers) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }

    fn pointer(point: egui::Pos2, pressed: bool) -> Vec<egui::Event> {
        vec![
            egui::Event::PointerMoved(point),
            egui::Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            },
        ]
    }

    fn text_shape(output: &egui::FullOutput, text: &str) -> (egui::Pos2, Arc<egui::Galley>) {
        output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(shape) if shape.galley.job.text == text => {
                    Some((shape.pos, shape.galley.clone()))
                }
                _ => None,
            })
            .expect("painted source text")
    }

    #[test]
    fn unavailable_source_menu_actions_stay_disabled_and_keep_the_popup_open() {
        let ctx = egui::Context::default();
        let mut reader = Reader::new("fn absent() {}\n");
        let output = reader.frame(&ctx, vec![]);
        let (origin, galley) = text_shape(&output, "fn absent() {}");
        let point = origin
            + galley
                .pos_from_ccursor(egui::text::CCursor::new(5))
                .center()
                .to_vec2();
        let events = crate::ui::test_support::click(point)
            .into_iter()
            .map(|event| match event {
                egui::Event::PointerButton {
                    pos,
                    pressed,
                    modifiers,
                    ..
                } => egui::Event::PointerButton {
                    pos,
                    pressed,
                    modifiers,
                    button: egui::PointerButton::Secondary,
                },
                other => other,
            })
            .collect();
        reader.frame(&ctx, events);
        for label in ["Copy Selection", "Go to Definition"] {
            let output = reader.frame(&ctx, vec![]);
            let (origin, galley) = text_shape(&output, label);
            let output = reader.frame(
                &ctx,
                crate::ui::test_support::click(origin + galley.size() / 2.0),
            );
            assert!(reader.last_action.is_none(), "{label}");
            assert!(output.platform_output.copied_text.is_empty());
            assert!(
                ctx.read_response(Id::new(("source_row", 0_usize)))
                    .unwrap()
                    .context_menu_opened(),
                "{label} must remain disabled"
            );
        }
    }

    #[test]
    fn control_click_resolves_exact_source_and_preserves_departure_selection_and_caret() {
        use crate::semantic::{ScipDocument, ScipIndex, ScipOccurrence, ScipRange, SemanticIndex};
        const HELPER: &str = "rust-analyzer cargo app 0 helper().";
        let text = "fn f() {\n let _ = \"😀\"; helper();\n}\nfn helper() {}\n";
        let call_line = text.lines().nth(1).unwrap();
        let start = call_line[..call_line.find("helper").unwrap()]
            .encode_utf16()
            .count() as i32;
        for (case, modifiers, expected) in [
            ("valid", Modifiers::CTRL, true),
            ("local", Modifiers::CTRL, true),
            ("before", Modifiers::CTRL, false),
            ("stale", Modifiers::CTRL, false),
            ("unresolved", Modifiers::CTRL, false),
            ("valid", Modifiers::CTRL | Modifiers::SHIFT, false),
            ("valid", Modifiers::CTRL | Modifiers::ALT, false),
        ] {
            let ctx = egui::Context::default();
            let mut reader = Reader::new(text);
            let symbol = if case == "local" { "local 0" } else { HELPER };
            let mut index = SemanticIndex::from_scip(ScipIndex {
                documents: vec![ScipDocument {
                    relative_path: reader.source.path.clone(),
                    position_encoding: 2,
                    occurrences: vec![
                        ScipOccurrence {
                            symbol: symbol.into(),
                            range: ScipRange::from_packed(&[1, start, start + 6]),
                            ..Default::default()
                        },
                        ScipOccurrence {
                            symbol: symbol.into(),
                            range: ScipRange::from_packed(&[3, 3, 9]),
                            symbol_roles: 1,
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            });
            if case == "unresolved" {
                index.definitions.clear();
            }
            if case == "stale" {
                index
                    .source_digests
                    .insert(reader.source.path.clone(), "wrong source".into());
            }
            if case == "before" {
                reader.source.revision = Some(ReviewRevision::Before);
            }
            reader.set_index(index);
            let output = reader.frame(&ctx, vec![]);
            reader.viewport.select(0..2);
            reader.column = 1;
            let selected = reader.viewport.selection;
            let (origin, galley) = text_shape(&output, call_line);
            let chars = call_line[..call_line.find("helper").unwrap()]
                .chars()
                .count()
                + 2;
            let point = origin
                + galley
                    .pos_from_ccursor(egui::text::CCursor::new(chars))
                    .center()
                    .to_vec2();
            let mut events = pointer(point, true);
            if let egui::Event::PointerButton {
                modifiers: actual, ..
            } = &mut events[1]
            {
                *actual = modifiers;
            }
            reader.frame(&ctx, events);
            if expected {
                assert_eq!(
                    reader.last_action,
                    Some(SourceViewAction::NavigateToDefinition(
                        crate::semantic::index::SemanticSymbolID::new(symbol, &reader.source.path)
                    )),
                    "{case}"
                );
                assert_eq!((reader.line, reader.column), (1, 1));
                assert_eq!(reader.viewport.selection, selected);
                assert!(!reader.viewport.dragging);
            } else {
                assert!(
                    !matches!(
                        reader.last_action,
                        Some(SourceViewAction::NavigateToDefinition(_))
                    ),
                    "{case}"
                );
                assert!(
                    matches!(
                        reader.last_action,
                        Some(SourceViewAction::CaretMoved { .. })
                    ),
                    "{case}"
                );
            }
        }
    }

    #[test]
    fn hover_dwell_and_popup_navigation_use_real_hit_regions_without_moving_the_reader() {
        use crate::semantic::{ScipDocument, ScipIndex, ScipOccurrence, ScipRange, SemanticIndex};
        const HELPER: &str = "rust-analyzer cargo app 0 helper().";
        let ctx = egui::Context::default();
        let text = "fn f() {\n helper();\n}\nfn helper() {\n open();\n}\n";
        let mut reader = Reader::new(text);
        reader.mode = SourceMode::Critical;
        reader.now = Some(0.0);
        let index = SemanticIndex::from_scip(ScipIndex {
            documents: vec![ScipDocument {
                relative_path: reader.source.path.clone(),
                occurrences: vec![
                    ScipOccurrence {
                        symbol: HELPER.into(),
                        range: ScipRange::from_packed(&[1, 1, 7]),
                        ..Default::default()
                    },
                    ScipOccurrence {
                        symbol: HELPER.into(),
                        range: ScipRange::from_packed(&[3, 3, 9]),
                        symbol_roles: 1,
                        ..Default::default()
                    },
                    ScipOccurrence {
                        symbol: "rust-analyzer cargo std 0 fs/open().".into(),
                        range: ScipRange::from_packed(&[4, 1, 5]),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        });
        reader.set_index(index);
        let initial = reader.frame(&ctx, Vec::new());
        let (origin, galley) = text_shape(&initial, " helper();");
        let point = origin
            + galley
                .pos_from_ccursor(egui::text::CCursor::new(3))
                .center()
                .to_vec2();
        reader.frame(&ctx, vec![egui::Event::PointerMoved(point)]);
        assert!(reader.viewport.hover.presentation().is_none());
        reader.now = Some(0.36);
        reader.frame(&ctx, Vec::new());
        assert!(reader.viewport.hover.presentation().is_some());
        reader.hover_ready_frame(&ctx, 0.4);
        reader.frame(&ctx, vec![key(Key::Escape, Modifiers::NONE)]);
        assert!(reader.viewport.hover.presentation().is_none());
        reader.now = Some(0.8);
        reader.frame(&ctx, Vec::new());
        assert!(reader.viewport.hover.presentation().is_none());
        reader.frame(&ctx, vec![egui::Event::PointerMoved(pos2(350.0, 330.0))]);
        reader.frame(&ctx, vec![egui::Event::PointerMoved(point)]);
        reader.now = Some(1.2);
        reader.frame(&ctx, Vec::new());
        let ready = reader.hover_ready_frame(&ctx, 1.3);
        let (title, _) = text_shape(&ready, "helper · Definition");
        assert_eq!((reader.line, reader.column), (1, 0));
        assert!(reader.viewport.selection.is_none());
        let click = title + vec2(300.0, 6.0);
        reader.frame(&ctx, pointer(click, true));
        assert!(reader.viewport.hover.presentation().is_some());
        reader.frame(&ctx, pointer(click, false));
        assert_eq!(
            reader.last_action,
            Some(SourceViewAction::NavigateToDefinition(
                crate::semantic::index::SemanticSymbolID::new(HELPER, "")
            ))
        );
        assert!(reader.viewport.hover.presentation().is_none());
        assert_eq!(reader.source.text, text);
        assert_eq!((reader.line, reader.column), (1, 0));
        assert!(reader.viewport.selection.is_none());
    }

    #[test]
    fn collapsed_critical_count_is_distinct_from_hidden_lines_and_preserves_fold_clicks() {
        use crate::critical::{annotations::CriticalAnnotations, worker::GraphInputs};
        use crate::semantic::{ScipDocument, ScipIndex, ScipOccurrence, ScipRange, SemanticIndex};
        let ctx = egui::Context::default();
        let mut reader = Reader::new("fn f() {\n open();\n}\n");
        let syntax = crate::syntax::DocumentSyntax::analyze(&reader.source);
        reader.ranges = syntax.folds;
        reader.folds.toggle_header(1, &reader.ranges);
        let mut annotations = CriticalAnnotations::default();
        annotations.start(
            GraphInputs {
                source: crate::source::loader::SourceRead::Buffer(Arc::new(reader.source.clone())),
                buffer: Arc::new(reader.source.clone()),
                calls: Arc::new(syntax.calls),
                index: Arc::new(SemanticIndex::from_scip(ScipIndex {
                    documents: vec![ScipDocument {
                        relative_path: reader.source.path.clone(),
                        occurrences: vec![ScipOccurrence {
                            symbol: "rust-analyzer cargo std 0 fs/open().".into(),
                            range: ScipRange::from_packed(&[1, 1, 5]),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                })),
                overrides: Default::default(),
            },
            reader.ranges.clone(),
        );
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while annotations.is_running() {
            annotations.poll();
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        reader.annotations = annotations;
        let normal = reader.frame(&ctx, Vec::new());
        let (_, hidden) = text_shape(&normal, "2");
        assert_eq!(hidden.job.text, "2");
        reader.mode = SourceMode::Critical;
        let critical = reader.frame(&ctx, Vec::new());
        // The source line number is also 1; find the count in its proportional font.
        let count = critical
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text)
                    if text.galley.job.text == "1"
                        && text.galley.job.sections[0].format.font_id.family
                            == egui::FontFamily::Proportional =>
                {
                    Some(text.pos)
                }
                _ => None,
            })
            .expect("reachable critical count");
        reader.frame(&ctx, pointer(count + vec2(2.0, 2.0), true));
        reader.frame(&ctx, pointer(count + vec2(2.0, 2.0), false));
        assert!(!reader.folds.is_header_folded(1, &reader.ranges));
        assert_eq!(reader.source.text, "fn f() {\n open();\n}\n");
        assert_eq!((reader.line, reader.column), (1, 0));
    }

    #[test]
    fn critical_projection_splits_colored_spans_without_changing_source_geometry_or_bytes() {
        let ctx = egui::Context::default();
        let text = r#"fn f() { let s = "😀"; unknown(); }"#;
        let mut reader = Reader::new(text);
        let color = Color32::from_rgb(200, 130, 80);
        reader.highlights = vec![vec![HighlightedSpan {
            bytes: 0..text.len(),
            text: text.into(),
            color,
            is_bold: false,
            is_italic: false,
        }]];
        let normal = reader.frame(&ctx, Vec::new());
        let (_, normal) = text_shape(&normal, text);
        let name = text.find("unknown").unwrap();
        reader.projection = Some(crate::critical::SourceProjection::new(
            &reader.source,
            vec![0..name, name + 7..text.len()],
        ));
        reader.mode = SourceMode::Critical;
        let output = reader.frame(&ctx, Vec::new());
        let (_, critical) = text_shape(&output, text);
        assert_eq!(critical.job.text, text);
        assert_eq!(critical.size(), normal.size());
        let at = |galley: &egui::Galley, byte| {
            galley
                .job
                .sections
                .iter()
                .find(|section| section.byte_range.contains(&byte))
                .unwrap()
                .format
                .color
        };
        assert_eq!(at(&critical, name), color);
        assert_ne!(at(&critical, 0), color);
        assert_ne!(at(&critical, name + 7), color);
        reader.mode = SourceMode::Normal;
        let output = reader.frame(&ctx, Vec::new());
        assert_eq!(at(&text_shape(&output, text).1, 0), color);
        reader.mode = SourceMode::Critical;
        reader.source = SourceBuffer::from_str("fixture.rs", &text.replace("unknown", "altered"));
        let output = reader.frame(&ctx, Vec::new());
        let (_, stale) = text_shape(&output, &text.replace("unknown", "altered"));
        assert_eq!(at(&stale, 0), color); // Wrong-content projection is ignored.
    }

    #[test]
    fn coalesced_navigation_selection_and_copy_follow_input_order_at_each_scale() {
        let text = "aé中🦀🫠z\ntail\n";
        for scale in [1.0, 1.5, 2.0] {
            let ctx = egui::Context::default();
            let mut definitions = egui::FontDefinitions::default();
            reader_fonts::append_fallbacks(&mut definitions);
            ctx.set_fonts(definitions);
            ctx.set_pixels_per_point(scale);
            for (events, initial, copied, column) in [
                (
                    vec![
                        key(Key::Home, Modifiers::NONE),
                        key(Key::End, Modifiers::SHIFT),
                        egui::Event::Copy,
                    ],
                    2,
                    "aé中🦀🫠z",
                    6,
                ),
                (
                    vec![
                        key(Key::ArrowRight, Modifiers::NONE),
                        key(Key::ArrowRight, Modifiers::NONE),
                        key(Key::ArrowRight, Modifiers::NONE),
                        key(Key::ArrowRight, Modifiers::SHIFT),
                        key(Key::ArrowRight, Modifiers::SHIFT),
                        key(Key::C, Modifiers::CTRL),
                    ],
                    0,
                    "🦀🫠",
                    5,
                ),
                (
                    vec![key(Key::A, Modifiers::CTRL), egui::Event::Copy],
                    0,
                    text,
                    0,
                ),
            ] {
                let mut reader = Reader::new(text);
                reader.column = initial;
                reader.frame(&ctx, Vec::new());
                let output = reader.frame(&ctx, events);
                assert_eq!(output.platform_output.copied_text, copied, "scale {scale}");
                assert_eq!((reader.line, reader.column), (1, column));
            }
        }
    }

    #[test]
    fn wrapped_rows_measure_hit_unicode_and_drag_copy_original_crlf() {
        let ctx = egui::Context::default();
        let long = "é😀 variable ".repeat(8);
        let source = format!("{long}\r\nsecond line\r\nlast");
        let mut reader = Reader::new(&source);
        let output = reader.frame(&ctx, Vec::new());
        let (origin, first) = text_shape(&output, &long);
        let (second_origin, _) = text_shape(&output, "second line");
        assert!(first.rows.len() > 1);
        assert!(second_origin.y >= origin.y + first.size().y);
        let start = origin
            + first
                .pos_from_ccursor(egui::text::CCursor::new(2))
                .center()
                .to_vec2();
        reader.frame(&ctx, pointer(start, true));
        assert_eq!(reader.column, 2);
        let end = second_origin + vec2(0.1, 5.0);
        reader.frame(&ctx, vec![egui::Event::PointerMoved(end)]);
        reader.frame(&ctx, pointer(end, false));
        assert_eq!(reader.line, 2);
        let output = reader.frame(&ctx, vec![egui::Event::Copy]);
        assert_eq!(
            output.platform_output.copied_text,
            source["é😀".len()..long.len() + 2]
        );
        // Vertical movement follows a shaped visual row before changing source line.
        reader.frame(&ctx, pointer(start, true));
        reader.frame(&ctx, pointer(start, false));
        reader.frame(&ctx, vec![key(Key::ArrowDown, Modifiers::NONE)]);
        assert_eq!(reader.line, 1);
        assert!(reader.column > 2);
    }

    #[test]
    fn selection_copy_survives_folding_and_other_controls_own_the_keyboard() {
        let ctx = egui::Context::default();
        let mut reader = Reader::new("one\r\nhidden\r\nthree\r\nlast");
        reader.frame(&ctx, Vec::new());
        reader.frame(&ctx, vec![key(Key::A, Modifiers::CTRL)]);
        reader.ranges.push(FoldRange {
            start_line: 1,
            end_line: 3,
            header_preview: "one".into(),
            byte_range: reader.source.line_byte_range(1).end..reader.source.line_byte_range(3).end,
            ..Default::default()
        });
        reader.folds.toggle_header(1, &reader.ranges);
        let output = reader.frame(&ctx, vec![egui::Event::Copy]);
        assert_eq!(output.platform_output.copied_text, reader.source.text);
        assert_eq!(reader.viewport.visible.len(), 2);
        let other = Id::new("other_control");
        ctx.memory_mut(|memory| memory.request_focus(other));
        let output = reader.frame(
            &ctx,
            vec![key(Key::ArrowDown, Modifiers::NONE), egui::Event::Copy],
        );
        assert_eq!(reader.line, 1);
        assert!(output.platform_output.copied_text.is_empty());
    }

    #[test]
    fn long_line_fragment_navigation_and_zoom_reveal_canonical_caret() {
        let ctx = egui::Context::default();
        let text = "é".repeat(FRAGMENT_BYTES * 3);
        let mut reader = Reader::new(&text);
        reader.column = 5000;
        reader.target = Some(1);
        reader.frame(&ctx, Vec::new());
        assert!(reader
            .viewport
            .fragments
            .iter()
            .all(|part| part.bytes.len() <= FRAGMENT_BYTES));
        assert!(reader.viewport.scroll_y > 0.0);
        let height = reader.viewport.rows.total_height();
        reader.font_size = 24.0;
        reader.target = Some(1);
        reader.frame(&ctx, Vec::new());
        assert!(reader.viewport.rows.total_height() > height);
        assert_eq!(reader.column, 5000);
        assert_eq!(reader.line, 1);
    }
    #[test]
    fn hidden_test_headers_offer_a_working_reveal_control_and_copy_retains_bytes() {
        let ctx = egui::Context::default();
        let text = "fn production() {}\n#[cfg(test)]\nmod checks {\n fn hidden_only() {}\n}\nfn tail() {}\n";
        let mut reader = Reader::new(text);
        reader.test_code.load(
            &reader.source,
            crate::syntax::TestRegionDetector::detect(&reader.source),
            true,
        );
        let output = reader.frame(&ctx, Vec::new());
        let painted = |output: &egui::FullOutput, needle: &str| {
            output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text.contains(needle)))
        };
        assert!(painted(&output, "Test code · 3 lines hidden"));
        assert!(!painted(&output, "hidden_only"));
        assert!(painted(&output, "fn tail"));
        reader.frame(&ctx, vec![key(Key::A, Modifiers::CTRL)]);
        let copied = reader.frame(&ctx, vec![egui::Event::Copy]);
        assert_eq!(copied.platform_output.copied_text, text);
        let (origin, label) = text_shape(&output, "Test code · 3 lines hidden");
        let point = origin + label.size() / 2.0;
        reader.frame(&ctx, pointer(point, true));
        reader.frame(&ctx, pointer(point, false));
        let output = reader.frame(&ctx, Vec::new());
        assert!(painted(&output, "hidden_only"));
        assert!(painted(&output, "Hide test code"));
    }
}
