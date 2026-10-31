//! Independently resizable declaration, definition/outline, and bookmark regions.

use super::bookmark_list::{BookmarkAction, BookmarkList};
use super::outline_list::{OutlineAction, OutlineList};
use crate::bookmarks::store::BookmarkStore;
use crate::semantic::reference_results::ReferenceResults;
use crate::source::SourceBuffer;
use crate::syntax::outline::DocumentSymbol;
use crate::theme::AppTheme;
use egui::{RichText, ScrollArea, Ui};
mod declaration_summary;
pub use declaration_summary::DeclarationSummary;

/// Navigation and management events from the inspector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InspectorAction {
    /// Opens the exact semantic definition under the source caret.
    GoToDefinition,
    /// Selects, discloses, or explicitly commits a reference row.
    Reference(crate::ui::reference_list::ReferenceAction),
    /// Releases an explicit reference target and resumes enclosing-declaration context.
    FollowCaret,
    /// Shows or hides the current definition details without navigating.
    ExpandDefinition(bool),
    /// Resolves a reference in its document's declared encoding and exact bytes.
    JumpToIndexedLocation {
        /// Repository-relative destination path.
        path: String,
        /// Original SCIP coordinates; not byte offsets.
        range: crate::semantic::ScipRange,
    },
    /// Navigates to an exact symbol byte in the current source.
    JumpToSymbol(usize),
    /// Opens, removes, reorders, or toggles an identity-based bookmark.
    Bookmark(BookmarkAction),
    /// Previews or commits an exact source-bound outline declaration.
    Outline(OutlineAction),
}

/// Independent enclosing-declaration and under-caret facts for one frame.
pub struct InspectorContent<'a> {
    /// Symbol under the caret, independently of the enclosing declaration.
    pub under_caret: Option<&'a crate::semantic::context::UnderCaretContext>,
    /// Symbol metadata is being prepared outside the UI thread.
    pub under_caret_loading: bool,
    /// Recoverable symbol-detail preparation failure.
    pub under_caret_error: Option<&'a str>,
    /// Bounded source excerpt for the canonical definition destination.
    pub definition_preview: &'a crate::source::preview::PreviewPhase,
    /// Whether the under-caret details are disclosed.
    pub definition_expanded: bool,
    /// Innermost parsed declaration enclosing the caret.
    pub declaration: Option<&'a DocumentSymbol>,
    /// Exact active source bytes for navigation and line counts.
    pub buffer: Option<&'a SourceBuffer>,
    /// Incoming references to the enclosing declaration.
    pub reference_results: &'a ReferenceResults,
    /// Selected reference row, independent of source caret.
    pub selected_reference: Option<usize>,
    /// Per-row source disclosures with independently cancellable preparation.
    pub reference_disclosures:
        &'a std::collections::BTreeMap<usize, crate::semantic::disclosure::ReferenceDisclosure>,
    /// Effective automatic or explicitly pinned reference name.
    pub reference_target: Option<&'a str>,
    /// SCIP kind of [`Self::reference_target`]; `0` when metadata is absent.
    ///
    /// Resolve document-local symbols using their captured document, independently
    /// of the currently displayed source. Unknown values remain visible by number.
    pub reference_kind: i32,
    /// Whether Follow Caret should release a sticky reference target.
    pub explicit_references: bool,
    /// Reference rows are being prepared off the UI thread.
    pub references_loading: bool,
    /// Recoverable reference-worker failure.
    pub reference_error: Option<&'a str>,
    /// Whether this revision has a semantic reference index.
    pub references_available: bool,
    /// Original structural symbols in source order.
    pub outline: &'a [DocumentSymbol],
    /// Source and structure are being prepared; previously accepted content may remain.
    pub outline_loading: bool,
    /// Native parser failure for the displayed source, independent of folding errors.
    pub outline_error: Option<&'a str>,
    /// Current test-region projection for outline visibility.
    pub test_code: Option<&'a crate::source::test_code::TestCodePresentation>,
    /// Bookmarks in the active checkout or PR scope.
    pub bookmarks: &'a BookmarkStore,
    /// Visible checkout or PR scope title.
    pub scope: &'a str,
    /// Whether checkout/review transitions permit bookmark interaction.
    pub bookmarks_enabled: bool,
    /// Whether the current source is ready for bookmarking.
    pub can_toggle_bookmark: bool,
    /// Whether a bookmark resolves to the current source line.
    pub current_bookmarked: bool,
    /// Whether source/review preparation permits outline navigation.
    pub outline_enabled: bool,
}

