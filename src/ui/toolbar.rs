//! Top application toolbar and quick-navigation control strip.
//!
//! Provides the primary repository status display, back/forward history navigation,
//! pull request review switcher, criticality mode selector, and quick palette shortcuts.

use crate::critical::lens::SourceMode;
use crate::repository::filters::SourceFilters;
use crate::source::history::NavigationHistory;
use crate::theme::AppTheme;
use crate::ui::icons::Icon;
use crate::ui::pull_request_menu::PullRequestMenu;
use egui::{RichText, Ui};

pub use super::commands::ReaderCommand as ToolbarAction;

/// Renders the top horizontal toolbar in the Radiology desktop interface.
pub struct Toolbar;

impl Toolbar {
    /// Renders the horizontal toolbar and returns any user-triggered action.
    ///
    /// # Arguments
    ///
    /// * `ui` - EGUI user interface context for rendering the strip.
    /// * `theme` - Active visual color palette.
    /// * `show_sidebar` - Whether the left navigation sidebar is visible.
    /// * `show_inspector` - Whether the right context inspector is visible.
    /// * `repo_name` - Optional name of the currently opened repository.
    /// * `branch_name` - Optional name of the current Git HEAD branch.
    /// * `recent_repos` - Slice of recent repository paths.
    /// * `review_menu` - Accepted review, recent choices and command availability.
    /// * `mode` - Active inspection mode (`Normal` or `Critical`).
    /// * `filters` - Repository file filter settings.
    /// * `wrap_lines` - Whether long lines wrap at viewport width.
    /// * `history` - Navigation history tracker used to enable/disable back/forward buttons.
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut Ui,
        theme: &AppTheme,
        show_sidebar: bool,
        show_inspector: bool,
        repo_name: Option<&str>,
        branch_name: Option<&str>,
        recent_repos: &[String],
        review_menu: PullRequestMenu<'_>,
        mode: SourceMode,
        filters: &SourceFilters,
        wrap_lines: bool,
        history: &NavigationHistory,
    ) -> Option<ToolbarAction> {
        let mut action = None;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;

            // Toggle Navigation Sidebar (Ctrl+Ctrl+S / Ctrl+Alt+S)
            let sidebar_btn = ui
                .selectable_label(show_sidebar, "Sidebar")
                .on_hover_text("Show or Hide Navigation Sidebar (Ctrl+Alt+S)");
            if sidebar_btn.clicked() {
                action = Some(ToolbarAction::ToggleSidebar);
            }

            ui.separator();

            // Navigation Back/Forward (macOS style chevron icons)
            ui.add_enabled_ui(history.can_go_back(), |ui| {
                if Icon::ArrowLeft
                    .button(ui, 12.0, theme.text_primary)
                    .on_hover_text("Back (Ctrl+[)")
                    .clicked()
                {
                    action = Some(ToolbarAction::NavigateBack);
                }
            });
            ui.add_enabled_ui(history.can_go_forward(), |ui| {
                if Icon::ArrowRight
                    .button(ui, 12.0, theme.text_primary)
                    .on_hover_text("Forward (Ctrl+])")
                    .clicked()
                {
                    action = Some(ToolbarAction::NavigateForward);
                }
            });

            ui.separator();

            // Repository Menu (macOS Swift RepositoryMenu parity)
            if let Some(name) = repo_name {
                ui.menu_button(RichText::new(name).strong().size(13.0), |ui| {
                    if ui.button("Open Repository… (Ctrl+O)").clicked() {
                        action = Some(ToolbarAction::OpenRepositoryDialog);
                        ui.close_menu();
                    }
                    if ui.button("Copy Repository Path").clicked() {
                        action = Some(ToolbarAction::CopyRepoPath);
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.menu_button("Open Recent", |ui| {
                        if recent_repos.is_empty() {
                            ui.colored_label(theme.text_muted, "No recent repositories");
                        } else {
                            for r in recent_repos {
                                if ui.button(r).clicked() {
                                    action = Some(ToolbarAction::OpenRecentRepo(r.clone()));
                                    ui.close_menu();
                                }
                            }
                        }
                    });
                });
                if let Some(branch) = branch_name {
                    ui.colored_label(theme.text_secondary, RichText::new(branch).size(10.5));
                }
            } else if ui.button("Open Repository… (Ctrl+O)").clicked() {
                action = Some(ToolbarAction::OpenRepositoryDialog);
            }

            ui.separator();

            if let Some(command) = review_menu.show(ui) {
                action = Some(command);
            }
            if review_menu.active.is_some() {
                // Change navigation buttons
                if Icon::ArrowUp
                    .button(ui, 11.0, theme.text_primary)
                    .on_hover_text("Previous Change Block (Alt+↑)")
                    .clicked()
                {
                    action = Some(ToolbarAction::PrevChangeBlock);
                }
                if Icon::ArrowDown
                    .button(ui, 11.0, theme.text_primary)
                    .on_hover_text("Next Change Block (Alt+↓)")
                    .clicked()
                {
                    action = Some(ToolbarAction::NextChangeBlock);
                }
            }

            ui.separator();

            // Mode Selector: Segmented Capsule [ Normal | Critical ]
            ui.horizontal(|ui| {
                let is_normal = mode == SourceMode::Normal;
                let is_critical = mode == SourceMode::Critical;
                if ui.selectable_label(is_normal, "Normal").clicked() && !is_normal {
                    action = Some(ToolbarAction::ToggleMode(SourceMode::Normal));
                }
                if ui.selectable_label(is_critical, "Critical").clicked() && !is_critical {
                    action = Some(ToolbarAction::ToggleMode(SourceMode::Critical));
                }
            });

            // View & Filters Menu
            ui.menu_button("View", |ui| {
                let theme_label = if theme.is_dark {
                    "Switch to Light Mode"
                } else {
                    "Switch to Dark Mode"
                };
                if ui.button(theme_label).clicked() {
                    action = Some(ToolbarAction::ToggleTheme);
                    ui.close_menu();
                }
                ui.separator();
                if ui
                    .checkbox(&mut { wrap_lines }, "Wrap Lines")
                    .on_hover_text(
                        "Wrap long lines at viewport edge instead of scrolling horizontally",
                    )
                    .clicked()
                {
                    action = Some(ToolbarAction::ToggleWrapLines);
                }
                ui.separator();
                if ui
                    .checkbox(&mut { filters.source_files_only }, "Source Files Only")
                    .clicked()
                {
                    action = Some(ToolbarAction::ToggleSourceFilesOnly);
                }
                if ui
                    .checkbox(&mut { filters.hides_tests }, "Hide Tests")
                    .clicked()
                {
                    action = Some(ToolbarAction::ToggleHideTests);
                }
                ui.separator();
                for (label, command) in [
                    (
                        "Larger Source Text (Ctrl++)",
                        ToolbarAction::IncreaseSourceFont,
                    ),
                    (
                        "Smaller Source Text (Ctrl+-)",
                        ToolbarAction::DecreaseSourceFont,
                    ),
                    (
                        "Reset Source Text (Ctrl+Alt+=)",
                        ToolbarAction::ResetSourceFont,
                    ),
                ] {
                    if ui.button(label).clicked() {
                        action = Some(command);
                        ui.close_menu();
                    }
                }
                ui.separator();
                for (label, next) in [
                    (
                        "Fold to Definitions (Ctrl+Alt+0)",
                        ToolbarAction::FoldToDefinitions,
                    ),
                    (
                        "Collapse Functions (Ctrl+Shift+0)",
                        ToolbarAction::CollapseFunctions,
                    ),
                    ("Show All Files", ToolbarAction::ShowAllFiles),
                ] {
                    if ui.button(label).clicked() {
                        action = Some(next);
                        ui.close_menu();
                    }
                }
                ui.separator();
                if ui.button("Bookmark Location (Ctrl+D)").clicked() {
                    action = Some(ToolbarAction::ToggleBookmark);
                    ui.close_menu();
                }
            });

            if let Some(command) = search_menu(ui) {
                action = Some(command);
            }

            // Right-aligned global shortcuts and theme toggle
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Toggle Context Inspector (Alt+Ctrl+I / Ctrl+Alt+I)
                let inspector_btn = ui
                    .selectable_label(show_inspector, "Inspector")
                    .on_hover_text("Show or Hide Context Inspector (Ctrl+Alt+I)");
                if inspector_btn.clicked() {
                    action = Some(ToolbarAction::ToggleInspector);
                }

                let theme_icon = if theme.is_dark { Icon::Sun } else { Icon::Moon };
                if theme_icon
                    .button(ui, 13.0, theme.text_primary)
                    .on_hover_text("Toggle Dark/Light Mode (Ctrl+Alt+T)")
                    .clicked()
                {
                    action = Some(ToolbarAction::ToggleTheme);
                }
            });
        });

        action
    }
}

