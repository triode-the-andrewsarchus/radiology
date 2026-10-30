//! Review preparation and publication, separate from window rendering.

use super::*;

impl RadiologyApp {
    /// Applies explicit launch intent before considering a saved session.
    pub fn launch(
        &mut self,
        repository: Option<&Path>,
        pull_request: Option<usize>,
        file: Option<&str>,
    ) {
        let saved = self.session.clone();
        let root = repository
            .map(Path::to_path_buf)
            .or_else(|| saved.last_repository.as_ref().map(PathBuf::from));
        if let Some(root) = root {
            self.open_repository(&root);
        }
        let number =
            pull_request.or_else(|| repository.is_none().then_some(saved.active_pr).flatten());
        if let Some(number) = number {
            self.open_pull_request(number);
        }
        if let Some(file) = file {
            if self.pending_pr.is_some() {
                self.pending_open_file = Some(file.to_string());
            } else {
                self.open_file(file);
            }
        }
    }

    /// Starts review preparation without blocking the source reader.
    ///
    /// Replacing this request or switching repositories drops its receiver, so a
    /// superseded completion cannot publish into a different reader session.
    pub fn open_pull_request(&mut self, number: usize) {
        if number == 0 {
            self.error = Some("Enter a positive pull request number".into());
            return;
        }
        let Some((root, cache)) = self.cached_repository() else {
            return;
        };
        let (sender, receiver) = std::sync::mpsc::channel();
        self.review_rx = Some(receiver);
        self.pending_pr = Some(number);
        self.error = None;
        crate::background::READ_WORKERS.spawn_blocking(move || {
            let result = GitHubClient::view_pull_request(&root, number)
                .and_then(|summary| ReviewSnapshot::prepare(&root, summary, &cache));
            let _ = sender.send(result);
        });
    }

    pub(super) fn poll_review(&mut self, ctx: &Context) {
        let Some(receiver) = &self.review_rx else {
            return;
        };
        match receiver.try_recv() {
            Ok(result) => {
                self.review_rx = None;
                self.pending_pr = None;
                match result {
                    Ok(snapshot) => self.accept_review(snapshot),
                    Err(error) => self.error = Some(format!("Unable to open review: {error}")),
                }
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                ctx.request_repaint_after(std::time::Duration::from_millis(50))
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.review_rx = None;
                self.pending_pr = None;
                self.error = Some("Review preparation stopped unexpectedly".into());
            }
        }
    }

    pub(super) fn accept_review(&mut self, snapshot: ReviewSnapshot) {
        if self
            .review
            .as_ref()
            .is_some_and(|old| old.head == snapshot.head && old.base == snapshot.base)
        {
            self.active_pr = Some(snapshot.summary.clone());
            self.review = Some(Arc::new(snapshot));
            return;
        }
        let location = self.open_buffer.as_ref().map(|buffer| {
            buffer.location_at_byte(buffer.byte_offset(self.caret_line, self.caret_col))
        });
        self.cancel_source_load();
        self.reset_index_state();
        self.save_bookmarks();
        self.active_pr = Some(snapshot.summary.clone());
        self.session.set_active_pr(Some(snapshot.summary.number));
        self.all_files = snapshot.files.clone();
        self.pr_changes = snapshot.changes.clone();
        self.inspect_index_projects();
        self.review = Some(Arc::new(snapshot));
        self.active_revision = Some(ReviewRevision::After);
        self.sidebar_tab = SidebarTab::Changes;
        self.history.clear();
        self.pin_store.clear();
        self.open_buffer = None;
        self.selected_path = None;
        self.reference_context = Default::default();
        self.reset_previews();
        self.outline_symbols.clear();
        self.fold_ranges.clear();
        self.fold_error = None;
        self.fold_store = FoldStore::default();
        self.line_changes = None;
        self.highlighted_lines.clear();
        self.refresh_file_tree();
        self.load_bookmarks();
        let path = self
            .pending_open_file
            .take()
            .or_else(|| location.as_ref().map(|location| location.path.clone()))
            .or_else(|| self.pr_changes.first().map(|change| change.path.clone()))
            .or_else(|| self.all_files.first().map(|file| file.path.clone()));
        if let Some(path) = path {
            let target = SourceLocation::with_revision(
                path,
                location.map_or(0, |location| location.byte_offset),
                None,
            );
            self.navigate(target, true);
        }
    }

    pub(super) fn resolve_location(
        &self,
        mut location: SourceLocation,
    ) -> Result<SourceLocation, String> {
        if !SourceLocation::is_repository_path(&location.path) {
            return Err("Invalid repository-relative source path".into());
        }
        if let Some(review) = &self.review {
            return review.resolve(location);
        }
        location.revision = None;
        Ok(location)
    }
}
