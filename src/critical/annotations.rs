//! Replaceable background counts for source function folds, separate from inspector UI.

use super::{
    graph::check_cancel,
    worker::{CallGraphWorker, GraphInputs},
    CountAnnotation,
};
use crate::{background::LatestRead, source::SourceBuffer, syntax::FoldRange};
use std::{
    collections::BTreeMap,
    sync::{atomic::AtomicBool, Arc, Mutex},
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

// A request owns its slot. Retirement drops the reader's handle, so an old
// worker cannot publish into a replacement even when source paths are identical.
#[derive(Clone)]
struct PendingCounts {
    buffer: Arc<SourceBuffer>,
    entries: Arc<Mutex<BTreeMap<usize, FoldAnnotation>>>,
}

impl PendingCounts {
    fn publish(&self, mut batch: BTreeMap<usize, FoldAnnotation>) {
        self.entries
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .append(&mut batch);
    }
}

/// Owns source-count requests without moving caret, selection or navigation history.
#[derive(Default)]
pub struct CriticalAnnotations {
    job: LatestRead<Result<(), String>>,
    pending: Option<PendingCounts>,
    current: Option<SourceAnnotations>,
    error: Option<String>,
}

impl CriticalAnnotations {
    /// Schedules a fold-only sweep on the existing bounded read-worker pool.
    pub fn start(&mut self, inputs: GraphInputs, folds: Vec<FoldRange>) {
        self.stop();
        let pending = PendingCounts {
            buffer: inputs.buffer.clone(),
            entries: Default::default(),
        };
        self.pending = Some(pending.clone());
        self.job
            .spawn(move |cancel| compute(inputs, &folds, cancel, |batch| pending.publish(batch)));
    }

    /// Retires pending work and all publications when any captured input changes.
    pub fn stop(&mut self) {
        self.job.stop();
        self.pending = None;
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

    /// Merges available batches while running and drains the final batch on completion.
    /// Completed function counts remain usable if later analysis fails.
    pub fn poll(&mut self) {
        // Observe completion first: its channel send follows the final slot write.
        // Draining afterward cannot lose a last batch racing this frame's poll.
        let completion = self.job.poll();
        self.take_progress();
        match completion {
            Ok(Some(Ok(()))) => {}
            Ok(Some(Err(error))) => self.error = Some(error),
            Err(_) => self.error = Some("Critical count worker stopped unexpectedly".into()),
            Ok(None) => {}
        }
    }

    fn take_progress(&mut self) {
        let Some(pending) = &self.pending else {
            return;
        };
        let batch = std::mem::take(
            &mut *pending
                .entries
                .lock()
                .unwrap_or_else(|error| error.into_inner()),
        );
        if batch.is_empty() {
            return;
        }
        let current = self.current.get_or_insert_with(|| SourceAnnotations {
            buffer: pending.buffer.clone(),
            entries: BTreeMap::new(),
        });
        current.entries.extend(batch);
    }
}

fn compute(
    inputs: GraphInputs,
    folds: &[FoldRange],
    cancel: &AtomicBool,
    mut publish: impl FnMut(BTreeMap<usize, FoldAnnotation>),
) -> Result<(), String> {
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
        check_cancel(cancel)?;
        if entries.len() >= 6 {
            publish(std::mem::take(&mut entries));
        }
        std::thread::yield_now();
    }
    check_cancel(cancel)?;
    if !entries.is_empty() {
        publish(entries);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
