//! Cooperative cache ownership, legacy migration and repository-scoped retention.

use super::*;
use crate::process::lease::{open_lock, try_lock};
use std::fs::File;
use std::time::{Duration, Instant};

const STAGE_PREFIX: &str = ".radiology-index-stage-";
const MIGRATED: &str = ".legacy-migrated";

pub(in crate::semantic::indexing) fn lease_file(
    request: &GenerationRequest,
) -> Result<File, String> {
    open_lock(&directory(request)?.join(".project-cache.lock"), true)
        .map_err(|error| format!("Project cache lease: {error}"))
}

pub(in crate::semantic::indexing) fn reader_lease(
    request: &GenerationRequest,
    cancel: &AtomicBool,
) -> Result<File, String> {
    let file = lease_file(request)?;
    let deadline = Instant::now() + request.timeout.min(Duration::from_secs(5));
    while !try_lock(&file, true)? {
        check_cancel(cancel)?;
        if Instant::now() >= deadline {
            return Err("Project cache maintenance is busy; retry generation".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    check_cancel(cancel)?;
    Ok(file)
}

pub(in crate::semantic::indexing) fn maintain(
    request: &GenerationRequest,
    report: &GenerationReport,
    cancel: &AtomicBool,
) -> Result<bool, String> {
    if report.stale_reason.is_some()
        || report
            .outcomes
            .iter()
            .any(|outcome| outcome.failure.is_some())
    {
        return Ok(false);
    }
    check_cancel(cancel)?;
    let file = lease_file(request)?;
    if !try_lock(&file, false)? {
        return Ok(false); // Another generator still owns records/stages; retry next time.
    }
    let namespace = scoped_directory(request)?;
    fs::create_dir_all(&namespace).map_err(|e| e.to_string())?;
    let root = fs::canonicalize(&request.attached_root).map_err(|e| e.to_string())?;
    let mut retained = HashSet::new();
    for strategy in report.plan.selected.iter().chain(&report.plan.skipped) {
        check_cancel(cancel)?;
        retained.insert(key(&root, strategy));
    }
    migrate_legacy(request, report, &namespace, &retained, cancel)?;
    prune(&namespace, &retained, cancel)?;
    Ok(true)
}

fn prune(
    namespace: &std::path::Path,
    retained: &HashSet<String>,
    cancel: &AtomicBool,
) -> Result<(), String> {
    for entry in fs::read_dir(namespace).map_err(|e| e.to_string())? {
        check_cancel(cancel)?;
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if entry.file_type().map_err(|e| e.to_string())?.is_file()
            && (name.starts_with(STAGE_PREFIX)
                || (is_record(&name) && !retained.contains(name.as_ref())))
        {
            fs::remove_file(entry.path()).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[derive(Deserialize)]
struct LegacyIdentity {
    root: PathBuf,
    outcome: LegacyOutcome,
}

#[derive(Deserialize)]
struct LegacyOutcome {
    strategy: IndexingStrategy,
}

fn is_record(name: &str) -> bool {
    name.strip_suffix(".json").is_some_and(|key| {
        key.len() == 64
            && key
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    })
}

fn migrate_legacy(
    request: &GenerationRequest,
    report: &GenerationReport,
    namespace: &std::path::Path,
    retained: &HashSet<String>,
    cancel: &AtomicBool,
) -> Result<(), String> {
    if let Ok(metadata) = fs::symlink_metadata(namespace.join(MIGRATED)) {
        return if metadata.is_file() {
            Ok(())
        } else {
            Err("Project migration marker must be a regular file".into())
        };
    }
    let root = fs::canonicalize(&request.attached_root).map_err(|e| e.to_string())?;
    let selected: HashSet<_> = report
        .plan
        .selected
        .iter()
        .map(|strategy| key(&root, strategy))
        .collect();
    for entry in
        fs::read_dir(namespace.parent().expect("project parent")).map_err(|e| e.to_string())?
    {
        check_cancel(cancel)?;
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.file_type().map_err(|e| e.to_string())?.is_file() && is_record(&name) {
            migrate_record(
                &root,
                &entry.path(),
                &namespace.join(&name),
                retained.contains(&name),
                selected.contains(&name),
                cancel,
            )?;
        }
    }
    check_cancel(cancel)?;
    File::create_new(namespace.join(MIGRATED)).map_err(|e| e.to_string())?;
    Ok(())
}

fn migrate_record(
    root: &std::path::Path,
    source: &std::path::Path,
    destination: &std::path::Path,
    retained: bool,
    selected: bool,
    cancel: &AtomicBool,
) -> Result<(), String> {
    let bytes = read_index_bytes(source, cancel);
    check_cancel(cancel)?;
    let Ok(Some(bytes)) = bytes else {
        return Ok(());
    };
    let identity: Result<LegacyIdentity, _> = decode(&bytes, || cancel.load(Ordering::Relaxed));
    check_cancel(cancel)?;
    let Ok(identity) = identity else {
        return Ok(());
    }; // Unknown ownership stays untouched.
    if identity.root != root
        || source.file_name() != Some(std::ffi::OsStr::new(&key(root, &identity.outcome.strategy)))
    {
        return Ok(());
    }
    if !retained || (selected && destination.exists()) {
        fs::remove_file(source).map_err(|e| e.to_string())?;
    } else if !destination.exists() {
        fs::rename(source, destination).map_err(|e| e.to_string())?;
    }
    Ok(())
}
