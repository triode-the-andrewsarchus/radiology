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
use egui::{Color32, ScrollArea, TextEdit, Ui};
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
        let heading = super::text_fit::lines(
            ui,
            &title,
            egui::TextStyle::Body.resolve(ui.style()),
            theme.text_primary,
            ui.available_width(),
            2,
        );
        ui.add(egui::Label::new(heading)).on_hover_text(title);
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
        let counts = if change.is_binary {
            "binary".into()
        } else {
            format!("+{} −{}", change.additions, change.deletions)
        };
        let detail = change
            .old_path
            .as_ref()
            .filter(|old| *old != &change.path)
            .map_or_else(
                || {
                    std::path::Path::new(&change.path)
                        .parent()
                        .filter(|path| !path.as_os_str().is_empty())
                        .map_or_else(|| "/".into(), |path| path.display().to_string())
                },
                |old| format!("from {old}"),
            );
        let response = egui::Frame::none()
            .inner_margin(6.0)
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.colored_label(color, change.status.badge_letter());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        let meta = egui::TextStyle::Small.resolve(ui.style());
                        let counts = super::text_fit::middle(
                            ui,
                            &counts,
                            meta.clone(),
                            theme.text_secondary,
                            (ui.available_width() * 0.45).min(110.0),
                        );
                        ui.add(egui::Label::new(counts));
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 2.0;
                            let name = std::path::Path::new(&change.path)
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy();
                            let name = super::text_fit::middle(
                                ui,
                                &name,
                                egui::TextStyle::Body.resolve(ui.style()),
                                theme.text_primary,
                                ui.available_width(),
                            );
                            ui.add(egui::Label::new(name));
                            let path = super::text_fit::middle(
                                ui,
                                &detail,
                                meta,
                                theme.text_muted,
                                ui.available_width(),
                            );
                            ui.add(egui::Label::new(path));
                        });
                    });
                });
            })
            .response;
        let response = ui
            .interact(
                response.rect,
                ui.make_persistent_id(("change_row", &change.path)),
                egui::Sense::click(),
            )
            .on_hover_text(format!(
                "{}\n{}\n{}\n{}",
                change.status.title(),
                change.path,
                detail,
                counts
            ));
        if selected == Some(change.path.as_str()) || response.hovered() {
            ui.painter()
                .rect_stroke(response.rect, 3.0, egui::Stroke::new(1.0_f32, theme.border));
        }
        Self::row_activated(&response)
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
        let meta = egui::TextStyle::Small.resolve(ui.style());
        let preview = egui::FontId::monospace(12.0);
        let row_height =
            ui.fonts(|fonts| fonts.row_height(&meta) + 2.0 * fonts.row_height(&preview)) + 8.0;
        ScrollArea::vertical()
            .id_source("repository_search")
            .show_rows(ui, row_height, search.matches.len(), |ui, rows| {
                for index in rows {
                    let hit = &search.matches[index];
                    if Self::search_row(ui, theme, hit, row_height, &meta, &preview) {
                        action = Some(SidebarAction::OpenSearchMatch(hit.clone()));
                    }
                }
            });
        action
    }

    fn search_row(
        ui: &mut Ui,
        theme: &AppTheme,
        hit: &SearchMatch,
        height: f32,
        meta: &egui::FontId,
        preview: &egui::FontId,
    ) -> bool {
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), height),
            egui::Sense::hover(),
        );
        let response = ui.interact(
            rect,
            ui.make_persistent_id(("search_match", hit.id())),
            egui::Sense::click(),
        );
        if response.hovered() || response.has_focus() {
            ui.painter()
                .rect_filled(rect, 3.0, ui.visuals().selection.bg_fill);
        }
        if response.has_focus() {
            ui.painter()
                .rect_stroke(rect, 3.0, ui.visuals().selection.stroke);
        }
        let content = rect.shrink2(egui::vec2(4.0, 3.0));
        let path = super::text_fit::middle(
            ui,
            &format!("{}:{}", hit.path, hit.line),
            meta.clone(),
            theme.text_secondary,
            content.width(),
        );
        let preview = super::text_fit::lines(
            ui,
            hit.preview.trim(),
            preview.clone(),
            theme.text_primary,
            content.width(),
            2,
        );
        let preview_y = content.top() + ui.fonts(|fonts| fonts.row_height(meta)) + 2.0;
        let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
        painter.galley(content.min, path, theme.text_secondary);
        painter.galley(
            egui::pos2(content.left(), preview_y),
            preview,
            theme.text_primary,
        );
        let response = response.on_hover_text(format!("{}:{}:{}", hit.path, hit.line, hit.column));
        Self::row_activated(&response)
    }

    fn row_activated(response: &egui::Response) -> bool {
        // A result can move after egui's hit test; validate the live rectangle.
        response.clicked()
            && (!response.clicked_by(egui::PointerButton::Primary)
                || response
                    .interact_pointer_pos()
                    .is_some_and(|position| response.interact_rect.contains(position)))
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
                        ui.add_space(indent.min((ui.available_width() - 62.0).max(0.0)));
                        chevron.ui(ui, 10.0, theme.text_muted);
                        Icon::Folder.ui(ui, 12.0, theme.accent);
                        ui.add_space(2.0);
                        let label = super::text_fit::middle(
                            ui,
                            &node.name,
                            egui::TextStyle::Body.resolve(ui.style()),
                            theme.text_primary,
                            ui.available_width(),
                        );
                        ui.add(egui::Label::new(label)).on_hover_text(&node.path);
                    });
                })
                .response;

            let response = resp
                .interact(egui::Sense::click())
                .on_hover_text(&node.path);
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
                        ui.add_space((indent + 12.0).min((ui.available_width() - 54.0).max(0.0)));
                        Icon::Code.ui(ui, 11.0, theme.text_muted);
                        ui.add_space(2.0);
                        let width =
                            (ui.available_width() - 6.0 - ui.spacing().item_spacing.x).max(0.0);
                        let label = super::text_fit::middle(
                            ui,
                            &node.name,
                            egui::TextStyle::Monospace.resolve(ui.style()),
                            theme.text_primary,
                            width,
                        );
                        ui.add(egui::Label::new(label)).on_hover_text(&node.path);
                        let (name_rect, _) =
                            ui.allocate_exact_size(egui::vec2(6.0, 6.0), egui::Sense::hover());
                        if is_changed {
                            ui.painter()
                                .circle_filled(name_rect.center(), 2.5, theme.accent);
                        }
                    });
                })
                .response;

            let interact_resp = resp
                .interact(egui::Sense::click())
                .on_hover_text(&node.path);
            action = Self::row_menu(&interact_resp, &node.path);

            if interact_resp.clicked() {
                action = Some(SidebarAction::OpenFile(node.path.clone()));
            }
        }

        action
    }
    fn row_menu(response: &egui::Response, path: &str) -> Option<SidebarAction> {
        #[derive(Clone)]
        enum FileMenuAction {
            Reveal,
            CopyPath,
        }
        use crate::ui::commands::MenuItem;
        match crate::ui::commands::context_menu(response, path, || {
            vec![
                MenuItem::action("Reveal in File Manager", FileMenuAction::Reveal),
                MenuItem::action("Copy Relative Path", FileMenuAction::CopyPath),
            ]
        }) {
            Some(FileMenuAction::Reveal) => {
                Some(SidebarAction::RevealInFileManager(path.to_owned()))
            }
            Some(FileMenuAction::CopyPath) => {
                response
                    .ctx
                    .output_mut(|output| output.copied_text = path.to_owned());
                None
            }
            None => None,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    mod results;

    #[test]
    fn long_and_deep_file_rows_do_not_expand_the_sidebar() {
        use crate::repository::file_node::RepositoryFile;
        let path = format!(
            "{}/a-very-long-source-filename-🦀-café-and-minified-long-lines.rs",
            "deep-directory-name".repeat(4)
        );
        let deep = format!("{}/tail.rs", vec!["nested"; 25].join("/"));
        let tree = FileNode::build_tree(&[RepositoryFile::new(&path), RepositoryFile::new(&deep)]);
        let mut expanded: HashSet<String> = [&path, &deep]
            .into_iter()
            .flat_map(|path| path.match_indices('/').map(|(at, _)| path[..at].to_owned()))
            .collect();
        for width in [180.0, 260.0, 420.0] {
            let measure = |nodes: &[FileNode], expanded: &mut HashSet<String>| {
                let ctx = egui::Context::default();
                let mut measured = 0.0;
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1280.0, 900.0),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        measured = egui::SidePanel::left("files")
                            .default_width(width)
                            .min_width(180.0)
                            .max_width(500.0)
                            .show(ctx, |ui| {
                                for node in nodes {
                                    Sidebar::render_tree_node(
                                        ui,
                                        &AppTheme::dark(),
                                        node,
                                        expanded,
                                        Some(&path),
                                        &HashSet::new(),
                                        0,
                                    );
                                }
                            })
                            .response
                            .rect
                            .width();
                    },
                );
                measured
            };
            // egui's panel minimum includes its frame margins. Compare to the
            // configured width or empty-panel minimum, whichever is larger.
            let expected = width.max(measure(&[], &mut HashSet::new()));
            let actual = measure(&tree, &mut expanded);
            assert!(
                actual <= expected + 1.0,
                "file labels expanded {expected} to {actual}"
            );
        }
    }
    #[test]
    fn elided_file_rows_keep_full_open_and_context_copy_targets() {
        use crate::repository::file_node::RepositoryFile;
        use crate::ui::test_support::{click, painted_frame, secondary_click};
        let ctx = egui::Context::default();
        let path = format!(
            "a-very-long-🦀-café-source-filename-{}.rs",
            "more-context-".repeat(6)
        );
        let tree = FileNode::build_tree(&[RepositoryFile::new(&path)]);
        let frame = |events| {
            let mut action = None;
            let output = painted_frame(&ctx, egui::vec2(220.0, 300.0), events, |ui| {
                ui.style_mut().interaction.selectable_labels = false;
                action = Sidebar::render_tree_node(
                    ui,
                    &AppTheme::dark(),
                    &tree[0],
                    &mut HashSet::new(),
                    Some(&path),
                    &HashSet::new(),
                    0,
                );
            });
            (output, action)
        };
        let (output, _) = frame(vec![]);
        let point = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.job.text.contains('…') => {
                    assert!(text.galley.job.text.ends_with(".rs"));
                    Some(text.pos + text.galley.size() / 2.0)
                }
                _ => None,
            })
            .expect("middle-elided file label");
        assert_eq!(
            frame(click(point)).1,
            Some(SidebarAction::OpenFile(path.clone()))
        );
        frame(secondary_click(point));
        let (output, action) = frame(vec![
            crate::ui::commands::test_key(egui::Key::End),
            crate::ui::commands::test_key(egui::Key::Enter),
        ]);
        assert_eq!(action, None);
        assert_eq!(output.platform_output.copied_text, path);
    }
}
