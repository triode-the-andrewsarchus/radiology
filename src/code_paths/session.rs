//! Serialized background collection I/O, independent of rendering and source navigation.
use super::{store::CodePathStore, CodePath};
use crate::background::LatestRead;
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use uuid::Uuid;

/// A successfully published collection operation; failed writes never change rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionChange {
    /// The current repository's saved traces finished loading.
    Loaded,
    /// A new trace was saved and can now be selected.
    Imported(Uuid),
    /// A saved trace was removed successfully.
    Deleted(Uuid),
}

struct Completed {
    paths: Vec<Arc<CodePath>>,
    change: CollectionChange,
}

struct Owner {
    root: PathBuf,
    epoch: u64,
    writing: bool,
}

/// One serialized collection transaction at a time on the shared bounded workers.
///
/// Scope changes retire presentation, never an accepted write. A new scope's load
/// waits for that write, so reopening a checkout cannot race its earlier save.
/// Hosts poll outside painting and disable mutation controls while busy. Call
/// [`Self::flush`] on controlled shutdown; ordinary frames never wait for I/O.
///
/// # Examples
/// ```
/// use radiology::code_paths::{session::CodePathSession, store::CodePathStore};
/// use std::time::Duration;
/// let owner = tempfile::tempdir()?;
/// let root = owner.path().join("checkout");
/// std::fs::create_dir(&root)?;
/// let mut paths = CodePathSession::new(Some(CodePathStore::new(owner.path().join("settings"))));
/// paths.open(root);
/// paths.flush(Duration::from_secs(5))?; // Only shutdown/tests wait; app frames poll.
/// assert!(paths.import_text("Boot\na.rs:boot:Enter".into()));
/// assert!(paths.paths().is_empty()); // Accepted rows change after the save completes.
/// paths.flush(Duration::from_secs(5))?;
/// assert_eq!(paths.paths()[0].title, "Boot");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct CodePathSession {
    store: Option<CodePathStore>,
    root: Option<PathBuf>,
    epoch: u64,
    loaded: bool,
    paths: Vec<Arc<CodePath>>,
    job: LatestRead<Result<Completed, String>>,
    owner: Option<Owner>,
    error: Option<String>,
    write_error: Option<String>,
}

impl CodePathSession {
    /// Configures private storage without reading or creating files.
    #[must_use]
    pub fn new(store: Option<CodePathStore>) -> Self {
        Self {
            store,
            root: None,
            epoch: 0,
            loaded: false,
            paths: Vec::new(),
            job: Default::default(),
            owner: None,
            error: None,
            write_error: None,
        }
    }

    /// Activates a canonical attached checkout, shared with its PR reviews.
    /// Opening the same root retains loaded rows and selection owned by the host.
    pub fn open(&mut self, root: PathBuf) {
        if self.root.as_ref() == Some(&root) {
            return;
        }
        self.close();
        self.root = Some(root);
        self.ensure_load();
    }

    /// Clears current presentation while allowing an accepted write to finish.
    pub fn close(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        self.root = None;
        self.paths.clear();
        self.loaded = false;
        self.error = None;
    }

    /// Returns accepted rows, never an optimistic unsaved collection.
    #[must_use]
    pub fn paths(&self) -> &[Arc<CodePath>] {
        &self.paths
    }

    /// Returns whether a serialized operation or a deferred scope load remains.
    #[must_use]
    pub fn is_busy(&self) -> bool {
        self.job.is_running() || (self.root.is_some() && !self.loaded)
    }

