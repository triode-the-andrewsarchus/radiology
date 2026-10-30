//! Repository mark lifecycle: replaceable loads and coalesced durable writes.

use super::overrides::{MarkEdit, MarkTarget, OverrideStore};
use super::CriticalityOverrides;
use crate::background::{LatestRead, READ_WORKERS};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{atomic::AtomicBool, Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
struct WriteRequest {
    revision: u64,
    base: Option<CriticalityOverrides>,
    edits: Vec<MarkEdit>,
}

impl WriteRequest {
    fn apply(&self, mut loaded: CriticalityOverrides) -> CriticalityOverrides {
        if let Some(base) = &self.base {
            loaded = base.clone();
        }
        for edit in &self.edits {
            edit.apply(&mut loaded);
        }
        loaded
    }
}

#[derive(Default)]
struct WriterState {
    running: bool,
    pending: BTreeMap<PathBuf, WriteRequest>,
    results: HashMap<PathBuf, Result<(), String>>,
}

#[derive(Clone, Default)]
struct Writer(Arc<(Mutex<WriterState>, Condvar)>);

impl Writer {
    fn pending(&self, root: &Path) -> Option<WriteRequest> {
        self.0
             .0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .pending
            .get(root)
            .cloned()
    }

    fn submit(
        &self,
        store: &OverrideStore,
        root: PathBuf,
        base: Option<CriticalityOverrides>,
        edits: Vec<MarkEdit>,
    ) {
        let mut state = self.0 .0.lock().unwrap_or_else(|error| error.into_inner());
        let revision = store.reserve_revision();
        let request = state
            .pending
            .entry(root.clone())
            .or_insert_with(|| WriteRequest {
                revision,
                base: None,
                edits: Vec::new(),
            });
        request.revision = revision;
        if base.is_some() {
            request.base = base;
            request.edits.clear();
        }
        request.edits.extend(edits);
        state.results.remove(&root);
        if state.running {
            return;
        }
        state.running = true;
        let writer = self.clone();
        let store = store.clone();
        READ_WORKERS.spawn_blocking(move || writer.drain(store));
    }

    fn drain(&self, store: OverrideStore) {
        loop {
            let next = {
                let mut state = self.0 .0.lock().unwrap_or_else(|error| error.into_inner());
                match state.pending.first_key_value() {
                    Some((root, request)) => Some((root.clone(), request.clone())),
                    None => {
                        state.running = false;
                        self.0 .1.notify_all();
                        None
                    }
                }
            };
            let Some((root, request)) = next else {
                return;
            };
            let result = self.write(&store, &root, &request);
            let mut state = self.0 .0.lock().unwrap_or_else(|error| error.into_inner());
            if state
                .pending
                .get(&root)
                .is_some_and(|pending| pending.revision == request.revision)
            {
                state.pending.remove(&root);
                state.results.insert(root, result);
            }
        }
    }

    fn write(
        &self,
        store: &OverrideStore,
        root: &Path,
        request: &WriteRequest,
    ) -> Result<(), String> {
        let loaded = if request.base.is_some() {
            Default::default()
        } else {
            store.load(root, &AtomicBool::new(false))?
        };
        let snapshot = request.apply(loaded);
        store.save(root, &snapshot, request.revision).map(|_| ())
    }

    fn status(&self, root: Option<&Path>) -> (bool, Option<String>) {
        let state = self.0 .0.lock().unwrap_or_else(|error| error.into_inner());
        let error = root
            .and_then(|root| state.results.get(root))
            .and_then(|result| result.as_ref().err())
            .cloned();
        (state.running, error)
    }

    fn flush(&self, timeout: Duration) -> Result<(), String> {
        let started = Instant::now();
        let mut state = self.0 .0.lock().unwrap_or_else(|error| error.into_inner());
        while state.running {
            let remaining = timeout.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                return Err("Critical Lens saves did not finish before shutdown deadline".into());
            }
            state = self
                .0
                 .1
                .wait_timeout(state, remaining)
                .unwrap_or_else(|error| error.into_inner())
                .0;
        }
        state
            .results
            .values()
            .find_map(|result| result.as_ref().err())
            .cloned()
            .map_or(Ok(()), Err)
    }
}

/// UI-independent repository decisions with bounded, coalesced persistence work.
///
/// Opening/closing retires read publications, never accepted edits. One shared
/// read-worker writer drains the latest snapshot/delta per root without an event
/// backlog. Hosts should call `flush` during controlled shutdown.
pub struct MarkSession {
    store: Option<OverrideStore>,
    writer: Writer,
    load: LatestRead<Result<CriticalityOverrides, String>>,
    root: Option<PathBuf>,
    overrides: CriticalityOverrides,
    pending_edits: Vec<MarkEdit>,
    load_error: Option<String>,
    save_error: Option<String>,
}

