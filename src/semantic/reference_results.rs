//! Source-validated definitions and references with explicit query coverage.

use super::{index::SemanticSymbolID, ScipRange};
use crate::source::{
    preview::{preview_line, PreviewLine, SourceQueryInputs},
    ReviewRevision, SourceBuffer,
};
use std::{
    collections::BTreeMap,
    ops::Range,
    sync::atomic::{AtomicBool, Ordering},
};

/// One indexed site resolved against exact source bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceHit {
    /// Repository-relative destination path.
    pub path: String,
    /// Original coordinates, retained for provenance-aware navigation.
    pub range: ScipRange,
    /// Canonical UTF-8 range in the validated source.
    pub byte_range: Range<usize>,
    /// Bounded first-line display text and exact visible match emphasis.
    pub source_line: PreviewLine,
    /// Whether this site defines the queried symbol.
    pub is_definition: bool,
}

/// One file's contiguous range in [`ReferenceResults::hits`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceGroup {
    /// Repository-relative file path.
    pub path: String,
    /// Hit indices in source order, including definitions.
    pub hits: Range<usize>,
    /// Exact source identity used to validate this file's hits.
    pub content_digest: String,
}

/// An indexed file that could not be read safely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnreadableReferenceFile {
    /// Indexed repository-relative path.
    pub path: String,
    /// Read/validation failure, suitable for an explanatory tooltip.
    pub message: String,
    /// Number of unique sites omitted with this file.
    pub sites: usize,
}

/// Results distinguish real references, definitions and incomplete coverage.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReferenceResults {
    /// Validated sites sorted by path and source position.
    pub hits: Vec<ReferenceHit>,
    /// Nonempty file groups, including exact source identities.
    pub groups: Vec<ReferenceGroup>,
    /// SHA-256 identities for every accepted source read, including unmapped files.
    pub source_identities: BTreeMap<String, String>,
    /// Validated uses, excluding definitions.
    pub reference_count: usize,
    /// Validated definitions, excluded from the reference count.
    pub definition_count: usize,
    /// Unique indexed sites whose source identity, encoding or range did not map.
    pub unmapped: usize,
    /// Files that could not be read, retaining their omitted-site counts.
    pub unreadable: Vec<UnreadableReferenceFile>,
    /// Unique sites not processed within the current resource limits.
    pub omitted: usize,
    /// Total unique indexed sites, before source validation.
    pub indexed_sites: usize,
}

impl ReferenceResults {
    /// Finds a listed site by its original encoded coordinates.
    #[must_use]
    pub fn find_hit(&self, path: &str, range: &ScipRange) -> Option<usize> {
        let group = &self.groups[self
            .groups
            .binary_search_by(|group| group.path.as_str().cmp(path))
            .ok()?];
        let offset = self.hits[group.hits.clone()]
            .binary_search_by_key(&coordinates(range), |hit| coordinates(&hit.range))
            .ok()?;
        Some(group.hits.start + offset)
    }

    pub(crate) fn source_digest(&self, path: &str) -> Option<&str> {
        self.source_identities.get(path).map(String::as_str)
    }
}

fn coordinates(range: &ScipRange) -> Coordinates {
    (
        range.start_line,
        range.start_col,
        range.end_line,
        range.end_col,
    )
}

/// Per-request bounds can grow when the reader explicitly requests more results.
#[derive(Debug, Clone, Copy)]
pub struct ReferenceLimits {
    /// Total source bytes accepted across files.
    pub source_bytes: usize,
    /// Total retained row text, paths and result metadata bytes.
    pub result_bytes: usize,
    /// Number of file reads attempted, including failures.
    pub files: usize,
}

impl Default for ReferenceLimits {
    fn default() -> Self {
        Self {
            source_bytes: 64 * 1024 * 1024,
            result_bytes: 16 * 1024 * 1024,
            files: 512,
        }
    }
}

