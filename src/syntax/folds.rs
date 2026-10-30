//! Code folding detection and fold state management.

use crate::source::buffer::SourceBuffer;
use std::collections::HashSet;
use std::ops::Range;

/// Parsed fold roles in the same precedence order as Swift's fold queries.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum FoldKind {
    /// Function, method or closure body.
    Callable,
    /// Type, protocol or declaration-container body.
    Type,
    /// Parsed multiline comment/string region.
    Comment,
    /// Grouped imports.
    Imports,
    /// Other structural block or collection.
    #[default]
    Block,
}

/// Represents a collapsible vertical region within a source document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FoldRange {
    /// 1-based start line of the fold header.
    pub start_line: usize,
    /// 1-based ending line of the foldable block.
    pub end_line: usize,
    /// Header line text displayed when collapsed.
    pub header_preview: String,
    /// Parsed fold role, independent of source spelling.
    pub kind: FoldKind,
    /// Exact original source bytes, excluding unrelated declaration headers.
    pub byte_range: Range<usize>,
    /// Whether this body contains a complete captured nested declaration.
    pub contains_declarations: bool,
    /// Callable declaration/annotation ownership used by initial PR visibility.
    pub owner_range: Option<Range<usize>>,
}

/// Stores the set of active collapsed fold lines.
///
/// # Examples
///
/// ```rust
/// use radiology::syntax::folds::{FoldRange, FoldStore};
///
/// let mut store = FoldStore::new();
/// let fold = FoldRange {
///     start_line: 10,
///     end_line: 25,
///     header_preview: "fn compute() {".to_string(),
///     ..Default::default()
/// };
/// store.toggle(10);
/// assert!(store.is_line_hidden(15, &[fold]));
/// ```
#[derive(Debug, Clone, Default)]
pub struct FoldStore {
    /// 1-based line numbers where folds are currently active.
    pub folded_lines: HashSet<usize>,
    initialized: bool,
    documents: crate::source::document_memory::DocumentMemory<(bool, HashSet<usize>)>,
}

impl FoldStore {
    /// Creates an empty [`FoldStore`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Restores choices only for identical successfully published source/revision.
    ///
    /// The host must reset this store at repository/review-snapshot boundaries.
    /// Only current fold headers survive; explicit navigation may then reveal its
    /// destination without expanding unrelated restored regions.
    pub fn activate(&mut self, buffer: &SourceBuffer, ranges: &[FoldRange]) {
        let mut state = (self.initialized, std::mem::take(&mut self.folded_lines));
        self.documents.activate(buffer, &mut state);
        self.initialized = state.0;
        self.folded_lines = state.1;
        let headers: HashSet<_> = ranges.iter().map(|range| range.start_line).collect();
        self.folded_lines.retain(|line| headers.contains(line));
    }

    /// Toggles the folded state for the given start line number.
    pub fn toggle(&mut self, start_line: usize) {
        self.initialized = true;
        if self.folded_lines.contains(&start_line) {
            self.folded_lines.remove(&start_line);
        } else {
            self.folded_lines.insert(start_line);
        }
    }

    /// Returns `true` if the fold starting at `start_line` is collapsed.
    #[must_use]
    pub fn is_folded(&self, start_line: usize) -> bool {
        self.folded_lines.contains(&start_line)
    }

    /// Returns `true` if `line` is hidden because an enclosing fold is collapsed.
    #[must_use]
    pub fn is_line_hidden(&self, line: usize, fold_ranges: &[FoldRange]) -> bool {
        for fold in fold_ranges {
            if self.is_folded(fold.start_line) && line > fold.start_line && line <= fold.end_line {
                return true;
            }
        }
        false
    }

    /// Collapses parsed callable bodies, preserving other current fold choices.
    pub fn collapse_all_functions(&mut self, ranges: &[FoldRange]) {
        self.initialized = true;
        self.folded_lines.extend(
            ranges
                .iter()
                .filter(|fold| fold.kind == FoldKind::Callable)
                .map(|fold| fold.start_line),
        );
    }

    /// Replaces folds with callable bodies and types that hide no nested declarations.
    /// Containers exposing methods, variants or other declarations remain open.
    pub fn fold_to_definitions(&mut self, ranges: &[FoldRange]) {
        self.initialized = true;
        self.folded_lines = ranges
            .iter()
            .filter(|fold| {
                fold.kind == FoldKind::Callable
                    || (fold.kind == FoldKind::Type && !fold.contains_declarations)
            })
            .map(|fold| fold.start_line)
            .collect();
    }

