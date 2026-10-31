//! Repository semantic symbol index, definition lookup, and reference discovery.

use crate::semantic::scip_proto::{
    ProtobufError, ScipDocument, ScipIndex, ScipOccurrence, ScipRange, ScipSymbolInformation,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// The four header fields of a SCIP symbol string (`scheme manager package version`).
///
/// Double spaces inside a field are escapes and never split.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SCIPSymbolHeader {
    /// SCIP indexer scheme (e.g. `rust-analyzer`).
    pub scheme: String,
    /// Package manager name (e.g. `cargo`).
    pub manager: String,
    /// Package or crate name (e.g. `libc` or `.` for unowned).
    pub package: String,
    /// Package version string.
    pub version: String,
}

impl SCIPSymbolHeader {
    /// Parses a SCIP symbol string into header components.
    ///
    /// Returns `None` for `local N` symbols or malformed strings.
    #[must_use]
    pub fn parse(symbol: &str) -> Option<Self> {
        if symbol.starts_with("local ") {
            return None;
        }
        let mut fields = Vec::with_capacity(4);
        let mut current = String::new();
        let mut chars = symbol.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == ' ' {
                if chars.peek() == Some(&' ') {
                    chars.next();
                    current.push(' ');
                    continue;
                }
                fields.push(current);
                current = String::new();
                if fields.len() == 3 {
                    break;
                }
            } else {
                current.push(ch);
            }
        }

        // The remainder is the version plus descriptor, but the 4th field is up to the next unescaped space.
        let mut version = String::new();
        while let Some(ch) = chars.next() {
            if ch == ' ' {
                if chars.peek() == Some(&' ') {
                    chars.next();
                    version.push(' ');
                    continue;
                }
                break;
            }
            version.push(ch);
        }
        fields.push(version);

        if fields.len() != 4 {
            return None;
        }

