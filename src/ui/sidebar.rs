//! Left sidebar pane containing repository file trees, pull request changes, and search results.
//!
//! Provides tabs for navigating the hierarchical file tree, viewing pull request modified
//! files with Git status badges, and performing ripgrep-powered repository-wide text searches.

use crate::pull_request::change::FileChange;
use crate::repository::file_node::FileNode;
use crate::search::match_item::SearchMatch;
use crate::search::ripgrep::SearchOptions;
use crate::theme::AppTheme;
use egui::{Color32, RichText, ScrollArea, TextEdit, Ui};
use std::collections::HashSet;

/// Active tab shown in the left [`Sidebar`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarTab {
    /// Full repository directory tree view.
    #[default]
    Files,
    /// List of modified files from the active pull request diff.
    Changes,
    /// Streaming ripgrep search results.
    Search,
}

/// User interactions and navigation events produced by the [`Sidebar`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidebarAction {
    /// Requests opening the file at the specified repository-relative path.
    OpenFile(String),
    /// Requests opening a file and navigating to the specific line/column of a search hit.
    OpenSearchMatch(SearchMatch),
    /// Triggers execution of a text search with current query and options.
    StartSearch,
    /// Clears active search query and results list.
    ClearSearch,
}

/// Renders the collapsible left sidebar panel in the Radiology desktop UI.
pub struct Sidebar;

