//! Atomic, repository-scoped persistence of generated semantic data and provenance.

use super::inventory::{check_cancel, InputInventory};
use super::service::{GeneratedIndex, GenerationReport, GenerationRequest};
use super::workspace::IndexingWorkspace;
use crate::semantic::{ScipIndex, SemanticIndex};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

// Version 1 could persist byte encodings as occurrence-column encodings.
const FORMAT: u32 = 2;
const IO_CHUNK: usize = 64 * 1024;
use crate::semantic::loader::{read_index_bytes, MAX_INDEX_BYTES as MAX_BYTES};

#[derive(Serialize, Deserialize)]
struct Entry<I, S, R, D> {
    format: u32,
    root: PathBuf,
    inventory: I,
    scip: S,
    report: R,
    source_digests: D,
}

type StoredEntry = Entry<InputInventory, ScipIndex, GenerationReport, HashMap<String, String>>;

pub(super) fn directory(request: &GenerationRequest) -> Result<PathBuf, String> {
    let parent = super::workspace::destination_path(&request.cache)?;
    for root in [&request.root, &request.attached_root] {
        if parent.starts_with(fs::canonicalize(root).map_err(|e| e.to_string())?) {
            return Err("Index cache must be outside the viewed repository".into());
        }
    }
    Ok(parent)
}

fn path(request: &GenerationRequest) -> Result<PathBuf, String> {
    let parent = directory(request)?;
    let mut identity = Sha256::new();
    identity.update(request.identity_root()?.as_os_str().as_encoded_bytes());
    if let Some(revision) = request.revision() {
        identity.update(b"\0");
        identity.update(revision);
    }
    let key = format!("{:x}", identity.finalize());
    Ok(parent.join(format!("{key}.index.json")))
}

pub(super) fn save(
    request: &GenerationRequest,
    inventory: &InputInventory,
    scip: &ScipIndex,
    source_digests: &HashMap<String, String>,
    report: &GenerationReport,
    cancel: &AtomicBool,
) -> Result<(), String> {
    check_cancel(cancel)?;
    if scip.documents.is_empty() || report.stale_reason.is_some() {
        return Ok(());
    }
    let destination = path(request)?;
    let entry = Entry {
        format: FORMAT,
        root: request.identity_root()?,
        inventory,
        scip,
        report,
        source_digests,
    };
    let temporary = stage(&destination, &entry, cancel)?;
    check_cancel(cancel)?;
    temporary.persist(destination).map_err(|e| e.to_string())?;
    Ok(())
}

fn stage<T: Serialize>(
    destination: &std::path::Path,
    entry: &T,
    cancel: &AtomicBool,
) -> Result<tempfile::NamedTempFile, String> {
    check_cancel(cancel)?;
    let parent = destination.parent().ok_or("Index cache has no parent")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    // Stream to a bounded file; large indexes do not require another encoded Vec.
    {
        let mut writer = LimitedWriter {
            file: BufWriter::new(temporary.as_file_mut()),
            written: 0,
            cancel,
        };
        serde_json::to_writer(&mut writer, entry).map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
    }
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    check_cancel(cancel)?;
    Ok(temporary)
}

/// Loads persisted semantic data without running tools or writing any files.
///
/// Exact input fingerprints qualify as current. Changed inputs retain the original
/// source digests and receive an explicit stale report, so destination reads still
/// reject mismatched bytes. A missing cache returns `Ok(None)`.
///
/// # Errors
///
/// Returns cache format/path/read limits, corrupt data, input discovery and
/// cancellation errors. Callers may fall back to an explicitly unverified import.
pub fn load(
    request: &GenerationRequest,
    cancel: &AtomicBool,
) -> Result<Option<GeneratedIndex>, String> {
    check_cancel(cancel)?;
    let destination = path(request)?;
    let Some(bytes) = read_index_bytes(&destination, cancel)? else {
        return Ok(None);
    };
    let entry = decode(&bytes, || cancel.load(Ordering::Relaxed))?;
    restore(entry, request, cancel).map(Some)
}

fn restore(
    mut entry: StoredEntry,
    request: &GenerationRequest,
    cancel: &AtomicBool,
) -> Result<GeneratedIndex, String> {
    let root = fs::canonicalize(&request.root).map_err(|e| e.to_string())?;
    validate(&entry, &request.identity_root()?, cancel)?;
    if entry.report.revision.as_deref() != request.revision() {
        return Err("Index cache belongs to another revision".into());
    }
    let inventory = request.inventory(cancel)?;
    let fingerprint =
        IndexingWorkspace::fingerprint_source(&root, &inventory, request.limits, cancel)?;
    if inventory != entry.inventory || fingerprint != entry.report.fingerprint {
        if request.review.is_some() {
            return Err("Pinned inputs differ from the cached index".into());
        }
        entry.report.stale_reason = Some("Repository inputs differ from the cached index".into());
    }
    check_cancel(cancel)?;
    entry.report.cached = true;
    let mut index = SemanticIndex::from_scip_cancellable(entry.scip, || {
        cancel.load(std::sync::atomic::Ordering::Relaxed)
    })
    .map_err(|error| error.to_string())?;
    index.source_digests = entry.source_digests;
    check_cancel(cancel)?;
    Ok(GeneratedIndex {
        index: Some(index),
        report: entry.report,
    })
}

