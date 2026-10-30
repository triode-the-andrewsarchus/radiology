//! Replaceable background counts for source function folds, separate from inspector UI.

use super::{
    graph::check_cancel,
    worker::{CallGraphWorker, GraphInputs},
    CountAnnotation,
};
use crate::{background::LatestRead, source::SourceBuffer, syntax::FoldRange};
use std::{
    collections::BTreeMap,
    sync::{atomic::AtomicBool, Arc},
};

/// Count and coverage detail for one source function's fold lane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoldAnnotation {
    /// Proven count, partial lower bound, or no-body marker.
    pub count: CountAnnotation,
    /// Full coverage explanation for tooltip/accessibility presentation.
    pub detail: String,
}

/// Immutable exact-source count publication; cannot annotate another revision's bytes.
pub struct SourceAnnotations {
    buffer: Arc<SourceBuffer>,
    entries: BTreeMap<usize, FoldAnnotation>,
}

impl SourceAnnotations {
    /// Returns whether these counts describe the exact displayed source/revision.
    #[must_use]
    pub fn matches(&self, buffer: &SourceBuffer) -> bool {
        self.buffer.path == buffer.path
            && self.buffer.revision == buffer.revision
            && self.buffer.content_digest() == buffer.content_digest()
    }

    /// Returns a callable fold count at its original one-based header line.
    #[must_use]
    pub fn at_line(&self, buffer: &SourceBuffer, line: usize) -> Option<&FoldAnnotation> {
        self.matches(buffer)
            .then(|| self.entries.get(&line))
            .flatten()
    }
}

/// Owns source-count requests without moving caret, selection or navigation history.
#[derive(Default)]
pub struct CriticalAnnotations {
    job: LatestRead<Result<SourceAnnotations, String>>,
    current: Option<SourceAnnotations>,
    error: Option<String>,
}

impl CriticalAnnotations {
    /// Schedules a fold-only sweep on the existing bounded read-worker pool.
    pub fn start(&mut self, inputs: GraphInputs, folds: Vec<FoldRange>) {
        self.stop();
        self.job
            .spawn(move |cancel| compute(inputs, &folds, cancel));
    }

    /// Retires pending work and all publications when any captured input changes.
    pub fn stop(&mut self) {
        self.job.stop();
        self.current = None;
        self.error = None;
    }

    /// Returns whether current source counts are awaiting publication.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.job.is_running()
    }

    /// Returns current counts only for the exact displayed bytes and revision.
    #[must_use]
    pub fn current(&self, buffer: &SourceBuffer) -> Option<&SourceAnnotations> {
        self.current
            .as_ref()
            .filter(|annotations| annotations.matches(buffer))
    }

    /// Returns a recoverable request failure independent of source loading.
    #[must_use]
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Publishes the current completed request, ignoring retired generations.
    pub fn poll(&mut self) {
        match self.job.poll() {
            Ok(Some(Ok(annotations))) => self.current = Some(annotations),
            Ok(Some(Err(error))) => self.error = Some(error),
            Err(_) => self.error = Some("Critical count worker stopped unexpectedly".into()),
            Ok(None) => {}
        }
    }
}

fn compute(
    inputs: GraphInputs,
    folds: &[FoldRange],
    cancel: &AtomicBool,
) -> Result<SourceAnnotations, String> {
    check_cancel(cancel)?;
    let buffer = inputs.buffer.clone();
    let mut scopes = BTreeMap::new();
    for (i, scope) in inputs
        .calls
        .scopes
        .iter()
        .enumerate()
        .filter(|(_, scope)| scope.has_body && !scope.closure)
    {
        scopes
            .entry(buffer.line_at_byte(scope.range.start))
            .or_insert(i);
    }
    let mut worker = CallGraphWorker::new(inputs);
    let mut entries = BTreeMap::new();
    for fold in folds {
        check_cancel(cancel)?;
        let Some(&scope) = scopes.get(&fold.start_line) else {
            continue;
        };
        let frontier = worker.frontier_in_source(scope, cancel)?;
        entries.insert(
            fold.start_line,
            FoldAnnotation {
                count: frontier.annotation(),
                detail: frontier.status_detail(),
            },
        );
    }
    Ok(SourceAnnotations { buffer, entries })
}

#[cfg(test)]
mod tests;