impl Sidebar {
    /// Renders the sidebar tabs, file tree, changed files, and ripgrep search controls.
    ///
    /// # Arguments
    ///
    /// * `ui` - EGUI UI layout context.
    /// * `theme` - Visual color palette.
    /// * `active_tab` - Mutable reference to the currently selected sidebar tab.
    /// * `file_tree` - Hierarchical file tree nodes discovered in the repository.
    /// * `expanded_dirs` - Set of directory paths currently expanded by the user.
    /// * `selected_path` - Path of the currently focused source file, if any.
    /// * `pr_changes` - List of modified files in the active pull request review.
    /// * `search_options` - Mutable search query and regex/case options.
    /// * `search_matches` - Matches found from previous search execution.
    /// * `is_searching` - Whether a ripgrep background process is currently streaming hits.
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut Ui,
        theme: &AppTheme,
        active_tab: &mut SidebarTab,
        file_tree: &[FileNode],
        expanded_dirs: &mut HashSet<String>,
        selected_path: Option<&str>,
        pr_changes: &[FileChange],
        search_options: &mut SearchOptions,
        search_matches: &[SearchMatch],
        is_searching: bool,
    ) -> Option<SidebarAction> {
        let mut action = None;

        ui.vertical(|ui| {
            // Tab Selector
            ui.horizontal(|ui| {
                ui.selectable_value(active_tab, SidebarTab::Files, "Files");
                if !pr_changes.is_empty() {
                    ui.selectable_value(
                        active_tab,
                        SidebarTab::Changes,
                        format!("Changes ({})", pr_changes.len()),
                    );
                }
                ui.selectable_value(
                    active_tab,
                    SidebarTab::Search,
                    if search_matches.is_empty() {
                        "Search".to_string()
                    } else {
                        format!("Search ({})", search_matches.len())
                    },
                );
            });

            ui.separator();

            match active_tab {
                SidebarTab::Files => {
                    ScrollArea::vertical().show(ui, |ui| {
                        for node in file_tree {
                            if let Some(act) = Self::render_tree_node(
                                ui,
                                theme,
                                node,
                                expanded_dirs,
                                selected_path,
                                0,
                            ) {
                                action = Some(act);
                            }
                        }
                    });
                }
                SidebarTab::Changes => {
                    ScrollArea::vertical().show(ui, |ui| {
                        for change in pr_changes {
                            let is_selected = selected_path == Some(&change.path);
                            let bg = if is_selected {
                                theme.accent.linear_multiply(0.25)
                            } else {
                                Color32::TRANSPARENT
                            };

                            let resp = egui::Frame::none()
                                .fill(bg)
                                .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                                .rounding(3.0)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let (status_color, badge) = match change.status {
                                            crate::pull_request::change::FileChangeStatus::Added => {
                                                (Color32::from_rgb(34, 197, 94), "A")
                                            }
                                            crate::pull_request::change::FileChangeStatus::Modified => {
                                                (Color32::from_rgb(234, 179, 8), "M")
                                            }
                                            crate::pull_request::change::FileChangeStatus::Deleted => {
                                                (Color32::from_rgb(239, 68, 68), "D")
                                            }
                                            _ => (theme.text_muted, "?"),
                                        };

                                        ui.colored_label(status_color, badge);
                                        ui.label(RichText::new(&change.path).monospace());

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if change.deletions > 0 {
                                                    ui.colored_label(
                                                        Color32::from_rgb(239, 68, 68),
                                                        format!("-{}", change.deletions),
                                                    );
                                                }
                                                if change.additions > 0 {
                                                    ui.colored_label(
                                                        Color32::from_rgb(34, 197, 94),
                                                        format!("+{}", change.additions),
                                                    );
                                                }
                                            },
                                        );
                                    });
                                })
                                .response;

                            if resp.interact(egui::Sense::click()).clicked() {
                                action = Some(SidebarAction::OpenFile(change.path.clone()));
                            }
                        }
                    });
                }
                SidebarTab::Search => {
                    ui.horizontal(|ui| {
                        let edit = TextEdit::singleline(&mut search_options.query)
                            .hint_text("Search repository (ripgrep)...")
                            .desired_width(ui.available_width() - 60.0);
                        let resp = ui.add(edit);
                        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            action = Some(SidebarAction::StartSearch);
                        }

                        if ui.button(if is_searching { "..." } else { "Go" }).clicked() {
                            action = Some(SidebarAction::StartSearch);
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut search_options.case_sensitive, "Case");
                        ui.checkbox(&mut search_options.is_regex, "Regex");
                    });

                    ui.separator();

                    ScrollArea::vertical().show(ui, |ui| {
                        for m in search_matches {
                            let resp = egui::Frame::none()
                                .inner_margin(egui::Margin::symmetric(6.0, 4.0))
                                .rounding(3.0)
                                .show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.colored_label(
                                                theme.accent,
                                                RichText::new(format!("{}:{}", m.path, m.line))
                                                    .monospace()
                                                    .strong(),
                                            );
                                        });
                                        ui.label(
                                            RichText::new(&m.preview)
                                                .monospace()
                                                .color(theme.text_secondary),
                                        );
                                    });
                                })
                                .response;

                            if resp.interact(egui::Sense::click()).clicked() {
                                action = Some(SidebarAction::OpenSearchMatch(m.clone()));
                            }
                        }
                    });
                }
            }
        });

        action
    }

    /// Recursively renders a single tree node (directory or file).
    fn render_tree_node(
        ui: &mut Ui,
        theme: &AppTheme,
        node: &FileNode,
        expanded_dirs: &mut HashSet<String>,
        selected_path: Option<&str>,
        indent_level: usize,
    ) -> Option<SidebarAction> {
        let mut action = None;
        let indent = indent_level as f32 * 14.0;

        if node.is_directory {
            let is_expanded = expanded_dirs.contains(&node.path);
            let icon = if is_expanded { "▼" } else { "▶" };

            ui.horizontal(|ui| {
                ui.add_space(indent);
                let btn = ui.selectable_label(
                    false,
                    RichText::new(format!("{icon} {}", node.name)).strong(),
                );
                if btn.clicked() {
                    if is_expanded {
                        expanded_dirs.remove(&node.path);
                    } else {
                        expanded_dirs.insert(node.path.clone());
                    }
                }
            });

            if is_expanded {
                for child in &node.children {
                    if let Some(act) = Self::render_tree_node(
                        ui,
                        theme,
                        child,
                        expanded_dirs,
                        selected_path,
                        indent_level + 1,
                    ) {
                        action = Some(act);
                    }
                }
            }
        } else {
            let is_selected = selected_path == Some(&node.path);
            let bg = if is_selected {
                theme.accent.linear_multiply(0.25)
            } else {
                Color32::TRANSPARENT
            };

            let resp = egui::Frame::none()
                .fill(bg)
                .inner_margin(egui::Margin::symmetric(4.0, 2.0))
                .rounding(3.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(indent);
                        ui.label(RichText::new(&node.name).monospace());
                    });
                })
                .response;

            if resp.interact(egui::Sense::click()).clicked() {
                action = Some(SidebarAction::OpenFile(node.path.clone()));
            }
        }

        action
    }
}
