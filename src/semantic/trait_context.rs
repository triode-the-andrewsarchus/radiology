//! Exact Rust trait context, independent of reference and declaration selection.

use super::{index::SemanticSymbolID, reference_results::check_cancel, SemanticIndex};
use crate::{
    source::{ReviewRevision, SourceBuffer},
    syntax::{DocumentSyntax, LanguageRegistry, SupportedGrammar, SymbolKind},
};
use std::{collections::HashSet, ops::Range, sync::atomic::AtomicBool};

/// Finds the trait under the caret or enclosing a member or explicit impl body.
///
/// `syntax` must belong to the supplied immutable `buffer`. Hosts should use the
/// accepted source snapshot and run this alongside their other context preparation.
/// No files are read or parsed here. A trait occurrence (SCIP kind 42 or 53) wins
/// over its enclosing trait. Otherwise the innermost parsed trait declaration or
/// explicit trait impl supplies an exact token to resolve. Identity never falls
/// back to spelling, relationships or implementing-type symbols.
///
/// Before revisions, stale source, non-Rust files and missing or ambiguous semantic
/// identities produce `None`. A trait can be selected even when it has no impls.
///
/// # Examples
/// A parsed trait with no semantic identity does not manufacture a context.
///
/// ```
/// use radiology::{semantic::{SemanticIndex, trait_context::trait_at_byte},
///     source::SourceBuffer, syntax::DocumentSyntax};
/// use std::sync::atomic::AtomicBool;
/// let buffer = SourceBuffer::from_str("reader.rs", "trait Read { fn read(&self); }");
/// let syntax = DocumentSyntax::analyze(&buffer);
/// let byte = buffer.text.find("read(").unwrap();
/// assert!(trait_at_byte(&buffer, &syntax, &SemanticIndex::default(), byte,
///     &AtomicBool::new(false))?.is_none());
/// # Ok::<(), String>(())
/// ```
///
/// # Errors
/// Returns an error if canceled, without returning partial context.
pub fn trait_at_byte(
    buffer: &SourceBuffer,
    syntax: &DocumentSyntax,
    index: &SemanticIndex,
    byte: usize,
    cancel: &AtomicBool,
) -> Result<Option<SemanticSymbolID>, String> {
    check_cancel(cancel)?;
    if buffer.revision == Some(ReviewRevision::Before)
        || !index.matches_buffer(buffer)
        || LanguageRegistry::for_path(&buffer.path).grammar != Some(SupportedGrammar::Rust)
        || byte > buffer.text.len()
    {
        return Ok(None);
    }
    if let Some(id) = caret_trait(index, buffer, byte, cancel)? {
        return Ok(Some(id));
    }
    let mut enclosing = Vec::new();
    for symbol in &syntax.symbols {
        check_cancel(cancel)?;
        symbol.breadcrumbs(byte, &mut enclosing);
    }
    if let Some(declaration) = enclosing.iter().rev().find(|s| s.kind == SymbolKind::Trait) {
        if let Some(id) = exact_identity(index, buffer, &declaration.selection_range, true, cancel)?
        {
            return Ok(Some(id));
        }
    }
    let implementation = syntax
        .rust_trait_impls
        .implementations
        .iter()
        .filter(|item| item.byte_range.contains(&byte))
        .min_by_key(|item| item.byte_range.len());
    match implementation {
        Some(item) => exact_identity(index, buffer, &item.trait_name_range, false, cancel),
        None => Ok(None),
    }
}

fn caret_trait(
    index: &SemanticIndex,
    buffer: &SourceBuffer,
    byte: usize,
    cancel: &AtomicBool,
) -> Result<Option<SemanticSymbolID>, String> {
    let Some(occurrence) = index.occurrence_at_byte(buffer, byte).filter(|occurrence| {
        let id = SemanticSymbolID::new(&occurrence.symbol, &buffer.path);
        occurrence.symbol_roles & super::scip_proto::SCIP_ROLE_IMPORT == 0
            && index
                .symbol_info_for_id(&id)
                .is_some_and(|info| matches!(info.kind, 42 | 53))
    }) else {
        return Ok(None);
    };
    let Some(range) = occurrence
        .range
        .as_ref()
        .and_then(|range| index.byte_range(buffer, range))
    else {
        return Ok(None);
    };
    exact_identity(index, buffer, &range, occurrence.is_definition(), cancel)
}

fn exact_identity(
    index: &SemanticIndex,
    buffer: &SourceBuffer,
    token: &Range<usize>,
    definition: bool,
    cancel: &AtomicBool,
) -> Result<Option<SemanticSymbolID>, String> {
    let Some(document) = index.documents.get(&buffer.path) else {
        return Ok(None);
    };
    let mut identities = HashSet::new();
    for occurrence in &document.occurrences {
        check_cancel(cancel)?;
        if occurrence.is_definition() == definition
            && !occurrence.symbol.is_empty()
            && (definition || !occurrence.symbol.starts_with("local "))
            && occurrence
                .range
                .as_ref()
                .is_some_and(|range| index.byte_range(buffer, range).as_ref() == Some(token))
        {
            identities.insert(occurrence.symbol.as_str());
        }
    }
    Ok((identities.len() == 1)
        .then(|| SemanticSymbolID::new(identities.into_iter().next().unwrap(), &buffer.path)))
}

#[cfg(test)]
mod tests;
