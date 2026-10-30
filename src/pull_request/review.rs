//! Immutable pull-request snapshots in a private Git object store.
//!
//! The attached checkout supplies repository identity and its origin URL only.
//! Fetches, worktrees, and revision reads operate on the private store.

use super::change::{FileChange, FileChangeStatus};
use super::control::ReviewControl;
use super::diff::{FileLineChanges, UnifiedDiffParser};
use super::github::PullRequestSummary;
use crate::repository::{file_node::RepositoryFile, git::GitRead};
use crate::source::location::ReviewRevision;
use crate::source::{SourceBuffer, SourceLocation};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};

/// A review whose source, hunks, and file list belong to pinned commits.
#[derive(Debug, Clone)]
pub struct ReviewSnapshot {
    /// GitHub metadata captured when the snapshot was prepared.
    pub summary: PullRequestSummary,
    /// Pinned head commit.
    pub head: String,
    /// Pinned merge base used by GitHub's three-dot comparison.
    pub base: String,
    /// Private bare object database.
    pub object_dir: PathBuf,
    /// Head worktree available for syntax and batch index generation.
    pub head_root: PathBuf,
    /// Files at the head revision, before presentation filtering.
    pub files: Vec<RepositoryFile>,
    /// Changed files, including removed files absent from the head tree.
    pub changes: Vec<FileChange>,
    /// Per-file exact zero-context hunks, including both rename paths.
    pub diffs: HashMap<String, FileLineChanges>,
}

impl ReviewSnapshot {
    /// Prepares pinned commits and an isolated head tree.
    ///
    /// Git children share a two-minute deadline. Failed attempts remove their
    /// temporary trees; accepted snapshots keep independently owned trees.
    ///
    /// # Errors
    /// Returns Git or metadata errors without changing the attached checkout.
    pub fn prepare(
        attached: &Path,
        summary: PullRequestSummary,
        cache: &Path,
    ) -> Result<Self, String> {
        Self::prepare_cancellable(
            attached,
            summary,
            cache,
            &ReviewControl::new(&AtomicBool::new(false)),
        )
    }

    pub(crate) fn prepare_cancellable(
        attached: &Path,
        summary: PullRequestSummary,
        cache: &Path,
        control: &ReviewControl<'_>,
    ) -> Result<Self, String> {
        control.check()?;
        let head = required_oid(summary.head_ref_oid.as_deref())?;
        let base_head = required_oid(summary.base_ref_oid.as_deref())?;
        let origin = String::from_utf8(
            control
                .git(attached)
                .required(&["remote", "get-url", "origin"])?,
        )
        .map_err(|error| error.to_string())?;
        let object_dir = GitStore::initialize(cache, control)?;
        let store = GitStore(control.git(&object_dir));
        store.fetch(origin.trim(), &base_head, None)?;
        store.fetch(origin.trim(), &head, Some(summary.number))?;
        let base = store.text(&["merge-base", &base_head, &head])?;
        let tree = store.materialize(&head)?;
        let head_root = tree.path().to_owned();
        let files = store.files(&head)?;
        let mut snapshot = Self {
            summary,
            head,
            base,
            object_dir,
            head_root,
            files,
            changes: Vec::new(),
            diffs: HashMap::new(),
        };
        snapshot.load_changes(control)?;
        control.check()?;
        let _ = tree.keep();
        Ok(snapshot)
    }

    fn load_changes(&mut self, control: &ReviewControl<'_>) -> Result<(), String> {
        control.check()?;
        let bytes = GitStore(control.git(&self.object_dir)).bytes(&[
            "diff",
            "--name-status",
            "-z",
            "-M",
            &self.base,
            &self.head,
        ])?;
        for change in FileChange::parse_name_status(&bytes)? {
            control.check()?;
            self.add_change(change, control)?;
        }
        Ok(())
    }

    fn add_change(
        &mut self,
        mut change: FileChange,
        control: &ReviewControl<'_>,
    ) -> Result<(), String> {
        let path = change.path.as_str();
        let old = change.old_path.as_deref();
        let store = GitStore(control.git(&self.object_dir));
        let patch = store.text(&[
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "-U0",
            "-M",
            &self.base,
            &self.head,
            "--",
            old.unwrap_or(path),
            path,
        ])?;
        let diff = UnifiedDiffParser::parse(&patch);
        change.additions = diff.total_additions();
        change.deletions = diff.total_deletions();
        change.is_binary = diff.is_binary;
        if let Some(old) = old {
            self.diffs.insert(old.into(), diff.clone());
        }
        self.diffs.insert(path.into(), diff);
        self.changes.push(change);
        Ok(())
    }

    /// Returns a changed file under either of its revision-specific paths.
    #[must_use]
    pub fn change(&self, path: &str) -> Option<&FileChange> {
        self.changes
            .iter()
            .find(|change| change.path == path || change.old_path.as_deref() == Some(path))
    }

