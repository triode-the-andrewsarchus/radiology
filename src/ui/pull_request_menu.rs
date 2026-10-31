//! Compact PR switcher shared by standalone and embedded reader toolbars.

use super::commands::ReaderCommand;
use crate::pull_request::{github::PullRequestSummary, recent::PullRequestListPhase};
use egui::Ui;

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
        let mut bar = super::context_menu::MenuBar::default();
        let action = self.add_to_bar(ui, &mut bar);
        bar.show().or(action)
    }

    pub(crate) fn add_to_bar(
        &self,
        ui: &mut Ui,
        bar: &mut super::context_menu::MenuBar<ReaderCommand>,
    ) -> Option<ReaderCommand> {
        let title = self
            .pending
            .map(|number| format!("#{number}…"))
            .or_else(|| self.active.map(|pr| format!("#{}", pr.number)))
            .unwrap_or_else(|| "Pull Request".into());
        let scope = (
            "pull_requests",
            self.active.map(|pr| pr.number),
            self.pending,
            self.recent
                .iter()
                .map(|pr| (pr.number, &pr.title, &pr.state))
                .collect::<Vec<_>>(),
        );
        let response = bar.button(ui, title, scope, self.items());
        if !super::context_menu::is_open(ui.ctx()) {
            response.on_hover_text(
                self.active
                    .map(|pr| format!("#{} {} · {}", pr.number, pr.title, pr.state_title()))
                    .unwrap_or_else(|| {
                        "Review a recent pull request or open a number (Ctrl+P).".into()
                    }),
            );
        }
        ((self.active.is_some() || self.pending.is_some())
            && ui
                .button("Exit PR")
                .on_hover_text("Return to the local checkout or cancel preparation.")
                .clicked())
        .then_some(ReaderCommand::ExitPRReview)
    }

    fn items(&self) -> Vec<super::context_menu::Item<ReaderCommand>> {
        use super::context_menu::Item;
        let local = self.active.is_none() && self.pending.is_none();
        let mut items = vec![Item::action("Local Checkout", ReaderCommand::ExitPRReview)
            .checked(local)
            .enabled(!local)];
        let missing = self
            .active
            .filter(|active| !self.recent.iter().any(|pr| pr.number == active.number));
        for pr in missing.into_iter().chain(self.recent.iter()) {
            let selected = self
                .pending
                .or_else(|| self.active.map(|active| active.number))
                == Some(pr.number);
            items.push(
                Item::action(
                    format!("#{}  {} · {}", pr.number, pr.title, pr.state_title()),
                    ReaderCommand::OpenPullRequest(pr.number),
                )
                .checked(selected)
                .enabled(self.enabled && !selected)
                .tooltip(format!(
                    "{} → {} · Updated {}",
                    pr.head_ref_name, pr.base_ref_name, pr.updated_at
                )),
            );
        }
        items.push(Item::separator());
        let loading = matches!(self.phase, PullRequestListPhase::Loading);
        items.extend([
            Item::action("Open Pull Request… (Ctrl+P)", ReaderCommand::OpenPRDialog)
                .enabled(self.enabled),
            Item::action("Refresh PR (Ctrl+R)", ReaderCommand::RefreshPR)
                .enabled(self.enabled && self.active.is_some() && self.pending.is_none()),
            Item::action("Refresh List", ReaderCommand::RefreshPRList)
                .enabled(self.enabled && !loading),
        ]);
        if loading {
            items.push(Item::action(
                "Cancel List Load",
                ReaderCommand::CancelPRList,
            ));
        }
        let status = match self.phase {
            PullRequestListPhase::Loading => Some("Loading pull requests…".into()),
            PullRequestListPhase::Loaded if self.recent.is_empty() => {
                Some("No recent pull requests.".into())
            }
            PullRequestListPhase::Failed(error) | PullRequestListPhase::Unavailable(error) => {
                let mut detail: String = error.chars().take(400).collect();
                if error.chars().nth(400).is_some() {
                    detail.push('…');
                }
                Some(detail)
            }
            PullRequestListPhase::Cancelled => Some("List loading cancelled.".into()),
            PullRequestListPhase::Loaded => None,
        };
        if let Some(status) = status {
            items.push(Item::label(status));
        }
        if !self.recent.is_empty() && !matches!(self.phase, PullRequestListPhase::Loaded) {
            items.push(Item::label("Showing the previous list."));
        }
        items
    }
}

#[cfg(all(test, unix))]
mod tests;
