//! Immutable pull-request snapshots in a private Git object store.
//!
//! The attached checkout supplies repository identity and its origin URL only.
//! Fetches, worktrees, and revision reads operate on the private store.

use super::change::{FileChange, FileChangeStatus};
use super::diff::{FileLineChanges, UnifiedDiffParser};
use super::github::PullRequestSummary;
use crate::repository::file_node::RepositoryFile;
use crate::source::location::ReviewRevision;
use crate::source::{SourceBuffer, SourceLocation};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    /// # Errors
    /// Returns Git or metadata errors without changing the attached checkout.
    pub fn prepare(
        attached: &Path,
        summary: PullRequestSummary,
        cache: &Path,
    ) -> Result<Self, String> {
        let head = required_oid(summary.head_ref_oid.as_deref())?;
        let base_head = required_oid(summary.base_ref_oid.as_deref())?;
        let origin = git_at(attached, &["remote", "get-url", "origin"])?;
        let object_dir = cache.join("review.git");
        std::fs::create_dir_all(cache).map_err(|error| error.to_string())?;
        if !object_dir.exists() {
            run(Command::new("git")
                .args(["init", "--bare"])
                .arg(&object_dir))?;
        }
        let store = GitStore(&object_dir);
        store.fetch(&origin, &base_head, None)?;
        store.fetch(&origin, &head, Some(summary.number))?;
        let base = store.text(&["merge-base", &base_head, &head])?;
        let head_root = cache.join("revisions").join(&head).join("tree");
        if !head_root.exists() {
            std::fs::create_dir_all(head_root.parent().expect("revision parent"))
                .map_err(|error| error.to_string())?;
            run(store
                .command()
                .args(["worktree", "add", "--detach"])
                .arg(&head_root)
                .arg(&head))?;
        }
        let paths = store.bytes(&["ls-tree", "-rz", "--name-only", &head])?;
        let files = paths
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
            .map(|path| {
                String::from_utf8(path.to_vec())
                    .map(RepositoryFile::new)
                    .map_err(|error| error.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
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
        snapshot.load_changes()?;
        Ok(snapshot)
    }

    fn load_changes(&mut self) -> Result<(), String> {
        let bytes = GitStore(&self.object_dir).bytes(&[
            "diff",
            "--name-status",
            "-z",
            "-M",
            &self.base,
            &self.head,
        ])?;
        let text = String::from_utf8(bytes).map_err(|error| error.to_string())?;
        let mut fields = text.split('\0').filter(|field| !field.is_empty());
        while let Some(status) = fields.next() {
            let first = fields.next().ok_or("Missing changed path")?;
            let (old, path) = if status.starts_with('R') || status.starts_with('C') {
                (Some(first), fields.next().ok_or("Missing renamed path")?)
            } else {
                (None, first)
            };
            self.add_change(status, old, path)?;
        }
        Ok(())
    }

    fn add_change(&mut self, status: &str, old: Option<&str>, path: &str) -> Result<(), String> {
        let store = GitStore(&self.object_dir);
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
        let status = match status.as_bytes().first() {
            Some(b'A') => FileChangeStatus::Added,
            Some(b'D') => FileChangeStatus::Deleted,
            Some(b'R') => FileChangeStatus::Renamed {
                similarity: status[1..].parse().unwrap_or(0),
            },
            Some(b'C') => FileChangeStatus::Copied,
            Some(b'M') => FileChangeStatus::Modified,
            _ => FileChangeStatus::Other(status.into()),
        };
        let change = FileChange {
            path: path.into(),
            old_path: old.map(str::to_string),
            status,
            before_blob: String::new(),
            after_blob: String::new(),
            additions: diff.total_additions(),
            deletions: diff.total_deletions(),
            is_binary: diff.is_binary,
        };
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
    /// Returns an error for an unavailable revision, invalid path, binary, or oversized source.
    pub fn read(&self, path: &str, revision: ReviewRevision) -> Result<SourceBuffer, String> {
        if !SourceLocation::is_repository_path(path) {
            return Err("Invalid repository-relative source path".into());
        }
        let commit = match revision {
            ReviewRevision::Before => &self.base,
            ReviewRevision::After => &self.head,
        };
        let object = format!("{commit}:{path}");
        let store = GitStore(&self.object_dir);
        let size: u64 = store
            .text(&["cat-file", "-s", &object])?
            .parse()
            .map_err(|_| "Invalid blob size")?;
        if size > SourceBuffer::MAX_FILE_SIZE as u64 {
            return Err("File exceeds maximum allowed size".into());
        }
        SourceBuffer::from_bytes(path, store.bytes(&["show", &object])?, Some(revision))
            .map_err(|error| error.to_string())
    }
}

fn required_oid(oid: Option<&str>) -> Result<String, String> {
    let oid = oid.ok_or("Pull request has no pinned commit metadata")?;
    if !(oid.len() == 40 || oid.len() == 64) || !oid.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("Pull request returned an invalid commit ID".into());
    }
    Ok(oid.to_string())
}

struct GitStore<'a>(&'a Path);
impl GitStore<'_> {
    fn command(&self) -> Command {
        let mut command = Command::new("git");
        command.arg("--git-dir").arg(self.0);
        command
    }
    fn bytes(&self, args: &[&str]) -> Result<Vec<u8>, String> {
        run(self.command().args(args))
    }
    fn text(&self, args: &[&str]) -> Result<String, String> {
        String::from_utf8(self.bytes(args)?)
            .map(|text| text.trim_end().to_string())
            .map_err(|error| error.to_string())
    }
    fn fetch(&self, origin: &str, oid: &str, number: Option<usize>) -> Result<(), String> {
        if self.text(&["cat-file", "-t", oid]).is_ok() {
            return Ok(());
        }
        if self.text(&["fetch", "--no-tags", origin, oid]).is_err() {
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

fn git_at(root: &Path, args: &[&str]) -> Result<String, String> {
    String::from_utf8(run(Command::new("git").current_dir(root).args(args))?)
        .map(|text| text.trim().to_string())
        .map_err(|error| error.to_string())
}

fn run(command: &mut Command) -> Result<Vec<u8>, String> {
    let output = command.output().map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(output.stdout)
}

#[cfg(test)]
pub(crate) mod tests;
