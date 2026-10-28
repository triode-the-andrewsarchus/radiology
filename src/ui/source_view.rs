//! Central source code viewer with syntax highlighting, diff markers, and gutter controls.
//!
//! Handles rendering line numbers, code folding chevrons, Syntect syntax-highlighted spans,
//! active caret line highlighting, pull request addition/deletion badges, and breadcrumb headers.

use crate::critical::lens::SourceMode;
use crate::pull_request::diff::FileLineChanges;
use crate::source::buffer::SourceBuffer;
use crate::source::location::ReviewRevision;
use crate::source::selection::OccurrencePinStore;
use crate::syntax::folds::{FoldRange, FoldStore};
use crate::syntax::highlighter::HighlightedSpan;
use crate::syntax::outline::DocumentSymbol;
use crate::theme::AppTheme;
use crate::ui::icons::Icon;
use egui::text::{LayoutJob, TextFormat};
use egui::{pos2, Color32, Key, Rect, RichText, ScrollArea, Stroke, TextStyle, Ui, Vec2};

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
    /// Requests collapsing or expanding all function bodies in the current file.
    CollapseFunctions,
    /// Requests jumping to the byte offset of a document symbol.
    JumpToSymbol(usize),
    /// Requests pinning all occurrences of a symbol for visual highlighting.
    PinOccurrence {
        /// Symbol text name.
        symbol: String,
        /// Byte ranges where the symbol appears in the current buffer.
        ranges: Vec<std::ops::Range<usize>>,
    },
    /// Requests opening the Go To Line modal dialog.
    OpenGoToLine,
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
    /// * `wrap_lines` - Whether long lines wrap at viewport edge instead of scrolling horizontally.
    /// * `enclosing_symbols` - Hierarchical list of symbols enclosing the current caret position.
    /// * `scroll_to_line` - Optional target line number to programmatically scroll and center into view.
    /// * `has_semantic_index` - Whether a SCIP semantic index is actively loaded for the repository.
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut Ui,
        theme: &AppTheme,
        mode: SourceMode,
        buffer: &SourceBuffer,
        highlighted_lines: &[Vec<HighlightedSpan>],
        caret_line: &mut usize,
        caret_col: &mut usize,
        fold_ranges: &[FoldRange],
        fold_store: &mut FoldStore,
        line_changes: Option<&FileLineChanges>,
        pin_store: &mut OccurrencePinStore,
        wrap_lines: bool,
        enclosing_symbols: &[DocumentSymbol],
        scroll_to_line: &mut Option<usize>,
        has_semantic_index: bool,
        keyboard_enabled: bool,
    ) -> Option<SourceViewAction> {
        let mut action = None;

        ui.vertical(|ui| {
            // Header / Breadcrumbs Bar
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.spacing_mut().item_spacing.x = 6.0;

                // Path breadcrumbs
                let parts: Vec<&str> = buffer.path.split('/').collect();
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        Icon::ChevronRight.ui(ui, 10.0, theme.text_muted);
                    }
                    if i == parts.len() - 1 {
                        ui.label(RichText::new(*part).strong());
                    } else {
                        ui.colored_label(theme.text_secondary, *part);
                    }
                }

                // Enclosing symbol breadcrumbs (interactive links)
                for sym in enclosing_symbols {
                    Icon::ChevronRight.ui(ui, 10.0, theme.text_muted);
                    let btn = ui.add(
                        egui::Button::new(RichText::new(&sym.name).strong().color(theme.accent))
                            .frame(false),
                    );
                    if btn.on_hover_text(format!("Jump to {}", sym.name)).clicked() {
                        action = Some(SourceViewAction::JumpToSymbol(sym.selection_range.start));
                    }
                }

                // PR Diff block jump controls if in review
                if let Some(diff) = line_changes {
                    if !diff.hunks.is_empty() {
                        ui.separator();
                        if Icon::ArrowUp
                            .button(ui, 11.0, theme.text_primary)
                            .on_hover_text("Previous change block (⌥↑)")
                            .clicked()
                        {
                            action = Some(SourceViewAction::PrevChangeBlock);
                        }
                        if Icon::ArrowDown
                            .button(ui, 11.0, theme.text_primary)
                            .on_hover_text("Next change block (⌥↓)")
                            .clicked()
                        {
                            action = Some(SourceViewAction::NextChangeBlock);
                        }
                    }
                }

                // Collapse functions button (matching macOS Swift SourceBreadcrumbs)
                ui.separator();
                if Icon::Wrap
                    .button(ui, 11.0, theme.text_secondary)
                    .on_hover_text("Collapse all function bodies")
                    .clicked()
                {
                    action = Some(SourceViewAction::CollapseFunctions);
                }

                // Right side: Modified status + Revision [ Before | After ] segmented pill
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(rev) = buffer.revision {
                        let is_after = rev == ReviewRevision::After;
                        let is_before = rev == ReviewRevision::Before;
                        ui.horizontal(|ui| {
                            if ui.selectable_label(is_after, "After").clicked() && !is_after {
                                action =
                                    Some(SourceViewAction::ToggleRevision(ReviewRevision::After));
                            }
                            if ui.selectable_label(is_before, "Before").clicked() && !is_before {
                                action =
                                    Some(SourceViewAction::ToggleRevision(ReviewRevision::Before));
                            }
                            ui.colored_label(theme.text_secondary, "Revision");
                            if line_changes.is_some() {
                                ui.colored_label(Color32::from_rgb(234, 179, 8), "Modified");
                            }
                        });
                    }
                });
            });

            ui.separator();

            // Central Code Area with Gutter
            let available_height = (ui.available_height() - 28.0).max(100.0);
            let row_height = 22.0; // matching macOS Swift Theme.sourceLineHeight: 22
            let font_id = TextStyle::Monospace.resolve(ui.style());
            let char_width = ui.fonts(|f| f.glyph_width(&font_id, ' ')).max(7.0);

            // Build visible line numbers skipping collapsed folds
            let visible_lines: Vec<usize> = (1..=buffer.line_count())
                .filter(|&l| !fold_store.is_line_hidden(l, fold_ranges))
                .collect();
            let total_rows = visible_lines.len();
            if keyboard_enabled && ui.input(|input| input.modifiers.is_none()) {
                let step = ui.input_mut(|input| {
                    if input.consume_key(egui::Modifiers::NONE, Key::ArrowDown) {
                        1_isize
                    } else if input.consume_key(egui::Modifiers::NONE, Key::ArrowUp) {
                        -1
                    } else {
                        0
                    }
                });
                if step != 0 {
                    let index = visible_lines.partition_point(|&line| line < *caret_line);
                    let target = index
                        .saturating_add_signed(step)
                        .min(total_rows.saturating_sub(1));
                    if let Some(&line) = visible_lines.get(target) {
                        *caret_line = line;
                        let byte = buffer.byte_offset(line, *caret_col);
                        *caret_col = buffer.column_at_byte(byte);
                        *scroll_to_line = Some(line);
                        action = Some(SourceViewAction::CaretMoved {
                            line,
                            column: *caret_col,
                            byte_offset: byte,
                        });
                    }
                }
            }

            let changed_lines = line_changes
                .map(|c| c.changed_lines(buffer.revision.unwrap_or(ReviewRevision::After)))
                .unwrap_or_default();

            let max_chars = buffer.max_line_len();
            let full_code_width =
                (max_chars as f32 * char_width * 1.1 + Self::GUTTER_WIDTH + 120.0)
                    .max(ui.available_width());
            let row_width = if wrap_lines {
                ui.available_width()
            } else {
                full_code_width
            };

            // Fixed editor viewport rectangle in window coordinates (captured before ScrollArea child UI)
            let editor_viewport = ui.available_rect_before_wrap();

            let mut scroll_area = ScrollArea::both()
                .id_source(&buffer.path)
                .auto_shrink([false, false])
                .max_height(available_height);

            if let Some(target) = *scroll_to_line {
                let target_idx = visible_lines
                    .iter()
                    .position(|&l| l == target)
                    .unwrap_or_else(|| target.saturating_sub(1));
                let line_y = target_idx as f32 * row_height;
                let scroll_y = if target == 1 {
                    0.0
                } else {
                    (line_y - available_height / 2.0 + row_height / 2.0).max(0.0)
                };
                scroll_area = scroll_area.vertical_scroll_offset(scroll_y);
                if target == 1 {
                    scroll_area = scroll_area.horizontal_scroll_offset(0.0);
                }
                *scroll_to_line = None;
            }

            scroll_area.show_rows(ui, row_height, total_rows, |ui, row_range| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;

                // Pinned gutter column across the full editor viewport height
                let gutter_painter = ui.painter().with_clip_rect(editor_viewport);
                let gutter_column = Rect::from_min_size(
                    pos2(editor_viewport.min.x, editor_viewport.min.y),
                    Vec2::new(Self::GUTTER_WIDTH, editor_viewport.height()),
                );
                gutter_painter.rect_filled(gutter_column, 0.0, theme.bg_gutter);
                gutter_painter.line_segment(
                    [gutter_column.right_top(), gutter_column.right_bottom()],
                    Stroke::new(1.0_f32, theme.border),
                );

                // Code clip rect: area to the right of the pinned gutter up to the right edge of the viewport.
                // Include 4px vertical clearance above editor_viewport.min.y to prevent clipping font ascenders on Line 1.
                let code_clip_min_x = editor_viewport.min.x + Self::GUTTER_WIDTH;
                let code_clip_rect = Rect::from_min_max(
                    pos2(code_clip_min_x, editor_viewport.min.y - 4.0),
                    editor_viewport.max,
                );
                let code_painter = ui.painter().with_clip_rect(code_clip_rect);

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

                    // Pinned Gutter at editor viewport left edge (pinned: gutter does not scroll horizontally)
                    let gutter_rect = Rect::from_min_size(
                        pos2(editor_viewport.min.x, rect.min.y),
                        Vec2::new(Self::GUTTER_WIDTH, row_height),
                    );

                    // Fold Chevron / Count Badge in Gutter
                    let mut chevron_clicked = false;
                    if let Some(fold) = fold_ranges.iter().find(|f| f.start_line == line_num) {
                        let is_folded = fold_store.is_folded(line_num);
                        if is_folded {
                            // Draw fold count badge capsule (matching lines 571 and 586 in screenshot: ▶ 8, ▶ 5)
                            let hidden_count = fold.end_line.saturating_sub(fold.start_line);
                            let badge_rect = Rect::from_min_size(
                                gutter_rect.left_center() + Vec2::new(4.0, -8.0),
                                Vec2::new(26.0, 16.0),
                            );
                            let badge_bg = if theme.is_dark {
                                Color32::from_rgb(50, 50, 54)
                            } else {
                                Color32::from_rgb(225, 225, 230)
                            };
                            gutter_painter.rect_filled(badge_rect, 8.0, badge_bg);
                            let chevron_rect = Rect::from_min_size(
                                badge_rect.min + Vec2::new(3.0, 3.5),
                                Vec2::new(9.0, 9.0),
                            );
                            Icon::ChevronRight.paint(
                                &gutter_painter,
                                chevron_rect,
                                theme.text_secondary,
                            );
                            gutter_painter.text(
                                badge_rect.left_center() + Vec2::new(13.0, 0.0),
                                egui::Align2::LEFT_CENTER,
                                hidden_count.to_string(),
                                TextStyle::Small.resolve(ui.style()),
                                theme.text_secondary,
                            );

                            if ui.rect_contains_pointer(badge_rect)
                                && ui.input(|i| i.pointer.primary_clicked())
                            {
                                fold_store.toggle(line_num);
                                action = Some(SourceViewAction::ToggleFold(line_num));
                                chevron_clicked = true;
                            }
                        } else {
                            let chevron_pos = gutter_rect.left_center() + Vec2::new(8.0, 0.0);
                            let chevron_rect =
                                Rect::from_center_size(chevron_pos, Vec2::new(12.0, 12.0));
                            Icon::ChevronDown.paint(
                                &gutter_painter,
                                chevron_rect,
                                theme.text_muted,
                            );

                            if ui.rect_contains_pointer(chevron_rect)
                                && ui.input(|i| i.pointer.primary_clicked())
                            {
                                fold_store.toggle(line_num);
                                action = Some(SourceViewAction::ToggleFold(line_num));
                                chevron_clicked = true;
                            }
                        }
                    }

                    // Background highlighting for active caret line (clipped to code area)
                    if is_caret_line {
                        code_painter.rect_filled(rect, 0.0, theme.bg_active_line);
                        // 2.5px vertical accent bar on the left edge of the code line (matching Swift reference)
                        let bar_x = rect.min.x + Self::GUTTER_WIDTH + 2.0;
                        let bar_rect = Rect::from_min_size(
                            pos2(bar_x, rect.min.y + 2.0),
                            Vec2::new(2.5, row_height - 4.0),
                        );
                        code_painter.rect_filled(bar_rect, 1.0, theme.accent);
                    }

                    // Diff line highlighting (clipped to code area)
                    let is_diff_changed = changed_lines.iter().any(|r| r.contains(&line_num));
                    if is_diff_changed {
                        let diff_bg = if buffer.revision == Some(ReviewRevision::Before) {
                            theme.diff_del
                        } else {
                            theme.diff_add
                        };
                        code_painter.rect_filled(rect, 0.0, diff_bg);

                        // 3px vertical color bar on the right edge of the gutter (Swift LineNumberGutter parity)
                        let bar_color = if buffer.revision == Some(ReviewRevision::Before) {
                            Color32::from_rgb(255, 59, 48)
                        } else {
                            Color32::from_rgb(52, 199, 89)
                        };
                        let bar_rect = Rect::from_min_size(
                            gutter_rect.right_top() - Vec2::new(3.0, 0.0),
                            Vec2::new(3.0, row_height),
                        );
                        gutter_painter.rect_filled(bar_rect, 0.0, bar_color);
                    }

                    // Line number in pinned gutter
                    let num_color = if is_caret_line {
                        theme.accent
                    } else {
                        theme.text_muted
                    };
                    gutter_painter.text(
                        gutter_rect.right_center() - Vec2::new(10.0, 0.0),
                        egui::Align2::RIGHT_CENTER,
                        line_num.to_string(),
                        TextStyle::Monospace.resolve(ui.style()),
                        num_color,
                    );

                    // Caret click: calculate actual character column from mouse position
                    let code_start_x = rect.min.x + Self::GUTTER_WIDTH + 12.0;
                    // Assemble unified line LayoutJob so all tokens share the exact same typographic baseline
                    let font_id = TextStyle::Monospace.resolve(ui.style());
                    let mut job = LayoutJob {
                        break_on_newline: false,
                        ..Default::default()
                    };
                    if wrap_lines {
                        job.wrap.max_width =
                            (ui.available_width() - Self::GUTTER_WIDTH - 24.0).max(100.0);
                    }

                    if let Some(spans) = highlighted_lines.get(line_num.saturating_sub(1)) {
                        if spans.is_empty() {
                            job.append(
                                line_text,
                                0.0,
                                TextFormat::simple(font_id.clone(), theme.text_primary),
                            );
                        } else {
                            for span in spans {
                                let mut color = span.color;
                                if mode == SourceMode::Critical {
                                    // In Critical Lens mode, recede non-critical tokens (keywords, punctuation, basic runtime plumbing)
                                    let is_critical = span.color == theme.accent
                                        || span.color == egui::Color32::from_rgb(224, 108, 117);
                                    if !is_critical {
                                        let factor = if theme.is_dark { 0.38 } else { 0.30 };
                                        color = color.linear_multiply(factor);
                                    }
                                }
                                job.append(
                                    span.text.trim_end_matches(['\r', '\n']),
                                    0.0,
                                    TextFormat::simple(font_id.clone(), color),
                                );
                            }
                        }
                    } else {
                        job.append(
                            line_text,
                            0.0,
                            TextFormat::simple(font_id.clone(), theme.text_primary),
                        );
                    }

                    let galley = code_painter.layout_job(job);
                    let text_y = rect.min.y + ((row_height - galley.size().y) / 2.0).max(0.0);
                    let text_origin = pos2(code_start_x, text_y);
                    code_painter.galley(text_origin, galley.clone(), theme.text_primary);

                    let pointer_column =
                        line_response.interact_pointer_pos().map_or(0, |position| {
                            galley
                                .cursor_from_pos(position - text_origin)
                                .ccursor
                                .index
                                .min(line_text.chars().count())
                        });

                    if (line_response.clicked() || line_response.secondary_clicked())
                        && !chevron_clicked
                    {
                        *caret_line = line_num;
                        let col = pointer_column;
                        *caret_col = col;
                        let offset = buffer.byte_offset(line_num, col);
                        action = Some(SourceViewAction::CaretMoved {
                            line: line_num,
                            column: col,
                            byte_offset: offset,
                        });
                    }

                    if line_response.double_clicked() && !chevron_clicked {
                        let col = pointer_column;
                        let chars: Vec<char> = line_text.chars().collect();
                        if !chars.is_empty() {
                            let mut start_idx = col.min(chars.len() - 1);
                            while start_idx > 0
                                && (chars[start_idx - 1].is_alphanumeric()
                                    || chars[start_idx - 1] == '_')
                            {
                                start_idx -= 1;
                            }
                            let mut end_idx = col.min(chars.len());
                            while end_idx < chars.len()
                                && (chars[end_idx].is_alphanumeric() || chars[end_idx] == '_')
                            {
                                end_idx += 1;
                            }
                            if end_idx > start_idx {
                                let word: String = chars[start_idx..end_idx].iter().collect();
                                if !word.trim().is_empty() {
                                    let mut ranges = Vec::new();
                                    let mut search_from = 0;
                                    while let Some(found) = buffer.text[search_from..].find(&word) {
                                        let start = search_from + found;
                                        let end = start + word.len();
                                        ranges.push(start..end);
                                        search_from = end;
                                    }
                                    action = Some(SourceViewAction::PinOccurrence {
                                        symbol: word,
                                        ranges,
                                    });
                                }
                            }
                        }
                    }

                    line_response.context_menu(|ui| {
                        if ui.button("Go to Definition (F12 / Return)").clicked() {
                            action = Some(SourceViewAction::JumpToDefinition);
                            ui.close_menu();
                        }
                        if ui.button("Copy Line").clicked() {
                            ui.output_mut(|o| o.copied_text = line_text.to_string());
                            ui.close_menu();
                        }
                    });

                    // Draw occurrence pins if any exist on this line
                    if !pin_store.is_empty() {
                        let line_start_byte = buffer.byte_offset(line_num, 0);
                        let line_end_byte = buffer.byte_offset(line_num + 1, 0);
                        for group in &pin_store.groups {
                            for range in &group.ranges {
                                if range.start < line_end_byte && range.end > line_start_byte {
                                    let col_start = range.start.saturating_sub(line_start_byte);
                                    let pin_x = code_start_x + col_start as f32 * char_width;
                                    let pin_w = (range.end - range.start) as f32 * char_width;
                                    let pin_rect = Rect::from_min_size(
                                        egui::pos2(pin_x, rect.min.y + 2.0),
                                        egui::vec2(pin_w.max(char_width), row_height - 4.0),
                                    );
                                    let color = theme.occurrence_tint(group.color);
                                    code_painter.rect_stroke(
                                        pin_rect,
                                        2.0,
                                        Stroke::new(1.0_f32, color),
                                    );
                                }
                            }
                        }
                    }
                }
            });

            // Status Bar at bottom (matching screenshot: "Rust · 815 lines" on left,
            // "ⓘ Index Ready · Line 564 · After · UTF-8 · Read Only" on right)
            ui.separator();
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                let ext = std::path::Path::new(&buffer.path)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Text");
                let lang_label = match ext {
                    "rs" => "Rust",
                    "py" => "Python",
                    "js" => "JavaScript",
                    "ts" | "tsx" => "TypeScript",
                    "go" => "Go",
                    "c" => "C",
                    "cpp" => "C++",
                    "swift" => "Swift",
                    _ => ext,
                };
                ui.colored_label(theme.text_secondary, lang_label);
                ui.colored_label(theme.text_muted, "·");
                ui.colored_label(
                    theme.text_secondary,
                    format!("{} lines", buffer.line_count()),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    ui.colored_label(theme.text_muted, "Read Only");
                    ui.colored_label(theme.text_muted, "·");
                    ui.colored_label(theme.text_muted, "UTF-8");

                    if let Some(rev) = buffer.revision {
                        ui.colored_label(theme.text_muted, "·");
                        ui.colored_label(theme.accent, rev.title());
                    }

                    ui.colored_label(theme.text_muted, "·");
                    let line_btn = ui.add(
                        egui::Button::new(
                            RichText::new(format!("Line {}", *caret_line))
                                .size(11.5)
                                .color(theme.text_secondary),
                        )
                        .frame(false),
                    );
                    if line_btn.on_hover_text("Go to Line (⌘L)").clicked() {
                        action = Some(SourceViewAction::OpenGoToLine);
                    }

                    ui.colored_label(theme.text_muted, "·");
                    if has_semantic_index {
                        ui.colored_label(Color32::from_rgb(34, 197, 94), "Index Ready");
                    } else {
                        ui.colored_label(
                            theme.text_muted,
                            if buffer.revision == Some(ReviewRevision::Before) {
                                "Not Indexed (Before)"
                            } else {
                                "No Index"
                            },
                        );
                    }
                });
            });
        });

        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_does_not_handle_arrows_when_another_control_owns_input() {
        let ctx = egui::Context::default();
        let buffer = SourceBuffer::from_str("focus.rs", "one\ntwo\n");
        let (mut line, mut column) = (1, 0);
        let mut folds = FoldStore::new();
        let mut pins = OccurrencePinStore::new();
        let mut scroll = None;
        let highlighter = crate::syntax::highlighter::SyntaxHighlighter::new();
        let highlighted = highlighter.highlight_document(
            highlighter.find_syntax_for_path("focus.rs"),
            &buffer.text,
            true,
        );
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 600.0),
                )),
                events: vec![egui::Event::Key {
                    key: Key::ArrowDown,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }],
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    SourceView::show(
                        ui,
                        &AppTheme::dark(),
                        SourceMode::Normal,
                        &buffer,
                        &highlighted,
                        &mut line,
                        &mut column,
                        &[],
                        &mut folds,
                        None,
                        &mut pins,
                        false,
                        &[],
                        &mut scroll,
                        false,
                        false,
                    );
                });
            },
        );
        assert_eq!(line, 1);
        let painted_text = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(painted_text.contains(&"one"));
        assert!(painted_text.iter().all(|text| !text.contains(['\r', '\n'])));
    }
}
