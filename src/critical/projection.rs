//! Call-focused Critical Lens projection from exact syntax and semantic identity.

use super::lens::{
    Criticality, CriticalityOverrides, CriticalityPolicy, SCIPSymbolHeader, SourceProjection,
};
use crate::semantic::index::SemanticSymbolID;
use crate::semantic::SemanticIndex;
use crate::source::{ReviewRevision, SourceBuffer};
use crate::syntax::calls::CallSites;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, Ordering};

impl CriticalityPolicy {
    /// Builds package ownership from indexed definitions, without spelling heuristics.
    #[must_use]
    pub fn for_index(index: Option<&SemanticIndex>, overrides: CriticalityOverrides) -> Self {
        let repository_packages = index
            .into_iter()
            .flat_map(|index| index.definitions.keys())
            .filter(|id| !id.is_local())
            .filter_map(|id| SCIPSymbolHeader::parse(&id.symbol))
            .filter_map(|header| header.package_name().map(str::to_owned))
            .collect();
        Self::new(repository_packages, overrides)
    }
}

/// Computes canonical receded bytes, keeping critical and unknown call names prominent.
///
/// Syntax establishes a call; indexed identity can demote an ordinary callee. No
/// identifier-name guessing is used. Unsupported syntax leaves the source unchanged;
/// supported syntax with no calls recedes the whole document. Stale/missing indexes,
/// unsupported encodings and Before revisions retain unresolved names prominently.
///
/// # Errors
///
/// Returns a cancellation error before publication. No files or UI state are changed.
pub fn make(
    buffer: &SourceBuffer,
    calls: &CallSites,
    index: Option<&SemanticIndex>,
    policy: &CriticalityPolicy,
    cancel: &AtomicBool,
) -> Result<SourceProjection, String> {
    check_cancel(cancel)?;
    if !calls.matches(buffer) {
        return Ok(SourceProjection::new(buffer, Vec::new()));
    }
    let index = index.filter(|index| {
        buffer.revision != Some(ReviewRevision::Before) && index.matches_buffer(buffer)
    });
    let occurrences = indexed_calls(buffer, index);
    let mut memo = HashMap::new();
    let mut prominent = Vec::new();
    for site in &calls.sites {
        check_cancel(cancel)?;
        let Some(name) = site.name.as_ref().filter(|range| {
            !range.is_empty()
                && range.end <= buffer.byte_len()
                && buffer.text.is_char_boundary(range.start)
                && buffer.text.is_char_boundary(range.end)
        }) else {
            continue;
        };
        let id = resolve(name, &occurrences);
        let decision = id.zip(index).map(|(id, index)| {
            *memo.entry(id.clone()).or_insert_with(|| {
                let signature = index
                    .symbol_info_for_id(id)
                    .map(|information| information.signature.as_str());
                policy
                    .classify(&id.symbol, signature, index.definitions.contains_key(id))
                    .criticality
            })
        });
        if decision != Some(Criticality::Ordinary) {
            prominent.push(name.clone());
        }
    }
    let mut receded = Vec::new();
    let mut cursor = 0;
    for name in SourceProjection::normalized(prominent) {
        if cursor < name.start {
            receded.push(cursor..name.start);
        }
        cursor = name.end;
    }
    if cursor < buffer.byte_len() {
        receded.push(cursor..buffer.byte_len());
    }
    check_cancel(cancel)?;
    Ok(SourceProjection::new(buffer, receded))
}

type IndexedCalls = HashMap<usize, Vec<(Range<usize>, SemanticSymbolID)>>;

fn indexed_calls(buffer: &SourceBuffer, index: Option<&SemanticIndex>) -> IndexedCalls {
    let mut occurrences: IndexedCalls = HashMap::new();
    if let Some(index) = index {
        if let Some(document) = index.documents.get(&buffer.path) {
            for occurrence in &document.occurrences {
                let Some(range) = occurrence
                    .range
                    .as_ref()
                    .filter(|range| range.start_line == range.end_line)
                    .and_then(|range| index.byte_range(buffer, range))
                    .filter(|range| !range.is_empty())
                else {
                    continue;
                };
                occurrences.entry(range.start).or_default().push((
                    range,
                    SemanticSymbolID::new(occurrence.symbol.clone(), &buffer.path),
                ));
            }
        }
    }
    occurrences
}

fn resolve<'a>(name: &Range<usize>, occurrences: &'a IndexedCalls) -> Option<&'a SemanticSymbolID> {
    let hits = occurrences.get(&name.start)?;
    hits.iter()
        .find(|(range, _)| range == name)
        .or_else(|| hits.first())
        .map(|(_, id)| id)
}

fn check_cancel(cancel: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        Err("Critical projection cancelled".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
