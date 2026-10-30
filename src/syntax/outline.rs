//! Hierarchical document symbol extraction powered by Tree-sitter.

use crate::source::buffer::SourceBuffer;
use crate::syntax::language::{LanguageRegistry, SupportedGrammar};
use serde::{Deserialize, Serialize};
use std::ops::Range;
use tree_sitter::Node;

/// Enumerates kinds of declarations extracted into the document outline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    /// Function declaration.
    Function,
    /// Method declaration.
    Method,
    /// Struct declaration.
    Struct,
    /// Class declaration.
    Class,
    /// Extension or implementation of an existing type.
    Extension,
    /// Enum declaration.
    Enum,
    /// Individual enum member or variant.
    EnumMember,
    /// Trait declaration.
    Trait,
    /// Interface declaration.
    Interface,
    /// Module declaration.
    Module,
    /// Constant definition.
    Constant,
    /// Variable definition.
    Variable,
    /// Type alias declaration.
    TypeAlias,
}

impl SymbolKind {
    /// Returns the human-readable display title for this kind.
    #[must_use]
    pub fn title(&self) -> &'static str {
        match self {
            Self::Function => "Function",
            Self::Method => "Method",
            Self::Struct => "Struct",
            Self::Class => "Class",
            Self::Extension => "Implementation",
            Self::Enum => "Enum",
            Self::EnumMember => "Enum Member",
            Self::Trait => "Trait",
            Self::Interface => "Interface",
            Self::Module => "Module",
            Self::Constant => "Constant",
            Self::Variable => "Variable",
            Self::TypeAlias => "Type",
        }
    }

    /// Returns the compact letter glyph used in outline views.
    #[must_use]
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Function => "ƒ",
            Self::Method => "m",
            Self::Struct => "S",
            Self::Class => "C",
            Self::Extension => "I",
            Self::Enum => "E",
            Self::EnumMember => "e",
            Self::Trait => "T",
            Self::Interface => "I",
            Self::Module => "M",
            Self::Constant => "c",
            Self::Variable => "v",
            Self::TypeAlias => "t",
        }
    }
}

/// Represents a symbol declaration in the document outline tree.
///
/// # Examples
///
/// ```rust
/// use radiology::source::SourceBuffer;
/// use radiology::syntax::outline::{SymbolExtractor, SymbolKind};
///
/// let code = "pub struct Point { x: f64, y: f64 }\n";
/// let buf = SourceBuffer::from_str("geom.rs", code);
/// let symbols = SymbolExtractor::extract(&buf);
///
/// assert_eq!(symbols.len(), 1);
/// assert_eq!(symbols[0].name, "Point");
/// assert_eq!(symbols[0].kind, SymbolKind::Struct);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentSymbol {
    /// Symbol name or identifier.
    pub name: String,
    /// Classification of the symbol declaration.
    pub kind: SymbolKind,
    /// Byte range encompassing the entire declaration.
    pub range: Range<usize>,
    /// Byte range of just the identifier name.
    pub selection_range: Range<usize>,
    /// 1-based line number of the declaration.
    pub line: usize,
    /// Child symbol declarations nested inside this symbol.
    pub children: Vec<DocumentSymbol>,
    /// Optional signature detail or parameter summary.
    pub detail: Option<String>,
}

impl DocumentSymbol {
    /// Returns whether this declaration is a function or method.
    #[must_use]
    pub fn is_callable(&self) -> bool {
        matches!(self.kind, SymbolKind::Function | SymbolKind::Method)
    }
    /// Returns `true` if this symbol's full range contains `byte`.
    #[must_use]
    pub fn contains_byte(&self, byte: usize) -> bool {
        self.range.contains(&byte)
    }

    /// Finds the innermost symbol containing the given byte offset.
    #[must_use]
    pub fn find_innermost(&self, byte: usize) -> Option<&DocumentSymbol> {
        if !self.contains_byte(byte) {
            return None;
        }
        for child in &self.children {
            if let Some(inner) = child.find_innermost(byte) {
                return Some(inner);
            }
        }
        Some(self)
    }

    /// Builds a breadcrumb chain of enclosing symbols containing `byte`.
    pub fn breadcrumbs(&self, byte: usize, trail: &mut Vec<DocumentSymbol>) {
        if self.contains_byte(byte) {
            trail.push(self.clone());
            for child in &self.children {
                child.breadcrumbs(byte, trail);
            }
        }
    }

    /// Returns the total number of lines spanned by this declaration.
    #[must_use]
    pub fn line_count(&self, buffer: &SourceBuffer) -> usize {
        let start = buffer.line_at_byte(self.range.start);
        let end = buffer.line_at_byte(self.range.end.saturating_sub(1));
        end.saturating_sub(start) + 1
    }
}

