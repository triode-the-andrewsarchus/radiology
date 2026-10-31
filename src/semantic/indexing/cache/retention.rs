//! Bounded published PR revision records, sharing the project writer lease.

use super::*;
use crate::process::lease::try_lock;
use std::path::Path;

// Match the managed worktree count; checkout records are independent.
const LIMIT: usize = 3;

#[derive(Deserialize)]
struct Identity {
    format: u32,
    root: PathBuf,
    report: Revision,
}

#[derive(Deserialize)]
struct Revision {
    revision: Option<String>,
}

pub(super) fn maintain(
    request: &GenerationRequest,
    report: &GenerationReport,
    current: &Path,
    cancel: &AtomicBool,
) -> Result<bool, String> {
    if request.revision().is_none()
        || report.stale_reason.is_some()
        || report
            .outcomes
            .iter()
            .any(|outcome| outcome.failure.is_some() || outcome.partial_build)
    {
        return Ok(false);
    }
    check_cancel(cancel)?;
    let lease = projects::lease_file(request)?;
    if !try_lock(&lease, false)? {
        return Ok(false); // Retry after the next successful publication.
    }
    prune(
        &directory(request)?,
        &request.identity_root()?,
        current,
        cancel,
    )
    .map(|()| true)
}

fn prune(parent: &Path, root: &Path, current: &Path, cancel: &AtomicBool) -> Result<(), String> {
    let mut revisions = collect_revisions(parent, root, current, cancel)?;
    revisions.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| b.1.cmp(&a.1))
            .then_with(|| a.2.cmp(&b.2))
    });
    for (_, _, path) in revisions.into_iter().skip(LIMIT) {
        check_cancel(cancel)?;
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn collect_revisions(
    parent: &Path,
    root: &Path,
    current: &Path,
    cancel: &AtomicBool,
) -> Result<Vec<(bool, std::time::SystemTime, PathBuf)>, String> {
    let mut revisions = Vec::new();
    for entry in fs::read_dir(parent).map_err(|e| e.to_string())? {
        check_cancel(cancel)?;
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(".radiology-index-stage-") {
            fs::remove_file(entry.path()).map_err(|e| e.to_string())?;
        } else if name.ends_with(".index.json") && owned(&entry.path(), root, cancel)? {
            let modified = entry
                .metadata()
                .map_err(|e| e.to_string())?
                .modified()
                .map_err(|e| e.to_string())?;
            revisions.push((entry.path() == current, modified, entry.path()));
        }
    }
    Ok(revisions)
}

fn owned(path: &Path, root: &Path, cancel: &AtomicBool) -> Result<bool, String> {
    let bytes = read_index_bytes(path, cancel);
    check_cancel(cancel)?;
    let Ok(Some(bytes)) = bytes else {
        return Ok(false);
    };
    let identity: Result<Identity, _> = decode(&bytes, || cancel.load(Ordering::Relaxed));
    check_cancel(cancel)?;
    let Ok(identity) = identity else {
        return Ok(false);
    };
    Ok(identity.format == FORMAT
        && identity.root == root
        && identity.report.revision.as_deref().is_some_and(|revision| {
            path.file_name() == Some(std::ffi::OsStr::new(&key(root, Some(revision))))
        }))
}

#[cfg(test)]
mod tests;
