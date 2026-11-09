//! Fold roles and declaration containment from shared Swift query resources.

use super::{FoldKind, FoldRange};
use crate::source::SourceBuffer;
use std::{collections::BTreeMap, ops::Range};
use tree_sitter::{Node, Query};

pub(super) fn extract(
    buffer: &SourceBuffer,
    root: Node<'_>,
    query: &Query,
    fact_limit: usize,
    control: &super::AnalysisControl<'_>,
) -> Result<Vec<FoldRange>, String> {
    let names = query.capture_names();
    let mut folds = BTreeMap::<(usize, usize), FoldRange>::new();
    let mut declarations = Vec::<Range<usize>>::new();
    let mut imports = Vec::new();
    super::queries::visit_matches(query, root, &buffer.data, control, "Folding", |found| {
        for capture in found.captures {
            control.check()?;
            let name = names[capture.index as usize];
            let range = capture.node.byte_range();
            if name.starts_with("symbol.") {
                declarations.push(range);
            } else if name == "fold.imports.member" {
                imports.push(capture.node);
            } else if let Some(kind) = fold_kind(name) {
                if let Some(fold) = make_fold(buffer, capture.node, kind, range) {
                    insert_fold(&mut folds, fold);
                }
            }
            if folds
                .len()
                .saturating_add(declarations.len())
                .saturating_add(imports.len())
                > fact_limit
            {
                return Err("Folding query exceeded its fact limit".into());
            }
        }
        Ok(())
    })?;
    imports.sort_unstable_by_key(|node| node.start_byte());
    imports.dedup_by_key(|node| node.id());
    for (node, range) in import_groups(buffer, &imports, control)? {
        if let Some(fold) = make_fold(buffer, node, FoldKind::Imports, range) {
            insert_fold(&mut folds, fold);
        }
    }
    declarations.sort_by_key(|range| (range.start, range.end));
    let mut folds: Vec<_> = folds.into_values().collect();
    for fold in &mut folds {
        control.check()?;
        let start = declarations.partition_point(|range| range.start < fold.byte_range.start);
        fold.contains_declarations = declarations[start..]
            .iter()
            .take_while(|range| range.start < fold.byte_range.end)
            .any(|range| range.end <= fold.byte_range.end);
    }
    // Swift retains nested candidates even when they share a physical header.
    // The gutter chooses the first (outermost) range, independently of defaults.
    folds.sort_by_key(|fold| {
        (
            fold.byte_range.start,
            std::cmp::Reverse(fold.byte_range.end),
        )
    });
    Ok(folds)
}

fn fold_kind(name: &str) -> Option<FoldKind> {
    match name {
        "fold.function" => Some(FoldKind::Callable),
        "fold.type" => Some(FoldKind::Type),
        "fold.comment" => Some(FoldKind::Comment),
        "fold.imports" => Some(FoldKind::Imports),
        "fold.block" => Some(FoldKind::Block),
        _ => None,
    }
}

fn insert_fold(folds: &mut BTreeMap<(usize, usize), FoldRange>, fold: FoldRange) {
    folds
        .entry((fold.byte_range.start, fold.byte_range.end))
        .and_modify(|old| {
            if fold.kind < old.kind {
                *old = fold.clone();
            }
        })
        .or_insert(fold);
}

// kotlin-ng has no import_list node. Reconstruct the pinned Swift grammar's
// groups from parsed imports/comments, with its two-line-separator delimiter.
fn import_groups<'tree>(
    buffer: &SourceBuffer,
    imports: &[Node<'tree>],
    control: &super::AnalysisControl<'_>,
) -> Result<Vec<(Node<'tree>, Range<usize>)>, String> {
    let mut result = Vec::new();
    let mut first_import = None;
    for (index, &node) in imports.iter().enumerate() {
        control.check()?;
        let first = *first_import.get_or_insert(node);
        let (end, next) = import_tail(buffer, node, control)?;
        let separator = import_separator(buffer, end, control);
        control.check()?;
        let continues = separator.is_none()
            && next.is_some_and(|next| {
                imports
                    .get(index + 1)
                    .is_some_and(|import| import.id() == next.id())
            });
        if !continues {
            result.push((first, first.start_byte()..separator.unwrap_or(end)));
            first_import = None;
        }
    }
    Ok(result)
}

fn import_tail<'tree>(
    buffer: &SourceBuffer,
    node: Node<'tree>,
    control: &super::AnalysisControl<'_>,
) -> Result<(usize, Option<Node<'tree>>), String> {
    let mut end = node.end_byte();
    let semicolon = buffer.data.get(end.saturating_sub(1)) == Some(&b';');
    let mut next = node.next_named_sibling();
    while let Some(comment) =
        next.filter(|node| matches!(node.kind(), "line_comment" | "block_comment"))
    {
        control.check()?;
        // An explicit semicolon completes the header. Only same-line extras
        // precede its list delimiter; otherwise comments belong to the header.
        if semicolon && buffer.line_at_byte(end) != buffer.line_at_byte(comment.start_byte()) {
            break;
        }
        end = comment.end_byte();
        next = comment.next_named_sibling();
    }
    Ok((end, next))
}

fn import_separator(
    buffer: &SourceBuffer,
    end: usize,
    control: &super::AnalysisControl<'_>,
) -> Option<usize> {
    let mut cursor = end;
    for _ in 0..2 {
        while matches!(
            buffer.data.get(cursor).copied(),
            Some(b' ' | b'\t' | b'\x0b')
        ) {
            if control.is_cancelled() {
                return None;
            }
            cursor += 1;
        }
        match buffer.data.get(cursor).copied()? {
            b'\n' => cursor += 1,
            b'\r' => {
                cursor += 1;
                if buffer.data.get(cursor) == Some(&b'\n') {
                    cursor += 1;
                }
            }
            _ => return None,
        }
    }
    Some(cursor)
}

fn make_fold(
    buffer: &SourceBuffer,
    node: Node<'_>,
    kind: FoldKind,
    range: Range<usize>,
) -> Option<FoldRange> {
    let start_line = buffer.line_at_byte(range.start);
    let end_line = buffer.line_at_byte(range.end.saturating_sub(1).max(range.start));
    (end_line > start_line).then(|| FoldRange {
        start_line,
        end_line,
        header_preview: buffer.line_text(start_line).trim().to_owned(),
        kind,
        byte_range: range,
        contains_declarations: false,
        owner_range: (kind == FoldKind::Callable)
            .then(|| {
                if node.kind() == "lambda_literal" {
                    Some(node)
                } else {
                    node.parent()
                }
            })
            .flatten()
            .map(super::calls::annotated_range),
    })
}

#[cfg(test)]
pub(crate) mod tests;