/// Renders the three simultaneously usable context regions.
pub struct ContextInspector;

impl ContextInspector {
    pub(crate) fn definition_focus_id() -> egui::Id {
        egui::Id::new("definition_context_focus")
    }

    pub(crate) fn definition_key(
        ctx: &egui::Context,
        key: egui::Key,
        modifiers: egui::Modifiers,
    ) -> Option<InspectorAction> {
        if !super::commands::focused_key(
            ctx,
            Self::definition_focus_id(),
            key,
            modifiers,
            &[
                egui::Key::ArrowLeft,
                egui::Key::ArrowRight,
                egui::Key::Enter,
            ],
        ) {
            return None;
        }
        match key {
            egui::Key::ArrowRight => Some(InspectorAction::ExpandDefinition(true)),
            egui::Key::ArrowLeft => Some(InspectorAction::ExpandDefinition(false)),
            egui::Key::Enter => Some(InspectorAction::GoToDefinition),
            _ => None,
        }
    }

    /// Renders independently scrolling regions and returns a requested action.
    pub fn show(
        ui: &mut Ui,
        theme: &AppTheme,
        content: InspectorContent<'_>,
        references: &mut crate::ui::reference_list::ReferenceViewport,
        bookmarks: &mut BookmarkList,
        outline: &mut OutlineList,
        declaration: &mut DeclarationSummary,
    ) -> Option<InspectorAction> {
        let mut action = None;
        let height = ui.available_height();
        egui::TopBottomPanel::bottom("bookmarks_region")
            .resizable(true)
            .default_height(height * 0.25)
            .min_height(100.0)
            .max_height((height * 0.5).max(100.0))
            .show_inside(ui, |ui| {
                let next = region(ui, |ui| {
                    ui.add_enabled_ui(content.bookmarks_enabled, |ui| {
                        bookmarks.show(
                            ui,
                            theme,
                            content.bookmarks,
                            content.scope,
                            content
                                .can_toggle_bookmark
                                .then_some(content.current_bookmarked),
                        )
                    })
                    .inner
                    .map(InspectorAction::Bookmark)
                });
                if let Some(next) = next {
                    action = Some(next);
                }
            });
        egui::TopBottomPanel::top("declaration_region")
            .resizable(true)
            .default_height(height * 0.38)
            .min_height(100.0)
            .max_height((ui.available_height() - 120.0).max(100.0))
            .show_inside(ui, |ui| {
                if let Some(next) = region(ui, |ui| {
                    Self::render_declaration_context(ui, theme, &content, references, declaration)
                }) {
                    action = Some(next);
                }
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show_inside(ui, |ui| {
                if let Some(next) = region(ui, |ui| {
                    Self::render_under_caret_context(ui, theme, &content, outline)
                }) {
                    action = Some(next);
                }
            });
        action
    }

    fn render_under_caret_context(
        ui: &mut Ui,
        theme: &AppTheme,
        content: &InspectorContent<'_>,
        outline: &mut OutlineList,
    ) -> Option<InspectorAction> {
        let mut action = None;
        let has_details = content.under_caret.is_some()
            || content.under_caret_loading
            || content.under_caret_error.is_some();
        if has_details {
            ui.interact(
                ui.available_rect_before_wrap(),
                Self::definition_focus_id(),
                egui::Sense::focusable_noninteractive(),
            );
        }
        ui.memory_mut(|memory| {
            memory.set_focus_lock_filter(
                Self::definition_focus_id(),
                egui::EventFilter {
                    horizontal_arrows: true,
                    ..Default::default()
                },
            )
        });
        let heading = if has_details { "Definition" } else { "Outline" };
        ui.label(RichText::new(heading).strong());
        if !has_details {
            if content.outline_loading {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Reading structure…");
                });
            }
            if let Some(error) = content.outline_error {
                super::text_fit::explanation(ui, error, theme.text_secondary);
                if content.outline.is_empty() {
                    return None;
                }
            }
            if content.buffer.is_none() && content.outline_loading {
                return None;
            }
            if content.buffer.is_none() {
                super::text_fit::explanation(
                    ui,
                    "Open a source file to see its declarations.",
                    theme.text_secondary,
                );
                return None;
            }
            return ui
                .add_enabled_ui(content.outline_enabled, |ui| outline.show(ui, theme))
                .inner
                .map(InspectorAction::Outline);
        }
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .id_source("definition_or_outline")
            .show(ui, |ui| {
                if let Some(context) = content.under_caret {
                    action = super::under_caret::show(
                        ui,
                        theme,
                        context,
                        content.definition_preview,
                        content.definition_expanded,
                    );
                } else if content.under_caret_loading {
                    ui.colored_label(theme.text_muted, "Loading symbol details…");
                } else if let Some(error) = content.under_caret_error {
                    ui.colored_label(theme.text_muted, "Symbol details unavailable")
                        .on_hover_text(error);
                }
            });
        action
    }

