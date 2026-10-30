//! Private Git-store initialization, pinned fetches and owned worktree recovery.

use super::{GitRead, RepositoryFile, ReviewControl, SourceLocation};
use std::path::{Path, PathBuf};

pub(super) struct GitStore<'a>(pub(super) GitRead<'a>);
impl GitStore<'_> {
    pub(super) fn initialize(cache: &Path, control: &ReviewControl<'_>) -> Result<PathBuf, String> {
        control.check()?;
        std::fs::create_dir_all(cache).map_err(|error| error.to_string())?;
        let cache = std::fs::canonicalize(cache).map_err(|error| error.to_string())?;
        let object_dir = cache.join("review.git");
        if existing_store(&object_dir, control)? {
            return Ok(object_dir);
        }
        let staging = tempfile::Builder::new()
            .prefix("review-init-")
            .tempdir_in(&cache)
            .map_err(|error| error.to_string())?;
        control.git(&cache).required(&[
            std::ffi::OsString::from("init"),
            "--bare".into(),
            staging.path().as_os_str().to_owned(),
        ])?;
        control.check()?;
        std::fs::rename(staging.path(), &object_dir).map_err(|error| error.to_string())?;
        Ok(object_dir)
    }

    pub(super) fn bytes(&self, args: &[&str]) -> Result<Vec<u8>, String> {
        self.0.required(args)
    }
    pub(super) fn text(&self, args: &[&str]) -> Result<String, String> {
        String::from_utf8(self.bytes(args)?)
            .map(|text| text.trim_end().to_string())
            .map_err(|error| error.to_string())
    }
    pub(super) fn files(&self, commit: &str) -> Result<Vec<RepositoryFile>, String> {
        let paths = self.bytes(&["ls-tree", "-rz", "--name-only", commit])?;
        paths
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
            .map(|path| {
                self.0.check()?;
                let path = std::str::from_utf8(path).map_err(|error| error.to_string())?;
                if !SourceLocation::is_repository_path(path) {
                    return Err("Invalid path in review tree".into());
                }
                Ok(RepositoryFile::new(path))
            })
            .collect()
    }

    pub(super) fn materialize(&self, head: &str) -> Result<tempfile::TempDir, String> {
        self.0.check()?;
        // Remove registrations from earlier interrupted attempts whose guards
        // removed their directories. Existing accepted trees remain registered.
        self.recover_missing_trees()?;
        self.bytes(&["worktree", "prune", "--expire", "now"])?;
        let revision = self
            .0
            .root
            .parent()
            .expect("private store has a cache parent")
            .join("revisions")
            .join(head);
        std::fs::create_dir_all(&revision).map_err(|error| error.to_string())?;
        // Each attempt owns its tree and cannot remove an accepted snapshot.
        let tree = tempfile::Builder::new()
            .prefix("tree-")
            .tempdir_in(&revision)
            .map_err(|error| error.to_string())?;
        self.0.required(&[
            std::ffi::OsString::from("worktree"),
            "add".into(),
            "--detach".into(),
            tree.path().as_os_str().to_owned(),
            head.into(),
        ])?;
        Ok(tree)
    }

    fn recover_missing_trees(&self) -> Result<(), String> {
        let entries = self.bytes(&["worktree", "list", "--porcelain", "-z"])?;
        let revisions = self
            .0
            .root
            .parent()
            .expect("private store parent")
            .join("revisions");
        for record in entries.split(|byte| *byte == 0) {
            self.0.check()?;
            let Some(raw) = record.strip_prefix(b"worktree ") else {
                continue;
            };
            let path = worktree_path(raw)?;
            if !owned_tree(&path, &revisions) || !missing_path(&path)? {
                continue;
            }
            self.0.required(&[
                std::ffi::OsString::from("worktree"),
                "remove".into(),
                "--force".into(),
                "--force".into(),
                path.into_os_string(),
            ])?;
        }
        Ok(())
    }

    pub(super) fn fetch(
        &self,
        origin: &str,
        oid: &str,
        number: Option<usize>,
    ) -> Result<(), String> {
        if self
            .0
            .optional(&["cat-file", "-e", &format!("{oid}^{{commit}}")])?
            .is_some()
        {
            return Ok(());
        }
        if !self.fetch_object(origin, oid)? {
            let number = number.ok_or("Unable to fetch base commit")?;
            // A unique ref proves what this request fetched without shared FETCH_HEAD.
            let reference = format!("refs/radiology/requests/{}", uuid::Uuid::new_v4());
            let refspec = format!("refs/pull/{number}/head:{reference}");
            if !self.fetch_object(origin, &refspec)? {
                return Err("Unable to fetch pinned pull request commit".into());
            }
            let fetched = self.text(&["rev-parse", &reference])?;
            self.bytes(&["update-ref", "-d", &reference])?;
            if !fetched.eq_ignore_ascii_case(oid) {
                return Err("Pull request moved during fetch; refresh and retry".into());
            }
        }
        self.bytes(&["cat-file", "-e", &format!("{oid}^{{commit}}")])?;
        Ok(())
    }

    fn fetch_object(&self, origin: &str, reference: &str) -> Result<bool, String> {
        self.0
            .optional(&[
                "-c",
                "fetch.writeCommitGraph=false",
                "-c",
                "gc.auto=0",
                "fetch",
                "--no-tags",
                "--no-write-fetch-head",
                "--no-auto-maintenance",
                origin,
                reference,
            ])
            .map(|output| output.is_some())
    }
}

fn existing_store(directory: &Path, control: &ReviewControl<'_>) -> Result<bool, String> {
    let metadata = match std::fs::symlink_metadata(directory) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.to_string()),
    };
    if !metadata.is_dir() || metadata.is_symlink() {
        return Err("Private review store must be a real directory".into());
    }
    if directory.join("HEAD").exists() {
        if control
            .git(directory)
            .required(&["rev-parse", "--is-bare-repository"])?
            != b"true\n"
        {
            return Err("Private review store is not a bare repository".into());
        }
        return Ok(true);
    }
    // Preserve legacy interrupted initialization instead of deleting unknown data.
    control.check()?;
    let quarantine = tempfile::Builder::new()
        .prefix("review-recovery-")
        .tempdir_in(directory.parent().expect("private store parent"))
        .map_err(|error| error.to_string())?;
    std::fs::rename(directory, quarantine.path().join("review.git"))
        .map_err(|error| error.to_string())?;
    let _ = quarantine.keep();
    Ok(false)
}

fn owned_tree(path: &Path, revisions: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(revisions) else {
        return false;
    };
    let mut components = relative.components();
    let (Some(std::path::Component::Normal(head)), Some(std::path::Component::Normal(tree)), None) =
        (components.next(), components.next(), components.next())
    else {
        return false;
    };
    let Some(head) = head.to_str() else {
        return false;
    };
    let Some(suffix) = tree.to_str().and_then(|tree| tree.strip_prefix("tree-")) else {
        return false;
    };
    super::required_oid(Some(head)).is_ok()
        && !suffix.is_empty()
        && suffix.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

fn missing_path(path: &Path) -> Result<bool, String> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(error.to_string()),
    }
}

#[cfg(unix)]
fn worktree_path(bytes: &[u8]) -> Result<PathBuf, String> {
    use std::os::unix::ffi::OsStringExt;
    Ok(std::ffi::OsString::from_vec(bytes.to_vec()).into())
}
#[cfg(not(unix))]
fn worktree_path(bytes: &[u8]) -> Result<PathBuf, String> {
    std::str::from_utf8(bytes)
        .map(PathBuf::from)
        .map_err(|error| error.to_string())
}