fn decode<T: DeserializeOwned>(bytes: &[u8], cancelled: impl Fn() -> bool) -> Result<T, String> {
    // Buffer outside the guard: serde reads individual bytes, but cancellation is
    // polled at most one 64-KiB input chunk apart, including within large strings.
    let reader = BufReader::with_capacity(
        IO_CHUNK,
        CacheReader {
            bytes,
            cancelled,
            stopped: false,
        },
    );
    serde_json::from_reader(reader).map_err(|error| error.to_string())
}

struct CacheReader<'a, F> {
    bytes: &'a [u8],
    cancelled: F,
    stopped: bool,
}

impl<F: Fn() -> bool> Read for CacheReader<'_, F> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if self.stopped || (self.cancelled)() {
            self.stopped = true;
            return Err(std::io::Error::other("Index cache decoding was cancelled"));
        }
        let limit = buffer.len().min(IO_CHUNK);
        self.bytes.read(&mut buffer[..limit])
    }
}

fn validate(
    entry: &StoredEntry,
    root: &std::path::Path,
    cancel: &AtomicBool,
) -> Result<(), String> {
    check_cancel(cancel)?;
    let inventory = validate_inventory(entry, root)?;
    validate_documents(&entry.scip, &inventory, &entry.source_digests, cancel)
}

fn validate_documents(
    scip: &ScipIndex,
    inventory: &InputInventory,
    source_digests: &HashMap<String, String>,
    cancel: &AtomicBool,
) -> Result<(), String> {
    let mut seen = HashSet::new();
    for document in &scip.documents {
        check_cancel(cancel)?;
        if inventory
            .paths
            .binary_search(&document.relative_path)
            .is_err()
            || !seen.insert(&document.relative_path)
            || !valid_document(document, source_digests, cancel)?
        {
            return Err("Index cache contains invalid semantic destinations".into());
        }
    }
    Ok(())
}

fn validate_inventory(
    entry: &StoredEntry,
    root: &std::path::Path,
) -> Result<InputInventory, String> {
    if entry.format != FORMAT || entry.root != root || entry.report.stale_reason.is_some() {
        return Err("Unsupported or inconsistent index cache provenance".into());
    }
    let inventory = InputInventory::from_paths(entry.inventory.paths.clone())?;
    if inventory.paths != entry.inventory.paths
        || entry.report.plan
            != super::IndexPlan::inspect(inventory.paths.iter().map(String::as_str))
        || entry.scip.documents.is_empty()
        || entry.report.documents != entry.scip.documents.len()
        || entry.source_digests.len() != entry.scip.documents.len()
    {
        return Err("Index cache inventory or coverage is inconsistent".into());
    }
    Ok(inventory)
}

fn valid_document(
    document: &crate::semantic::ScipDocument,
    source_digests: &HashMap<String, String>,
    cancel: &AtomicBool,
) -> Result<bool, String> {
    check_cancel(cancel)?;
    let valid_digest = source_digests
        .get(&document.relative_path)
        .is_some_and(|digest| {
            digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
        });
    if !valid_digest || !(1..=3).contains(&document.position_encoding) {
        return Ok(false);
    }
    for occurrence in &document.occurrences {
        check_cancel(cancel)?;
        for range in [&occurrence.range, &occurrence.enclosing_range]
            .into_iter()
            .flatten()
        {
            if crate::semantic::ScipRange::from_packed(&[
                range.start_line,
                range.start_col,
                range.end_line,
                range.end_col,
            ])
            .is_none()
            {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

struct LimitedWriter<'a, W> {
    file: W,
    written: u64,
    cancel: &'a AtomicBool,
}

impl<W: Write> Write for LimitedWriter<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        check_cancel(self.cancel).map_err(std::io::Error::other)?;
        if self.written.saturating_add(bytes.len() as u64) > MAX_BYTES {
            return Err(std::io::Error::other(
                "Index cache exceeds the 256 MiB limit",
            ));
        }
        let count = self.file.write(&bytes[..bytes.len().min(IO_CHUNK)])?;
        self.written += count as u64;
        Ok(count)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        check_cancel(self.cancel).map_err(std::io::Error::other)?;
        self.file.flush()?;
        check_cancel(self.cancel).map_err(std::io::Error::other)
    }
}

#[cfg(test)]
mod tests;

pub(super) mod projects;
