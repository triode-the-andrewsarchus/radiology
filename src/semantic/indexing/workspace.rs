//! Disposable copies of real build inputs with byte/type/mode verification.

use super::inventory::{check_cancel, validate_input_path, InputInventory};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File, Metadata};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::AtomicBool;

/// Copy limits applied before and during materialization.
#[derive(Debug, Clone, Copy)]
pub struct WorkspaceLimits {
    /// Maximum bytes copied into one workspace.
    pub total_bytes: u64,
    /// Maximum bytes in one input file.
    pub file_bytes: u64,
}

impl Default for WorkspaceLimits {
    fn default() -> Self {
        Self {
            total_bytes: 2 * 1024 * 1024 * 1024,
            file_bytes: 256 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Identity {
    blob: String,
    sha256: String,
    mode: u32,
    link: bool,
}

#[derive(Debug)]
struct InputRecord {
    original: Identity,
    materialized: Identity,
}

/// Owns a unique private workspace and removes it on every drop path.
#[derive(Debug)]
pub struct IndexingWorkspace {
    directory: tempfile::TempDir,
    source: PathBuf,
    inputs: BTreeMap<String, InputRecord>,
    limits: WorkspaceLimits,
}

impl IndexingWorkspace {
    /// Copies a validated inventory into a disposable directory outside `source`.
    ///
    /// Internal links are redirected inside the copy. External Markdown links
    /// become literal link text, matching Swift; external source/build links fail.
    /// Git submodules/directories must be omitted by the inventory, never invented.
    ///
    /// # Errors
    ///
    /// Returns path, copy, limit, cancellation or unsupported input errors.
    pub fn create(
        source: &Path,
        inventory: &InputInventory,
        parent: &Path,
        limits: WorkspaceLimits,
        cancel: &AtomicBool,
    ) -> Result<Self, String> {
        check_cancel(cancel)?;
        let source = fs::canonicalize(source).map_err(|e| e.to_string())?;
        let parent = destination_path(parent)?;
        if parent.starts_with(&source) {
            return Err("Index workspaces must be outside the viewed repository".into());
        }
        // Validate the entire inventory before creating anything.
        for path in &inventory.paths {
            validate_input_path(path)?;
        }
        fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
        let directory = tempfile::Builder::new()
            .prefix("radiology-build-inputs-")
            .tempdir_in(&parent)
            .map_err(|e| e.to_string())?;
        let mut workspace = Self {
            directory,
            source,
            inputs: BTreeMap::new(),
            limits,
        };
        workspace.materialize(inventory, cancel)?;
        Ok(workspace)
    }

    fn materialize(
        &mut self,
        inventory: &InputInventory,
        cancel: &AtomicBool,
    ) -> Result<(), String> {
        let mut total = 0;
        // Do not create links until every regular-file destination is materialized.
        let mut links = Vec::new();
        for path in &inventory.paths {
            check_cancel(cancel)?;
            let input = self.source.join(path);
            reject_link_parents(&self.source, path)?;
            let metadata = fs::symlink_metadata(&input).map_err(|e| format!("{path}: {e}"))?;
            if metadata.file_type().is_symlink() {
                links.push(path);
                continue;
            }
            if !metadata.is_file() {
                return Err(format!("Unsupported build input: {path}"));
            }
            let destination = self.root().join(path);
            fs::create_dir_all(destination.parent().expect("input parent"))
                .map_err(|e| e.to_string())?;
            let identity = copy_file(
                &input,
                &destination,
                &metadata,
                self.limits,
                &mut total,
                cancel,
            )?;
            self.inputs.insert(
                path.clone(),
                InputRecord {
                    original: identity.clone(),
                    materialized: identity,
                },
            );
        }
        for path in links {
            check_cancel(cancel)?;
            let record = self.copy_link(path, &mut total)?;
            self.inputs.insert(path.clone(), record);
        }
        Ok(())
    }

    /// Returns the isolated repository root for child working directories.
    #[must_use]
    pub fn root(&self) -> &Path {
        self.directory.path()
    }

    /// Returns the exact digest for a regular source input, excluding link text.
    #[must_use]
    pub fn source_digest(&self, path: &str) -> Option<&str> {
        self.inputs
            .get(path)
            .filter(|input| !input.original.link)
            .map(|input| input.original.sha256.as_str())
    }

    /// Returns a deterministic SHA-256 fingerprint of original paths, bytes and modes.
    #[must_use]
    pub fn fingerprint(&self) -> String {
        let mut hash = Sha256::new();
        hash.update(b"radiology-build-inputs-v1\0");
        for (path, input) in &self.inputs {
            hash.update(path.as_bytes());
            hash.update([0]);
            hash.update(input.original.sha256.as_bytes());
            hash.update([0]);
            hash.update(input.original.mode.to_le_bytes());
            hash.update([u8::from(input.original.link)]);
        }
        format!("{:x}", hash.finalize())
    }

    /// Checks that copied inputs still have their original materialized bytes/modes.
    ///
    /// # Errors
    ///
    /// Rejects rewritten, replaced, missing or newly linked input paths.
    pub fn verify_inputs(&self, cancel: &AtomicBool) -> Result<(), String> {
        self.verify(self.root(), false, cancel)
    }

    /// Checks that the viewed tree still matches the bytes copied for this run.
    ///
    /// # Errors
    ///
    /// Rejects changed listed inputs. A fresh inventory is separately required to
    /// detect added files before claiming a current repository-wide index.
    pub fn verify_source(&self, cancel: &AtomicBool) -> Result<(), String> {
        self.verify(&self.source, true, cancel)
    }

    /// Checks copied source blob identities against an exact Git snapshot inventory.
    ///
    /// # Errors
    ///
    /// Rejects missing, extra or changed paths rather than indexing modified head files.
    pub fn verify_snapshot(&self, expected: &BTreeMap<String, String>) -> Result<(), String> {
        if self.inputs.len() != expected.len() {
            return Err("Snapshot input inventory changed".into());
        }
        for (path, input) in &self.inputs {
            if expected.get(path) != Some(&input.original.blob) {
                return Err(format!("Snapshot input changed: {path}"));
            }
        }
        Ok(())
    }

    fn verify(&self, root: &Path, original: bool, cancel: &AtomicBool) -> Result<(), String> {
        for (path, input) in &self.inputs {
            check_cancel(cancel)?;
            reject_link_parents(root, path)?;
            let identity = read_identity(&root.join(path), self.limits.file_bytes, cancel)
                .map_err(|e| format!("{path}: {e}"))?;
            let expected = if original {
                &input.original
            } else {
                &input.materialized
            };
            if identity != *expected {
                return Err(format!("Build input changed or was rewritten: {path}"));
            }
        }
        Ok(())
    }

    fn copy_link(&self, path: &str, total: &mut u64) -> Result<InputRecord, String> {
        let source = self.source.join(path);
        let text = fs::read_link(&source).map_err(|e| e.to_string())?;
        let bytes = link_bytes(&text)?;
        add_budget(bytes.len() as u64, self.limits, total)?;
        let original = bytes_identity(bytes, 0, true);
        let target = if text.is_absolute() {
            text.clone()
        } else {
            source.parent().expect("link parent").join(&text)
        };
        let target = normalize(&target)?;
        let resolved = fs::canonicalize(&target).unwrap_or(target);
        let destination = self.root().join(path);
        fs::create_dir_all(destination.parent().expect("link parent"))
            .map_err(|e| e.to_string())?;
        let materialized = if let Ok(relative) = resolved.strip_prefix(&self.source) {
            let copied_target = self.root().join(relative);
            create_link(&copied_target, &destination)?;
            bytes_identity(link_bytes(&copied_target)?, 0, true)
        } else {
            let extension = source
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if !["md", "markdown"].contains(&extension.as_str()) {
                return Err(format!("Cannot index an external build input: {path}"));
            }
            fs::write(&destination, bytes).map_err(|e| e.to_string())?;
            bytes_identity(
                bytes,
                file_mode(&fs::metadata(&destination).map_err(|e| e.to_string())?),
                false,
            )
        };
        Ok(InputRecord {
            original,
            materialized,
        })
    }
}

fn add_budget(bytes: u64, limits: WorkspaceLimits, total: &mut u64) -> Result<(), String> {
    *total = total
        .checked_add(bytes)
        .ok_or("Build input byte count overflow")?;
    if bytes > limits.file_bytes || *total > limits.total_bytes {
        Err("Build inputs exceed the configured workspace byte limit".into())
    } else {
        Ok(())
    }
}

fn copy_file(
    source: &Path,
    destination: &Path,
    metadata: &Metadata,
    limits: WorkspaceLimits,
    total: &mut u64,
    cancel: &AtomicBool,
) -> Result<Identity, String> {
    add_budget(metadata.len(), limits, total)?;
    let mut file = open_input(source)?;
    let mut output = File::create_new(destination).map_err(|e| e.to_string())?;
    let identity = hash_file(
        &mut file,
        metadata.len(),
        file_mode(metadata),
        cancel,
        |bytes| output.write_all(bytes).map_err(|e| e.to_string()),
    )?;
    fs::set_permissions(destination, metadata.permissions()).map_err(|e| e.to_string())?;
    Ok(identity)
}

fn open_input(path: &Path) -> Result<File, String> {
    let mut options = File::options();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(path).map_err(|e| e.to_string())?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("Input was replaced by a nonregular file".into());
    }
    Ok(file)
}

fn read_identity(path: &Path, limit: u64, cancel: &AtomicBool) -> Result<Identity, String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if metadata.file_type().is_symlink() {
        return Ok(bytes_identity(
            link_bytes(&fs::read_link(path).map_err(|e| e.to_string())?)?,
            0,
            true,
        ));
    }
    if !metadata.is_file() || metadata.len() > limit {
        return Err("Input type or size changed".into());
    }
    let mut file = open_input(path)?;
    hash_file(
        &mut file,
        metadata.len(),
        file_mode(&metadata),
        cancel,
        |_| Ok(()),
    )
}

fn hash_file(
    file: &mut File,
    size: u64,
    mode: u32,
    cancel: &AtomicBool,
    mut consume: impl FnMut(&[u8]) -> Result<(), String>,
) -> Result<Identity, String> {
    let mut blob = Sha1::new();
    blob.update(format!("blob {size}\0"));
    let mut digest = Sha256::new();
    let mut copied = 0;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        check_cancel(cancel)?;
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        copied += count as u64;
        if copied > size {
            return Err("Input grew while preparing the index".into());
        }
        let bytes = &buffer[..count];
        blob.update(bytes);
        digest.update(bytes);
        consume(bytes)?;
    }
    if copied != size {
        return Err("Input shrank while preparing the index".into());
    }
    Ok(Identity {
        blob: format!("{:x}", blob.finalize()),
        sha256: format!("{:x}", digest.finalize()),
        mode,
        link: false,
    })
}