/// One outline row with its actual source span and presentation indentation.
pub struct OutlineRow<'a> {
    /// Original declaration; source identity and navigation ranges are preserved.
    pub symbol: &'a DocumentSymbol,
    /// Nesting depth in the non-callable declaration forest.
    pub depth: usize,
    /// Exact number of canonical source lines occupied by the declaration.
    pub line_count: usize,
}

/// Orders functions largest first, followed by a hierarchy of other declarations.
/// Callable containers are removed from that hierarchy, promoting their children.
#[must_use]
pub fn outline_rows<'a>(
    symbols: &'a [DocumentSymbol],
    buffer: &SourceBuffer,
) -> Vec<OutlineRow<'a>> {
    fn collect<'a>(
        symbols: &'a [DocumentSymbol],
        depth: usize,
        buffer: &SourceBuffer,
        functions: &mut Vec<OutlineRow<'a>>,
        declarations: &mut Vec<OutlineRow<'a>>,
    ) {
        for symbol in symbols {
            let callable = symbol.is_callable();
            let row = OutlineRow {
                symbol,
                depth: if callable { 0 } else { depth },
                line_count: symbol.line_count(buffer),
            };
            if callable {
                functions.push(row);
            } else {
                declarations.push(row);
            }
            collect(
                &symbol.children,
                depth + usize::from(!callable),
                buffer,
                functions,
                declarations,
            );
        }
    }
    let (mut functions, mut declarations) = (Vec::new(), Vec::new());
    collect(symbols, 0, buffer, &mut functions, &mut declarations);
    functions.sort_by_key(|row| (std::cmp::Reverse(row.line_count), row.symbol.range.start));
    functions.extend(declarations);
    functions
}

/// Extracts hierarchical document symbols using Tree-sitter parsers.
pub struct SymbolExtractor;

impl SymbolExtractor {
    /// Extracts a tree of [`DocumentSymbol`] declarations from the given [`SourceBuffer`].
    #[must_use]
    pub fn extract(buffer: &SourceBuffer) -> Vec<DocumentSymbol> {
        let lang_info = LanguageRegistry::for_path(&buffer.path);
        let Some(grammar) = lang_info.grammar else {
            return Self::extract_fallback(buffer);
        };

        let Some(tree) = grammar.parse(&buffer.data) else {
            return Self::extract_fallback(buffer);
        };

        Self::from_tree(buffer, grammar, tree.root_node())
    }

    pub(crate) fn from_tree(
        buffer: &SourceBuffer,
        grammar: SupportedGrammar,
        root: Node<'_>,
    ) -> Vec<DocumentSymbol> {
        let mut symbols = Vec::new();
        Self::collect_symbols(root, buffer, grammar, &mut symbols);
        symbols
    }

