//! Syntactic call sites shared by Critical Lens and future call graph queries.

use super::SupportedGrammar;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use tree_sitter::{Node, QueryCursor, StreamingIterator};

/// A parsed call expression; syntax alone does not establish a semantic edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallSite {
    /// `call`, `macro`, or `dynamic` for a callee without a supported name token.
    pub kind: String,
    /// Exact original-byte callee name, when supported by the grammar.
    pub name: Option<Range<usize>>,
    /// Full call or macro expression in original bytes.
    pub range: Range<usize>,
    /// Index of the innermost lexical callable, when one contains this site.
    pub owner: Option<usize>,
}

/// A parsed callable scope; its calls need not execute with its enclosing scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallableScope {
    /// True for closures/lambdas, false for named functions and methods.
    pub closure: bool,
    /// Complete declaration or closure bytes.
    pub range: Range<usize>,
    /// Declaration name bytes, when the grammar exposes a name field.
    pub name: Option<Range<usize>>,
    /// Parsed body bytes, absent for abstract signatures.
    pub body_range: Option<Range<usize>>,
    /// Declaration bytes including attached Rust attributes or Python decorators.
    pub annotated_range: Range<usize>,
}

impl CallableScope {
    /// Returns whether this callable has a parsed body rather than an abstract signature.
    #[must_use]
    pub fn has_body(&self) -> bool {
        self.body_range.is_some()
    }

    fn from_node(node: Node<'_>, closure: bool) -> Self {
        let mut cursor = node.walk();
        let body = node.child_by_field_name("body").or_else(|| {
            node.named_children(&mut cursor)
                .find(|child| matches!(child.kind(), "block" | "function_body"))
        });
        Self {
            closure,
            range: node.byte_range(),
            name: node
                .child_by_field_name("name")
                .map(|name| name.byte_range()),
            body_range: body
                .map(|body| body.byte_range())
                .or_else(|| closure.then(|| node.byte_range())),
            annotated_range: annotated_range(node),
        }
    }
}

pub(super) fn annotated_range(node: Node<'_>) -> Range<usize> {
    let mut range = node.byte_range();
    if node.kind() == "function_definition" {
        if let Some(parent) = node
            .parent()
            .filter(|parent| parent.kind() == "decorated_definition")
        {
            return parent.byte_range();
        }
    }
    if node.kind() == "function_item" {
        let mut previous = node.prev_named_sibling();
        while let Some(sibling) = previous {
            match sibling.kind() {
                "attribute_item" => range.start = sibling.start_byte(),
                "line_comment" | "block_comment" => {}
                _ => break,
            }
            previous = sibling.prev_named_sibling();
        }
    }
    range
}

/// Call-query coverage for one exact document parse.
#[derive(Debug, Clone, Default)]
pub struct CallSites {
    /// Complete source-ordered sites, including unresolved/dynamic callees.
    pub sites: Vec<CallSite>,
    /// Lexical scopes in source order, with outer scopes before inner scopes.
    pub scopes: Vec<CallableScope>,
    /// True when the tree contains syntax errors or missing nodes.
    pub parse_errors: bool,
    /// False for unsupported grammar/query or a bounded query failure.
    pub supported: bool,
    /// Query compilation/limit failure, distinct from an unsupported language.
    pub error: Option<String>,
    identity: Option<crate::source::buffer::SourceIdentity>,
}

impl CallSites {
    /// Extracts calls from an existing parse using shared Swift query resources.
    #[must_use]
    pub fn from_tree(
        grammar: SupportedGrammar,
        root: Node<'_>,
        buffer: &crate::source::SourceBuffer,
    ) -> Self {
        match extract(grammar, root, &buffer.data) {
            Ok((sites, scopes)) => Self {
                sites,
                scopes,
                parse_errors: root.has_error(),
                supported: true,
                error: None,
                identity: Some(crate::source::buffer::SourceIdentity::new(buffer)),
            },
            Err(error) => Self {
                error: Some(error),
                ..Default::default()
            },
        }
    }
    /// Returns whether complete call-query facts describe the exact source/revision.
    #[must_use]
    pub fn matches(&self, buffer: &crate::source::SourceBuffer) -> bool {
        self.supported
            && self
                .identity
                .as_ref()
                .is_some_and(|identity| identity.matches(buffer))
    }
}

