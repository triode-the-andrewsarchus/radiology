//! Precomputed explicit trait implementations joined by exact SCIP identity.
use super::{index::SemanticSymbolID, SemanticIndex};
use crate::source::{
    loader::{check_source, SourceRead, SOURCE_READ_TIMEOUT},
    ReviewRevision, SourceBuffer, SourceLocation,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    ops::Range,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
mod cache;

/// An explicit impl tied to the exact source bytes used during enrichment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitImplementation {
    /// Workspace-global implemented trait identity, never a display-name join.
    pub trait_symbol: String,
    /// Canonical repository-relative source path.
    pub path: String,
    /// Full original impl bytes.
    pub byte_range: Range<usize>,
    /// Implemented trait's terminal name bytes.
    pub trait_name_range: Range<usize>,
    /// One-based source line of the impl declaration.
    pub line: usize,
    /// Original trimmed header, including modifiers and constraints.
    pub signature: String,
    /// Exact implementing type spelling.
    pub target_type: String,
    /// Explicit negative implementation.
    pub is_negative: bool,
    /// SHA-256 identity required for activation.
    pub source_digest: String,
}

impl TraitImplementation {
    /// Revalidates the entire source before resolving a navigation destination.
    ///
    /// # Errors
    /// Rejects stale bytes, invalid ranges, unreadable paths and canceled reads.
    pub fn resolve(
        &self,
        source: &SourceRead,
        cancel: &AtomicBool,
    ) -> Result<SourceLocation, String> {
        let buffer = read(source, &self.path, cancel)?;
        if buffer.content_digest() != self.source_digest
            || self.byte_range.is_empty()
            || self.byte_range.end > buffer.byte_len()
            || !buffer.text.is_char_boundary(self.byte_range.start)
        {
            return Err("Implementation source changed; reindex before navigating".into());
        }
        Ok(buffer.location_at_byte(self.byte_range.start))
    }
}

/// Immutable source-validated lookup, including explicit incomplete-coverage state.
#[derive(Debug, Clone, Default)]
pub struct TraitImplementationIndex {
    /// Source blocks per exact trait, sorted by path then byte offset.
    pub by_trait: HashMap<SemanticSymbolID, Vec<TraitImplementation>>,
    /// Impl headers without exactly one nonlocal SCIP reference identity.
    pub unresolved_count: usize,
    /// Rust documents that could not be read.
    pub unreadable_paths: Vec<String>,
    /// Documents with parser errors or unmappable semantic occurrences.
    pub parse_error_paths: Vec<String>,
    /// Explicitly excluded or source-stale semantic documents.
    pub excluded_paths: Vec<String>,
    /// Repository Rust files missing from the index; None when inventory is unknown.
    pub unindexed_file_count: Option<usize>,
    /// Whether enrichment completed, including empty results.
    pub is_prepared: bool,
    /// Parsed documents in this build, for cache verification rather than UI.
    pub parsed_document_count: usize,
    /// Exact cache records reused in this build.
    pub cached_document_count: usize,
    /// Imported semantic data has no source freshness provenance.
    pub unverified_semantics: bool,
}

