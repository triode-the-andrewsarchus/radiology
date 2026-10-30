use super::*;
use crate::syntax::{DocumentSyntax, FoldStore, SupportedGrammar};

#[test]
fn shared_fold_and_declaration_queries_compile_for_every_linked_grammar() {
    for (path, text) in [
        (
            "a.rs",
            "struct A {\n x: i32,\n}\nimpl A {\n fn run(&self) {\n one();\n two();\n }\n}\n",
        ),
        (
            "a.py",
            "class A:\n    def run(self):\n        one()\n        two()\n",
        ),
        (
            "a.go",
            "package p\ntype A struct {\n x int\n}\nfunc run() {\n one()\n two()\n}\n",
        ),
        ("a.js", "class A {\n run() {\n one();\n two();\n }\n}\n"),
        (
            "a.ts",
            "class A {\n run(): void {\n one();\n two();\n }\n}\n",
        ),
        (
            "a.tsx",
            "class A {\n run() {\n one();\n return <X/>;\n }\n}\n",
        ),
        (
            "a.swift",
            "class A {\n func run() {\n one()\n two()\n }\n}\n",
        ),
        (
            "a.kt",
            "class A {\n val value = 1\n fun run() {\n one()\n two()\n }\n}\n",
        ),
        (
            "a.c",
            "struct A {\n int x;\n int y;\n};\nvoid run(void) {\n one();\n two();\n}\n",
        ),
    ] {
        let buffer = SourceBuffer::from_str(path, text);
        let syntax = DocumentSyntax::analyze(&buffer);
        assert!(syntax.folds_supported, "{path}: {:?}", syntax.fold_error);
        assert!(syntax.calls.supported, "independent calls: {path}");
        assert!(
            syntax
                .folds
                .iter()
                .any(|fold| fold.kind == FoldKind::Callable),
            "{path}"
        );
        for fold in &syntax.folds {
            assert!(
                fold.byte_range.start < fold.byte_range.end && fold.byte_range.end <= text.len(),
                "{path}"
            );
            assert!(fold.start_line < fold.end_line, "{path}");
        }
    }
}

pub(crate) const DEFINITIONS: &str = "struct Leaf {\n x: i32,\n y: i32,\n}\nimpl Leaf {\n fn method(&self) {\n act();\n }\n}\nenum Choice {\n One,\n Two,\n}\nmod nested {\n fn visible() {\n act();\n }\n}\n/* real comment\n still comment\n*/\nuse crate::{\n A,\n B,\n};\n";

#[test]
fn fold_to_definitions_keeps_nested_declarations_visible_and_replaces_unrelated_folds() {
    let buffer = SourceBuffer::from_str("a.rs", DEFINITIONS);
    let syntax = DocumentSyntax::analyze(&buffer);
    assert!(syntax.folds_supported, "{:?}", syntax.fold_error);
    let at = |line| {
        syntax
            .folds
            .iter()
            .find(|fold| fold.start_line == line)
            .unwrap()
    };
    assert_eq!(at(1).kind, FoldKind::Type);
    assert!(
        !at(1).contains_declarations,
        "struct fields are not outline declarations in Swift's Rust queries"
    );
    for line in [5, 10, 14] {
        assert_eq!(at(line).kind, FoldKind::Type);
        assert!(at(line).contains_declarations, "container at {line}");
    }
    assert_eq!(at(19).kind, FoldKind::Comment);
    assert_eq!(at(22).kind, FoldKind::Imports);
    let mut store = FoldStore::new();
    store.activate(&buffer, &syntax.folds);
    store
        .folded_ranges
        .extend(syntax.folds.iter().map(|fold| fold.byte_range.clone()));
    store.fold_to_definitions(&syntax.folds);
    assert_eq!(
        store.collapsed_headers(&syntax.folds),
        std::collections::HashSet::from([1, 6, 15])
    );
    assert!(store.is_line_hidden(2, &syntax.folds));
    for line in [6, 11, 12, 15, 19, 20, 22, 23] {
        assert!(
            !store.is_line_hidden(line, &syntax.folds),
            "declaration/comment/import line {line}"
        );
    }
    assert!(store.is_line_hidden(7, &syntax.folds));
    assert!(store.is_line_hidden(16, &syntax.folds));
}

#[test]
fn parsed_comments_exclude_string_impostors_and_callable_role_wins_over_block() {
    let text="const TEXT: &str = r#\"/*\n fake comment\n*/\"#;\n/* real\n comment\n*/\nfn café() {\n if ready() {\n act();\n }\n}\n";
    let buffer = SourceBuffer::from_str("a.rs", text);
    let syntax = DocumentSyntax::analyze(&buffer);
    assert!(syntax.folds_supported);
    let comments: Vec<_> = syntax
        .folds
        .iter()
        .filter(|fold| fold.kind == FoldKind::Comment)
        .collect();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].byte_range.start, text.find("/* real").unwrap());
    let body = syntax
        .folds
        .iter()
        .find(|fold| fold.start_line == buffer.line_at_byte(text.find("fn café").unwrap()))
        .unwrap();
    assert_eq!(body.kind, FoldKind::Callable);
    assert_eq!(
        &text[body.byte_range.clone()],
        "{\n if ready() {\n act();\n }\n}"
    );
    assert!(syntax.folds.iter().any(|fold| fold.kind == FoldKind::Block));
}

