//! Background import of repository-provided SCIP indexes.
//!
//! Imported files have no freshness provenance. Generation and validated caches
//! belong to the indexing service; an import must never claim a current index.

use super::{ScipIndex, SemanticIndex};
use crate::background::LatestRead;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) const MAX_INDEX_BYTES: u64 = 256 * 1024 * 1024;

/// Lifecycle of an imported index, independent of any desktop toolkit.
#[derive(Debug, Default, PartialEq, Eq)]
pub enum IndexImportStatus {
    /// No repository import has been requested.
    #[default]
    Idle,
    /// Files are being read and decoded outside the UI thread.
    Loading,
    /// Neither supported import path exists.
    Missing,
    /// An index was imported; its relationship to source bytes is unknown.
    Imported {
        /// Path of the successfully decoded file.
        path: PathBuf,
        /// Number of documents with semantic data.
        documents: usize,
    },
    /// All existing candidate files failed to decode.
    Failed(String),
    /// The user stopped the import; an already published index remains usable.
    Cancelled,
}

/// A decoded import and the file that supplied it.
#[derive(Debug)]
pub struct ImportedIndex {
    /// Decoded semantic data.
    pub index: SemanticIndex,
    /// Input file, retained for diagnostics.
    pub path: PathBuf,
}

/// Replaceable import job with bounded reads and stale-publication protection.
#[derive(Default)]
pub struct IndexImporter {
    job: LatestRead<Result<Option<ImportedIndex>, String>>,
    status: IndexImportStatus,
}

impl IndexImporter {
    /// Starts importing `index.scip`, falling back to `.scip` in `root`.
    pub fn start(&mut self, root: PathBuf) {
        self.status = IndexImportStatus::Loading;
        self.job.spawn(move |cancel| import(&root, cancel));
    }

    /// Stops a pending import without invalidating already published data.
    pub fn stop(&mut self) {
        if self.job.is_running() {
            self.job.stop();
            self.status = IndexImportStatus::Cancelled;
        }
    }

    /// Returns the latest lifecycle status.
    #[must_use]
    pub fn status(&self) -> &IndexImportStatus {
        &self.status
    }

    /// Returns whether an import is awaiting completion.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.job.is_running()
    }

    /// Publishes the current job, retiring errors into the diagnostic state.
    pub fn poll(&mut self) -> Option<SemanticIndex> {
        let result = match self.job.poll() {
            Ok(None) => return None,
            Ok(Some(result)) => result,
            Err(_) => Err("Index import stopped unexpectedly".into()),
        };
        match result {
            Ok(Some(imported)) => {
                self.status = IndexImportStatus::Imported {
                    documents: imported.index.documents.len(),
                    path: imported.path,
                };
                Some(imported.index)
            }
            Ok(None) => {
                self.status = IndexImportStatus::Missing;
                None
            }
            Err(error) => {
                self.status = IndexImportStatus::Failed(error);
                None
            }
        }
    }
}

fn import(root: &Path, cancel: &AtomicBool) -> Result<Option<ImportedIndex>, String> {
    let mut failures = Vec::new();
    for path in [root.join("index.scip"), root.join(".scip")] {
        if cancel.load(Ordering::Relaxed) {
            return Ok(None);
        }
        match read_scip(&path, cancel) {
            Ok(Some(scip)) => {
                return Ok(Some(ImportedIndex {
                    index: SemanticIndex::from_scip(scip),
                    path,
                }))
            }
            Ok(None) => {}
            Err(error) => failures.push(format!("{}: {error}", path.display())),
        }
    }
    if failures.is_empty() {
        Ok(None)
    } else {
        Err(failures.join("\n"))
    }
}

pub(crate) fn read_generated(path: &Path, cancel: &AtomicBool) -> Result<ScipIndex, String> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|e| format!("No regular index output: {e}"))?;
    if !metadata.is_file() {
        return Err("Generated index output must be a regular file".into());
    }
    read_scip(path, cancel)?.ok_or("Indexer did not write index.scip".into())
}

pub(crate) fn read_scip(path: &Path, cancel: &AtomicBool) -> Result<Option<ScipIndex>, String> {
    let Some(bytes) = read_index_bytes(path, cancel)? else {
        return Ok(None);
    };
    let scip = ScipIndex::decode(&bytes).map_err(|e| e.to_string())?;
    if scip.documents.is_empty() {
        return Err("Index contains no documents".into());
    }
    // Imported paths must remain repository-relative even for symbol navigation.
    if scip.documents.iter().any(|document| {
        !crate::source::location::SourceLocation::is_repository_path(&document.relative_path)
    }) {
        return Err("Index contains an invalid repository-relative document path".into());
    }
    if cancel.load(Ordering::Relaxed) {
        return Ok(None);
    }
    Ok(Some(scip))
}

