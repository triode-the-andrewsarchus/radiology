//! Independently resizable declaration, definition/outline, and bookmark regions.

use super::bookmark_list::{BookmarkAction, BookmarkList};
use super::outline_list::{OutlineAction, OutlineList};
use crate::bookmarks::store::BookmarkStore;
use crate::semantic::availability::SemanticAvailability;
use crate::semantic::reference_results::ReferenceResults;
use crate::source::SourceBuffer;
use crate::syntax::outline::DocumentSymbol;
use crate::theme::AppTheme;
use egui::{RichText, ScrollArea, Ui};
mod declaration_summary;
pub(crate) mod panes;
pub use declaration_summary::DeclarationSummary;

/// Navigation and management events from the inspector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InspectorAction {
    /// Selects or activates a contextual Rust trait implementation.
    Implementation(crate::ui::trait_implementations::TraitAction),
    /// Applies a shared reader presentation command.
    Command(crate::ui::commands::ReaderCommand),
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
    /// Exact identity shown in the enclosing-declaration or pinned summary above.
    ///
    /// Pass `None` when that summary has no indexed identity. Document-scoped
    /// local IDs keep same-named symbols in different files distinct.
    pub summary_identity: Option<&'a crate::semantic::index::SemanticSymbolID>,
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
    /// Source coverage and index lifecycle, shared with the Outline status.
    pub semantic_availability: SemanticAvailability<'a>,
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
    /// Visible checkout or PR scope title; absent until a repository is accepted.
    pub scope: Option<&'a str>,
    /// Whether checkout/review transitions permit bookmark interaction.
    pub bookmarks_enabled: bool,
    /// Whether the current source is ready for bookmarking.
    pub can_toggle_bookmark: bool,
    /// Whether a bookmark belongs to the exact current source location.
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
        Self::show_with_panes(
            ui,
            theme,
            content,
            references,
            bookmarks,
            outline,
            declaration,
            &Default::default(),
        )
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
        ui.horizontal(|ui| {
            ui.label(RichText::new(heading).strong());
            if let Some(detail) = content.semantic_availability.header_detail() {
                let galley = super::text_fit::middle(
                    ui,
                    detail,
                    egui::TextStyle::Small.resolve(ui.style()),
                    theme.text_secondary,
                    ui.available_width(),
                );
                ui.add(egui::Label::new(galley))
                    .on_hover_text(super::text_fit::bounded(
                        content
                            .semantic_availability
                            .explanation()
                            .unwrap_or(detail),
                    ));
            }
        });
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
            if content.buffer.is_none() {
                if !content.outline_loading {
                    super::text_fit::explanation(
                        ui,
                        "Open a source file to see its declarations.",
                        theme.text_secondary,
                    );
                }
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
                        content.summary_identity == Some(&context.id),
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
        if content.declaration.is_none() && !content.explicit_references {
            *summary = Default::default();
            let (title, detail) = match content.buffer {
                None => (
                    "No File",
                    "Open a source file to see the declaration around the caret.",
                ),
                Some(buffer) => {
                    let detail = if content.outline_loading {
                        "Reading structure…"
                    } else if crate::syntax::LanguageRegistry::for_path(&buffer.path)
                        .grammar
                        .is_some()
                        && content.outline_error.is_none()
                    {
                        "Place the caret inside a function, type, or other declaration."
                    } else {
                        "This file has no parsed structure."
                    };
                    ("No Declaration", detail)
                }
            };
            ui.colored_label(theme.text_muted, title);
            super::text_fit::explanation(ui, detail, theme.text_secondary);
            return None;
        }
        // Keep the name first; reserve the reference heading, coverage and a file/row.
        // Long or expanded summaries scroll within their share of the region.
        let reference_space = 3.0 * ui.text_style_height(&egui::TextStyle::Body)
            + 5.0 * ui.spacing().item_spacing.y
            + 48.0;
        let summary_height = (ui.available_height() - reference_space).max(18.0);
        let mut action = ScrollArea::vertical()
            .id_source((
                "declaration_summary",
                content
                    .buffer
                    .map(crate::source::buffer::SourceIdentity::new),
                content
                    .declaration
                    .map(|symbol| symbol.selection_range.clone()),
                content
                    .explicit_references
                    .then_some(content.reference_target),
            ))
            .auto_shrink([false, true])
            .min_scrolled_height(0.0)
            .drag_to_scroll(false)
            .max_height(summary_height)
            .show(ui, |ui| {
                summary.show(
                    ui,
                    theme,
                    content.declaration,
                    content.buffer,
                    content.explicit_references.then_some((
                        content.reference_target.unwrap_or("Pinned symbol"),
                        content.reference_kind,
                    )),
                )
            })
            .inner;
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
        let status = reference_status(content);
        let heading = if status.is_some() {
            "References".to_owned()
        } else {
            format!("References · {}", content.reference_results.reference_count)
        };
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(RichText::new(heading).strong());
            if content.explicit_references && ui.small_button("Follow Caret").clicked() {
                action = Some(InspectorAction::FollowCaret);
            }
        });
        if content.semantic_availability == SemanticAvailability::Available
            && content.reference_target.is_some()
            && content.reference_error.is_none()
        {
            let coverage_height = (ui.available_height() / 3.0).max(0.0);
            if let Some(next) = ScrollArea::vertical()
                .id_source(("reference_coverage", content.reference_target))
                .auto_shrink([false, true])
                .min_scrolled_height(0.0)
                .drag_to_scroll(false)
                .max_height(coverage_height)
                .show(ui, |ui| {
                    references.show_coverage(
                        ui,
                        theme,
                        content.reference_results,
                        content.references_loading,
                    )
                })
                .inner
            {
                action = Some(InspectorAction::Reference(next));
            }
        }
        if let Some((title, status)) = status {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .id_source("reference_status")
                .show(ui, |ui| {
                    ui.label(RichText::new(title).strong());
                    if content.reference_error.is_some() {
                        // Preserve the complete worker diagnostic in the scrollable region.
                        ui.colored_label(theme.text_muted, status);
                    } else {
                        super::text_fit::explanation(ui, status, theme.text_secondary);
                    }
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

// Keep status precedence separate from presentation and preserve worker diagnostics.
fn reference_status<'a>(content: &InspectorContent<'a>) -> Option<(&'static str, &'a str)> {
    if let Some(error) = content.reference_error {
        return Some(("References unavailable", error));
    }
    if let Some(reason) = content.semantic_availability.explanation() {
        return Some((
            content
                .semantic_availability
                .header_detail()
                .unwrap_or("No File"),
            reason,
        ));
    }
    if content.references_loading && content.reference_results.hits.is_empty() {
        return Some(("Finding references…", "References are being prepared."));
    }
    content.reference_target.is_none().then_some(("Not in the index",
        "The index records no symbol for this declaration. It may be inactive in the indexed configuration (for example, a cfg-gated item), or the indexer does not name this construct."))
}

// Panel layout owns its extent; clipped child content cannot enlarge a sibling region.
fn region(
    ui: &mut Ui,
    pane: crate::session::inspector::InspectorPane,
    render: impl FnOnce(&mut Ui) -> Option<InspectorAction>,
) -> Option<InspectorAction> {
    let rect = ui.available_rect_before_wrap();
    let mut child = ui.child_ui(rect, egui::Layout::top_down(egui::Align::Min));
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    let action = super::commands::owned_ui(
        &mut child,
        egui::Id::new(("radiology_context_pane", pane)),
        render,
    );
    // Content overflow must not enlarge a resizable region into its neighbors.
    ui.allocate_space(rect.size().max(egui::Vec2::ZERO));
    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct InspectorFixture {
        bookmarks: BookmarkStore,
        results: ReferenceResults,
        disclosures:
            std::collections::BTreeMap<usize, crate::semantic::disclosure::ReferenceDisclosure>,
        preview: crate::source::preview::PreviewPhase,
    }

    impl InspectorFixture {
        fn content(&self) -> InspectorContent<'_> {
            InspectorContent {
                under_caret: None,
                under_caret_loading: false,
                under_caret_error: None,
                definition_preview: &self.preview,
                definition_expanded: false,
                declaration: None,
                buffer: None,
                reference_results: &self.results,
                selected_reference: None,
                reference_disclosures: &self.disclosures,
                reference_target: None,
                summary_identity: None,
                reference_kind: 0,
                explicit_references: false,
                references_loading: false,
                reference_error: None,
                semantic_availability: SemanticAvailability::NoFile,
                outline: &[],
                outline_loading: false,
                outline_error: None,
                test_code: None,
                bookmarks: &self.bookmarks,
                scope: Some("Checkout"),
                bookmarks_enabled: false,
                can_toggle_bookmark: false,
                current_bookmarked: false,
                outline_enabled: false,
            }
        }
    }

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
    fn reference_and_outline_status_share_source_coverage_without_false_zero_results() {
        use crate::ui::test_support::{painted_frame, text_shapes, themed_context};
        let fixture = InspectorFixture::default();
        let buffer = SourceBuffer::from_str("a.rs", "fn alpha() {}\n");
        for (dark, scale) in [(true, 1.0), (false, 1.5), (false, 2.0)] {
            for availability in [
                SemanticAvailability::RepositoryNotIndexed("The index could not be read."),
                SemanticAvailability::Indexing,
                SemanticAvailability::FileNotIndexed,
                SemanticAvailability::SourceChanged,
                SemanticAvailability::RevisionNotIndexed,
            ] {
                let (ctx, theme) = themed_context(dark, scale);
                let content = InspectorContent {
                    buffer: Some(&buffer),
                    semantic_availability: availability,
                    reference_target: Some("alpha"),
                    ..fixture.content()
                };
                let mut viewport = crate::ui::reference_list::ReferenceViewport::default();
                let mut outline = OutlineList::default();
                painted_frame(&ctx, egui::vec2(240.0, 400.0), vec![], |_| {});
                let output = painted_frame(&ctx, egui::vec2(240.0, 400.0), vec![], |ui| {
                    ContextInspector::render_under_caret_context(
                        ui,
                        &theme,
                        &content,
                        &mut outline,
                    );
                    ContextInspector::render_reference_context(ui, &theme, &content, &mut viewport);
                });
                let texts = text_shapes(&output)
                    .map(|t| t.galley.job.text.clone())
                    .collect::<Vec<_>>();
                let heading = availability.header_detail().unwrap();
                assert_eq!(
                    texts.iter().filter(|t| t.as_str() == heading).count(),
                    2,
                    "{texts:?}"
                );
                assert!(
                    texts
                        .iter()
                        .any(|t| t == availability.explanation().unwrap()),
                    "{texts:?}"
                );
                assert!(
                    !texts.iter().any(|t| t.contains("No references shown")),
                    "{texts:?}"
                );
                for text in text_shapes(&output) {
                    assert!(
                        text.galley.size().x <= 224.5,
                        "scale={scale} dark={dark} width={} {:?} layout={:?}: {texts:?}",
                        text.galley.size().x,
                        text.galley.job.text,
                        text.galley.job.wrap
                    );
                }
            }
        }
        let mut content = InspectorContent {
            semantic_availability: SemanticAvailability::Available,
            ..fixture.content()
        };
        assert_eq!(reference_status(&content).unwrap().0, "Not in the index");
        assert!(reference_status(&content).unwrap().1.contains("cfg-gated"));
        content.reference_target = Some("alpha");
        assert!(reference_status(&content).is_none()); // A real loaded zero uses coverage.
        content.references_loading = true;
        assert_eq!(reference_status(&content).unwrap().0, "Finding references…");
        content.reference_error = Some("full worker failure details");
        assert_eq!(
            reference_status(&content),
            Some(("References unavailable", "full worker failure details"))
        );
        let (ctx, theme) = themed_context(true, 1.0);
        let mut viewport = crate::ui::reference_list::ReferenceViewport::default();
        let texts = crate::ui::test_support::painted_text(&ctx, egui::vec2(240.0, 400.0), |ui| {
            ContextInspector::render_reference_context(ui, &theme, &content, &mut viewport);
        });
        assert!(texts.iter().any(|t| t == "full worker failure details"));
        assert!(!texts.iter().any(|t| t.contains("No references shown")));
    }

    #[test]
    fn definition_heading_reuse_requires_the_exact_global_or_local_identity() {
        use crate::semantic::index::SemanticSymbolID;
        use crate::ui::test_support::{
            click, indexed_under_caret_context, painted_frame, text_center, text_shapes,
            themed_context,
        };
        let base = indexed_under_caret_context();
        let local_here = SemanticSymbolID::new("local 0", "source.rs");
        let local_elsewhere = SemanticSymbolID::new("local 0", "other.rs");
        let fixture = InspectorFixture::default();
        for (dark, scale) in [(true, 1.0), (false, 1.5), (false, 2.0)] {
            for (header, caret, repeated) in [
                (None, base.id.clone(), false),
                (Some(base.id.clone()), base.id.clone(), true),
                (
                    Some(SemanticSymbolID::new("other-global", "source.rs")),
                    base.id.clone(),
                    false,
                ),
                (Some(local_here.clone()), local_here.clone(), true),
                (Some(local_here.clone()), local_elsewhere.clone(), false),
            ] {
                let (ctx, theme) = themed_context(dark, scale);
                let mut context = base.clone();
                context.id = caret;
                let content = InspectorContent {
                    under_caret: Some(&context),
                    reference_target: Some(&context.name),
                    summary_identity: header.as_ref(),
                    ..fixture.content()
                };
                let mut outline = OutlineList::default();
                let mut frame = |events| {
                    let mut action = None;
                    let output = painted_frame(&ctx, egui::vec2(240.0, 300.0), events, |ui| {
                        action = ContextInspector::render_under_caret_context(
                            ui,
                            &theme,
                            &content,
                            &mut outline,
                        );
                    });
                    (output, action)
                };
                frame(vec![]);
                let (output, action) = frame(vec![]);
                assert_eq!(action, None);
                assert_eq!(
                    text_shapes(&output).any(|text| text.galley.job.text == context.name),
                    !repeated
                );
                assert!(ctx.used_rect().width() <= 241.0);
                let (clicked, action) = frame(click(text_center(&output, "source.rs:1")));
                assert_eq!(
                    action,
                    Some(InspectorAction::JumpToIndexedLocation {
                        path: "source.rs".into(),
                        range: context.destination.as_ref().unwrap().range.clone(),
                    })
                );
                assert_eq!(
                    frame(click(text_center(&clicked, "Show details"))).1,
                    Some(InspectorAction::ExpandDefinition(true))
                );
                for (key, expected) in [
                    (
                        egui::Key::ArrowRight,
                        InspectorAction::ExpandDefinition(true),
                    ),
                    (
                        egui::Key::ArrowLeft,
                        InspectorAction::ExpandDefinition(false),
                    ),
                    (egui::Key::Enter, InspectorAction::GoToDefinition),
                ] {
                    assert_eq!(
                        ContextInspector::definition_key(&ctx, key, egui::Modifiers::NONE),
                        Some(expected)
                    );
                }
            }
        }
    }

    #[test]
    fn hidden_reference_summary_cannot_suppress_the_definition_name() {
        let fixture = InspectorFixture::default();
        let context = crate::ui::test_support::indexed_under_caret_context();
        for (dark, scale) in [(true, 1.0), (false, 2.0)] {
            let (ctx, theme) = crate::ui::test_support::themed_context(dark, scale);
            let panes = crate::session::inspector::InspectorPanes {
                references: false,
                definition: true,
                implementations: false,
                bookmarks: false,
            };
            let texts =
                crate::ui::test_support::painted_text(&ctx, egui::vec2(240.0, 400.0), |ui| {
                    let content = InspectorContent {
                        under_caret: Some(&context),
                        summary_identity: Some(&context.id),
                        reference_target: Some(&context.name),
                        ..fixture.content()
                    };
                    ContextInspector::show_with_panes(
                        ui,
                        &theme,
                        content,
                        &mut Default::default(),
                        &mut Default::default(),
                        &mut Default::default(),
                        &mut Default::default(),
                        &panes,
                    );
                });
            assert!(texts.contains(&context.name));
            assert!(!texts.iter().any(|text| text.starts_with("References")));
        }
    }

    #[test]
    fn declaration_absence_reasons_stay_bounded_without_a_reference_region() {
        let parsed = SourceBuffer::from_str("empty.rs", "// Parsed without declarations\n");
        let plain = SourceBuffer::from_str("notes.md", "Plain text\n");
        let fixture = InspectorFixture::default();
        for (dark, scale) in [(true, 1.0), (false, 1.5), (false, 2.0)] {
            for width in [180.0, 300.0] {
                for (buffer, loading, error, title, detail) in [
                    (
                        None,
                        false,
                        None,
                        "No File",
                        "Open a source file to see the declaration around the caret.",
                    ),
                    (
                        Some(&parsed),
                        false,
                        None,
                        "No Declaration",
                        "Place the caret inside a function, type, or other declaration.",
                    ),
                    (
                        Some(&parsed),
                        true,
                        None,
                        "No Declaration",
                        "Reading structure…",
                    ),
                    (
                        Some(&plain),
                        false,
                        None,
                        "No Declaration",
                        "This file has no parsed structure.",
                    ),
                    (
                        Some(&plain),
                        true,
                        None,
                        "No Declaration",
                        "Reading structure…",
                    ),
                    (
                        Some(&parsed),
                        false,
                        Some("Parser failed"),
                        "No Declaration",
                        "This file has no parsed structure.",
                    ),
                    (
                        Some(&parsed),
                        true,
                        Some("Parser failed"),
                        "No Declaration",
                        "Reading structure…",
                    ),
                ] {
                    let (ctx, theme) = crate::ui::test_support::themed_context(dark, scale);
                    let content = InspectorContent {
                        buffer,
                        outline_loading: loading,
                        outline_error: error,
                        ..fixture.content()
                    };
                    let mut summary = DeclarationSummary::default();
                    let mut references = Default::default();
                    let mut render = || {
                        crate::ui::test_support::painted_frame(
                            &ctx,
                            egui::vec2(width, 140.0),
                            vec![],
                            |ui| {
                                assert_eq!(
                                    ContextInspector::render_declaration_context(
                                        ui,
                                        &theme,
                                        &content,
                                        &mut references,
                                        &mut summary
                                    ),
                                    None
                                );
                            },
                        )
                    };
                    render();
                    let output = render();
                    crate::ui::test_support::text_center(&output, title);
                    let shapes = crate::ui::test_support::text_shapes(&output).collect::<Vec<_>>();
                    assert!(!shapes.iter().any(|text| text
                        .galley
                        .job
                        .text
                        .starts_with("References")));
                    let explanation = shapes
                        .iter()
                        .find(|text| text.galley.job.text.starts_with(detail))
                        .unwrap();
                    assert_explanation_layout(&explanation.galley, detail);
                    assert!(ctx.used_rect().width() <= width + 1.0);
                }
            }
        }
    }

    #[test]
    fn small_reference_regions_keep_coverage_and_the_first_hit_in_separate_viewports() {
        use crate::ui::test_support::{
            painted_frame, reference_hit, reference_results, themed_context,
        };
        for (dark, scale) in [(true, 1.0), (false, 1.5), (false, 2.0)] {
            let (ctx, theme) = themed_context(dark, scale);
            let buffer = SourceBuffer::from_str("a.rs", "fn alpha() {}\n");
            let symbols = crate::syntax::outline::SymbolExtractor::extract(&buffer);
            let fixture = InspectorFixture {
                results: ReferenceResults {
                    unmapped: 3,
                    omitted: 4,
                    ..reference_results(vec![reference_hit(0)])
                },
                ..Default::default()
            };
            let content = InspectorContent {
                buffer: Some(&buffer),
                declaration: Some(&symbols[0]),
                reference_target: Some("alpha"),
                selected_reference: Some(0),
                semantic_availability: SemanticAvailability::Available,
                ..fixture.content()
            };
            let mut summary = DeclarationSummary::default();
            let mut references = Default::default();
            let mut action = None;
            let mut render = |events| {
                painted_frame(&ctx, egui::vec2(240.0, 160.0), events, |ui| {
                    action = ContextInspector::render_declaration_context(
                        ui,
                        &theme,
                        &content,
                        &mut references,
                        &mut summary,
                    );
                })
            };
            render(vec![]);
            let output = render(vec![]);
            for label in ["alpha", "References · 1", "caller0"] {
                crate::ui::test_support::assert_visible_text(&output, label);
            }
            let coverage = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Text(text) if text.galley.job.text.contains("unmapped") => {
                        Some(shape.clip_rect)
                    }
                    _ => None,
                })
                .expect("coverage viewport");
            render(vec![
                egui::Event::PointerMoved(coverage.center()),
                egui::Event::Scroll(egui::vec2(0.0, -200.0)),
            ]);
            for _ in 0..8 {
                render(vec![]);
            }
            let scrolled = render(vec![]);
            let button = crate::ui::test_support::text_center(&scrolled, "Load More");
            render(crate::ui::test_support::click(button));
            assert_eq!(
                action,
                Some(InspectorAction::Reference(
                    crate::ui::reference_list::ReferenceAction::More
                ))
            );
        }
    }

    #[test]
    fn leaving_a_declaration_retires_its_signature_disclosure() {
        use crate::ui::test_support::{painted_frame, text_shapes, themed_context};
        for (dark, scale, height) in [(true, 1.0, 160.0), (false, 1.5, 160.0), (false, 2.0, 300.0)]
        {
            let (ctx, theme) = themed_context(dark, scale);
            let buffer = SourceBuffer::from_str("a.rs", "fn alpha(argument: u8) {}\n");
            let symbols = crate::syntax::outline::SymbolExtractor::extract(&buffer);
            let fixture = InspectorFixture::default();
            let mut content = InspectorContent {
                buffer: Some(&buffer),
                declaration: Some(&symbols[0]),
                semantic_availability: SemanticAvailability::RepositoryNotIndexed(
                    "Generate an index to find references.",
                ),
                ..fixture.content()
            };
            let mut summary = DeclarationSummary::default();
            let mut references = Default::default();
            let mut frame = |content: &InspectorContent<'_>, events| {
                painted_frame(&ctx, egui::vec2(300.0, height), events, |ui| {
                    ContextInspector::render_declaration_context(
                        ui,
                        &theme,
                        content,
                        &mut references,
                        &mut summary,
                    );
                })
            };
            let closed = frame(&content, vec![]);
            frame(
                &content,
                vec![crate::ui::commands::test_key(egui::Key::Tab)],
            );
            let expanded = frame(
                &content,
                vec![crate::ui::commands::test_key(egui::Key::Space)],
            );
            frame(
                &content,
                vec![
                    egui::Event::PointerMoved(egui::pos2(100.0, 16.0)),
                    egui::Event::Scroll(egui::vec2(0.0, -200.0)),
                ],
            );
            for _ in 0..8 {
                frame(&content, vec![]);
            }
            let scrolled = frame(&content, vec![]);
            assert!(text_shapes(&scrolled).any(|text| text
                .galley
                .job
                .text
                .starts_with("fn alpha")));
            content.declaration = None;
            let absent = frame(&content, vec![]);
            crate::ui::test_support::text_center(&absent, "No Declaration");
            content.declaration = Some(&symbols[0]);
            frame(&content, vec![]);
            let returned = frame(&content, vec![]);
            assert!(!text_shapes(&returned).any(|text| text
                .galley
                .job
                .text
                .starts_with("fn alpha")));
            content.explicit_references = true;
            content.reference_target = Some("pinned");
            content.reference_kind = 17;
            let pinned = frame(&content, vec![]);
            for (output, name) in [
                (&closed, "alpha"),
                (&expanded, "alpha"),
                (&returned, "alpha"),
                (&pinned, "pinned"),
            ] {
                for label in ["References", name, "Follow Caret"] {
                    if label == "Follow Caret" && name != "pinned" {
                        continue;
                    }
                    crate::ui::test_support::assert_visible_text(output, label);
                }
            }
        }
    }

    #[test]
    fn outline_loading_and_failure_messages_stay_bounded_in_each_theme_and_scale() {
        let failure = format!(
            "Structure unavailable: {}",
            "失敗😀 long diagnostic ".repeat(1000)
        );
        let fixture = InspectorFixture::default();
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
                            outline_loading: loading,
                            outline_error: error,
                            ..fixture.content()
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
        let fixture = InspectorFixture {
            bookmarks,
            ..Default::default()
        };
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
                                                declaration: Some(symbol),
                                                buffer: Some(&buffer),
                                                outline: &symbols,
                                                scope: Some(
                                                    &"Review scope with long title".repeat(200),
                                                ),
                                                bookmarks_enabled: true,
                                                can_toggle_bookmark: true,
                                                outline_enabled: true,
                                                ..fixture.content()
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
        for heading in ["References", "Outline", "Bookmarks", "Ctrl+0"] {
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
