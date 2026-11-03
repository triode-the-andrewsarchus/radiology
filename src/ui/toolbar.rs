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

/// Immutable toolbar inputs shared by standalone and embedded reader hosts.
pub struct ToolbarContent<'a> {
    /// Navigation sidebar visibility.
    pub show_sidebar: bool,
    /// Context inspector visibility.
    pub show_inspector: bool,
    /// Accepted repository name.
    pub repo_name: Option<&'a str>,
    /// Accepted branch name.
    pub branch_name: Option<&'a str>,
    /// Recent repository paths.
    pub recent_repos: &'a [String],
    /// Review choices and current availability.
    pub review_menu: PullRequestMenu<'a>,
    /// Current source presentation.
    pub mode: SourceMode,
    /// Current file filters.
    pub filters: &'a SourceFilters,
    /// Source wrapping preference.
    pub wrap_lines: bool,
    /// Committed navigation history.
    pub history: &'a NavigationHistory,
    /// Accepted change targets.
    pub change_navigation: crate::pull_request::diff::ChangeNavigation,
    /// Bookmarks in the current scope.
    pub bookmarks: &'a crate::bookmarks::store::BookmarkStore,
    /// Whether the exact caret is bookmarked.
    pub current_bookmarked: bool,
}

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
    /// * `bookmarks` - Current repository/review bookmark slots.
    /// * `current_bookmarked` - Whether the accepted caret location has a bookmark.
    /// * `command_enabled` - Current application availability for menu commands.
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
        bookmarks: &crate::bookmarks::store::BookmarkStore,
        current_bookmarked: bool,
        command_enabled: impl Fn(&ToolbarAction) -> bool,
    ) -> Option<ToolbarAction> {
        Self::show_with_panes(
            ui,
            theme,
            ToolbarContent {
                show_sidebar,
                show_inspector,
                repo_name,
                branch_name,
                recent_repos,
                review_menu,
                mode,
                filters,
                wrap_lines,
                history,
                change_navigation,
                bookmarks,
                current_bookmarked,
            },
            &Default::default(),
            command_enabled,
        )
    }

    /// Renders the toolbar with host-owned persistent inspector choices.
    #[allow(clippy::too_many_arguments)]
    pub fn show_with_panes(
        ui: &mut Ui,
        theme: &AppTheme,
        content: ToolbarContent<'_>,
        panes: &crate::session::inspector::InspectorPanes,
        command_enabled: impl Fn(&ToolbarAction) -> bool,
    ) -> Option<ToolbarAction> {
        let ToolbarContent {
            show_sidebar,
            show_inspector,
            repo_name,
            branch_name,
            recent_repos,
            review_menu,
            mode,
            filters,
            wrap_lines,
            history,
            change_navigation,
            bookmarks,
            current_bookmarked,
        } = content;
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
                let title = super::text_fit::middle(
                    ui,
                    &super::text_fit::bounded(name).replace(['\r', '\n'], " "),
                    egui::FontId::proportional(13.0),
                    theme.text_primary,
                    140.0 - 2.0 * ui.spacing().button_padding.x,
                );
                menus
                    .button(
                        ui,
                        RichText::new(title.job.text.clone()).strong().size(13.0),
                        ("repository", name, recent_repos),
                        vec![
                            Item::action(
                                "Open Repository… (Ctrl+O)",
                                ToolbarAction::OpenRepositoryDialog,
                            ),
                            Item::action("Copy Repository Path", ToolbarAction::CopyRepoPath),
                            Item::action(
                                "Import Code Path… (Ctrl+Shift+P)",
                                ToolbarAction::ImportCodePath,
                            )
                            .enabled(command_enabled(&ToolbarAction::ImportCodePath)),
                            Item::action(
                                if review_menu.active.is_some() {
                                    "Refresh PR (Ctrl+R)"
                                } else {
                                    "Reload Repository (Ctrl+R)"
                                },
                                ToolbarAction::Refresh,
                            )
                            .enabled(command_enabled(&ToolbarAction::Refresh)),
                            Item::separator(),
                            Item::submenu(
                                "Open Recent",
                                recent_repos
                                    .iter()
                                    .map(|path| {
                                        Item::action(
                                            path,
                                            ToolbarAction::OpenRecentRepo(path.clone()),
                                        )
                                    })
                                    .collect(),
                            ),
                        ],
                    )
                    .on_hover_ui(|ui| super::text_fit::explanation(ui, name, theme.text_secondary));
                if let Some(branch) = branch_name {
                    let label = super::text_fit::middle(
                        ui,
                        &super::text_fit::bounded(branch).replace(['\r', '\n'], " "),
                        egui::FontId::proportional(10.5),
                        theme.text_secondary,
                        80.0,
                    );
                    ui.label(label).on_hover_ui(|ui| {
                        super::text_fit::explanation(ui, branch, theme.text_secondary)
                    });
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
            ui.add_enabled_ui(command_enabled(&ToolbarAction::ToggleMode(mode)), |ui| {
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
            menus.button(
                ui,
                "View",
                "view",
                view_items(
                    theme,
                    filters,
                    wrap_lines,
                    bookmarks,
                    current_bookmarked,
                    &command_enabled,
                    panes,
                ),
            );
            menus.button(ui, "Search", "search", search_items(&command_enabled));
            menus.button(
                ui,
                "Help",
                "help",
                vec![Item::action(
                    "Keyboard Shortcuts (Ctrl+/)",
                    ToolbarAction::OpenKeyboardShortcuts,
                )],
            );

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
    bookmarks: &crate::bookmarks::store::BookmarkStore,
    current_bookmarked: bool,
    command_enabled: &impl Fn(&ToolbarAction) -> bool,
    panes: &crate::session::inspector::InspectorPanes,
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
    items.push(Item::submenu(
        "Context Panes",
        super::context_inspector::panes::menu(*panes),
    ));
    items.push(Item::separator());
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
        let enabled = command_enabled(&action);
        items.push(Item::action(label, action).enabled(enabled));
    }
    items.push(Item::separator());
    for (label, action) in [
        (
            "Toggle Current Function (x in source)",
            ToolbarAction::ToggleCurrentFunction,
        ),
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
        let enabled = command_enabled(&action);
        items.push(Item::action(label, action).enabled(enabled));
    }
    items.push(Item::separator());
    items.push(
        Item::action(
            if current_bookmarked {
                "Remove Bookmark (Ctrl+D)"
            } else {
                "Bookmark Location (Ctrl+D)"
            },
            ToolbarAction::ToggleBookmark,
        )
        .enabled(command_enabled(&ToolbarAction::ToggleBookmark)),
    );
    items.push(Item::submenu(
        "Go to Bookmark",
        (0..10)
            .map(|slot| {
                let command = ToolbarAction::OpenBookmark(slot);
                let label = bookmarks.get_by_index(slot).map_or_else(
                    || format!("Bookmark {slot}"),
                    |bookmark| bookmark.label.clone(),
                );
                let enabled = command_enabled(&command);
                Item::action(label, command)
                    .hint(format!("Ctrl+{slot}"))
                    .enabled(enabled)
            })
            .collect(),
    ));
    items.push(Item::separator());
    items.push(
        Item::action("Show Code Paths (Ctrl+Alt+P)", ToolbarAction::ShowCodePaths)
            .enabled(command_enabled(&ToolbarAction::ShowCodePaths)),
    );
    items
}

/// Search and navigation menu shares one data-driven command dispatch.
fn search_items(command_enabled: &impl Fn(&ToolbarAction) -> bool) -> Vec<Item<ToolbarAction>> {
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
        items.extend(group.iter().map(|(label, action)| {
            Item::action(*label, action.clone()).enabled(command_enabled(action))
        }));
    }
    items
}

#[cfg(test)]
fn search_menu(ui: &mut Ui, enabled: impl Fn(&ToolbarAction) -> bool) -> Option<ToolbarAction> {
    let mut menus = MenuBar::default();
    menus.button(ui, "Search", "search", search_items(&enabled));
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
            action = search_menu(ui, |_| true);
        });
        painted_frame(
            &ctx,
            egui::vec2(600.0, 600.0),
            click(text_center(&output, "Search")),
            |ui| {
                action = search_menu(ui, |_| true);
            },
        );
        let output = painted_frame(&ctx, egui::vec2(600.0, 600.0), vec![], |ui| {
            action = search_menu(ui, |_| true);
        });
        assert!(action.is_none());
        painted_frame(
            &ctx,
            egui::vec2(600.0, 600.0),
            click(text_center(&output, "Go to Definition (Ctrl+Alt+J / F12)")),
            |ui| {
                action = search_menu(ui, |_| true);
            },
        );
        assert_eq!(action, Some(ToolbarAction::GoToDefinition));
    }

    #[test]
    fn search_menu_rechecks_availability_while_open_and_skips_disabled_commands() {
        let ctx = egui::Context::default();
        let size = egui::vec2(600.0, 600.0);
        let mut action = None;
        let mut frame = |events, available| {
            painted_frame(&ctx, size, events, |ui| {
                action = search_menu(ui, |command| {
                    available || matches!(command, ToolbarAction::OpenSearch)
                });
            })
        };
        let output = frame(vec![], true);
        frame(click(text_center(&output, "Search")), true);
        let output = frame(vec![], true);
        frame(click(text_center(&output, "Go to Line… (Ctrl+L)")), false);
        assert!(
            action.is_none(),
            "a stale enabled row dispatched after its source vanished"
        );
        crate::ui::test_support::menu_frame(
            &ctx,
            size,
            vec![
                crate::ui::commands::test_key(egui::Key::Home),
                crate::ui::commands::test_key(egui::Key::Enter),
            ],
            |ui| {
                action = search_menu(ui, |command| matches!(command, ToolbarAction::OpenSearch));
            },
        );
        assert_eq!(action, Some(ToolbarAction::OpenSearch));
    }

    #[test]
    fn view_bookmark_menu_updates_toggle_and_navigates_the_tenth_slot() {
        use crate::ui::{commands::test_key, test_support::menu_frame};
        let ctx = egui::Context::default();
        let source = crate::source::SourceBuffer::from_str("a.rs", "fn alpha() {}\n");
        let mut bookmarks = crate::bookmarks::store::BookmarkStore::default();
        for slot in 0..10 {
            bookmarks
                .bookmarks
                .push(crate::bookmarks::bookmark::SourceBookmark::new(
                    &source,
                    0,
                    format!("Slot {slot}"),
                ));
        }
        let mut action = None;
        let mut frame = |events, current| {
            menu_frame(&ctx, egui::vec2(900.0, 700.0), events, |ui| {
                let mut menus = MenuBar::default();
                menus.button(
                    ui,
                    "View",
                    "view",
                    view_items(
                        &AppTheme::dark(),
                        &SourceFilters::default(),
                        false,
                        &bookmarks,
                        current,
                        &|command| matches!(command, ToolbarAction::OpenBookmark(9)),
                        &Default::default(),
                    ),
                );
                action = menus.show();
            })
        };
        let output = frame(vec![], false);
        frame(click(text_center(&output, "View")), false);
        let output = frame(vec![], false);
        text_center(&output, "Bookmark Location (Ctrl+D)");
        let output = frame(vec![], true);
        text_center(&output, "Remove Bookmark (Ctrl+D)");
        frame(click(text_center(&output, "Go to Bookmark")), true);
        let output = frame(vec![], true);
        for slot in 0..10 {
            text_center(&output, &format!("Slot {slot}"));
        }
        frame(
            vec![test_key(egui::Key::Home), test_key(egui::Key::Enter)],
            true,
        );
        assert_eq!(action, Some(ToolbarAction::OpenBookmark(9)));
    }
}
