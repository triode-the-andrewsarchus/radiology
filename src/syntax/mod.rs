//! Syntax highlighting, document outline extraction, and code folding.
//!
//! Provides multi-language syntax highlighting powered by `syntect` ([`SyntaxHighlighter`]),
//! language identification ([`LanguageRegistry`]), Tree-sitter AST queries for hierarchical
//! document symbols ([`SymbolExtractor`]), and foldable block detection ([`FoldDetector`]).

pub mod calls;
pub mod folds;
pub mod highlighter;
pub mod language;
pub mod outline;
mod review_folds;
pub mod test_detector;

pub use folds::{FoldDetector, FoldRange, FoldStore};
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
    /// Conservative test-only regions and file-level gating.
    pub tests: TestDetection,
    /// Parsed calls with explicit query coverage; shared by lens and graph analysis.
    pub calls: calls::CallSites,
}

impl DocumentSyntax {
    /// Computes outline, folds and tests from one shared native tree.
    #[must_use]
    pub fn analyze(buffer: &crate::source::SourceBuffer) -> Self {
        let parsed = LanguageRegistry::for_path(&buffer.path)
            .grammar
            .and_then(|grammar| grammar.parse(&buffer.data).map(|tree| (grammar, tree)));
        let (symbols, tests, calls) = if let Some((grammar, tree)) = parsed {
            (
                SymbolExtractor::from_tree(buffer, grammar, tree.root_node()),
                TestRegionDetector::from_tree(buffer, grammar, tree.root_node()),
                calls::CallSites::from_tree(grammar, tree.root_node(), buffer),
            )
        } else {
            (
                SymbolExtractor::extract(buffer),
                TestDetection::default(),
                calls::CallSites::default(),
            )
        };
        let folds = FoldDetector::from_symbols(buffer, &symbols, &calls);
        Self {
            symbols,
            folds,
            tests,
            calls,
        }
    }
}

#[cfg(test)]
mod fold_defaults_tests;
