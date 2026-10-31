//! Repository opening transactions that retain the accepted reader until discovery succeeds.

use super::*;
use crate::repository::discovery::DiscoveredRepository;

#[derive(Clone)]
pub(super) struct RepositoryRequest {
    pub path: PathBuf,
    pub destination: RepositoryDestination,
}

#[derive(Clone)]
pub(super) enum RepositoryDestination {
    Default,
    Source {
        location: SourceLocation,
        target: crate::source::loader::SourceTarget,
    },
    Review {
        number: usize,
        file: Option<String>,
    },
    RestoreCurrent,
}

impl RadiologyApp {
    /// Requests repository discovery while retaining the current reader.
    ///
    /// Hosts drive [`Self::poll_repository`] or the normal frame loop to publish
    /// the new root and file inventory. Subsequent source and semantic work also
    /// publish asynchronously through [`Self::poll_source`] and [`Self::poll_index`].
    pub fn open_repository(&mut self, path: &Path) {
        self.request_repository(RepositoryRequest {
            path: path.into(),
            destination: RepositoryDestination::Default,
        });
    }

    pub(super) fn request_repository(&mut self, request: RepositoryRequest) {
        // A newer root supersedes any unaccepted review intent, while retaining
        // the currently accepted checkout/review until discovery succeeds.
        self.cancel_review_preparation();
        self.recent_pull_requests.cancel();
        let path = request.path.clone();
        let filters = self.session.source_filters.clone();
        self.pending_repository = Some(request);
        self.repository_error = None;
        self.repository_open.spawn(move |cancel| {
            RepositoryDiscovery::discover(&path, &filters, cancel)
                .map_err(|error| error.to_string())
        });
    }

    /// Returns whether a repository discovery is awaiting publication.
    #[must_use]
    pub fn is_repository_opening(&self) -> bool {
        self.repository_open.is_running()
    }

    /// Cancels discovery without changing the accepted repository or session.
    pub fn cancel_repository_open(&mut self) {
        self.repository_open.stop();
        self.pending_repository = None;
        self.repository_error = None;
    }

    /// Publishes the latest successful root and inventory, then applies its launch intent.
    pub fn poll_repository(&mut self, ctx: &Context) {
        match self.repository_open.poll() {
            Ok(Some(Ok(repository))) => {
                if let Some(request) = self.pending_repository.take() {
                    if repository.filters == self.session.source_filters {
                        crate::ui::commands::close_context_menu(ctx);
                        self.accept_repository(repository, request);
                    } else {
                        self.request_repository(request);
                    }
                }
            }
            Ok(Some(Err(error))) => self.repository_error = Some(error),
            Err(_) => {
                self.repository_error = Some("Repository discovery stopped unexpectedly".into())
            }
            Ok(None) => {}
        }
        if self.is_repository_opening() {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }

    fn accept_repository(&mut self, repository: DiscoveredRepository, request: RepositoryRequest) {
        let DiscoveredRepository {
            root, files, tree, ..
        } = repository;
        let destination = match request.destination {
            RepositoryDestination::RestoreCurrent => (self.repository_root.as_ref() == Some(&root))
                .then(|| self.checkout_refresh_destination())
                .flatten()
                .map(|(location, target)| RepositoryDestination::Source { location, target })
                .unwrap_or(RepositoryDestination::Default),
            destination => destination,
        };
        self.save_bookmarks();
        self.reset_repository_state();
        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repository")
            .to_string();

        self.repository_root = Some(root.clone());
        self.recent_pull_requests.open(root.clone());
        self.critical_marks.open(root.clone());
        self.repository_name = Some(name);
        self.session.active_pr = None;
        self.session.record_repository(root.to_string_lossy());

        self.all_files = files;
        self.file_tree = tree;
        self.load_bookmarks();

        self.load_repository_index();
        self.inspect_index_projects();

        match destination {
            RepositoryDestination::Review { number, file } => {
                self.pending_open_file = file;
                self.open_pull_request(number);
            }
            RepositoryDestination::Source { location, target } => self.request_navigation(
                location,
                target,
                source::NavigationIntent {
                    record_history: true,
                    reveal: true,
                    ..Default::default()
                },
            ),
            RepositoryDestination::Default | RepositoryDestination::RestoreCurrent => {
                let first = self
                    .all_files
                    .iter()
                    .find(|file| file.path.eq_ignore_ascii_case("readme.md"))
                    .or_else(|| self.all_files.iter().find(|file| file.is_source))
                    .or_else(|| self.all_files.first())
                    .map(|file| file.path.clone());
                if let Some(path) = first {
                    self.open_file(&path);
                }
            }
        }

        self.refresh_git_changes();
    }

    pub(super) fn show_repository_status(&mut self, ctx: &Context) {
        if self.is_repository_opening() {
            TopBottomPanel::top("repository_open").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    let label = ui.label("Opening repository…");
                    if let Some(request) = &self.pending_repository {
                        label.on_hover_text(request.path.display().to_string());
                    }
                    if ui.button("Cancel").clicked() {
                        self.cancel_repository_open();
                    }
                });
            });
        } else if let Some(error) = self.repository_error.clone() {
            TopBottomPanel::top("repository_open_error").show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("Unable to open repository: {error}"));
                    if ui.button("Retry").clicked() {
                        if let Some(request) = self.pending_repository.clone() {
                            self.request_repository(request);
                        }
                    }
                    if ui.button("Dismiss").clicked() {
                        self.cancel_repository_open();
                    }
                });
            });
        }
    }
}