fn extract(
    grammar: SupportedGrammar,
    root: Node<'_>,
    bytes: &[u8],
) -> Result<(Vec<CallSite>, Vec<CallableScope>), String> {
    let query = super::queries::query(grammar, super::queries::QueryRole::Calls)?;
    let names = query.capture_names();
    let mut cursor = QueryCursor::new();
    cursor.set_match_limit(65_536);
    let mut scopes = BTreeMap::new();
    let mut named = BTreeMap::new();
    let mut fallback = BTreeSet::new();
    let mut matches = cursor.matches(query, root, bytes);
    while let Some(found) = matches.next() {
        if let Some(capture) = found
            .captures
            .iter()
            .find(|capture| names[capture.index as usize].starts_with("scope."))
        {
            let scope = CallableScope::from_node(
                capture.node,
                names[capture.index as usize] == "scope.closure",
            );
            scopes.insert(
                (scope.range.start, std::cmp::Reverse(scope.range.end)),
                scope,
            );
            continue;
        }
        let Some(site) = found.captures.iter().find(|capture| {
            let name = names[capture.index as usize];
            name == "call" || (name.starts_with("call.") && name != "call.name")
        }) else {
            continue;
        };
        let range = site.node.byte_range();
        let key = (range.start, range.end);
        if range.end > bytes.len() {
            continue;
        }
        let kind = names[site.index as usize];
        if kind == "call.any" {
            fallback.insert(key);
        } else if let Some(name) = found
            .captures
            .iter()
            .find(|capture| names[capture.index as usize] == "call.name")
        {
            let name = name.node.byte_range();
            if !name.is_empty() && name.end <= bytes.len() {
                named.entry(key).or_insert(CallSite {
                    kind: kind.strip_prefix("call.").unwrap_or("call").into(),
                    name: Some(name),
                    range,
                    owner: None,
                });
            }
        }
    }
    drop(matches);
    if cursor.did_exceed_match_limit() {
        return Err("Call query exceeded its match limit".into());
    }
    for (start, end) in fallback {
        named.entry((start, end)).or_insert(CallSite {
            kind: "dynamic".into(),
            name: None,
            range: start..end,
            owner: None,
        });
    }
    let mut sites: Vec<_> = named.into_values().collect();
    sites.sort_by_key(|site| {
        (
            site.name
                .as_ref()
                .map_or(site.range.start, |range| range.start),
            site.range.start,
        )
    });
    let scopes: Vec<_> = scopes.into_values().collect();
    assign_owners(&mut sites, &scopes);
    Ok((sites, scopes))
}

fn assign_owners(sites: &mut [CallSite], scopes: &[CallableScope]) {
    // Scopes are nested/disjoint parse nodes. Sweep by call start with a stack,
    // avoiding a whole-scope search for every call in a large document.
    let mut order: Vec<_> = (0..sites.len()).collect();
    order.sort_by_key(|&i| sites[i].range.start);
    let mut next = 0;
    let mut active = Vec::<usize>::new();
    for i in order {
        let byte = sites[i].range.start;
        while next < scopes.len() && scopes[next].range.start <= byte {
            while active
                .last()
                .is_some_and(|&index| scopes[index].range.end <= scopes[next].range.start)
            {
                active.pop();
            }
            active.push(next);
            next += 1;
        }
        while active
            .last()
            .is_some_and(|&index| scopes[index].range.end <= byte)
        {
            active.pop();
        }
        sites[i].owner = active.last().copied();
    }
}

#[cfg(test)]
mod tests {
    use crate::source::SourceBuffer;
    use crate::syntax::DocumentSyntax;

    #[test]
    fn shared_call_queries_compile_for_every_linked_grammar_and_preserve_unicode_bytes() {
        for (path, text, expected) in [
            (
                "a.rs",
                "fn f() { let café = 1; obj.send(); unknown(); }",
                vec!["send", "unknown"],
            ),
            (
                "a.py",
                "def f():\n    obj.send()\n    unknown()\n",
                vec!["send", "unknown"],
            ),
            (
                "a.go",
                "package p\nfunc f() { obj.Send(); unknown() }",
                vec!["Send", "unknown"],
            ),
            (
                "a.js",
                "function f() { obj.send(); new Widget(); }",
                vec!["send", "Widget"],
            ),
            (
                "a.ts",
                "function f(): void { obj.send(); unknown(); }",
                vec!["send", "unknown"],
            ),
            (
                "a.tsx",
                "function f() { obj.send(); return <X/>; }",
                vec!["send"],
            ),
            (
                "a.swift",
                "func f() { obj.send(); unknown() }",
                vec!["send", "unknown"],
            ),
            (
                "a.kt",
                "fun f() { obj.send(); unknown() }",
                vec!["send", "unknown"],
            ),
            (
                "a.c",
                "void f() { obj->send(); unknown(); }",
                vec!["send", "unknown"],
            ),
        ] {
            let buffer = SourceBuffer::from_str(path, text);
            let syntax = DocumentSyntax::analyze(&buffer);
            assert!(syntax.calls.supported, "{path}: {:?}", syntax.calls.error);
            let names: Vec<_> = syntax
                .calls
                .sites
                .iter()
                .filter_map(|site| site.name.as_ref().map(|range| &text[range.clone()]))
                .collect();
            assert_eq!(names, expected, "{path}");
            assert!(
                !syntax.calls.scopes.is_empty(),
                "missing callable scope for {path}"
            );
            assert!(syntax.calls.scopes[0].has_body(), "missing body for {path}");
            assert!(
                syntax.calls.sites.iter().all(|site| site.owner == Some(0)),
                "wrong lexical ownership for {path}"
            );
        }
    }

    #[test]
    fn comments_strings_declarations_and_macro_arguments_are_not_guessed_calls() {
        let text = "fn declaration() { // imaginary()\n let s = \"fake()\"; (callee)(); obj.method::<u8>(); log!(nested()); }";
        let syntax = DocumentSyntax::analyze(&SourceBuffer::from_str("a.rs", text));
        let calls = syntax.calls.sites;
        assert_eq!(calls.len(), 3);
        assert!(calls
            .iter()
            .any(|site| site.kind == "dynamic" && site.name.is_none()));
        assert!(calls.iter().any(|site| site.kind == "macro"
            && site
                .name
                .as_ref()
                .is_some_and(|range| &text[range.clone()] == "log")));
        assert!(calls.iter().any(|site| site
            .name
            .as_ref()
            .is_some_and(|range| &text[range.clone()] == "method")));
        assert!(
            !DocumentSyntax::analyze(&SourceBuffer::from_str("a.txt", "f()"))
                .calls
                .supported
        );
    }
}
