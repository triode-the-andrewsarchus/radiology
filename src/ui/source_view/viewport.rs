//! Measured source viewport. Coordinates always refer to original UTF-8 bytes.

use super::{paint_match, SourceView, SourceViewAction};
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
use reader_text::{source_fragments, ByteSelection, RowHeights, SourceFragment};
use std::{collections::HashSet, ops::Range, sync::Arc};

const FRAGMENT_BYTES: usize = 4096;
const TEXT_INSET: f32 = 12.0;

/// Per-view geometry and canonical selection, separate from immutable source.
#[derive(Default)]
pub struct SourceViewport {
    document: Option<(String, Option<ReviewRevision>, u64)>,
    fragments: Vec<SourceFragment>,
    visible: Vec<usize>,
    folded: HashSet<usize>,
    rows: RowHeights,
    scroll_y: f32,
    scroll_x: f32,
    anchor_after_reset: Option<usize>,
    selection: Option<ByteSelection>,
    dragging: bool,
    max_bytes: usize,
}

pub(super) struct CanvasContent<'a> {
    pub theme: &'a AppTheme,
    pub mode: SourceMode,
    pub buffer: &'a SourceBuffer,
    pub highlights: &'a [Vec<HighlightedSpan>],
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

    /// Replaces the selection with original-byte endpoints (for local find).
    pub fn select(&mut self, range: Range<usize>) {
        self.selection = Some(ByteSelection {
            anchor: range.start,
            focus: range.end,
        });
    }

    fn sync(&mut self, content: &CanvasContent<'_>) {
        let source = content.buffer;
        let key = (
            source.path.clone(),
            source.revision,
            source.content_identity(),
        );
        let changed = self.document.as_ref() != Some(&key);
        if changed {
            self.fragments = source_fragments(&source.text, FRAGMENT_BYTES);
            self.max_bytes = self
                .fragments
                .iter()
                .map(|part| part.bytes.len())
                .max()
                .unwrap_or(0);
            self.selection = None;
            self.dragging = false;
            self.scroll_y = 0.0;
            self.scroll_x = 0.0;
            self.document = Some(key);
        }
        if changed || self.folded != content.fold_store.folded_lines {
            let old_fragment = self
                .visible
                .get(self.rows.row_at(self.scroll_y))
                .copied()
                .unwrap_or(0);
            self.visible = self
                .fragments
                .iter()
                .enumerate()
                .filter_map(|(index, part)| {
                    (!content.fold_store.is_line_hidden(part.line, content.folds)).then_some(index)
                })
                .collect();
            self.anchor_after_reset = Some(
                self.visible
                    .partition_point(|&index| index <= old_fragment)
                    .saturating_sub(1),
            );
            self.folded.clone_from(&content.fold_store.folded_lines);
            self.rows = RowHeights::default();
        }
    }

    pub(super) fn show(
        &mut self,
        ui: &mut Ui,
        mut content: CanvasContent<'_>,
    ) -> Option<SourceViewAction> {
        self.sync(&content);
        let geometry = CanvasGeometry::new(ui, &content);
        let (scroll_y, mut action) = self.prepare(ui, &mut content, &geometry);
        let response = ui.interact(
            geometry.code_rect,
            Self::focus_id(),
            Sense::click_and_drag(),
        );
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
                if let Some(event) =
                    self.render_rows(ui, viewport, &geometry, &response, &mut content)
                {
                    action = Some(event);
                }
            });
        self.scroll_y = output.state.offset.y;
        self.scroll_x = output.state.offset.x;
        self.finish_drag(ui, geometry.code_rect);
        self.context_menu(&response, &content).or(action)
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
            if let Some(byte) =
                self.keyboard(ui, content, &geometry.font, geometry.width, geometry.height)
            {
                action = Some(move_caret(content, byte));
                keyboard_reveal = true;
            }
        }
        if let Some(line) = content.target.take() {
            let caret = content
                .buffer
                .byte_offset(*content.caret_line, *content.caret_col);
            let byte = if content.buffer.line_at_byte(caret) == line {
                caret
            } else {
                content.buffer.byte_offset(line, 0)
            };
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
            if self
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
        response: &egui::Response,
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
            let text = &content.buffer.text[fragment.bytes.clone()];
            let galley = layout(ui, content, &fragment, &geometry.font, geometry.width);
            let row_height = (galley.size().y + 4.0).max(geometry.minimum_height);
            self.rows.set_height(row, row_height);
            let text_origin = origin + vec2(SourceView::GUTTER_WIDTH + TEXT_INSET, y + 2.0);
            let rect = Rect::from_min_size(
                pos2(geometry.code_rect.left(), origin.y + y),
                vec2(geometry.code_rect.width(), row_height),
            );
            self.paint_row(&painter, &galley, text_origin, rect, &fragment, content);
            if fragment.first_in_line {
                if let Some(fold) =
                    paint_gutter(ui, &gutter, geometry.surface, rect, &fragment, content)
                {
                    content.fold_store.toggle(fold);
                    action = Some(SourceViewAction::ToggleFold(fold));
                }
            }
            if let Some(pointer) = response
                .interact_pointer_pos()
                .or_else(|| ui.ctx().pointer_interact_pos())
            {
                let target_y = pointer
                    .y
                    .clamp(geometry.code_rect.top(), geometry.code_rect.bottom() - 0.1);
                if target_y >= rect.top() && target_y < rect.bottom() {
                    let byte = hit_byte(&galley, pointer - text_origin, text, fragment.bytes.start);
                    if let Some(event) = self.pointer_input(
                        ui,
                        response,
                        byte,
                        geometry.code_rect.contains(pointer),
                        content,
                    ) {
                        action = Some(event);
                    }
                    if response.double_clicked() {
                        action = Some(SourceViewAction::PinAtByte(byte));
                    }
                }
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
        response.context_menu(|ui| {
            if ui.button("Copy Selection").clicked() {
                self.copy(ui.ctx(), content.buffer);
                ui.close_menu();
            }
            if ui.button("Copy Line").clicked() {
                ui.output_mut(|output| {
                    output.copied_text = content.buffer.line_text(*content.caret_line).to_owned()
                });
                ui.close_menu();
            }
            if ui.button("Go to Definition (F12 / Return)").clicked() {
                action = Some(SourceViewAction::JumpToDefinition);
                ui.close_menu();
            }
        });
        action
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
            response.request_focus();
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
        self.rows
            .set_height(row, (galley.size().y + 4.0).max(minimum_height));
        let local = byte.saturating_sub(part.bytes.start).min(part.bytes.len());
        let chars = content.buffer.text[part.bytes.start..part.bytes.start + local]
            .chars()
            .count();
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
            return;
        }
        let started = std::time::Instant::now();
        for _ in 0..64 {
            let row = self.rows.next_measurement;
            if row >= self.visible.len() {
                break;
            }
            let part = &self.fragments[self.visible[row]];
            let text = &content.buffer.text[part.bytes.clone()];
            let galley = ui
                .fonts(|fonts| fonts.layout(text.to_owned(), font.clone(), Color32::WHITE, width));
            self.rows
                .set_height(row, (galley.size().y + 4.0).max(height));
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
    ) -> Option<usize> {
        let copy = ui.input_mut(|input| {
            input.consume_key(Modifiers::COMMAND, Key::C)
                || input.consume_key(Modifiers::CTRL, Key::C)
                || input
                    .events
                    .iter()
                    .any(|event| matches!(event, egui::Event::Copy))
        });
        if copy {
            self.copy(ui.ctx(), content.buffer);
        }
        if ui.input_mut(|input| {
            input.consume_key(Modifiers::COMMAND, Key::A)
                || input.consume_key(Modifiers::CTRL, Key::A)
        }) {
            self.select(0..content.buffer.byte_len());
            ui.ctx().request_repaint();
            return None;
        }
        let key = ui.input_mut(|input| {
            let event = input.events.iter().position(|event| matches!(event, egui::Event::Key { key: Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight | Key::Home | Key::End | Key::PageUp | Key::PageDown, pressed: true, modifiers, .. } if !modifiers.alt && !modifiers.command && !modifiers.ctrl));
            event.map(|index| input.events.remove(index))
        });
        let Some(egui::Event::Key { key, modifiers, .. }) = key else {
            return None;
        };
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
        if modifiers.shift {
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
        Some(target)
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
        let text = &content.buffer.text[part.bytes.clone()];
        let local = byte.saturating_sub(part.bytes.start).min(text.len());
        let galley = layout(ui, content, part, font, width);
        let cursor = galley.from_ccursor(egui::text::CCursor::new(text[..local].chars().count()));
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
            return part.bytes.start
                + text
                    .char_indices()
                    .nth(next.ccursor.index)
                    .map_or(text.len(), |(byte, _)| byte);
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
            &content.buffer.text[part.bytes.clone()],
            part.bytes.start,
        )
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
        let theme = content.theme;
        if part.line == *content.caret_line {
            painter.rect_filled(rect, 0.0, theme.bg_active_line);
        }
        if content
            .changes
            .iter()
            .any(|range| range.contains(&part.line))
        {
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
        let text = &content.buffer.text[part.bytes.clone()];
        for (range, active) in content.find.matches(part.bytes.clone()) {
            paint_match(
                painter,
                galley,
                origin,
                text,
                part.bytes.start,
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
                    text,
                    part.bytes.start,
                    range,
                    theme.accent.gamma_multiply(0.35),
                );
            }
        }
        for group in &content.pins.groups {
            for range in group
                .ranges
                .iter()
                .filter(|range| range.start < part.bytes.end && range.end > part.bytes.start)
            {
                paint_match(
                    painter,
                    galley,
                    origin,
                    text,
                    part.bytes.start,
                    range.clone(),
                    theme.occurrence_tint(group.color).gamma_multiply(0.35),
                );
            }
        }
        painter.galley(origin, Arc::new(galley.clone()), theme.text_primary);
        let byte = content
            .buffer
            .byte_offset(*content.caret_line, *content.caret_col);
        if self
            .row_for_byte(byte)
            .is_some_and(|(_, owner)| owner == part)
        {
            let chars = text[..byte.saturating_sub(part.bytes.start).min(text.len())]
                .chars()
                .count();
            let cursor = galley.from_ccursor(egui::text::CCursor::new(chars));
            let caret = galley.pos_from_cursor(&cursor).translate(origin.to_vec2());
            painter.line_segment(
                [caret.left_top(), caret.left_bottom()],
                Stroke::new(1.5_f32, theme.accent),
            );
        }
    }
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

fn hit_byte(galley: &egui::Galley, position: egui::Vec2, text: &str, start: usize) -> usize {
    let index = galley.cursor_from_pos(position).ccursor.index;
    start
        + text
            .char_indices()
            .nth(index)
            .map_or(text.len(), |(byte, _)| byte)
}

fn keyboard_target(
    key: Key,
    byte: usize,
    buffer: &SourceBuffer,
    parts: &[SourceFragment],
    visible: &[usize],
    page: isize,
) -> usize {
    let line = buffer.line_at_byte(byte);
    let column = buffer.column_at_byte(byte);
    let visible_lines: Vec<usize> = visible
        .iter()
        .filter_map(|&index| parts[index].first_in_line.then_some(parts[index].line))
        .collect();
    let position = visible_lines.partition_point(|&candidate| candidate < line);
    match key {
        Key::ArrowLeft if byte <= buffer.line_byte_range(line).start => visible_lines
            .get(position.saturating_sub(1))
            .map_or(byte, |&previous| buffer.line_byte_range(previous).end),
        Key::ArrowRight if byte >= buffer.line_byte_range(line).end => visible_lines
            .get(position + 1)
            .map_or(byte, |&next| buffer.line_byte_range(next).start),
        Key::ArrowLeft => buffer.text[..byte]
            .char_indices()
            .next_back()
            .map_or(0, |(index, _)| index),
        Key::ArrowRight => byte + buffer.text[byte..].chars().next().map_or(0, char::len_utf8),
        Key::Home => buffer.line_byte_range(line).start,
        Key::End => buffer.line_byte_range(line).end,
        _ => {
            let step = match key {
                Key::ArrowUp => -1,
                Key::ArrowDown => 1,
                Key::PageUp => -page,
                _ => page,
            };
            visible_lines
                .get(
                    position
                        .saturating_add_signed(step)
                        .min(visible_lines.len().saturating_sub(1)),
                )
                .map_or(byte, |&line| buffer.byte_offset(line, column))
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
    let line_start = content.buffer.byte_offset(part.line, 0);
    if let Some(spans) = content.highlights.get(part.line - 1) {
        let first = spans.partition_point(|span| line_start + span.bytes.end <= part.bytes.start);
        for span in &spans[first..] {
            let offset = line_start + span.bytes.start;
            let end = line_start + span.bytes.end;
            let start = offset.max(part.bytes.start);
            let stop = end.min(part.bytes.end);
            if start < stop {
                let mut color = span.color;
                if content.mode == SourceMode::Critical
                    && color != content.theme.accent
                    && color != Color32::from_rgb(224, 108, 117)
                {
                    color = color.linear_multiply(if content.theme.is_dark { 0.38 } else { 0.30 });
                }
                job.append(
                    &content.buffer.text[start..stop],
                    0.0,
                    TextFormat::simple(font.clone(), color),
                );
            }
            if end >= part.bytes.end {
                break;
            }
        }
    }
    if job.text.is_empty() {
        job.append(
            &content.buffer.text[part.bytes.clone()],
            0.0,
            TextFormat::simple(font.clone(), content.theme.text_primary),
        );
    }
    ui.fonts(|fonts| fonts.layout_job(job))
}

fn paint_gutter(
    ui: &mut Ui,
    painter: &egui::Painter,
    surface: Rect,
    row: Rect,
    part: &SourceFragment,
    content: &CanvasContent<'_>,
) -> Option<usize> {
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
    let fold = content
        .folds
        .iter()
        .find(|fold| fold.start_line == part.line)?;
    let rect = Rect::from_min_size(
        pos2(surface.left() + 4.0, row.top() + 3.0),
        vec2(28.0, 16.0),
    );
    let collapsed = content.fold_store.is_folded(part.line);
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
    if collapsed {
        painter.text(
            rect.right_top(),
            egui::Align2::RIGHT_TOP,
            fold.end_line.saturating_sub(fold.start_line).to_string(),
            FontId::proportional(10.0),
            theme.text_secondary,
        );
    }
    ui.interact(rect, Id::new(("source_fold", part.line)), Sense::click())
        .clicked()
        .then_some(part.line)
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
            }
        }

        fn frame(&mut self, ctx: &egui::Context, events: Vec<egui::Event>) -> egui::FullOutput {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(400.0, 360.0))),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        self.viewport.show(
                            ui,
                            CanvasContent {
                                theme: &AppTheme::dark(),
                                mode: SourceMode::Normal,
                                buffer: &self.source,
                                highlights: &[],
                                caret_line: &mut self.line,
                                caret_col: &mut self.column,
                                folds: &self.ranges,
                                fold_store: &mut self.folds,
                                changes: &[],
                                pins: &OccurrencePinStore::default(),
                                wrap: self.wrap,
                                font_size: self.font_size,
                                target: &mut self.target,
                                keyboard_enabled: true,
                                find: &mut self.find,
                            },
                        );
                    });
                },
            )
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
        });
        reader.folds.toggle(1);
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
}
