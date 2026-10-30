//! Replaceable reference context, independent of caret and navigation ownership.

use super::caller_tree::{caller_pill_slot, ReferenceCallerNode};
use super::index::{SemanticIndex, SemanticSymbolID};
use crate::background::LatestRead;
use crate::source::{ReviewRevision, SourceBuffer};
use crate::syntax::outline::DocumentSymbol;
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
    source: Option<(String, String)>,
    target: Option<ReferenceTarget>,
    job: LatestRead<Vec<ReferenceCallerNode>>,
    pub rows: Vec<ReferenceCallerNode>,
    pub error: Option<String>,
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
        let source =
            buffer.map(|buffer| (buffer.position_key(), buffer.content_digest().to_owned()));
        let changed = self.source != source
            || match (&self.index, &index) {
                (Some(old), Some(new)) => !Arc::ptr_eq(old, new),
                (None, None) => false,
                _ => true,
            };
        self.automatic = automatic;
        self.index = index;
        self.source = source;
        if changed {
            self.target = None;
            self.job.stop();
            self.rows.clear();
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
        self.error = None;
        self.target = target.cloned();
        if let (Some(index), Some(target)) = (&self.index, &self.target) {
            let index = index.clone();
            let id = target.id.clone();
            self.job
                .spawn(move |cancel| direct_references(&index, &id, cancel));
        }
    }

    pub fn poll(&mut self) -> bool {
        match self.job.poll() {
            Ok(None) => false,
            Ok(Some(rows)) => {
                self.rows = rows;
                true
            }
            Err(_) => {
                self.error = Some("Reference loading stopped unexpectedly.".into());
                true
            }
        }
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
            .documents
            .get(path)
            .and_then(|doc| {
                doc.occurrences
                    .iter()
                    .filter(|occ| occ.is_definition())
                    .filter_map(|occ| {
                        let extent = occ.enclosing_range.as_ref()?;
                        let lower = (extent.start_line, extent.start_col);
                        let upper = (extent.end_line, extent.end_col);
                        (lower <= start && end <= upper).then_some((lower, upper, occ))
                    })
                    .min_by_key(|(lower, upper, _)| {
                        (upper.0 - lower.0, i64::from(upper.1) - i64::from(lower.1))
                    })
                    .map(|(_, _, occ)| {
                        index.display_name(&SemanticSymbolID::new(&occ.symbol, path))
                    })
            })
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
    fn direct_reference_rows_are_sorted_deduplicated_and_never_fake_call_trees() {
        let (_, index) = fixture();
        let mut index = (*index).clone();
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
        assert!(rows
            .iter()
            .all(|row| row.children.is_empty() && !row.is_recursive));
    }
}
