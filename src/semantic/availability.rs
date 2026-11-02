//! Source-specific semantic availability, independent of any desktop toolkit.

use super::{indexing::service::CacheLoadStatus, loader::IndexImportStatus, SemanticIndex};
use crate::{source::location::ReviewRevision, source::SourceBuffer};

/// Why the displayed source can or cannot use the repository's semantic data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticAvailability<'a> {
    /// No source is open.
    NoFile,
    /// No usable repository index exists; retains the lifecycle diagnostic.
    RepositoryNotIndexed(&'a str),
    /// Index work is running without a usable previous index.
    Indexing,
    /// A usable repository index does not cover this source path.
    FileNotIndexed,
    /// Generated semantic data describes different source bytes.
    SourceChanged,
    /// PR Before remains source-only; semantic navigation belongs to After.
    RevisionNotIndexed,
    /// Semantic data may be queried for this source, including unverified imports.
    Available,
}

impl<'a> SemanticAvailability<'a> {
    /// Resolves availability from the existing generation and import owners.
    #[must_use]
    pub fn from_services(
        buffer: Option<&SourceBuffer>,
        index: Option<&SemanticIndex>,
        generator: &'a super::indexing::service::IndexGenerator,
        importer: &'a super::loader::IndexImporter,
    ) -> Self {
        Self::resolve(
            buffer,
            index,
            generator.is_running() || importer.is_running(),
            generator.cache_status(),
            importer.status(),
            generator.error().or_else(|| {
                generator
                    .was_cancelled()
                    .then_some("Index generation was cancelled. Use Index Projects to try again.")
            }),
        )
    }

    /// Resolves source coverage before considering replacement-index failures.
    ///
    /// Retained indexes remain usable during generation or after failure. Imports
    /// without source provenance remain usable without claiming freshness.
    #[must_use]
    pub fn resolve(
        buffer: Option<&SourceBuffer>,
        index: Option<&SemanticIndex>,
        indexing: bool,
        cache: &'a CacheLoadStatus,
        import: &'a IndexImportStatus,
        generation_error: Option<&'a str>,
    ) -> Self {
        let Some(buffer) = buffer else {
            return Self::NoFile;
        };
        if buffer.revision == Some(ReviewRevision::Before) {
            return Self::RevisionNotIndexed;
        }
        if let Some(index) = index.filter(|index| !index.documents.is_empty()) {
            return if !index.documents.contains_key(&buffer.path) {
                Self::FileNotIndexed
            } else if !index.matches_buffer(buffer) {
                Self::SourceChanged
            } else {
                Self::Available
            };
        }
        if indexing {
            return Self::Indexing;
        }
        let reason = match (cache, generation_error, import) {
            (CacheLoadStatus::Failed(error), _, _) => error.as_str(),
            (_, Some(error), _) => error,
            (_, _, IndexImportStatus::Failed(error)) => error.as_str(),
            (CacheLoadStatus::Cancelled, _, _) | (_, _, IndexImportStatus::Cancelled) =>
                "Index loading was cancelled. Use Index Projects to try again.",
            _ if index.is_some() => "The semantic index has no documents.",
            _ => "No semantic index is available. Use Index Projects to generate one; structure browsing remains available.",
        };
        Self::RepositoryNotIndexed(reason)
    }

