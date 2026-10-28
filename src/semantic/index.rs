//! Repository semantic symbol index, definition lookup, and reference discovery.

use crate::semantic::scip_proto::{
    ProtobufError, ScipDocument, ScipIndex, ScipOccurrence, ScipRange, ScipSymbolInformation,
};
use std::collections::HashMap;
use std::path::Path;

/// Identifies one symbol across the index.
///
/// SCIP `local` symbols are only unique within their document, so they carry
/// the document path. All other symbol identifiers are workspace-global.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SemanticSymbolID {
    /// Raw SCIP symbol string.
    pub symbol: String,
    /// Document path if this is a local symbol (`local N`).
    pub document: Option<String>,
}

impl SemanticSymbolID {
    /// Creates a new [`SemanticSymbolID`].
    ///
    /// If `symbol` begins with `"local "`, it is scoped to `document`.
    #[must_use]
    pub fn new(symbol: impl Into<String>, document: &str) -> Self {
        let s = symbol.into();
        let doc = if s.starts_with("local ") {
            Some(document.to_string())
        } else {
            None
        };
        Self {
            symbol: s,
            document: doc,
        }
    }

    /// Returns `true` if this symbol is document-local.
    #[must_use]
    pub fn is_local(&self) -> bool {
        self.document.is_some()
    }
}

/// Helper for extracting clean fallback display names from SCIP symbol descriptors.
pub struct SCIPSymbolString;

impl SCIPSymbolString {
    /// Extracts the name of the last descriptor when an indexer omits `display_name`.
    #[must_use]
    pub fn fallback_name(symbol: &str) -> String {
        if let Some(rest) = symbol.strip_prefix("local ") {
            return rest.to_string();
        }
        let descriptors = Self::descriptor_text(symbol);
        let chars: Vec<char> = descriptors.chars().collect();
        let mut names: Vec<String> = Vec::new();
        let mut current = String::new();
        let mut escaped = false;
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];
            if escaped {
                if ch == '`' && i + 1 < chars.len() && chars[i + 1] == '`' {
                    current.push('`');
                    i += 2;
                    continue;
                }
                if ch == '`' {
                    escaped = false;
                } else {
                    current.push(ch);
                }
            } else if ch == '`' {
                escaped = true;
            } else if "/#.:!".contains(ch) {
                if !current.is_empty() {
                    names.push(std::mem::take(&mut current));
                }
            } else if ch == '(' {
                if !current.is_empty() {
                    names.push(std::mem::take(&mut current));
                }
                while i < chars.len() && chars[i] != ')' {
                    i += 1;
                }
            } else if ch == '[' {
                if !current.is_empty() {
                    names.push(std::mem::take(&mut current));
                }
                let mut quoted = false;
                i += 1;
                while i < chars.len() && (quoted || chars[i] != ']') {
                    if chars[i] == '`' {
                        quoted = !quoted;
                    }
                    i += 1;
                }
            } else if ch != ')' && ch != ']' {
                current.push(ch);
            }
            i += 1;
        }

        if !current.is_empty() {
            names.push(current);
        }

        names.pop().unwrap_or_else(|| symbol.to_string())
    }

    fn descriptor_text(symbol: &str) -> &str {
        let chars: Vec<(usize, char)> = symbol.char_indices().collect();
        let mut fields = 0;
        let mut i = 0;
        while i < chars.len() && fields < 4 {
            let (_, ch) = chars[i];
            if ch == ' ' {
                if i + 1 < chars.len() && chars[i + 1].1 == ' ' {
                    i += 2;
                    continue;
                }
                fields += 1;
                if fields == 4 {
                    if i + 1 < chars.len() {
                        return &symbol[chars[i + 1].0..];
                    }
                    return "";
                }
            }
            i += 1;
        }
        symbol
    }
}

/// An indexed symbol entry ready for workspace-wide symbol search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolSearchEntry {
    /// Semantic symbol identifier.
    pub id: SemanticSymbolID,
    /// Clean display or fallback name.
    pub name: String,
    /// SCIP symbol kind integer code.
    pub kind: i32,
    /// Primary repository-relative file path where defined.
    pub path: String,
    /// 1-based line number of definition.
    pub line: usize,
    /// Total number of definitions found across the repository.
    pub definition_count: usize,
}