    fn collect_symbols(
        node: Node,
        buffer: &SourceBuffer,
        grammar: SupportedGrammar,
        out: &mut Vec<DocumentSymbol>,
    ) {
        if let Some(symbol) = Self::parse_node(node, buffer, grammar) {
            out.push(symbol);
        } else {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                Self::collect_symbols(child, buffer, grammar, out);
            }
        }
    }

    fn parse_node(
        node: Node,
        buffer: &SourceBuffer,
        grammar: SupportedGrammar,
    ) -> Option<DocumentSymbol> {
        let kind_str = node.kind();
        let (kind, name_field) = match (grammar, kind_str) {
            (SupportedGrammar::Rust, "function_item") => (SymbolKind::Function, "name"),
            (SupportedGrammar::Rust, "struct_item") => (SymbolKind::Struct, "name"),
            (SupportedGrammar::Rust, "enum_item") => (SymbolKind::Enum, "name"),
            (SupportedGrammar::Rust, "trait_item") => (SymbolKind::Trait, "name"),
            (SupportedGrammar::Rust, "impl_item") => (SymbolKind::Extension, "type"),
            (SupportedGrammar::Rust, "mod_item") => (SymbolKind::Module, "name"),
            (SupportedGrammar::Rust, "type_item") => (SymbolKind::TypeAlias, "name"),
            (SupportedGrammar::Rust, "const_item") => (SymbolKind::Constant, "name"),
            (SupportedGrammar::Rust, "static_item") => (SymbolKind::Variable, "name"),
            (SupportedGrammar::Rust, "enum_variant") => (SymbolKind::EnumMember, "name"),

            (SupportedGrammar::Python, "function_definition") => (SymbolKind::Function, "name"),
            (SupportedGrammar::Python, "class_definition") => (SymbolKind::Class, "name"),

            (SupportedGrammar::Go, "function_declaration") => (SymbolKind::Function, "name"),
            (SupportedGrammar::Go, "method_declaration") => (SymbolKind::Method, "name"),
            (SupportedGrammar::Go, "type_spec") => (SymbolKind::Struct, "name"),

            (
                SupportedGrammar::JavaScript | SupportedGrammar::TypeScript | SupportedGrammar::Tsx,
                "function_declaration",
            ) => (SymbolKind::Function, "name"),
            (
                SupportedGrammar::JavaScript | SupportedGrammar::TypeScript | SupportedGrammar::Tsx,
                "class_declaration",
            ) => (SymbolKind::Class, "name"),
            (
                SupportedGrammar::JavaScript | SupportedGrammar::TypeScript | SupportedGrammar::Tsx,
                "method_definition",
            ) => (SymbolKind::Method, "name"),
            (SupportedGrammar::TypeScript | SupportedGrammar::Tsx, "interface_declaration") => {
                (SymbolKind::Interface, "name")
            }
            (SupportedGrammar::TypeScript | SupportedGrammar::Tsx, "type_alias_declaration") => {
                (SymbolKind::TypeAlias, "name")
            }

            (
                SupportedGrammar::Swift,
                "function_declaration" | "init_declaration" | "deinit_declaration",
            ) => (SymbolKind::Function, "name"),
            (SupportedGrammar::Swift, "class_declaration") => {
                let kind = node
                    .child_by_field_name("declaration_kind")
                    .map(|node| node_text(node, buffer));
                (
                    match kind.as_deref() {
                        Some("struct") => SymbolKind::Struct,
                        Some("enum") => SymbolKind::Enum,
                        Some("extension") => SymbolKind::Extension,
                        _ => SymbolKind::Class,
                    },
                    "name",
                )
            }
            (SupportedGrammar::Swift, "protocol_declaration") => (SymbolKind::Trait, "name"),
            (SupportedGrammar::Swift, "typealias_declaration") => (SymbolKind::TypeAlias, "name"),
            (SupportedGrammar::Kotlin, "function_declaration") => (SymbolKind::Function, "name"),
            (SupportedGrammar::Kotlin, "class_declaration" | "object_declaration") => {
                (SymbolKind::Class, "name")
            }
            (SupportedGrammar::Kotlin, "type_alias") => (SymbolKind::TypeAlias, "type"),

            (SupportedGrammar::C, "function_definition") => (SymbolKind::Function, "declarator"),
            (SupportedGrammar::C, "struct_specifier") => (SymbolKind::Struct, "name"),
            (SupportedGrammar::C, "enum_specifier") => (SymbolKind::Enum, "name"),

            _ => return None,
        };

        let name_node = node.child_by_field_name(name_field).unwrap_or(node);
        let name = node_text(name_node, buffer);
        if name.is_empty() {
            return None;
        }

        let range = node.start_byte()..node.end_byte();
        let selection_range = name_node.start_byte()..name_node.end_byte();
        let line = buffer.line_at_byte(range.start);

        let mut children = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::collect_symbols(child, buffer, grammar, &mut children);
        }

        Some(DocumentSymbol {
            name,
            kind,
            range,
            selection_range,
            line,
            children,
            detail: node
                .child_by_field_name("body")
                .or_else(|| {
                    let mut cursor = node.walk();
                    let body = node.named_children(&mut cursor).find(|child| {
                        matches!(
                            child.kind(),
                            "function_body" | "class_body" | "enum_class_body"
                        )
                    });
                    body
                })
                .map(|body| {
                    buffer.text[node.start_byte()..body.start_byte()]
                        .trim()
                        .to_string()
                }),
        })
    }

    fn extract_fallback(buffer: &SourceBuffer) -> Vec<DocumentSymbol> {
        let mut symbols = Vec::new();
        for line_num in 1..=buffer.line_count() {
            let line = buffer.line_text(line_num);
            let trimmed = line.trim();
            if trimmed.starts_with("def ")
                || trimmed.starts_with("fn ")
                || trimmed.starts_with("func ")
            {
                let range = buffer.line_byte_range(line_num);
                let name = trimmed
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or(trimmed)
                    .split('(')
                    .next()
                    .unwrap_or("")
                    .to_string();
                if !name.is_empty() {
                    symbols.push(DocumentSymbol {
                        name,
                        kind: SymbolKind::Function,
                        range: range.clone(),
                        selection_range: range,
                        line: line_num,
                        children: Vec::new(),
                        detail: None,
                    });
                }
            }
        }
        symbols
    }
}

