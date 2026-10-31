//! Left sidebar pane containing repository file trees, pull request changes, and search results.
//!
//! Provides tabs for navigating the hierarchical file tree, viewing pull request modified
//! files with Git status badges, and performing ripgrep-powered repository-wide text searches.

use crate::pull_request::change::FileChange;
use crate::pull_request::change::ReviewChangesPresentation;
use crate::pull_request::github::PullRequestSummary;
use crate::repository::file_node::FileNode;
use crate::repository::filters::SourceFilters;
use crate::search::match_item::SearchMatch;
use crate::search::{RepositorySearch, SearchOptions};
use crate::theme::AppTheme;
use crate::ui::icons::Icon;
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
    /// Selects a Files row in the desktop file manager without opening its content.
    RevealInFileManager(String),
    /// Requests opening a file and navigating to the specific line/column of a search hit.
    OpenSearchMatch(SearchMatch),
    /// Triggers execution of a text search with current query and options.
    StartSearch,
    /// Clears active search query and results list.
    ClearSearch,
    /// Stops the current search while keeping completed matches.
    StopSearch,
    /// Rebuilds the Files tree after its filter query changes.
    FiltersChanged,
    /// Resets the source/test/path filters when they exclude all rows.
    ShowAllFiles,
}

/// Renders the collapsible left sidebar panel in the Radiology desktop UI.
pub struct Sidebar;