impl TraitImplementationIndex {
    /// Enriches an immutable index from checkout or pinned After source on a worker.
    ///
    /// `repository_paths` must describe the full inventory, not browser filters.
    /// An optional cache directory must be app-owned and outside the source root.
    /// Missing, corrupt or unwritable caches affect speed only. Excluded and stale
    /// documents never contribute rows. A warm cache still verifies actual bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{collections::HashSet, sync::{Arc, atomic::AtomicBool}};
    /// use radiology::{semantic::{SemanticIndex, ScipIndex, ScipDocument, ScipOccurrence, ScipRange,
    ///     trait_impls::TraitImplementationIndex}, source::{SourceBuffer, loader::SourceRead}};
    /// let source = SourceRead::Buffer(Arc::new(SourceBuffer::from_str("a.rs", "impl Read for File {}")));
    /// let index = SemanticIndex::from_scip(ScipIndex { documents: vec![ScipDocument {
    ///     relative_path: "a.rs".into(), position_encoding: 1, occurrences: vec![ScipOccurrence {
    ///         symbol: "scip test fixture 1 Read#".into(), range: ScipRange::from_packed(&[0,5,9]),
    ///         ..Default::default() }], ..Default::default() }], ..Default::default() });
    /// let catalog = TraitImplementationIndex::build(&index, &source, Some(&["a.rs".into()]),
    ///     None, &HashSet::new(), &AtomicBool::new(false)).unwrap();
    /// assert_eq!(catalog.by_trait.values().next().unwrap()[0].target_type, "File");
    /// ```
    ///
    /// # Errors
    /// Returns cancellation failures; per-file read/timeouts/parse gaps are retained.
    pub fn build(
        index: &SemanticIndex,
        source: &SourceRead,
        repository_paths: Option<&[String]>,
        cache_directory: Option<&Path>,
        excluded: &HashSet<String>,
        cancel: &AtomicBool,
    ) -> Result<Self, String> {
        let destination = cache_directory.and_then(|directory| cache::path(source, directory));
        let previous = destination
            .as_deref()
            .map(|path| cache::load(path, cancel))
            .unwrap_or_default();
        check(cancel)?;
        let mut next = cache::Cache::default();
        let mut result = Self {
            is_prepared: true,
            unverified_semantics: index.source_digests.is_empty(),
            ..Self::default()
        };
        let mut documents: Vec<_> = index
            .documents
            .values()
            .filter(|doc| doc.relative_path.ends_with(".rs"))
            .collect();
        documents.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        let indexed: HashSet<_> = documents
            .iter()
            .map(|doc| doc.relative_path.as_str())
            .collect();
        result.unindexed_file_count = repository_paths.map(|paths| {
            paths
                .iter()
                .filter(|path| path.ends_with(".rs") && !indexed.contains(path.as_str()))
                .collect::<HashSet<_>>()
                .len()
        });
        for document in documents {
            check(cancel)?;
            let path = &document.relative_path;
            if excluded.contains(path) {
                result.excluded_paths.push(path.clone());
                continue;
            }
            let buffer = match read(source, path, cancel) {
                Ok(buffer) => buffer,
                Err(_) => {
                    check(cancel)?;
                    result.unreadable_paths.push(path.clone());
                    continue;
                }
            };
            if !index.matches_buffer(&buffer) {
                result.excluded_paths.push(path.clone());
                continue;
            }
            let (record, cached) =
                FileRecord::load(index, document, &buffer, previous.files.get(path), cancel)?;
            result.cached_document_count += usize::from(cached);
            result.parsed_document_count += usize::from(!cached);
            result.unresolved_count += record.unresolved;
            if record.has_errors {
                result.parse_error_paths.push(path.clone());
            }
            for item in &record.implementations {
                result
                    .by_trait
                    .entry(SemanticSymbolID::new(&item.trait_symbol, path))
                    .or_default()
                    .push(item.clone());
            }
            next.files.insert(path.clone(), record);
        }
        check(cancel)?;
        if let Some(path) = destination {
            cache::save(&path, &next, cancel);
        }
        check(cancel)?;
        Ok(result)
    }

    /// Returns truthful coverage context for a tooltip or diagnostics view.
    #[must_use]
    pub fn coverage_detail(&self) -> String {
        let mut notes = vec!["Explicit impls in indexed source; macro-generated impls and inactive configurations may be absent.".to_owned()];
        if !self.is_prepared {
            notes.push("Implementation data has not been prepared.".into());
        }
        if self.unverified_semantics {
            notes.push("Imported semantic source freshness is unknown.".into());
        }
        if self.unresolved_count > 0 {
            notes.push(format!(
                "{} impl headers could not be resolved.",
                self.unresolved_count
            ));
        }
        match self.unindexed_file_count {
            Some(0) => {}
            Some(count) => notes.push(format!("{count} Rust files are not indexed.")),
            None => notes.push("Repository Rust file coverage is unknown.".into()),
        }
        for (paths, label) in [
            (&self.excluded_paths, "Stale/excluded semantic files"),
            (&self.unreadable_paths, "Unreadable files"),
            (&self.parse_error_paths, "Parse or coordinate errors"),
        ] {
            if !paths.is_empty() {
                notes.push(format!("{label}: {}.", paths.join(", ")));
            }
        }
        notes.join(" ")
    }