/// Search and navigation menu shares one data-driven command dispatch.
fn search_menu(ui: &mut Ui) -> Option<ToolbarAction> {
    let mut action = None;
    ui.menu_button("Search", |ui| {
        let groups = [
            &[
                ("Find in File (Ctrl+F)", ToolbarAction::FindInFile),
                ("Next Match (Ctrl+G)", ToolbarAction::NextFindMatch),
                (
                    "Previous Match (Ctrl+Shift+G)",
                    ToolbarAction::PreviousFindMatch,
                ),
            ][..],
            &[
                ("Go to File… (Ctrl+K)", ToolbarAction::OpenQuickFilePicker),
                ("Go to Symbol… (Ctrl+T)", ToolbarAction::OpenGoToSymbol),
                ("Go to Line… (Ctrl+L)", ToolbarAction::OpenGoToLine),
                (
                    "Find References (Ctrl+Alt+R)",
                    ToolbarAction::FindReferences,
                ),
            ][..],
            &[(
                "Search Repository (Ctrl+Shift+F)",
                ToolbarAction::OpenSearch,
            )][..],
        ];
        for group in groups {
            for (label, command) in group {
                if ui.button(*label).clicked() {
                    action = Some(command.clone());
                    ui.close_menu();
                }
            }
            ui.separator();
        }
    });
    action
}
