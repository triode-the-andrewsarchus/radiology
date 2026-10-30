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

impl FoldRange {
    /// Returns whether the capture retains visible opening and closing delimiters.
    #[must_use]
    pub fn is_delimited(&self, buffer: &SourceBuffer) -> bool {
        let range = &self.byte_range;
        range.len() >= 2
            && range.end <= buffer.data.len()
            && b"{[(\"'`".contains(&buffer.data[range.start])
            && b"}])\"'`".contains(&buffer.data[range.end - 1])
    }

    /// Returns the canonical bytes hidden by this fold, preserving delimiters.
    #[must_use]
    pub fn hidden_range(&self, buffer: &SourceBuffer) -> Range<usize> {
        if self.is_delimited(buffer) {
            self.byte_range.start + 1..self.byte_range.end - 1
        } else {
            self.byte_range.clone()
        }
    }

    /// Returns the original physical line owning the disclosure.
    #[must_use]
    pub fn header_line(&self, buffer: &SourceBuffer) -> usize {
        if self.is_delimited(buffer) {
            self.start_line
        } else {
            self.start_line.saturating_sub(1).max(1)
        }
    }
}

/// Stores the set of active collapsed source-byte ranges.
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
///     byte_range: 100..250,
///     ..Default::default()
/// };
/// store.toggle(fold.byte_range.clone());
/// assert!(store.is_line_hidden(15, &[fold]));
/// ```
#[derive(Debug, Clone, Default)]
pub struct FoldStore {
    /// Exact source-byte spans of collapsed folds in the active document.
    pub folded_ranges: HashSet<Range<usize>>,
    initialized: bool,
    documents: crate::source::document_memory::DocumentMemory<(bool, HashSet<Range<usize>>)>,
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
    /// Only current exact fold ranges survive; explicit navigation may then reveal its
    /// destination without expanding unrelated restored regions.
    pub fn activate(&mut self, buffer: &SourceBuffer, ranges: &[FoldRange]) {
        let mut state = (self.initialized, std::mem::take(&mut self.folded_ranges));
        self.documents.activate(buffer, &mut state);
        self.initialized = state.0;
        self.folded_ranges = state.1;
        let identities: HashSet<_> = ranges
            .iter()
            .map(|range| range.byte_range.clone())
            .collect();
        self.folded_ranges
            .retain(|range| identities.contains(range));
    }

    /// Toggles the folded state for the given exact byte range.
    pub fn toggle(&mut self, range: Range<usize>) {
        self.initialized = true;
        if self.folded_ranges.contains(&range) {
            self.folded_ranges.remove(&range);
        } else {
            self.folded_ranges.insert(range);
        }
    }

    /// Returns `true` if the fold with the exact `range` is collapsed.
    #[must_use]
    pub fn is_folded(&self, range: &Range<usize>) -> bool {
        self.folded_ranges.contains(range)
    }

    /// Returns `true` if `line` is hidden because an enclosing fold is collapsed.
    #[must_use]
    pub fn is_line_hidden(&self, line: usize, fold_ranges: &[FoldRange]) -> bool {
        for fold in fold_ranges {
            if self.is_folded(&fold.byte_range) && line > fold.start_line && line <= fold.end_line {
                return true;
            }
        }
        false
    }

    /// Collapses parsed callable bodies, preserving other current fold choices.
    pub fn collapse_all_functions(&mut self, ranges: &[FoldRange]) {
        self.initialized = true;
        self.folded_ranges.extend(
            ranges
                .iter()
                .filter(|fold| fold.kind == FoldKind::Callable)
                .map(|fold| fold.byte_range.clone()),
        );
    }

