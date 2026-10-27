use crate::source::buffer::SourceBuffer;
use crate::syntax::language::{LanguageRegistry, SupportedGrammar};
use serde::{Deserialize, Serialize};
use std::ops::Range;
use tree_sitter::{Node, Parser};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Method,
    Struct,
    Class,
    Enum,
    EnumMember,
    Trait,
    Interface,
    Module,
    Constant,
    Variable,
    TypeAlias,
}

impl SymbolKind {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Function => "Function",
            Self::Method => "Method",
            Self::Struct => "Struct",
            Self::Class => "Class",
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

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Function => "ƒ",
            Self::Method => "m",
            Self::Struct => "S",
            Self::Class => "C",
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub range: Range<usize>,
    pub selection_range: Range<usize>,
    pub line: usize,
    pub children: Vec<DocumentSymbol>,
    pub detail: Option<String>,
}

impl DocumentSymbol {
    pub fn contains_byte(&self, byte: usize) -> bool {
        self.range.contains(&byte)
    }

    /// Finds the innermost symbol containing the given byte offset.
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

    /// Builds a breadcrumb chain of enclosing symbols for a given byte offset.
    pub fn breadcrumbs(&self, byte: usize, trail: &mut Vec<DocumentSymbol>) {
        if self.contains_byte(byte) {
            trail.push(self.clone());
            for child in &self.children {
                child.breadcrumbs(byte, trail);
            }
        }
    }
}

pub struct SymbolExtractor;

impl SymbolExtractor {
    pub fn extract(buffer: &SourceBuffer) -> Vec<DocumentSymbol> {
        let lang_info = LanguageRegistry::for_path(&buffer.path);
        let Some(grammar) = lang_info.grammar else {
            return Self::extract_fallback(buffer);
        };

        let mut parser = Parser::new();
        let ts_language = match grammar {
            SupportedGrammar::Rust => tree_sitter_rust::LANGUAGE.into(),
            SupportedGrammar::Python => tree_sitter_python::LANGUAGE.into(),
            SupportedGrammar::Go => tree_sitter_go::LANGUAGE.into(),
            SupportedGrammar::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            SupportedGrammar::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            SupportedGrammar::C => tree_sitter_c::LANGUAGE.into(),
        };

        if parser.set_language(&ts_language).is_err() {
            return Self::extract_fallback(buffer);
        }

        let Some(tree) = parser.parse(&buffer.data, None) else {
            return Self::extract_fallback(buffer);
        };

        let root = tree.root_node();
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
            (SupportedGrammar::Rust, "impl_item") => (SymbolKind::Class, "type"),
            (SupportedGrammar::Rust, "mod_item") => (SymbolKind::Module, "name"),
            (SupportedGrammar::Rust, "type_item") => (SymbolKind::TypeAlias, "name"),

            (SupportedGrammar::Python, "function_definition") => (SymbolKind::Function, "name"),
            (SupportedGrammar::Python, "class_definition") => (SymbolKind::Class, "name"),

            (SupportedGrammar::Go, "function_declaration") => (SymbolKind::Function, "name"),
            (SupportedGrammar::Go, "method_declaration") => (SymbolKind::Method, "name"),
            (SupportedGrammar::Go, "type_spec") => (SymbolKind::Struct, "name"),

            (
                SupportedGrammar::JavaScript | SupportedGrammar::TypeScript,
                "function_declaration",
            ) => (SymbolKind::Function, "name"),
            (SupportedGrammar::JavaScript | SupportedGrammar::TypeScript, "class_declaration") => {
                (SymbolKind::Class, "name")
            }
            (SupportedGrammar::JavaScript | SupportedGrammar::TypeScript, "method_definition") => {
                (SymbolKind::Method, "name")
            }
            (SupportedGrammar::TypeScript, "interface_declaration") => {
                (SymbolKind::Interface, "name")
            }
            (SupportedGrammar::TypeScript, "type_alias_declaration") => {
                (SymbolKind::TypeAlias, "name")
            }

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
            detail: None,
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
            // If the declarator has children (e.g. C function_declarator), isolate the identifier name
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

        assert_eq!(symbols[1].kind, SymbolKind::Class); // impl Point
        assert_eq!(symbols[1].children.len(), 1);
        assert_eq!(symbols[1].children[0].name, "distance");
        assert_eq!(symbols[1].children[0].kind, SymbolKind::Function);

        assert_eq!(symbols[2].name, "calculate");
        assert_eq!(symbols[2].kind, SymbolKind::Function);
    }
}
