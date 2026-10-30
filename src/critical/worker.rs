//! Snapshot-bound call graph evaluation for background reader jobs.
//!
//! One worker owns one immutable source/index/policy configuration. Its caches
//! cannot survive a host reconfiguration. The displayed bytes prime its file
//! cache before any disk read; other files use the captured source storage.

use super::{
    frontier::{self, CriticalFrontier, FrontierLimits},
    graph::{self, CallableSummary, NoBodyReason},
    CriticalityOverrides, CriticalityPolicy,
};
use crate::{
    semantic::{index::SemanticSymbolID, SemanticIndex},
    source::{
        loader::{check_source, SourceRead, SOURCE_READ_TIMEOUT},
        SourceBuffer, SourceLocation,
    },
    syntax::{calls::CallSites, DocumentSyntax},
};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
    time::Instant,
};

const SOURCE_BUDGET: usize = 64 * 1024 * 1024;
const SUMMARY_CACHE_LIMIT: usize = 512;
const FILE_CACHE_LIMIT: usize = 512;

/// Captured inputs shared by source annotation and hover requests.
#[derive(Clone)]
pub struct GraphInputs {
    /// Storage for other repository files, including pinned PR objects.
    pub source: SourceRead,
    /// Immutable displayed source, preferred over the underlying storage.
    pub buffer: Arc<SourceBuffer>,
    /// Call sites/scopes from the displayed source's existing parse.
    pub calls: Arc<CallSites>,
    /// Immutable index snapshot whose source coordinates will be validated.
    pub index: Arc<SemanticIndex>,
    /// Captured repository policy decisions.
    pub overrides: CriticalityOverrides,
}

struct GraphFile {
    buffer: Arc<SourceBuffer>,
    calls: Arc<CallSites>,
}

/// Evaluates frontiers with request-local immutable file and summary caches.
///
/// Construction performs no I/O. Evaluation belongs on a background worker;
/// hosts retire the complete instance on source, storage, index or policy changes.
/// Additional source bytes are capped at 64 MiB; exhausted requests report a
/// truncated lower bound. Reads and parses are checked for cancellation before
/// and after their bounded native operation.
pub struct CallGraphWorker {
    source: SourceRead,
    index: Arc<SemanticIndex>,
    policy: CriticalityPolicy,
    primed: Arc<GraphFile>,
    files: HashMap<String, Result<Arc<GraphFile>, NoBodyReason>>,
    summaries: HashMap<SemanticSymbolID, CallableSummary>,
    source_bytes: usize,
    source_budget: usize,
}

impl CallGraphWorker {
    /// Captures a configuration and primes the displayed bytes without reading files.
    #[must_use]
    pub fn new(inputs: GraphInputs) -> Self {
        let policy = CriticalityPolicy::for_index(Some(&inputs.index), inputs.overrides);
        Self {
            source: inputs.source,
            index: inputs.index,
            policy,
            primed: Arc::new(GraphFile {
                buffer: inputs.buffer,
                calls: inputs.calls,
            }),
            files: HashMap::new(),
            summaries: HashMap::new(),
            source_bytes: 0,
            source_budget: SOURCE_BUDGET,
        }
    }

    /// Computes the frontier of an exact callable scope in the displayed source.
    ///
    /// # Errors
    /// Returns cancellation or an expired source-read budget without publishing
    /// a partially evaluated frontier.
    pub fn frontier_in_source(
        &mut self,
        scope: usize,
        cancel: &AtomicBool,
    ) -> Result<CriticalFrontier, String> {
        let deadline = Instant::now() + SOURCE_READ_TIMEOUT;
        let root = self.summary_in_source(scope, cancel)?;
        frontier::build(root, FrontierLimits::default(), cancel, |id| {
            self.summary_of(id, cancel, deadline)
        })
    }

    /// Computes a callable frontier, loading its canonical definition when needed.
    ///
    /// # Errors
    /// Returns cancellation or an expired source-read budget without publishing
    /// a partially evaluated frontier.
    pub fn frontier_of(
        &mut self,
        id: &SemanticSymbolID,
        cancel: &AtomicBool,
    ) -> Result<CriticalFrontier, String> {
        let deadline = Instant::now() + SOURCE_READ_TIMEOUT;
        let root = self.summary_of(id, cancel, deadline)?;
        frontier::build(root, FrontierLimits::default(), cancel, |id| {
            self.summary_of(id, cancel, deadline)
        })
    }

