//! Independent source excerpts and validated caller jobs for expanded references.

use super::{
    incoming_calls::{self, CallerLimits, CallerTree},
    reference_results::ReferenceHit,
};
use crate::{
    background::LatestRead,
    source::preview::{
        PreviewKind, PreviewPhase, PreviewTarget, SourcePreviewLoader, SourceQueryInputs,
    },
};
use std::sync::Arc;

/// Owns one reference disclosure, including both replaceable background reads.
///
/// Dropping the disclosure retires all publication channels. Larger caller
/// requests retain the previous tree while preparing its replacement.
pub struct ReferenceDisclosure {
    preview: SourcePreviewLoader,
    hit: ReferenceHit,
    digest: String,
    job: LatestRead<Result<CallerTree, String>>,
    limits: CallerLimits,
    tree: Option<CallerTree>,
    error: Option<String>,
}

impl ReferenceDisclosure {
    /// Returns the independently prepared source excerpt.
    #[must_use]
    pub fn phase(&self) -> &PreviewPhase {
        self.preview.phase()
    }

    /// Returns the last accepted caller tree, including during expansion.
    #[must_use]
    pub fn callers(&self) -> Option<&CallerTree> {
        self.tree.as_ref()
    }

    /// Returns a recoverable caller-trace failure.
    #[must_use]
    pub fn caller_error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Returns whether a replacement caller tree is being prepared.
    #[must_use]
    pub fn callers_loading(&self) -> bool {
        self.job.is_running()
    }

    pub(crate) fn new(inputs: Arc<SourceQueryInputs>, hit: ReferenceHit, digest: String) -> Self {
        let mut disclosure = Self {
            preview: Default::default(),
            hit,
            digest,
            job: Default::default(),
            limits: Default::default(),
            tree: None,
            error: None,
        };
        disclosure.request_preview(inputs.clone());
        disclosure.start_trace(inputs);
        disclosure
    }

    fn request_preview(&mut self, inputs: Arc<SourceQueryInputs>) {
        self.preview.request(
            Some(inputs),
            Some(PreviewTarget {
                path: self.hit.path.clone(),
                range: self.hit.range.clone(),
                kind: PreviewKind::Reference,
            }),
        );
    }

    fn start_trace(&mut self, inputs: Arc<SourceQueryInputs>) {
        let (hit, digest, limits) = (self.hit.clone(), self.digest.clone(), self.limits);
        self.error = None;
        self.job
            .spawn(move |cancel| incoming_calls::trace(&inputs, &hit, &digest, limits, cancel));
    }

    pub(crate) fn rebind_pending(&mut self, inputs: Arc<SourceQueryInputs>) {
        if self.preview.is_running() {
            self.request_preview(inputs.clone());
        }
        if self.job.is_running() {
            self.start_trace(inputs);
        }
    }

    pub(crate) fn more_callers(&mut self, inputs: Arc<SourceQueryInputs>) {
        if self.tree.as_ref().is_some_and(|tree| tree.has_more) && !self.job.is_running() {
            self.limits = self.limits.expanded();
            self.start_trace(inputs);
        }
    }

    pub(crate) fn is_running(&self) -> bool {
        self.preview.is_running() || self.job.is_running()
    }

    pub(crate) fn poll(&mut self) -> bool {
        let preview_changed = self.preview.poll();
        match self.job.poll() {
            Ok(None) => preview_changed,
            Ok(Some(Ok(tree))) => {
                self.tree = Some(tree);
                true
            }
            Ok(Some(Err(error))) => {
                self.error = Some(error);
                true
            }
            Err(_) => {
                self.error = Some("Caller tracing stopped unexpectedly.".into());
                true
            }
        }
    }
}
