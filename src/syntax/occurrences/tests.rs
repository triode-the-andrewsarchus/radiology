use super::*;
use crate::source::ReviewRevision;

#[test]
fn occurrence_queries_compile_for_every_native_grammar() {
    for grammar in [
        SupportedGrammar::Rust,
        SupportedGrammar::Python,
        SupportedGrammar::Go,
        SupportedGrammar::JavaScript,
        SupportedGrammar::TypeScript,
        SupportedGrammar::Tsx,
        SupportedGrammar::Swift,
        SupportedGrammar::Kotlin,
        SupportedGrammar::C,
    ] {
        assert!(
            super::super::queries::query(grammar, super::super::queries::QueryRole::Occurrences)
                .is_ok(),
            "{grammar:?}: {:?}",
            super::super::queries::query(grammar, super::super::queries::QueryRole::Occurrences)
                .err()
        );
    }
}

#[test]
fn syntax_exclusions_and_unicode_crlf_use_exact_spelling_and_callable_boundaries() {
    let text = "const café: i32 = 0;\r\nfn first(café: i32) { café; let cafés = 1; // café\r\n let text = \"café\"; let raw = r#\"café\"#; café;\r\n}\r\nfn other(café: i32) { café; }\r\n";
    let buffer = SourceBuffer::from_str("a.rs", text);
    let syntax = super::super::DocumentSyntax::analyze(&buffer);
    assert!(
        syntax.occurrences.error.is_none(),
        "{:?}",
        syntax.occurrences.error
    );
    let byte = text.find("first(café").unwrap() + "first(".len();
    let selection = syntax.occurrences.selection(&buffer, byte).unwrap();
    assert_eq!(selection.ranges.len(), 3);
    assert!(selection
        .ranges
        .iter()
        .all(|range| &text[range.clone()] == "café"));
    assert_eq!(selection.scope.start, text.find("fn first").unwrap());
    for byte in [
        text.find("const café").unwrap() + 6,
        text.find("// café").unwrap() + 3,
        text.find("\"café\"").unwrap() + 1,
        text.find("r#\"café").unwrap() + 3,
        text.find("let text").unwrap(),
        text.find("= 1").unwrap() + 2,
    ] {
        assert!(
            syntax.occurrences.selection(&buffer, byte).is_none(),
            "byte {byte}"
        );
    }
    assert!(syntax
        .occurrences
        .selection(&SourceBuffer::from_str("other.rs", text), byte)
        .is_none());
    assert!(syntax
        .occurrences
        .selection(
            &SourceBuffer::from_str("a.rs", &(text.to_owned() + " ")),
            byte
        )
        .is_none());
    let before =
        SourceBuffer::from_bytes("a.rs", buffer.data.clone(), Some(ReviewRevision::Before))
            .unwrap();
    assert!(syntax.occurrences.selection(&before, byte).is_none());
}

#[test]
fn innermost_function_and_closure_groups_are_decorations_not_shadow_binding_resolution() {
    let text = "fn outer(value:i32) { value; { let value=0; value; } let f=|value| value; fn inner(value:i32) { value; } }";
    let buffer = SourceBuffer::from_str("a.rs", text);
    let facts = super::super::DocumentSyntax::analyze(&buffer).occurrences;
    let outer = facts
        .selection(&buffer, text.find("value").unwrap())
        .unwrap();
    assert_eq!(
        outer.ranges.len(),
        8,
        "shadowed spellings intentionally remain one decoration"
    );
    let closure = facts
        .selection(&buffer, text.find("|value").unwrap() + 1)
        .unwrap();
    assert_eq!(closure.ranges.len(), 2);
    assert!(closure.scope.len() < outer.scope.len());
    let inner = facts
        .selection(&buffer, text.find("inner(value").unwrap() + "inner(".len())
        .unwrap();
    assert_eq!(inner.ranges.len(), 2);
    assert!(inner.scope.len() < outer.scope.len());
}

#[test]
fn native_grammar_groups_exclude_comments_strings_numbers_and_keywords() {
    for (path, text, comment) in [
        ("a.py", "def f(value):\n value\n # value\n text = 'value'\n value\n", "# value"),
        ("a.go", "package p\nfunc f(value int) {\n value; // value\n text := \"value\"; value\n}\n", "// value"),
        ("a.js", "function f(value) { value; // value\n const text = `value`; value; }", "// value"),
        ("a.ts", "function f(value: number) { value; // value\n const text = 'value'; value; }", "// value"),
        ("a.tsx", "function f(value: number) { value; // value\n const text = 'value'; value; return <X/>; }", "// value"),
        ("a.swift", "func f(value: Int) { value; // value\n let text = \"value\"; value }", "// value"),
        ("a.kt", "fun f(value: Int) { value; // value\n val text = \"value\"; value }", "// value"),
        ("a.c", "void f(int value) { value; // value\n char *text = \"value\"; value; }", "// value"),
    ] {
        let buffer = SourceBuffer::from_str(path, text);
        let facts = super::super::DocumentSyntax::analyze(&buffer).occurrences;
        assert!(facts.error.is_none(), "{path}: {:?}", facts.error);
        let selection = facts.selection(&buffer, text.find("value").unwrap()).unwrap();
        assert_eq!(selection.ranges.len(), 3, "{path}");
        assert!(facts.selection(&buffer, text.find(comment).unwrap() + comment.len() - 3).is_none(), "{path}");
    }
}

#[test]
fn occurrence_limits_fail_without_partial_groups_or_disabling_independent_calls() {
    let buffer = SourceBuffer::from_str("a.rs", "fn f(value:i32) { value; call(); }\n");
    let grammar = SupportedGrammar::Rust;
    let tree = grammar.parse(&buffer.data).unwrap();
    assert!(extract(
        &buffer,
        grammar,
        tree.root_node(),
        1,
        &crate::syntax::AnalysisControl::new(&|| false)
    )
    .unwrap_err()
    .contains("fact limit"));
    let calls = super::super::calls::CallSites::from_tree(grammar, tree.root_node(), &buffer);
    assert!(calls.supported);
    assert_eq!(calls.sites.len(), 1);
    let unsupported =
        super::super::DocumentSyntax::analyze(&SourceBuffer::from_str("a.txt", "value value"));
    assert!(unsupported.occurrences.selection(&buffer, 0).is_none());
}
