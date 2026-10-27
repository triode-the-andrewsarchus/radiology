use egui::{Key, Rect, RichText, ScrollArea, Stroke, TextStyle, Ui, Vec2};
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxReference;
use crate::pull_request::diff::FileLineChanges;
use crate::source::buffer::SourceBuffer;
use crate::source::location::ReviewRevision;
use crate::source::selection::OccurrencePinStore;
use crate::syntax::folds::{FoldRange, FoldStore};
use crate::syntax::highlighter::SyntaxHighlighter;
use crate::syntax::outline::DocumentSymbol;
use crate::theme::AppTheme;

pub enum SourceViewAction {
    CaretMoved { line: usize, column: usize, byte_offset: usize },
    JumpToDefinition,
    ToggleFold(usize),
    NextChangeBlock,
    PrevChangeBlock,
    ToggleRevision(ReviewRevision),
    PinOccurrence { symbol: String, ranges: Vec<std::ops::Range<usize>> },
}

pub struct SourceView;

impl SourceView {
    pub const GUTTER_WIDTH: f32 = 72.0;

    pub fn show(
        ui: &mut Ui,
        theme: &AppTheme,
        highlighter: &SyntaxHighlighter,
        syntax_ref: Option<&SyntaxReference>,
        buffer: &SourceBuffer,
        caret_line: &mut usize,
        caret_col: &mut usize,
        fold_ranges: &[FoldRange],
        fold_store: &mut FoldStore,
        line_changes: Option<&FileLineChanges>,
        _pin_store: &mut OccurrencePinStore,
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
                *caret_line += 1;
                let offset = buffer.byte_offset(*caret_line, *caret_col);
                action = Some(SourceViewAction::CaretMoved {
                    line: *caret_line,
                    column: *caret_col,
                    byte_offset: offset,
                });
            }
            if ui.ctx().input(|i| i.key_pressed(Key::ArrowUp)) && *caret_line > 1 {
                *caret_line -= 1;
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
            let available_height = ui.available_height() - 24.0; // Reserve for status bar
            let row_height = 20.0;

            ScrollArea::both()
                .auto_shrink([false, false])
                .max_height(available_height)
                .show(ui, |ui| {
                    let mut syn_highlighter = syntax_ref.map(|s| {
                        HighlightLines::new(s, highlighter.theme(theme.is_dark))
                    });

                    // Set of added and removed lines from diff
                    let changed_lines = line_changes.map(|c| {
                        c.changed_lines(buffer.revision.unwrap_or(ReviewRevision::After))
                    }).unwrap_or_default();

                    for line_num in 1..=buffer.line_count() {
                        // Skip line if hidden inside a collapsed fold
                        if fold_store.is_line_hidden(line_num, fold_ranges) {
                            continue;
                        }

                        let is_caret_line = line_num == *caret_line;
                        let line_text = buffer.line_text(line_num);

                        let (rect, line_response) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width().max(800.0), row_height),
                            egui::Sense::click_and_drag(),
                        );

                        if line_response.clicked() {
                            *caret_line = line_num;
                            let offset = buffer.byte_offset(line_num, 0);
                            action = Some(SourceViewAction::CaretMoved {
                                line: line_num,
                                column: 0,
                                byte_offset: offset,
                            });
                        }

                        // Background highlighting for active caret line
                        if is_caret_line {
                            ui.painter().rect_filled(rect, 0.0, theme.bg_active_line);
                        }

                        // Diff line highlighting
                        let is_diff_changed = changed_lines.iter().any(|r| r.contains(&line_num));
                        if is_diff_changed {
                            let diff_bg = if buffer.revision == Some(ReviewRevision::Before) {
                                theme.diff_del
                            } else {
                                theme.diff_add
                            };
                            ui.painter().rect_filled(rect, 0.0, diff_bg);
                        }

                        // Draw Gutter
                        let gutter_rect = Rect::from_min_size(
                            rect.min,
                            Vec2::new(Self::GUTTER_WIDTH, row_height),
                        );
                        ui.painter().rect_filled(gutter_rect, 0.0, theme.bg_gutter);

                        // Line number rule: print every 5th line or active caret line
                        let should_print_number = line_num % 5 == 0 || is_caret_line;
                        if should_print_number {
                            let num_color = if is_caret_line {
                                theme.accent
                            } else {
                                theme.text_muted
                            };
                            ui.painter().text(
                                gutter_rect.right_center() - Vec2::new(12.0, 0.0),
                                egui::Align2::RIGHT_CENTER,
                                line_num.to_string(),
                                TextStyle::Monospace.resolve(ui.style()),
                                num_color,
                            );
                        }

                        // Fold Chevron in Gutter
                        if let Some(_fold) = fold_ranges.iter().find(|f| f.start_line == line_num) {
                            let is_folded = fold_store.is_folded(line_num);
                            let chevron = if is_folded { "▶" } else { "▼" };
                            let chevron_pos = gutter_rect.left_center() + Vec2::new(6.0, 0.0);
                            ui.painter().text(
                                chevron_pos,
                                egui::Align2::LEFT_CENTER,
                                chevron,
                                TextStyle::Small.resolve(ui.style()),
                                theme.text_secondary,
                            );

                            let chevron_rect = Rect::from_center_size(chevron_pos, Vec2::new(16.0, 16.0));
                            if ui.rect_contains_pointer(chevron_rect)
                                && ui.input(|i| i.pointer.primary_clicked())
                            {
                                fold_store.toggle(line_num);
                                action = Some(SourceViewAction::ToggleFold(line_num));
                            }
                        }

                        // Gutter right border
                        ui.painter().line_segment(
                            [gutter_rect.right_top(), gutter_rect.right_bottom()],
                            Stroke::new(1.0, theme.border),
                        );

                        // Draw Code text
                        let code_pos = rect.left_center() + Vec2::new(Self::GUTTER_WIDTH + 8.0, 0.0);
                        if let Some(hl) = &mut syn_highlighter {
                            let spans = highlighter.highlight_line(hl, line_text);
                            let mut x_offset = 0.0;
                            for span in spans {
                                let span_text = span.text;
                                let font_id = TextStyle::Monospace.resolve(ui.style());
                                let galley = ui.painter().layout_no_wrap(
                                    span_text,
                                    font_id,
                                    span.color,
                                );
                                let span_pos = code_pos + Vec2::new(x_offset, 0.0);
                                ui.painter().galley(span_pos - Vec2::new(0.0, galley.size().y / 2.0), galley.clone(), span.color);
                                x_offset += galley.size().x;
                            }
                        } else {
                            ui.painter().text(
                                code_pos,
                                egui::Align2::LEFT_CENTER,
                                line_text,
                                TextStyle::Monospace.resolve(ui.style()),
                                theme.text_primary,
                            );
                        }
                    }
                });

            // Status Bar at bottom
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new(&buffer.path).monospace());
                ui.separator();
                ui.colored_label(theme.text_secondary, format!("Line {}, Col {}", *caret_line, *caret_col));
                ui.separator();
                ui.colored_label(theme.text_secondary, format!("{} lines", buffer.line_count()));
                ui.separator();
                ui.colored_label(theme.accent, "UTF-8");
            });
        });

        action
    }
}
