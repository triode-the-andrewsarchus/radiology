//! Atomic, repository-scoped persistence of generated semantic data and provenance.

use super::inventory::{check_cancel, InputInventory};
use super::service::{GeneratedIndex, GenerationReport, GenerationRequest};
use super::workspace::IndexingWorkspace;
use crate::semantic::{ScipIndex, SemanticIndex};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

const FORMAT: u32 = 1;
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

fn path(request: &GenerationRequest) -> Result<PathBuf, String> {
    let root = fs::canonicalize(&request.root).map_err(|e| e.to_string())?;
    let parent = super::workspace::destination_path(&request.cache)?;
    if parent.starts_with(&root) {
        return Err("Index cache must be outside the viewed repository".into());
    }
    let key = format!("{:x}", Sha256::digest(root.as_os_str().as_encoded_bytes()));
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
        root: fs::canonicalize(&request.root).map_err(|e| e.to_string())?,
        inventory,
        scip,
        report,
        source_digests,
    };
    fs::create_dir_all(&request.cache).map_err(|e| e.to_string())?;
    let mut temporary =
        tempfile::NamedTempFile::new_in(&request.cache).map_err(|e| e.to_string())?;
    // Stream to a bounded file; large indexes do not require another encoded Vec.
    {
        let mut writer = LimitedWriter {
            file: BufWriter::new(temporary.as_file_mut()),
            written: 0,
            cancel,
        };
        serde_json::to_writer(&mut writer, &entry).map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
    }
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    check_cancel(cancel)?;
    temporary.persist(destination).map_err(|e| e.to_string())?;
    Ok(())
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
    let mut entry: StoredEntry = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let root = fs::canonicalize(&request.root).map_err(|e| e.to_string())?;
    validate(&entry, &root)?;
    let inventory = InputInventory::discover(&root, cancel)?;
    let fingerprint =
        IndexingWorkspace::fingerprint_source(&root, &inventory, request.limits, cancel)?;
    if inventory != entry.inventory || fingerprint != entry.report.fingerprint {
        entry.report.stale_reason = Some("Repository inputs differ from the cached index".into());
    }
    check_cancel(cancel)?;
    entry.report.cached = true;
    let mut index = SemanticIndex::from_scip(entry.scip);
    index.source_digests = entry.source_digests;
    check_cancel(cancel)?;
    Ok(Some(GeneratedIndex {
        index: Some(index),
        report: entry.report,
    }))
}

fn validate(entry: &StoredEntry, root: &std::path::Path) -> Result<(), String> {
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
    let mut seen = HashSet::new();
    for document in &entry.scip.documents {
        if inventory
            .paths
            .binary_search(&document.relative_path)
            .is_err()
            || !seen.insert(&document.relative_path)
            || !valid_document(document, &entry.source_digests)
        {
            return Err("Index cache contains invalid semantic destinations".into());
        }
    }
    Ok(())
}

fn valid_document(
    document: &crate::semantic::ScipDocument,
    source_digests: &HashMap<String, String>,
) -> bool {
    let valid_digest = source_digests
        .get(&document.relative_path)
        .is_some_and(|digest| {
            digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
        });
    valid_digest
        && (1..=3).contains(&document.position_encoding)
        && document.occurrences.iter().all(|occurrence| {
            [&occurrence.range, &occurrence.enclosing_range]
                .into_iter()
                .flatten()
                .all(|range| {
                    crate::semantic::ScipRange::from_packed(&[
                        range.start_line,
                        range.start_col,
                        range.end_line,
                        range.end_col,
                    ])
                    .is_some()
                })
        })
}

struct LimitedWriter<'a, W> {
    file: W,
    written: u64,
    cancel: &'a AtomicBool,
}

impl<W: Write> Write for LimitedWriter<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        check_cancel(self.cancel).map_err(std::io::Error::other)?;
        if self.written + bytes.len() as u64 > MAX_BYTES {
            return Err(std::io::Error::other(
                "Index cache exceeds the 256 MiB limit",
            ));
        }
        let count = self.file.write(bytes)?;
        self.written += count as u64;
        Ok(count)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}
