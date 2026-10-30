//! Fold roles and declaration containment from shared Swift query resources.

use super::{FoldKind, FoldRange};
use crate::source::SourceBuffer;
use std::{collections::BTreeMap, ops::Range};
use tree_sitter::{Node, Query, QueryCursor, StreamingIterator};

pub(super) fn extract(
    buffer: &SourceBuffer,
    root: Node<'_>,
    query: &Query,
    fact_limit: usize,
) -> Result<Vec<FoldRange>, String> {
    let mut cursor = QueryCursor::new();
    cursor.set_match_limit(65_536);
    let names = query.capture_names();
    let mut folds = BTreeMap::<(usize, usize), FoldRange>::new();
    let mut declarations = Vec::<Range<usize>>::new();
    let mut import_members = 0usize;
    let mut matches = cursor.matches(query, root, &buffer.data[..]);
    while let Some(found) = matches.next() {
        for capture in found.captures {
            let name = names[capture.index as usize];
            let range = capture.node.byte_range();
            if name.starts_with("symbol.") {
                declarations.push(range);
            } else if let Some(kind) = fold_kind(name) {
                let range = if name == "fold.imports.member" {
                    import_members += 1;
                    import_group(capture.node)
                } else {
                    Some(range)
                };
                if let Some(fold) =
                    range.and_then(|range| make_fold(buffer, capture.node, kind, range))
                {
                    folds
                        .entry((fold.byte_range.start, fold.byte_range.end))
                        .and_modify(|old| {
                            if fold.kind < old.kind {
                                *old = fold.clone();
                            }
                        })
                        .or_insert(fold);
                }
            }
            if folds
                .len()
                .saturating_add(declarations.len())
                .saturating_add(import_members)
                > fact_limit
            {
                return Err("Folding query exceeded its fact limit".into());
            }
        }
    }
    drop(matches);
    if cursor.did_exceed_match_limit() {
        return Err("Folding query exceeded its match limit".into());
    }
    declarations.sort_by_key(|range| (range.start, range.end));
    let mut folds: Vec<_> = folds.into_values().collect();
    for fold in &mut folds {
        let start = declarations.partition_point(|range| range.start < fold.byte_range.start);
        fold.contains_declarations = declarations[start..]
            .iter()
            .take_while(|range| range.start < fold.byte_range.end)
            .any(|range| range.end <= fold.byte_range.end);
    }
    // The source gutter currently has one disclosure per physical header row.
    // Prefer the semantic callable/type role before generic same-row blocks.
    folds.sort_by_key(|fold| {
        (
            fold.start_line,
            fold.kind,
            std::cmp::Reverse(fold.end_line),
            fold.byte_range.start,
        )
    });
    folds.dedup_by_key(|fold| fold.start_line);
    Ok(folds)
}

fn fold_kind(name: &str) -> Option<FoldKind> {
    match name {
        "fold.function" => Some(FoldKind::Callable),
        "fold.type" => Some(FoldKind::Type),
        "fold.comment" => Some(FoldKind::Comment),
        "fold.imports" | "fold.imports.member" => Some(FoldKind::Imports),
        "fold.block" => Some(FoldKind::Block),
        _ => None,
    }
}

// kotlin-ng has no import_list node. Reconstruct only a parsed sibling run;
// comments may separate members, but errors and declarations must stay visible.
fn import_group(node: Node<'_>) -> Option<Range<usize>> {
    let mut previous = node.prev_named_sibling();
    while let Some(sibling) = previous {
        match sibling.kind() {
            "import" => return None, // The first member already owns this run.
            "line_comment" | "block_comment" => previous = sibling.prev_named_sibling(),
            _ => break,
        }
    }
    let mut range = node.byte_range();
    let mut next = node.next_named_sibling();
    while let Some(sibling) = next {
        match sibling.kind() {
            "import" => range.end = sibling.end_byte(),
            "line_comment" | "block_comment" => {}
            _ => break,
        }
        next = sibling.next_named_sibling();
    }
    Some(range)
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