#[test]
fn type_containment_includes_kotlin_properties_and_rust_enum_cases_from_shared_symbol_queries() {
    let buffer = SourceBuffer::from_str("a.kt", "class A {\n val value = 1\n val other = 2\n}\n");
    let syntax = DocumentSyntax::analyze(&buffer);
    assert!(syntax.folds_supported, "{:?}", syntax.fold_error);
    let class = syntax
        .folds
        .iter()
        .find(|fold| fold.kind == FoldKind::Type)
        .unwrap();
    assert!(class.contains_declarations);
    let mut store = FoldStore::new();
    store.fold_to_definitions(&syntax.folds);
    assert!(
        store.collapsed_headers(&syntax.folds).is_empty(),
        "property declarations must stay visible"
    );
}

#[test]
fn folding_fact_limits_do_not_return_partial_fold_facts_or_disable_independent_calls() {
    let buffer = SourceBuffer::from_str("a.rs", DEFINITIONS);
    let grammar = SupportedGrammar::Rust;
    let tree = grammar.parse(&buffer.data).unwrap();
    let query =
        crate::syntax::queries::query(grammar, crate::syntax::queries::QueryRole::Folds).unwrap();
    assert!(extract(&buffer, tree.root_node(), query, 1)
        .unwrap_err()
        .contains("fact limit"));
    let calls = crate::syntax::calls::CallSites::from_tree(grammar, tree.root_node(), &buffer);
    assert!(calls.supported);
    assert!(!calls.sites.is_empty());
    assert_eq!(buffer.text, DEFINITIONS);
}

#[test]
fn kotlin_lambda_header_changes_do_not_open_sibling_closures() {
    use crate::{
        pull_request::diff::{DiffHunk, FileLineChanges},
        source::ReviewRevision,
    };
    let text="fun parent() {\n pair(\n  { first();\n    more();\n  },\n  { second();\n    more();\n  }\n )\n}\n";
    let buffer = SourceBuffer::from_bytes(
        "a.kt",
        text.as_bytes().to_vec(),
        Some(ReviewRevision::After),
    )
    .unwrap();
    let syntax = DocumentSyntax::analyze(&buffer);
    assert!(syntax.folds_supported, "{:?}", syntax.fold_error);
    let changes = FileLineChanges {
        hunks: vec![DiffHunk {
            before_start: 3,
            before_count: 1,
            after_start: 3,
            after_count: 1,
        }],
        ..Default::default()
    };
    let mut store = FoldStore::new();
    store.apply_initial_defaults(&buffer, &syntax.folds, Some(&changes));
    assert!(!store.is_header_folded(1, &syntax.folds));
    assert!(!store.is_header_folded(3, &syntax.folds));
    assert!(store.is_header_folded(6, &syntax.folds));
}

#[test]
fn kotlin_import_lists_group_parsed_members_with_comments_and_exact_unicode_crlf_bounds() {
    let text = "// leading café 😀\r\npackage demo\r\n\r\nimport demo.First as Premier\r\n// between 😀\r\n\r\nimport demo.second.*;\r\n/* between */\r\nimport demo.Third\r\n// trailing: must remain visible\r\nfun run() {\r\n work()\r\n}\r\n";
    let buffer = SourceBuffer::from_str("example.kt", text);
    let syntax = DocumentSyntax::analyze(&buffer);
    assert!(syntax.folds_supported, "{:?}", syntax.fold_error);
    let imports: Vec<_> = syntax
        .folds
        .iter()
        .filter(|fold| fold.kind == FoldKind::Imports)
        .collect();
    assert_eq!(imports.len(), 1);
    let fold = imports[0];
    assert_eq!(
        fold.byte_range,
        text.find("import").unwrap()..text.find("\r\n// trailing").unwrap()
    );
    assert_eq!((fold.start_line, fold.end_line), (4, 9));
    assert!(!fold.contains_declarations);
    assert!(fold.owner_range.is_none());
    let mut store = FoldStore::new();
    store.apply_initial_defaults(&buffer, &syntax.folds, None);
    assert!(
        !store.is_header_folded(4, &syntax.folds),
        "imports stay open by default"
    );
    store.toggle_header(4, &syntax.folds);
    for line in 5..=9 {
        assert!(store.is_line_hidden(line, &syntax.folds));
    }
    for line in [1, 2, 3, 4, 10, 11] {
        assert!(!store.is_line_hidden(line, &syntax.folds));
    }
    store.fold_to_definitions(&syntax.folds);
    assert!(!store.is_header_folded(4, &syntax.folds));
    assert!(store.is_header_folded(11, &syntax.folds));
    assert_eq!(buffer.text, text);
}

