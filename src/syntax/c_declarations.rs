//! C/C++ declarator spines and navigation-only recovery, matching the Swift parser.
use super::{AnalysisControl, DocumentSymbol, SupportedGrammar, SymbolKind};
use crate::source::SourceBuffer;
use tree_sitter::Node;

pub(super) fn name(
    mut node: Node<'_>,
    require_function: bool,
    allow_pointer: bool,
) -> Option<Node<'_>> {
    let mut function = false;
    loop {
        node = match node.kind() {
            "identifier"
            | "field_identifier"
            | "type_identifier"
            | "namespace_identifier"
            | "nested_namespace_specifier"
            | "destructor_name"
            | "operator_name" => return (!require_function || function).then_some(node),
            "function_declarator" => {
                function = true;
                let child = node.child_by_field_name("declarator")?;
                if !allow_pointer && child.kind() == "parenthesized_declarator" {
                    return None;
                }
                child
            }
            "qualified_identifier" | "template_function" | "template_method" | "template_type" => {
                node.child_by_field_name("name")?
            }
            "reference_declarator" | "parenthesized_declarator" => {
                node.named_child(node.named_child_count().checked_sub(1)?)?
            }
            _ => node.child_by_field_name("declarator")?,
        };
    }
}

pub(super) fn recover(
    buffer: &SourceBuffer,
    grammar: SupportedGrammar,
    root: Node<'_>,
    control: &AnalysisControl<'_>,
) -> Result<Vec<DocumentSymbol>, String> {
    if !root.has_error() || !matches!(grammar, SupportedGrammar::C | SupportedGrammar::Cpp) {
        return Ok(Vec::new());
    }
    let mut pending = vec![root];
    let mut declarations = Vec::new();
    while let Some(node) = pending.pop() {
        control.check()?;
        if node.kind() == "function_declarator" {
            if let Some(name) = name(node, true, false) {
                let mut end = node.end_byte();
                while buffer
                    .data
                    .get(end)
                    .is_some_and(|byte| matches!(byte, b'\t' | b'\r' | b'\n' | b' '))
                {
                    end += 1;
                }
                if buffer.data.get(end) == Some(&b'{') {
                    declarations.push(DocumentSymbol {
                        name: buffer.text[name.byte_range()].into(),
                        kind: SymbolKind::Function,
                        range: node.start_byte()..end + 1,
                        selection_range: name.byte_range(),
                        line: buffer.line_at_byte(node.start_byte()),
                        children: Vec::new(),
                        detail: None,
                    });
                }
            }
        }
        let mut cursor = node.walk();
        pending.extend(node.named_children(&mut cursor));
    }
    Ok(declarations)
}
