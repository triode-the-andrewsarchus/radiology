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
    let mut import_members = 0usize;
    super::queries::visit_matches(query, root, &buffer.data, control, "Folding", |found| {
        for capture in found.captures {
            control.check()?;
            let name = names[capture.index as usize];
            let range = capture.node.byte_range();
            if name.starts_with("symbol.") {
                declarations.push(range);
            } else if let Some(kind) = fold_kind(name) {
                let range = if name == "fold.imports.member" {
                    import_members += 1;
                    import_group(capture.node, control)
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
        Ok(())
    })?;
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
        "fold.imports" | "fold.imports.member" => Some(FoldKind::Imports),
        "fold.block" => Some(FoldKind::Block),
        _ => None,
    }
}

// kotlin-ng has no import_list node. Reconstruct only a parsed sibling run;
// comments may separate members, but errors and declarations must stay visible.
fn import_group(node: Node<'_>, control: &super::AnalysisControl<'_>) -> Option<Range<usize>> {
    let mut previous = node.prev_named_sibling();
    while let Some(sibling) = previous {
        if control.is_cancelled() {
            return None;
        }
        match sibling.kind() {
            "import" => return None, // The first member already owns this run.
            "line_comment" | "block_comment" => previous = sibling.prev_named_sibling(),
            _ => break,
        }
    }
    let mut range = node.byte_range();
    let mut next = node.next_named_sibling();
    while let Some(sibling) = next {
        if control.is_cancelled() {
            return None;
        }
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