    /// Replaces folds with callable bodies and types that hide no nested declarations.
    /// Containers exposing methods, variants or other declarations remain open.
    pub fn fold_to_definitions(&mut self, ranges: &[FoldRange]) {
        self.initialized = true;
        self.folded_ranges = ranges
            .iter()
            .filter(|fold| {
                fold.kind == FoldKind::Callable
                    || (fold.kind == FoldKind::Type && !fold.contains_declarations)
            })
            .map(|fold| fold.byte_range.clone())
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
            let expanded = super::review_folds::expanded_ranges(buffer, ranges, changes, revision);
            self.folded_ranges.retain(|range| !expanded.contains(range));
        }
    }

    /// Adopts defaults prepared with the successfully published source bundle.
    /// Restored or already edited documents retain their current choices.
    pub fn apply_prepared_defaults(&mut self, defaults: Option<HashSet<Range<usize>>>) {
        if !self.initialized {
            if let Some(defaults) = defaults {
                self.folded_ranges = defaults;
                self.initialized = true;
            }
        }
    }

    /// Unfolds all active folds.
    pub fn unfold_all(&mut self) {
        self.initialized = true;
        self.folded_ranges.clear();
    }

    /// Reveals exact bytes, including a hidden suffix on an otherwise visible row.
    pub fn reveal_byte(&mut self, byte: usize, buffer: &SourceBuffer, ranges: &[FoldRange]) {
        for fold in ranges {
            if fold.hidden_range(buffer).contains(&byte) {
                self.folded_ranges.remove(&fold.byte_range);
            }
        }
    }

    /// Reveals a canonical source line without expanding unrelated folds.
    pub fn reveal_line(&mut self, line: usize, ranges: &[FoldRange]) {
        for fold in ranges {
            if line > fold.start_line && line <= fold.end_line {
                self.folded_ranges.remove(&fold.byte_range);
            }
        }
    }
}

// Line-oriented assertions keep broad navigation fixtures readable. Product state
// and the overlapping-range regressions use canonical byte identities directly.
#[cfg(test)]
impl FoldStore {
    pub(crate) fn collapsed_headers(&self, ranges: &[FoldRange]) -> HashSet<usize> {
        ranges
            .iter()
            .filter(|fold| self.is_folded(&fold.byte_range))
            .map(|fold| fold.start_line)
            .collect()
    }

    pub(crate) fn toggle_header(&mut self, line: usize, ranges: &[FoldRange]) {
        if let Some(fold) = ranges.iter().find(|fold| fold.start_line == line) {
            self.toggle(fold.byte_range.clone());
        }
    }

    pub(crate) fn is_header_folded(&self, line: usize, ranges: &[FoldRange]) -> bool {
        ranges
            .iter()
            .any(|fold| fold.start_line == line && self.is_folded(&fold.byte_range))
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

        store.toggle(fn_fold.byte_range.clone());
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
        store.toggle_header(4, &ranges);
        store.toggle(999..1000);
        store.activate(&a, &ranges);
        assert_eq!(store.collapsed_headers(&ranges), HashSet::from([4]));
        store.activate(&b, &other_ranges);
        assert!(store.collapsed_headers(&ranges).is_empty());
        store.toggle_header(1, &other_ranges);
        for revision in [ReviewRevision::After, ReviewRevision::Before] {
            let snapshot =
                SourceBuffer::from_bytes("a.rs", a.data.clone(), Some(revision)).unwrap();
            store.activate(&snapshot, &ranges);
            assert!(store.collapsed_headers(&ranges).is_empty());
            store.toggle_header(1, &ranges);
        }
        store.activate(&a, &ranges);
        assert_eq!(store.collapsed_headers(&ranges), HashSet::from([4]));
        store.unfold_all();
        store.activate(&b, &other_ranges);
        assert_eq!(store.collapsed_headers(&other_ranges), HashSet::from([1]));
        store.activate(&a, &ranges);
        assert!(
            store.collapsed_headers(&ranges).is_empty(),
            "expanded choices must not resurrect old folds"
        );
        store.toggle_header(4, &ranges);
        let changed = SourceBuffer::from_str("a.rs", &a.text.replace("beta", "zeta"));
        store.activate(&changed, &ranges);
        assert!(store.collapsed_headers(&ranges).is_empty());
        store.activate(&a, &ranges);
        assert!(
            store.collapsed_headers(&ranges).is_empty(),
            "old-byte choices were invalidated"
        );
    }
}
