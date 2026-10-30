//! Independently resizable declaration, definition/outline, and bookmark regions.

use crate::bookmarks::store::BookmarkStore;
use crate::semantic::reference_results::ReferenceResults;
use crate::semantic::scip_proto::ScipSymbolInformation;
use crate::source::SourceBuffer;
use crate::syntax::outline::{outline_rows, DocumentSymbol, OutlineRow};
use crate::theme::AppTheme;
use egui::{RichText, ScrollArea, Ui};
use uuid::Uuid;

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
    /// Removes a persisted bookmark.
    RemoveBookmark(Uuid),
    /// Moves a bookmark toward the start of the list.
    MoveBookmarkUp(usize),
    /// Moves a bookmark toward the end of the list.
    MoveBookmarkDown(usize),
    /// Resolves a bookmark against current bytes before navigating.
    OpenBookmark(usize),
}

/// Independent enclosing-declaration and under-caret facts for one frame.
pub struct InspectorContent<'a> {
    /// Symbol under the caret, independently of the enclosing declaration.
    pub current_symbol: Option<&'a ScipSymbolInformation>,
    /// Bounded source excerpt for the canonical definition destination.
    pub definition_preview: &'a crate::source::preview::PreviewPhase,
    /// Whether the under-caret details are disclosed.
    pub definition_expanded: bool,
    /// Definition path and canonical line for the symbol under the caret.
    pub definition: Option<(&'a str, usize)>,
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
    /// Current test-region projection for outline visibility.
    pub test_code: Option<&'a crate::source::test_code::TestCodePresentation>,
    /// Bookmarks in the active checkout or PR scope.
    pub bookmarks: &'a BookmarkStore,
    /// Visible checkout or PR scope title.
    pub scope: &'a str,
}

/// Renders the three simultaneously usable context regions.
pub struct ContextInspector;

impl ContextInspector {
    pub(crate) fn definition_focus_id() -> egui::Id {
        egui::Id::new("definition_context_focus")
    }

    pub(crate) fn take_definition_actions(ctx: &egui::Context) -> Vec<InspectorAction> {
        if !ctx.memory(|memory| memory.has_focus(Self::definition_focus_id())) {
            return Vec::new();
        }
        super::commands::take_key_actions(ctx, |key, modifiers| {
            if modifiers != egui::Modifiers::NONE {
                return None;
            }
            match key {
                egui::Key::ArrowRight => Some(InspectorAction::ExpandDefinition(true)),
                egui::Key::ArrowLeft => Some(InspectorAction::ExpandDefinition(false)),
                egui::Key::Enter => Some(InspectorAction::GoToDefinition),
                _ => None,
            }
        })
    }

