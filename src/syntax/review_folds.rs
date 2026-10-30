//! Initial PR fold visibility from exact parsed body/header ownership.

use super::{calls::CallSites, FoldRange};
use crate::{
    pull_request::diff::FileLineChanges,
    source::{ReviewRevision, SourceBuffer},
};
use std::{
    collections::{HashMap, HashSet},
    ops::Range,
};

pub(super) fn expanded_headers(
    buffer: &SourceBuffer,
    folds: &[FoldRange],
    calls: &CallSites,
    changes: &FileLineChanges,
    revision: ReviewRevision,
) -> HashSet<usize> {
    if changes.is_binary || !calls.matches(buffer) {
        return HashSet::new();
    }
    let mut owners = HashMap::new();
    for scope in calls.scopes.iter().filter(|scope| scope.has_body()) {
        owners
            .entry((
                buffer.line_at_byte(scope.body_range.as_ref().unwrap().start),
                buffer.line_at_byte(scope.body_range.as_ref().unwrap().end),
            ))
            .or_insert(scope);
    }
    let mut selected = HashSet::new();
    for hunk in &changes.hunks {
        let lines = match revision {
            ReviewRevision::Before => hunk.removed_lines(),
            ReviewRevision::After => hunk.added_lines(),
        };
        let changed = buffer.byte_offset(lines.start, 0)..buffer.byte_offset(lines.end, 0);
        let point = buffer.byte_offset(lines.start.saturating_add(1).max(1), 0);
        for fold in folds {
            let owner = owners.get(&(fold.start_line, fold.end_line));
            let body = owner
                .and_then(|owner| owner.body_range.as_ref())
                .cloned()
                .unwrap_or_else(|| {
                    buffer.byte_offset(fold.start_line, 0)
                        ..buffer.line_byte_range(fold.end_line).end
                });
            let intersects = if lines.is_empty() {
                owns_point(&body, point, buffer)
            } else {
                overlaps(&body, &changed)
                    || owner.is_some_and(|owner| overlaps(&owner.annotated_range, &changed))
            };
            if intersects {
                selected.insert(fold.start_line);
            }
        }
    }
    include_ancestors(folds, &mut selected);
    selected
}

fn include_ancestors(folds: &[FoldRange], selected: &mut HashSet<usize>) {
    let mut ancestors = Vec::<&FoldRange>::new();
    for fold in folds {
        while ancestors.last().is_some_and(|parent| {
            fold.end_line > parent.end_line || fold.start_line > parent.end_line
        }) {
            ancestors.pop();
        }
        if selected.contains(&fold.start_line) {
            for parent in ancestors.iter().rev() {
                if !selected.insert(parent.start_line) {
                    break;
                }
            }
        }
        ancestors.push(fold);
    }
}

fn overlaps(a: &Range<usize>, b: &Range<usize>) -> bool {
    a.start < b.end && b.start < a.end
}

fn owns_point(body: &Range<usize>, point: usize, buffer: &SourceBuffer) -> bool {
    body.contains(&point)
        || (point == buffer.byte_len()
            && body.end == point
            && !matches!(buffer.data.last(), Some(b'\n' | b'}')))
}
