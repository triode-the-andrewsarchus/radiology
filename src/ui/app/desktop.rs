//! Files-row handoff ownership; accepted file managers own their own lifetime.

use super::*;

impl RadiologyApp {
    pub(super) fn reveal_in_file_manager(&mut self, path: String) {
        let Some(root) = self.search_root() else {
            return;
        };
        let review = self.review.clone();
        let revision = self
            .open_buffer
            .as_ref()
            .filter(|buffer| buffer.path == path)
            .and_then(|buffer| buffer.revision)
            .unwrap_or(ReviewRevision::After);
        self.reveal_job.spawn(move |cancel| {
            let (root, tree_lease) = match &review {
                Some(review) => review.reveal_tree(revision, cancel)?,
                None => (root, None),
            };
            let result = crate::desktop::prepare_reveal(&root, &path).and_then(|command| {
                crate::desktop::show_items(&command, cancel, std::time::Duration::from_secs(5))
            });
            // Keep the exact pinned revision alive until the desktop acknowledges it.
            drop(tree_lease);
            drop(review);
            result
        });
    }

    pub(super) fn poll_reveal(&mut self, ctx: &Context) {
        match self.reveal_job.poll() {
            Ok(Some(Err(error))) => self.error = Some(error),
            Err(_) => self.error = Some("File-manager request stopped unexpectedly".into()),
            Ok(_) => {}
        }
        if self.reveal_job.is_running() {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }
}
