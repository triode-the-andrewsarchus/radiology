//! Independently resizable declaration, definition/outline, and bookmark regions.

use crate::bookmarks::store::BookmarkStore;
use crate::semantic::caller_tree::{caller_pill_color, ReferenceCallerNode};
use crate::semantic::scip_proto::ScipSymbolInformation;
use crate::source::SourceBuffer;
use crate::syntax::outline::{outline_rows, DocumentSymbol, OutlineRow};
use crate::theme::AppTheme;
use egui::{Color32, RichText, ScrollArea, Ui, Vec2};
use uuid::Uuid;

/// Navigation and management events from the inspector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InspectorAction {
    /// Opens the exact semantic definition under the source caret.
    GoToDefinition,
    /// Releases an explicit reference target and resumes enclosing-declaration context.
    FollowCaret,
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
    /// Definition path and canonical line for the symbol under the caret.
    pub definition: Option<(&'a str, usize)>,
    /// Innermost parsed declaration enclosing the caret.
    pub declaration: Option<&'a DocumentSymbol>,
    /// Exact active source bytes for navigation and line counts.
    pub buffer: Option<&'a SourceBuffer>,
    /// Incoming references to the enclosing declaration.
    pub callers: &'a [ReferenceCallerNode],
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
    /// Renders independently scrolling regions and returns a requested action.
    pub fn show(
        ui: &mut Ui,
        theme: &AppTheme,
        content: InspectorContent<'_>,
    ) -> Option<InspectorAction> {
        let mut action = None;
        let height = ui.available_height();
        egui::TopBottomPanel::top("declaration_region")
            .resizable(true)
            .default_height(height * 0.38)
            .min_height(100.0)
            .max_height((height - 220.0).max(100.0))
            .show_inside(ui, |ui| {
                action = Self::render_declaration_context(ui, theme, &content);
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
                            ui.label(RichText::new(&symbol.display_name).strong());
                            if let Some((path, line)) = content.definition {
                                if ui.link(format!("{path}:{line}")).clicked() {
                                    action = Some(InspectorAction::GoToDefinition);
                                }
                            }
                            if !symbol.signature.is_empty() {
                                ui.monospace(&symbol.signature);
                            }
                            for doc in &symbol.documentation {
                                ui.label(doc);
                            }
                        } else if let Some(buffer) = content.buffer {
                            for row in
                                outline_rows(content.outline, buffer)
                                    .into_iter()
                                    .filter(|row| {
                                        !content.test_code.is_some_and(|tests| {
                                            tests.hides_declaration(&row.symbol.range)
                                        })
                                    })
                            {
                                if let Some(next) =
                                    Self::render_outline_row(ui, row, content.declaration)
                                {
                                    action = Some(next);
                                }
                            }
                        }
                    });
            });
        action
    }

    fn render_declaration_context(
        ui: &mut Ui,
        theme: &AppTheme,
        content: &InspectorContent<'_>,
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
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("References · {}", content.callers.len())).strong());
            if content.explicit_references && ui.small_button("Follow Caret").clicked() {
                action = Some(InspectorAction::FollowCaret);
            }
        });
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .id_source("declaration_references")
            .show_rows(ui, 24.0, content.callers.len().max(1), |ui, visible| {
                let status = if let Some(error) = content.reference_error {
                    Some(error)
                } else if content.references_loading {
                    Some("Loading references…")
                } else if !content.references_available {
                    Some("Semantic data unavailable for this revision.")
                } else if content.reference_target.is_none() {
                    Some("No exact indexed declaration here.")
                } else if content.callers.is_empty() {
                    Some("No indexed incoming references.")
                } else {
                    None
                };
                if let Some(status) = status {
                    ui.colored_label(theme.text_muted, status);
                }
                for node in content.callers.get(visible).unwrap_or_default() {
                    if let Some(next) = Self::render_caller_node(ui, theme, node, 0) {
                        action = Some(next);
                    }
                }
            });
        action
    }

    /// Renders a single caller node in the incoming references tree with identity pill.
    fn render_caller_node(
        ui: &mut Ui,
        theme: &AppTheme,
        node: &ReferenceCallerNode,
        indent: usize,
    ) -> Option<InspectorAction> {
        let mut action = None;
        let indent_px = indent as f32 * 14.0;
        let row_height = 24.0;

        let (rect, resp) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), row_height),
            egui::Sense::click(),
        );

        let is_hovered = resp.hovered();
        if is_hovered {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            let hover_bg = if theme.is_dark {
                Color32::from_rgba_unmultiplied(255, 255, 255, 12)
            } else {
                Color32::from_rgba_unmultiplied(0, 0, 0, 10)
            };
            ui.painter().rect_filled(rect, 3.0, hover_bg);
        }

        let mut child_ui = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
        child_ui.add_space(indent_px + 4.0);

        // Deterministic 12-color capsule identity pill
        let pill_color = caller_pill_color(&node.name, &node.path);
        egui::Frame::none()
            .fill(pill_color.linear_multiply(0.2))
            .stroke(egui::Stroke::new(1.0_f32, pill_color))
            .inner_margin(egui::Margin::symmetric(6.0, 2.0))
            .rounding(10.0)
            .show(&mut child_ui, |ui| {
                ui.label(
                    RichText::new(&node.name)
                        .color(pill_color)
                        .strong()
                        .size(11.0),
                );
            });

        let file_short = std::path::Path::new(&node.path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| node.path.clone());
        child_ui.add_space(4.0);
        child_ui.colored_label(
            theme.text_secondary,
            RichText::new(format!("{file_short}:{}", node.line)).size(11.0),
        );

        if node.is_recursive {
            child_ui.colored_label(Color32::from_rgb(234, 179, 8), "(recursion loop)");
        }

        if resp.clicked() {
            action = Some(InspectorAction::JumpToIndexedLocation {
                path: node.path.clone(),
                range: node.range.clone(),
            });
        }

        for child in &node.children {
            if let Some(act) = Self::render_caller_node(ui, theme, child, indent + 1) {
                action = Some(act);
            }
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
                            definition: None,
                            declaration: None,
                            buffer: None,
                            callers: &[],
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
    fn caller_row_pointer_navigation_preserves_the_encoded_reference_range() {
        let context = egui::Context::default();
        let range = crate::semantic::ScipRange::from_packed(&[1, 7, 13]).unwrap();
        let node = ReferenceCallerNode {
            name: "caller".into(),
            path: "b.ts".into(),
            line: 2,
            range: range.clone(),
            depth: 0,
            color_slot: 0,
            is_recursive: false,
            children: vec![],
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
                action = ContextInspector::render_caller_node(ui, &AppTheme::dark(), &node, 0);
            });
        });
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
                        action =
                            ContextInspector::render_caller_node(ui, &AppTheme::dark(), &node, 0);
                    });
                },
            );
        }
        assert_eq!(
            action,
            Some(InspectorAction::JumpToIndexedLocation {
                path: "b.ts".into(),
                range
            })
        );
    }
}
