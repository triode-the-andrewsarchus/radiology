//! Repository semantic symbol index, definition lookup, and reference discovery.

use crate::semantic::scip_proto::{
    ProtobufError, ScipDocument, ScipIndex, ScipOccurrence, ScipRange, ScipSymbolInformation,
};
use std::collections::HashMap;
use std::path::Path;

/// In-memory indexed database of SCIP symbols, definitions, and references.
///
/// # Examples
///
/// ```rust
/// use radiology::semantic::index::SemanticIndex;
/// use radiology::semantic::scip_proto::{ScipDocument, ScipIndex, ScipOccurrence, ScipRange};
///
/// let mut doc = ScipDocument {
///     relative_path: "src/main.rs".to_string(),
///     ..Default::default()
/// };
/// doc.occurrences.push(ScipOccurrence {
///     range: Some(ScipRange { start_line: 0, start_col: 3, end_line: 0, end_col: 7 }),
///     symbol: "my_sym".to_string(),
///     symbol_roles: 1, // Definition
///     enclosing_range: None,
/// });
///
/// let mut scip = ScipIndex::default();
/// scip.documents.push(doc);
/// let index = SemanticIndex::from_scip(scip);
///
/// assert!(index.find_definition("my_sym").is_some());
/// ```
#[derive(Debug, Clone, Default)]
pub struct SemanticIndex {
    /// Documents keyed by repository-relative path.
    pub documents: HashMap<String, ScipDocument>,
    /// Definition location table: `symbol -> (path, range)`.
    pub definitions: HashMap<String, (String, ScipRange)>,
    /// Reference locations table: `symbol -> [(path, range)]`.
    pub references: HashMap<String, Vec<(String, ScipRange)>>,
    /// Symbol documentation and metadata table: `symbol -> info`.
    pub symbol_info: HashMap<String, ScipSymbolInformation>,
}

impl SemanticIndex {
    /// Builds an indexed [`SemanticIndex`] from a decoded [`ScipIndex`].
    #[must_use]
    pub fn from_scip(scip: ScipIndex) -> Self {
        let mut idx = Self::default();

        for sym in scip.external_symbols {
            idx.symbol_info.insert(sym.symbol.clone(), sym);
        }

        for doc in scip.documents {
            for sym in &doc.symbols {
                idx.symbol_info.insert(sym.symbol.clone(), sym.clone());
            }

            for occ in &doc.occurrences {
                if occ.symbol.is_empty() {
                    continue;
                }
                let Some(range) = &occ.range else {
                    continue;
                };

                if occ.is_definition() {
                    idx.definitions.insert(
                        occ.symbol.clone(),
                        (doc.relative_path.clone(), range.clone()),
                    );
                } else {
                    idx.references
                        .entry(occ.symbol.clone())
                        .or_default()
                        .push((doc.relative_path.clone(), range.clone()));
                }
            }

            idx.documents.insert(doc.relative_path.clone(), doc);
        }

        idx
    }

    /// Reads and decodes a `.scip` file from disk into a [`SemanticIndex`].
    ///
    /// # Errors
    ///
    /// Returns [`ProtobufError`] if reading or protobuf decoding fails.
    pub fn load_file(path: &Path) -> Result<Self, ProtobufError> {
        let bytes = std::fs::read(path).map_err(|_| ProtobufError::Truncated)?;
        let scip = ScipIndex::decode(&bytes)?;
        Ok(Self::from_scip(scip))
    }

    /// Resolves the canonical definition file path and coordinate range for `symbol`.
    #[must_use]
    pub fn find_definition(&self, symbol: &str) -> Option<(&str, &ScipRange)> {
        self.definitions.get(symbol).map(|(p, r)| (p.as_str(), r))
    }

    /// Finds all reference file paths and coordinate ranges targeting `symbol`.
    #[must_use]
    pub fn find_references(&self, symbol: &str) -> Vec<(&str, &ScipRange)> {
        self.references
            .get(symbol)
            .map(|refs| refs.iter().map(|(p, r)| (p.as_str(), r)).collect())
            .unwrap_or_default()
    }

    /// Returns documentation and metadata for `symbol` if indexed.
    #[must_use]
    pub fn symbol_info(&self, symbol: &str) -> Option<&ScipSymbolInformation> {
        self.symbol_info.get(symbol)
    }

    /// Finds a symbol occurrence in `path` covering the zero-based `(line, col)` position.
    #[must_use]
    pub fn occurrence_at(&self, path: &str, line: usize, col: usize) -> Option<&ScipOccurrence> {
        let doc = self.documents.get(path)?;
        let line = line as i32;
        let col = col as i32;

        for occ in &doc.occurrences {
            if let Some(r) = &occ.range {
                if r.start_line == line && col >= r.start_col && col <= r.end_col {
                    return Some(occ);
                }
                if r.start_line <= line
                    && r.end_line >= line
                    && (line > r.start_line || col >= r.start_col)
                    && (line < r.end_line || col <= r.end_col)
                {
                    return Some(occ);
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
    fn test_index_lookup() {
        let mut doc = ScipDocument {
            relative_path: "src/main.rs".to_string(),
            ..Default::default()
        };
        doc.occurrences.push(ScipOccurrence {
            range: Some(ScipRange {
                start_line: 5,
                start_col: 3,
                end_line: 5,
                end_col: 10,
            }),
            symbol: "scip-rust/main".to_string(),
            symbol_roles: 1,
            enclosing_range: None,
        });

        let mut scip = ScipIndex::default();
        scip.documents.push(doc);

        let index = SemanticIndex::from_scip(scip);
        let def = index.find_definition("scip-rust/main");
        assert!(def.is_some());
        let (path, range) = def.unwrap();
        assert_eq!(path, "src/main.rs");
        assert_eq!(range.start_line, 5);

        let occ = index.occurrence_at("src/main.rs", 5, 4);
        assert!(occ.is_some());
        assert_eq!(occ.unwrap().symbol, "scip-rust/main");
    }
}
