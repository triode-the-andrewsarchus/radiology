//! Central source code viewer with syntax highlighting, diff markers, and gutter controls.
//!
//! Handles rendering line numbers, code folding chevrons, Syntect syntax-highlighted spans,
//! active caret line highlighting, pull request addition/deletion badges, and breadcrumb headers.

use crate::pull_request::diff::FileLineChanges;
use crate::source::buffer::SourceBuffer;
use crate::source::location::ReviewRevision;
use crate::source::selection::OccurrencePinStore;
use crate::syntax::folds::{FoldRange, FoldStore};
use crate::syntax::highlighter::HighlightedSpan;
use crate::syntax::outline::DocumentSymbol;
use crate::theme::AppTheme;
use egui::{Key, Rect, RichText, ScrollArea, Stroke, TextStyle, Ui, Vec2};

/// User interactions and navigation events produced within the [`SourceView`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceViewAction {
    /// Caret position moved to a new line and column.
    CaretMoved {
        /// 1-based line number.
        line: usize,
        /// 0-based column offset.
        column: usize,
        /// 0-based byte offset in the buffer.
        byte_offset: usize,
    },
    /// Requests jumping to the definition of the symbol under the current caret.
    JumpToDefinition,
    /// Requests expanding or collapsing the code fold starting at the given line number.
    ToggleFold(usize),
    /// Requests jumping to the next modified hunk in the active pull request review.
    NextChangeBlock,
    /// Requests jumping to the previous modified hunk in the active pull request review.
    PrevChangeBlock,
    /// Switches the active review revision (Before vs After).
    ToggleRevision(ReviewRevision),
    /// Requests pinning all occurrences of a symbol for visual highlighting.
    PinOccurrence {
        /// Symbol text name.
        symbol: String,
        /// Byte ranges where the symbol appears in the current buffer.
        ranges: Vec<std::ops::Range<usize>>,
    },
}

/// Renders the primary code editor and reading canvas.
pub struct SourceView;

impl SourceView {
    /// Width in points reserved for the gutter area (line numbers, fold chevrons, diff stripes).
    pub const GUTTER_WIDTH: f32 = 72.0;

