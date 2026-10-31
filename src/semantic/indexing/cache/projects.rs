//! One bounded latest record per attached repository and project, across revisions.

use super::*;
use crate::semantic::indexing::service::StrategyOutcome;
use crate::semantic::indexing::IndexingStrategy;

mod maintenance;

pub(in crate::semantic::indexing) use maintenance::{maintain, reader_lease};

#[derive(Serialize, Deserialize)]
struct Project<S, O, D> {
    format: u32,
    root: PathBuf,
    fingerprint: String,
    revision: Option<String>,
    outcome: O,
    scip: S,
    source_digests: D,
}

type StoredProject = Project<ScipIndex, StrategyOutcome, HashMap<String, String>>;

pub(in crate::semantic::indexing) struct PendingProject {
    file: tempfile::NamedTempFile,
    destination: PathBuf,
}

impl PendingProject {
    pub(in crate::semantic::indexing) fn publish(self, cancel: &AtomicBool) -> Result<(), String> {
        check_cancel(cancel)?;
        self.file
            .persist(self.destination)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub(in crate::semantic::indexing) fn publish(
    pending: Vec<PendingProject>,
    request: &GenerationRequest,
    report: &GenerationReport,
    lease: fs::File,
    cancel: &AtomicBool,
) -> Vec<String> {
    let mut warnings: Vec<_> = report
        .outcomes
        .iter()
        .filter_map(|outcome| {
            outcome
                .cache_warning
                .as_ref()
                .map(|warning| format!("{}: {warning}", outcome.strategy.id()))
        })
        .collect();
    if report.stale_reason.is_none() {
        for entry in pending {
            if let Err(error) = entry.publish(cancel) {
                warnings.push(error);
            }
        }
    }
    drop(lease);
    if let Err(error) = maintain(request, report, cancel) {
        warnings.push(format!("Project cache cleanup: {error}"));
    }
    warnings
}

fn key(root: &std::path::Path, strategy: &IndexingStrategy) -> String {
    let mut identity = Sha256::new();
    identity.update(root.as_os_str().as_encoded_bytes());
    identity.update(b"\0");
    identity.update(strategy.id());
    format!("{:x}.json", identity.finalize())
}

fn scoped_directory(request: &GenerationRequest) -> Result<PathBuf, String> {
    let cache = directory(request)?;
    let parent = super::super::workspace::destination_path(&cache.join("projects"))?;
    if !parent.starts_with(&cache) {
        return Err("Project cache directory escapes the private cache".into());
    }
    let root = fs::canonicalize(&request.attached_root).map_err(|e| e.to_string())?;
    let namespace = parent.join(format!(
        "{:x}",
        Sha256::digest(root.as_os_str().as_encoded_bytes())
    ));
    if fs::symlink_metadata(&namespace).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err("Project cache namespace must not be a symbolic link".into());
    }
    Ok(namespace)
}

fn path(request: &GenerationRequest, strategy: &IndexingStrategy) -> Result<PathBuf, String> {
    let root = fs::canonicalize(&request.attached_root).map_err(|e| e.to_string())?;
    Ok(scoped_directory(request)?.join(key(&root, strategy)))
}

pub(in crate::semantic::indexing) fn load(
    request: &GenerationRequest,
    inventory: &InputInventory,
    baseline: &IndexingWorkspace,
    outcome: &StrategyOutcome,
    cancel: &AtomicBool,
) -> Result<Option<(StrategyOutcome, ScipIndex)>, String> {
    let Some(bytes) = read_record(&path(request, &outcome.strategy)?, cancel)? else {
        return Ok(None);
    };
    let mut entry: StoredProject = decode(&bytes, || cancel.load(Ordering::Relaxed))?;
    if entry.format != FORMAT {
        return Err("Unsupported project cache format".into());
    }
    if entry.root != fs::canonicalize(&request.attached_root).map_err(|e| e.to_string())?
        || entry.fingerprint != baseline.fingerprint()
        || entry.outcome.strategy != outcome.strategy
        || entry.outcome.version != outcome.version
    {
        return Ok(None);
    }
    validate_documents(&entry.scip, inventory, &entry.source_digests, cancel)?;
    validate_project(&entry, baseline, cancel)?;
    entry.outcome.reused_from = Some(entry.revision.unwrap_or_else(|| "checkout".into()));
    entry.outcome.cache_warning = None;
    Ok(Some((entry.outcome, entry.scip)))
}

fn read_record(
    destination: &std::path::Path,
    cancel: &AtomicBool,
) -> Result<Option<Vec<u8>>, String> {
    if let Some(bytes) = read_index_bytes(destination, cancel)? {
        return Ok(Some(bytes));
    }
    let legacy = destination
        .parent()
        .expect("project namespace")
        .parent()
        .expect("projects directory")
        .join(destination.file_name().expect("project key"));
    read_index_bytes(&legacy, cancel)
}

fn validate_project(
    entry: &StoredProject,
    baseline: &IndexingWorkspace,
    cancel: &AtomicBool,
) -> Result<(), String> {
    let outcome = &entry.outcome;
    if outcome.failure.is_some()
        || outcome.version.is_none()
        || entry.scip.documents.is_empty()
        || outcome.documents != entry.scip.documents.len()
        || entry.source_digests.len() != entry.scip.documents.len()
    {
        return Err("Inconsistent project cache coverage".into());
    }
    for document in &entry.scip.documents {
        check_cancel(cancel)?;
        if !super::super::contains(&outcome.strategy.directory, &document.relative_path)
            || !outcome
                .strategy
                .indexer
                .covers_path(&document.relative_path)
            || entry
                .source_digests
                .get(&document.relative_path)
                .map(String::as_str)
                != baseline.source_digest(&document.relative_path)
        {
            return Err("Project cache source provenance differs".into());
        }
    }
    Ok(())
}

pub(in crate::semantic::indexing) fn stage_project(
    request: &GenerationRequest,
    baseline: &IndexingWorkspace,
    outcome: &StrategyOutcome,
    scip: &ScipIndex,
    cancel: &AtomicBool,
) -> Result<Option<PendingProject>, String> {
    if outcome.reused_from.is_some() {
        return Ok(None);
    }
    let destination = path(request, &outcome.strategy)?;
    let source_digests: HashMap<_, _> = scip
        .documents
        .iter()
        .map(|doc| {
            (
                doc.relative_path.clone(),
                baseline
                    .source_digest(&doc.relative_path)
                    .expect("normalized source input")
                    .to_owned(),
            )
        })
        .collect();
    let entry = Project {
        format: FORMAT,
        root: fs::canonicalize(&request.attached_root).map_err(|e| e.to_string())?,
        fingerprint: baseline.fingerprint(),
        revision: request.revision().map(str::to_owned),
        outcome,
        scip,
        source_digests,
    };
    let file = stage(&destination, &entry, cancel)?;
    Ok(Some(PendingProject { file, destination }))
}
