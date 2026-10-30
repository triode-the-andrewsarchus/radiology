//! Repository-scoped Critical Lens decisions with atomic, ordered persistence.

use super::{Criticality, CriticalityOverrides};
use crate::semantic::indexing::workspace::destination_path;
use crate::semantic::loader::read_bytes_limited;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};

const MAX_BYTES: u64 = 4 * 1024 * 1024;

/// Stable identity selected by a source menu; locals cannot be persisted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkTarget {
    /// Complete workspace-global SCIP symbol, including package version.
    Symbol(String),
    /// Version-independent `scheme manager package` key.
    Package(String),
}

/// An edit that can be replayed over a delayed load without losing stored choices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkEdit {
    /// Identity of the decision being changed.
    pub target: MarkTarget,
    /// Critical/Ordinary, or None/Unknown to restore the built-in policy.
    pub decision: Option<Criticality>,
}

impl MarkEdit {
    /// Applies this edit; returns whether the targeted decision actually changed.
    pub fn apply(&self, overrides: &mut CriticalityOverrides) -> bool {
        let decision = self
            .decision
            .filter(|decision| *decision != Criticality::Unknown);
        match &self.target {
            MarkTarget::Symbol(symbol) => {
                if !CriticalityOverrides::is_overridable(symbol)
                    || overrides.symbols.get(symbol).copied() == decision
                {
                    return false;
                }
                overrides.set_symbol(symbol, decision);
            }
            MarkTarget::Package(package) => {
                if package.is_empty() || overrides.packages.get(package).copied() == decision {
                    return false;
                }
                overrides.set_package(package, decision);
            }
        }
        true
    }
}

#[derive(Default)]
struct OrderingState {
    next: AtomicU64,
    attempts: Mutex<HashMap<PathBuf, u64>>,
}

/// Purely configured storage; construction and missing-file loads write nothing.
///
/// Clones share save sequencing. Call `reserve_revision` when the reader accepts an
/// edit, before queueing its snapshot. Lower revisions cannot replace a newer save
/// attempt for the same file, including after deletion. The host runs I/O off-thread.
#[derive(Clone)]
pub struct OverrideStore {
    directory: PathBuf,
    ordering: Arc<OrderingState>,
}

impl OverrideStore {
    /// Selects a private settings directory without creating it.
    #[must_use]
    pub fn new(directory: PathBuf) -> Self {
        Self {
            directory,
            ordering: Default::default(),
        }
    }

    /// Reserves the next ordered save revision across clones of this store.
    #[must_use]
    pub fn reserve_revision(&self) -> u64 {
        self.ordering.next.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn location(&self, root: &Path) -> Result<(PathBuf, String), String> {
        let root = std::fs::canonicalize(root).map_err(|error| error.to_string())?;
        let directory = destination_path(&self.directory)?;
        if directory.starts_with(&root) {
            return Err("Critical marks must be stored outside the viewed repository".into());
        }
        let canonical = root
            .to_str()
            .ok_or("Repository root is not UTF-8")?
            .to_owned();
        let key = format!("{:x}", Sha256::digest(canonical.as_bytes()));
        Ok((directory.join(format!("{key}.json")), canonical))
    }

    /// Loads one repository's decisions, returning an empty set for a missing file.
    ///
    /// # Errors
    /// Rejects unsafe paths, foreign roots, unsupported formats, corrupt/oversized
    /// JSON, nonregular files and cancellation. No directories are created.
    pub fn load(&self, root: &Path, cancel: &AtomicBool) -> Result<CriticalityOverrides, String> {
        let (path, canonical) = self.location(root)?;
        let Some(bytes) = read_bytes_limited(
            &path,
            cancel,
            MAX_BYTES,
            "Critical marks exceed the 4 MiB limit",
        )?
        else {
            return Ok(CriticalityOverrides {
                repository_root: canonical,
                ..Default::default()
            });
        };
        let mut overrides: CriticalityOverrides =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if overrides.format_version != 1 || overrides.repository_root != canonical {
            return Err("Unsupported or foreign Critical Lens decisions".into());
        }
        normalize(&mut overrides);
        Ok(overrides)
    }

    /// Saves or deletes a snapshot atomically, returning false for an obsolete revision.
    ///
    /// Failures keep the previous file intact and remain visible to the host; session
    /// decisions must still apply. Writes are durable requests, not cancellable reads.
    /// Sequencing is shared within this store and its clones, not across processes.
    ///
    /// # Errors
    /// Reports unsafe paths, format/size errors, serialization, write or deletion
    /// failures. An attempted newer revision prevents delayed older writes even if
    /// that attempt fails, so user intent is never rolled back by a stale task.
    pub fn save(
        &self,
        root: &Path,
        overrides: &CriticalityOverrides,
        revision: u64,
    ) -> Result<bool, String> {
        let (path, canonical) = self.location(root)?;
        if overrides.format_version != 1 {
            return Err("Unsupported Critical Lens decisions format".into());
        }
        let mut attempts = self
            .ordering
            .attempts
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if attempts
            .get(&path)
            .is_some_and(|written| *written > revision)
        {
            return Ok(false);
        }
        attempts.insert(path.clone(), revision);
        let mut snapshot = overrides.clone();
        normalize(&mut snapshot);
        snapshot.repository_root = canonical;
        if snapshot.is_empty() {
            return match std::fs::remove_file(&path) {
                Ok(()) => Ok(true),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
                Err(error) => Err(error.to_string()),
            };
        }
        let bytes = serde_json::to_vec_pretty(&snapshot).map_err(|error| error.to_string())?;
        if bytes.len() as u64 > MAX_BYTES {
            return Err("Critical marks exceed the 4 MiB limit".into());
        }
        let parent = path.parent().expect("settings file parent");
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let mut temporary =
            tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
        temporary
            .write_all(&bytes)
            .map_err(|error| error.to_string())?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|error| error.to_string())?;
        temporary.persist(path).map_err(|error| error.to_string())?;
        Ok(true)
    }
}

fn normalize(overrides: &mut CriticalityOverrides) {
    overrides.symbols.retain(|symbol, decision| {
        CriticalityOverrides::is_overridable(symbol) && *decision != Criticality::Unknown
    });
    overrides
        .packages
        .retain(|package, decision| !package.is_empty() && *decision != Criticality::Unknown);
}

#[cfg(test)]
mod tests;