    fn render_declaration_context(
        ui: &mut Ui,
        theme: &AppTheme,
        content: &InspectorContent<'_>,
        references: &mut crate::ui::reference_list::ReferenceViewport,
        summary: &mut DeclarationSummary,
    ) -> Option<InspectorAction> {
        ui.heading("Declaration");
        let mut action = summary.show(
            ui,
            theme,
            content.declaration,
            content.buffer,
            content.explicit_references.then_some((
                content.reference_target.unwrap_or("Pinned symbol"),
                content.reference_kind,
            )),
        );
        if let Some(next) = Self::render_reference_context(ui, theme, content, references) {
            action = Some(next);
        }
        action
    }

    fn render_reference_context(
        ui: &mut Ui,
        theme: &AppTheme,
        content: &InspectorContent<'_>,
        references: &mut crate::ui::reference_list::ReferenceViewport,
    ) -> Option<InspectorAction> {
        let mut action = None;
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "References · {}",
                    content.reference_results.reference_count
                ))
                .strong(),
            );
            if content.explicit_references && ui.small_button("Follow Caret").clicked() {
                action = Some(InspectorAction::FollowCaret);
            }
        });
        if content.references_available && content.reference_target.is_some() {
            if let Some(next) = references.show_coverage(
                ui,
                theme,
                content.reference_results,
                content.references_loading,
            ) {
                action = Some(InspectorAction::Reference(next));
            }
        }
        let status = if let Some(error) = content.reference_error {
            Some(error)
        } else if content.references_loading && content.reference_results.hits.is_empty() {
            Some("Loading references…")
        } else if !content.references_available {
            Some("Semantic data unavailable for this revision.")
        } else if content.reference_target.is_none() {
            Some("No exact indexed declaration here.")
        } else {
            None
        };
        if let Some(status) = status {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .id_source("reference_status")
                .show(ui, |ui| {
                    ui.colored_label(theme.text_muted, status);
                });
        } else if let Some(next) = references.show(
            ui,
            theme,
            content.reference_results,
            content.selected_reference,
            content.reference_disclosures,
        ) {
            action = Some(InspectorAction::Reference(next));
        }
        action
    }
}