    /// Renders independently scrolling regions and returns a requested action.
    pub fn show(
        ui: &mut Ui,
        theme: &AppTheme,
        content: InspectorContent<'_>,
        references: &mut crate::ui::reference_list::ReferenceViewport,
    ) -> Option<InspectorAction> {
        let mut action = None;
        let height = ui.available_height();
        egui::TopBottomPanel::top("declaration_region")
            .resizable(true)
            .default_height(height * 0.38)
            .min_height(100.0)
            .max_height((height - 220.0).max(100.0))
            .show_inside(ui, |ui| {
                action = Self::render_declaration_context(ui, theme, &content, references);
            });
        egui::TopBottomPanel::bottom("bookmarks_region")
            .resizable(true)
            .default_height(height * 0.25)
            .min_height(100.0)
            .max_height((height * 0.5).max(100.0))
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Bookmarks").strong());
                    ui.colored_label(theme.text_muted, content.scope);
                });
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .id_source("reader_bookmarks")
                    .show(ui, |ui| {
                        if content.bookmarks.bookmarks.is_empty() {
                            ui.colored_label(theme.text_muted, "Ctrl+D to bookmark this location.");
                        }
                        for (index, bookmark) in content.bookmarks.bookmarks.iter().enumerate() {
                            let label = if index < 10 {
                                format!("Ctrl+{index}  {}", bookmark.label)
                            } else {
                                bookmark.label.clone()
                            };
                            let response = ui.add(egui::Button::new(label).frame(false));
                            if response.clicked() {
                                action = Some(InspectorAction::OpenBookmark(index));
                            }
                            if let Some(anchor) = &bookmark.anchor {
                                ui.colored_label(
                                    theme.text_secondary,
                                    RichText::new(anchor).monospace(),
                                );
                            }
                            response.context_menu(|ui| {
                                let entries = [
                                    ("Go to Bookmark", true, InspectorAction::OpenBookmark(index)),
                                    ("Move Up", index > 0, InspectorAction::MoveBookmarkUp(index)),
                                    (
                                        "Move Down",
                                        index + 1 < content.bookmarks.bookmarks.len(),
                                        InspectorAction::MoveBookmarkDown(index),
                                    ),
                                    (
                                        "Remove Bookmark",
                                        true,
                                        InspectorAction::RemoveBookmark(bookmark.id),
                                    ),
                                ];
                                for (label, enabled, next) in entries {
                                    if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
                                        action = Some(next);
                                        ui.close_menu();
                                    }
                                }
                            });
                        }
                    });
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show_inside(ui, |ui| {
                if let Some(next) = Self::render_under_caret_context(ui, theme, &content) {
                    action = Some(next);
                }
            });
        action
    }

    fn render_under_caret_context(
        ui: &mut Ui,
        theme: &AppTheme,
        content: &InspectorContent<'_>,
    ) -> Option<InspectorAction> {
        let mut action = None;
        if content.current_symbol.is_some() {
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
        let heading = if content.current_symbol.is_some() {
            "Definition"
        } else {
            "Outline"
        };
        ui.label(RichText::new(heading).strong());
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .id_source("definition_or_outline")
            .show(ui, |ui| {
                if let Some(symbol) = content.current_symbol {
                    if ui
                        .add(
                            egui::Button::new(RichText::new(&symbol.display_name).strong())
                                .frame(false),
                        )
                        .clicked()
                    {
                        ui.memory_mut(|memory| memory.request_focus(Self::definition_focus_id()));
                    }
                    if let Some((path, line)) = content.definition {
                        if ui.link(format!("{path}:{line}")).clicked() {
                            action = Some(InspectorAction::GoToDefinition);
                        }
                    }
                    let label = if content.definition_expanded {
                        "Hide details"
                    } else {
                        "Show details"
                    };
                    if ui.small_button(label).clicked() {
                        ui.memory_mut(|memory| memory.request_focus(Self::definition_focus_id()));
                        action = Some(InspectorAction::ExpandDefinition(
                            !content.definition_expanded,
                        ));
                    }
                    if content.definition_expanded {
                        if !symbol.signature.is_empty() {
                            ui.monospace(&symbol.signature);
                        }
                        for doc in &symbol.documentation {
                            ui.label(doc);
                        }
                        crate::ui::source_preview::show(ui, theme, content.definition_preview);
                    }
                } else if let Some(buffer) = content.buffer {
                    for row in outline_rows(content.outline, buffer)
                        .into_iter()
                        .filter(|row| {
                            !content
                                .test_code
                                .is_some_and(|tests| tests.hides_declaration(&row.symbol.range))
                        })
                    {
                        if let Some(next) = Self::render_outline_row(ui, row, content.declaration) {
                            action = Some(next);
                        }
                    }
                }
            });
        action
    }

    fn render_declaration_context(
        ui: &mut Ui,
        theme: &AppTheme,
        content: &InspectorContent<'_>,
        references: &mut crate::ui::reference_list::ReferenceViewport,
    ) -> Option<InspectorAction> {
        let mut action = None;
        ui.heading("Declaration");
        if content.explicit_references {
            ui.label(RichText::new(content.reference_target.unwrap_or("Pinned symbol")).strong());
        } else if let Some(declaration) = content.declaration {
            if ui.link(&declaration.name).clicked() {
                action = Some(InspectorAction::JumpToSymbol(
                    declaration.selection_range.start,
                ));
            }
            if let Some(signature) = &declaration.detail {
                ui.monospace(signature);
            }
        } else {
            ui.colored_label(theme.text_muted, "No enclosing declaration");
        }
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

    fn render_outline_row(
        ui: &mut Ui,
        row: OutlineRow<'_>,
        active: Option<&DocumentSymbol>,
    ) -> Option<InspectorAction> {
        let selected =
            active.is_some_and(|active| active.selection_range == row.symbol.selection_range);
        let label = format!(
            "{}{}  {} L",
            "  ".repeat(row.depth),
            row.symbol.name,
            row.line_count
        );
        let response = ui
            .selectable_label(selected, label)
            .on_hover_text(row.symbol.kind.title());
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        response.clicked().then_some(InspectorAction::JumpToSymbol(
            row.symbol.selection_range.start,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_three_context_regions_are_visible_in_one_frame() {
        let ctx = egui::Context::default();
        let bookmarks = BookmarkStore::new();
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(320.0, 800.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ContextInspector::show(
                        ui,
                        &AppTheme::dark(),
                        InspectorContent {
                            current_symbol: None,
                            definition_preview: &crate::source::preview::PreviewPhase::None,
                            definition_expanded: false,
                            definition: None,
                            declaration: None,
                            buffer: None,
                            reference_results: &ReferenceResults::default(),
                            selected_reference: None,
                            reference_disclosures: &Default::default(),
                            reference_target: None,
                            explicit_references: false,
                            references_loading: false,
                            reference_error: None,
                            references_available: false,
                            outline: &[],
                            test_code: None,
                            bookmarks: &bookmarks,
                            scope: "Checkout",
                        },
                        &mut Default::default(),
                    );
                });
            },
        );
        let text = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        for heading in ["Declaration", "Outline", "Bookmarks", "Checkout"] {
            assert!(text.contains(&heading), "missing {heading}: {text:?}");
        }
        let y = |heading| {
            output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Text(text) if text.galley.job.text == heading => Some(text.pos.y),
                    _ => None,
                })
                .unwrap()
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
                .map(|key| egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                })
                .collect(),
                ..Default::default()
            },
            |ctx| {
                ctx.memory_mut(|memory| {
                    memory.request_focus(ContextInspector::definition_focus_id())
                });
                actions = ContextInspector::take_definition_actions(ctx);
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