impl ReferenceLimits {
    /// Doubles each limit without integer overflow.
    #[must_use]
    pub fn expanded(self) -> Self {
        Self {
            source_bytes: self.source_bytes.saturating_mul(2),
            result_bytes: self.result_bytes.saturating_mul(2),
            files: self.files.saturating_mul(2),
        }
    }
}

type Coordinates = (i32, i32, i32, i32);
type Sites = BTreeMap<String, BTreeMap<Coordinates, bool>>;

/// Resolves all indexed definitions and uses against captured source storage.
///
/// Each file is read once, with displayed bytes taking precedence. Duplicate
/// coordinates collapse to one site, preferring a definition role. Resource
/// limits stop a sorted prefix and report every remaining site in `omitted`.
///
/// # Errors
/// Returns cancellation or an attempt to use After semantics in Before source.
/// Individual file/range failures remain visible in the returned coverage.
pub fn query(
    inputs: &SourceQueryInputs,
    id: &SemanticSymbolID,
    limits: ReferenceLimits,
    cancel: &AtomicBool,
) -> Result<ReferenceResults, String> {
    if inputs.buffer.revision == Some(ReviewRevision::Before) {
        return Err("After reference data is unavailable in Before source.".into());
    }
    let positions = positions(inputs, id, cancel)?;
    let mut results = ReferenceResults {
        indexed_sites: positions.values().map(BTreeMap::len).sum(),
        ..Default::default()
    };
    let mut remaining = results.indexed_sites;
    let mut budget = QueryBudget {
        limits,
        source_bytes: 0,
        result_bytes: 0,
        files: 0,
        stopped: false,
    };
    for (path, sites) in positions {
        check_cancel(cancel)?;
        if budget.exhausted() {
            results.omitted += remaining;
            break;
        }
        remaining -= sites.len();
        resolve_file(inputs, path, sites, &mut budget, &mut results, cancel)?;
    }
    Ok(results)
}

fn positions(
    inputs: &SourceQueryInputs,
    id: &SemanticSymbolID,
    cancel: &AtomicBool,
) -> Result<Sites, String> {
    let mut positions = Sites::new();
    let definitions = inputs.index.definition_locations_for_id(id);
    let uses = inputs
        .index
        .references
        .get(id)
        .into_iter()
        .flatten()
        .map(|(path, range)| (path.as_str(), range));
    let sites = definitions
        .into_iter()
        .map(|(path, range)| (path, range, true))
        .chain(uses.map(|(path, range)| (path, range, false)));
    for (path, range, definition) in sites {
        check_cancel(cancel)?;
        positions
            .entry(path.to_owned())
            .or_default()
            .entry((
                range.start_line,
                range.start_col,
                range.end_line,
                range.end_col,
            ))
            .and_modify(|is_definition| *is_definition |= definition)
            .or_insert(definition);
    }

    Ok(positions)
}

struct QueryBudget {
    limits: ReferenceLimits,
    source_bytes: usize,
    result_bytes: usize,
    files: usize,
    stopped: bool,
}

impl QueryBudget {
    fn exhausted(&self) -> bool {
        self.stopped
            || self.source_bytes >= self.limits.source_bytes
            || self.result_bytes >= self.limits.result_bytes
            || self.files >= self.limits.files
    }
    fn retain(&mut self, bytes: usize) -> bool {
        if bytes > self.limits.result_bytes.saturating_sub(self.result_bytes) {
            self.stopped = true;
            false
        } else {
            self.result_bytes += bytes;
            true
        }
    }
}