    /// Returns the current scope's failure or an unacknowledged background write failure.
    #[must_use]
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref().or(self.write_error.as_deref())
    }

    /// Retries the current load after external storage repair, preserving accepted rows.
    pub fn reload(&mut self) {
        if !self.job.is_running() {
            self.loaded = false;
            self.ensure_load();
        }
    }

    fn ensure_load(&mut self) {
        if self.loaded || self.job.is_running() {
            return;
        }
        let Some(root) = self.root.clone() else {
            return;
        };
        let Some(store) = self.store.clone() else {
            self.loaded = true;
            self.error = Some("Private code path storage is unavailable".into());
            return;
        };
        self.owner = Some(Owner {
            root: root.clone(),
            epoch: self.epoch,
            writing: false,
        });
        self.job.spawn(move |_| {
            store
                .load(&root)
                .map(|paths| Completed {
                    paths: paths.into_iter().map(Arc::new).collect(),
                    change: CollectionChange::Loaded,
                })
                .map_err(|error| error.to_string())
        });
    }

    /// Queues atomic text import; parse/save failures preserve rows and saved identities.
    /// Returns `false` when another transaction owns the collection or no scope exists.
    pub fn import_text(&mut self, text: String) -> bool {
        self.mutate(move |paths| {
            let trace = CodePath::parse(&text).map_err(|error| error.to_string())?;
            let id = trace.id;
            paths.push(trace);
            Ok(CollectionChange::Imported(id))
        })
    }

    /// Queues deletion by identity, publishing only after a successful save.
    pub fn delete(&mut self, id: Uuid) -> bool {
        if !self.paths.iter().any(|path| path.id == id) {
            return false;
        }
        self.mutate(move |paths| {
            paths.retain(|path| path.id != id);
            Ok(CollectionChange::Deleted(id))
        })
    }

    fn mutate(
        &mut self,
        change: impl FnOnce(&mut Vec<CodePath>) -> Result<CollectionChange, String> + Send + 'static,
    ) -> bool {
        if self.is_busy() {
            return false;
        }
        let (Some(root), Some(store)) = (self.root.clone(), self.store.clone()) else {
            return false;
        };
        let snapshot = self.paths.clone();
        self.owner = Some(Owner {
            root: root.clone(),
            epoch: self.epoch,
            writing: true,
        });
        self.error = None;
        self.job.spawn(move |_| {
            let mut paths: Vec<_> = snapshot.iter().map(|path| (**path).clone()).collect();
            let change = change(&mut paths)?;
            store
                .save(&root, &paths)
                .map_err(|error| error.to_string())?;
            Ok(Completed {
                paths: paths.into_iter().map(Arc::new).collect(),
                change,
            })
        });
        true
    }

    /// Publishes one current completion and schedules a deferred scope load if needed.
    /// Old-scope rows/errors cannot replace current rows; failed writes stay reportable.
    pub fn poll(&mut self) -> Option<CollectionChange> {
        let result = match self.job.poll() {
            Ok(result) => result,
            Err(_) => Some(Err("Code path storage worker stopped unexpectedly".into())),
        };
        let mut change = None;
        if let Some(result) = result {
            if let Some(owner) = self.owner.take() {
                let current = self.root.as_ref() == Some(&owner.root) && self.epoch == owner.epoch;
                if owner.writing {
                    self.write_error = result
                        .as_ref()
                        .err()
                        .map(|error| format!("Code paths for {}: {error}", owner.root.display()));
                }
                if current {
                    self.loaded = true;
                    match result {
                        Ok(completed) => {
                            self.paths = completed.paths;
                            self.error = None;
                            change = Some(completed.change);
                        }
                        Err(error) => self.error = Some(error),
                    }
                }
            }
        }
        self.ensure_load();
        change
    }

    /// Waits for accepted writes and pending loads during controlled shutdown.
    ///
    /// # Errors
    /// Returns a storage/worker failure or a timeout; never used during rendering.
    pub fn flush(&mut self, timeout: Duration) -> Result<(), String> {
        let deadline = Instant::now() + timeout;
        while self.is_busy() {
            self.poll();
            if Instant::now() >= deadline {
                return Err("Code path storage did not finish before shutdown deadline".into());
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        self.error().map_or(Ok(()), |error| Err(error.into()))
    }
}

#[cfg(test)]
mod tests;
