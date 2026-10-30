//! Repository-owned recent PR lists, independent of GUI rendering.

use super::{
    control::ReviewControl,
    github::{GitHubClient, PullRequestSummary},
};
use crate::background::LatestRead;
use std::path::{Path, PathBuf};

/// Lifecycle of the recent list, independent of opening or refreshing a review.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PullRequestListPhase {
    /// No usable repository origin is available.
    Unavailable(String),
    /// Loading or refreshing while retaining accepted entries.
    Loading,
    /// A complete list, including an empty list, was accepted.
    Loaded,
    /// Refresh failed; previously accepted entries remain available.
    Failed(String),
    /// The owner stopped a request; previously accepted entries remain available.
    Cancelled,
}

/// Owns one replaceable recent-list request and its accepted repository-scoped data.
pub struct RecentPullRequests {
    root: Option<PathBuf>,
    items: Vec<PullRequestSummary>,
    phase: PullRequestListPhase,
    job: LatestRead<Result<Option<Vec<PullRequestSummary>>, String>>,
}

impl Default for RecentPullRequests {
    fn default() -> Self {
        Self {
            root: None,
            items: vec![],
            phase: PullRequestListPhase::Unavailable(
                "Open a repository to list pull requests.".into(),
            ),
            job: Default::default(),
        }
    }
}

impl RecentPullRequests {
    /// Attaches a repository and automatically loads up to 25 recent PRs.
    ///
    /// Reattaching the same root preserves the list, including when exiting a review.
    /// A different root retires old work and clears its accepted data.
    pub fn open(&mut self, root: PathBuf) {
        if self.root.as_ref() == Some(&root) {
            return;
        }
        self.cancel();
        self.root = Some(root);
        self.items.clear();
        self.refresh();
    }

    /// Starts a new bounded refresh without discarding accepted entries.
    pub fn refresh(&mut self) {
        let Some(root) = self.root.clone() else {
            return;
        };
        self.phase = PullRequestListPhase::Loading;
        self.job
            .spawn(move |cancel| Self::load(&root, &ReviewControl::new(cancel)));
    }

    fn load(
        root: &Path,
        control: &ReviewControl<'_>,
    ) -> Result<Option<Vec<PullRequestSummary>>, String> {
        if control
            .git(root)
            .optional(&["remote", "get-url", "origin"])?
            .is_none()
        {
            return Ok(None);
        }
        GitHubClient::list_cancellable(root, 25, control).map(Some)
    }

    /// Retires pending work while retaining accepted list entries.
    pub fn cancel(&mut self) {
        if self.job.is_running() {
            self.phase = PullRequestListPhase::Cancelled;
        }
        self.job.stop();
    }

    /// Publishes the latest completed list; hosts call this from their event loop.
    pub fn poll(&mut self) {
        match self.job.poll() {
            Ok(Some(Ok(Some(items)))) => {
                self.items = items;
                self.phase = PullRequestListPhase::Loaded;
            }
            Ok(Some(Ok(None))) => {
                self.phase = PullRequestListPhase::Unavailable(
                    "Pull requests need a Git repository with an origin.".into(),
                )
            }
            Ok(Some(Err(error))) => self.phase = PullRequestListPhase::Failed(error),
            Err(_) => {
                self.phase = PullRequestListPhase::Failed(
                    "Pull request listing stopped unexpectedly.".into(),
                )
            }
            Ok(None) => {}
        }
    }

    /// Updates an existing list entry after a review obtains newer metadata.
    pub fn remember(&mut self, summary: &PullRequestSummary) {
        if let Some(item) = self
            .items
            .iter_mut()
            .find(|item| item.number == summary.number)
        {
            *item = summary.clone();
        }
    }

    /// Returns accepted entries in the order reported by GitHub.
    #[must_use]
    pub fn items(&self) -> &[PullRequestSummary] {
        &self.items
    }

    /// Returns the current list lifecycle, including recoverable errors.
    #[must_use]
    pub fn phase(&self) -> &PullRequestListPhase {
        &self.phase
    }

    /// Returns whether a background list request is awaiting publication.
    #[must_use]
    pub fn is_loading(&self) -> bool {
        self.job.is_running()
    }
}

#[cfg(all(test, unix))]
mod tests;