        Some(Self {
            scheme: fields.remove(0),
            manager: fields.remove(0),
            package: fields.remove(0),
            version: fields.remove(0),
        })
    }

    /// The package name alone: `.` is SCIP's placeholder for "no package".
    #[must_use]
    pub fn package_name(&self) -> Option<&str> {
        if self.package.is_empty() || self.package == "." {
            None
        } else {
            Some(&self.package)
        }
    }

    /// `scheme manager name`, without the version: the key for package-level decisions.
    #[must_use]
    pub fn package_key(&self) -> Option<String> {
        self.package_name()
            .map(|name| format!("{} {} {}", self.scheme, self.manager, name))
    }
}

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

    /// Returns whether SCIP's descriptor or indexed kind identifies a callable.
    #[must_use]
    pub fn is_callable(&self, index: &SemanticIndex) -> bool {
        self.symbol.ends_with("().")
            || index.symbol_info_for_id(self).is_some_and(|info| {
                super::symbol_kind::SymbolCategory::from_kind(info.kind)
                    == super::symbol_kind::SymbolCategory::Function
            })
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

    /// Returns descriptor text after the four escaped SCIP header fields.
    #[must_use]
    pub fn descriptor_text(symbol: &str) -> &str {
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
    /// Package ownership derived once during background index construction.
    pub repository_packages: HashSet<String>,
    /// Default column encoding for documents without an explicit encoding.
    pub default_encoding: i32,
    /// Exact source SHA-256 identities for generated documents; absent for unverified imports.
    pub source_digests: HashMap<String, String>,
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
        Self::from_scip_cancellable(scip, || false)
            .expect("uncancelled semantic index construction")
    }

    pub(crate) fn from_scip_cancellable(
        scip: ScipIndex,
        cancelled: impl Fn() -> bool + Sync,
    ) -> Result<Self, ProtobufError> {
        super::scip_proto::check_cancelled(&cancelled)?;
        let mut idx = Self {
            default_encoding: scip.text_document_encoding,
            ..Default::default()
        };

        for sym in scip.external_symbols {
            super::scip_proto::check_cancelled(&cancelled)?;
            let id = SemanticSymbolID::new(sym.symbol.clone(), "");
            if !id.is_local() {
                idx.symbols.entry(id).or_insert(sym);
            }
        }

        for doc in scip.documents {
            super::scip_proto::check_cancelled(&cancelled)?;
            idx.add_document(doc, &cancelled)?;
        }

        // Symbols appearing only in occurrences still get fallback entries:
        for id in idx.definitions.keys().chain(idx.references.keys()) {
            super::scip_proto::check_cancelled(&cancelled)?;
            idx.symbols
                .entry(id.clone())
                .or_insert_with(|| ScipSymbolInformation {
                    symbol: id.symbol.clone(),
                    display_name: SCIPSymbolString::fallback_name(&id.symbol),
                    ..Default::default()
                });
        }

        idx.build_search_entries(&cancelled)?;
        super::scip_proto::check_cancelled(&cancelled)?;
        Ok(idx)
    }

    fn add_document(
        &mut self,
        doc: ScipDocument,
        cancelled: &(dyn Fn() -> bool + Sync),
    ) -> Result<(), ProtobufError> {
        let doc_path = doc.relative_path.clone();

        for sym in &doc.symbols {
            super::scip_proto::check_cancelled(cancelled)?;
            let id = SemanticSymbolID::new(sym.symbol.clone(), &doc_path);
            if let Some(existing) = self.symbols.get_mut(&id) {
                if existing.display_name.is_empty() && !sym.display_name.is_empty() {
                    *existing = sym.clone();
                }
            } else {
                self.symbols.insert(id, sym.clone());
            }
        }

        for occ in &doc.occurrences {
            super::scip_proto::check_cancelled(cancelled)?;
            if occ.symbol.is_empty() {
                continue;
            }
            let Some(range) = &occ.range else {
                continue;
            };

            let id = SemanticSymbolID::new(occ.symbol.clone(), &doc_path);
            if occ.is_definition() {
                self.definitions
                    .entry(id.clone())
                    .or_default()
                    .push((doc_path.clone(), range.clone()));

                if let Some(enclosing) = &occ.enclosing_range {
                    self.definition_extents
                        .entry(id.clone())
                        .or_insert_with(|| (doc_path.clone(), enclosing.clone()));
                }
            } else {
                self.references
                    .entry(id.clone())
                    .or_default()
                    .push((doc_path.clone(), range.clone()));
            }
        }

        self.documents.insert(doc_path, doc);
        Ok(())
    }

    fn build_search_entries(
        &mut self,
        cancelled: &(dyn Fn() -> bool + Sync),
    ) -> Result<(), ProtobufError> {
        // Build search entries from non-local definitions:
        for (id, defs) in &self.definitions {
            super::scip_proto::check_cancelled(cancelled)?;
            if id.is_local() {
                continue;
            }
            self.repository_packages.extend(
                SCIPSymbolHeader::parse(&id.symbol)
                    .and_then(|header| header.package_name().map(str::to_owned)),
            );
            if let Some((first_path, first_range)) = canonical_definition(defs) {
                let sym_info = self.symbols.get(id);
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
                    self.search_entries.push(SymbolSearchEntry {
                        id: id.clone(),
                        name,
                        kind: sym_info.map_or(0, |s| s.kind),
                        path: first_path.to_owned(),
                        line: first_range.start_line as usize + 1,
                        definition_count: defs.len(),
                    });
                }
            }
        }

        self.search_entries.sort_by(|a, b| {
            (a.name.as_str(), a.path.as_str(), a.line).cmp(&(
                b.name.as_str(),
                b.path.as_str(),
                b.line,
            ))
        });

        super::scip_proto::check_cancelled(cancelled)?;
        Ok(())
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

    /// Returns all direct definitions, falling back to indexed definition relationships.
    pub(crate) fn definition_locations_for_id(
        &self,
        id: &SemanticSymbolID,
    ) -> Vec<(&str, &ScipRange)> {
        if let Some(definitions) = self
            .definitions
            .get(id)
            .filter(|definitions| !definitions.is_empty())
        {
            return definitions
                .iter()
                .map(|(path, range)| (path.as_str(), range))
                .collect();
        }
        self.symbols
            .get(id)
            .into_iter()
            .flat_map(|symbol| &symbol.relationships)
            .filter(|relationship| relationship.is_definition)
            .flat_map(|relationship| {
                let related = SemanticSymbolID::new(
                    &relationship.symbol,
                    id.document.as_deref().unwrap_or(""),
                );
                self.definitions.get(&related).into_iter().flatten()
            })
            .map(|(path, range)| (path.as_str(), range))
            .collect()
    }

    /// Resolves the canonical definition file path and coordinate range for `id`.
    #[must_use]
    pub fn find_definition_for_id(&self, id: &SemanticSymbolID) -> Option<(&str, &ScipRange)> {
        if let Some(definition) = self
            .definitions
            .get(id)
            .and_then(|defs| canonical_definition(defs))
        {
            return Some(definition);
        }
        // Relationship fallback (Relationship.is_definition)
        if let Some(info) = self.symbols.get(id) {
            for rel in &info.relationships {
                if rel.is_definition {
                    let rel_id = SemanticSymbolID::new(
                        rel.symbol.clone(),
                        id.document.as_deref().unwrap_or(""),
                    );
                    if let Some(definition) = self
                        .definitions
                        .get(&rel_id)
                        .and_then(|defs| canonical_definition(defs))
                    {
                        return Some(definition);
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

    /// Finds an indexed lexical owner using full coordinates, without inferring calls.
    pub(crate) fn enclosing_definition(
        &self,
        path: &str,
        reference: &ScipRange,
    ) -> Option<SemanticSymbolID> {
        let start = (reference.start_line, reference.start_col);
        let end = (reference.end_line, reference.end_col);
        if start.0 < 0 || start.1 < 0 || end.1 < 0 || start >= end {
            return None;
        }
        self.documents
            .get(path)?
            .occurrences
            .iter()
            .filter(|occ| occ.is_definition())
            .filter_map(|occ| {
                let extent = occ.enclosing_range.as_ref()?;
                let lower = (extent.start_line, extent.start_col);
                let upper = (extent.end_line, extent.end_col);
                (lower.0 >= 0 && lower.1 >= 0 && lower <= start && end <= upper)
                    .then_some((lower, upper, occ))
            })
            .min_by_key(|(lower, upper, _)| {
                (
                    i64::from(upper.0) - i64::from(lower.0),
                    i64::from(upper.1) - i64::from(lower.1),
                )
            })
            .map(|(_, _, occ)| SemanticSymbolID::new(&occ.symbol, path))
    }

    /// Returns the declared column encoding for one indexed document.
    #[must_use]
    pub fn column_encoding(&self, path: &str) -> Option<crate::source::ColumnEncoding> {
        let encoding = match self.documents.get(path)?.position_encoding {
            0 => self.default_encoding,
            explicit => explicit,
        };
        match encoding {
            0 | 1 => Some(crate::source::ColumnEncoding::Utf8),
            2 => Some(crate::source::ColumnEncoding::Utf16),
            3 => Some(crate::source::ColumnEncoding::Utf32),
            _ => None,
        }
    }

    /// Returns whether semantic data can describe these exact source bytes.
    ///
    /// Imports without provenance remain usable with their unknown-freshness status.
    #[must_use]
    pub fn matches_buffer(&self, buffer: &crate::source::SourceBuffer) -> bool {
        self.source_digests.is_empty()
            || self
                .source_digests
                .get(&buffer.path)
                .is_some_and(|digest| digest == buffer.content_digest())
    }

    /// Converts an indexed range to exact canonical bytes using its document encoding.
    #[must_use]
    pub fn byte_range(
        &self,
        buffer: &crate::source::SourceBuffer,
        range: &ScipRange,
    ) -> Option<std::ops::Range<usize>> {
        if !self.matches_buffer(buffer) {
            return None;
        }
        let encoding = self.column_encoding(&buffer.path)?;
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

/// One canonical ordering for search labels, definitions and relationship fallbacks.
fn canonical_definition(definitions: &[(String, ScipRange)]) -> Option<(&str, &ScipRange)> {
    definitions
        .iter()
        .filter(|(path, range)| {
            crate::source::SourceLocation::is_repository_path(path)
                && range.start_line >= 0
                && range.start_col >= 0
        })
        .min_by(|(a, ar), (b, br)| {
            (a, ar.start_line, ar.start_col, ar.end_line, ar.end_col).cmp(&(
                b,
                br.start_line,
                br.start_col,
                br.end_line,
                br.end_col,
            ))
        })
        .map(|(path, range)| (path.as_str(), range))
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
    #[test]
    fn generated_coordinates_require_exact_buffer_and_destination_bytes() {
        use crate::source::{loader::SourceTarget, ColumnEncoding, SourceBuffer};
        let original = SourceBuffer::from_str("a.rs", "café\r\n");
        let changed = SourceBuffer::from_str("a.rs", "nope\r\n");
        let range = ScipRange::from_packed(&[0, 0, 5]).unwrap();
        let mut index = SemanticIndex::from_scip(ScipIndex {
            documents: vec![ScipDocument {
                relative_path: "a.rs".into(),
                occurrences: vec![ScipOccurrence {
                    range: Some(range.clone()),
                    symbol: "value".into(),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        });
        index
            .source_digests
            .insert("a.rs".into(), original.content_digest().into());
        assert!(index.occurrence_at_byte(&original, 1).is_some());
        assert!(index.occurrence_at_byte(&changed, 1).is_none());
        assert!(index.byte_range(&changed, &range).is_none());
        let target = SourceTarget::Encoded {
            line: 1,
            column: 0,
            encoding: ColumnEncoding::Utf8,
            content_digest: Some(original.content_digest().into()),
        };
        assert_eq!(target.resolve(&original).unwrap().byte_offset, 0);
        assert!(target
            .resolve(&changed)
            .unwrap_err()
            .contains("bytes changed"));
        index.source_digests.clear(); // An explicitly unverified imported index.
        assert!(index.matches_buffer(&changed));
    }
    #[test]
    fn search_and_definition_resolution_share_a_deterministic_destination() {
        let symbol = "scip test fixture 1 target().";
        let mut index = SemanticIndex::from_scip(ScipIndex {
            documents: ["z.ts", "a.ts"]
                .into_iter()
                .map(|path| ScipDocument {
                    relative_path: path.into(),
                    occurrences: vec![ScipOccurrence {
                        symbol: symbol.into(),
                        symbol_roles: 1,
                        range: ScipRange::from_packed(&[1, 4, 10]),
                        ..Default::default()
                    }],
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        });
        let (path, range) = index.find_definition(symbol).unwrap();
        let entry = &index.search_symbols("", 1)[0];
        assert_eq!(path, "a.ts");
        assert_eq!(path, entry.path);
        assert_eq!(range.start_line as usize + 1, entry.line);
        let id = SemanticSymbolID::new(symbol, "");
        index.definitions.get_mut(&id).unwrap().reverse();
        assert_eq!(index.find_definition_for_id(&id).unwrap().0, "a.ts");
        let boundary = SemanticIndex::from_scip(ScipIndex {
            documents: vec![ScipDocument {
                relative_path: "edge.ts".into(),
                occurrences: vec![ScipOccurrence {
                    symbol: "edge".into(),
                    symbol_roles: 1,
                    range: ScipRange::from_packed(&[i32::MAX, 0, 1]),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        });
        assert_eq!(
            boundary.search_symbols("", 1)[0].line,
            i32::MAX as usize + 1
        );
    }
    #[test]
    fn unsupported_document_encodings_are_not_guessed_as_utf8() {
        let buffer = crate::source::SourceBuffer::from_str("a.rs", "abc");
        let mut index = SemanticIndex::default();
        index.documents.insert(
            "a.rs".into(),
            ScipDocument {
                position_encoding: 99,
                ..Default::default()
            },
        );
        assert!(index.column_encoding("a.rs").is_none());
        assert!(index
            .byte_range(&buffer, &ScipRange::from_packed(&[0, 0, 1]).unwrap())
            .is_none());
    }
}

#[cfg(test)]
mod cancellation_tests;
