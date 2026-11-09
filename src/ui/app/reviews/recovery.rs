//! Repository-owned failed review intent and bounded recovery controls.

use super::*;

pub(super) struct ReviewFailure {
    root: PathBuf,
    number: usize,
    message: String,
}

#[derive(Clone, Copy)]
pub(in crate::ui::app) enum RecoveryAction {
    Retry,
    ChooseNumber,
    Dismiss,
}

impl RadiologyApp {
    /// Returns the current repository's review failure, independently of reader errors.
    ///
    /// Recovery retains the failed PR number until dismissed or replaced. Opening
    /// another repository, exiting review or cancelling retires that intent.
    #[must_use]
    pub fn review_error(&self) -> Option<&str> {
        self.review_refresh
            .failure
            .as_ref()
            .filter(|failure| self.repository_root.as_ref() == Some(&failure.root))
            .map(|failure| failure.message.as_str())
    }

    pub(super) fn record_review_failure(&mut self, number: Option<usize>, message: String) {
        if let Some((root, number)) = self.repository_root.clone().zip(number) {
            self.review_refresh.failure = Some(ReviewFailure {
                root,
                number,
                message: format!("PR #{number}: {message}"),
            });
            self.review_refresh.failure_notice = Default::default();
        } else {
            self.error = Some(message);
        }
    }

    pub(in crate::ui::app) fn recover_review(&mut self, action: RecoveryAction) {
        let Some(failure) = self.review_refresh.failure.take() else {
            return;
        };
        self.review_refresh.failure_notice = Default::default();
        if self.repository_root.as_ref() != Some(&failure.root) || !self.can_choose_review() {
            return;
        }
        match action {
            RecoveryAction::Retry => self.open_pull_request(failure.number),
            RecoveryAction::ChooseNumber => self.show_pull_request_prompt(Some(failure.number)),
            RecoveryAction::Dismiss => {}
        }
    }

    pub(super) fn show_review_recovery(&mut self, ctx: &Context) {
        let Some(failure) = self.review_refresh.failure.as_ref() else {
            return;
        };
        if self.repository_root.as_ref() != Some(&failure.root) {
            self.review_refresh.failure = None;
            return;
        }
        let mut action = None;
        TopBottomPanel::top("review_failure").show(ctx, |ui| {
            if self.review_refresh.failure_notice.show_inline(
                ui,
                &self.theme,
                &failure.message,
                None,
            ) {
                action = Some(RecoveryAction::Dismiss);
            }
            ui.horizontal_wrapped(|ui| {
                if ui.button("Try Again").clicked() {
                    action = Some(RecoveryAction::Retry);
                }
                if ui.button("Choose Number…").clicked() {
                    action = Some(RecoveryAction::ChooseNumber);
                }
            });
        });
        if let Some(action) = action {
            self.recover_review(action);
        }
    }
}
