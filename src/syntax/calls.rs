//! Syntactic call sites shared by Critical Lens and future call graph queries.

use super::SupportedGrammar;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ops::Range;
use std::sync::OnceLock;
use tree_sitter::{Node, Query, QueryCursor, StreamingIterator};

/// A parsed call expression; syntax alone does not establish a semantic edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallSite {
    /// `call`, `macro`, or `dynamic` for a callee without a supported name token.
    pub kind: String,
    /// Exact original-byte callee name, when supported by the grammar.
    pub name: Option<Range<usize>>,
    /// Full call or macro expression in original bytes.
    pub range: Range<usize>,
}

/// Call-query coverage for one exact document parse.
#[derive(Debug, Clone, Default)]
pub struct CallSites {
    /// Complete source-ordered sites, including unresolved/dynamic callees.
    pub sites: Vec<CallSite>,
    /// False for unsupported grammar/query or a bounded query failure.
    pub supported: bool,
    /// Query compilation/limit failure, distinct from an unsupported language.
    pub error: Option<String>,
    path: String,
    revision: Option<crate::source::ReviewRevision>,
    content_digest: String,
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
            Ok(sites) => Self {
                sites,
                supported: true,
                error: None,
                path: buffer.path.clone(),
                revision: buffer.revision,
                content_digest: buffer.content_digest().to_owned(),
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
            && self.path == buffer.path
            && self.revision == buffer.revision
            && self.content_digest == buffer.content_digest()
    }
}

fn query(grammar: SupportedGrammar) -> Result<&'static Query, String> {
    static QUERIES: OnceLock<HashMap<SupportedGrammar, Result<Query, String>>> = OnceLock::new();
    QUERIES.get_or_init(|| {
        use SupportedGrammar::*;
        [
            (Rust, include_str!("../../Sources/Radiology/Queries/rust/calls.scm")),
            (Python, include_str!("../../Sources/Radiology/Queries/python/calls.scm")),
            (Go, include_str!("../../Sources/Radiology/Queries/go/calls.scm")),
            (JavaScript, include_str!("../../Sources/Radiology/Queries/ecmascript/calls.scm")),
            (TypeScript, include_str!("../../Sources/Radiology/Queries/ecmascript/calls.scm")),
            (Tsx, include_str!("../../Sources/Radiology/Queries/ecmascript/calls.scm")),
            (Swift, include_str!("../../Sources/Radiology/Queries/swift/calls.scm")),
            (Kotlin, // The Rust port links kotlin-ng, whose node names differ from Swift's grammar.
                    "(call_expression . (identifier) @call.name) @call\n(call_expression (navigation_expression (identifier) @call.name .)) @call\n(call_expression) @call.any"),
            (C, "(call_expression function: (identifier) @call.name) @call\n(call_expression function: (field_expression field: (field_identifier) @call.name)) @call\n(call_expression) @call.any"),
        ].into_iter().map(|(grammar, source)| {
            (grammar, Query::new(&grammar.language(), source).map_err(|error| error.to_string()))
        }).collect()
    }).get(&grammar).expect("registered call grammar").as_ref().map_err(Clone::clone)
}

fn extract(
    grammar: SupportedGrammar,
    root: Node<'_>,
    bytes: &[u8],
) -> Result<Vec<CallSite>, String> {
    let query = query(grammar)?;
    let names = query.capture_names();
    let mut cursor = QueryCursor::new();
    cursor.set_match_limit(65_536);
    let mut named = BTreeMap::new();
    let mut fallback = BTreeSet::new();
    let mut matches = cursor.matches(query, root, bytes);
    while let Some(found) = matches.next() {
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
    Ok(sites)
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
