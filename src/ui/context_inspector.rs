//! Right-hand context inspector panel showing symbol definitions, callers, AST outlines, and bookmarks.
//!
//! Provides tabs for inspecting the symbol definition under the caret, viewing recursive
//! call trees with 12-color caller identity pills, browsing the file AST outline, and managing
//! persistent code bookmarks.

use crate::bookmarks::store::BookmarkStore;
use crate::semantic::caller_tree::{caller_pill_color, ReferenceCallerNode};
use crate::semantic::scip_proto::ScipSymbolInformation;
use crate::syntax::outline::DocumentSymbol;
use crate::theme::AppTheme;
use crate::ui::icons::Icon;
use egui::{Color32, RichText, ScrollArea, Ui, Vec2};
use uuid::Uuid;

/// Active tab selected in the [`ContextInspector`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InspectorTab {
    /// Active symbol documentation, type signature, and definition file location.
    #[default]
    Symbol,
    /// Incoming caller call tree with 12-color identity pills.
    References,
    /// Tree-sitter AST document outline.
    Outline,
    /// Bookmarked code locations with auto-relocating textual anchors.
    Bookmarks,
}

/// Navigation and management events produced by user interaction in the [`ContextInspector`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InspectorAction {
    /// Requests jumping to a specific file path, line, and byte offset.
    JumpToLocation {
        /// Target file path relative to repository root.
        path: String,
        /// 1-based line number.
        line: usize,
        /// 0-based byte offset.
        byte_offset: usize,
    },
    /// Requests jumping to the byte offset of a document symbol.
    JumpToSymbol(usize),
    /// Removes the bookmark matching the given unique identifier.
    RemoveBookmark(Uuid),
    /// Re-orders a bookmark upwards in the list.
    MoveBookmarkUp(usize),
    /// Re-orders a bookmark downwards in the list.
    MoveBookmarkDown(usize),
}

/// Renders the right inspector panel for semantic, structural, and bookmark context.
pub struct ContextInspector;

