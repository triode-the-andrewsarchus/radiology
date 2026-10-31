//! Explicit feature cfgs from the isolated project and its nested manifests.

use super::inventory::{check_cancel, InputInventory};
use super::{Indexer, IndexingStrategy};
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::atomic::AtomicBool;

const MAX_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;

pub(super) fn collect(
    strategy: &IndexingStrategy,
    inventory: &InputInventory,
    workspace: &Path,
    cancel: &AtomicBool,
) -> Result<Vec<String>, String> {
    if strategy.indexer != Indexer::Rust {
        return Ok(Vec::new());
    }
    let mut features = BTreeSet::new();
    for path in &inventory.paths {
        check_cancel(cancel)?;
        if super::contains(&strategy.directory, path)
            && Path::new(path)
                .file_name()
                .is_some_and(|name| name == "Cargo.toml")
        {
            collect_manifest(&workspace.join(path), workspace, cancel, &mut features)
                .map_err(|error| format!("{path}: {error}"))?;
        }
    }
    Ok(features.into_iter().collect())
}

fn collect_manifest(
    path: &Path,
    workspace: &Path,
    cancel: &AtomicBool,
    features: &mut BTreeSet<String>,
) -> Result<(), String> {
    let canonical = path.canonicalize().map_err(|error| error.to_string())?;
    if !canonical.starts_with(workspace) {
        return Err("Cargo manifest escapes the isolated workspace".into());
    }
    let bytes = crate::semantic::loader::read_bytes_limited(
        &canonical,
        cancel,
        MAX_MANIFEST_BYTES,
        "Cargo manifest exceeds the 4 MiB limit",
    )?
    .ok_or("Cargo manifest disappeared")?;
    // Cargo diagnoses malformed manifests it uses. Unused/invalid nested files
    // do not make an otherwise indexable parent project fail here.
    let Ok(text) = String::from_utf8(bytes) else {
        return Ok(());
    };
    let document = toml_edit::Document::parse(text);
    check_cancel(cancel)?;
    let Ok(document) = document else {
        return Ok(());
    };
    if let Some(table) = document
        .as_table()
        .get("features")
        .and_then(toml_edit::Item::as_table_like)
    {
        for (name, _) in table.iter() {
            check_cancel(cancel)?;
            if name != "default" {
                features.insert(name.to_owned());
            }
        }
    }
    Ok(())
}
