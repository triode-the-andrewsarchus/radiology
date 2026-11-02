//! Explicit Rust impl structure; trait identities are resolved separately with SCIP.
use super::{AnalysisControl, SupportedGrammar};
use crate::source::SourceBuffer;
use serde::{Deserialize, Serialize};
use std::ops::Range;
use tree_sitter::Node;

/// One explicit trait impl from original source, excluding unexpanded macro tokens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustTraitImpl {
    /// Entire impl declaration, including its body.
    pub byte_range: Range<usize>,
    /// Terminal implemented-trait name, excluding generics and qualification.
    pub trait_name_range: Range<usize>,
    /// Exact trimmed header, including constraints and modifiers.
    pub signature: String,
    /// Exact implementing type spelling.
    pub target_type: String,
    /// Whether this is an explicit negative impl.
    pub is_negative: bool,
}

/// Lightweight structural facts, with parse errors retained as coverage gaps.
#[derive(Debug, Clone, Default)]
pub struct RustTraitImpls {
    /// Explicit, well-formed source blocks in source order.
    pub implementations: Vec<RustTraitImpl>,
    /// The Rust parse contained errors, even if some blocks were recovered.
    pub has_errors: bool,
}

impl RustTraitImpls {
    /// Parses only Rust impl facts without computing highlighting or other reader data.
    ///
    /// ```
    /// use radiology::{source::SourceBuffer, syntax::rust_impls::RustTraitImpls};
    /// let source = SourceBuffer::from_str("a.rs", "impl Read for File {} ");
    /// let facts = RustTraitImpls::analyze_cancellable(&source, || false).unwrap();
    /// assert_eq!(facts.implementations[0].target_type, "File");
    /// ```
    ///
    /// # Errors
    /// Returns an error if canceled or a registered Rust tree cannot be parsed.
    pub fn analyze_cancellable(
        buffer: &SourceBuffer,
        cancelled: impl Fn() -> bool,
    ) -> Result<Self, String> {
        let control = AnalysisControl::new(&cancelled);
        let Some((grammar, tree)) = control.parse(buffer)? else {
            return Err("Rust structure is unavailable for this source".into());
        };
        Self::from_tree(buffer, grammar, tree.root_node(), &control)
    }

    pub(super) fn from_tree(
        buffer: &SourceBuffer,
        grammar: SupportedGrammar,
        root: Node<'_>,
        control: &AnalysisControl<'_>,
    ) -> Result<Self, String> {
        let mut facts = Self::default();
        if grammar != SupportedGrammar::Rust {
            return Ok(facts);
        }
        facts.has_errors = root.has_error();
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            control.check()?;
            if node.kind() == "token_tree" {
                continue;
            }
            if let Some(item) = explicit_impl(node, &buffer.text) {
                facts.implementations.push(item);
            }
            let mut cursor = node.walk();
            pending.extend(node.named_children(&mut cursor));
        }
        facts
            .implementations
            .sort_by_key(|item| item.byte_range.start);
        Ok(facts)
    }
}

fn explicit_impl(node: Node<'_>, text: &str) -> Option<RustTraitImpl> {
    if node.kind() != "impl_item" || node.has_error() {
        return None;
    }
    let trait_node = node.child_by_field_name("trait")?;
    let name = terminal_name(trait_node)?;
    let target = node.child_by_field_name("type")?;
    let body = node.child_by_field_name("body")?;
    let mut cursor = node.walk();
    let is_negative = node.children(&mut cursor).any(|child| child.kind() == "!");
    Some(RustTraitImpl {
        byte_range: node.byte_range(),
        trait_name_range: name.byte_range(),
        signature: text[node.start_byte()..body.start_byte()].trim().to_owned(),
        target_type: text[target.byte_range()].to_owned(),
        is_negative,
    })
}

fn terminal_name(node: Node<'_>) -> Option<Node<'_>> {
    match node.kind() {
        "type_identifier" | "identifier" => Some(node),
        "generic_type" => terminal_name(node.child_by_field_name("type")?),
        "scoped_type_identifier" | "scoped_identifier" => {
            terminal_name(node.child_by_field_name("name")?)
        }
        _ => None,
    }
}