impl ContextInspector {
    /// Renders the inspector tabs and active panel contents.
    ///
    /// # Arguments
    ///
    /// * `ui` - EGUI UI layout context.
    /// * `theme` - Visual color palette.
    /// * `active_tab` - Mutable reference to the currently displayed tab.
    /// * `current_symbol` - SCIP symbol metadata for the token under the caret.
    /// * `definition_location` - Optional target file and line where the active symbol is defined.
    /// * `enclosing_declaration` - AST document symbol enclosing the current caret position.
    /// * `enclosing_line` - 1-based line number of the enclosing declaration.
    /// * `current_file_path` - Path of the currently open source file.
    /// * `caller_tree` - Hierarchical caller tree rooted at incoming references.
    /// * `outline` - Hierarchical AST document symbols for the active file.
    /// * `bookmarks` - Store containing active bookmarks for the repository.
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut Ui,
        theme: &AppTheme,
        active_tab: &mut InspectorTab,
        current_symbol: Option<&ScipSymbolInformation>,
        definition_location: Option<(&str, usize)>,
        enclosing_declaration: Option<&DocumentSymbol>,
        enclosing_line: usize,
        current_file_path: Option<&str>,
        caller_tree: &[ReferenceCallerNode],
        outline: &[DocumentSymbol],
        bookmarks: &BookmarkStore,
    ) -> Option<InspectorAction> {
        let mut action = None;

        ui.style_mut().interaction.selectable_labels = false;

        ui.vertical(|ui| {
            // Tab Selector
            ui.horizontal(|ui| {
                ui.selectable_value(active_tab, InspectorTab::Symbol, "Symbol");
                ui.selectable_value(
                    active_tab,
                    InspectorTab::References,
                    if caller_tree.is_empty() {
                        "Callers".to_string()
                    } else {
                        format!("Callers ({})", caller_tree.len())
                    },
                );
                ui.selectable_value(active_tab, InspectorTab::Outline, "Outline");
                ui.selectable_value(
                    active_tab,
                    InspectorTab::Bookmarks,
                    format!("Bookmarks ({})", bookmarks.bookmarks.len()),
                );
            });

            ui.separator();

            match active_tab {
                InspectorTab::Symbol => {
                    ScrollArea::vertical().show(ui, |ui| {
                        // Declaration Section
                        let ref_count = caller_tree.len();
                        let ref_str = if ref_count == 1 {
                            "1 reference".to_string()
                        } else {
                            format!("{ref_count} references")
                        };

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Declaration").strong());
                            ui.colored_label(theme.text_muted, ref_str);
                        });
                        ui.add_space(4.0);

                        if let Some(sym) = current_symbol {
                            ui.horizontal(|ui| {
                                ui.colored_label(theme.accent, "ƒ(x)");
                                ui.label(RichText::new(&sym.display_name).strong());
                                if let Some((def_path, def_line)) = definition_location {
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            let file_short = std::path::Path::new(def_path)
                                                .file_name()
                                                .map(|f| f.to_string_lossy().to_string())
                                                .unwrap_or_else(|| def_path.to_string());
                                            if ui.link(format!("{file_short}:{def_line}")).clicked()
                                            {
                                                action = Some(InspectorAction::JumpToLocation {
                                                    path: def_path.to_string(),
                                                    line: def_line,
                                                    byte_offset: 0,
                                                });
                                            }
                                        },
                                    );
                                }
                            });

                            if let Some((def_path, _)) = definition_location {
                                ui.colored_label(
                                    theme.text_muted,
                                    RichText::new(def_path).size(11.0),
                                );
                            }

                            if !sym.signature.is_empty() {
                                ui.add_space(4.0);
                                egui::Frame::none()
                                    .fill(theme.bg_surface)
                                    .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                                    .rounding(4.0)
                                    .show(ui, |ui| {
                                        ui.monospace(&sym.signature);
                                    });
                            }

                            if !sym.documentation.is_empty() {
                                ui.separator();
                                ui.label(RichText::new("Documentation:").strong());
                                for doc in &sym.documentation {
                                    ui.label(RichText::new(doc).color(theme.text_secondary));
                                }
                            }
                        } else if let Some(enclosing) = enclosing_declaration {
                            ui.horizontal(|ui| {
                                ui.colored_label(theme.accent, "ƒ(x)");
                                ui.label(RichText::new(&enclosing.name).strong());

                                if let Some(curr_path) = current_file_path {
                                    let file_short = std::path::Path::new(curr_path)
                                        .file_name()
                                        .map(|f| f.to_string_lossy().to_string())
                                        .unwrap_or_else(|| curr_path.to_string());
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui
                                                .link(format!("{file_short}:{enclosing_line}"))
                                                .clicked()
                                            {
                                                action = Some(InspectorAction::JumpToLocation {
                                                    path: curr_path.to_string(),
                                                    line: enclosing_line,
                                                    byte_offset: 0,
                                                });
                                            }
                                        },
                                    );
                                }
                            });

                            if let Some(detail) = &enclosing.detail {
                                ui.add_space(4.0);
                                egui::Frame::none()
                                    .fill(theme.bg_surface)
                                    .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                                    .rounding(4.0)
                                    .show(ui, |ui| {
                                        ui.monospace(detail);
                                    });
                            }
                        } else {
                            ui.colored_label(
                                theme.text_muted,
                                "Place caret inside a declaration or symbol to inspect.",
                            );
                        }

                        // Outline Section below (Swift VSplitView parity)
                        if !outline.is_empty() {
                            ui.add_space(8.0);
                            ui.separator();
                            ui.label(RichText::new("Outline").strong());
                            ui.add_space(4.0);

                            let active_name = current_symbol
                                .map(|s| s.display_name.as_str())
                                .or_else(|| enclosing_declaration.map(|s| s.name.as_str()));
                            for sym in outline {
                                if let Some(act) =
                                    Self::render_outline_node(ui, theme, sym, active_name, 0)
                                {
                                    action = Some(act);
                                }
                            }
                        }
                    });
                }
                InspectorTab::References => {
                    ScrollArea::vertical().show(ui, |ui| {
                        if caller_tree.is_empty() {
                            ui.colored_label(
                                theme.text_muted,
                                "No callers or incoming references found for active symbol.",
                            );
                        } else {
                            for node in caller_tree {
                                if let Some(act) = Self::render_caller_node(ui, theme, node, 0) {
                                    action = Some(act);
                                }
                            }
                        }
                    });
                }
                InspectorTab::Outline => {
                    ScrollArea::vertical().show(ui, |ui| {
                        let active_name = current_symbol.map(|s| s.display_name.as_str());
                        if outline.is_empty() {
                            ui.colored_label(theme.text_muted, "No document symbols in this file.");
                        } else {
                            for sym in outline {
                                if let Some(act) =
                                    Self::render_outline_node(ui, theme, sym, active_name, 0)
                                {
                                    action = Some(act);
                                }
                            }
                        }
                    });
                }
                InspectorTab::Bookmarks => {
                    ScrollArea::vertical().show(ui, |ui| {
                        if bookmarks.bookmarks.is_empty() {
                            ui.colored_label(
                                theme.text_muted,
                                "No bookmarks set. Press Ctrl+D to bookmark the caret line.",
                            );
                        } else {
                            for (idx, b) in bookmarks.bookmarks.iter().enumerate() {
                                let row_id = ui.make_persistent_id(b.id);
                                let is_hovered = ui.ctx().animate_bool(row_id, false);
                                let bg = if is_hovered > 0.0 {
                                    theme.bg_active_line
                                } else {
                                    Color32::TRANSPARENT
                                };

                                let resp = egui::Frame::none()
                                    .fill(bg)
                                    .inner_margin(egui::Margin::symmetric(6.0, 4.0))
                                    .rounding(3.0)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            if idx < 10 {
                                                ui.colored_label(theme.accent, format!("[⌘{idx}]"));
                                            }
                                            ui.label(RichText::new(&b.label).strong());

                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if Icon::Close
                                                        .button(ui, 10.0, theme.text_muted)
                                                        .on_hover_text("Delete bookmark")
                                                        .clicked()
                                                    {
                                                        action = Some(
                                                            InspectorAction::RemoveBookmark(b.id),
                                                        );
                                                    }
                                                    if idx + 1 < bookmarks.bookmarks.len()
                                                        && Icon::ArrowDown
                                                            .button(ui, 10.0, theme.text_muted)
                                                            .on_hover_text("Move down")
                                                            .clicked()
                                                    {
                                                        action = Some(
                                                            InspectorAction::MoveBookmarkDown(idx),
                                                        );
                                                    }
                                                    if idx > 0
                                                        && Icon::ArrowUp
                                                            .button(ui, 10.0, theme.text_muted)
                                                            .on_hover_text("Move up")
                                                            .clicked()
                                                    {
                                                        action = Some(
                                                            InspectorAction::MoveBookmarkUp(idx),
                                                        );
                                                    }
                                                },
                                            );
                                        });

                                        if let Some(anchor) = &b.anchor {
                                            ui.colored_label(
                                                theme.text_secondary,
                                                RichText::new(anchor).monospace(),
                                            );
                                        }
                                    })
                                    .response;

                                let row_click = resp.interact(egui::Sense::click());
                                if row_click.hovered() {
                                    ui.ctx().animate_bool(row_id, true);
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                row_click.context_menu(|ui| {
                                    if ui.button("Go to Bookmark").clicked() {
                                        action = Some(InspectorAction::JumpToLocation {
                                            path: b.path.clone(),
                                            line: b.line,
                                            byte_offset: b.byte_offset,
                                        });
                                        ui.close_menu();
                                    }
                                    if idx > 0 && ui.button("Move Up").clicked() {
                                        action = Some(InspectorAction::MoveBookmarkUp(idx));
                                        ui.close_menu();
                                    }
                                    if idx + 1 < bookmarks.bookmarks.len()
                                        && ui.button("Move Down").clicked()
                                    {
                                        action = Some(InspectorAction::MoveBookmarkDown(idx));
                                        ui.close_menu();
                                    }
                                    ui.separator();
                                    if ui.button("Remove Bookmark").clicked() {
                                        action = Some(InspectorAction::RemoveBookmark(b.id));
                                        ui.close_menu();
                                    }
                                });
                                if action.is_none() && row_click.clicked() {
                                    action = Some(InspectorAction::JumpToLocation {
                                        path: b.path.clone(),
                                        line: b.line,
                                        byte_offset: b.byte_offset,
                                    });
                                }
                            }
                        }
                    });
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
            .stroke(egui::Stroke::new(1.0, pill_color))
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
            action = Some(InspectorAction::JumpToLocation {
                path: node.path.clone(),
                line: node.line,
                byte_offset: 0,
            });
        }

        for child in &node.children {
            if let Some(act) = Self::render_caller_node(ui, theme, child, indent + 1) {
                action = Some(act);
            }
        }

        action
    }

    /// Renders a document symbol node and its children in the outline tree.
    fn render_outline_node(
        ui: &mut Ui,
        theme: &AppTheme,
        sym: &DocumentSymbol,
        active_name: Option<&str>,
        indent: usize,
    ) -> Option<InspectorAction> {
        let mut action = None;
        let indent_px = indent as f32 * 14.0;
        let is_active = active_name == Some(sym.name.as_str());

        let row_height = 22.0;
        let (rect, resp) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), row_height),
            egui::Sense::click(),
        );

        let is_hovered = resp.hovered();
        if is_hovered {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let bg = if is_active {
            if theme.is_dark {
                Color32::from_rgba_unmultiplied(255, 255, 255, 22)
            } else {
                Color32::from_rgb(238, 238, 240)
            }
        } else if is_hovered {
            if theme.is_dark {
                Color32::from_rgba_unmultiplied(255, 255, 255, 12)
            } else {
                Color32::from_rgba_unmultiplied(0, 0, 0, 10)
            }
        } else {
            Color32::TRANSPARENT
        };

        if bg != Color32::TRANSPARENT {
            ui.painter().rect_filled(rect, 3.0, bg);
        }

        if is_active {
            ui.painter().rect_filled(
                egui::Rect::from_min_size(rect.min, Vec2::new(2.0, rect.height())),
                1.0,
                theme.accent,
            );
        }

        let mut child_ui = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
        child_ui.add_space(indent_px + 4.0);
        child_ui.colored_label(theme.accent, "ƒ(x)");
        child_ui.label(RichText::new(&sym.name).strong());

        let span = sym.range.end.saturating_sub(sym.range.start) / 30;
        child_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(6.0);
            ui.colored_label(theme.text_muted, format!("{} L", span.max(1)));
        });

        if resp.clicked() {
            action = Some(InspectorAction::JumpToSymbol(sym.selection_range.start));
        }

        for child in &sym.children {
            if let Some(act) = Self::render_outline_node(ui, theme, child, active_name, indent + 1)
            {
                action = Some(act);
            }
        }

        action
    }
}