/// Opens a regular file without following its final symlink or blocking on a FIFO.
pub(crate) fn open_regular_file(path: &Path) -> std::io::Result<std::fs::File> {
    let mut options = std::fs::File::options();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(path)?;
    if !file.metadata()?.is_file() {
        return Err(std::io::Error::other("Index/input is not a regular file"));
    }
    Ok(file)
}

/// Reads bounded index data for both SCIP imports and persisted normalized indexes.
pub(crate) fn read_index_bytes(
    path: &Path,
    cancel: &AtomicBool,
) -> Result<Option<Vec<u8>>, String> {
    let file = match open_regular_file(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    if file.metadata().map_err(|e| e.to_string())?.len() > MAX_INDEX_BYTES {
        return Err("Index exceeds the 256 MiB limit".into());
    }
    let mut reader = file.take(MAX_INDEX_BYTES + 1);
    let mut bytes = Vec::new();
    let mut chunk = [0; 64 * 1024];
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err("Index read was cancelled".into());
        }
        let count = reader.read(&mut chunk).map_err(|e| e.to_string())?;
        if count == 0 {
            return Ok(Some(bytes));
        }
        bytes.extend_from_slice(&chunk[..count]);
        if bytes.len() as u64 > MAX_INDEX_BYTES {
            return Err("Index exceeds the 256 MiB limit".into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn write_index(path: &Path, document: &str) {
        // Index.documents (tag 2), Document.relative_path (tag 1).
        let mut bytes = vec![0x12, (document.len() + 2) as u8, 0x0a, document.len() as u8];
        bytes.extend_from_slice(document.as_bytes());
        std::fs::write(path, bytes).unwrap();
    }

    fn settle(importer: &mut IndexImporter) -> Option<SemanticIndex> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let mut published = None;
        while importer.is_running() {
            published = importer.poll().or(published);
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        published
    }

    #[test]
    fn fallback_reports_provenance_and_failures_are_visible() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("index.scip"), [0xff]).unwrap();
        write_index(&root.path().join(".scip"), "src/main.rs");
        let mut importer = IndexImporter::default();
        importer.start(root.path().to_owned());
        assert!(settle(&mut importer)
            .unwrap()
            .documents
            .contains_key("src/main.rs"));
        assert_eq!(
            importer.status(),
            &IndexImportStatus::Imported {
                path: root.path().join(".scip"),
                documents: 1,
            }
        );
        std::fs::remove_file(root.path().join(".scip")).unwrap();
        importer.start(root.path().to_owned());
        assert!(settle(&mut importer).is_none());
        assert!(
            matches!(importer.status(), IndexImportStatus::Failed(error) if error.contains("index.scip"))
        );
    }

    #[test]
    fn replacement_and_cancellation_retire_completed_imports() {
        let first = tempfile::tempdir().unwrap();
        let next = tempfile::tempdir().unwrap();
        write_index(&first.path().join("index.scip"), "old.rs");
        write_index(&next.path().join("index.scip"), "new.rs");
        let mut importer = IndexImporter::default();
        importer.start(first.path().to_owned());
        importer.start(next.path().to_owned());
        let index = settle(&mut importer).unwrap();
        assert!(index.documents.contains_key("new.rs"));
        assert!(!index.documents.contains_key("old.rs"));
        importer.start(first.path().to_owned());
        importer.stop();
        assert!(importer.poll().is_none());
        assert_eq!(importer.status(), &IndexImportStatus::Cancelled);
    }

    #[test]
    fn missing_empty_oversized_and_escaping_indexes_are_distinct() {
        let root = tempfile::tempdir().unwrap();
        let cancel = AtomicBool::new(false);
        assert!(import(root.path(), &cancel).unwrap().is_none());
        let path = root.path().join("index.scip");
        std::fs::write(&path, []).unwrap();
        assert!(import(root.path(), &cancel)
            .unwrap_err()
            .contains("no documents"));
        write_index(&path, "../outside.rs");
        assert!(import(root.path(), &cancel)
            .unwrap_err()
            .contains("invalid repository-relative"));
        std::fs::File::create(&path)
            .unwrap()
            .set_len(MAX_INDEX_BYTES + 1)
            .unwrap();
        assert!(import(root.path(), &cancel)
            .unwrap_err()
            .contains("256 MiB"));
    }
}