fn resolve_file(
    inputs: &SourceQueryInputs,
    path: String,
    sites: BTreeMap<Coordinates, bool>,
    budget: &mut QueryBudget,
    results: &mut ReferenceResults,
    cancel: &AtomicBool,
) -> Result<(), String> {
    if !budget.retain(path.len().saturating_mul(2).saturating_add(
        std::mem::size_of::<ReferenceGroup>() + std::mem::size_of::<(String, String)>() + 128,
    )) {
        results.omitted += sites.len();
        return Ok(());
    }
    budget.files += 1;
    let buffer = match inputs.read(&path) {
        Ok(buffer) => buffer,
        Err(message) => {
            let message: String = message.chars().take(1024).collect();
            if !budget.retain(message.len()) {
                results.omitted += sites.len();
                return Ok(());
            }
            results.unreadable.push(UnreadableReferenceFile {
                path,
                message,
                sites: sites.len(),
            });
            return Ok(());
        }
    };
    check_cancel(cancel)?;
    if buffer.byte_len()
        > budget
            .limits
            .source_bytes
            .saturating_sub(budget.source_bytes)
    {
        budget.stopped = true;
        results.omitted += sites.len();
        return Ok(());
    }
    budget.source_bytes += buffer.byte_len();
    results
        .source_identities
        .insert(path.clone(), buffer.content_digest().to_owned());
    let first = results.hits.len();
    let total = sites.len();
    for (processed, ((start_line, start_col, end_line, end_col), is_definition)) in
        sites.into_iter().enumerate()
    {
        check_cancel(cancel)?;
        let range = ScipRange {
            start_line,
            start_col,
            end_line,
            end_col,
        };
        let Some(hit) = hit(inputs, &buffer, range, is_definition) else {
            results.unmapped += 1;
            continue;
        };
        let cost = std::mem::size_of::<ReferenceHit>()
            .saturating_add(hit.path.len())
            .saturating_add(hit.source_line.text.len());
        if !budget.retain(cost) {
            results.omitted += total - processed;
            break;
        }
        results.reference_count += usize::from(!hit.is_definition);
        results.definition_count += usize::from(hit.is_definition);
        results.hits.push(hit);
    }
    if first < results.hits.len() {
        results.groups.push(ReferenceGroup {
            path,
            hits: first..results.hits.len(),
            content_digest: buffer.content_digest().to_owned(),
        });
    }
    Ok(())
}

fn hit(
    inputs: &SourceQueryInputs,
    buffer: &SourceBuffer,
    range: ScipRange,
    is_definition: bool,
) -> Option<ReferenceHit> {
    let bytes = inputs
        .index
        .byte_range(buffer, &range)
        .filter(|range| !range.is_empty())?;
    let mut source_line = preview_line(buffer, buffer.line_at_byte(bytes.start), &bytes);
    let indentation = source_line
        .text
        .bytes()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count()
        .min(
            source_line
                .emphasis
                .as_ref()
                .map_or(usize::MAX, |range| range.start),
        );
    source_line.text.drain(..indentation);
    if let Some(range) = &mut source_line.emphasis {
        range.start -= indentation;
        range.end -= indentation;
    }
    Some(ReferenceHit {
        path: buffer.path.clone(),
        range,
        byte_range: bytes,
        source_line,
        is_definition,
    })
}

