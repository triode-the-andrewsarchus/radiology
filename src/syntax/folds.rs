//! Code folding detection and fold state management.

use crate::source::buffer::SourceBuffer;
use crate::syntax::outline::{DocumentSymbol, SymbolExtractor};
use std::collections::HashSet;

/// Represents a collapsible vertical region within a source document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoldRange {
    /// 1-based start line of the fold header.
    pub start_line: usize,
    /// 1-based ending line of the foldable block.
    pub end_line: usize,
    /// Header line text displayed when collapsed.
    pub header_preview: String,
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
/// };
/// store.toggle(10);
/// assert!(store.is_line_hidden(15, &[fold]));
/// ```
#[derive(Debug, Clone, Default)]
pub struct FoldStore {
    /// 1-based line numbers where folds are currently active.
    pub folded_lines: HashSet<usize>,
}

impl FoldStore {
    /// Creates an empty [`FoldStore`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggles the folded state for the given start line number.
    pub fn toggle(&mut self, start_line: usize) {
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

    /// Folds all function and method bodies by default.
    pub fn collapse_all_functions(&mut self, symbols: &[DocumentSymbol]) {
        fn collect_function_lines(syms: &[DocumentSymbol], out: &mut HashSet<usize>) {
            for sym in syms {
                if matches!(
                    sym.kind,
                    crate::syntax::outline::SymbolKind::Function
                        | crate::syntax::outline::SymbolKind::Method
                ) {
                    out.insert(sym.line);
                }
                collect_function_lines(&sym.children, out);
            }
        }
        collect_function_lines(symbols, &mut self.folded_lines);
    }

    /// Unfolds all active folds.
    pub fn unfold_all(&mut self) {
        self.folded_lines.clear();
    }
}

/// Detects foldable regions from document symbols and multi-line comments.
pub struct FoldDetector;

impl FoldDetector {
    /// Detects all foldable line ranges within the given [`SourceBuffer`].
    #[must_use]
    pub fn detect_folds(buffer: &SourceBuffer) -> Vec<FoldRange> {
        let symbols = SymbolExtractor::extract(buffer);
        let mut folds = Vec::new();

        fn add_symbol_folds(
            syms: &[DocumentSymbol],
            buffer: &SourceBuffer,
            folds: &mut Vec<FoldRange>,
        ) {
            for sym in syms {
                let start_line = buffer.line_at_byte(sym.range.start);
                let end_line = buffer.line_at_byte(sym.range.end);
                if end_line > start_line {
                    let header = buffer.line_text(start_line).trim().to_string();
                    folds.push(FoldRange {
                        start_line,
                        end_line,
                        header_preview: header,
                    });
                }
                add_symbol_folds(&sym.children, buffer, folds);
            }
        }

        add_symbol_folds(&symbols, buffer, &mut folds);
        Self::detect_comment_folds(buffer, &mut folds);

        folds.sort_by_key(|f| (f.start_line, f.end_line));
        folds.dedup_by_key(|f| f.start_line);
        folds
    }

    fn detect_comment_folds(buffer: &SourceBuffer, folds: &mut Vec<FoldRange>) {
        let mut in_block = false;
        let mut block_start = 0;

        for line_idx in 1..=buffer.line_count() {
            let text = buffer.line_text(line_idx).trim();
            if !in_block && text.starts_with("/*") && !text.contains("*/") {
                in_block = true;
                block_start = line_idx;
            } else if in_block && text.contains("*/") {
                if line_idx > block_start {
                    folds.push(FoldRange {
                        start_line: block_start,
                        end_line: line_idx,
                        header_preview: buffer.line_text(block_start).trim().to_string(),
                    });
                }
                in_block = false;
            }
        }
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
}
