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
    /// * `history` - Navigation history tracker used to enable/disable back/forward buttons.
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut Ui,
        theme: &AppTheme,
        repo_name: Option<&str>,
        branch_name: Option<&str>,
        active_pr: Option<&PullRequestSummary>,
        revision: Option<ReviewRevision>,
        mode: SourceMode,
        filters: &SourceFilters,
        history: &NavigationHistory,
    ) -> Option<ToolbarAction> {
        let mut action = None;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            // Open Repository
            if ui.button("📁 Open Repository…").clicked() {
                action = Some(ToolbarAction::OpenRepositoryDialog);
            }

            // Repository Name & Branch
            if let Some(name) = repo_name {
                ui.separator();
                ui.label(RichText::new(name).strong());
                if let Some(branch) = branch_name {
                    ui.colored_label(theme.accent, format!("({branch})"));
                }
            }

            ui.separator();

            // Navigation Back/Forward
            ui.add_enabled_ui(history.can_go_back(), |ui| {
                if ui.button("◀ Back").clicked() {
                    action = Some(ToolbarAction::NavigateBack);
                }
            });
            ui.add_enabled_ui(history.can_go_forward(), |ui| {
                if ui.button("Forward ▶").clicked() {
                    action = Some(ToolbarAction::NavigateForward);
                }
            });

            // Bookmark shortcut
            if ui.button("🔖 Bookmark (Ctrl+D)").clicked() {
                action = Some(ToolbarAction::ToggleBookmark);
            }

            ui.separator();

            // Pull Request Review section
            if let Some(pr) = active_pr {
                ui.colored_label(
                    theme.accent,
                    format!("PR #{} [{}]: {}", pr.number, pr.state, pr.title),
                );

                // Before / After toggle
                let curr_rev = revision.unwrap_or(ReviewRevision::After);
                ui.selectable_value(&mut { curr_rev }, ReviewRevision::Before, "Before");
                if curr_rev != ReviewRevision::Before && ui.button("Before").clicked() {
                    action = Some(ToolbarAction::ToggleRevision(ReviewRevision::Before));
                }
                if curr_rev != ReviewRevision::After && ui.button("After").clicked() {
                    action = Some(ToolbarAction::ToggleRevision(ReviewRevision::After));
                }

                if ui.button("Exit Review").clicked() {
                    action = Some(ToolbarAction::ExitPRReview);
                }
            } else if ui.button("🔀 Pull Request… (Ctrl+P)").clicked() {
                action = Some(ToolbarAction::OpenPRDialog);
            }

            ui.separator();

            // Mode Selector: Normal / Critical
            let mut curr_mode = mode;
            egui::ComboBox::from_id_source("mode_picker")
                .selected_text(mode.title())
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(&mut curr_mode, SourceMode::Normal, "Normal")
                        .clicked()
                    {
                        action = Some(ToolbarAction::ToggleMode(SourceMode::Normal));
                    }
                    if ui
                        .selectable_value(&mut curr_mode, SourceMode::Critical, "Critical")
                        .clicked()
                    {
                        action = Some(ToolbarAction::ToggleMode(SourceMode::Critical));
                    }
                });

            // Filters Menu
            ui.menu_button("View", |ui| {
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
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Symbol ⌘T").clicked() {
                    action = Some(ToolbarAction::OpenGoToSymbol);
                }
                if ui.button("Line ⌘L").clicked() {
                    action = Some(ToolbarAction::OpenGoToLine);
                }
                if ui.button("File ⌘K").clicked() {
                    action = Some(ToolbarAction::OpenQuickFilePicker);
                }
            });
        });

        action
    }
}
