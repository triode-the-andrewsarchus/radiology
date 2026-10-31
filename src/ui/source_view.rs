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
mod breadcrumbs;
mod destination;
mod viewport;
use egui::{pos2, Color32, Rect, RichText, Ui};
use viewport::CanvasContent;
pub use viewport::SourceViewport;

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
    /// Pins reference context to the indexed occurrence at this source byte.
    FindReferences(usize),
    /// Navigates a pointer or menu destination independently of the current caret.
    NavigateToDefinition(crate::semantic::index::SemanticSymbolID),
    /// Requests expanding or collapsing the code fold starting at the given line number.
    ToggleFold(std::ops::Range<usize>),
    /// Toggles one detected test region without changing manual folds.
    ToggleTestRegion(usize),
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
    /// Pins the indexed symbol or scoped lexical identifier at original bytes.
    PinAtByte(usize),
    /// Selects a drawing-only occurrence group after an ordinary source click.
    SelectOccurrences(usize),
    /// Removes the pin group identified by this document-scoped symbol key.
    RemovePin(String),
    /// Removes all pins from the current document.
    ClearPins,
    /// Changes an exact semantic symbol/package decision.
    SetCriticality(crate::critical::overrides::MarkEdit),
    /// Requests opening the Go To Line modal dialog.
    OpenGoToLine,
}

/// Borrowed source presentation for one frame; geometry is owned by the view.
pub struct SourceViewContent<'a> {
    /// Active source presentation.
    pub mode: SourceMode,
    /// Current semantic/policy snapshot for exact source-menu targets.
    pub mark_context: Option<crate::critical::menu::MarkContext<'a>>,
    /// Recoverable load/save failure; current-session decisions remain active.
    pub mark_error: Option<&'a str>,
    /// Exact-source call projection; absent while preparing or in Normal mode.
    pub projection: Option<&'a crate::critical::SourceProjection>,
    /// Background reachable-operation counts for source function folds.
    pub annotations: Option<&'a crate::critical::annotations::SourceAnnotations>,
    /// Stable captured graph configuration for pointer-dwell summaries.
    pub hover_inputs: Option<std::sync::Arc<crate::critical::worker::GraphInputs>>,
    /// Active lens preparation/coverage, independent of semantic index status.
    pub critical_status: Option<&'static str>,
    /// Immutable source snapshot.
    pub buffer: &'a SourceBuffer,
    /// Precomputed syntax colors.
    pub highlighted_lines: &'a [Vec<HighlightedSpan>],
    /// One-based caret line.
    pub caret_line: &'a mut usize,
    /// Zero-based Unicode scalar column.
    pub caret_col: &'a mut usize,
    /// Detected fold ranges.
    pub fold_ranges: &'a [FoldRange],
    /// Recoverable fold-query failure; source and other semantic features remain usable.
    pub fold_error: Option<&'a str>,
    /// Recoverable parsed occurrence preparation failure.
    pub occurrence_error: Option<&'a str>,
    /// Optional reason the native colors fell back to basic highlighting.
    pub highlight_error: Option<&'a str>,
    /// Active collapsed ranges.
    pub fold_store: &'a mut FoldStore,
    /// Revision-specific diff overlays.
    pub line_changes: Option<&'a FileLineChanges>,
    /// Change metadata determines whether Before and After exist for this path.
    pub file_change: Option<&'a crate::pull_request::change::FileChange>,
    /// Pinned source ranges.
    pub pin_store: &'a OccurrencePinStore,
    /// Whether source wraps at the viewport edge.
    pub wrap_lines: bool,
    /// Current declaration breadcrumbs.
    pub enclosing_symbols: &'a [DocumentSymbol],
    /// One-shot navigation destination.
    pub scroll_to_line: &'a mut Option<usize>,
    /// Whether this revision has semantic coverage.
    pub has_semantic_index: bool,
    /// Whether source input is enabled outside modal sheets.
    pub keyboard_enabled: bool,
    /// Local-find state and highlights.
    pub find: &'a mut crate::ui::source_find::SourceFindPanel,
    /// Inline-test presentation from the exact source analysis.
    pub test_code: &'a crate::source::test_code::TestCodePresentation,
    /// Source font size, independent of interface labels.
    pub font_size: f32,
}

