//! Top application toolbar and quick-navigation control strip.
//!
//! Provides the primary repository status display, back/forward history navigation,
//! pull request review switcher, criticality mode selector, and quick palette shortcuts.

use super::context_menu::{Item, MenuBar};
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
    /// * `change_navigation` - Available change targets for the accepted source.
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
        change_navigation: crate::pull_request::diff::ChangeNavigation,
    ) -> Option<ToolbarAction> {
        let previous_focus = ui.memory(|memory| memory.focused());
        let mut menus = MenuBar::default();
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
                menus.button(
                    ui,
                    RichText::new(name).strong().size(13.0),
                    ("repository", name, recent_repos),
                    vec![
                        Item::action(
                            "Open Repository… (Ctrl+O)",
                            ToolbarAction::OpenRepositoryDialog,
                        ),
                        Item::action("Copy Repository Path", ToolbarAction::CopyRepoPath),
                        Item::separator(),
                        Item::submenu(
                            "Open Recent",
                            recent_repos
                                .iter()
                                .map(|path| {
                                    Item::action(path, ToolbarAction::OpenRecentRepo(path.clone()))
                                })
                                .collect(),
                        ),
                    ],
                );
                if let Some(branch) = branch_name {
                    ui.colored_label(theme.text_secondary, RichText::new(branch).size(10.5));
                }
            } else if ui.button("Open Repository… (Ctrl+O)").clicked() {
                action = Some(ToolbarAction::OpenRepositoryDialog);
            }

            ui.separator();

            if let Some(command) = review_menu.add_to_bar(ui, &mut menus) {
                action = Some(command);
            }
            if review_menu.active.is_some() {
                // Change navigation buttons
                ui.add_enabled_ui(change_navigation.previous.is_some(), |ui| {
                    if Icon::ArrowUp
                        .button(ui, 11.0, theme.text_primary)
                        .on_hover_text("Previous Change Block (Alt+↑)")
                        .clicked()
                    {
                        action = Some(ToolbarAction::PrevChangeBlock);
                    }
                });
                ui.add_enabled_ui(change_navigation.next.is_some(), |ui| {
                    if Icon::ArrowDown
                        .button(ui, 11.0, theme.text_primary)
                        .on_hover_text("Next Change Block (Alt+↓)")
                        .clicked()
                    {
                        action = Some(ToolbarAction::NextChangeBlock);
                    }
                });
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
            menus.button(ui, "View", "view", view_items(theme, filters, wrap_lines));
            menus.button(ui, "Search", "search", search_items());

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

        let menu_action = menus.show();
        if action.is_some() && menu_action.is_none() {
            ui.memory_mut(|memory| {
                if let Some(previous) = previous_focus {
                    memory.request_focus(previous);
                } else if let Some(focused) = memory.focused() {
                    memory.surrender_focus(focused);
                }
            });
        }
        menu_action.or(action)
    }
}

fn view_items(
    theme: &AppTheme,
    filters: &SourceFilters,
    wrap_lines: bool,
) -> Vec<Item<ToolbarAction>> {
    let mut items = vec![
        Item::action(
            if theme.is_dark {
                "Switch to Light Mode"
            } else {
                "Switch to Dark Mode"
            },
            ToolbarAction::ToggleTheme,
        ),
        Item::separator(),
        Item::action("Wrap Lines", ToolbarAction::ToggleWrapLines).checked(wrap_lines),
        Item::separator(),
        Item::action("Source Files Only", ToolbarAction::ToggleSourceFilesOnly)
            .checked(filters.source_files_only),
        Item::action("Hide Tests", ToolbarAction::ToggleHideTests).checked(filters.hides_tests),
        Item::separator(),
    ];
    for (label, action) in [
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
        items.push(Item::action(label, action));
    }
    items.push(Item::separator());
    for (label, action) in [
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
        items.push(Item::action(label, action));
    }
    items.push(Item::separator());
    items.push(Item::action(
        "Bookmark Location (Ctrl+D)",
        ToolbarAction::ToggleBookmark,
    ));
    items
}

/// Search and navigation menu shares one data-driven command dispatch.
fn search_items() -> Vec<Item<ToolbarAction>> {
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
                "Go to Definition (Ctrl+Alt+J / F12)",
                ToolbarAction::GoToDefinition,
            ),
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
    let mut items = Vec::new();
    for group in groups {
        if !items.is_empty() {
            items.push(Item::separator());
        }
        items.extend(
            group
                .iter()
                .map(|(label, action)| Item::action(*label, action.clone())),
        );
    }
    items
}

#[cfg(test)]
fn search_menu(ui: &mut Ui) -> Option<ToolbarAction> {
    let mut menus = MenuBar::default();
    menus.button(ui, "Search", "search", search_items());
    menus.show()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::test_support::{click, painted_frame, text_center};

    #[test]
    fn search_menu_exposes_definition_and_dispatches_the_shared_command() {
        let ctx = egui::Context::default();
        let mut action = None;
        let output = painted_frame(&ctx, egui::vec2(600.0, 600.0), vec![], |ui| {
            action = search_menu(ui);
        });
        painted_frame(
            &ctx,
            egui::vec2(600.0, 600.0),
            click(text_center(&output, "Search")),
            |ui| {
                action = search_menu(ui);
            },
        );
        let output = painted_frame(&ctx, egui::vec2(600.0, 600.0), vec![], |ui| {
            action = search_menu(ui);
        });
        assert!(action.is_none());
        painted_frame(
            &ctx,
            egui::vec2(600.0, 600.0),
            click(text_center(&output, "Go to Definition (Ctrl+Alt+J / F12)")),
            |ui| {
                action = search_menu(ui);
            },
        );
        assert_eq!(action, Some(ToolbarAction::GoToDefinition));
    }
}
