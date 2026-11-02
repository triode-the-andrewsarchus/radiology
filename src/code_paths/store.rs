//! Atomic app-managed storage keyed by canonical attached checkout, shared by PRs.
use super::CodePath;
use crate::{
    repository::identity::open_regular_file, semantic::indexing::workspace::destination_path,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    io::BufReader,
    path::{Path, PathBuf},
};

/// A storage failure that preserves existing traces and remains visible to the host.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// File access or atomic replacement failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// The existing archive cannot be decoded, or new data cannot be serialized.
    #[error("Invalid code path storage: {0}")]
    Json(#[from] serde_json::Error),
    /// The configured app-owned storage directory cannot be resolved.
    #[error("Could not resolve code path storage: {0}")]
    Location(String),
    /// A settings directory would write into the source checkout.
    #[error("Code paths must be stored outside the viewed repository")]
    InsideRepository,
    /// New records contain invalid values or duplicate trace identities.
    #[error("Invalid or duplicate code path identity")]
    InvalidRecords,
    /// An archive has an unsupported version or belongs to another canonical root.
    #[error("Unsupported or foreign code path storage")]
    ForeignOrUnsupported,
}

#[derive(Serialize, Deserialize)]
struct Archive<P> {
    version: u32,
    repository_root: PathBuf,
    paths: P,
}

/// Private, repository-scoped trace storage; construction and missing loads write nothing.
///
/// Pass the attached checkout root for both checkout and PR use. The host chooses
/// an app-owned directory outside source and runs file I/O outside UI rendering.
/// Mutation is synchronous and atomic; callers serialize mutations for a repository.
///
/// # Examples
/// ```
/// use radiology::code_paths::{CodePath, store::CodePathStore};
/// let owner = tempfile::tempdir()?;
/// let root = owner.path().join("checkout");
/// std::fs::create_dir(&root)?;
/// let settings = owner.path().join("settings/code-paths");
/// let store = CodePathStore::new(settings.clone());
/// assert!(store.load(&root)?.is_empty());
/// assert!(!settings.exists());
/// let trace = CodePath::parse("Boot\na.c:start:Enter the kernel")?;
/// store.save(&root, std::slice::from_ref(&trace))?;
/// assert_eq!(store.load(&root.join("."))?, vec![trace]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct CodePathStore {
    directory: PathBuf,
}

impl CodePathStore {
    /// Selects the app-owned parent without creating directories or modifying source.
    #[must_use]
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    fn location(&self, root: &Path) -> Result<(PathBuf, PathBuf), StoreError> {
        let root = root.canonicalize()?;
        let directory = destination_path(&self.directory).map_err(StoreError::Location)?;
        if directory.starts_with(&root) {
            return Err(StoreError::InsideRepository);
        }
        let key = format!(
            "{:x}.json",
            Sha256::digest(root.as_os_str().as_encoded_bytes())
        );
        Ok((directory.join(key), root))
    }

    /// Loads valid unique traces in authored order, preserving repeated titles and steps.
    /// Invalid record values and duplicate trace IDs are ignored as in the Swift store.
    ///
    /// # Errors
    /// Reports invalid storage locations, I/O, corrupt JSON, unsupported versions or
    /// foreign repository ownership. A failure never creates or overwrites storage.
    pub fn load(&self, root: &Path) -> Result<Vec<CodePath>, StoreError> {
        let (path, root) = self.location(root)?;
        let mut seen = HashSet::new();
        Ok(read(&path, &root)?
            .into_iter()
            .filter(|trace| trace.is_valid() && seen.insert(trace.id))
            .collect())
    }

    /// Atomically replaces this repository's traces after checking existing storage.
    /// Corrupt storage cannot be silently overwritten by an import or deletion.
    ///
    /// # Errors
    /// Reports invalid records/locations, corrupt existing storage and serialization
    /// or write failures. Errors preserve the previous complete storage file.
    pub fn save(&self, root: &Path, paths: &[CodePath]) -> Result<(), StoreError> {
        let (path, root) = self.location(root)?;
        let mut seen = HashSet::new();
        if paths
            .iter()
            .any(|trace| !trace.is_valid() || !seen.insert(trace.id))
        {
            return Err(StoreError::InvalidRecords);
        }
        read(&path, &root)?;
        let bytes = serde_json::to_vec_pretty(&Archive {
            version: 1,
            repository_root: root,
            paths,
        })?;
        crate::storage::atomic_write(&path, &bytes)?;
        Ok(())
    }
}

fn read(path: &Path, root: &Path) -> Result<Vec<CodePath>, StoreError> {
    let file = match open_regular_file(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(StoreError::Io(error)),
    };
    let archive: Archive<Vec<CodePath>> = serde_json::from_reader(BufReader::new(file))?;
    if archive.version != 1 || archive.repository_root != root {
        return Err(StoreError::ForeignOrUnsupported);
    }
    Ok(archive.paths)
}

#[cfg(test)]
mod tests;