    /// Returns whether known gaps prevent a complete-source interpretation.
    #[must_use]
    pub fn has_gaps(&self) -> bool {
        !self.is_prepared
            || self.unresolved_count > 0
            || self.unindexed_file_count != Some(0)
            || !self.unreadable_paths.is_empty()
            || !self.parse_error_paths.is_empty()
            || !self.excluded_paths.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileRecord {
    key: String,
    implementations: Vec<TraitImplementation>,
    unresolved: usize,
    has_errors: bool,
}

impl FileRecord {
    fn load(
        index: &SemanticIndex,
        document: &super::ScipDocument,
        buffer: &SourceBuffer,
        cached: Option<&Self>,
        cancel: &AtomicBool,
    ) -> Result<(Self, bool), String> {
        let key = cache::key(index, document, buffer, cancel)?;
        match cached.filter(|record| record.key == key) {
            Some(record) => Ok((record.clone(), true)),
            None => Ok((Self::build(index, document, buffer, key, cancel)?, false)),
        }
    }

    fn build(
        index: &SemanticIndex,
        document: &super::ScipDocument,
        buffer: &SourceBuffer,
        key: String,
        cancel: &AtomicBool,
    ) -> Result<Self, String> {
        let syntax =
            match crate::syntax::rust_impls::RustTraitImpls::analyze_cancellable(buffer, || {
                cancel.load(Ordering::Relaxed)
            }) {
                Ok(syntax) => syntax,
                Err(_) => {
                    check(cancel)?;
                    crate::syntax::rust_impls::RustTraitImpls {
                        has_errors: true,
                        ..Default::default()
                    }
                }
            };
        let mut record = Self {
            key,
            implementations: Vec::new(),
            unresolved: 0,
            has_errors: syntax.has_errors,
        };
        let mut references: HashMap<Range<usize>, HashSet<String>> = HashMap::new();
        for occurrence in &document.occurrences {
            check(cancel)?;
            let mapped = occurrence
                .range
                .as_ref()
                .and_then(|range| index.byte_range(buffer, range));
            let Some(range) = mapped else {
                record.has_errors = true;
                continue;
            };
            if !occurrence.is_definition()
                && !occurrence.symbol.is_empty()
                && !occurrence.symbol.starts_with("local ")
            {
                references
                    .entry(range)
                    .or_default()
                    .insert(occurrence.symbol.clone());
            }
        }
        for item in syntax.implementations {
            check(cancel)?;
            let candidates = references.get(&item.trait_name_range);
            let Some(trait_symbol) = candidates
                .filter(|ids| ids.len() == 1)
                .and_then(|ids| ids.iter().next())
            else {
                record.unresolved += 1;
                continue;
            };
            record.implementations.push(TraitImplementation {
                trait_symbol: trait_symbol.clone(),
                path: buffer.path.clone(),
                line: buffer.line_at_byte(item.byte_range.start),
                byte_range: item.byte_range,
                trait_name_range: item.trait_name_range,
                signature: item.signature,
                target_type: item.target_type,
                is_negative: item.is_negative,
                source_digest: buffer.content_digest().to_owned(),
            });
        }
        Ok(record)
    }
}

fn check(cancel: &AtomicBool) -> Result<(), String> {
    check_source(cancel, Instant::now() + SOURCE_READ_TIMEOUT)
}

fn location(source: &SourceRead, path: &str) -> SourceLocation {
    SourceLocation::with_revision(
        path,
        0,
        matches!(source, SourceRead::Review(_)).then_some(ReviewRevision::After),
    )
}

fn read(
    source: &SourceRead,
    path: &str,
    cancel: &AtomicBool,
) -> Result<std::sync::Arc<SourceBuffer>, String> {
    source.read_contained_cancellable(
        &location(source, path),
        cancel,
        Instant::now() + SOURCE_READ_TIMEOUT,
    )
}

#[cfg(test)]
pub(crate) mod tests;
