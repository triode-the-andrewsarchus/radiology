//! Replaceable reference context, independent of caret and navigation ownership.

use super::disclosure::ReferenceDisclosure;
use super::index::{SemanticIndex, SemanticSymbolID};
use super::reference_results::{ReferenceLimits, ReferenceResults};
use crate::background::LatestRead;
use crate::source::preview::{PreviewPhase, SourceQueryInputs};
use crate::source::{ReviewRevision, SourceBuffer};
use crate::syntax::outline::DocumentSymbol;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReferenceTarget {
    pub id: SemanticSymbolID,
    pub name: String,
}

impl ReferenceTarget {
    pub fn at_byte(index: &SemanticIndex, buffer: &SourceBuffer, byte: usize) -> Option<Self> {
        if buffer.revision == Some(ReviewRevision::Before) {
            return None;
        }
        let occurrence = index.occurrence_at_byte(buffer, byte)?;
        let id = SemanticSymbolID::new(&occurrence.symbol, &buffer.path);
        Some(Self {
            name: index.display_name(&id),
            id,
        })
    }

    pub fn declaration(
        index: &SemanticIndex,
        buffer: &SourceBuffer,
        declaration: &DocumentSymbol,
    ) -> Option<Self> {
        if buffer.revision == Some(ReviewRevision::Before) {
            return None;
        }
        // Bind only the parsed name's first byte in the declared encoding.
        // Definitions take precedence; implementation headers may refer to a type.
        let hits = index
            .documents
            .get(&buffer.path)?
            .occurrences
            .iter()
            .filter(|occ| {
                !occ.symbol.is_empty()
                    && occ.range.as_ref().is_some_and(|range| {
                        range.start_line == range.end_line
                            && index.byte_range(buffer, range).is_some_and(|bytes| {
                                bytes.start == declaration.selection_range.start
                                    && !bytes.is_empty()
                            })
                    })
            });
        let occurrence = hits.clone().find(|occ| occ.is_definition()).or_else(|| {
            (declaration.kind == crate::syntax::SymbolKind::Extension)
                .then(|| {
                    hits.into_iter()
                        .find(|occ| occ.symbol_roles & super::scip_proto::SCIP_ROLE_IMPORT == 0)
                })
                .flatten()
        })?;
        let id = SemanticSymbolID::new(&occurrence.symbol, &buffer.path);
        Some(Self {
            name: index.display_name(&id),
            id,
        })
    }
}

#[derive(Default)]
pub(crate) struct ReferenceContext {
    explicit: Option<ReferenceTarget>,
    automatic: Option<ReferenceTarget>,
    inputs: Option<Arc<SourceQueryInputs>>,
    source_digests: HashMap<String, String>,
    target: Option<ReferenceTarget>,
    job: LatestRead<Result<ReferenceResults, String>>,
    limits: ReferenceLimits,
    pub results: ReferenceResults,
    pub error: Option<String>,
    pub selected: Option<usize>,
    pub disclosures: BTreeMap<usize, ReferenceDisclosure>,
}

impl ReferenceContext {
    pub fn explicit(&self) -> Option<&ReferenceTarget> {
        self.explicit.as_ref()
    }
    pub fn target(&self) -> Option<&ReferenceTarget> {
        self.target.as_ref()
    }
    pub fn is_running(&self) -> bool {
        self.job.is_running()
            || self
                .disclosures
                .values()
                .any(ReferenceDisclosure::is_running)
    }
    pub fn is_query_running(&self) -> bool {
        self.job.is_running()
    }
    pub fn available(&self) -> bool {
        self.inputs.is_some()
    }

    pub fn show(&mut self, target: ReferenceTarget) {
        self.explicit = Some(target);
        self.refresh(false);
    }
    pub fn follow_caret(&mut self) {
        self.explicit = None;
        self.refresh(false);
    }

