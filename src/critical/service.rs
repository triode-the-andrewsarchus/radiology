//! Replaceable background projection ownership for standalone and embedded readers.

use super::{CriticalityOverrides, CriticalityPolicy, SourceProjection};
use crate::background::LatestRead;
use crate::semantic::SemanticIndex;
use crate::source::SourceBuffer;
use crate::syntax::calls::CallSites;
use std::sync::Arc;

/// Owns one current Critical Lens request and its exact-source publication.
#[derive(Default)]
pub struct CriticalLens {
    job: LatestRead<Result<SourceProjection, String>>,
    projection: Option<SourceProjection>,
    error: Option<String>,
}

impl CriticalLens {
    /// Computes a projection off-thread and retires all older publications.
    pub fn start(
        &mut self,
        buffer: Arc<SourceBuffer>,
        calls: Arc<CallSites>,
        index: Option<Arc<SemanticIndex>>,
        overrides: CriticalityOverrides,
    ) {
        self.projection = None;
        self.error = None;
        self.job.spawn(move |cancel| {
            let policy = CriticalityPolicy::for_index(index.as_deref(), overrides);
            super::projection::make(&buffer, &calls, index.as_deref(), &policy, cancel)
        });
    }

    /// Retires pending work and clears presentation when the mode/repository closes.
    pub fn stop(&mut self) {
        self.job.stop();
        self.projection = None;
        self.error = None;
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
