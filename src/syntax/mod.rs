//! Syntax highlighting, document outline extraction, and code folding.
//!
//! Provides multi-language syntax highlighting powered by `syntect` ([`SyntaxHighlighter`]),
//! language identification ([`LanguageRegistry`]), Tree-sitter AST queries for hierarchical
//! document symbols ([`SymbolExtractor`]), and foldable block detection ([`FoldDetector`]).

pub mod calls;
pub(crate) mod fold_queries;
pub mod folds;
pub mod highlighter;
pub mod language;
pub mod occurrences;
pub mod outline;
mod queries;
mod review_folds;
pub mod test_detector;

pub use folds::{FoldDetector, FoldKind, FoldRange, FoldStore};
pub use highlighter::{HighlightedSpan, SyntaxHighlighter};
pub use language::{LanguageInfo, LanguageRegistry, SupportedGrammar};
pub use outline::{DocumentSymbol, SymbolExtractor, SymbolKind};
pub use test_detector::{TestDetection, TestRegion, TestRegionDetector};

/// Immutable structural facts derived from one parse of an exact source buffer.
pub struct DocumentSyntax {
    /// Full outline, unaffected by source presentation filters.
    pub symbols: Vec<DocumentSymbol>,
    /// Manual fold candidates from declarations and comments.
    pub folds: Vec<FoldRange>,
    /// True when structural query facts are complete for this exact parse.
    pub folds_supported: bool,
    /// Independent folding query error; other source facts remain usable.
    pub fold_error: Option<String>,
    /// Conservative test-only regions and file-level gating.
    pub tests: TestDetection,
    /// Parsed calls with explicit query coverage; shared by lens and graph analysis.
    pub calls: calls::CallSites,
    /// Exact parsed callable scopes and excluded syntax for non-semantic pins.
    pub occurrences: occurrences::SyntacticOccurrences,
}

impl DocumentSyntax {
    /// Computes outline, folds and tests from one shared native tree.
    #[must_use]
    pub fn analyze(buffer: &crate::source::SourceBuffer) -> Self {
        let parsed = LanguageRegistry::for_path(&buffer.path)
            .grammar
            .and_then(|grammar| grammar.parse(&buffer.data).map(|tree| (grammar, tree)));
        let (symbols, tests, calls, folds, fold_error, folds_supported, occurrences) =
            if let Some((grammar, tree)) = parsed {
                let (folds, error) =
                    match FoldDetector::from_tree(buffer, grammar, tree.root_node()) {
                        Ok(folds) => (folds, None),
                        Err(error) => (Vec::new(), Some(error)),
                    };
                let supported = error.is_none();
                (
                    SymbolExtractor::from_tree(buffer, grammar, tree.root_node()),
                    TestRegionDetector::from_tree(buffer, grammar, tree.root_node()),
                    calls::CallSites::from_tree(grammar, tree.root_node(), buffer),
                    folds,
                    error,
                    supported,
                    occurrences::SyntacticOccurrences::from_tree(buffer, grammar, tree.root_node()),
                )
            } else {
                (
                    SymbolExtractor::extract(buffer),
                    TestDetection::default(),
                    calls::CallSites::default(),
                    Vec::new(),
                    None,
                    false,
                    occurrences::SyntacticOccurrences::default(),
                )
            };
        Self {
            symbols,
            folds,
            folds_supported,
            fold_error,
            tests,
            calls,
            occurrences,
        }
    }
}

#[cfg(test)]
mod fold_defaults_tests;
