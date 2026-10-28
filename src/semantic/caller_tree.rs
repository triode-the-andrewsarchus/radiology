//! Multi-level caller tree tracing with cycle detection and deterministic identity pills.

use crate::semantic::index::{SemanticIndex, SemanticSymbolID};
use egui::Color32;
use std::collections::HashSet;

/// Deterministic 12-color identity palette for caller capsules.
///
/// The same method and file pair retains its hue across branches and application launches.
pub const CALLER_PALETTE: [Color32; 12] = [
    Color32::from_rgb(59, 130, 246), // Blue
    Color32::from_rgb(20, 184, 166), // Teal
    Color32::from_rgb(168, 85, 247), // Purple
    Color32::from_rgb(249, 115, 22), // Orange
    Color32::from_rgb(236, 72, 153), // Pink
    Color32::from_rgb(99, 102, 241), // Indigo
    Color32::from_rgb(34, 197, 94),  // Green
    Color32::from_rgb(6, 182, 212),  // Cyan
    Color32::from_rgb(162, 114, 80), // Brown
    Color32::from_rgb(52, 211, 153), // Mint
    Color32::from_rgb(239, 68, 68),  // Red
    Color32::from_rgb(234, 179, 8),  // Yellow
];

/// Computes the deterministic 0..12 palette slot using FNV-1a hash over
/// method name, null separator, and repository-relative file path.
///
/// # Examples
///
/// ```rust
/// use radiology::semantic::caller_tree::caller_pill_slot;
///
/// let slot = caller_pill_slot("handle_request", "src/server.rs");
/// assert!(slot < 12);
/// ```
#[must_use]
pub fn caller_pill_slot(method: &str, file: &str) -> usize {
    let mut hash: u64 = 14_695_981_039_346_656_037;
    for byte in method.bytes() {
        hash = (hash ^ (byte as u64)).wrapping_mul(1_099_511_628_211);
    }
    hash = hash.wrapping_mul(1_099_511_628_211);
    for byte in file.bytes() {
        hash = (hash ^ (byte as u64)).wrapping_mul(1_099_511_628_211);
    }
    (hash % (CALLER_PALETTE.len() as u64)) as usize
}

/// Returns the deterministic [`Color32`] assigned to the method and file pair.
#[must_use]
pub fn caller_pill_color(method: &str, file: &str) -> Color32 {
    CALLER_PALETTE[caller_pill_slot(method, file)]
}

/// A node in the reverse call graph representing a caller site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceCallerNode {
    /// Calling function or method name.
    pub name: String,
    /// Repository-relative path containing the call.
    pub path: String,
    /// 1-based line number of the call site.
    pub line: usize,
    /// Depth within the caller tree (0 is direct caller).
    pub depth: usize,
    /// Deterministic color slot index (0..12).
    pub color_slot: usize,
    /// `true` if this caller creates a recursive loop back to an ancestor.
    pub is_recursive: bool,
    /// Callers of this caller.
    pub children: Vec<ReferenceCallerNode>,
}

/// Traces incoming callers backward through SCIP references.
pub struct CallerTreeBuilder;

impl CallerTreeBuilder {
    /// Recursively traces callers of `symbol` up to `max_depth`, preventing recursion cycles.
    #[must_use]
    pub fn build(
        index: &SemanticIndex,
        symbol: &str,
        max_depth: usize,
    ) -> Vec<ReferenceCallerNode> {
        Self::build_for_id(index, &SemanticSymbolID::new(symbol, ""), max_depth)
    }

    /// Traces a scoped symbol while retaining document-local identities.
    #[must_use]
    pub fn build_for_id(
        index: &SemanticIndex,
        symbol: &SemanticSymbolID,
        max_depth: usize,
    ) -> Vec<ReferenceCallerNode> {
        let mut active_path = HashSet::from([symbol.clone()]);
        Self::trace(index, symbol, 0, max_depth, &mut active_path)
    }

    fn trace(
        index: &SemanticIndex,
        symbol: &SemanticSymbolID,
        depth: usize,
        max_depth: usize,
        active_path: &mut HashSet<SemanticSymbolID>,
    ) -> Vec<ReferenceCallerNode> {
        if depth >= max_depth {
            return Vec::new();
        }

        let refs = index.find_references_for_id(symbol);
        let mut nodes = Vec::new();

        for (ref_path, ref_range) in refs {
            let caller_sym = Self::find_enclosing_symbol(index, ref_path, ref_range.start_line);
            let caller_name = caller_sym
                .as_ref()
                .map(|id| index.display_name(id))
                .unwrap_or_else(|| ref_path.to_string());

            let slot = caller_pill_slot(&caller_name, ref_path);
            let is_recursive = caller_sym.as_ref().is_some_and(|s| active_path.contains(s));

            let mut node = ReferenceCallerNode {
                name: caller_name.to_string(),
                path: ref_path.to_string(),
                line: (ref_range.start_line + 1) as usize,
                depth,
                color_slot: slot,
                is_recursive,
                children: Vec::new(),
            };

            if !is_recursive {
                if let Some(cs) = &caller_sym {
                    active_path.insert(cs.clone());
                    node.children = Self::trace(index, cs, depth + 1, max_depth, active_path);
                    active_path.remove(cs);
                }
            }

            nodes.push(node);
        }

        nodes
    }

    fn find_enclosing_symbol(
        index: &SemanticIndex,
        path: &str,
        line: i32,
    ) -> Option<SemanticSymbolID> {
        let document = index.documents.get(path)?;
        document
            .occurrences
            .iter()
            .filter(|occ| occ.is_definition())
            .filter_map(|occ| {
                let range = occ.enclosing_range.as_ref().or(occ.range.as_ref())?;
                (range.start_line <= line && line <= range.end_line).then_some((range, occ))
            })
            .min_by_key(|(range, _)| {
                (
                    range.end_line - range.start_line,
                    range.end_col - range.start_col,
                )
            })
            .map(|(_, occ)| SemanticSymbolID::new(&occ.symbol, path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caller_palette_determinism() {
        let slot1 = caller_pill_slot("handleRequest", "src/server.rs");
        let slot2 = caller_pill_slot("handleRequest", "src/server.rs");
        assert_eq!(slot1, slot2);

        let color = caller_pill_color("handleRequest", "src/server.rs");
        assert_eq!(color, CALLER_PALETTE[slot1]);
    }
}
