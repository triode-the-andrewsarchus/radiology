//! Compact PR switcher shared by standalone and embedded reader toolbars.

use super::commands::ReaderCommand;
use crate::pull_request::{github::PullRequestSummary, recent::PullRequestListPhase};
use egui::{ScrollArea, Ui};

/// Borrowed review/list presentation with no request ownership inside rendering.
pub struct PullRequestMenu<'a> {
    /// Accepted review metadata, including a review absent from the recent list.
    pub active: Option<&'a PullRequestSummary>,
    /// Review currently being prepared while accepted content remains visible.
    pub pending: Option<usize>,
    /// Repository-scoped accepted list and its independent refresh lifecycle.
    pub recent: &'a [PullRequestSummary],
    /// Independent list refresh lifecycle, including retained-list errors.
    pub phase: &'a PullRequestListPhase,
    /// Whether repository-dependent commands can run in this reader session.
    pub enabled: bool,
}

impl PullRequestMenu<'_> {
    /// Renders the switcher and explicit Exit PR control, returning a host command.
    pub fn show(&self, ui: &mut Ui) -> Option<ReaderCommand> {
        let mut action = None;
        let title = self
            .pending
            .map(|number| format!("#{number}…"))
            .or_else(|| self.active.map(|pr| format!("#{}", pr.number)))
            .unwrap_or_else(|| "Pull Request".into());
        let menu = ui.menu_button(title, |ui| action = self.contents(ui));
        if menu.inner.is_none() {
            menu.response.on_hover_text(
                self.active
                    .map(|pr| format!("#{} {} · {}", pr.number, pr.title, pr.state_title()))
                    .unwrap_or_else(|| {
                        "Review a recent pull request or open a number (Ctrl+P).".into()
                    }),
            );
        }
        if (self.active.is_some() || self.pending.is_some())
            && ui
                .button("Exit PR")
                .on_hover_text("Return to the local checkout or cancel preparation.")
                .clicked()
        {
            action = Some(ReaderCommand::ExitPRReview);
        }
        action
    }

    fn contents(&self, ui: &mut Ui) -> Option<ReaderCommand> {
        ui.set_width((ui.ctx().screen_rect().width() - 24.0).clamp(100.0, 420.0));
        let mut action = self.choices(ui);
        let loading = matches!(self.phase, PullRequestListPhase::Loading);
        ui.separator();
        if ui
            .add_enabled(
                self.enabled,
                egui::Button::new("Open Pull Request… (Ctrl+P)"),
            )
            .clicked()
        {
            action = Some(ReaderCommand::OpenPRDialog);
        }
        if ui
            .add_enabled(
                self.enabled && self.active.is_some() && self.pending.is_none(),
                egui::Button::new("Refresh PR (Ctrl+R)"),
            )
            .clicked()
        {
            action = Some(ReaderCommand::RefreshPR);
        }
        if ui
            .add_enabled(self.enabled && !loading, egui::Button::new("Refresh List"))
            .clicked()
        {
            action = Some(ReaderCommand::RefreshPRList);
        }
        if loading && ui.button("Cancel List Load").clicked() {
            action = Some(ReaderCommand::CancelPRList);
        }
        self.status(ui);
        if action.is_some() {
            ui.close_menu();
        }
        action
    }

    fn choices(&self, ui: &mut Ui) -> Option<ReaderCommand> {
        let mut action = None;
        let local = self.active.is_none() && self.pending.is_none();
        if ui
            .add_enabled(
                self.enabled || !local,
                egui::SelectableLabel::new(local, "Local Checkout"),
            )
            .clicked()
            && !local
        {
            action = Some(ReaderCommand::ExitPRReview);
        }
        let missing = self
            .active
            .filter(|active| !self.recent.iter().any(|pr| pr.number == active.number));
        ScrollArea::vertical()
            .id_source("recent_pull_requests")
            .max_height(280.0)
            .show(ui, |ui| {
                for pr in missing.into_iter().chain(self.recent.iter()) {
                    let selected = self
                        .pending
                        .or_else(|| self.active.map(|active| active.number))
                        == Some(pr.number);
                    let label = format!("#{}  {} · {}", pr.number, pr.title, pr.state_title());
                    let response = ui
                        .add_enabled(self.enabled, egui::SelectableLabel::new(selected, label))
                        .on_hover_text(format!(
                            "{} → {} · Updated {}",
                            pr.head_ref_name, pr.base_ref_name, pr.updated_at
                        ));
                    if response.gained_focus() {
                        response.scroll_to_me(None);
                    }
                    if response.clicked() && !selected {
                        action = Some(ReaderCommand::OpenPullRequest(pr.number));
                    }
                }
            });
        action
    }

    fn status(&self, ui: &mut Ui) {
        match self.phase {
            PullRequestListPhase::Loading => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Loading pull requests…");
                });
            }
            PullRequestListPhase::Loaded if self.recent.is_empty() => {
                ui.label("No recent pull requests.");
            }
            PullRequestListPhase::Failed(error) | PullRequestListPhase::Unavailable(error) => {
                let mut detail: String = error.chars().take(400).collect();
                if error.chars().nth(400).is_some() {
                    detail.push('…');
                }
                ui.add(egui::Label::new(detail).wrap(true));
            }
            PullRequestListPhase::Cancelled => {
                ui.label("List loading cancelled.");
            }
            PullRequestListPhase::Loaded => {}
        }
        if !self.recent.is_empty() && !matches!(self.phase, PullRequestListPhase::Loaded) {
            ui.weak("Showing the previous list.");
        }
    }
}

#[cfg(all(test, unix))]
mod tests;
