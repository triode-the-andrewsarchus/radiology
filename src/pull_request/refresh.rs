//! Exact current-document restoration between two pinned review snapshots.

use super::{
    change::{FileChange, FileChangeStatus},
    control::ReviewControl,
    diff::{FileLineChanges, UnifiedDiffParser},
    review::ReviewSnapshot,
};
use crate::source::{ReviewRevision, SourceBuffer, SourceLocation};
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Instant,
};

pub(crate) struct RefreshedSource {
    pub buffer: Arc<SourceBuffer>,
    pub changes: Option<FileLineChanges>,
    pub notice: Option<String>,
}

pub(crate) fn prepare(
    old: &ReviewSnapshot,
    new: &ReviewSnapshot,
    original: &SourceBuffer,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<Option<RefreshedSource>, String> {
    let control = ReviewControl {
        deadline,
        ..ReviewControl::new(cancel)
    };
    control.check()?;
    let revision = original.revision.unwrap_or(ReviewRevision::After);
    let old_commit = commit(old, revision);
    let new_commit = commit(new, revision);
    let path = renamed_path(new, old_commit, new_commit, &original.path, &control)?;
    let Some(location) = destination(new, &path, revision) else {
        return Ok(None);
    };
    let chosen = location.revision.expect("review revision");
    let buffer = Arc::new(new.read_cancellable(&location.path, chosen, cancel, deadline)?);
    let changes = if buffer.data == original.data {
        None
    } else {
        let git = control.git(&new.object_dir);
        let before = format!("{old_commit}:{}", original.path);
        let after = format!("{}:{}", commit(new, chosen), buffer.path);
        let patch = git.required(&[
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "--no-color",
            "-U0",
            &before,
            &after,
        ])?;
        Some(UnifiedDiffParser::parse(
            &String::from_utf8(patch).map_err(|e| e.to_string())?,
        ))
    };
    control.check()?;
    Ok(Some(RefreshedSource {
        buffer,
        changes,
        notice: (chosen != revision).then(|| {
            format!(
                "This file is now available in {}; showing that revision.",
                chosen.title()
            )
        }),
    }))
}

fn commit(snapshot: &ReviewSnapshot, revision: ReviewRevision) -> &str {
    match revision {
        ReviewRevision::After => &snapshot.head,
        ReviewRevision::Before => &snapshot.base,
    }
}

fn renamed_path(
    new: &ReviewSnapshot,
    old: &str,
    fresh: &str,
    path: &str,
    control: &ReviewControl<'_>,
) -> Result<String, String> {
    if old == fresh {
        return Ok(path.into());
    }
    let bytes = control.git(&new.object_dir).required(&[
        "diff",
        "--name-status",
        "-z",
        "-M",
        old,
        fresh,
    ])?;
    Ok(FileChange::parse_name_status(&bytes)?
        .into_iter()
        .find(|change| {
            matches!(change.status, FileChangeStatus::Renamed { .. })
                && change.old_path.as_deref() == Some(path)
        })
        .map_or_else(|| path.to_owned(), |change| change.path))
}

fn destination(
    snapshot: &ReviewSnapshot,
    path: &str,
    revision: ReviewRevision,
) -> Option<SourceLocation> {
    if snapshot.change(path).is_none() && !snapshot.files.iter().any(|file| file.path == path) {
        return None;
    }
    [
        revision,
        match revision {
            ReviewRevision::After => ReviewRevision::Before,
            ReviewRevision::Before => ReviewRevision::After,
        },
    ]
    .into_iter()
    .find_map(|revision| {
        snapshot
            .resolve(SourceLocation::with_revision(path, 0, Some(revision)))
            .ok()
    })
}

pub(crate) fn map_byte(
    original: &SourceBuffer,
    fresh: &SourceBuffer,
    changes: Option<&FileLineChanges>,
    byte: usize,
) -> usize {
    if original.data == fresh.data {
        return fresh.clamp_to_char_boundary(byte.min(fresh.data.len()));
    }
    let old_line = original.line_at_byte(byte);
    let new_line = changes
        .map_or(old_line, |changes| {
            changes.map_line(old_line, ReviewRevision::Before)
        })
        .min(fresh.line_count())
        .max(1);
    let before = original.line_byte_range(old_line);
    let after = fresh.line_byte_range(new_line);
    let mapped = if original.data[before.clone()] == fresh.data[after.clone()] {
        after.start + byte.saturating_sub(before.start).min(after.len())
    } else {
        after.start
    };
    fresh.clamp_to_char_boundary(mapped)
}