    pub fn update(
        &mut self,
        inputs: Option<Arc<SourceQueryInputs>>,
        automatic: Option<ReferenceTarget>,
    ) {
        let changed_source = self.observe_source(inputs.as_deref());
        let changed_index = match (&self.inputs, &inputs) {
            (Some(old), Some(new)) => !Arc::ptr_eq(&old.index, &new.index),
            (None, None) => false,
            _ => true,
        };
        let changed_inputs = match (&self.inputs, &inputs) {
            (Some(old), Some(new)) => !Arc::ptr_eq(old, new),
            (None, None) => false,
            _ => true,
        };
        self.inputs = inputs;
        self.automatic = automatic;
        let refreshed = self.refresh(changed_source || changed_index);
        if changed_inputs && !refreshed {
            self.rebind_pending();
        }
    }

    fn observe_source(&mut self, inputs: Option<&SourceQueryInputs>) -> bool {
        let Some(inputs) = inputs else {
            return false;
        };
        let changed = self
            .source_digests
            .insert(
                inputs.buffer.position_key(),
                inputs.buffer.content_digest().to_owned(),
            )
            .is_some_and(|old| old != inputs.buffer.content_digest());
        changed
            || self
                .results
                .unreadable
                .iter()
                .any(|file| file.path == inputs.buffer.path)
    }

    fn refresh(&mut self, invalidated: bool) -> bool {
        let target = self
            .inputs
            .as_ref()
            .and(self.explicit.as_ref().or(self.automatic.as_ref()))
            .cloned();
        if !invalidated && self.target == target {
            return false;
        }
        self.job.stop();
        self.results = Default::default();
        self.selected = None;
        self.disclosures.clear();
        self.error = None;
        self.target = target;
        self.limits = Default::default();
        self.start_query();
        true
    }

    fn start_query(&mut self) {
        self.error = None;
        if let (Some(inputs), Some(target)) = (&self.inputs, &self.target) {
            let (inputs, id, limits) = (inputs.clone(), target.id.clone(), self.limits);
            self.job
                .spawn(move |cancel| super::reference_results::query(&inputs, &id, limits, cancel));
        }
    }

    fn rebind_pending(&mut self) {
        if self.job.is_running() {
            self.start_query();
        }
        if let Some(inputs) = &self.inputs {
            for disclosure in self.disclosures.values_mut() {
                disclosure.rebind_pending(inputs.clone());
            }
        }
    }

    pub fn more(&mut self) {
        if self.results.omitted > 0 && !self.job.is_running() {
            self.limits = self.limits.expanded();
            self.start_query();
        }
    }

    pub fn select(&mut self, row: usize) {
        if row < self.results.hits.len() {
            self.selected = Some(row);
        }
    }
    pub fn move_selection(&mut self, forward: bool) {
        if self.results.hits.is_empty() {
            return;
        }
        self.selected = Some(match self.selected {
            None if forward => 0,
            None => self.results.hits.len() - 1,
            Some(row) if forward => row.saturating_add(1).min(self.results.hits.len() - 1),
            Some(row) => row.saturating_sub(1),
        });
    }

    pub fn expand(&mut self, row: usize, expanded: bool) {
        let Some(hit) = self.results.hits.get(row) else {
            return;
        };
        self.selected = Some(row);
        if !expanded {
            self.disclosures.remove(&row);
            return;
        }
        if let (Some(inputs), Some(digest)) = (&self.inputs, self.results.source_digest(&hit.path))
        {
            self.disclosures.entry(row).or_insert_with(|| {
                ReferenceDisclosure::new(inputs.clone(), hit.clone(), digest.to_owned())
            });
        }
    }

    pub fn more_callers(&mut self, row: usize) {
        if let (Some(inputs), Some(disclosure)) = (&self.inputs, self.disclosures.get_mut(&row)) {
            disclosure.more_callers(inputs.clone());
        }
    }

    pub fn poll(&mut self) -> bool {
        let mut changed = match self.job.poll() {
            Ok(None) => false,
            Ok(Some(Ok(results))) => {
                self.publish(results);
                true
            }
            Ok(Some(Err(error))) => {
                self.error = Some(error);
                true
            }
            Err(_) => {
                self.error = Some("Reference loading stopped unexpectedly.".into());
                true
            }
        };
        let mut source_changed = false;
        for preview in self.disclosures.values_mut() {
            let published = preview.poll();
            if published {
                if let PreviewPhase::Loaded(source) = preview.phase() {
                    source_changed |= self.results.source_digest(&source.path)
                        != Some(source.content_digest.as_str());
                }
            }
            if published {
                if let Some(tree) = preview.callers() {
                    let revision = self
                        .inputs
                        .as_ref()
                        .and_then(|inputs| inputs.buffer.revision);
                    remember_sources(&mut self.source_digests, revision, &tree.source_identities);
                }
            }
            changed |= published;
        }
        if source_changed {
            self.start_query();
        }
        changed
    }