impl Sidebar {
    /// Renders navigation modes with independent query and search controls.
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut Ui,
        theme: &AppTheme,
        active_tab: &mut SidebarTab,
        file_tree: &[FileNode],
        filters: &mut SourceFilters,
        expanded_dirs: &mut HashSet<String>,
        selected_path: Option<&str>,
        active_pr: Option<&PullRequestSummary>,
        all_changes: &[FileChange],
        search_options: &mut SearchOptions,
        search: &RepositorySearch,
        visible_count: usize,
        total_count: usize,
        branch: Option<&str>,
    ) -> Option<SidebarAction> {
        let mut action = None;
        let mut change_filters = filters.clone();
        change_filters.path_query.clear();
        let changes = ReviewChangesPresentation::new(all_changes, &change_filters);
        ui.horizontal(|ui| {
            for (tab, title) in [
                (SidebarTab::Files, "Files"),
                (SidebarTab::Changes, "Changes"),
                (SidebarTab::Search, "Search"),
            ] {
                ui.selectable_value(active_tab, tab, title);
            }
        });
        ui.separator();
        let count = match active_tab {
            SidebarTab::Changes => changes.file_count_label(),
            SidebarTab::Search => format!("{} matches", search.matches.len()),
            SidebarTab::Files => format!("{visible_count} of {total_count} files"),
        };
        egui::TopBottomPanel::bottom("repository_footer").show_inside(ui, |ui| {
            ui.colored_label(theme.text_secondary, count);
        });
        match active_tab {
            SidebarTab::Files => {
                let changed: HashSet<&str> = all_changes
                    .iter()
                    .map(|change| change.path.as_str())
                    .collect();
                if ui
                    .add(TextEdit::singleline(&mut filters.path_query).hint_text("Filter files"))
                    .changed()
                {
                    action = Some(SidebarAction::FiltersChanged);
                }
                if selected_path.is_some_and(|path| !filters.includes_path(path)) {
                    ui.colored_label(
                        theme.text_muted,
                        "The open file is excluded by these filters.",
                    );
                }
                if file_tree.is_empty() && total_count > 0 {
                    action = Self::filtered_empty(ui, theme);
                }
                ScrollArea::vertical()
                    .id_source("repository_files")
                    .show(ui, |ui| {
                        for node in file_tree {
                            if let Some(next) = Self::render_tree_node(
                                ui,
                                theme,
                                node,
                                expanded_dirs,
                                selected_path,
                                &changed,
                                0,
                            ) {
                                action = Some(next);
                            }
                        }
                    });
            }
            SidebarTab::Changes => {
                action = Self::changes(ui, theme, active_pr, &changes, selected_path, branch);
            }
            SidebarTab::Search => {
                action = Self::search(ui, theme, search_options, search);
            }
        }
        action
    }

    fn filtered_empty(ui: &mut Ui, theme: &AppTheme) -> Option<SidebarAction> {
        ui.colored_label(theme.text_muted, "No files match the current filters.");
        ui.button("Show All Files")
            .clicked()
            .then_some(SidebarAction::ShowAllFiles)
    }

    fn changes(
        ui: &mut Ui,
        theme: &AppTheme,
        pr: Option<&PullRequestSummary>,
        presentation: &ReviewChangesPresentation,
        selected: Option<&str>,
        branch: Option<&str>,
    ) -> Option<SidebarAction> {
        let mut action = None;
        let title = pr.map_or_else(
            || branch.unwrap_or("Working Tree").to_string(),
            |pr| format!("#{} · {}", pr.number, pr.title),
        );
        ui.label(RichText::new(title).strong());
        ui.horizontal(|ui| {
            ui.colored_label(
                Color32::from_rgb(34, 197, 94),
                format!("+{}", presentation.additions()),
            );
            ui.colored_label(
                Color32::from_rgb(239, 68, 68),
                format!("−{}", presentation.deletions()),
            );
        });
        ui.separator();
        if presentation.is_filtered_empty() {
            return Self::filtered_empty(ui, theme);
        }
        if presentation.is_genuinely_empty() {
            ui.colored_label(theme.text_muted, "No changed files.");
        }
        ScrollArea::vertical()
            .id_source("repository_changes")
            .show(ui, |ui| {
                for change in &presentation.changes {
                    if Self::change_row(ui, theme, change, selected) {
                        action = Some(SidebarAction::OpenFile(change.path.clone()));
                    }
                }
            });
        action
    }

    fn change_row(
        ui: &mut Ui,
        theme: &AppTheme,
        change: &FileChange,
        selected: Option<&str>,
    ) -> bool {
        use crate::pull_request::change::FileChangeStatus;
        let color = match change.status {
            FileChangeStatus::Added => Color32::from_rgb(34, 197, 94),
            FileChangeStatus::Deleted => Color32::from_rgb(239, 68, 68),
            FileChangeStatus::Modified => Color32::from_rgb(234, 179, 8),
            _ => theme.accent,
        };
        let response = egui::Frame::none()
            .inner_margin(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(color, change.status.badge_letter());
                    ui.label(
                        RichText::new(
                            std::path::Path::new(&change.path)
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy(),
                        )
                        .strong(),
                    );
                    ui.colored_label(
                        theme.text_secondary,
                        format!("+{} −{}", change.additions, change.deletions),
                    );
                });
                let path = change.old_path.as_ref().map_or_else(
                    || change.path.clone(),
                    |old| format!("{old} → {}", change.path),
                );
                ui.colored_label(theme.text_muted, path);
            })
            .response;
        let response = response.interact(egui::Sense::click());
        if selected == Some(change.path.as_str()) || response.hovered() {
            ui.painter()
                .rect_stroke(response.rect, 3.0, egui::Stroke::new(1.0_f32, theme.border));
        }
        response.clicked()
    }

    fn search(
        ui: &mut Ui,
        theme: &AppTheme,
        options: &mut SearchOptions,
        search: &RepositorySearch,
    ) -> Option<SidebarAction> {
        let mut action = None;
        let response =
            ui.add(TextEdit::singleline(&mut options.query).hint_text("Search repository"));
        if response.lost_focus()
            && ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Enter))
        {
            action = Some(SidebarAction::StartSearch);
        }
        ui.horizontal(|ui| {
            ui.checkbox(&mut options.case_sensitive, "Case");
            ui.checkbox(&mut options.is_regex, "Regex");
            if ui.button("Go").clicked() {
                action = Some(SidebarAction::StartSearch);
            }
            if search.running && ui.button("Stop").clicked() {
                action = Some(SidebarAction::StopSearch);
            }
        });
        ui.add(
            TextEdit::singleline(&mut options.include_glob).hint_text("Include glob, e.g. *.rs"),
        );
        ui.add(
            TextEdit::singleline(&mut options.exclude_glob)
                .hint_text("Exclude glob, e.g. tests/**"),
        );
        if let Some(error) = &search.error {
            ui.colored_label(Color32::from_rgb(220, 90, 60), error);
        }
        if search.limited {
            ui.label("First 5,000 matches. Narrow the query to see more.");
        }
        ui.separator();
        ScrollArea::vertical()
            .id_source("repository_search")
            .show_rows(ui, 42.0, search.matches.len(), |ui, rows| {
                for index in rows {
                    let hit = &search.matches[index];
                    let preview: String = hit.preview.trim().chars().take(180).collect();
                    let label = format!("{}:{}\n{preview}", hit.path, hit.line);
                    if ui
                        .add_sized(
                            [ui.available_width(), 42.0],
                            egui::Button::new(RichText::new(label).color(theme.text_secondary))
                                .frame(false),
                        )
                        .clicked()
                    {
                        action = Some(SidebarAction::OpenSearchMatch(hit.clone()));
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
        changed_paths: &HashSet<&str>,
        indent_level: usize,
    ) -> Option<SidebarAction> {
        let mut action;
        let indent = indent_level as f32 * 14.0;

        if node.is_directory {
            let is_expanded = expanded_dirs.contains(&node.path);
            let chevron = if is_expanded {
                Icon::ChevronDown
            } else {
                Icon::ChevronRight
            };

            let resp = egui::Frame::none()
                .inner_margin(egui::Margin::symmetric(4.0, 2.0))
                .rounding(3.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(indent);
                        chevron.ui(ui, 10.0, theme.text_muted);
                        Icon::Folder.ui(ui, 12.0, theme.accent);
                        ui.add_space(2.0);
                        ui.label(RichText::new(&node.name).strong());
                    });
                })
                .response;

            let response = resp.interact(egui::Sense::click());
            action = Self::row_menu(&response, &node.path);
            if response.clicked() {
                if is_expanded {
                    expanded_dirs.remove(&node.path);
                } else {
                    expanded_dirs.insert(node.path.clone());
                }
            }

            if is_expanded {
                for child in &node.children {
                    if let Some(act) = Self::render_tree_node(
                        ui,
                        theme,
                        child,
                        expanded_dirs,
                        selected_path,
                        changed_paths,
                        indent_level + 1,
                    ) {
                        action = Some(act);
                    }
                }
            }
        } else {
            let is_selected = selected_path == Some(&node.path);
            let is_changed = changed_paths.contains(node.path.as_str());
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
                        ui.add_space(indent + 12.0);
                        Icon::Code.ui(ui, 11.0, theme.text_muted);
                        ui.add_space(2.0);
                        ui.label(RichText::new(&node.name).monospace());
                        let (name_rect, _) =
                            ui.allocate_exact_size(egui::vec2(6.0, 6.0), egui::Sense::hover());
                        if is_changed {
                            ui.painter()
                                .circle_filled(name_rect.center(), 2.5, theme.accent);
                        }
                    });
                })
                .response;

            let interact_resp = resp.interact(egui::Sense::click());
            action = Self::row_menu(&interact_resp, &node.path);

            if interact_resp.clicked() {
                action = Some(SidebarAction::OpenFile(node.path.clone()));
            }
        }

        action
    }
    fn row_menu(response: &egui::Response, path: &str) -> Option<SidebarAction> {
        let mut action = None;
        crate::ui::commands::context_menu(response, |ui| {
            if ui.button("Reveal in File Manager").clicked() {
                action = Some(SidebarAction::RevealInFileManager(path.to_owned()));
                ui.close_menu();
            }
            if ui.button("Copy Relative Path").clicked() {
                ui.output_mut(|output| output.copied_text = path.to_owned());
                ui.close_menu();
            }
        });
        action
    }
}
