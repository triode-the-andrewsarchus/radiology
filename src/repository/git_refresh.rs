//! Owned checkout Git snapshots prepared on the existing blocking worker pool.

use crate::{
    process::{self, OutputStream, ProcessCommand},
    pull_request::{
        change::{nul_records, FileChange, FileChangeStatus},
        diff::{DiffHunk, FileLineChanges, UnifiedDiffParser},
    },
    source::{buffer::SourceIdentity, SourceBuffer},
};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

pub(crate) struct GitSnapshot {
    pub root: PathBuf,
    pub branch: Option<String>,
    pub changes: Vec<FileChange>,
    pub source: Option<SourceIdentity>,
    pub lines: Option<FileLineChanges>,
}

pub(crate) fn read(
    root: PathBuf,
    source: Option<Arc<SourceBuffer>>,
    cancel: &AtomicBool,
) -> Result<GitSnapshot, String> {
    read_with(
        root,
        source,
        cancel,
        Path::new("git"),
        Duration::from_secs(30),
    )
}

fn read_with(
    root: PathBuf,
    source: Option<Arc<SourceBuffer>>,
    cancel: &AtomicBool,
    executable: &Path,
    timeout: Duration,
) -> Result<GitSnapshot, String> {
    let git = GitRead {
        root: &root,
        executable,
        cancel,
        deadline: Instant::now() + timeout,
    };
    let mut snapshot = GitSnapshot {
        root: root.clone(),
        branch: None,
        changes: Vec::new(),
        source: source.as_ref().map(|buffer| SourceIdentity::new(buffer)),
        lines: None,
    };
    let (inside, worktree) = git.run(&["rev-parse", "--is-inside-work-tree"])?;
    if !inside || worktree != b"true\n" {
        return Ok(snapshot);
    }
    let (ok, branch) = git.run(&["symbolic-ref", "--quiet", "--short", "HEAD"])?;
    snapshot.branch = ok
        .then(|| String::from_utf8_lossy(&branch).trim().to_owned())
        .filter(|branch| !branch.is_empty());
    let (has_head, head) = git.run(&["rev-parse", "--verify", "HEAD"])?;
    snapshot.changes = git.changes(has_head)?;
    let added_source = source.as_ref().is_some_and(|buffer| {
        snapshot
            .changes
            .iter()
            .any(|change| change.path == buffer.path && change.status == FileChangeStatus::Added)
    });
    snapshot.lines = git.source_lines(source.as_deref(), has_head && !added_source)?;
    let (still_has_head, current_head) = git.run(&["rev-parse", "--verify", "HEAD"])?;
    if still_has_head != has_head || current_head != head {
        return Err("Repository HEAD changed during refresh; retry.".into());
    }
    snapshot.changes.sort_by(|a, b| a.path.cmp(&b.path));
    git.check()?;
    Ok(snapshot)
}

fn same_source(root: &Path, buffer: &SourceBuffer) -> bool {
    SourceBuffer::read(&root.join(&buffer.path), &buffer.path, None)
        .is_ok_and(|disk| disk.content_digest() == buffer.content_digest())
}

struct GitRead<'a> {
    root: &'a Path,
    executable: &'a Path,
    cancel: &'a AtomicBool,
    deadline: Instant,
}

