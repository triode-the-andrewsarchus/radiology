use crate::semantic::index::SemanticIndex;
use egui::Color32;
use std::collections::HashSet;

/// Deterministic 12-color identity palette for caller capsules.
/// The same method/file pair keeps its hue across branches and application launches.
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

pub fn caller_pill_color(method: &str, file: &str) -> Color32 {
    CALLER_PALETTE[caller_pill_slot(method, file)]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceCallerNode {
    pub name: String,
    pub path: String,
    pub line: usize,
    pub depth: usize,
    pub color_slot: usize,
    pub is_recursive: bool,
    pub children: Vec<ReferenceCallerNode>,
}

pub struct CallerTreeBuilder;

impl CallerTreeBuilder {
    pub fn build(
        index: &SemanticIndex,
        symbol: &str,
        max_depth: usize,
    ) -> Vec<ReferenceCallerNode> {
        let mut active_path = HashSet::new();
        Self::trace(index, symbol, 0, max_depth, &mut active_path)
    }

    fn trace(
        index: &SemanticIndex,
        symbol: &str,
        depth: usize,
        max_depth: usize,
        active_path: &mut HashSet<String>,
    ) -> Vec<ReferenceCallerNode> {
        if depth >= max_depth {
            return Vec::new();
        }

        let refs = index.find_references(symbol);
        let mut nodes = Vec::new();

        for (ref_path, ref_range) in refs {
            // Find enclosing definition in the caller file
            let caller_sym = Self::find_enclosing_symbol(index, ref_path, ref_range.start_line);
            let caller_name = caller_sym
                .as_deref()
                .map(|s| s.rsplit('/').next().unwrap_or(s))
                .unwrap_or(ref_path);

            let slot = caller_pill_slot(caller_name, ref_path);
            let is_recursive = caller_sym
                .as_deref()
                .is_some_and(|s| active_path.contains(s));

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

    fn find_enclosing_symbol(index: &SemanticIndex, path: &str, line: i32) -> Option<String> {
        let doc = index.documents.get(path)?;
        for occ in &doc.occurrences {
            if occ.is_definition() {
                if let Some(enclosing) = &occ.enclosing_range {
                    if enclosing.start_line <= line && enclosing.end_line >= line {
                        return Some(occ.symbol.clone());
                    }
                } else if let Some(r) = &occ.range {
                    if r.start_line <= line && r.end_line >= line {
                        return Some(occ.symbol.clone());
                    }
                }
            }
        }
        None
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
