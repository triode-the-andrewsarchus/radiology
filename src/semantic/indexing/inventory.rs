//! Full build-input inventories, independent of browser exclusions.

use super::command::IndexCommand;
use super::process::{self, OutputStream};
use crate::repository::discovery::RepositoryDiscovery;
use crate::source::location::SourceLocation;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const MAX_PATH_BYTES: usize = 64 * 1024 * 1024;
const MAX_FILES: usize = 200_000;

/// Real paths needed to reproduce a repository's build inputs.
#[derive(Debug, Clone, Default)]
pub struct InputInventory {
    /// Sorted repository-relative regular-file and symbolic-link candidates.
    pub paths: Vec<String>,
    /// Directory entries (including Git submodules) that were not fabricated.
    pub omitted_directories: Vec<String>,
}

impl InputInventory {
    /// Validates, deduplicates and sorts an explicit input inventory.
    ///
    /// # Errors
    ///
    /// Rejects ambiguous/escaping paths, Git metadata or excessive inventories.
    pub fn from_paths(paths: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut unique = BTreeSet::new();
        let mut bytes = 0;
        for path in paths {
            validate_input_path(&path)?;
            let length = path.len();
            if unique.insert(path) {
                bytes += length;
            }
            if bytes > MAX_PATH_BYTES || unique.len() > MAX_FILES {
                return Err("Build input inventory exceeds its path or file limit".into());
            }
        }
        Ok(Self {
            paths: unique.into_iter().collect(),
            omitted_directories: Vec::new(),
        })
    }

    /// Discovers tracked and unignored files, including tracked build/vendor inputs.
    ///
    /// Non-Git directories use a non-following walk with browser build/dependency
    /// exclusions, retaining `.cargo` configuration. No files are written.
    ///
    /// # Errors
    ///
    /// Reports Git, enumeration, UTF-8, inventory-limit and cancellation failures.
    pub fn discover(root: &Path, cancel: &AtomicBool) -> Result<Self, String> {
        let root = std::fs::canonicalize(root).map_err(|e| e.to_string())?;
        let paths = if root.join(".git").exists() {
            git_paths(&root, cancel)?
        } else {
            walk_paths(&root, cancel)?
        };
        let mut inventory = Self::from_paths(paths)?;
        let mut present = Vec::new();
        for path in inventory.paths {
            check_cancel(cancel)?;
            match std::fs::symlink_metadata(root.join(&path)) {
                Ok(metadata) if metadata.is_dir() => inventory.omitted_directories.push(path),
                Ok(_) => present.push(path),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {} // unstaged deletion
                Err(error) => return Err(format!("{path}: {error}")),
            }
        }
        inventory.paths = present;
        Ok(inventory)
    }
}

pub(super) fn validate_input_path(path: &str) -> Result<(), String> {
    if !SourceLocation::is_repository_path(path)
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == ".." || part == ".git")
    {
        return Err(format!("Invalid repository build input: {path}"));
    }
    Ok(())
}

pub(super) fn check_cancel(cancel: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        Err("Indexing was cancelled".into())
    } else {
        Ok(())
    }
}

fn git_paths(root: &Path, cancel: &AtomicBool) -> Result<Vec<String>, String> {
    let command = IndexCommand {
        executable: "git".into(),
        arguments: [
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ]
        .into_iter()
        .map(Into::into)
        .collect(),
        directory: root.into(),
        environment: BTreeMap::from([
            ("GIT_OPTIONAL_LOCKS".into(), "0".into()),
            ("GIT_TERMINAL_PROMPT".into(), "0".into()),
        ]),
        resources: Vec::new(),
    };
    let mut paths = Vec::new();
    let mut pending = Vec::new();
    let mut bytes_seen = 0;
    let result = process::run_observed(
        &command,
        cancel,
        Duration::from_secs(30),
        |stream, bytes| {
            if stream != OutputStream::Stdout {
                return Ok(());
            }
            bytes_seen += bytes.len();
            if bytes_seen > MAX_PATH_BYTES {
                return Err(io::Error::other(
                    "Git input inventory exceeds the 64 MiB path limit",
                ));
            }
            for byte in bytes {
                if *byte == 0 {
                    if paths.len() >= MAX_FILES {
                        return Err(io::Error::other(
                            "Git input inventory exceeds 200,000 files",
                        ));
                    }
                    let path = String::from_utf8(std::mem::take(&mut pending))
                        .map_err(io::Error::other)?;
                    paths.push(path);
                } else {
                    pending.push(*byte);
                }
            }
            Ok(())
        },
    )
    .map_err(|error| {
        format!(
            "Cannot enumerate Git inputs: {error}\n{}",
            error.stderr.text
        )
    })?;
    if !result.succeeded() || !pending.is_empty() {
        return Err(format!(
            "Git input discovery did not complete: {}",
            result.stderr.text
        ));
    }
    Ok(paths)
}

fn walk_paths(root: &Path, cancel: &AtomicBool) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    let mut path_bytes = 0;
    let entries = walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            if entry.depth() == 0 {
                return true;
            }
            let name = entry.file_name().to_string_lossy();
            name != ".git"
                && (name == ".cargo"
                    || !RepositoryDiscovery::EXCLUDED_DIRS.contains(&name.as_ref()))
        });
    for entry in entries {
        check_cancel(cancel)?;
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().is_dir() {
            continue;
        }
        if paths.len() >= MAX_FILES {
            return Err("Build input inventory exceeds 200,000 files".into());
        }
        let relative = entry
            .path()
            .strip_prefix(root)
            .map_err(|e| e.to_string())?
            .to_str()
            .ok_or("Build input path is not UTF-8")?;
        path_bytes += relative.len();
        if path_bytes > MAX_PATH_BYTES {
            return Err("Build input inventory exceeds the 64 MiB path limit".into());
        }
        paths.push(relative.to_owned());
    }
    Ok(paths)
}
