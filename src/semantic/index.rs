use std::collections::HashMap;
use std::path::Path;
use crate::semantic::scip_proto::{ProtobufError, ScipDocument, ScipIndex, ScipOccurrence, ScipRange, ScipSymbolInformation};

#[derive(Debug, Clone, Default)]
pub struct SemanticIndex {
    pub documents: HashMap<String, ScipDocument>,
    pub definitions: HashMap<String, (String, ScipRange)>,
    pub references: HashMap<String, Vec<(String, ScipRange)>>,
    pub symbol_info: HashMap<String, ScipSymbolInformation>,
}

impl SemanticIndex {
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
                    idx.definitions.insert(occ.symbol.clone(), (doc.relative_path.clone(), range.clone()));
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

    pub fn load_file(path: &Path) -> Result<Self, ProtobufError> {
        let bytes = std::fs::read(path).map_err(|_| ProtobufError::Truncated)?;
        let scip = ScipIndex::decode(&bytes)?;
        Ok(Self::from_scip(scip))
    }

    pub fn find_definition(&self, symbol: &str) -> Option<(&str, &ScipRange)> {
        self.definitions.get(symbol).map(|(p, r)| (p.as_str(), r))
    }

    pub fn find_references(&self, symbol: &str) -> Vec<(&str, &ScipRange)> {
        self.references
            .get(symbol)
            .map(|refs| refs.iter().map(|(p, r)| (p.as_str(), r)).collect())
            .unwrap_or_default()
    }

    pub fn symbol_info(&self, symbol: &str) -> Option<&ScipSymbolInformation> {
        self.symbol_info.get(symbol)
    }

    /// Finds occurrence in a document for zero-based line and character column.
    pub fn occurrence_at(&self, path: &str, line: usize, col: usize) -> Option<&ScipOccurrence> {
        let doc = self.documents.get(path)?;
        let line = line as i32;
        let col = col as i32;

        for occ in &doc.occurrences {
            if let Some(r) = &occ.range {
                if r.start_line == line && col >= r.start_col && col <= r.end_col {
                    return Some(occ);
                }
                if r.start_line <= line && r.end_line >= line {
                    if (line > r.start_line || col >= r.start_col)
                        && (line < r.end_line || col <= r.end_col)
                    {
                        return Some(occ);
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
    fn test_index_lookup() {
        let mut doc = ScipDocument::default();
        doc.relative_path = "src/main.rs".to_string();
        doc.occurrences.push(ScipOccurrence {
            range: Some(ScipRange { start_line: 5, start_col: 3, end_line: 5, end_col: 10 }),
            symbol: "scip-rust/main".to_string(),
            symbol_roles: 1, // Definition
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