/// Renders the primary code editor and reading canvas.
pub struct SourceView;

impl SourceView {
    /// Width in points reserved for the gutter area (line numbers, fold chevrons, diff stripes).
    pub const GUTTER_WIDTH: f32 = 104.0;

    /// Renders source chrome and the measured, selectable source viewport.
    pub fn show(
        ui: &mut Ui,
        theme: &AppTheme,
        content: SourceViewContent<'_>,
        viewport: &mut SourceViewport,
    ) -> Option<SourceViewAction> {
        let SourceViewContent {
            mode,
            mark_context,
            mark_error,
            projection,
            annotations,
            hover_inputs,
            critical_status,
            buffer,
            highlighted_lines,
            caret_line,
            caret_col,
            fold_ranges,
            fold_error,
            occurrence_error,
            highlight_error,
            fold_store,
            line_changes,
            file_change,
            pin_store,
            wrap_lines,
            enclosing_symbols,
            scroll_to_line,
            has_semantic_index,
            keyboard_enabled,
            find,
            font_size,
            test_code,
        } = content;
        let mut action = None;

        ui.vertical(|ui| {
            // Header / Breadcrumbs Bar
            ui.add_space(4.0);
            action = breadcrumbs::show(
                ui,
                theme,
                breadcrumbs::Header {
                    buffer,
                    symbols: enclosing_symbols,
                    line: *caret_line,
                    changes: line_changes,
                    change: file_change,
                    can_collapse: fold_ranges
                        .iter()
                        .any(|fold| fold.kind == crate::syntax::folds::FoldKind::Callable),
                },
            );

            ui.separator();
            action = show_pin_groups(ui, theme, pin_store).or(action.take());

            let changed_lines = line_changes
                .map(|diff| diff.changed_lines(buffer.revision.unwrap_or(ReviewRevision::After)))
                .unwrap_or_default();
            if let Some(event) = viewport.show(
                ui,
                CanvasContent {
                    theme,
                    mode,
                    mark_context,
                    projection,
                    annotations,
                    hover_inputs,
                    buffer,
                    highlights: highlighted_lines,
                    function_shade: enclosing_symbols
                        .iter()
                        .rev()
                        .find(|symbol| symbol.is_callable())
                        .map(|symbol| &symbol.range),
                    caret_line,
                    caret_col,
                    folds: fold_ranges,
                    fold_store,
                    changes: &changed_lines,
                    pins: pin_store,
                    wrap: wrap_lines,
                    font_size,
                    target: scroll_to_line,
                    keyboard_enabled,
                    find,
                    test_code,
                },
            ) {
                action = Some(event);
            }

            // Status Bar at bottom (matching screenshot: "Rust · 815 lines" on left,
            // "ⓘ Semantic data · Line 564 · After · UTF-8 · Read Only" on right)
            ui.separator();
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                let language = crate::syntax::language::LanguageRegistry::for_path(&buffer.path);
                let lang_label = language.display_name;
                ui.colored_label(theme.text_secondary, lang_label);
                ui.colored_label(theme.text_muted, "·");
                ui.colored_label(
                    theme.text_secondary,
                    format!("{} lines", buffer.line_count()),
                );
                for (label, error) in [
                    ("Mark settings unavailable", mark_error),
                    ("Folding unavailable", fold_error),
                    ("Occurrence highlighting unavailable", occurrence_error),
                    ("Basic syntax colors", highlight_error),
                ]
                .into_iter()
                .filter_map(|(label, error)| error.map(|error| (label, error)))
                {
                    ui.colored_label(theme.text_secondary, label)
                        .on_hover_text(error);
                }
                let test_status = test_code.status();
                for status in critical_status.into_iter().chain(test_status.as_deref()) {
                    ui.colored_label(theme.text_muted, "·");
                    ui.colored_label(theme.text_muted, status);
                }

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
                    if line_btn.on_hover_text("Go to Line (Ctrl+L)").clicked() {
                        action = Some(SourceViewAction::OpenGoToLine);
                    }

                    ui.colored_label(theme.text_muted, "·");
                    if has_semantic_index {
                        ui.colored_label(Color32::from_rgb(34, 197, 94), "Semantic data");
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

/// Paints exact source matches using shaped glyph positions, including Unicode.
fn paint_match(
    painter: &egui::Painter,
    galley: &egui::Galley,
    origin: egui::Pos2,
    part: &reader_text::ProjectedFragment,
    source: &str,
    range: std::ops::Range<usize>,
    color: Color32,
) {
    for_each_match_rect(galley, origin, part, source, range, |rect| {
        painter.rect_filled(rect, 1.0, color);
    });
}

fn for_each_match_rect(
    galley: &egui::Galley,
    origin: egui::Pos2,
    part: &reader_text::ProjectedFragment,
    source: &str,
    range: std::ops::Range<usize>,
    mut draw: impl FnMut(Rect),
) {
    let first = part.char_index(source, range.start);
    let last = part.char_index(source, range.end);
    if first == last && (!range.is_empty() || !part.contains_byte(range.start)) {
        return;
    }
    let mut row_start = 0;
    for row in &galley.rows {
        let row_end = row_start + row.char_count_including_newline();
        let from = first.clamp(row_start, row_end) - row_start;
        let to = last.clamp(row_start, row_end) - row_start;
        if from < to || (range.is_empty() && first >= row_start && first <= row_end) {
            let min = origin + egui::vec2(row.x_offset(from), row.min_y());
            let max = origin + egui::vec2(row.x_offset(to), row.max_y());
            draw(Rect::from_min_max(min, pos2(max.x.max(min.x + 2.0), max.y)));
        }
        row_start = row_end;
    }
}

fn show_pin_groups(
    ui: &mut Ui,
    theme: &AppTheme,
    pins: &OccurrencePinStore,
) -> Option<SourceViewAction> {
    if pins.groups.is_empty() {
        return None;
    }
    let mut action = None;
    ui.horizontal(|ui| {
        ui.label(RichText::new("Pins").color(theme.text_secondary));
        if ui
            .small_button("Clear Pins")
            .on_hover_text("Remove pins from this document")
            .clicked()
        {
            action = Some(SourceViewAction::ClearPins);
        }
        egui::ScrollArea::horizontal()
            .id_source("source_pin_groups")
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for group in &pins.groups {
                        let mut spelling: String = group.label.chars().take(40).collect();
                        if spelling.len() < group.label.len() {
                            spelling.push('…');
                        }
                        let label = format!("{spelling} · {} ×", group.ranges.len());
                        let button = egui::Button::new(label).small().fill(
                            theme
                                .occurrence_color(group.color)
                                .gamma_multiply(if theme.is_dark { 0.22 } else { 0.15 }),
                        );
                        if ui
                            .add(button)
                            .on_hover_text(format!(
                                "{}: {} occurrences. Click to remove this pin.",
                                group.label,
                                group.ranges.len()
                            ))
                            .clicked()
                        {
                            action = Some(SourceViewAction::RemovePin(group.symbol_name.clone()));
                        }
                    }
                });
            });
    });
    ui.separator();
    action
}

fn revision_buttons(
    ui: &mut Ui,
    current: ReviewRevision,
    change: Option<&crate::pull_request::change::FileChange>,
) -> Option<ReviewRevision> {
    let mut action = None;
    for revision in [ReviewRevision::After, ReviewRevision::Before] {
        let available = change.is_none_or(|change| change.path_for_revision(revision).is_some());
        let response = ui.add_enabled(
            available,
            egui::SelectableLabel::new(current == revision, revision.title()),
        );
        if response
            .on_hover_text(change.map_or_else(
                || format!("View {}", revision.title()),
                |change| format!("{} · {}", revision.title(), change.status.title()),
            ))
            .on_disabled_hover_text(format!("This file is unavailable in {}", revision.title()))
            .clicked()
            && current != revision
        {
            action = Some(revision);
        }
    }
    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_group_controls_emit_remove_and_clear_actions_from_pointer_input() {
        let ctx = egui::Context::default();
        let theme = AppTheme::dark();
        let mut pins = OccurrencePinStore::new();
        pins.toggle_pin("alpha", std::iter::once(0..5).collect());
        let frame = |events: Vec<egui::Event>| {
            let mut action = None;
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(700.0, 200.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        action = show_pin_groups(ui, &theme, &pins);
                    });
                },
            );
            (output, action)
        };
        frame(Vec::new());
        let (output, _) = frame(Vec::new());
        for (label, expected) in [
            ("alpha · 1 ×", SourceViewAction::RemovePin("alpha".into())),
            ("Clear Pins", SourceViewAction::ClearPins),
        ] {
            let position = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::epaint::Shape::Text(text) if text.galley.text() == label => {
                        Some(text.pos + text.galley.size() / 2.0)
                    }
                    _ => None,
                })
                .expect(label);
            frame(vec![
                egui::Event::PointerMoved(position),
                egui::Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
            ]);
            let (_, action) = frame(vec![egui::Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }]);
            assert_eq!(action, Some(expected));
        }
    }

    #[test]
    fn source_does_not_handle_arrows_when_another_control_owns_input() {
        let ctx = egui::Context::default();
        let buffer = SourceBuffer::from_str("focus.rs", "one\ntwo\n");
        let (mut line, mut column) = (1, 0);
        let mut folds = FoldStore::new();
        let pins = OccurrencePinStore::new();
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
                    key: egui::Key::ArrowDown,
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
                        SourceViewContent {
                            mode: SourceMode::Normal,
                            mark_context: None,
                            mark_error: None,
                            projection: None,
                            annotations: None,
                            hover_inputs: None,
                            critical_status: None,
                            buffer: &buffer,
                            highlighted_lines: &highlighted,
                            caret_line: &mut line,
                            caret_col: &mut column,
                            fold_ranges: &[],
                            fold_error: None,
                            occurrence_error: None,
                            highlight_error: None,
                            fold_store: &mut folds,
                            line_changes: None,
                            file_change: None,
                            pin_store: &pins,
                            wrap_lines: false,
                            enclosing_symbols: &[],
                            scroll_to_line: &mut scroll,
                            has_semantic_index: false,
                            keyboard_enabled: false,
                            find: &mut Default::default(),
                            font_size: 14.0,
                            test_code: &Default::default(),
                        },
                        &mut Default::default(),
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
    #[test]
    fn revision_buttons_do_not_dispatch_unavailable_or_already_selected_revisions() {
        use crate::pull_request::change::{FileChange, FileChangeStatus};
        use crate::ui::test_support::{click, painted_frame, text_center};
        for (status, current, requested, expected) in [
            (
                FileChangeStatus::Added,
                ReviewRevision::After,
                ReviewRevision::Before,
                None,
            ),
            (
                FileChangeStatus::Deleted,
                ReviewRevision::Before,
                ReviewRevision::After,
                None,
            ),
            (
                FileChangeStatus::Modified,
                ReviewRevision::After,
                ReviewRevision::Before,
                Some(ReviewRevision::Before),
            ),
            (
                FileChangeStatus::Renamed { similarity: 100 },
                ReviewRevision::Before,
                ReviewRevision::After,
                Some(ReviewRevision::After),
            ),
            (
                FileChangeStatus::Added,
                ReviewRevision::After,
                ReviewRevision::After,
                None,
            ),
        ] {
            let change = FileChange {
                path: "new.rs".into(),
                old_path: Some("old.rs".into()),
                status,
                before_blob: "before".into(),
                after_blob: "after".into(),
                additions: 1,
                deletions: 1,
                is_binary: false,
            };
            let ctx = egui::Context::default();
            let size = egui::vec2(240.0, 80.0);
            let mut action = None;
            let output = painted_frame(&ctx, size, vec![], |ui| {
                action = revision_buttons(ui, current, Some(&change));
            });
            let target = text_center(&output, requested.title());
            painted_frame(&ctx, size, click(target), |ui| {
                action = revision_buttons(ui, current, Some(&change));
            });
            assert_eq!(action, expected, "{:?}", change.status);
        }
    }
}