// Panel layout owns its extent; clipped child content cannot enlarge a sibling region.
fn region(
    ui: &mut Ui,
    render: impl FnOnce(&mut Ui) -> Option<InspectorAction>,
) -> Option<InspectorAction> {
    let rect = ui.available_rect_before_wrap();
    let mut child = ui.child_ui(rect, egui::Layout::top_down(egui::Align::Min));
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    let action = render(&mut child);
    ui.allocate_space(rect.size().max(egui::Vec2::ZERO));
    action
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_explanation_layout(galley: &egui::Galley, description: &str) {
        assert!(galley.job.text.len() <= 4100);
        assert!(galley.rows.len() <= 3);
        if description.is_ascii() {
            for row in galley.rows.iter().take(galley.rows.len().saturating_sub(1)) {
                for word in row.text().split_whitespace() {
                    assert!(
                        description.split_whitespace().any(|whole| whole == word),
                        "prose word was split: {word}"
                    );
                }
            }
        }
        if description.contains("Ctrl+Alt+H") {
            assert!(
                galley.job.text.contains("Ctrl+Alt+H"),
                "hide-tests recovery shortcut was elided"
            );
        }
    }

    #[test]
    fn outline_loading_and_failure_messages_stay_bounded_in_each_theme_and_scale() {
        let failure = format!(
            "Structure unavailable: {}",
            "失敗😀 long diagnostic ".repeat(1000)
        );
        let results = ReferenceResults::default();
        let bookmarks = BookmarkStore::default();
        let preview = crate::source::preview::PreviewPhase::None;
        let disclosures = Default::default();
        for dark in [true, false] {
            for scale in [1.0, 2.0] {
                for width in [200.0, 300.0] {
                    for (loading, error, expected) in [
                        (true, None, "Reading structure…"),
                        (false, None, "Open a source file"),
                        (false, Some(failure.as_str()), "Structure unavailable:"),
                        (false, Some("No structural parser is available for this file. Source text remains available."), "No structural parser"),
                        (false, Some("Every declaration in this file is hidden test code. Turn off Hide Tests (Ctrl+Alt+H) to list them."), "Every declaration"),
                    ] {
                        let (ctx, theme) = crate::ui::test_support::themed_context(dark, scale);
                        crate::ui::test_support::painted_frame(
                            &ctx,
                            egui::vec2(width, 180.0),
                            vec![],
                            |_| {},
                        );
                        let content = InspectorContent {
                            under_caret: None,
                            under_caret_loading: false,
                            under_caret_error: None,
                            definition_preview: &preview,
                            definition_expanded: false,
                            declaration: None,
                            buffer: None,
                            reference_results: &results,
                            selected_reference: None,
                            reference_disclosures: &disclosures,
                            reference_target: None,
                            reference_kind: 0,
                            explicit_references: false,
                            references_loading: false,
                            reference_error: None,
                            references_available: false,
                            outline: &[],
                            outline_loading: loading,
                            outline_error: error,
                            test_code: None,
                            bookmarks: &bookmarks,
                            scope: "Checkout",
                            bookmarks_enabled: false,
                            can_toggle_bookmark: false,
                            current_bookmarked: false,
                            outline_enabled: false,
                        };
                        let mut outline = OutlineList::default();
                        let mut render = || {
                            crate::ui::test_support::painted_frame(
                                &ctx,
                                egui::vec2(width, 180.0),
                                vec![],
                                |ui| {
                                    ContextInspector::render_under_caret_context(
                                        ui,
                                        &theme,
                                        &content,
                                        &mut outline,
                                    );
                                    assert!(ui.min_rect().width() <= width, "width={width} scale={scale} loading={loading} error={} rect={:?}", error.is_some(), ui.min_rect());
                                },
                            )
                        };
                        render();
                        let output = render();
                        let text = crate::ui::test_support::text_shapes(&output)
                            .find(|text| text.galley.job.text.starts_with(expected))
                            .unwrap();
                        assert!(text.galley.size().x <= width - 16.0);
                        if let Some(error) = error {
                            assert_explanation_layout(&text.galley, error);
                        }
                        if loading {
                            assert!(!crate::ui::test_support::text_shapes(&output)
                                .any(|text| text.galley.job.text.contains("Open a source file")));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn all_three_context_regions_are_visible_in_one_frame() {
        let name = format!("λ{}_TAIL", "identifier".repeat(100));
        let buffer = SourceBuffer::from_str(
            format!("nested/{}.rs", "long-filename-".repeat(20)),
            &format!(
                "{}fn {name}() {{}}{}",
                "mod nested {".repeat(20),
                "}".repeat(20)
            ),
        );
        let symbols = crate::syntax::outline::SymbolExtractor::extract(&buffer);
        let mut symbol = &symbols[0];
        while let Some(child) = symbol.children.first() {
            symbol = child;
        }
        let mut bookmarks = BookmarkStore::new();
        bookmarks
            .bookmarks
            .push(crate::bookmarks::bookmark::SourceBookmark::new(
                &buffer,
                symbol.selection_range.start,
                &name,
            ));
        for dark in [true, false] {
            for scale in [1.0, 2.0] {
                for width in [220.0, 300.0, 420.0] {
                    let (ctx, theme) = crate::ui::test_support::themed_context(dark, scale);
                    let mut outline = OutlineList::default();
                    outline.sync(
                        &buffer,
                        &symbols,
                        &Default::default(),
                        symbol.selection_range.start,
                        None,
                    );
                    crate::ui::test_support::painted_frame(
                        &ctx,
                        egui::vec2(1280.0, 800.0),
                        vec![],
                        |_| {},
                    );
                    let mut render = || {
                        let mut pane = egui::Rect::NOTHING;
                        let output = crate::ui::test_support::painted_frame(
                            &ctx,
                            egui::vec2(1280.0, 800.0),
                            vec![],
                            |ui| {
                                pane = egui::SidePanel::right("inspector-test")
                                    .default_width(width)
                                    .min_width(200.0)
                                    .show_inside(ui, |ui| {
                                        ui.style_mut().interaction.selectable_labels = false;
                                        ContextInspector::show(
                                            ui,
                                            &theme,
                                            InspectorContent {
                                                under_caret: None,
                                                under_caret_loading: false,
                                                under_caret_error: None,
                                                definition_preview:
                                                    &crate::source::preview::PreviewPhase::None,
                                                definition_expanded: false,
                                                declaration: Some(symbol),
                                                buffer: Some(&buffer),
                                                reference_results: &ReferenceResults::default(),
                                                selected_reference: None,
                                                reference_disclosures: &Default::default(),
                                                reference_target: None,
                                                reference_kind: 0,
                                                explicit_references: false,
                                                references_loading: false,
                                                reference_error: None,
                                                references_available: false,
                                                outline: &symbols,
                                                outline_loading: false,
                                                outline_error: None,
                                                test_code: None,
                                                bookmarks: &bookmarks,
                                                scope: &"Review scope with long title".repeat(200),
                                                bookmarks_enabled: true,
                                                can_toggle_bookmark: true,
                                                current_bookmarked: false,
                                                outline_enabled: true,
                                            },
                                            &mut Default::default(),
                                            &mut Default::default(),
                                            &mut outline,
                                            &mut Default::default(),
                                        );
                                    })
                                    .response
                                    .rect;
                            },
                        );
                        (output, pane)
                    };
                    let (output, pane) = render();
                    assert_region_geometry(&output, pane, width);
                    ctx.data_mut(|data| {
                        for name in ["declaration_region", "bookmarks_region"] {
                            data.insert_persisted(
                                egui::Id::new(name),
                                egui::containers::panel::PanelState {
                                    rect: egui::Rect::from_min_size(
                                        egui::Pos2::ZERO,
                                        egui::vec2(width, 10000.0),
                                    ),
                                },
                            );
                        }
                    });
                    let (output, pane) = render();
                    assert_region_geometry(&output, pane, width);
                }
            }
        }
    }
    fn assert_region_geometry(output: &egui::FullOutput, pane: egui::Rect, width: f32) {
        let labels: Vec<_> = crate::ui::test_support::text_shapes(output).collect();
        for heading in ["Declaration", "Outline", "Bookmarks", "Ctrl+0"] {
            assert!(
                labels.iter().any(|text| text.galley.job.text == heading),
                "missing {heading}"
            );
        }
        // egui's 200-point content minimum includes a further 16 points of panel padding.
        assert!(
            pane.width() <= width.max(216.0) + 1.0,
            "{width} grew to {}",
            pane.width()
        );
        assert_eq!(
            labels
                .iter()
                .filter(|text| text.galley.job.text.ends_with("_TAIL"))
                .count(),
            3,
            "declaration, deep outline and bookmark labels must keep suffixes"
        );
        for text in &labels {
            assert!(
                text.pos.x >= pane.left() - 0.1
                    && text.pos.x + text.galley.size().x <= pane.right() + 0.1,
                "label escaped {width}: {:?}",
                text.galley.job.text
            );
        }
        let y = |heading| {
            labels
                .iter()
                .find(|text| text.galley.job.text == heading)
                .unwrap()
                .pos
                .y
        };
        assert!(
            y("Outline") > 250.0,
            "declaration region collapsed around its content"
        );
        assert!(y("Bookmarks") > y("Outline") + 100.0);
    }

    #[test]
    fn fast_definition_keys_preserve_event_order_and_do_not_leak() {
        let ctx = egui::Context::default();
        let mut actions = Vec::new();
        let _ = ctx.run(
            egui::RawInput {
                events: [
                    egui::Key::ArrowLeft,
                    egui::Key::ArrowRight,
                    egui::Key::Enter,
                ]
                .into_iter()
                .map(crate::ui::commands::test_key)
                .collect(),
                ..Default::default()
            },
            |ctx| {
                ctx.memory_mut(|memory| {
                    memory.request_focus(ContextInspector::definition_focus_id())
                });
                let mut events = ctx.input_mut(|input| std::mem::take(&mut input.events));
                actions = crate::ui::commands::take_event_actions(&mut events, |key, modifiers| {
                    (modifiers == egui::Modifiers::NONE)
                        .then(|| ContextInspector::definition_key(ctx, key, modifiers))
                        .flatten()
                });
                ctx.input_mut(|input| input.events = events);
                assert!(ctx.input(|input| input.events.is_empty()));
            },
        );
        assert_eq!(
            actions,
            [
                InspectorAction::ExpandDefinition(false),
                InspectorAction::ExpandDefinition(true),
                InspectorAction::GoToDefinition
            ]
        );
    }
}