/// A matched symbol result ranked by search tier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolSearchMatch {
    /// Semantic symbol identifier.
    pub id: SemanticSymbolID,
    /// Clean display or fallback name.
    pub name: String,
    /// SCIP symbol kind integer code.
    pub kind: i32,
    /// Primary definition file path.
    pub path: String,
    /// 1-based definition line number.
    pub line: usize,
    /// Ranking tier (0: exact, 1: prefix, 2: initials/component, 3: subsequence fuzzy).
    pub tier: usize,
}

/// 4-tier ranking engine for workspace symbol lookup.
pub struct SymbolSearch;

impl SymbolSearch {
    /// Ranks `entries` against `query`, returning at most `limit` matches.
    #[must_use]
    pub fn rank(
        query: &str,
        entries: &[SymbolSearchEntry],
        limit: usize,
    ) -> Vec<SymbolSearchMatch> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return entries
                .iter()
                .take(limit)
                .map(|e| SymbolSearchMatch {
                    id: e.id.clone(),
                    name: e.name.clone(),
                    kind: e.kind,
                    path: e.path.clone(),
                    line: e.line,
                    tier: 4,
                })
                .collect();
        }
        let needle = trimmed.to_lowercase();
        let mut matches = Vec::new();
        for entry in entries {
            if let Some(tier) = Self::tier(&entry.name, &needle) {
                matches.push(SymbolSearchMatch {
                    id: entry.id.clone(),
                    name: entry.name.clone(),
                    kind: entry.kind,
                    path: entry.path.clone(),
                    line: entry.line,
                    tier,
                });
            }
        }

        matches.sort_by(|a, b| {
            a.tier
                .cmp(&b.tier)
                .then_with(|| a.name.len().cmp(&b.name.len()))
                .then_with(|| a.name.cmp(&b.name))
                .then_with(|| a.path.cmp(&b.path))
        });

        matches.truncate(limit);
        matches
    }

    fn tier(name: &str, needle: &str) -> Option<usize> {
        let lowered = name.to_lowercase();
        if lowered == needle {
            return Some(0);
        }
        if lowered.starts_with(needle) {
            return Some(1);
        }
        if Self::initials(name).starts_with(needle) || Self::component_prefix_match(name, needle) {
            return Some(2);
        }
        if Self::is_subsequence(needle, &lowered) {
            return Some(3);
        }
        None
    }

    /// Computes component initials from camelCase, snake_case, and kebab-case identifiers.
    #[must_use]
    pub fn initials(name: &str) -> String {
        let mut result = String::new();
        let mut prev = '_';
        for ch in name.chars() {
            if "_-./:".contains(ch) {
                prev = ch;
                continue;
            }
            if "_-./:".contains(prev)
                || (ch.is_uppercase() && !prev.is_uppercase())
                || (ch.is_numeric() && !prev.is_numeric())
            {
                for l in ch.to_lowercase() {
                    result.push(l);
                }
            }
            prev = ch;
        }
        result
    }

    fn component_prefix_match(name: &str, needle: &str) -> bool {
        let components: Vec<String> = name
            .split(|c: char| "_-./:".contains(c) || c.is_uppercase())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect();
        let humps = Self::hump_components(name);
        let pieces: Vec<&str> = needle
            .split(|c: char| "_-./:".contains(c))
            .filter(|s| !s.is_empty())
            .collect();
        if pieces.len() <= 1 {
            return false;
        }

        for candidates in [&components, &humps] {
            let mut idx = 0;
            let mut matched = 0;
            for piece in &pieces {
                while idx < candidates.len() && !candidates[idx].starts_with(piece) {
                    idx += 1;
                }
                if idx >= candidates.len() {
                    break;
                }
                matched += 1;
                idx += 1;
            }
            if matched == pieces.len() {
                return true;
            }
        }
        false
    }

    fn hump_components(name: &str) -> Vec<String> {
        let mut components = Vec::new();
        let mut current = String::new();
        let mut prev = 'a';
        for ch in name.chars() {
            if "_-./:".contains(ch) {
                if !current.is_empty() {
                    components.push(std::mem::take(&mut current).to_lowercase());
                }
            } else if ch.is_uppercase() && !prev.is_uppercase() && !current.is_empty() {
                components.push(std::mem::take(&mut current).to_lowercase());
                current.push(ch);
            } else {
                current.push(ch);
            }
            prev = ch;
        }
        if !current.is_empty() {
            components.push(current.to_lowercase());
        }
        components
    }

    fn is_subsequence(needle: &str, haystack: &str) -> bool {
        let mut hay_chars = haystack.chars();
        for n in needle.chars() {
            if !hay_chars.any(|h| h == n) {
                return false;
            }
        }
        true
    }
}

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
    /// Default column encoding for documents without an explicit encoding.
    pub default_encoding: i32,
    /// Documents keyed by repository-relative path.
    pub documents: HashMap<String, ScipDocument>,
    /// Definition location table: `symbol_id -> [(path, range)]`.
    pub definitions: HashMap<SemanticSymbolID, Vec<(String, ScipRange)>>,
    /// Reference locations table: `symbol_id -> [(path, range)]`.
    pub references: HashMap<SemanticSymbolID, Vec<(String, ScipRange)>>,
    /// First definition enclosing range bounding body: `symbol_id -> (path, range)`.
    pub definition_extents: HashMap<SemanticSymbolID, (String, ScipRange)>,
    /// Symbol documentation and metadata table: `symbol_id -> info`.
    pub symbols: HashMap<SemanticSymbolID, ScipSymbolInformation>,
    /// Pre-indexed entries for workspace-wide symbol search.
    pub search_entries: Vec<SymbolSearchEntry>,
}

