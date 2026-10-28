//! Cancellable repository search state usable without an egui host.

use super::{SearchMatch, SearchOptions, SearchRunner};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::time::{Duration, Instant};

enum SearchEvent {
    Matches(Vec<SearchMatch>),
    Finished(Result<bool, String>),
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
    cancel: Arc<AtomicBool>,
    receiver: Option<mpsc::Receiver<SearchEvent>>,
    scheduled: Option<(PathBuf, SearchOptions, Instant)>,
}

impl RepositorySearch {
    /// Replaces the pending query after a short typing debounce.
    pub fn schedule(&mut self, root: &Path, options: SearchOptions) {
        self.stop();
        self.scheduled = Some((
            root.to_path_buf(),
            options,
            Instant::now() + Duration::from_millis(150),
        ));
    }

    /// Starts a request immediately and prevents superseded results from publishing.
    pub fn start(&mut self, root: &Path, options: SearchOptions) {
        self.stop();
        self.matches.clear();
        self.error = None;
        self.limited = false;
        if options.query.trim().is_empty() {
            return;
        }
        self.running = true;
        self.cancel = Arc::new(AtomicBool::new(false));
        let cancel = self.cancel.clone();
        let root = root.to_path_buf();
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        std::thread::spawn(move || {
            let result = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| error.to_string())
                .and_then(|runtime| {
                    runtime.block_on(async {
                        SearchRunner::search(&root, &options, cancel, |matches| {
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
        self.cancel.store(true, Ordering::Relaxed);
        self.receiver = None;
        self.scheduled = None;
        self.running = false;
    }

    /// Publishes all queued batches without consuming a batch just to check liveness.
    /// Returns whether the host should poll again soon.
    pub fn poll(&mut self) -> bool {
        if self
            .scheduled
            .as_ref()
            .is_some_and(|(_, _, when)| Instant::now() >= *when)
        {
            let (root, options, _) = self.scheduled.take().expect("scheduled request");
            self.start(&root, options);
        }
        while let Some(receiver) = &self.receiver {
            match receiver.try_recv() {
                Ok(SearchEvent::Matches(matches)) => self.matches.extend(matches),
                Ok(SearchEvent::Finished(result)) => {
                    self.running = false;
                    self.receiver = None;
                    match result {
                        Ok(limited) => self.limited = limited,
                        Err(error) => self.error = Some(error),
                    }
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.running = false;
                    self.receiver = None;
                    self.error = Some("Search worker stopped unexpectedly".into());
                }
            }
        }
        self.running || self.scheduled.is_some()
    }
}

impl Drop for RepositorySearch {
    fn drop(&mut self) {
        self.stop();
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
    fn replacement_cannot_publish_a_retired_request() {
        let (sender, receiver) = mpsc::channel();
        let mut search = RepositorySearch {
            receiver: Some(receiver),
            running: true,
            matches: Vec::new(),
            limited: false,
            error: None,
            cancel: Arc::new(AtomicBool::new(false)),
            scheduled: None,
        };
        let retired_cancel = search.cancel.clone();
        search.stop();
        assert!(retired_cancel.load(Ordering::Relaxed));
        assert!(sender.send(SearchEvent::Finished(Ok(true))).is_err());
        assert!(!search.poll());
        assert!(!search.limited);
    }
}