impl GitRead<'_> {
    fn changes(&self, has_head: bool) -> Result<Vec<FileChange>, String> {
        let mut changes = if has_head {
            self.tracked_changes()?
        } else {
            Vec::new()
        };
        let mut files = HashSet::new();
        if !has_head {
            files.extend(paths(&self.required(&["ls-files", "--cached", "-z"])?)?);
        }
        files.extend(paths(&self.required(&[
            "ls-files",
            "--others",
            "--exclude-standard",
            "-z",
        ])?)?);
        for path in files {
            self.check()?;
            if let Some(change) = added_file(self.root, &path)? {
                changes.push(change);
            }
        }
        Ok(changes)
    }

    fn tracked_changes(&self) -> Result<Vec<FileChange>, String> {
        let mut changes = FileChange::parse_name_status(&self.required(&[
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "--name-status",
            "-z",
            "-M",
            "HEAD",
            "--",
        ])?)?;
        let stats = numstat(&self.required(&[
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "--numstat",
            "-z",
            "-M",
            "HEAD",
            "--",
        ])?)?;
        for change in &mut changes {
            if let Some(&(additions, deletions, binary)) = stats.get(&change.path) {
                change.additions = additions;
                change.deletions = deletions;
                change.is_binary = binary;
            }
        }
        Ok(changes)
    }

    fn source_lines(
        &self,
        source: Option<&SourceBuffer>,
        has_head: bool,
    ) -> Result<Option<FileLineChanges>, String> {
        let Some(buffer) = source.filter(|buffer| buffer.revision.is_none()) else {
            return Ok(None);
        };
        if !same_source(self.root, buffer) {
            return Ok(None);
        }
        let lines = if has_head {
            let patch = self.required(&[
                "diff",
                "--no-ext-diff",
                "--no-textconv",
                "-U0",
                "HEAD",
                "--",
                &buffer.path,
            ])?;
            UnifiedDiffParser::parse(&String::from_utf8_lossy(&patch))
        } else {
            FileLineChanges {
                hunks: vec![DiffHunk {
                    before_start: 0,
                    before_count: 0,
                    after_start: 1,
                    after_count: buffer.text.lines().count(),
                }],
                is_binary: false,
            }
        };
        Ok(same_source(self.root, buffer).then_some(lines))
    }

    fn check(&self) -> Result<(), String> {
        if self.cancel.load(Ordering::Relaxed) {
            Err("Git refresh cancelled".into())
        } else if Instant::now() >= self.deadline {
            Err("Git refresh timed out".into())
        } else {
            Ok(())
        }
    }

    fn run(&self, arguments: &[&str]) -> Result<(bool, Vec<u8>), String> {
        self.check()?;
        let command = ProcessCommand {
            executable: self.executable.into(),
            directory: self.root.into(),
            arguments: ["-c", "color.ui=false", "-c", "core.quotepath=false"]
                .into_iter()
                .chain(arguments.iter().copied())
                .map(Into::into)
                .collect(),
            environment: [("GIT_OPTIONAL_LOCKS".into(), "0".into())].into(),
            resources: Vec::new(),
        };
        let mut bytes = Vec::new();
        let result = process::run_observed(
            &command,
            self.cancel,
            self.deadline.saturating_duration_since(Instant::now()),
            |stream, chunk| {
                if stream == OutputStream::Stdout {
                    if bytes.len().saturating_add(chunk.len()) > 16 * 1024 * 1024 {
                        return Err(std::io::Error::other("Git refresh output exceeds 16 MiB"));
                    }
                    bytes.extend_from_slice(chunk);
                }
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
        Ok((result.succeeded(), bytes))
    }

    fn required(&self, arguments: &[&str]) -> Result<Vec<u8>, String> {
        let (success, bytes) = self.run(arguments)?;
        if success {
            Ok(bytes)
        } else {
            Err(format!("Git {} failed", arguments[0]))
        }
    }
}

fn paths(bytes: &[u8]) -> Result<Vec<String>, String> {
    nul_records(bytes)?
        .map(|path| {
            if crate::source::SourceLocation::is_repository_path(path) {
                Ok(path.to_owned())
            } else {
                Err("Invalid Git source path".into())
            }
        })
        .collect()
}

fn numstat(bytes: &[u8]) -> Result<HashMap<String, (usize, usize, bool)>, String> {
    let mut fields = nul_records(bytes)?;
    let mut stats = HashMap::new();
    while let Some(record) = fields.next() {
        let mut columns = record.splitn(3, '\t');
        let added = columns.next().ok_or("Missing addition count")?;
        let removed = columns.next().ok_or("Missing deletion count")?;
        let path = match columns.next().ok_or("Missing numstat path")? {
            "" => {
                fields.next().ok_or("Missing original numstat path")?;
                fields.next().ok_or("Missing renamed numstat path")?
            }
            path => path,
        };
        let binary = added == "-" || removed == "-";
        let counts = if binary {
            (0, 0, true)
        } else {
            (
                added.parse().map_err(|_| "Invalid addition count")?,
                removed.parse().map_err(|_| "Invalid deletion count")?,
                false,
            )
        };
        stats.insert(path.to_owned(), counts);
    }
    Ok(stats)
}

fn added_file(root: &Path, path: &str) -> Result<Option<FileChange>, String> {
    let full = root.join(path);
    let metadata = match std::fs::symlink_metadata(&full) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let (additions, is_binary) = if metadata.file_type().is_symlink() {
        (1, false)
    } else if !metadata.is_file() {
        return Ok(None);
    } else if metadata.len() > SourceBuffer::MAX_FILE_SIZE as u64 {
        (0, true)
    } else {
        let bytes = std::fs::read(full).map_err(|error| error.to_string())?;
        match std::str::from_utf8(&bytes)
            .ok()
            .filter(|text| !text.contains('\0'))
        {
            Some(text) => (text.lines().count(), false),
            None => (0, true),
        }
    };
    Ok(Some(FileChange {
        path: path.into(),
        old_path: None,
        status: FileChangeStatus::Added,
        before_blob: String::new(),
        after_blob: String::new(),
        additions,
        deletions: 0,
        is_binary,
    }))
}

#[cfg(test)]
mod tests;
