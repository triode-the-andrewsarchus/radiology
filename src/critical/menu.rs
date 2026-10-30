//! Exact semantic targets and built-in decisions for source marking controls.

use super::{
    Criticality, CriticalityClassification, CriticalityOverrides, CriticalityPolicy,
    SCIPSymbolHeader,
};
use crate::semantic::index::SemanticSymbolID;
use crate::semantic::SemanticIndex;
use crate::source::{ReviewRevision, SourceBuffer};

/// Captured index/policy snapshot, independent of source caret changes.
#[derive(Clone, Copy)]
pub struct MarkContext<'a> {
    /// Immutable current index.
    pub index: &'a SemanticIndex,
    /// Current repository's explicit decisions.
    pub overrides: &'a CriticalityOverrides,
}

/// Owned target suitable for a source-menu action; no source spelling inference.
#[derive(Debug, Clone)]
pub struct MarkMenu {
    /// Human-readable source spelling.
    pub name: String,
    /// Full persistent SCIP identity.
    pub symbol: String,
    /// Package display name, when present.
    pub package_name: Option<String>,
    /// Version-independent package identity.
    pub package_key: Option<String>,
    /// Classification without manual decisions.
    pub built_in: CriticalityClassification,
    /// Explicit symbol decision, if any.
    pub symbol_decision: Option<Criticality>,
    /// Explicit package decision, if any.
    pub package_decision: Option<Criticality>,
}

impl MarkContext<'_> {
    /// Resolves the clicked byte, excluding locals, Before and mismatched sources.
    #[must_use]
    pub fn at_byte(&self, buffer: &SourceBuffer, byte: usize) -> Option<MarkMenu> {
        if buffer.revision == Some(ReviewRevision::Before) {
            return None;
        }
        let occurrence = self.index.occurrence_at_byte(buffer, byte)?;
        if !CriticalityOverrides::is_overridable(&occurrence.symbol) {
            return None;
        }
        let id = SemanticSymbolID::new(occurrence.symbol.clone(), &buffer.path);
        let header = SCIPSymbolHeader::parse(&id.symbol);
        let package_key = header.as_ref().and_then(|header| header.package_key());
        let package_name = header
            .as_ref()
            .and_then(|header| header.package_name().map(str::to_owned));
        let info = self.index.symbol_info_for_id(&id);
        let built_in = CriticalityPolicy::for_index(Some(self.index), Default::default()).classify(
            &id.symbol,
            info.map(|info| info.signature.as_str()),
            self.index.definitions.contains_key(&id),
        );
        let name = occurrence
            .range
            .as_ref()
            .and_then(|range| self.index.byte_range(buffer, range))
            .and_then(|range| buffer.text.get(range))
            .unwrap_or(&id.symbol)
            .to_owned();
        Some(MarkMenu {
            name,
            symbol: id.symbol.clone(),
            package_name,
            symbol_decision: self.overrides.symbols.get(&id.symbol).copied(),
            package_decision: package_key
                .as_ref()
                .and_then(|key| self.overrides.packages.get(key).copied()),
            package_key,
            built_in,
        })
    }
}

#[cfg(test)]
mod tests;