pub(crate) fn check_cancel(cancel: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        Err("Reference query cancelled.".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        semantic::{ScipDocument, ScipIndex, ScipOccurrence, SemanticIndex},
        source::loader::SourceRead,
    };
    use std::sync::Arc;

    const SYMBOL: &str = "scip test fixture 1 target().";

    fn occurrence(
        buffer: &SourceBuffer,
        text: &str,
        symbol: &str,
        definition: bool,
    ) -> ScipOccurrence {
        let byte = buffer.text.find(text).unwrap();
        let line = buffer.line_at_byte(byte);
        let column = buffer.text[buffer.line_starts[line - 1]..byte]
            .encode_utf16()
            .count() as i32;
        ScipOccurrence {
            symbol: symbol.into(),
            symbol_roles: i32::from(definition),
            range: ScipRange::from_packed(&[
                line as i32 - 1,
                column,
                column + text.encode_utf16().count() as i32,
            ]),
            ..Default::default()
        }
    }

    fn fixture() -> (tempfile::TempDir, SourceQueryInputs, SemanticSymbolID) {
        let root = tempfile::tempdir().unwrap();
        let a = Arc::new(SourceBuffer::from_str("a.rs", "fn target() {}\n"));
        let b = SourceBuffer::from_str("b.rs", "fn caller() { let café = \"😀\"; target(); }\n");
        for buffer in [&*a, &b] {
            std::fs::write(root.path().join(&buffer.path), &buffer.text).unwrap();
        }
        let mut index = SemanticIndex::from_scip(ScipIndex {
            documents: vec![
                ScipDocument {
                    relative_path: a.path.clone(),
                    position_encoding: 2,
                    occurrences: vec![occurrence(&a, "target", SYMBOL, true)],
                    ..Default::default()
                },
                ScipDocument {
                    relative_path: b.path.clone(),
                    position_encoding: 2,
                    occurrences: vec![occurrence(&b, "target", SYMBOL, false)],
                    ..Default::default()
                },
            ],
            ..Default::default()
        });
        for buffer in [&*a, &b] {
            index
                .source_digests
                .insert(buffer.path.clone(), buffer.content_digest().into());
        }
        let inputs = SourceQueryInputs {
            source: SourceRead::Checkout(root.path().into()),
            buffer: a,
            index: Arc::new(index),
        };
        (root, inputs, SemanticSymbolID::new(SYMBOL, "a.rs"))
    }

    fn run(inputs: &SourceQueryInputs, id: &SemanticSymbolID) -> ReferenceResults {
        let results = query(
            inputs,
            id,
            ReferenceLimits::default(),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(
            results.indexed_sites,
            results.hits.len()
                + results.unmapped
                + results.omitted
                + results
                    .unreadable
                    .iter()
                    .map(|file| file.sites)
                    .sum::<usize>()
        );
        results
    }

    #[test]
    fn groups_real_source_definitions_and_utf16_references_without_counting_definitions_as_uses() {
        let (_root, inputs, id) = fixture();
        let results = run(&inputs, &id);
        assert_eq!((results.reference_count, results.definition_count), (1, 1));
        assert_eq!(
            results
                .groups
                .iter()
                .map(|group| (group.path.as_str(), group.hits.clone()))
                .collect::<Vec<_>>(),
            [("a.rs", 0..1), ("b.rs", 1..2)]
        );
        let reference = &results.hits[1];
        assert_eq!(
            &reference.source_line.text[reference.source_line.emphasis.clone().unwrap()],
            "target"
        );
        assert_eq!(reference.byte_range.len(), 6);
        assert!(
            reference.byte_range.start > reference.range.start_col as usize,
            "UTF-16 columns were treated as UTF-8 bytes"
        );
        assert_eq!(
            results.groups[0].content_digest,
            inputs.buffer.content_digest()
        );
    }

    #[test]
    fn duplicate_coordinates_prefer_definition_roles_and_locals_stay_document_scoped() {
        let (_root, mut inputs, id) = fixture();
        let index = Arc::make_mut(&mut inputs.index);
        let definition = index.definitions[&id][0].clone();
        index
            .references
            .get_mut(&id)
            .unwrap()
            .extend([definition.clone(), definition]);
        let local = SemanticSymbolID::new("local 1", "a.rs");
        index.references.insert(
            local.clone(),
            vec![("a.rs".into(), ScipRange::from_packed(&[0, 3, 9]).unwrap())],
        );
        index.references.insert(
            SemanticSymbolID::new("local 1", "b.rs"),
            vec![("b.rs".into(), ScipRange::from_packed(&[0, 0, 2]).unwrap())],
        );
        let results = run(&inputs, &id);
        assert_eq!(
            (
                results.indexed_sites,
                results.definition_count,
                results.reference_count
            ),
            (2, 1, 1)
        );
        let local_results = run(&inputs, &local);
        assert_eq!(local_results.hits.len(), 1);
        assert_eq!(local_results.hits[0].path, "a.rs");
    }

    #[test]
    fn stale_ranges_encodings_missing_files_and_bad_paths_have_explicit_coverage() {
        let (root, mut inputs, id) = fixture();
        std::fs::write(root.path().join("b.rs"), "changed bytes").unwrap();
        let index = Arc::make_mut(&mut inputs.index);
        index.references.get_mut(&id).unwrap().extend([
            (
                "a.rs".into(),
                ScipRange {
                    start_col: -1,
                    ..Default::default()
                },
            ),
            (
                "missing.rs".into(),
                ScipRange::from_packed(&[0, 0, 1]).unwrap(),
            ),
            (
                "../escape.rs".into(),
                ScipRange::from_packed(&[0, 0, 1]).unwrap(),
            ),
        ]);
        let results = run(&inputs, &id);
        assert_eq!(results.unmapped, 2);
        assert!(results.source_identities.contains_key("b.rs"));
        assert_eq!(results.unreadable.len(), 2);
        assert_eq!(results.hits.len(), 1);
        index_encoding_unknown(&mut inputs, "a.rs");
        assert_eq!(run(&inputs, &id).unmapped, 3);
    }

    fn index_encoding_unknown(inputs: &mut SourceQueryInputs, path: &str) {
        Arc::make_mut(&mut inputs.index)
            .documents
            .get_mut(path)
            .unwrap()
            .position_encoding = 99;
    }

    #[test]
    fn resource_limits_report_a_prefix_and_can_expand_to_complete_results() {
        let (_root, inputs, id) = fixture();
        let limited = ReferenceLimits {
            files: 1,
            ..Default::default()
        };
        let result = query(&inputs, &id, limited, &AtomicBool::new(false)).unwrap();
        assert_eq!((result.hits.len(), result.omitted), (1, 1));
        let complete = query(&inputs, &id, limited.expanded(), &AtomicBool::new(false)).unwrap();
        assert_eq!((complete.hits.len(), complete.omitted), (2, 0));
        for limits in [
            ReferenceLimits {
                source_bytes: 1,
                ..Default::default()
            },
            ReferenceLimits {
                result_bytes: 1,
                ..Default::default()
            },
        ] {
            let result = query(&inputs, &id, limits, &AtomicBool::new(false)).unwrap();
            assert_eq!(result.omitted, 2);
            assert!(result.hits.is_empty());
        }
    }

    #[test]
    fn displayed_bytes_win_over_disk_and_before_or_cancelled_queries_do_not_publish() {
        let (root, mut inputs, id) = fixture();
        std::fs::write(root.path().join("a.rs"), "checkout changed").unwrap();
        assert!(run(&inputs, &id).hits[0]
            .source_line
            .text
            .contains("fn target"));
        assert!(query(
            &inputs,
            &id,
            ReferenceLimits::default(),
            &AtomicBool::new(true)
        )
        .is_err());
        Arc::make_mut(&mut inputs.buffer).revision = Some(ReviewRevision::Before);
        assert!(query(
            &inputs,
            &id,
            ReferenceLimits::default(),
            &AtomicBool::new(false)
        )
        .unwrap_err()
        .contains("Before"));
    }

    #[test]
    fn relationship_definitions_include_every_indexed_destination() {
        let (_root, mut inputs, id) = fixture();
        let related = SemanticSymbolID::new("scip test fixture 1 implementation().", "");
        let index = Arc::make_mut(&mut inputs.index);
        let mut definitions = index.definitions.remove(&id).unwrap();
        definitions.push(index.references[&id][0].clone());
        index.definitions.insert(related.clone(), definitions);
        index
            .symbols
            .entry(id.clone())
            .or_default()
            .relationships
            .push(crate::semantic::scip_proto::ScipRelationship {
                symbol: related.symbol,
                is_definition: true,
                ..Default::default()
            });
        let results = run(&inputs, &id);
        assert_eq!(results.definition_count, 2);
        assert_eq!(results.reference_count, 0);
    }

    #[test]
    fn multiline_and_long_unicode_sites_keep_visible_emphasis_and_exact_navigation_ranges() {
        let (_root, mut inputs, id) = fixture();
        let text = format!("    {}target\n continuation\n", "é".repeat(3000));
        let buffer = Arc::new(SourceBuffer::from_str("a.rs", &text));
        let index = Arc::make_mut(&mut inputs.index);
        index.source_digests.clear();
        index.definitions.clear();
        index.references.insert(
            id.clone(),
            vec![(
                "a.rs".into(),
                ScipRange::from_packed(&[0, 3004, 1, 4]).unwrap(),
            )],
        );
        inputs.buffer = buffer;
        let results = run(&inputs, &id);
        let hit = &results.hits[0];
        assert!(hit.source_line.clipped);
        assert_eq!(
            &hit.source_line.text[hit.source_line.emphasis.clone().unwrap()],
            "target"
        );
        assert_eq!(hit.range.end_line, 1);
        assert!(hit.byte_range.end > hit.byte_range.start + "target".len());
    }
    #[test]
    fn pinned_review_results_resolve_git_bytes_independently_of_attached_checkout() {
        let (_temp, root, snapshot) = crate::pull_request::review::tests::fixture_with_sources(
            "fn old() {}\n",
            "fn target() { review_only(); }\n",
        );
        let source = SourceRead::Review(Arc::new(snapshot));
        let head = source
            .read(&crate::source::SourceLocation::with_revision(
                "a file.rs",
                0,
                Some(ReviewRevision::After),
            ))
            .unwrap();
        let displayed = source
            .read(&crate::source::SourceLocation::with_revision(
                "added.rs",
                0,
                Some(ReviewRevision::After),
            ))
            .unwrap();
        let mut index = SemanticIndex::from_scip(ScipIndex {
            documents: vec![
                ScipDocument {
                    relative_path: head.path.clone(),
                    position_encoding: 2,
                    occurrences: vec![occurrence(&head, "target", SYMBOL, true)],
                    ..Default::default()
                },
                ScipDocument {
                    relative_path: displayed.path.clone(),
                    position_encoding: 2,
                    occurrences: vec![occurrence(&displayed, "new", SYMBOL, false)],
                    ..Default::default()
                },
            ],
            ..Default::default()
        });
        for buffer in [&*head, &*displayed] {
            index
                .source_digests
                .insert(buffer.path.clone(), buffer.content_digest().into());
        }
        std::fs::write(root.join("a file.rs"), "fn checkout_only() {}\n").unwrap();
        let inputs = SourceQueryInputs {
            source,
            buffer: displayed,
            index: Arc::new(index),
        };
        let results = run(&inputs, &SemanticSymbolID::new(SYMBOL, ""));
        assert_eq!((results.reference_count, results.definition_count), (1, 1));
        assert!(results.hits[0].source_line.text.contains("review_only"));
        assert_eq!(results.groups[0].content_digest, head.content_digest());
    }

    #[test]
    fn empty_display_lines_still_consume_a_bounded_result_budget() {
        let (_root, mut inputs, id) = fixture();
        inputs.buffer = Arc::new(SourceBuffer::from_str("a.rs", &"\n".repeat(100)));
        let index = Arc::make_mut(&mut inputs.index);
        index.source_digests.clear();
        index.definitions.clear();
        index.references.insert(
            id.clone(),
            (0..100)
                .map(|line| {
                    (
                        "a.rs".into(),
                        ScipRange::from_packed(&[line, 0, line + 1, 0]).unwrap(),
                    )
                })
                .collect(),
        );
        let result = query(
            &inputs,
            &id,
            ReferenceLimits {
                result_bytes: 1024,
                ..Default::default()
            },
            &AtomicBool::new(false),
        )
        .unwrap();
        assert!(!result.hits.is_empty());
        assert!(result
            .hits
            .iter()
            .all(|hit| hit.source_line.text.is_empty()));
        assert!(
            result.omitted > 0,
            "empty display text bypassed the retained-row budget"
        );
        assert_eq!(result.hits.len() + result.omitted, 100);
    }
}