    /// Renders the source view including path breadcrumbs, code lines, gutter, and status bar.
    ///
    /// # Arguments
    ///
    /// * `ui` - EGUI UI layout context.
    /// * `theme` - Visual color palette.
    /// * `buffer` - Source buffer containing file text, lines, and revision info.
    /// * `highlighted_lines` - Pre-computed syntax highlighted token spans for each buffer line.
    /// * `caret_line` - Mutable 1-based line number of the user caret.
    /// * `caret_col` - Mutable 0-based column offset of the user caret.
    /// * `fold_ranges` - Detected foldable AST syntax nodes in the document.
    /// * `fold_store` - Set of currently collapsed fold start lines.
    /// * `line_changes` - Pull request line change markers for before/after diff overlays.
    /// * `pin_store` - Highlighted symbol occurrence pins.
    /// * `enclosing_symbols` - Hierarchical list of symbols enclosing the current caret position.
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut Ui,
        theme: &AppTheme,
        buffer: &SourceBuffer,
        highlighted_lines: &[Vec<HighlightedSpan>],
        caret_line: &mut usize,
        caret_col: &mut usize,
        fold_ranges: &[FoldRange],
        fold_store: &mut FoldStore,
        line_changes: Option<&FileLineChanges>,
        pin_store: &mut OccurrencePinStore,
        enclosing_symbols: &[DocumentSymbol],
    ) -> Option<SourceViewAction> {
        let mut action = None;

        ui.vertical(|ui| {
            // Header / Breadcrumbs Bar
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;

                // Path breadcrumbs
                let parts: Vec<&str> = buffer.path.split('/').collect();
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        ui.colored_label(theme.text_muted, "›");
                    }
                    if i == parts.len() - 1 {
                        ui.label(RichText::new(*part).strong());
                    } else {
                        ui.colored_label(theme.text_secondary, *part);
                    }
                }

                // Enclosing symbol breadcrumb
                if let Some(last_sym) = enclosing_symbols.last() {
                    ui.colored_label(theme.text_muted, "›");
                    ui.colored_label(theme.accent, RichText::new(&last_sym.name).strong());
                }

                // PR Diff block jump controls if in review
                if let Some(diff) = line_changes {
                    if !diff.hunks.is_empty() {
                        ui.separator();
                        if ui.button("▲ Prev Change").clicked() {
                            action = Some(SourceViewAction::PrevChangeBlock);
                        }
                        if ui.button("▼ Next Change").clicked() {
                            action = Some(SourceViewAction::NextChangeBlock);
                        }
                    }
                }

                // Revision indicator (Before / After)
                if let Some(rev) = buffer.revision {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.colored_label(theme.accent, format!("[{}]", rev.title()));
                    });
                }
            });

            ui.separator();

            // Keyboard navigation (Arrow keys & Enter)
            let max_lines = buffer.line_count();
            if ui.ctx().input(|i| i.key_pressed(Key::ArrowDown)) && *caret_line < max_lines {
                let mut next_line = *caret_line + 1;
                while next_line < max_lines && fold_store.is_line_hidden(next_line, fold_ranges) {
                    next_line += 1;
                }
                *caret_line = next_line;
                let offset = buffer.byte_offset(*caret_line, *caret_col);
                action = Some(SourceViewAction::CaretMoved {
                    line: *caret_line,
                    column: *caret_col,
                    byte_offset: offset,
                });
            }
            if ui.ctx().input(|i| i.key_pressed(Key::ArrowUp)) && *caret_line > 1 {
                let mut prev_line = *caret_line - 1;
                while prev_line > 1 && fold_store.is_line_hidden(prev_line, fold_ranges) {
                    prev_line -= 1;
                }
                *caret_line = prev_line;
                let offset = buffer.byte_offset(*caret_line, *caret_col);
                action = Some(SourceViewAction::CaretMoved {
                    line: *caret_line,
                    column: *caret_col,
                    byte_offset: offset,
                });
            }
            if ui.ctx().input(|i| i.key_pressed(Key::Enter)) {
                action = Some(SourceViewAction::JumpToDefinition);
            }

            // Central Code Area with Gutter
            let available_height = (ui.available_height() - 28.0).max(100.0);
            let row_height = 20.0;

            // Build visible line numbers skipping collapsed folds
            let visible_lines: Vec<usize> = (1..=buffer.line_count())
                .filter(|&l| !fold_store.is_line_hidden(l, fold_ranges))
                .collect();
            let total_rows = visible_lines.len();

            let changed_lines = line_changes
                .map(|c| c.changed_lines(buffer.revision.unwrap_or(ReviewRevision::After)))
                .unwrap_or_default();

            ScrollArea::both()
                .auto_shrink([false, false])
                .max_height(available_height)
                .show_rows(ui, row_height, total_rows, |ui, row_range| {
                    let clip_rect = ui.clip_rect();
                    let painter = ui.painter().with_clip_rect(clip_rect);
                    let row_width = ui.available_width().max(400.0);

                    for idx in row_range {
                        if idx >= visible_lines.len() {
                            break;
                        }
                        let line_num = visible_lines[idx];
                        let is_caret_line = line_num == *caret_line;
                        let line_text = buffer.line_text(line_num);

                        let (rect, line_response) = ui.allocate_exact_size(
                            Vec2::new(row_width, row_height),
                            egui::Sense::click_and_drag(),
                        );

                        // Draw Gutter
                        let gutter_rect = Rect::from_min_size(
                            rect.min,
                            Vec2::new(Self::GUTTER_WIDTH, row_height),
                        );
                        painter.rect_filled(gutter_rect, 0.0, theme.bg_gutter);

                        // Fold Chevron in Gutter
                        let mut chevron_clicked = false;
                        if let Some(_fold) = fold_ranges.iter().find(|f| f.start_line == line_num) {
                            let is_folded = fold_store.is_folded(line_num);
                            let chevron = if is_folded { "▶" } else { "▼" };
                            let chevron_pos = gutter_rect.left_center() + Vec2::new(6.0, 0.0);
                            painter.text(
                                chevron_pos,
                                egui::Align2::LEFT_CENTER,
                                chevron,
                                TextStyle::Small.resolve(ui.style()),
                                theme.text_secondary,
                            );

                            let chevron_rect =
                                Rect::from_center_size(chevron_pos, Vec2::new(16.0, 16.0));
                            if ui.rect_contains_pointer(chevron_rect)
                                && ui.input(|i| i.pointer.primary_clicked())
                            {
                                fold_store.toggle(line_num);
                                action = Some(SourceViewAction::ToggleFold(line_num));
                                chevron_clicked = true;
                            }
                        }

                        // Caret click: calculate actual character column from mouse position
                        if line_response.clicked() && !chevron_clicked {
                            *caret_line = line_num;
                            let mut col = 0;
                            if let Some(pos) = line_response.interact_pointer_pos() {
                                let code_start_x = rect.min.x + Self::GUTTER_WIDTH + 8.0;
                                if pos.x > code_start_x {
                                    let char_width = 8.5;
                                    col = ((pos.x - code_start_x) / char_width).floor() as usize;
                                    col = col.min(line_text.chars().count());
                                }
                            }
                            *caret_col = col;
                            let offset = buffer.byte_offset(line_num, col);
                            action = Some(SourceViewAction::CaretMoved {
                                line: line_num,
                                column: col,
                                byte_offset: offset,
                            });
                        }

                        // Background highlighting for active caret line
                        if is_caret_line {
                            painter.rect_filled(rect, 0.0, theme.bg_active_line);
                        }

                        // Diff line highlighting
                        let is_diff_changed = changed_lines.iter().any(|r| r.contains(&line_num));
                        if is_diff_changed {
                            let diff_bg = if buffer.revision == Some(ReviewRevision::Before) {
                                theme.diff_del
                            } else {
                                theme.diff_add
                            };
                            painter.rect_filled(rect, 0.0, diff_bg);
                        }

                        // Line number rule: print every 5th line or active caret line
                        let should_print_number = line_num.is_multiple_of(5) || is_caret_line;
                        if should_print_number {
                            let num_color = if is_caret_line {
                                theme.accent
                            } else {
                                theme.text_muted
                            };
                            painter.text(
                                gutter_rect.right_center() - Vec2::new(12.0, 0.0),
                                egui::Align2::RIGHT_CENTER,
                                line_num.to_string(),
                                TextStyle::Monospace.resolve(ui.style()),
                                num_color,
                            );
                        }

                        // Gutter right border
                        painter.line_segment(
                            [gutter_rect.right_top(), gutter_rect.right_bottom()],
                            Stroke::new(1.0, theme.border),
                        );

                        // Draw pre-highlighted code spans strictly clipped
                        let code_pos =
                            rect.left_center() + Vec2::new(Self::GUTTER_WIDTH + 8.0, 0.0);
                        if let Some(spans) = highlighted_lines.get(line_num.saturating_sub(1)) {
                            let mut x_offset = 0.0;
                            let font_id = TextStyle::Monospace.resolve(ui.style());
                            for span in spans {
                                let galley = painter.layout_no_wrap(
                                    span.text.clone(),
                                    font_id.clone(),
                                    span.color,
                                );
                                let span_pos = code_pos + Vec2::new(x_offset, 0.0);
                                painter.galley(
                                    span_pos - Vec2::new(0.0, galley.size().y / 2.0),
                                    galley.clone(),
                                    span.color,
                                );
                                x_offset += galley.size().x;
                            }
                        } else {
                            painter.text(
                                code_pos,
                                egui::Align2::LEFT_CENTER,
                                line_text,
                                TextStyle::Monospace.resolve(ui.style()),
                                theme.text_primary,
                            );
                        }

                        // Draw occurrence pins if any exist on this line
                        if !pin_store.is_empty() {
                            let line_start_byte = buffer.byte_offset(line_num, 0);
                            let line_end_byte = buffer.byte_offset(line_num + 1, 0);
                            for group in &pin_store.groups {
                                for range in &group.ranges {
                                    if range.start < line_end_byte && range.end > line_start_byte {
                                        let col_start = range.start.saturating_sub(line_start_byte);
                                        let char_w = 8.5;
                                        let pin_x = code_pos.x + col_start as f32 * char_w;
                                        let pin_w = (range.end - range.start) as f32 * char_w;
                                        let pin_rect = Rect::from_min_size(
                                            egui::pos2(pin_x, rect.min.y + 2.0),
                                            egui::vec2(pin_w.max(char_w), row_height - 4.0),
                                        );
                                        let color = theme.accent.linear_multiply(0.3);
                                        painter.rect_stroke(pin_rect, 2.0, Stroke::new(1.0, color));
                                    }
                                }
                            }
                        }
                    }
                });

            // Status Bar at bottom
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new(&buffer.path).monospace());
                ui.separator();
                ui.colored_label(
                    theme.text_secondary,
                    format!("Line {}, Col {}", *caret_line, *caret_col),
                );
                ui.separator();
                ui.colored_label(
                    theme.text_secondary,
                    format!("{} lines", buffer.line_count()),
                );
                ui.separator();
                ui.colored_label(theme.accent, "UTF-8");
            });
        });

        action
    }
}