    /// Applies callable-body defaults once per exact document, before navigation reveal.
    /// PR changes must belong to the same captured source/revision; restored/manual
    /// choices, including an empty expanded set, always take precedence.
    pub fn apply_initial_defaults(
        &mut self,
        buffer: &SourceBuffer,
        ranges: &[FoldRange],
        changes: Option<&crate::pull_request::diff::FileLineChanges>,
    ) {
        if self.initialized {
            return;
        }
        self.collapse_all_functions(ranges);
        if let (Some(changes), Some(revision)) = (changes, buffer.revision) {
            let expanded = super::review_folds::expanded_headers(buffer, ranges, changes, revision);
            self.folded_lines.retain(|line| !expanded.contains(line));
        }
    }

    /// Adopts defaults prepared with the successfully published source bundle.
    /// Restored or already edited documents retain their current choices.
    pub fn apply_prepared_defaults(&mut self, defaults: Option<HashSet<usize>>) {
        if !self.initialized {
            if let Some(defaults) = defaults {
                self.folded_lines = defaults;
                self.initialized = true;
            }
        }
    }

    /// Unfolds all active folds.
    pub fn unfold_all(&mut self) {
        self.initialized = true;
        self.folded_lines.clear();
    }

    /// Reveals a canonical source line without expanding unrelated folds.
    pub fn reveal_line(&mut self, line: usize, ranges: &[FoldRange]) {
        for fold in ranges {
            if line > fold.start_line && line <= fold.end_line {
                self.folded_lines.remove(&fold.start_line);
            }
        }
    }
}

/// Detects typed structural folds from the source's existing shared parse.
pub struct FoldDetector;

impl FoldDetector {
    /// Detects parsed multiline folds without text-based comment guessing.
    #[must_use]
    pub fn detect_folds(buffer: &SourceBuffer) -> Vec<FoldRange> {
        super::DocumentSyntax::analyze(buffer).folds
    }

    pub(crate) fn from_tree(
        buffer: &SourceBuffer,
        grammar: super::SupportedGrammar,
        root: tree_sitter::Node<'_>,
    ) -> Result<Vec<FoldRange>, String> {
        let query = super::queries::query(grammar, super::queries::QueryRole::Folds)?;
        super::fold_queries::extract(buffer, root, query, 200_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folds() {
        let code = r#"
fn long_func() {
    let a = 1;
    let b = 2;
    println!("hello");
}

fn single_line() {}
"#;
        let buf = SourceBuffer::from_str("test.rs", code);
        let folds = FoldDetector::detect_folds(&buf);

        assert!(!folds.is_empty());
        let fn_fold = &folds[0];
        assert_eq!(fn_fold.start_line, 2);
        assert_eq!(fn_fold.end_line, 6);

        let mut store = FoldStore::new();
        assert!(!store.is_line_hidden(3, &folds));

        store.toggle(fn_fold.start_line);
        assert!(store.is_line_hidden(3, &folds));
        assert!(store.is_line_hidden(6, &folds));
        assert!(!store.is_line_hidden(2, &folds));
        assert!(!store.is_line_hidden(8, &folds));
    }

    #[test]
    fn choices_restore_only_for_exact_document_revision_and_valid_fold_headers() {
        use crate::source::ReviewRevision;
        let a = SourceBuffer::from_str(
            "a.rs",
            "fn alpha() {\n one();\n}\nfn beta() {\n two();\n}\n",
        );
        let b = SourceBuffer::from_str("b.rs", "fn gamma() {\n three();\n}\n");
        let ranges = FoldDetector::detect_folds(&a);
        let other_ranges = FoldDetector::detect_folds(&b);
        let mut store = FoldStore::new();
        store.activate(&a, &ranges);
        store.toggle(4);
        store.toggle(999);
        store.activate(&a, &ranges);
        assert_eq!(store.folded_lines, HashSet::from([4]));
        store.activate(&b, &other_ranges);
        assert!(store.folded_lines.is_empty());
        store.toggle(1);
        for revision in [ReviewRevision::After, ReviewRevision::Before] {
            let snapshot =
                SourceBuffer::from_bytes("a.rs", a.data.clone(), Some(revision)).unwrap();
            store.activate(&snapshot, &ranges);
            assert!(store.folded_lines.is_empty());
            store.toggle(1);
        }
        store.activate(&a, &ranges);
        assert_eq!(store.folded_lines, HashSet::from([4]));
        store.unfold_all();
        store.activate(&b, &other_ranges);
        assert_eq!(store.folded_lines, HashSet::from([1]));
        store.activate(&a, &ranges);
        assert!(
            store.folded_lines.is_empty(),
            "expanded choices must not resurrect old folds"
        );
        store.toggle(4);
        let changed = SourceBuffer::from_str("a.rs", &a.text.replace("beta", "zeta"));
        store.activate(&changed, &ranges);
        assert!(store.folded_lines.is_empty());
        store.activate(&a, &ranges);
        assert!(
            store.folded_lines.is_empty(),
            "old-byte choices were invalidated"
        );
    }
}