impl MarkSession {
    /// Configures private storage without reading or creating any files.
    #[must_use]
    pub fn new(store: Option<OverrideStore>) -> Self {
        Self {
            store,
            writer: Default::default(),
            load: Default::default(),
            root: None,
            overrides: Default::default(),
            pending_edits: Vec::new(),
            load_error: None,
            save_error: None,
        }
    }

    /// Opens an already canonical attached root and retires the previous read.
    pub fn open(&mut self, root: PathBuf) {
        self.close();
        self.root = Some(root.clone());
        if let Some(store) = self.store.clone() {
            let overlay = self.writer.pending(&root);
            self.load.spawn(move |cancel| {
                store.load(&root, cancel).map(|loaded| match overlay {
                    Some(request) => request.apply(loaded),
                    None => loaded,
                })
            });
        }
    }

    /// Closes presentation while retaining durable writes, including pre-load edits.
    pub fn close(&mut self) {
        if !self.pending_edits.is_empty() {
            if let (Some(store), Some(root)) = (&self.store, &self.root) {
                self.writer.submit(
                    store,
                    root.clone(),
                    None,
                    std::mem::take(&mut self.pending_edits),
                );
            }
        }
        self.load.stop();
        self.root = None;
        self.overrides = Default::default();
        self.pending_edits.clear();
        self.load_error = None;
        self.save_error = None;
    }

    /// Applies a decision immediately; pending loads replay even a no-op reset.
    pub fn edit(&mut self, edit: MarkEdit) -> bool {
        let valid = match &edit.target {
            MarkTarget::Symbol(symbol) => CriticalityOverrides::is_overridable(symbol),
            MarkTarget::Package(package) => !package.is_empty(),
        };
        if self.root.is_none() || !valid {
            return false;
        }
        let changed = edit.apply(&mut self.overrides);
        if self.load.is_running() {
            self.pending_edits.push(edit);
            return changed;
        }
        if changed {
            self.save_snapshot();
        }
        changed
    }

    fn save_snapshot(&mut self) {
        if let (Some(store), Some(root)) = (&self.store, &self.root) {
            self.writer.submit(
                store,
                root.clone(),
                Some(self.overrides.clone()),
                Vec::new(),
            );
        } else {
            self.save_error =
                Some("No private settings directory; marks apply only to this session".into());
        }
    }

    /// Publishes a current load and returns whether policy changed.
    pub fn poll(&mut self) -> bool {
        let mut changed = false;
        let result = match self.load.poll() {
            Ok(Some(result)) => Some(result),
            Ok(None) => None,
            Err(_) => Some(Err("Critical Lens mark loading stopped unexpectedly".into())),
        };
        if let Some(result) = result {
            let mut loaded = match result {
                Ok(loaded) => loaded,
                Err(error) => {
                    self.load_error = Some(error);
                    Default::default()
                }
            };
            let had_edits = !self.pending_edits.is_empty();
            for edit in self.pending_edits.drain(..) {
                edit.apply(&mut loaded);
            }
            changed = self.overrides.symbols != loaded.symbols
                || self.overrides.packages != loaded.packages;
            self.overrides = loaded;
            if had_edits {
                self.save_snapshot();
            }
        }
        if self.store.is_some() {
            self.save_error = self.writer.status(self.root.as_deref()).1;
        }
        changed
    }

    /// Returns the current repository's session decisions.
    #[must_use]
    pub fn overrides(&self) -> &CriticalityOverrides {
        &self.overrides
    }
    /// Returns load failure or latest current-root save failure.
    #[must_use]
    pub fn error(&self) -> Option<&str> {
        self.save_error.as_deref().or(self.load_error.as_deref())
    }
    /// Returns whether loading or durable write work remains, including prior roots.
    #[must_use]
    pub fn is_busy(&self) -> bool {
        self.load.is_running() || self.writer.status(None).0
    }
    /// Finishes accepted edits during controlled shutdown, with a bounded wait.
    ///
    /// # Errors
    /// Reports saving failures or failure to drain within `timeout`.
    pub fn flush(&mut self, timeout: Duration) -> Result<(), String> {
        if self.load.is_running() && !self.pending_edits.is_empty() {
            self.close();
        }
        self.writer.flush(timeout)
    }
}

impl Drop for MarkSession {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests;
