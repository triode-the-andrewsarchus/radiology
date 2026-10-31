//! Replaceable background projection ownership for standalone and embedded readers.

use super::{CriticalityOverrides, SourceProjection};
use crate::background::LatestRead;
use crate::semantic::SemanticIndex;
use crate::source::SourceBuffer;
use crate::syntax::calls::CallSites;
use std::sync::{Arc, Mutex};

mod cache;

/// Owns one current Critical Lens request and its exact-source publication.
#[derive(Default)]
pub struct CriticalLens {
    job: LatestRead<Result<SourceProjection, String>>,
    projection: Option<SourceProjection>,
    error: Option<String>,
    cache: Arc<Mutex<cache::ProjectionCache>>,
}

impl CriticalLens {
    /// Computes or reuses an exact projection off-thread, retiring older publications.
    pub fn start(
        &mut self,
        buffer: Arc<SourceBuffer>,
        calls: Arc<CallSites>,
        index: Option<Arc<SemanticIndex>>,
        overrides: CriticalityOverrides,
    ) {
        self.projection = None;
        self.error = None;
        let cache = self.cache.clone();
        self.job.spawn(move |cancel| {
            cache::project(&cache, &buffer, calls, index.as_ref(), overrides, cancel)
        });
    }

    /// Retires pending work and presentation, retaining bounded facts for revisits.
    pub fn stop(&mut self) {
        self.job.stop();
        self.projection = None;
        self.error = None;
    }

    /// Retires presentation and memoized facts when a repository or index scope closes.
    pub fn clear(&mut self) {
        self.stop();
        self.cache = Default::default();
    }

    #[cfg(test)]
    pub(crate) fn cache_counts(&self) -> (usize, usize) {
        let cache = self.cache.lock().unwrap();
        (cache.builds, cache.hits)
    }

    /// Returns whether a projection is awaiting publication.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.job.is_running()
    }

    /// Returns the last computation failure.
    #[must_use]
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Returns a published projection only for the exact displayed source.
    #[must_use]
    pub fn projection(&self, buffer: &SourceBuffer) -> Option<&SourceProjection> {
        self.projection
            .as_ref()
            .filter(|projection| projection.matches(buffer))
    }

    /// Publishes the current completion and returns a newly observed failure once.
    pub fn poll(&mut self) -> Option<String> {
        match self.job.poll() {
            Ok(Some(Ok(projection))) => {
                self.projection = Some(projection);
                None
            }
            Ok(Some(Err(error))) => {
                self.error = Some(error.clone());
                Some(error)
            }
            Ok(None) => None,
            Err(_) => {
                let error = "Critical Lens worker stopped unexpectedly".to_owned();
                self.error = Some(error.clone());
                Some(error)
            }
        }
    }
}

#[cfg(test)]
mod tests;