fn bytes_identity(bytes: &[u8], mode: u32, link: bool) -> Identity {
    let mut blob = Sha1::new();
    blob.update(format!("blob {}\0", bytes.len()));
    blob.update(bytes);
    Identity {
        blob: format!("{:x}", blob.finalize()),
        sha256: format!("{:x}", Sha256::digest(bytes)),
        mode,
        link,
    }
}

fn reject_link_parents(root: &Path, path: &str) -> Result<(), String> {
    let mut parent = root.to_path_buf();
    let mut components = Path::new(path).components().peekable();
    while let Some(component) = components.next() {
        if components.peek().is_none() {
            break;
        }
        parent.push(component);
        if fs::symlink_metadata(&parent)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
        {
            return Err(format!(
                "Build input traverses a symbolic-link directory: {path}"
            ));
        }
    }
    Ok(())
}

fn normalize(path: &Path) -> Result<PathBuf, String> {
    let mut normalized = PathBuf::new();
    for part in path.components() {
        match part {
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err("Input path escapes the filesystem root".into());
                }
            }
            Component::CurDir => {}
            _ => normalized.push(part.as_os_str()),
        }
    }
    Ok(normalized)
}

fn destination_path(path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err("Workspace parent must be absolute".into());
    }
    let path = normalize(path)?;
    let mut prefix = path.as_path();
    let mut missing = Vec::new();
    while !prefix.exists() {
        missing.push(
            prefix
                .file_name()
                .ok_or("Invalid workspace parent")?
                .to_owned(),
        );
        prefix = prefix.parent().ok_or("Invalid workspace parent")?;
    }
    let mut resolved = fs::canonicalize(prefix).map_err(|e| e.to_string())?;
    for component in missing.into_iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

#[cfg(unix)]
fn link_bytes(path: &Path) -> Result<&[u8], String> {
    use std::os::unix::ffi::OsStrExt;
    Ok(path.as_os_str().as_bytes())
}
#[cfg(not(unix))]
fn link_bytes(path: &Path) -> Result<&[u8], String> {
    path.to_str()
        .map(str::as_bytes)
        .ok_or("Link target is not UTF-8".into())
}

#[cfg(unix)]
fn create_link(target: &Path, destination: &Path) -> Result<(), String> {
    std::os::unix::fs::symlink(target, destination).map_err(|e| e.to_string())
}
#[cfg(not(unix))]
fn create_link(_: &Path, _: &Path) -> Result<(), String> {
    Err("Build-input symbolic links require a POSIX host".into())
}

#[cfg(unix)]
fn file_mode(metadata: &Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o777
}
#[cfg(not(unix))]
fn file_mode(metadata: &Metadata) -> u32 {
    u32::from(metadata.permissions().readonly())
}

#[cfg(test)]
mod tests;
