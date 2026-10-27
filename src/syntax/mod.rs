pub mod folds;
pub mod highlighter;
pub mod language;
pub mod outline;

pub use folds::{FoldDetector, FoldRange, FoldStore};
pub use highlighter::{HighlightedSpan, SyntaxHighlighter};
pub use language::{LanguageInfo, LanguageRegistry, SupportedGrammar};
pub use outline::{DocumentSymbol, SymbolExtractor, SymbolKind};
