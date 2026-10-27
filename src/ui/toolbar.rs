//! Top application toolbar and quick-navigation control strip.
//!
//! Provides the primary repository status display, back/forward history navigation,
//! pull request review switcher, criticality mode selector, and quick palette shortcuts.

use crate::critical::lens::SourceMode;
use crate::pull_request::github::PullRequestSummary;
use crate::repository::filters::SourceFilters;
use crate::source::history::NavigationHistory;
use crate::source::location::ReviewRevision;
use crate::theme::AppTheme;
use crate::ui::icons::Icon;
use egui::{RichText, Ui};

/// Actions triggered by interacting with controls in the top [`Toolbar`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolbarAction {
    /// Opens the native directory chooser dialog to select a repository root.
    OpenRepositoryDialog,
    /// Navigates backwards in the file and caret navigation history.
    NavigateBack,
    /// Navigates forward in the file and caret navigation history.
    NavigateForward,
    /// Requests jumping to the next modified hunk in the active pull request review.
    NextChangeBlock,
    /// Requests jumping to the previous modified hunk in the active pull request review.
    PrevChangeBlock,
    /// Toggles a bookmark on the currently focused source line.
    ToggleBookmark,
    /// Opens the quick file picker modal dialog (Ctrl/Cmd+K).
    OpenQuickFilePicker,
    /// Opens the jump-to-line modal dialog (Ctrl/Cmd+L).
    OpenGoToLine,
    /// Opens the jump-to-symbol outline search dialog (Ctrl/Cmd+T).
    OpenGoToSymbol,
    /// Opens the pull request number prompt dialog.
    OpenPRDialog,
    /// Exits the active pull request review session and clears diff overlays.
    ExitPRReview,
    /// Changes the active [`SourceMode`] (e.g. Normal vs Critical lens).
    ToggleMode(SourceMode),
    /// Switches the active review revision (Before vs After).
    ToggleRevision(ReviewRevision),
    /// Toggles the filter to hide non-source code files.
    ToggleSourceFilesOnly,
    /// Toggles the filter to hide test directories and test files.
    ToggleHideTests,
    /// Toggles between dark and light appearance themes.
    ToggleTheme,
    /// Toggles whether long source code lines are soft-wrapped.
    ToggleWrapLines,
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
    /// * `repo_name` - Optional name of the currently opened repository.
    /// * `branch_name` - Optional name of the current Git HEAD branch.
    /// * `active_pr` - Metadata of the currently reviewed pull request, if any.
    /// * `revision` - Active revision view (`Before` or `After`).
    /// * `mode` - Active inspection mode (`Normal` or `Critical`).
    /// * `filters` - Repository file filter settings.
    /// * `wrap_lines` - Whether long lines wrap at viewport width.
    /// * `history` - Navigation history tracker used to enable/disable back/forward buttons.
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut Ui,
        theme: &AppTheme,
        repo_name: Option<&str>,
        branch_name: Option<&str>,
        active_pr: Option<&PullRequestSummary>,
        _revision: Option<ReviewRevision>,
        mode: SourceMode,
        filters: &SourceFilters,
        wrap_lines: bool,
        history: &NavigationHistory,
    ) -> Option<ToolbarAction> {
        let mut action = None;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;

            // Navigation Back/Forward (macOS style chevron icons)
            ui.add_enabled_ui(history.can_go_back(), |ui| {
                if Icon::ArrowLeft
                    .button(ui, 12.0, theme.text_primary)
                    .on_hover_text("Back (⌘[)")
                    .clicked()
                {
                    action = Some(ToolbarAction::NavigateBack);
                }
            });
            ui.add_enabled_ui(history.can_go_forward(), |ui| {
                if Icon::ArrowRight
                    .button(ui, 12.0, theme.text_primary)
                    .on_hover_text("Forward (⌘])")
                    .clicked()
                {
                    action = Some(ToolbarAction::NavigateForward);
                }
            });

            // Change Navigation Up/Down & Exit PR (visible when reviewing PR)
            if active_pr.is_some() {
                ui.separator();
                if Icon::ArrowUp
                    .button(ui, 11.0, theme.text_primary)
                    .on_hover_text("Previous Change Block (⌥↑)")
                    .clicked()
                {
                    action = Some(ToolbarAction::PrevChangeBlock);
                }
                if Icon::ArrowDown
                    .button(ui, 11.0, theme.text_primary)
                    .on_hover_text("Next Change Block (⌥↓)")
                    .clicked()
                {
                    action = Some(ToolbarAction::NextChangeBlock);
                }
                if ui
                    .button("↶ Exit PR")
                    .on_hover_text("Exit PR Review session")
                    .clicked()
                {
                    action = Some(ToolbarAction::ExitPRReview);
                }
            }

            ui.separator();

            // Open Repository
            if ui.button("Open…").clicked() {
                action = Some(ToolbarAction::OpenRepositoryDialog);
            }

            // Repository Name & Branch / PR Title (Center)
            if let Some(name) = repo_name {
                ui.separator();
                ui.vertical(|ui| {
                    ui.label(RichText::new(name).strong().size(13.0));
                    if let Some(pr) = active_pr {
                        let subtitle = format!("#{} · {}", pr.number, pr.title);
                        ui.colored_label(theme.text_secondary, RichText::new(subtitle).size(10.5));
                    } else if let Some(branch) = branch_name {
                        ui.colored_label(theme.text_secondary, RichText::new(branch).size(10.5));
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
                if ui.button("Bookmark (Ctrl+D)").clicked() {
                    action = Some(ToolbarAction::ToggleBookmark);
                    ui.close_menu();
                }
            });

            // Right-aligned global shortcuts and theme toggle
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let theme_icon = if theme.is_dark { Icon::Sun } else { Icon::Moon };
                if theme_icon
                    .button(ui, 13.0, theme.text_primary)
                    .on_hover_text("Toggle Dark/Light Mode (Ctrl+Alt+T)")
                    .clicked()
                {
                    action = Some(ToolbarAction::ToggleTheme);
                }
                if ui.button("Symbol ⌘T").clicked() {
                    action = Some(ToolbarAction::OpenGoToSymbol);
                }
                if ui.button("Line ⌘L").clicked() {
                    action = Some(ToolbarAction::OpenGoToLine);
                }
                if ui.button("File ⌘K").clicked() {
                    action = Some(ToolbarAction::OpenQuickFilePicker);
                }
                if active_pr.is_none() && ui.button("Review PR…").clicked() {
                    action = Some(ToolbarAction::OpenPRDialog);
                }
            });
        });

        action
    }
}
