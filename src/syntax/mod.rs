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
pub mod native_highlights;
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

/// One latched stop request shared by native parsing and derived fact stages.
struct AnalysisControl<'a> {
    cancelled: &'a dyn Fn() -> bool,
    stopped: std::cell::Cell<bool>,
}

impl<'a> AnalysisControl<'a> {
    fn new(cancelled: &'a dyn Fn() -> bool) -> Self {
        Self {
            cancelled,
            stopped: std::cell::Cell::new(false),
        }
    }

    fn parse(
        &self,
        buffer: &crate::source::SourceBuffer,
    ) -> Result<Option<(SupportedGrammar, tree_sitter::Tree)>, String> {
        self.check()?;
        LanguageRegistry::for_path(&buffer.path)
            .grammar
            .map(|grammar| {
                grammar
                    .parse_cancellable(&buffer.data, self)
                    .map(|tree| tree.map(|tree| (grammar, tree)))
            })
            .transpose()
            .map(Option::flatten)
    }

    fn is_cancelled(&self) -> bool {
        if !self.stopped.get() && (self.cancelled)() {
            self.stopped.set(true);
        }
        self.stopped.get()
    }

    fn check(&self) -> Result<(), String> {
        if self.is_cancelled() {
            Err("Syntax analysis cancelled".into())
        } else {
            Ok(())
        }
    }
}

/// Immutable structural facts derived from one parse of an exact source buffer.
#[derive(Default)]
pub struct DocumentSyntax {
    /// Full outline, unaffected by source presentation filters.
    pub symbols: Vec<DocumentSymbol>,
    /// Native parser failure for a registered language; fallback rows may remain.
    ///
    /// Unsupported formats and recoverable errors inside a returned syntax tree
    /// do not set this field. Hosts should retain source and explain degraded structure.
    pub parse_error: Option<String>,
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
    /// Native source colors retained for exact-buffer theme changes.
    pub native_highlights: std::sync::Arc<native_highlights::NativeHighlights>,
}

impl DocumentSyntax {
    /// Computes outline, folds and tests from one shared native tree.
    #[must_use]
    pub fn analyze(buffer: &crate::source::SourceBuffer) -> Self {
        Self::analyze_cancellable(buffer, || false).expect("uncancelled syntax analysis")
    }

    pub(crate) fn analyze_cancellable(
        buffer: &crate::source::SourceBuffer,
        cancelled: impl Fn() -> bool,
    ) -> Result<Self, String> {
        let control = AnalysisControl::new(&cancelled);
        control.check()?;
        let parsed = control.parse(buffer)?;
        let Some((grammar, tree)) = parsed else {
            return Self::without_tree(buffer, &control);
        };
        let root = tree.root_node();
        let (folds, fold_error) = match FoldDetector::from_tree(buffer, grammar, root, &control) {
            Ok(folds) => (folds, None),
            Err(error) => (Vec::new(), Some(error)),
        };
        control.check()?;
        let symbols = SymbolExtractor::from_tree(buffer, grammar, root, &control);
        control.check()?;
        let tests = TestRegionDetector::from_tree(buffer, grammar, root, &control);
        control.check()?;
        let calls = calls::CallSites::from_tree_cancellable(grammar, root, buffer, &control);
        control.check()?;
        let occurrences =
            occurrences::SyntacticOccurrences::from_tree(buffer, grammar, root, &control);
        control.check()?;
        let native_highlights = std::sync::Arc::new(
            native_highlights::NativeHighlights::from_tree(buffer, grammar, root, &control),
        );
        control.check()?;
        Ok(Self {
            symbols,
            tests,
            calls,
            folds,
            folds_supported: fold_error.is_none(),
            fold_error,
            occurrences,
            native_highlights,
            parse_error: None,
        })
    }

    fn without_tree(
        buffer: &crate::source::SourceBuffer,
        control: &AnalysisControl<'_>,
    ) -> Result<Self, String> {
        let language = LanguageRegistry::for_path(&buffer.path);
        let parse_error = language.grammar.map(|_| format!("Could not read {} structure. Source remains available; any outline rows are approximate.", language.display_name));
        let symbols = SymbolExtractor::extract_fallback(buffer, control);
        control.check()?;
        Ok(Self {
            symbols,
            parse_error,
            ..Self::default()
        })
    }
}

#[cfg(test)]
mod fold_defaults_tests;

#[cfg(test)]
mod cancellation_tests;
