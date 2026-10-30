//! Byte/type/mode proof against the immutable tree, independent of Git index flags.

use super::{store::GitStore, ReviewControl, SourceLocation};
use crate::repository::identity::read_identity;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

pub(super) struct Entry {
    pub mode: String,
    pub blob: String,
}

pub(super) fn verify(
    store: &GitStore<'_>,
    head: &str,
    root: &Path,
    control: &ReviewControl<'_>,
) -> Result<(), String> {
    verify_owner(store, head, root, control)?;
    let entries = entries(store, head)?;
    let directories = directories(&entries);
    verify_inventory(root, &entries, &directories, control)
}

fn verify_owner(
    store: &GitStore<'_>,
    head: &str,
    root: &Path,
    control: &ReviewControl<'_>,
) -> Result<(), String> {
    control.check()?;
    let metadata = std::fs::symlink_metadata(root).map_err(|error| error.to_string())?;
    if !metadata.is_dir() || metadata.is_symlink() {
        return Err("Review tree is not a real directory".into());
    }
    if !std::fs::symlink_metadata(root.join(".git")).is_ok_and(|metadata| metadata.is_file()) {
        return Err("Review tree has invalid Git metadata".into());
    }
    let git = control.git(root);
    let actual = git.required(&["rev-parse", "--verify", "HEAD^{commit}"])?;
    if !String::from_utf8_lossy(&actual)
        .trim()
        .eq_ignore_ascii_case(head)
    {
        return Err("Review tree HEAD differs from the pinned commit".into());
    }
    let common = git.required(&["rev-parse", "--path-format=absolute", "--git-common-dir"])?;
    let common = super::store::worktree_path(common.strip_suffix(b"\n").unwrap_or(&common))?;
    if std::fs::canonicalize(common).map_err(|error| error.to_string())? != store.0.root {
        return Err("Review tree belongs to a different object store".into());
    }
    Ok(())
}

fn directories(entries: &BTreeMap<String, Entry>) -> BTreeSet<PathBuf> {
    let mut directories = BTreeSet::new();
    for (path, entry) in entries {
        for parent in Path::new(path)
            .ancestors()
            .skip(1)
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            directories.insert(parent.to_owned());
        }
        if entry.mode == "160000" {
            directories.insert(PathBuf::from(path));
        }
    }
    directories
}

fn verify_inventory(
    root: &Path,
    entries: &BTreeMap<String, Entry>,
    directories: &BTreeSet<PathBuf>,
    control: &ReviewControl<'_>,
) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for item in walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| entry.path() != root.join(".git"))
    {
        control.check()?;
        let item = item.map_err(|error| error.to_string())?;
        if item.depth() == 0 {
            continue;
        }
        if let Some(path) = verify_entry(&item, root, entries, directories, control)? {
            seen.insert(path);
        }
    }
    if seen.len() != entries.len() {
        return Err("Review tree has missing tracked files".into());
    }
    control.check()
}

fn verify_entry(
    item: &walkdir::DirEntry,
    root: &Path,
    entries: &BTreeMap<String, Entry>,
    directories: &BTreeSet<PathBuf>,
    control: &ReviewControl<'_>,
) -> Result<Option<String>, String> {
    let relative = item
        .path()
        .strip_prefix(root)
        .map_err(|error| error.to_string())?;
    let path = relative.to_str().ok_or("Review tree path is not UTF-8")?;
    if item.file_type().is_dir() {
        if !directories.contains(relative) {
            return Err(format!("Unexpected review directory: {path}"));
        }
        if entries
            .get(path)
            .is_some_and(|entry| entry.mode == "160000")
        {
            return Ok(Some(path.to_owned()));
        }
    } else {
        let entry = entries
            .get(path)
            .ok_or_else(|| format!("Unexpected review file: {path}"))?;
        verify_file(item.path(), entry, control)?;
        return Ok(Some(path.to_owned()));
    }
    Ok(None)
}

pub(super) fn entries(store: &GitStore<'_>, head: &str) -> Result<BTreeMap<String, Entry>, String> {
    let bytes = store.bytes(&["ls-tree", "-rz", head])?;
    let mut entries = BTreeMap::new();
    for record in bytes
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        store.0.check()?;
        let tab = record
            .iter()
            .position(|byte| *byte == b'\t')
            .ok_or("Malformed Git tree entry")?;
        let fields: Vec<_> = std::str::from_utf8(&record[..tab])
            .map_err(|error| error.to_string())?
            .split(' ')
            .collect();
        if fields.len() != 3
            || !matches!(
                (fields[0], fields[1]),
                ("100644" | "100755" | "120000", "blob") | ("160000", "commit")
            )
        {
            return Err("Unsupported Git tree entry".into());
        }
        let path = std::str::from_utf8(&record[tab + 1..]).map_err(|error| error.to_string())?;
        if !SourceLocation::is_repository_path(path) || path == ".git" || path.starts_with(".git/")
        {
            return Err("Invalid Git snapshot path".into());
        }
        super::required_oid(Some(fields[2]))?;
        entries.insert(
            path.to_owned(),
            Entry {
                mode: fields[0].into(),
                blob: fields[2].into(),
            },
        );
    }
    Ok(entries)
}

fn verify_file(path: &Path, entry: &Entry, control: &ReviewControl<'_>) -> Result<(), String> {
    let identity = read_identity(path, u64::MAX, || control.check())?;
    let blob = if entry.blob.len() == 64 {
        &identity.blob_sha256
    } else {
        &identity.blob
    };
    let kind = if identity.link {
        "120000"
    } else if identity.mode & 0o100 != 0 {
        "100755"
    } else {
        "100644"
    };
    if blob != &entry.blob || kind != entry.mode {
        return Err(format!(
            "Review bytes or mode differ from the pinned tree: {}",
            path.display()
        ));
    }
    Ok(())
}
