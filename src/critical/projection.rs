//! Call-focused Critical Lens projection from exact syntax and semantic identity.

use super::lens::{Criticality, CriticalityOverrides, CriticalityPolicy, SourceProjection};
use crate::semantic::SemanticIndex;
use crate::source::SourceBuffer;
use crate::syntax::calls::CallSites;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

impl CriticalityPolicy {
    /// Builds package ownership from indexed definitions, without spelling heuristics.
    #[must_use]
    pub fn for_index(index: Option<&SemanticIndex>, overrides: CriticalityOverrides) -> Self {
        let repository_packages = index
            .map(|index| index.repository_packages.clone())
            .unwrap_or_default();
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
    let resolver = crate::semantic::callees::CalleeResolver::new(buffer, index);
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
        let id = resolver.resolve(name);
        let decision = id.zip(resolver.index).map(|(id, index)| {
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

fn check_cancel(cancel: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        Err("Critical projection cancelled".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