impl SemanticIndex {
    /// Builds an indexed [`SemanticIndex`] from a decoded [`ScipIndex`].
    #[must_use]
    pub fn from_scip(scip: ScipIndex) -> Self {
        let mut idx = Self {
            default_encoding: scip.text_document_encoding,
            ..Default::default()
        };

        for sym in scip.external_symbols {
            let id = SemanticSymbolID::new(sym.symbol.clone(), "");
            if !id.is_local() {
                idx.symbols.entry(id).or_insert(sym);
            }
        }

        for doc in scip.documents {
            let doc_path = doc.relative_path.clone();

            for sym in &doc.symbols {
                let id = SemanticSymbolID::new(sym.symbol.clone(), &doc_path);
                if let Some(existing) = idx.symbols.get_mut(&id) {
                    if existing.display_name.is_empty() && !sym.display_name.is_empty() {
                        *existing = sym.clone();
                    }
                } else {
                    idx.symbols.insert(id, sym.clone());
                }
            }

            for occ in &doc.occurrences {
                if occ.symbol.is_empty() {
                    continue;
                }
                let Some(range) = &occ.range else {
                    continue;
                };

                let id = SemanticSymbolID::new(occ.symbol.clone(), &doc_path);
                if occ.is_definition() {
                    idx.definitions
                        .entry(id.clone())
                        .or_default()
                        .push((doc_path.clone(), range.clone()));

                    if let Some(enclosing) = &occ.enclosing_range {
                        idx.definition_extents
                            .entry(id.clone())
                            .or_insert_with(|| (doc_path.clone(), enclosing.clone()));
                    }
                } else {
                    idx.references
                        .entry(id.clone())
                        .or_default()
                        .push((doc_path.clone(), range.clone()));
                }
            }

            idx.documents.insert(doc_path, doc);
        }

        // Symbols appearing only in occurrences still get fallback entries:
        let all_ids: Vec<SemanticSymbolID> = idx
            .definitions
            .keys()
            .chain(idx.references.keys())
            .cloned()
            .collect();
        for id in all_ids {
            idx.symbols
                .entry(id.clone())
                .or_insert_with(|| ScipSymbolInformation {
                    symbol: id.symbol.clone(),
                    display_name: SCIPSymbolString::fallback_name(&id.symbol),
                    ..Default::default()
                });
        }

        // Build search entries from non-local definitions:
        for (id, defs) in &idx.definitions {
            if id.is_local() {
                continue;
            }
            if let Some((first_path, first_range)) = defs
                .iter()
                .min_by(|a, b| (a.0.as_str(), a.1.start_line).cmp(&(b.0.as_str(), b.1.start_line)))
            {
                let sym_info = idx.symbols.get(id);
                let name = sym_info
                    .map(|s| {
                        if s.display_name.is_empty() {
                            SCIPSymbolString::fallback_name(&id.symbol)
                        } else {
                            s.display_name.clone()
                        }
                    })
                    .unwrap_or_else(|| SCIPSymbolString::fallback_name(&id.symbol));

                if !name.is_empty() {
                    idx.search_entries.push(SymbolSearchEntry {
                        id: id.clone(),
                        name,
                        kind: sym_info.map_or(0, |s| s.kind),
                        path: first_path.clone(),
                        line: (first_range.start_line + 1) as usize,
                        definition_count: defs.len(),
                    });
                }
            }
        }

        idx.search_entries.sort_by(|a, b| {
            (a.name.as_str(), a.path.as_str(), a.line).cmp(&(
                b.name.as_str(),
                b.path.as_str(),
                b.line,
            ))
        });

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
        let id = SemanticSymbolID {
            symbol: symbol.to_string(),
            document: None,
        };
        self.find_definition_for_id(&id)
    }

