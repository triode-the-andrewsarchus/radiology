//! Cancellable repository search state usable without an egui host.

use super::{SearchMatch, SearchOptions, SearchRunner};
use crate::{background::LatestRead, pull_request::review::ReviewSnapshot};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

enum SearchEvent {
    Matches(Vec<SearchMatch>),
    Finished(Result<bool, String>),
}

struct SearchRequest {
    root: PathBuf,
    options: SearchOptions,
    // Keep the private tree leased through debounce and worker retirement.
    review: Option<Arc<ReviewSnapshot>>,
}

/// Owns one search request; replacing it retires its publication channel.
#[derive(Default)]
pub struct RepositorySearch {
    /// Matches from the current request only.
    pub matches: Vec<SearchMatch>,
    /// Whether its ripgrep worker is still running.
    pub running: bool,
    /// Whether the current result list reached its configured bound.
    pub limited: bool,
    /// Current query/tool error, independently of source-reader errors.
    pub error: Option<String>,
    job: LatestRead<SearchEvent>,
    scheduled: Option<(SearchRequest, Instant)>,
}

impl RepositorySearch {
    /// Replaces the pending checkout query after a short typing debounce.
    pub fn schedule(&mut self, root: &Path, options: SearchOptions) {
        self.schedule_request(SearchRequest {
            root: root.into(),
            options,
            review: None,
        });
    }

    pub(crate) fn schedule_review(&mut self, review: Arc<ReviewSnapshot>, options: SearchOptions) {
        self.schedule_request(SearchRequest {
            root: review.head_root.clone(),
            options,
            review: Some(review),
        });
    }

    fn schedule_request(&mut self, request: SearchRequest) {
        self.stop();
        self.scheduled = Some((request, Instant::now() + Duration::from_millis(150)));
    }

    /// Starts a checkout request immediately and retires superseded publication.
    pub fn start(&mut self, root: &Path, options: SearchOptions) {
        self.start_request(SearchRequest {
            root: root.into(),
            options,
            review: None,
        });
    }

    pub(crate) fn start_review(&mut self, review: Arc<ReviewSnapshot>, options: SearchOptions) {
        self.start_request(SearchRequest {
            root: review.head_root.clone(),
            options,
            review: Some(review),
        });
    }

    fn start_request(&mut self, request: SearchRequest) {
        self.stop();
        self.matches.clear();
        self.error = None;
        self.limited = false;
        if request.options.query.trim().is_empty() {
            return;
        }
        self.running = true;
        self.job.spawn_with_sender(move |cancel, sender| {
            let SearchRequest {
                root,
                options,
                review: _review,
            } = request;
            let result = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| error.to_string())
                .and_then(|runtime| {
                    runtime.block_on(async {
                        SearchRunner::search_cancellable(&root, &options, cancel, |matches| {
                            let _ = sender.send(SearchEvent::Matches(matches));
                        })
                        .await
                        .map_err(|error| error.to_string())
                    })
                });
            let _ = sender.send(SearchEvent::Finished(result));
        });
    }

    /// Cancels work and retires its receiver while keeping already published matches.
    pub fn stop(&mut self) {
        self.job.stop();
        self.scheduled = None;
        self.running = false;
    }

    /// Publishes all queued batches without consuming a batch just to check liveness.
    /// Returns whether the host should poll again soon.
    pub fn poll(&mut self) -> bool {
        if self
            .scheduled
            .as_ref()
            .is_some_and(|(_, when)| Instant::now() >= *when)
        {
            let (request, _) = self.scheduled.take().expect("scheduled request");
            self.start_request(request);
        }
        while self.running {
            match self.job.poll_next() {
                Ok(Some(SearchEvent::Matches(matches))) => self.matches.extend(matches),
                Ok(Some(SearchEvent::Finished(result))) => {
                    self.running = false;
                    self.job.stop();
                    match result {
                        Ok(limited) => self.limited = limited,
                        Err(error) => self.error = Some(error),
                    }
                }
                Ok(None) => break,
                Err(_) => {
                    self.running = false;
                    self.job.stop();
                    self.error = Some("Search worker stopped unexpectedly".into());
                }
            }
        }
        self.running || self.scheduled.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finish(search: &mut RepositorySearch) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while search.poll() {
            assert!(Instant::now() < deadline, "search did not finish");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn real_search_reports_glob_matches_and_regex_errors() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "needle\nneedle\n").unwrap();
        std::fs::write(dir.path().join("b.txt"), "needle\n").unwrap();
        let mut search = RepositorySearch::default();
        search.start(
            dir.path(),
            SearchOptions {
                query: "needle".into(),
                include_glob: "*.rs".into(),
                ..Default::default()
            },
        );
        finish(&mut search);
        assert!(search.error.is_none(), "{:?}", search.error);
        assert_eq!(search.matches.len(), 2);
        search.start(
            dir.path(),
            SearchOptions {
                query: "[".into(),
                is_regex: true,
                ..Default::default()
            },
        );
        finish(&mut search);
        assert!(search.error.is_some());
        assert!(search.matches.is_empty());
    }

    #[test]
    fn review_debounce_retains_the_snapshot_then_searches_pinned_bytes_and_releases_it() {
        let (_owner, root, snapshot) = crate::pull_request::review::tests::fixture();
        let snapshot = Arc::new(snapshot);
        let weak = Arc::downgrade(&snapshot);
        let mut search = RepositorySearch::default();
        let options = SearchOptions {
            query: "changed".into(),
            ..Default::default()
        };
        search.schedule_review(snapshot, options.clone());
        assert!(weak.upgrade().is_some(), "debounced work owns its tree");
        finish(&mut search);
        assert!(search.error.is_none(), "{:?}", search.error);
        assert_eq!(search.matches.len(), 1);
        assert_eq!(search.matches[0].path, "a file.rs");
        let deadline = Instant::now() + Duration::from_secs(5);
        while weak.upgrade().is_some() {
            assert!(
                Instant::now() < deadline,
                "finished worker retained its snapshot"
            );
            std::thread::yield_now();
        }
        assert_eq!(
            std::fs::read_to_string(root.join("a file.rs")).unwrap(),
            "checkout only\n"
        );

        let (_other, _, snapshot) = crate::pull_request::review::tests::fixture();
        let snapshot = Arc::new(snapshot);
        let weak = Arc::downgrade(&snapshot);
        search.schedule_review(snapshot, options);
        search.stop();
        assert!(
            weak.upgrade().is_none(),
            "retired debounce releases its lease"
        );
    }

    #[test]
    fn replacement_cannot_publish_a_retired_request() {
        use std::sync::{atomic::Ordering, mpsc};
        let (started, ready) = mpsc::channel();
        let (resume, resumed) = mpsc::channel();
        let (finished, result) = mpsc::channel();
        let mut search = RepositorySearch {
            running: true,
            ..Default::default()
        };
        search.job.spawn_with_sender(move |cancel, sender| {
            started.send(()).unwrap();
            resumed.recv_timeout(Duration::from_secs(5)).unwrap();
            finished
                .send((
                    cancel.load(Ordering::Relaxed),
                    sender.send(SearchEvent::Finished(Ok(true))).is_err(),
                ))
                .unwrap();
        });
        ready.recv_timeout(Duration::from_secs(5)).unwrap();
        search.stop();
        resume.send(()).unwrap();
        assert_eq!(
            result.recv_timeout(Duration::from_secs(5)).unwrap(),
            (true, true)
        );
        assert!(!search.poll());
        assert!(!search.limited);
    }
}
