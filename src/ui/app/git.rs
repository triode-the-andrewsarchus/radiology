//! Checkout Git refresh publication, independent of source and review jobs.

use super::*;

impl RadiologyApp {
    /// Refreshes branch and checkout changes without blocking the reader.
    /// Existing accepted changes remain visible until a complete snapshot arrives.
    pub fn refresh_git_changes(&mut self) {
        if self.active_pr.is_some() || self.pending_pr.is_some() {
            return;
        }
        let Some(root) = self.repository_root.clone() else {
            return;
        };
        let source = self.open_buffer.clone();
        self.git_error = None;
        self.git_refresh
            .spawn(move |cancel| crate::repository::git_refresh::read(root, source, cancel));
    }

    /// Returns whether a checkout Git refresh is awaiting publication.
    #[must_use]
    pub fn is_git_refreshing(&self) -> bool {
        self.git_refresh.is_running()
    }

    /// Cancels pending checkout work, retaining its last accepted snapshot.
    pub fn cancel_git_refresh(&mut self) {
        self.git_refresh.stop();
    }

    /// Publishes only snapshots belonging to the current checkout session.
    pub fn poll_git_refresh(&mut self, ctx: &Context) {
        match self.git_refresh.poll() {
            Ok(Some(Ok(snapshot))) => {
                if self.active_pr.is_some()
                    || self.pending_pr.is_some()
                    || self.repository_root.as_ref() != Some(&snapshot.root)
                {
                    return;
                }
                self.branch_name = snapshot.branch;
                self.pr_changes = snapshot.changes;
                if snapshot.source.as_ref().is_some_and(|identity| {
                    self.open_buffer
                        .as_ref()
                        .is_some_and(|buffer| identity.matches(buffer))
                }) {
                    self.line_changes = snapshot.lines;
                    self.sync_outline();
                }
                self.git_error = None;
            }
            Ok(Some(Err(error))) => self.git_error = Some(error),
            Err(_) => self.git_error = Some("Git refresh stopped unexpectedly".into()),
            Ok(None) => {}
        }
        if self.git_refresh.is_running() {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }

    pub(super) fn show_git_status(&mut self, ctx: &Context) {
        if self.is_git_refreshing() {
            egui::TopBottomPanel::top("git_refresh").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Refreshing changes…");
                    if ui.button("Cancel").clicked() {
                        self.cancel_git_refresh();
                    }
                });
            });
        } else if let Some(error) = self.git_error.clone() {
            egui::TopBottomPanel::top("git_refresh_error").show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("Unable to refresh changes: {error}"));
                    if ui.button("Retry").clicked() {
                        self.refresh_git_changes();
                    }
                    if ui.button("Dismiss").clicked() {
                        self.git_error = None;
                    }
                });
            });
        }
    }
}