    /// Returns the compact status beside the Outline heading.
    #[must_use]
    pub fn header_detail(self) -> Option<&'static str> {
        match self {
            Self::NoFile | Self::Available => None,
            Self::RepositoryNotIndexed(_) => Some("No semantic index"),
            Self::Indexing => Some("Indexing…"),
            Self::FileNotIndexed => Some("File not indexed"),
            Self::SourceChanged => Some("Source changed"),
            Self::RevisionNotIndexed => Some("Before not indexed"),
        }
    }

    /// Returns the explanation shared by reference status and outline tooltips.
    #[must_use]
    pub fn explanation(self) -> Option<&'a str> {
        match self {
            Self::NoFile => Some("Open a source file."),
            Self::RepositoryNotIndexed(reason) => Some(reason),
            Self::Indexing => Some("References appear when indexing finishes."),
            Self::FileNotIndexed => Some("The repository's semantic index does not cover this file, so it has no references. Structure browsing still works."),
            Self::SourceChanged => Some("This file has changed since it was indexed. Reindex to restore exact semantic navigation; structure browsing still works."),
            Self::RevisionNotIndexed => Some("The Before revision of a pull request is not indexed. Semantic navigation applies to After; structure browsing works in both."),
            Self::Available => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{ScipDocument, ScipIndex};

    #[test]
    fn coverage_and_exact_source_take_precedence_over_replacement_lifecycle() {
        let source = SourceBuffer::from_str("a.rs", "fn alpha() {}\n");
        let changed = SourceBuffer::from_str("a.rs", "fn beta() {}\n");
        let other = SourceBuffer::from_str("other.rs", "fn alpha() {}\n");
        let mut before = source.clone();
        before.revision = Some(ReviewRevision::Before);
        let mut index = SemanticIndex::from_scip(ScipIndex {
            documents: vec![ScipDocument {
                relative_path: "a.rs".into(),
                ..Default::default()
            }],
            ..Default::default()
        });
        for running in [false, true] {
            for cache in [
                CacheLoadStatus::Idle,
                CacheLoadStatus::Failed("cache broken".into()),
            ] {
                let import = IndexImportStatus::Failed("import broken".into());
                let resolve = |buffer| {
                    SemanticAvailability::resolve(
                        buffer,
                        Some(&index),
                        running,
                        &cache,
                        &import,
                        Some("generation failed"),
                    )
                };
                assert_eq!(resolve(None), SemanticAvailability::NoFile);
                assert_eq!(
                    resolve(Some(&before)),
                    SemanticAvailability::RevisionNotIndexed
                );
                assert_eq!(resolve(Some(&other)), SemanticAvailability::FileNotIndexed);
                assert_eq!(resolve(Some(&source)), SemanticAvailability::Available);
                // Unverified imports keep their existing usable, unknown-freshness behavior.
                assert_eq!(resolve(Some(&changed)), SemanticAvailability::Available);
            }
        }
        index
            .source_digests
            .insert(source.path.clone(), source.content_digest().into());
        assert_eq!(
            SemanticAvailability::resolve(
                Some(&source),
                Some(&index),
                true,
                &CacheLoadStatus::Loading,
                &IndexImportStatus::Idle,
                None
            ),
            SemanticAvailability::Available
        );
        assert_eq!(
            SemanticAvailability::resolve(
                Some(&changed),
                Some(&index),
                false,
                &CacheLoadStatus::Idle,
                &IndexImportStatus::Idle,
                None
            ),
            SemanticAvailability::SourceChanged
        );
    }

    #[test]
    fn absent_or_empty_indexes_preserve_lifecycle_reasons_and_before_scope() {
        let source = SourceBuffer::from_str("a.rs", "fn alpha() {}\n");
        let empty = SemanticIndex::from_scip(ScipIndex::default());
        for index in [None, Some(&empty)] {
            for (cache, import, error, expected) in [
                (
                    CacheLoadStatus::Failed("cache broken".into()),
                    IndexImportStatus::Failed("import broken".into()),
                    Some("generator failed"),
                    "cache broken",
                ),
                (
                    CacheLoadStatus::Idle,
                    IndexImportStatus::Failed("import broken".into()),
                    Some("generator failed"),
                    "generator failed",
                ),
                (
                    CacheLoadStatus::Idle,
                    IndexImportStatus::Failed("import broken".into()),
                    None,
                    "import broken",
                ),
                (
                    CacheLoadStatus::Cancelled,
                    IndexImportStatus::Idle,
                    None,
                    "Index loading was cancelled. Use Index Projects to try again.",
                ),
            ] {
                assert_eq!(
                    SemanticAvailability::resolve(
                        Some(&source),
                        index,
                        false,
                        &cache,
                        &import,
                        error
                    ),
                    SemanticAvailability::RepositoryNotIndexed(expected)
                );
                assert_eq!(
                    SemanticAvailability::resolve(
                        Some(&source),
                        index,
                        true,
                        &cache,
                        &import,
                        error
                    ),
                    SemanticAvailability::Indexing
                );
            }
        }
        let resolve = |buffer, index| {
            SemanticAvailability::resolve(
                buffer,
                index,
                false,
                &CacheLoadStatus::Idle,
                &IndexImportStatus::Idle,
                None,
            )
        };
        assert!(resolve(Some(&source), None)
            .explanation()
            .unwrap()
            .contains("Index Projects"));
        assert_eq!(
            resolve(Some(&source), Some(&empty)),
            SemanticAvailability::RepositoryNotIndexed("The semantic index has no documents.")
        );
        let mut before = source.clone();
        before.revision = Some(ReviewRevision::Before);
        assert_eq!(
            resolve(Some(&before), None),
            SemanticAvailability::RevisionNotIndexed
        );
    }
}
