//! Review preparation and publication, separate from window rendering.

mod refresh;
use super::*;
pub(super) use refresh::ReviewRefresh;

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
        let number =
            pull_request.or_else(|| repository.is_none().then_some(saved.active_pr).flatten());
        if let Some(root) = root {
            self.request_repository(repository::RepositoryRequest {
                path: root,
                destination: if let Some(number) = number {
                    repository::RepositoryDestination::Review {
                        number,
                        file: file.map(str::to_owned),
                    }
                } else if let Some(file) = file {
                    repository::RepositoryDestination::Source {
                        location: SourceLocation::with_revision(file, 0, None),
                        target: crate::source::loader::SourceTarget::Byte(0),
                    }
                } else {
                    repository::RepositoryDestination::Default
                },
            });
        }
    }

    pub(super) fn can_choose_review(&self) -> bool {
        self.repository_root.is_some() && !self.is_repository_opening()
    }

    pub(super) fn show_pull_request_prompt(&mut self) {
        if self.can_choose_review() {
            let number = self
                .active_pr
                .as_ref()
                .or_else(|| self.recent_pull_requests.items().first())
                .map(|pr| pr.number);
            self.modal = ActiveModal::PullRequest(PullRequestModal::new(number));
        }
    }

    pub(super) fn poll_recent_pull_requests(&mut self, ctx: &Context) {
        self.recent_pull_requests.poll();
        if self.recent_pull_requests.is_loading() {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }

    /// Starts review preparation without blocking the source reader.
    ///
    /// Replacement, cancellation and repository switches retire publication and
    /// interrupt child processes. Preparation shares one two-minute deadline.
    pub fn open_pull_request(&mut self, number: usize) {
        if number == 0 {
            self.error = Some("Enter a positive pull request number".into());
            return;
        }
        let Some((root, cache)) = self.cached_repository() else {
            return;
        };
        self.cancel_repository_open();
        self.git_refresh.stop();
        self.git_error = None;

        self.review_refresh = Default::default();
        self.pending_pr = Some(number);
        self.error = None;
        self.review_job.spawn(move |cancel| {
            let control = crate::pull_request::control::ReviewControl::new(cancel);
            GitHubClient::view_cancellable(&root, number, &control).and_then(|summary| {
                ReviewSnapshot::prepare_cancellable(&root, summary, &cache, &control)
            })
        });
    }

    /// Cancels pending review preparation while retaining the accepted reader.
    pub fn cancel_review_preparation(&mut self) {
        self.review_job.stop();
        self.review_refresh = Default::default();
        self.pending_pr = None;
        self.pending_open_file = None;
    }

    pub(super) fn poll_review(&mut self, ctx: &Context) {
        match self.review_job.poll() {
            Ok(Some(result)) => {
                self.pending_pr = None;
                match result {
                    Ok(snapshot) => self.accept_review(snapshot),
                    Err(error) => {
                        self.pending_open_file = None;
                        self.error = Some(format!("Unable to open review: {error}"));
                    }
                }
            }
            Ok(None) if self.review_job.is_running() => {
                ctx.request_repaint_after(std::time::Duration::from_millis(50));
            }
            Ok(None) => {}
            Err(_) => {
                self.cancel_review_preparation();
                self.error = Some("Review preparation stopped unexpectedly".into());
            }
        }
        self.poll_review_refresh(ctx);
    }

    pub(super) fn accept_review(&mut self, snapshot: ReviewSnapshot) {
        self.cancel_repository_open();
        self.git_refresh.stop();
        self.git_error = None;
        if self.review.as_ref().is_some_and(|old| {
            old.summary.number == snapshot.summary.number
                && old.head == snapshot.head
                && old.base == snapshot.base
        }) {
            if self
                .review
                .as_ref()
                .is_some_and(|old| old.head_root != snapshot.head_root)
            {
                self.repository_search = Default::default();
            }
            self.recent_pull_requests.remember(&snapshot.summary);
            self.review_refresh.notice = Some("PR is up to date.".into());
            self.active_pr = Some(snapshot.summary.clone());
            self.review = Some(Arc::new(snapshot));
            return;
        }
        if self
            .review
            .as_ref()
            .is_some_and(|old| old.summary.number == snapshot.summary.number)
        {
            self.pending_pr = Some(snapshot.summary.number);
            self.review_refresh.queue(Arc::new(snapshot));
            return;
        }
        let location = self.open_buffer.as_ref().map(|buffer| {
            buffer.location_at_byte(buffer.byte_offset(self.caret_line, self.caret_col))
        });
        self.install_review(Arc::new(snapshot));
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

    pub(super) fn install_review(&mut self, snapshot: Arc<ReviewSnapshot>) {
        self.recent_pull_requests.remember(&snapshot.summary);
        self.review_refresh.notice = None;
        self.cancel_source_load();
        self.repository_search = Default::default();
        self.reset_index_state();
        self.save_bookmarks();
        self.active_pr = Some(snapshot.summary.clone());
        self.session.set_active_pr(Some(snapshot.summary.number));
        self.all_files = snapshot.files.clone();
        self.pr_changes = snapshot.changes.clone();
        self.inspect_index_projects();
        self.review = Some(snapshot);
        self.load_repository_index();
        self.active_revision = Some(ReviewRevision::After);
        self.sidebar_tab = SidebarTab::Changes;
        self.history.clear();
        self.pin_store.clear();
        self.source_viewport = Default::default();
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