    fn summary_in_source(
        &self,
        scope: usize,
        cancel: &AtomicBool,
    ) -> Result<CallableSummary, String> {
        let file = &self.primed;
        let callable = file
            .calls
            .scopes
            .get(scope)
            .and_then(|scope| scope.name.as_ref())
            .and_then(|name| self.index.occurrence_at_byte(&file.buffer, name.start))
            .filter(|occurrence| occurrence.is_definition())
            .map(|occurrence| SemanticSymbolID::new(&occurrence.symbol, &file.buffer.path));
        self.summarize(file, scope, callable, cancel)
    }

    fn summary_of(
        &mut self,
        id: &SemanticSymbolID,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> Result<CallableSummary, String> {
        check_source(cancel, deadline)?;
        if let Some(summary) = self.summaries.get(id) {
            return Ok(summary.clone());
        }
        let summary = self.build_summary(id, cancel, deadline)?;
        check_source(cancel, deadline)?;
        if self.summaries.len() < SUMMARY_CACHE_LIMIT {
            self.summaries.insert(id.clone(), summary.clone());
        }
        Ok(summary)
    }

    fn build_summary(
        &mut self,
        id: &SemanticSymbolID,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> Result<CallableSummary, String> {
        let no_body = |reason| Ok(CallableSummary::no_body(reason, Some(id.clone())));
        let Some((path, range)) = self.index.find_definition_for_id(id) else {
            return no_body(NoBodyReason::External);
        };
        let (path, range) = (path.to_owned(), range.clone());
        let loaded = self.load(&path, cancel, deadline);
        check_source(cancel, deadline)?;
        let file = match loaded {
            Ok(file) => file,
            Err(reason) => return no_body(reason),
        };
        let Some(definition) = self.index.byte_range(&file.buffer, &range) else {
            return no_body(NoBodyReason::NotIndexed);
        };
        let Some(scope) = file.calls.scopes.iter().position(|scope| {
            scope
                .name
                .as_ref()
                .is_some_and(|name| name.start == definition.start)
        }) else {
            return no_body(if file.calls.supported {
                NoBodyReason::Abstract
            } else {
                NoBodyReason::UnsupportedLanguage
            });
        };
        self.summarize(&file, scope, Some(id.clone()), cancel)
    }

    fn summarize(
        &self,
        file: &GraphFile,
        scope: usize,
        id: Option<SemanticSymbolID>,
        cancel: &AtomicBool,
    ) -> Result<CallableSummary, String> {
        graph::summarize(
            &file.buffer,
            &file.calls,
            scope,
            id,
            Some(&self.index),
            &self.policy,
            cancel,
        )
    }

    fn load(
        &mut self,
        path: &str,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> Result<Arc<GraphFile>, NoBodyReason> {
        if path == self.primed.buffer.path {
            return Ok(self.primed.clone());
        }
        if let Some(file) = self.files.get(path) {
            return file.clone();
        }
        if self.files.len() >= FILE_CACHE_LIMIT {
            return Err(NoBodyReason::ResourceLimit);
        }
        let loaded = self.read(path, cancel, deadline);
        if check_source(cancel, deadline).is_ok() {
            self.files.insert(path.to_owned(), loaded.clone());
        }
        loaded
    }

    fn read(
        &mut self,
        path: &str,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> Result<Arc<GraphFile>, NoBodyReason> {
        if self.source_bytes >= self.source_budget {
            return Err(NoBodyReason::ResourceLimit);
        }
        let location = SourceLocation::with_revision(path, 0, self.primed.buffer.revision);
        let buffer = self
            .source
            .read_cancellable(&location, cancel, deadline)
            .map_err(|_| NoBodyReason::Unreadable)?;
        if buffer.byte_len() > self.source_budget.saturating_sub(self.source_bytes) {
            return Err(NoBodyReason::ResourceLimit);
        }
        self.source_bytes += buffer.byte_len();
        if !self.index.matches_buffer(&buffer) {
            return Err(NoBodyReason::NotIndexed);
        }
        let syntax = DocumentSyntax::analyze_cancellable(&buffer, || {
            check_source(cancel, deadline).is_err()
        })
        .map_err(|_| NoBodyReason::Unreadable)?;
        Ok(Arc::new(GraphFile {
            buffer,
            calls: Arc::new(syntax.calls),
        }))
    }
}

#[cfg(test)]
pub(super) mod tests;