    /// Resolves the canonical definition file path and coordinate range for `id`.
    #[must_use]
    pub fn find_definition_for_id(&self, id: &SemanticSymbolID) -> Option<(&str, &ScipRange)> {
        if let Some(defs) = self.definitions.get(id) {
            if let Some((p, r)) = defs.first() {
                return Some((p.as_str(), r));
            }
        }
        // Relationship fallback (Relationship.is_definition)
        if let Some(info) = self.symbols.get(id) {
            for rel in &info.relationships {
                if rel.is_definition {
                    let rel_id = SemanticSymbolID::new(
                        rel.symbol.clone(),
                        id.document.as_deref().unwrap_or(""),
                    );
                    if let Some(defs) = self.definitions.get(&rel_id) {
                        if let Some((p, r)) = defs.first() {
                            return Some((p.as_str(), r));
                        }
                    }
                }
            }
        }
        None
    }

    /// Finds all reference file paths and coordinate ranges targeting `symbol`.
    #[must_use]
    pub fn find_references(&self, symbol: &str) -> Vec<(&str, &ScipRange)> {
        let id = SemanticSymbolID {
            symbol: symbol.to_string(),
            document: None,
        };
        self.find_references_for_id(&id)
    }

    /// Finds all reference file paths and coordinate ranges targeting `id`.
    #[must_use]
    pub fn find_references_for_id(&self, id: &SemanticSymbolID) -> Vec<(&str, &ScipRange)> {
        self.references
            .get(id)
            .map(|refs| refs.iter().map(|(p, r)| (p.as_str(), r)).collect())
            .unwrap_or_default()
    }

    /// Returns documentation and metadata for `symbol` if indexed.
    #[must_use]
    pub fn symbol_info(&self, symbol: &str) -> Option<&ScipSymbolInformation> {
        let id = SemanticSymbolID {
            symbol: symbol.to_string(),
            document: None,
        };
        self.symbols.get(&id)
    }

    /// Returns documentation and metadata for `id` if indexed.
    #[must_use]
    pub fn symbol_info_for_id(&self, id: &SemanticSymbolID) -> Option<&ScipSymbolInformation> {
        self.symbols.get(id)
    }

    /// Returns the display name for `id` or computes a clean fallback name.
    #[must_use]
    pub fn display_name(&self, id: &SemanticSymbolID) -> String {
        if let Some(info) = self.symbols.get(id) {
            if !info.display_name.is_empty() {
                return info.display_name.clone();
            }
        }
        SCIPSymbolString::fallback_name(&id.symbol)
    }

    /// Queries pre-indexed workspace symbols ranked by 4-tier match quality.
    #[must_use]
    pub fn search_symbols(&self, query: &str, limit: usize) -> Vec<SymbolSearchMatch> {
        SymbolSearch::rank(query, &self.search_entries, limit)
    }

    /// Finds a symbol occurrence in `path` covering the zero-based `(line, col)` position.
    #[must_use]
    pub fn occurrence_at(&self, path: &str, line: usize, col: usize) -> Option<&ScipOccurrence> {
        let doc = self.documents.get(path)?;
        let point = (line as i32, col as i32);
        doc.occurrences
            .iter()
            .filter(|occ| {
                occ.range.as_ref().is_some_and(|range| {
                    (range.start_line, range.start_col) <= point
                        && point < (range.end_line, range.end_col)
                })
            })
            .min_by_key(|occ| {
                let range = occ.range.as_ref().expect("range checked");
                (
                    range.end_line - range.start_line,
                    range.end_col - range.start_col,
                )
            })
    }

    /// Converts an indexed range to exact canonical bytes using its document encoding.
    #[must_use]
    pub fn byte_range(
        &self,
        buffer: &crate::source::SourceBuffer,
        range: &ScipRange,
    ) -> Option<std::ops::Range<usize>> {
        let encoding = match self.documents.get(&buffer.path)?.position_encoding {
            0 => self.default_encoding,
            explicit => explicit,
        };
        let encoding = match encoding {
            2 => crate::source::ColumnEncoding::Utf16,
            3 => crate::source::ColumnEncoding::Utf32,
            _ => crate::source::ColumnEncoding::Utf8,
        };
        let position = |line: i32, column: i32| {
            buffer.byte_offset_encoded(
                usize::try_from(line).ok()?.checked_add(1)?,
                usize::try_from(column).ok()?,
                encoding,
            )
        };
        let start = position(range.start_line, range.start_col)?;
        let end = position(range.end_line, range.end_col)?;
        (start <= end).then_some(start..end)
    }

