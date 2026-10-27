use egui::{RichText, Ui};
use crate::critical::lens::SourceMode;
use crate::pull_request::github::PullRequestSummary;
use crate::repository::filters::SourceFilters;
use crate::source::history::NavigationHistory;
use crate::source::location::ReviewRevision;
use crate::theme::AppTheme;

pub enum ToolbarAction {
    OpenRepositoryDialog,
    NavigateBack,
    NavigateForward,
    ToggleBookmark,
    OpenQuickFilePicker,
    OpenGoToLine,
    OpenGoToSymbol,
    OpenPRDialog,
    ExitPRReview,
    ToggleMode(SourceMode),
    ToggleRevision(ReviewRevision),
    ToggleSourceFilesOnly,
    ToggleHideTests,
}

pub struct Toolbar;

impl Toolbar {
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
                    if ui.selectable_value(&mut curr_mode, SourceMode::Normal, "Normal").clicked() {
                        action = Some(ToolbarAction::ToggleMode(SourceMode::Normal));
                    }
                    if ui.selectable_value(&mut curr_mode, SourceMode::Critical, "Critical").clicked() {
                        action = Some(ToolbarAction::ToggleMode(SourceMode::Critical));
                    }
                });

            // Filters Menu
            ui.menu_button("View", |ui| {
                if ui.checkbox(&mut { filters.source_files_only }, "Source Files Only").clicked() {
                    action = Some(ToolbarAction::ToggleSourceFilesOnly);
                }
                if ui.checkbox(&mut { filters.hides_tests }, "Hide Tests").clicked() {
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
