//! Syntax highlighting, document outline extraction, and code folding.
//!
//! Provides multi-language syntax highlighting powered by `syntect` ([`SyntaxHighlighter`]),
//! language identification ([`LanguageRegistry`]), Tree-sitter AST queries for hierarchical
//! document symbols ([`SymbolExtractor`]), and foldable block detection ([`FoldDetector`]).

pub mod folds;
pub mod highlighter;
pub mod language;
pub mod outline;

pub use folds::{FoldDetector, FoldRange, FoldStore};
pub use highlighter::{HighlightedSpan, SyntaxHighlighter};
pub use language::{LanguageInfo, LanguageRegistry, SupportedGrammar};
pub use outline::{DocumentSymbol, SymbolExtractor, SymbolKind};