    /// Finds the narrowest indexed occurrence at a canonical source byte.
    #[must_use]
    pub fn occurrence_at_byte(
        &self,
        buffer: &crate::source::SourceBuffer,
        byte: usize,
    ) -> Option<&ScipOccurrence> {
        self.documents
            .get(&buffer.path)?
            .occurrences
            .iter()
            .filter_map(|occurrence| {
                let range = self.byte_range(buffer, occurrence.range.as_ref()?)?;
                range.contains(&byte).then_some((range.len(), occurrence))
            })
            .min_by_key(|(size, _)| *size)
            .map(|(_, occurrence)| occurrence)
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

    #[test]
    fn test_local_symbol_separation() {
        let mut doc1 = ScipDocument {
            relative_path: "src/a.rs".to_string(),
            ..Default::default()
        };
        doc1.occurrences.push(ScipOccurrence {
            range: Some(ScipRange {
                start_line: 1,
                start_col: 0,
                end_line: 1,
                end_col: 5,
            }),
            symbol: "local 1".to_string(),
            symbol_roles: 1,
            enclosing_range: None,
        });

        let mut doc2 = ScipDocument {
            relative_path: "src/b.rs".to_string(),
            ..Default::default()
        };
        doc2.occurrences.push(ScipOccurrence {
            range: Some(ScipRange {
                start_line: 2,
                start_col: 0,
                end_line: 2,
                end_col: 5,
            }),
            symbol: "local 1".to_string(),
            symbol_roles: 1,
            enclosing_range: None,
        });

        let mut scip = ScipIndex::default();
        scip.documents.push(doc1);
        scip.documents.push(doc2);

        let index = SemanticIndex::from_scip(scip);
        let id1 = SemanticSymbolID::new("local 1", "src/a.rs");
        let id2 = SemanticSymbolID::new("local 1", "src/b.rs");

        assert_eq!(index.definitions.get(&id1).unwrap()[0].0, "src/a.rs");
        assert_eq!(index.definitions.get(&id2).unwrap()[0].0, "src/b.rs");
    }

    #[test]
    fn test_fallback_name_parsing() {
        assert_eq!(SCIPSymbolString::fallback_name("local 42"), "42");
        assert_eq!(
            SCIPSymbolString::fallback_name("rust-analyzer cargo pkg 0.1.0 src/lib.rs/MyType#"),
            "MyType"
        );
        assert_eq!(
            SCIPSymbolString::fallback_name(
                "rust-analyzer cargo pkg 0.1.0 src/lib.rs/MyType#my_func()."
            ),
            "my_func"
        );
    }

    #[test]
    fn test_symbol_search_ranking() {
        let entry1 = SymbolSearchEntry {
            id: SemanticSymbolID::new("foo", ""),
            name: "calculate_total".to_string(),
            kind: 17,
            path: "src/billing.rs".to_string(),
            line: 10,
            definition_count: 1,
        };
        let entry2 = SymbolSearchEntry {
            id: SemanticSymbolID::new("bar", ""),
            name: "calc".to_string(),
            kind: 17,
            path: "src/calc.rs".to_string(),
            line: 5,
            definition_count: 1,
        };
        let entry3 = SymbolSearchEntry {
            id: SemanticSymbolID::new("baz", ""),
            name: "CustomerAccountLifecycle".to_string(),
            kind: 7,
            path: "src/models.rs".to_string(),
            line: 20,
            definition_count: 1,
        };

        let entries = vec![entry1, entry2, entry3];

        // Exact match
        let matches = SymbolSearch::rank("calc", &entries, 10);
        assert_eq!(matches[0].name, "calc");
        assert_eq!(matches[0].tier, 0);

        // Prefix match
        let matches = SymbolSearch::rank("calculate", &entries, 10);
        assert_eq!(matches[0].name, "calculate_total");
        assert_eq!(matches[0].tier, 1);

        // CamelCase initials match
        let matches = SymbolSearch::rank("cal", &entries, 10);
        assert!(matches
            .iter()
            .any(|m| m.name == "CustomerAccountLifecycle" && m.tier == 2));
    }
}
