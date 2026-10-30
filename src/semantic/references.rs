//! Replaceable reference context, independent of caret and navigation ownership.

use super::caller_tree::{caller_pill_slot, ReferenceCallerNode};
use super::index::{SemanticIndex, SemanticSymbolID};
use crate::background::LatestRead;
use crate::source::preview::{
    PreviewKind, PreviewPhase, PreviewTarget, SourcePreviewLoader, SourceQueryInputs,
};
use crate::source::{ReviewRevision, SourceBuffer};
use crate::syntax::outline::DocumentSymbol;
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicBool, Ordering};
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
        // A name on the same line is insufficient: require an exact definition
        // at the parsed name's first byte, in the document's declared encoding.
        let occurrence = index
            .documents
            .get(&buffer.path)?
            .occurrences
            .iter()
            .find(|occ| {
                occ.is_definition()
                    && occ.range.as_ref().is_some_and(|range| {
                        range.start_line == range.end_line
                            && index.byte_range(buffer, range).is_some_and(|bytes| {
                                bytes.start == declaration.selection_range.start
                                    && !bytes.is_empty()
                            })
                    })
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
    index: Option<Arc<SemanticIndex>>,
    source_digests: HashMap<String, String>,
    target: Option<ReferenceTarget>,
    job: LatestRead<Vec<ReferenceCallerNode>>,
    pub rows: Vec<ReferenceCallerNode>,
    pub error: Option<String>,
    pub selected: Option<usize>,
    pub previews: BTreeMap<usize, SourcePreviewLoader>,
    preview_inputs: Option<Arc<SourceQueryInputs>>,
}

impl ReferenceContext {
    pub fn explicit(&self) -> Option<&ReferenceTarget> {
        self.explicit.as_ref()
    }
    pub fn target(&self) -> Option<&ReferenceTarget> {
        self.target.as_ref()
    }
    pub fn is_running(&self) -> bool {
        self.job.is_running() || self.previews.values().any(SourcePreviewLoader::is_running)
    }
    pub fn available(&self) -> bool {
        self.index.is_some()
    }

    pub fn show(&mut self, target: ReferenceTarget) {
        self.explicit = Some(target);
        self.refresh();
    }

    pub fn follow_caret(&mut self) {
        self.explicit = None;
        self.refresh();
    }

    pub fn update(
        &mut self,
        index: Option<Arc<SemanticIndex>>,
        buffer: Option<&SourceBuffer>,
        automatic: Option<ReferenceTarget>,
    ) {
        let source_changed = buffer.is_some_and(|buffer| {
            self.source_digests
                .insert(buffer.position_key(), buffer.content_digest().to_owned())
                .is_some_and(|previous| previous != buffer.content_digest())
        });
        let changed = source_changed
            || match (&self.index, &index) {
                (Some(old), Some(new)) => !Arc::ptr_eq(old, new),
                (None, None) => false,
                _ => true,
            };
        self.automatic = automatic;
        self.index = index;
        if changed {
            self.target = None;
            self.job.stop();
            self.rows.clear();
            self.selected = None;
            self.previews.clear();
            self.error = None;
        }
        self.refresh();
    }

    fn refresh(&mut self) {
        let target = self
            .index
            .as_ref()
            .and(self.explicit.as_ref().or(self.automatic.as_ref()));
        if self.target.as_ref() == target {
            return;
        }
        self.job.stop();
        self.rows.clear();
        self.selected = None;
        self.previews.clear();
        self.error = None;
        self.target = target.cloned();
        if let (Some(index), Some(target)) = (&self.index, &self.target) {
            let index = index.clone();
            let id = target.id.clone();
            self.job
                .spawn(move |cancel| direct_references(&index, &id, cancel));
        }
    }

    pub fn configure_previews(&mut self, inputs: Option<Arc<SourceQueryInputs>>) {
        self.preview_inputs = inputs;
        // Loaded excerpts survive navigation; pending requests must capture the
        // newly displayed bytes before a late disk read can publish over them.
        for (&row, preview) in &mut self.previews {
            if preview.is_running() {
                if let Some(node) = self.rows.get(row) {
                    preview.request(
                        self.preview_inputs.clone(),
                        Some(PreviewTarget {
                            path: node.path.clone(),
                            range: node.range.clone(),
                            kind: PreviewKind::Reference,
                        }),
                    );
                }
            }
        }
    }

    pub fn select(&mut self, row: usize) {
        if row < self.rows.len() {
            self.selected = Some(row);
        }
    }

    pub fn move_selection(&mut self, forward: bool) {
        if self.rows.is_empty() {
            return;
        }
        self.selected = Some(match self.selected {
            None if forward => 0,
            None => self.rows.len() - 1,
            Some(row) if forward => row.saturating_add(1).min(self.rows.len() - 1),
            Some(row) => row.saturating_sub(1),
        });
    }

    pub fn expand(&mut self, row: usize, expanded: bool) {
        let Some(node) = self.rows.get(row) else {
            return;
        };
        self.selected = Some(row);
        if !expanded {
            self.previews.remove(&row);
            return;
        }
        let loader = self.previews.entry(row).or_default();
        loader.request(
            self.preview_inputs.clone(),
            Some(PreviewTarget {
                path: node.path.clone(),
                range: node.range.clone(),
                kind: PreviewKind::Reference,
            }),
        );
    }

    pub fn poll(&mut self) -> bool {
        let mut changed = match self.job.poll() {
            Ok(None) => false,
            Ok(Some(rows)) => {
                self.rows = rows;
                true
            }
            Err(_) => {
                self.error = Some("Reference loading stopped unexpectedly.".into());
                true
            }
        };
        for preview in self.previews.values_mut() {
            let published = preview.poll();
            if published {
                if let PreviewPhase::Loaded(source) = preview.phase() {
                    let key = crate::source::SourceLocation::format_position_key(
                        &source.path,
                        source.revision,
                    );
                    self.source_digests
                        .insert(key, source.content_digest.clone());
                }
            }
            changed |= published;
        }
        changed
    }
}

fn direct_references(
    index: &SemanticIndex,
    id: &SemanticSymbolID,
    cancel: &AtomicBool,
) -> Vec<ReferenceCallerNode> {
    let mut rows = Vec::new();
    for (path, range) in index.references.get(id).into_iter().flatten() {
        if cancel.load(Ordering::Relaxed) {
            return Vec::new();
        }
        let start = (range.start_line, range.start_col);
        let end = (range.end_line, range.end_col);
        if range.start_line < 0
            || range.start_col < 0
            || range.end_col < 0
            || start >= end
            || index.column_encoding(path).is_none()
            || !crate::source::SourceLocation::is_repository_path(path)
        {
            continue;
        }
        // Indexed lexical extents can provide a label. They do not prove this
        // reference is a call; parsed/validated call-tree expansion is separate.
        let name = index
            .enclosing_definition(path, range)
            .map(|id| index.display_name(&id))
            .unwrap_or_else(|| path.clone());
        rows.push(ReferenceCallerNode {
            color_slot: caller_pill_slot(&name, path),
            name,
            path: path.clone(),
            line: range.start_line as usize + 1,
            range: range.clone(),
            depth: 0,
            is_recursive: false,
            children: Vec::new(),
        });
    }
    rows.sort_by(|a, b| {
        (
            &a.path,
            a.range.start_line,
            a.range.start_col,
            a.range.end_line,
            a.range.end_col,
        )
            .cmp(&(
                &b.path,
                b.range.start_line,
                b.range.start_col,
                b.range.end_line,
                b.range.end_col,
            ))
    });
    rows.dedup_by(|a, b| a.path == b.path && a.range == b.range);
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{ScipDocument, ScipIndex, ScipOccurrence, ScipRange};
    use crate::syntax::outline::SymbolExtractor;

    fn occurrence(symbol: &str, role: i32, columns: &[i32]) -> ScipOccurrence {
        ScipOccurrence {
            symbol: symbol.into(),
            symbol_roles: role,
            range: ScipRange::from_packed(columns),
            ..Default::default()
        }
    }

    fn fixture() -> (SourceBuffer, Arc<SemanticIndex>) {
        let source = "fn one() {} fn two() { one(); }";
        let buffer = SourceBuffer::from_bytes("a.rs", source.as_bytes().to_vec(), None).unwrap();
        let index = SemanticIndex::from_scip(ScipIndex {
            documents: vec![
                ScipDocument {
                    relative_path: "a.rs".into(),
                    occurrences: vec![
                        occurrence("local 1", 1, &[0, 3, 6]),
                        occurrence("local 2", 1, &[0, 15, 18]),
                        occurrence("local 1", 0, &[0, 23, 26]),
                    ],
                    ..Default::default()
                },
                ScipDocument {
                    relative_path: "b.rs".into(),
                    occurrences: vec![
                        occurrence("local 1", 1, &[0, 3, 6]),
                        occurrence("local 1", 0, &[0, 9, 12]),
                    ],
                    ..Default::default()
                },
            ],
            ..Default::default()
        });
        (buffer, Arc::new(index))
    }

    fn settle(context: &mut ReferenceContext) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while context.is_running() {
            context.poll();
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(context.error.is_none());
    }

    #[test]
    fn declaration_identity_uses_exact_name_not_same_line_and_checks_source() {
        let (mut buffer, index) = fixture();
        let symbols = SymbolExtractor::extract(&buffer);
        let first = ReferenceTarget::declaration(&index, &buffer, &symbols[0]).unwrap();
        let second = ReferenceTarget::declaration(&index, &buffer, &symbols[1]).unwrap();
        assert_eq!(first.id, SemanticSymbolID::new("local 1", "a.rs"));
        assert_eq!(second.id, SemanticSymbolID::new("local 2", "a.rs"));
        let mut wrong = symbols[1].clone();
        wrong.selection_range = 14..18;
        assert!(ReferenceTarget::declaration(&index, &buffer, &wrong).is_none());
        let mut stale = (*index).clone();
        stale
            .source_digests
            .insert("a.rs".into(), "other bytes".into());
        assert!(ReferenceTarget::at_byte(&stale, &buffer, 24).is_none());
        buffer.revision = Some(ReviewRevision::Before);
        assert!(ReferenceTarget::at_byte(&index, &buffer, 24).is_none());
        assert!(ReferenceTarget::declaration(&index, &buffer, &symbols[0]).is_none());
    }

    #[test]
    fn sticky_focus_follows_caret_only_when_released_and_locals_remain_scoped() {
        let (buffer, index) = fixture();
        let one = ReferenceTarget::at_byte(&index, &buffer, 4).unwrap();
        let two = ReferenceTarget::at_byte(&index, &buffer, 16).unwrap();
        let mut context = ReferenceContext::default();
        context.update(Some(index.clone()), Some(&buffer), Some(two.clone()));
        context.show(one.clone());
        settle(&mut context);
        assert_eq!(context.rows.len(), 1);
        assert_eq!(context.rows[0].path, "a.rs");
        assert_eq!(context.rows[0].range.start_col, 23);
        context.update(Some(index.clone()), Some(&buffer), Some(two.clone()));
        assert!(
            !context.is_running(),
            "caret movement rebuilt the same query"
        );
        assert_eq!(context.target(), Some(&one));
        let other = SourceBuffer::from_bytes("b.rs", b"fn one(){one();}".to_vec(), None).unwrap();
        context.update(Some(index), Some(&other), None);
        settle(&mut context);
        assert_eq!(context.target(), Some(&one));
        assert_eq!(context.rows[0].path, "a.rs");
        context.update(context.index.clone(), Some(&buffer), Some(two.clone()));
        context.follow_caret();
        settle(&mut context);
        assert_eq!(context.target(), Some(&two));
        assert!(context.rows.is_empty());
        assert!(context.explicit().is_none());
    }

    #[test]
    fn replacement_index_and_retired_requests_cannot_publish_old_rows() {
        let (buffer, index) = fixture();
        let target = ReferenceTarget::at_byte(&index, &buffer, 4).unwrap();
        let mut context = ReferenceContext::default();
        context.update(Some(index.clone()), Some(&buffer), None);
        context.show(target.clone());
        let mut next = (*index).clone();
        next.references.clear();
        context.update(Some(Arc::new(next)), Some(&buffer), None);
        settle(&mut context);
        assert!(context.rows.is_empty());
        assert_eq!(context.explicit(), Some(&target));
        context.update(None, Some(&buffer), None);
        assert!(!context.available());
        assert!(context.target().is_none());
        assert_eq!(context.explicit(), Some(&target));
        context.update(Some(index), Some(&buffer), None);
        settle(&mut context);
        assert_eq!(context.rows.len(), 1);
    }

    #[test]
    fn utf16_targets_and_ranges_preserve_exact_coordinates_and_full_extent_labels() {
        let buffer = SourceBuffer::from_bytes(
            "a.rs",
            "// 😀\nfn café(){ café(); }".as_bytes().to_vec(),
            None,
        )
        .unwrap();
        let mut definition = occurrence("global", 1, &[1, 3, 7]);
        definition.enclosing_range = ScipRange::from_packed(&[1, 0, 20]);
        let index = SemanticIndex::from_scip(ScipIndex {
            documents: vec![ScipDocument {
                relative_path: "a.rs".into(),
                position_encoding: 2,
                occurrences: vec![definition, occurrence("global", 0, &[1, 11, 15])],
                ..Default::default()
            }],
            ..Default::default()
        });
        let byte = buffer.text.find("café").unwrap();
        let target = ReferenceTarget::at_byte(&index, &buffer, byte).unwrap();
        let rows = direct_references(&index, &target.id, &AtomicBool::new(false));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, index.display_name(&target.id));
        assert_eq!(rows[0].range.start_col, 11);
        assert_eq!(rows[0].range.end_col, 15);
        assert_eq!(
            index.byte_range(&buffer, &rows[0].range).unwrap().len(),
            "café".len()
        );
        assert!(direct_references(&index, &target.id, &AtomicBool::new(true)).is_empty());
    }

    #[test]
    fn indexed_owner_labels_distinguish_multiple_declarations_on_one_line() {
        let (_, index) = fixture();
        let mut index = (*index).clone();
        let occurrences = &mut index.documents.get_mut("a.rs").unwrap().occurrences;
        occurrences[0].enclosing_range = ScipRange::from_packed(&[0, 0, 11]);
        occurrences[1].enclosing_range = ScipRange::from_packed(&[0, 12, 30]);
        let id = SemanticSymbolID::new("local 1", "a.rs");
        let rows = direct_references(&index, &id, &AtomicBool::new(false));
        let owner = SemanticSymbolID::new("local 2", "a.rs");
        assert_eq!(rows[0].name, index.display_name(&owner));
        assert_eq!(
            index.enclosing_definition("a.rs", &rows[0].range),
            Some(owner)
        );
    }

    #[test]
    fn direct_reference_rows_are_sorted_deduplicated_and_never_fake_call_trees() {
        let (_, index) = fixture();
        let mut index = (*index).clone();
        index.documents.get_mut("a.rs").unwrap().occurrences[0].enclosing_range = Some(ScipRange {
            start_line: i32::MIN,
            start_col: 0,
            end_line: i32::MAX,
            end_col: i32::MAX,
        });
        let id = SemanticSymbolID::new("global", "");
        let range = ScipRange::from_packed(&[0, 9, 12]).unwrap();
        index.references.insert(
            id.clone(),
            vec![
                ("b.rs".into(), range.clone()),
                ("a.rs".into(), range.clone()),
                ("a.rs".into(), range),
                ("../escape".into(), ScipRange::default()),
                (
                    "a.rs".into(),
                    ScipRange {
                        start_col: -1,
                        ..Default::default()
                    },
                ),
            ],
        );
        let rows = direct_references(&index, &id, &AtomicBool::new(false));
        assert_eq!(
            rows.iter().map(|row| row.path.as_str()).collect::<Vec<_>>(),
            ["a.rs", "b.rs"]
        );
        assert_eq!(
            rows[0].name, "a.rs",
            "invalid enclosing extents cannot label references"
        );
        assert!(rows
            .iter()
            .all(|row| row.children.is_empty() && !row.is_recursive));
    }
    #[test]
    fn pending_preview_rebinds_to_newly_displayed_source_before_publication() {
        let (buffer, index) = fixture();
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("b.rs"), "fn one(){one();}").unwrap();
        let mut context = ReferenceContext::default();
        context.update(Some(index.clone()), Some(&buffer), None);
        context.show(ReferenceTarget {
            id: SemanticSymbolID::new("local 1", "b.rs"),
            name: "one".into(),
        });
        settle(&mut context);
        context.configure_previews(Some(Arc::new(SourceQueryInputs {
            source: crate::source::loader::SourceRead::Checkout(directory.path().into()),
            buffer: Arc::new(buffer),
            index: index.clone(),
        })));
        context.expand(0, true);
        let displayed = Arc::new(SourceBuffer::from_str("b.rs", "fn one(){new();}"));
        context.update(Some(index.clone()), Some(&displayed), None);
        context.configure_previews(Some(Arc::new(SourceQueryInputs {
            source: crate::source::loader::SourceRead::Buffer(displayed.clone()),
            buffer: displayed.clone(),
            index,
        })));
        settle(&mut context);
        let PreviewPhase::Loaded(preview) = context.previews[&0].phase() else {
            panic!("preview did not load");
        };
        assert_eq!(preview.content_digest, displayed.content_digest());
        assert_eq!(preview.lines[0].text, "fn one(){new();}");
    }
}