    /// Resolves the requested path to a revision, choosing Before for deleted files.
    ///
    /// # Errors
    /// Returns an error when an explicitly requested revision has no such file.
    pub fn resolve(&self, mut location: SourceLocation) -> Result<SourceLocation, String> {
        let change = self.change(&location.path);
        let revision = location.revision.unwrap_or_else(|| {
            if change.is_some_and(|change| change.status == FileChangeStatus::Deleted) {
                ReviewRevision::Before
            } else {
                ReviewRevision::After
            }
        });
        if let Some(change) = change {
            location.path = change
                .path_for_revision(revision)
                .ok_or_else(|| format!("{} is unavailable in {}", location.path, revision.title()))?
                .to_string();
        }
        location.revision = Some(revision);
        Ok(location)
    }

    /// Reads exact review bytes, never the attached checkout's contents.
    ///
    /// # Errors
    /// Returns an error for an unavailable revision, invalid path, binary, oversized
    /// source, subprocess failure, or a read exceeding its 30-second deadline.
    pub fn read(&self, path: &str, revision: ReviewRevision) -> Result<SourceBuffer, String> {
        self.read_cancellable(
            path,
            revision,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(30),
        )
    }

    pub(crate) fn read_cancellable(
        &self,
        path: &str,
        revision: ReviewRevision,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> Result<SourceBuffer, String> {
        self.read_from_git(
            path,
            revision,
            &GitRead {
                root: &self.object_dir,
                executable: Path::new("git"),
                cancel,
                deadline,
            },
        )
    }

    fn read_from_git(
        &self,
        path: &str,
        revision: ReviewRevision,
        git: &GitRead<'_>,
    ) -> Result<SourceBuffer, String> {
        git.check()?;
        if !SourceLocation::is_repository_path(path) {
            return Err("Invalid repository-relative source path".into());
        }
        let commit = match revision {
            ReviewRevision::Before => &self.base,
            ReviewRevision::After => &self.head,
        };
        let object = format!("{commit}:{path}");
        let size = git.required(&["cat-file", "-s", &object])?;
        let size: usize = std::str::from_utf8(&size)
            .map_err(|error| error.to_string())?
            .trim()
            .parse()
            .map_err(|_| "Invalid blob size")?;
        if size > SourceBuffer::MAX_FILE_SIZE {
            return Err("File exceeds maximum allowed size".into());
        }
        let bytes = git.required(&["cat-file", "blob", &object])?;
        if bytes.len() != size {
            return Err("Pinned blob size does not match its metadata".into());
        }
        let buffer = SourceBuffer::from_bytes(path, bytes, Some(revision))
            .map_err(|error| error.to_string())?;
        git.check()?;
        Ok(buffer)
    }
}

fn required_oid(oid: Option<&str>) -> Result<String, String> {
    let oid = oid.ok_or("Pull request has no pinned commit metadata")?;
    if !(oid.len() == 40 || oid.len() == 64) || !oid.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("Pull request returned an invalid commit ID".into());
    }
    Ok(oid.to_string())
}

struct GitStore<'a>(GitRead<'a>);
impl GitStore<'_> {
    fn initialize(cache: &Path, control: &ReviewControl<'_>) -> Result<PathBuf, String> {
        control.check()?;
        std::fs::create_dir_all(cache).map_err(|error| error.to_string())?;
        let cache = std::fs::canonicalize(cache).map_err(|error| error.to_string())?;
        let object_dir = cache.join("review.git");
        // Reinitializing is idempotent and repairs an interrupted init.
        control.git(&cache).required(&[
            std::ffi::OsString::from("init"),
            "--bare".into(),
            object_dir.as_os_str().to_owned(),
        ])?;
        Ok(object_dir)
    }

    fn bytes(&self, args: &[&str]) -> Result<Vec<u8>, String> {
        self.0.required(args)
    }
    fn text(&self, args: &[&str]) -> Result<String, String> {
        String::from_utf8(self.bytes(args)?)
            .map(|text| text.trim_end().to_string())
            .map_err(|error| error.to_string())
    }
    fn files(&self, commit: &str) -> Result<Vec<RepositoryFile>, String> {
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

    fn materialize(&self, head: &str) -> Result<tempfile::TempDir, String> {
        self.0.check()?;
        // Remove registrations from earlier interrupted attempts whose guards
        // removed their directories. Existing accepted trees remain registered.
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

    fn fetch(&self, origin: &str, oid: &str, number: Option<usize>) -> Result<(), String> {
        if self.0.optional(&["cat-file", "-t", oid])?.is_some() {
            return Ok(());
        }
        if self
            .0
            .optional(&["fetch", "--no-tags", origin, oid])?
            .is_none()
        {
            let reference = format!(
                "refs/pull/{}/head",
                number.ok_or("Unable to fetch base commit")?
            );
            self.text(&["fetch", "--no-tags", origin, &reference])?;
            if self.text(&["rev-parse", "FETCH_HEAD"])?.trim() != oid {
                return Err("Pull request moved during fetch; refresh and retry".into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod tests;

#[cfg(all(test, unix))]
mod cancellation_tests;
