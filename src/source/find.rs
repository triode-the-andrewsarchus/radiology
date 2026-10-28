//! Asynchronous exact-byte local find over immutable source.

use super::SourceBuffer;
use reader_text::{FindOptions, MatchSpool};
use std::ops::Range;
use std::sync::{atomic::Ordering, Arc};

/// View-owned local search with complete bounded-memory result storage.
#[derive(Default)]
pub struct SourceFind {
    /// Literal/regex and case options currently displayed by the host.
    pub options: FindOptions,
    /// Whether a query is still being evaluated outside the UI thread.
    pub busy: bool,
    /// Regex or storage error for the current query.
    pub error: Option<String>,
    matches: Option<MatchSpool>,
    active: Option<u64>,
    job: crate::background::LatestRead<Result<MatchSpool, String>>,
}

impl SourceFind {
    /// Starts local find on immutable bytes and retires older completions.
    pub fn start(&mut self, source: Arc<SourceBuffer>) {
        self.stop();
        self.matches = None;
        self.active = None;
        self.error = None;
        if self.options.query.is_empty() {
            return;
        }
        let options = self.options.clone();
        self.busy = true;
        self.job.spawn(move |cancel| {
            MatchSpool::build(&source.text, &options, || cancel.load(Ordering::Relaxed))
        });
    }

    /// Retires pending work without letting its result enter another document.
    pub fn stop(&mut self) {
        self.job.stop();
        self.busy = false;
    }

    /// Publishes one completion. Returns true when new results are ready to reveal.
    pub fn poll(&mut self) -> bool {
        match self.job.poll() {
            Ok(Some(result)) => {
                self.busy = false;
                match result {
                    Ok(matches) => {
                        self.matches = Some(matches);
                        return true;
                    }
                    Err(error) => self.error = Some(error),
                }
            }
            Ok(None) => {}
            Err(_) => {
                self.busy = false;
                self.error = Some("Local find worker stopped unexpectedly".into());
            }
        }
        false
    }

    /// Returns the complete number of local matches.
    #[must_use]
    pub fn count(&self) -> u64 {
        self.matches.as_ref().map_or(0, MatchSpool::len)
    }

    /// Returns the active one-based match number, or zero before selection.
    #[must_use]
    pub fn active_number(&self) -> u64 {
        self.active.map_or(0, |index| index + 1)
    }

    /// Selects the next or previous result, wrapping at the ends.
    pub fn step(&mut self, forward: bool) -> Option<Range<usize>> {
        let count = self.count();
        if count == 0 {
            return None;
        }
        let index = match (self.active, forward) {
            (None, true) => 0,
            (None, false) => count - 1,
            (Some(index), true) => (index + 1) % count,
            (Some(index), false) => (index + count - 1) % count,
        };
        self.active = Some(index);
        let result = self.matches.as_mut()?.get(index);
        self.read_result(result)
    }

    /// Returns matches intersecting one bounded display fragment.
    pub fn in_fragment(&mut self, fragment: Range<usize>) -> Vec<(u64, Range<usize>)> {
        let Some(matches) = &mut self.matches else {
            return Vec::new();
        };
        let result = matches.in_fragment(fragment);
        self.read_result(result).unwrap_or_default()
    }

    fn read_result<T>(&mut self, result: std::io::Result<T>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(error) => {
                self.error = Some(format!("Cannot read local find results: {error}"));
                self.matches = None;
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn finish(find: &mut SourceFind) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while find.busy {
            find.poll();
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn local_find_replaces_stale_queries_and_wraps_original_byte_matches() {
        let source = Arc::new(SourceBuffer::from_str("unicode.rs", "é😀\r\nfirst first"));
        let mut find = SourceFind::default();
        find.options.query = "é".into();
        find.start(source.clone());
        find.options.query = "first".into();
        find.start(source);
        finish(&mut find);
        assert!(find.error.is_none());
        assert_eq!(find.count(), 2);
        assert_eq!(find.step(true), Some(8..13));
        assert_eq!(find.step(true), Some(14..19));
        assert_eq!(find.step(true), Some(8..13));
        assert_eq!(find.step(false), Some(14..19));
    }
}
