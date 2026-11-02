//! Native outline facts from the bundled Swift symbol queries used for folding.

use super::*;
use crate::syntax::{queries, AnalysisControl};
use std::{cmp::Reverse, collections::BTreeMap, iter::Peekable, vec::IntoIter};

pub(super) fn extract(
    buffer: &SourceBuffer,
    grammar: SupportedGrammar,
    root: Node<'_>,
    control: &AnalysisControl<'_>,
) -> Result<Vec<DocumentSymbol>, String> {
    let query = queries::query(grammar, queries::QueryRole::Folds)?;
    let names = query.capture_names();
    let mut symbols = BTreeMap::new();
    queries::visit_matches(query, root, &buffer.data, control, "Outline", |found| {
        let declaration = found
            .captures
            .iter()
            .find(|c| names[c.index as usize].starts_with("symbol."));
        let name = found
            .captures
            .iter()
            .find(|c| names[c.index as usize] == "name");
        if let (Some(declaration), Some(name)) = (declaration, name) {
            let node = declaration.node;
            let kind = match names[declaration.index as usize] {
                "symbol.type" => type_kind(node, buffer),
                "symbol.protocol" => {
                    if matches!(
                        grammar,
                        SupportedGrammar::TypeScript | SupportedGrammar::Tsx
                    ) {
                        SymbolKind::Interface
                    } else {
                        SymbolKind::Trait
                    }
                }
                "symbol.method" => SymbolKind::Method,
                "symbol.enum" => SymbolKind::Enum,
                "symbol.module" => SymbolKind::Module,
                "symbol.extension" => SymbolKind::Extension,
                "symbol.constant" => SymbolKind::Constant,
                "symbol.variable" => SymbolKind::Variable,
                "symbol.macro" => SymbolKind::Macro,
                "symbol.function" => SymbolKind::Function,
                "symbol.initializer" => SymbolKind::Initializer,
                "symbol.deinitializer" => SymbolKind::Deinitializer,
                "symbol.typealias" => SymbolKind::TypeAlias,
                "symbol.property" => SymbolKind::Property,
                "symbol.case" => SymbolKind::EnumMember,
                _ => return Ok(()),
            };
            let symbol = DocumentSymbol {
                name: buffer.text[name.node.byte_range()].to_owned(),
                kind,
                range: node.byte_range(),
                selection_range: name.node.byte_range(),
                line: buffer.line_at_byte(node.start_byte()),
                children: Vec::new(),
                detail: declaration_header(node, buffer).or_else(|| {
                    kind.is_callable()
                        .then(|| buffer.text[node.byte_range()].trim().to_owned())
                }),
            };
            symbols
                .entry((node.start_byte(), Reverse(node.end_byte())))
                .and_modify(|existing: &mut DocumentSymbol| {
                    if symbol_rank(kind) < symbol_rank(existing.kind) {
                        *existing = symbol.clone();
                    }
                })
                .or_insert(symbol);
            if symbols.len() > 65_536 {
                return Err("Outline query exceeded its declaration limit".into());
            }
        }
        Ok(())
    })?;
    let output = forest(
        &mut symbols
            .into_values()
            .collect::<Vec<_>>()
            .into_iter()
            .peekable(),
        None,
        control,
    );
    control.check()?;
    Ok(output)
}

// Match Swift's most-specific capture preference for overlapping declaration patterns.
fn symbol_rank(kind: SymbolKind) -> u8 {
    use SymbolKind::*;
    match kind {
        Struct | Union | Class | Trait | Interface | Enum => 0,
        Function | Method | Initializer | Deinitializer => 1,
        Extension | Module | TypeAlias | Macro => 2,
        Property | Constant | EnumMember => 3,
        Variable => 4,
    }
}

fn type_kind(node: Node<'_>, buffer: &SourceBuffer) -> SymbolKind {
    match node.kind() {
        "struct_item" => return SymbolKind::Struct,
        "union_item" => return SymbolKind::Union,
        "type_spec" => {
            return match node.child_by_field_name("type").map(|node| node.kind()) {
                Some("struct_type") => SymbolKind::Struct,
                Some("interface_type") => SymbolKind::Interface,
                _ => SymbolKind::TypeAlias,
            }
        }
        _ => {}
    }
    if let Some(kind) = node.child_by_field_name("declaration_kind") {
        return match node_text(kind, buffer).as_str() {
            "struct" => SymbolKind::Struct,
            "enum" => SymbolKind::Enum,
            "extension" => SymbolKind::Extension,
            _ => SymbolKind::Class,
        };
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "interface" {
            return SymbolKind::Interface;
        }
        if child.kind() == "modifiers" {
            let mut cursor = child.walk();
            if child.named_children(&mut cursor).any(|modifier| {
                modifier.kind() == "class_modifier" && node_text(modifier, buffer) == "enum"
            }) {
                return SymbolKind::Enum;
            }
        }
    }
    SymbolKind::Class
}

fn forest(
    symbols: &mut Peekable<IntoIter<DocumentSymbol>>,
    parent: Option<&Range<usize>>,
    control: &AnalysisControl<'_>,
) -> Vec<DocumentSymbol> {
    let mut output = Vec::new();
    while symbols.peek().is_some_and(|symbol| {
        parent.is_none_or(|parent| {
            parent.start <= symbol.range.start && symbol.range.end <= parent.end
        })
    }) {
        if control.is_cancelled() {
            break;
        }
        let mut symbol = symbols.next().expect("peeked declaration");
        symbol.children = forest(symbols, Some(&symbol.range), control);
        output.push(symbol);
    }
    output
}