    fn publish(&mut self, results: ReferenceResults) {
        let retained = |old: usize| {
            let hit = self.results.hits.get(old)?;
            (self.results.source_digest(&hit.path) == results.source_digest(&hit.path))
                .then(|| results.find_hit(&hit.path, &hit.range))
                .flatten()
        };
        self.selected = self.selected.and_then(retained);
        self.disclosures = std::mem::take(&mut self.disclosures)
            .into_iter()
            .filter_map(|(old, preview)| retained(old).map(|new| (new, preview)))
            .collect();
        let revision = self
            .inputs
            .as_ref()
            .and_then(|inputs| inputs.buffer.revision);
        remember_sources(
            &mut self.source_digests,
            revision,
            &results.source_identities,
        );
        self.results = results;
    }
}

fn remember_sources(
    ledger: &mut HashMap<String, String>,
    revision: Option<ReviewRevision>,
    sources: &BTreeMap<String, String>,
) {
    ledger.extend(sources.iter().map(|(path, digest)| {
        (
            crate::source::SourceLocation::format_position_key(path, revision),
            digest.clone(),
        )
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{ScipDocument, ScipIndex, ScipOccurrence, ScipRange};
    use crate::source::loader::SourceRead;

    #[test]
    fn implementation_headers_link_only_exact_type_references_and_prefer_definitions() {
        for (path, text) in [
            ("a.rs", "struct Thing;\nimpl Thing {}\n"),
            ("a.swift", "struct Thing {}\nextension Thing {}\n"),
        ] {
            let buffer = SourceBuffer::from_str(path, text);
            let symbols = crate::syntax::SymbolExtractor::extract(&buffer);
            assert_eq!(
                symbols[1].kind,
                crate::syntax::SymbolKind::Extension,
                "{path}: {symbols:?}"
            );
            let first = text.find("Thing").unwrap();
            let last = text.rfind("Thing").unwrap();
            let definitions =
                crate::semantic::test_support::occurrence(&buffer, first..first + 5, "type", true);
            let reference =
                crate::semantic::test_support::occurrence(&buffer, last..last + 5, "type", false);
            let mut index = SemanticIndex::from_scip(ScipIndex {
                documents: vec![ScipDocument {
                    relative_path: path.into(),
                    position_encoding: 2,
                    occurrences: vec![definitions, reference.clone()],
                    ..Default::default()
                }],
                ..Default::default()
            });
            assert_eq!(
                ReferenceTarget::declaration(&index, &buffer, &symbols[1])
                    .unwrap()
                    .id
                    .symbol,
                "type"
            );
            let mut ordinary = symbols[1].clone();
            ordinary.kind = crate::syntax::SymbolKind::Class;
            assert!(ReferenceTarget::declaration(&index, &buffer, &ordinary).is_none());
            let mut alias = reference;
            alias.symbol = "alias".into();
            alias.symbol_roles = 1;
            index
                .documents
                .get_mut(path)
                .unwrap()
                .occurrences
                .push(alias);
            assert_eq!(
                ReferenceTarget::declaration(&index, &buffer, &symbols[1])
                    .unwrap()
                    .id
                    .symbol,
                "alias"
            );
            index.source_digests.insert(path.into(), "changed".into());
            assert!(ReferenceTarget::declaration(&index, &buffer, &symbols[1]).is_none());
        }
    }

    fn fixture() -> (tempfile::TempDir, Arc<SourceQueryInputs>) {
        let root = tempfile::tempdir().unwrap();
        let buffer = Arc::new(SourceBuffer::from_str(
            "a.rs",
            "fn one() {} fn two() { one(); }",
        ));
        std::fs::write(root.path().join("a.rs"), &buffer.text).unwrap();
        std::fs::write(root.path().join("b.rs"), "fn one(){one();}").unwrap();
        let occurrence = |name: &str, role, coordinates: &[i32]| ScipOccurrence {
            symbol: name.into(),
            symbol_roles: role,
            range: ScipRange::from_packed(coordinates),
            ..Default::default()
        };
        let index = Arc::new(SemanticIndex::from_scip(ScipIndex {
            documents: vec![
                ScipDocument {
                    relative_path: "a.rs".into(),
                    occurrences: vec![
                        occurrence("local 1", 1, &[0, 3, 6]),
                        occurrence("local 2", 1, &[0, 15, 18]),
                        occurrence("local 1", 0, &[0, 23, 26]),
                        occurrence("shared", 1, &[0, 3, 6]),
                    ],
                    ..Default::default()
                },
                ScipDocument {
                    relative_path: "b.rs".into(),
                    occurrences: vec![
                        occurrence("local 1", 1, &[0, 3, 6]),
                        occurrence("local 1", 0, &[0, 9, 12]),
                        occurrence("shared", 0, &[0, 9, 12]),
                    ],
                    ..Default::default()
                },
            ],
            ..Default::default()
        }));
        let inputs = Arc::new(SourceQueryInputs {
            source: SourceRead::Checkout(root.path().into()),
            buffer,
            index,
        });
        (root, inputs)
    }

    fn target(inputs: &SourceQueryInputs, byte: usize) -> ReferenceTarget {
        ReferenceTarget::at_byte(&inputs.index, &inputs.buffer, byte).unwrap()
    }
    fn settle(context: &mut ReferenceContext) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while context.is_running() {
            context.poll();
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(context.error.is_none(), "{:?}", context.error);
    }

    #[test]
    fn declaration_identity_requires_the_exact_parsed_name_and_fresh_source() {
        let (_root, inputs) = fixture();
        let symbols = crate::syntax::outline::SymbolExtractor::extract(&inputs.buffer);
        let first =
            ReferenceTarget::declaration(&inputs.index, &inputs.buffer, &symbols[0]).unwrap();
        let second =
            ReferenceTarget::declaration(&inputs.index, &inputs.buffer, &symbols[1]).unwrap();
        assert_eq!(first.id, SemanticSymbolID::new("local 1", "a.rs"));
        assert_eq!(second.id, SemanticSymbolID::new("local 2", "a.rs"));
        let mut wrong = symbols[1].clone();
        wrong.selection_range = 14..18;
        assert!(ReferenceTarget::declaration(&inputs.index, &inputs.buffer, &wrong).is_none());
        let mut stale = (*inputs.index).clone();
        stale.source_digests.insert("a.rs".into(), "other".into());
        assert!(ReferenceTarget::at_byte(&stale, &inputs.buffer, 24).is_none());
        let mut before = (*inputs.buffer).clone();
        before.revision = Some(ReviewRevision::Before);
        assert!(ReferenceTarget::at_byte(&inputs.index, &before, 24).is_none());
    }

    #[test]
    fn explicit_scoped_focus_survives_files_and_follow_caret_restores_latest_declaration() {
        let (_root, inputs) = fixture();
        let (one, two) = (target(&inputs, 4), target(&inputs, 16));
        let mut context = ReferenceContext::default();
        context.update(Some(inputs.clone()), Some(two.clone()));
        context.show(one.clone());
        settle(&mut context);
        assert_eq!(
            (
                context.results.reference_count,
                context.results.definition_count
            ),
            (1, 1)
        );
        context.update(Some(inputs.clone()), Some(two.clone()));
        assert!(!context.is_running());
        let other = inputs
            .source
            .read(&crate::source::SourceLocation::new("b.rs", 0))
            .unwrap();
        context.update(
            Some(Arc::new(SourceQueryInputs {
                source: inputs.source.clone(),
                buffer: other,
                index: inputs.index.clone(),
            })),
            None,
        );
        assert!(!context.is_running());
        assert_eq!(context.target(), Some(&one));
        assert!(context.results.hits.iter().all(|hit| hit.path == "a.rs"));
        context.update(Some(inputs), Some(two.clone()));
        context.follow_caret();
        settle(&mut context);
        assert_eq!(context.target(), Some(&two));
        assert_eq!(context.results.reference_count, 0);
        assert_eq!(context.results.definition_count, 1);
        assert!(context.explicit().is_none());
    }

    #[test]
    fn replacement_indexes_and_unavailable_revisions_retire_old_publications() {
        let (_root, inputs) = fixture();
        let one = target(&inputs, 4);
        let mut context = ReferenceContext::default();
        context.update(Some(inputs.clone()), None);
        context.show(one.clone());
        let mut index = (*inputs.index).clone();
        index.references.clear();
        context.update(
            Some(Arc::new(SourceQueryInputs {
                source: inputs.source.clone(),
                buffer: inputs.buffer.clone(),
                index: Arc::new(index),
            })),
            None,
        );
        settle(&mut context);
        assert_eq!(context.results.reference_count, 0);
        assert_eq!(context.results.definition_count, 1);
        context.update(None, None);
        assert!(!context.available());
        assert!(context.results.hits.is_empty());
        assert_eq!(context.explicit(), Some(&one));
        context.update(Some(inputs), None);
        settle(&mut context);
        assert_eq!(context.results.reference_count, 1);
    }

    #[test]
    fn load_more_retains_valid_selection_and_source_disclosures() {
        let (_root, inputs) = fixture();
        let mut context = ReferenceContext::default();
        context.update(Some(inputs), None);
        context.show(ReferenceTarget {
            id: SemanticSymbolID::new("shared", ""),
            name: "shared".into(),
        });
        context.limits.files = 1;
        context.start_query();
        settle(&mut context);
        assert_eq!(context.results.omitted, 1);
        context.expand(0, true);
        settle(&mut context);
        let preview = context.disclosures[&0].phase().clone();
        let tree = context.disclosures[&0].callers().unwrap().clone();
        context.more();
        settle(&mut context);
        assert_eq!(context.results.omitted, 0);
        assert_eq!(context.results.hits.len(), 2);
        assert_eq!(context.selected, Some(0));
        assert_eq!(context.disclosures[&0].phase(), &preview);
        assert_eq!(context.disclosures[&0].callers(), Some(&tree));
    }

    #[test]
    fn independent_disclosures_cancel_together_and_retire_on_index_replacement() {
        let (_root, inputs) = fixture();
        let mut context = ReferenceContext::default();
        context.update(Some(inputs.clone()), Some(target(&inputs, 4)));
        settle(&mut context);
        context.expand(0, true);
        context.expand(1, true);
        settle(&mut context);
        assert_eq!(context.disclosures.len(), 2);
        assert_eq!(context.disclosures[&0].callers().unwrap().rows.len(), 2);
        let retained = context.disclosures[&1].callers().cloned();
        context.expand(0, false);
        assert_eq!(context.disclosures[&1].callers(), retained.as_ref());
        context.expand(0, true);
        let replacement = Arc::new(SourceQueryInputs {
            source: inputs.source.clone(),
            buffer: inputs.buffer.clone(),
            index: Arc::new((*inputs.index).clone()),
        });
        context.update(Some(replacement), Some(target(&inputs, 4)));
        assert!(context.disclosures.is_empty());
        settle(&mut context);
        assert!(
            context.disclosures.is_empty(),
            "a retired caller job published after replacement"
        );
    }

    #[test]
    fn caller_only_source_reads_join_first_visit_invalidation() {
        let (root, inputs) = fixture();
        let mut documents = inputs.index.documents.values().cloned().collect::<Vec<_>>();
        documents
            .iter_mut()
            .find(|doc| doc.relative_path == "a.rs")
            .unwrap()
            .occurrences[1]
            .symbol = "two".into();
        let text = "fn three() { two(); }";
        std::fs::write(root.path().join("c.rs"), text).unwrap();
        documents.push(ScipDocument {
            relative_path: "c.rs".into(),
            occurrences: vec![
                ScipOccurrence {
                    symbol: "three".into(),
                    symbol_roles: 1,
                    range: ScipRange::from_packed(&[0, 3, 8]),
                    ..Default::default()
                },
                ScipOccurrence {
                    symbol: "two".into(),
                    range: ScipRange::from_packed(&[0, 13, 16]),
                    ..Default::default()
                },
            ],
            ..Default::default()
        });
        let inputs = Arc::new(SourceQueryInputs {
            source: inputs.source.clone(),
            buffer: inputs.buffer.clone(),
            index: Arc::new(SemanticIndex::from_scip(ScipIndex {
                documents,
                ..Default::default()
            })),
        });
        let mut context = ReferenceContext::default();
        context.update(Some(inputs.clone()), Some(target(&inputs, 4)));
        settle(&mut context);
        context.expand(0, true);
        settle(&mut context);
        assert!(!context.results.source_identities.contains_key("c.rs"));
        assert!(context.disclosures[&0]
            .callers()
            .unwrap()
            .source_identities
            .contains_key("c.rs"));
        let displayed = Arc::new(SourceBuffer::from_str("c.rs", "fn three() { new(); }"));
        context.update(
            Some(Arc::new(SourceQueryInputs {
                source: inputs.source.clone(),
                buffer: displayed,
                index: inputs.index.clone(),
            })),
            None,
        );
        assert!(
            context.disclosures.is_empty(),
            "changed caller-only source retained an old tree"
        );
    }

    #[test]
    fn pending_queries_rebind_before_a_newly_displayed_source_can_be_overwritten() {
        let (_root, inputs) = fixture();
        let mut context = ReferenceContext::default();
        context.update(Some(inputs.clone()), None);
        context.show(ReferenceTarget {
            id: SemanticSymbolID::new("local 1", "b.rs"),
            name: "one".into(),
        });
        let displayed = Arc::new(SourceBuffer::from_str("b.rs", "fn one(){new();}"));
        context.update(
            Some(Arc::new(SourceQueryInputs {
                source: inputs.source.clone(),
                buffer: displayed.clone(),
                index: inputs.index.clone(),
            })),
            None,
        );
        settle(&mut context);
        assert_eq!(
            context.results.source_digest("b.rs"),
            Some(displayed.content_digest())
        );
        assert!(context
            .results
            .hits
            .iter()
            .all(|hit| hit.source_line.text.contains("new();")));
    }

    #[test]
    fn same_line_indexed_owner_lookup_uses_columns_and_rejects_negative_extents() {
        let (_root, inputs) = fixture();
        let mut index = (*inputs.index).clone();
        let definitions = &mut index.documents.get_mut("a.rs").unwrap().occurrences;
        definitions[0].enclosing_range = ScipRange::from_packed(&[0, 0, 11]);
        definitions[1].enclosing_range = ScipRange::from_packed(&[0, 12, 30]);
        let range = ScipRange::from_packed(&[0, 23, 26]).unwrap();
        assert_eq!(
            index.enclosing_definition("a.rs", &range),
            Some(SemanticSymbolID::new("local 2", "a.rs"))
        );
        index.documents.get_mut("a.rs").unwrap().occurrences[1].enclosing_range = Some(ScipRange {
            start_line: i32::MIN,
            start_col: 0,
            end_line: i32::MAX,
            end_col: i32::MAX,
        });
        assert!(index.enclosing_definition("a.rs", &range).is_none());
    }
    #[test]
    fn opening_a_previously_unreadable_file_refreshes_coverage_once() {
        let (root, inputs) = fixture();
        std::fs::remove_file(root.path().join("b.rs")).unwrap();
        let mut context = ReferenceContext::default();
        context.update(Some(inputs.clone()), None);
        context.show(ReferenceTarget {
            id: SemanticSymbolID::new("shared", ""),
            name: "shared".into(),
        });
        settle(&mut context);
        assert_eq!(context.results.unreadable.len(), 1);
        let displayed = Arc::new(SourceBuffer::from_str("b.rs", "fn one(){one();}"));
        let current = Arc::new(SourceQueryInputs {
            source: inputs.source.clone(),
            buffer: displayed,
            index: inputs.index.clone(),
        });
        context.update(Some(current.clone()), None);
        settle(&mut context);
        assert!(context.results.unreadable.is_empty());
        assert_eq!(context.results.reference_count, 1);
        context.update(Some(current), None);
        assert!(!context.is_running());
    }
}