fn node_text(node: Node, buffer: &SourceBuffer) -> String {
    let start = node.start_byte();
    let end = node.end_byte();
    if start < end && end <= buffer.data.len() {
        if let Ok(s) = std::str::from_utf8(&buffer.data[start..end]) {
            let first_word = s.split('(').next().unwrap_or(s).trim();
            return first_word
                .split_whitespace()
                .next_back()
                .unwrap_or(first_word)
                .to_string();
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_constants_statics_and_enum_variants_keep_exact_name_ranges() {
        let text = "// café 😀\npub const LIMIT: usize = 4;\npub static READY: bool = true;\npub enum Choice { First, Second(i32) }\n";
        let buffer = SourceBuffer::from_str("main.rs", text);
        let symbols = SymbolExtractor::extract(&buffer);
        for (symbol, name, kind) in symbols
            .iter()
            .zip([
                ("LIMIT", SymbolKind::Constant),
                ("READY", SymbolKind::Variable),
                ("Choice", SymbolKind::Enum),
            ])
            .map(|(symbol, (name, kind))| (symbol, name, kind))
        {
            assert_eq!((&symbol.name, symbol.kind), (&name.to_string(), kind));
            assert_eq!(&text[symbol.selection_range.clone()], name);
        }
        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[2].children.len(), 2);
        for (variant, name) in symbols[2].children.iter().zip(["First", "Second"]) {
            assert_eq!(variant.kind, SymbolKind::EnumMember);
            assert_eq!(&text[variant.selection_range.clone()], name);
        }
    }

    #[test]
    fn test_extract_rust_symbols() {
        let code = r#"
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn distance(&self) -> f64 {
        0.0
    }
}

fn calculate() {
}
"#;
        let buf = SourceBuffer::from_str("src/geom.rs", code);
        let symbols = SymbolExtractor::extract(&buf);

        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0].name, "Point");
        assert_eq!(symbols[0].kind, SymbolKind::Struct);

        assert_eq!(symbols[1].kind, SymbolKind::Extension);
        assert_eq!(symbols[1].children.len(), 1);
        assert_eq!(symbols[1].children[0].name, "distance");
        assert_eq!(symbols[1].children[0].kind, SymbolKind::Function);

        assert_eq!(symbols[2].name, "calculate");
        assert_eq!(symbols[2].kind, SymbolKind::Function);
    }
    #[test]
    fn tsx_structure_and_outline_order_use_real_source_spans() {
        let source = "function short() { return <div/>; }\nfunction long() {\n const value = <div/>;\n return value;\n}\ninterface Widget { name: string }\n";
        let buffer = SourceBuffer::from_str("view.tsx", source);
        let symbols = SymbolExtractor::extract(&buffer);
        let rows = outline_rows(&symbols, &buffer);
        assert_eq!(
            rows.iter()
                .map(|row| row.symbol.name.as_str())
                .collect::<Vec<_>>(),
            vec!["long", "short", "Widget"]
        );
        assert_eq!(rows[0].line_count, 4);
        assert!(rows[0]
            .symbol
            .detail
            .as_ref()
            .unwrap()
            .contains("function long()"));
        assert!(!crate::syntax::folds::FoldDetector::detect_folds(&buffer).is_empty());
    }
    #[test]
    fn swift_and_kotlin_use_native_structural_ranges_and_headers() {
        for (path, source, expected) in [
            (
                "Reader.swift",
                "struct Reader {\n func read(value: Int) -> Int {\n value + 1\n }\n}\n",
                "Reader",
            ),
            (
                "Reader.kt",
                "class Reader {\n fun read(value: Int): Int {\n return value + 1\n }\n}\n",
                "Reader",
            ),
        ] {
            let buffer = SourceBuffer::from_str(path, source);
            let grammar = LanguageRegistry::for_path(path).grammar.unwrap();
            let tree = grammar.parse(&buffer.data).unwrap();
            assert!(
                !tree.root_node().has_error(),
                "{}",
                tree.root_node().to_sexp()
            );
            let symbols = SymbolExtractor::extract(&buffer);
            assert_eq!(symbols[0].name, expected);
            assert_eq!(symbols[0].line_count(&buffer), 5);
            let function = &symbols[0].children[0];
            assert_eq!(function.name, "read");
            assert_eq!(&source[function.selection_range.clone()], "read");
            assert_eq!(function.line_count(&buffer), 3);
            assert!(function.detail.as_deref().unwrap().contains("value"));
            assert!(!function.detail.as_deref().unwrap().contains("+ 1"));
            assert!(!crate::syntax::folds::FoldDetector::detect_folds(&buffer).is_empty());
        }
    }
}