#[test]
fn kotlin_import_grouping_does_not_turn_strings_single_lines_or_errors_into_import_lists() {
    for text in [
        "import demo.One\nval text = \"import demo.Fake\\nimport demo.Other\"\n",
        "val text = \"import demo.One\\nimport demo.Two\"\n",
    ] {
        let buffer = SourceBuffer::from_str("a.kt", text);
        let syntax = DocumentSyntax::analyze(&buffer);
        assert!(syntax.folds_supported, "{:?}", syntax.fold_error);
        assert!(!syntax
            .folds
            .iter()
            .any(|fold| fold.kind == FoldKind::Imports));
    }
    let buffer =
        SourceBuffer::from_str("a.kt", "import demo.One\n!!! broken !!!\nimport demo.Two\n");
    let syntax = DocumentSyntax::analyze(&buffer);
    assert!(!syntax
        .folds
        .iter()
        .any(|fold| fold.kind == FoldKind::Imports));
    let grammar = SupportedGrammar::Kotlin;
    let buffer = SourceBuffer::from_str(
        "a.kt",
        "import demo.One\nimport demo.Two\nimport demo.Three\n",
    );
    let tree = grammar.parse(&buffer.data).unwrap();
    let query =
        crate::syntax::queries::query(grammar, crate::syntax::queries::QueryRole::Folds).unwrap();
    assert!(extract(&buffer, tree.root_node(), query, 2)
        .unwrap_err()
        .contains("fact limit"));
}

#[test]
fn same_row_nested_folds_keep_independent_byte_identities_across_defaults_and_restore() {
    let text = "impl Demo { fn first() {\n one();\n }\n fn second() {\n two();\n }\n}\n";
    let buffer = SourceBuffer::from_str("a.rs", text);
    let syntax = DocumentSyntax::analyze(&buffer);
    let outer = syntax
        .folds
        .iter()
        .find(|fold| fold.kind == FoldKind::Type)
        .unwrap();
    let first = syntax
        .folds
        .iter()
        .find(|fold| fold.kind == FoldKind::Callable)
        .unwrap();
    let second = syntax
        .folds
        .iter()
        .find(|fold| fold.kind == FoldKind::Callable && fold.start_line == 4)
        .unwrap();
    assert_eq!(outer.start_line, first.start_line);
    assert_eq!(
        syntax.folds[0], *outer,
        "gutter chooses Swift's first outer range"
    );
    let identities: std::collections::HashSet<_> = syntax
        .folds
        .iter()
        .map(|fold| fold.byte_range.clone())
        .collect();
    assert_eq!(
        identities.len(),
        syntax.folds.len(),
        "exact duplicate roles remain canonicalized"
    );
    let review = SourceBuffer::from_bytes(
        "a.rs",
        buffer.data.clone(),
        Some(crate::source::ReviewRevision::After),
    )
    .unwrap();
    let changes = crate::pull_request::diff::FileLineChanges {
        hunks: vec![crate::pull_request::diff::DiffHunk {
            before_start: 2,
            before_count: 1,
            after_start: 2,
            after_count: 1,
        }],
        ..Default::default()
    };
    let expanded = crate::syntax::review_folds::expanded_ranges(
        &review,
        &syntax.folds,
        &changes,
        crate::source::ReviewRevision::After,
    );
    assert!(expanded.contains(&outer.byte_range));
    assert!(expanded.contains(&first.byte_range));
    assert!(!expanded.contains(&second.byte_range));
    let mut review_store = FoldStore::new();
    review_store.apply_initial_defaults(&review, &syntax.folds, Some(&changes));
    assert_eq!(
        review_store.folded_ranges,
        std::collections::HashSet::from([second.byte_range.clone()])
    );
    let mut store = FoldStore::new();
    store.activate(&buffer, &syntax.folds);
    store.apply_initial_defaults(&buffer, &syntax.folds, None);
    assert!(!store.is_folded(&outer.byte_range));
    assert!(store.is_folded(&first.byte_range));
    assert!(store.is_folded(&second.byte_range));
    assert!(!store.is_line_hidden(4, &syntax.folds));
    store.toggle(outer.byte_range.clone());
    assert!(store.is_line_hidden(4, &syntax.folds));
    store.toggle(first.byte_range.clone());
    let saved = store.folded_ranges.clone();
    let other = SourceBuffer::from_str("b.rs", "fn other() {\n work();\n}\n");
    let other_syntax = DocumentSyntax::analyze(&other);
    store.activate(&other, &other_syntax.folds);
    assert!(store.folded_ranges.is_empty());
    store.activate(&buffer, &syntax.folds);
    assert_eq!(store.folded_ranges, saved);
    store.toggle(outer.byte_range.clone());
    assert!(!store.is_line_hidden(2, &syntax.folds));
    assert!(store.is_line_hidden(5, &syntax.folds));
    store.fold_to_definitions(&syntax.folds);
    assert!(!store.is_folded(&outer.byte_range));
    assert!(store.is_folded(&first.byte_range));
}
