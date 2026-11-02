use super::*;
use crate::syntax::{DocumentSyntax, LanguageRegistry};

fn flattened(symbols: &[DocumentSymbol]) -> Vec<&DocumentSymbol> {
    symbols
        .iter()
        .flat_map(|symbol| std::iter::once(symbol).chain(flattened(&symbol.children)))
        .collect()
}

fn declarations(path: &str, source: &str) -> (SourceBuffer, Vec<DocumentSymbol>) {
    let buffer = SourceBuffer::from_str(path, source);
    let tree = LanguageRegistry::for_path(path)
        .grammar
        .expect("native fixture grammar")
        .parse(&buffer.data)
        .expect("native fixture tree");
    assert!(
        !tree.root_node().has_error(),
        "{path}: {}",
        tree.root_node().to_sexp()
    );
    let syntax = DocumentSyntax::analyze(&buffer);
    assert!(syntax.parse_error.is_none() && syntax.fold_error.is_none());
    let symbols = SymbolExtractor::extract(&buffer);
    for symbol in flattened(&symbols) {
        assert_eq!(&source[symbol.selection_range.clone()], symbol.name);
        assert!(
            symbol.range.start <= symbol.selection_range.start
                && symbol.selection_range.end <= symbol.range.end
        );
    }
    (buffer, symbols)
}

#[test]
fn swift_protocol_requirements_and_lifecycle_declarations_have_exact_names() {
    let source = "protocol Reader {\n func café(value: Int) -> Int\n}\nclass Store: Reader {\n init(value: Int) {\n  use(value)\n }\n deinit {\n  clean()\n }\n func café(value: Int) -> Int {\n  value\n }\n}\nextension Store {\n func extra() {}\n}\ntypealias Alias = Store\n";
    let (buffer, symbols) = declarations("Reader.swift", source);
    let rows = flattened(&symbols);
    assert_eq!(
        rows.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
        ["Reader", "café", "Store", "init", "deinit", "café", "Store", "extra", "Alias"]
    );
    for row in rows {
        assert_eq!(&source[row.selection_range.clone()], row.name);
        assert!(
            row.range.start <= row.selection_range.start
                && row.selection_range.end <= row.range.end
        );
    }
    assert_eq!(
        symbols[0].children[0].detail.as_deref(),
        Some("func café(value: Int) -> Int")
    );
    assert!(symbols[1].children.iter().all(DocumentSymbol::is_callable));
    assert_eq!(symbols[1].children[0].kind, SymbolKind::Initializer);
    assert_eq!(symbols[1].children[1].kind, SymbolKind::Deinitializer);
    assert_eq!(symbols[1].children[0].name, "init");
    assert_eq!(symbols[1].children[0].line_count(&buffer), 3);
    assert_eq!(symbols[1].children[1].name, "deinit");
    assert_eq!(symbols[2].kind, SymbolKind::Extension);
    let syntax = DocumentSyntax::analyze(&buffer);
    assert!(syntax.parse_error.is_none() && syntax.fold_error.is_none());
}

#[test]
fn kotlin_outline_matches_swift_query_scope_for_properties_cases_and_constructors() {
    let source = "val top = 1\ntypealias Count = Int\nenum class State {\n Ready,\n Done;\n}\ninterface Reader {\n fun read(value: Int): Int\n}\nclass Store {\n val café: Int = 1\n constructor(value: Int) {\n use(value)\n }\n init {\n use(café)\n }\n companion object Factory {\n fun make() = Store(1)\n }\n fun outer() {\n  val local = 1\n  fun nested() = local\n }\n}\n";
    let (buffer, symbols) = declarations("Reader.kt", source);
    let rows = flattened(&symbols);
    assert_eq!(
        rows.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
        [
            "top",
            "Count",
            "State",
            "Ready",
            "Done",
            "Reader",
            "read",
            "Store",
            "café",
            "constructor",
            "init",
            "Factory",
            "make",
            "outer",
            "nested"
        ]
    );
    for row in rows {
        assert_eq!(&source[row.selection_range.clone()], row.name);
    }
    assert_eq!(symbols[0].kind, SymbolKind::Property);
    assert_eq!(symbols[2].kind, SymbolKind::Enum);
    assert!(symbols[2]
        .children
        .iter()
        .all(|symbol| symbol.kind == SymbolKind::EnumMember));
    assert_eq!(symbols[3].kind, SymbolKind::Interface);
    let store = symbols.iter().find(|s| s.name == "Store").unwrap();
    assert_eq!(store.children[0].kind, SymbolKind::Property);
    assert_eq!(store.children[1].kind, SymbolKind::Initializer);
    assert_eq!(store.children[2].kind, SymbolKind::Initializer);
    assert_eq!(
        store
            .children
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>(),
        ["café", "constructor", "init", "Factory", "outer"]
    );
    let factory = store.children.iter().find(|s| s.name == "Factory").unwrap();
    assert_eq!(factory.children[0].name, "make");
    let syntax = DocumentSyntax::analyze(&buffer);
    assert!(syntax.parse_error.is_none() && syntax.fold_error.is_none());
}

#[test]
fn query_names_keep_escaped_identifier_bytes_and_cancellation_rejects_partial_facts() {
    for (path, source, expected) in [
        ("Escaped.kt", "fun `spaced name`() = 1\n", "`spaced name`"),
        ("Escaped.swift", "func `repeat`() {}\n", "`repeat`"),
    ] {
        let (buffer, symbols) = declarations(path, source);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, expected);
        assert_eq!(&buffer.text[symbols[0].selection_range.clone()], expected);
    }
    let source = (0..3000)
        .map(|i| format!("val value{i} = {i}\n"))
        .collect::<String>();
    let buffer = SourceBuffer::from_str("Many.kt", &source);
    let tree = SupportedGrammar::Kotlin.parse(&buffer.data).unwrap();
    let checks = std::sync::atomic::AtomicUsize::new(0);
    let cancelled = || checks.fetch_add(1, std::sync::atomic::Ordering::Relaxed) > 50;
    let control = crate::syntax::AnalysisControl::new(&cancelled);
    assert!(native::extract(
        &buffer,
        SupportedGrammar::Kotlin,
        tree.root_node(),
        &control
    )
    .is_err());
    assert_eq!(SymbolExtractor::extract(&buffer).len(), 3000);
}

#[test]
fn rust_query_declarations_include_trait_requirements_unions_and_macros() {
    let source = "trait Reader { fn read(&self) -> usize; }\nunion Bits { value: u32, bytes: [u8; 4] }\nmacro_rules! make { () => {} }\nstatic VALUE: usize = 1;\nimpl<T> Reader for Box<T> { fn read(&self) -> usize { 1 } }\n";
    let (_, symbols) = declarations("reader.rs", source);
    let rows = flattened(&symbols);
    assert_eq!(
        rows.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
        ["Reader", "read", "Bits", "make", "VALUE", "Box<T>", "read"]
    );
    assert_eq!(symbols[1].kind, SymbolKind::Union);
    assert_eq!(symbols[2].kind, SymbolKind::Macro);
    assert!(symbols[0].children[0].is_callable());
    assert_eq!(
        symbols[0].children[0].detail.as_deref(),
        Some("fn read(&self) -> usize;")
    );
    assert_eq!(symbols[4].kind, SymbolKind::Extension);
    assert_eq!(&source[symbols[4].selection_range.clone()], "Box<T>");
}

#[test]
fn ecmascript_queries_keep_named_closures_and_exclude_local_values() {
    let source = "export const café = (value) => { return value; };\nconst run = function() {};\nfunction* values() { yield 1; }\nlet top = 1;\nfunction outer() { let local = 2; const nested = () => local; }\n";
    let (_, symbols) = declarations("reader.js", source);
    let rows = flattened(&symbols);
    assert_eq!(
        rows.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
        ["café", "run", "values", "top", "outer", "nested"]
    );
    assert!(symbols[0].is_callable() && symbols[1].is_callable());
    assert_eq!(symbols[0].detail.as_deref(), Some("café = (value) =>"));
    assert_eq!(symbols[1].detail.as_deref(), Some("run = function()"));
    assert_eq!(symbols[3].kind, SymbolKind::Variable);
}

#[test]
fn typescript_queries_add_signatures_properties_enums_and_modules() {
    let source = "interface Reader { value: number; read(): number; }\nabstract class Base { abstract run(): void; field: number = 1; }\nenum State { Ready, Done = 2 }\ndeclare function fetch(value: string): void;\nnamespace Inner { export const top = 1; }\nconst view = () => <div/>;\n";
    let (_, symbols) = declarations("reader.tsx", source);
    let rows = flattened(&symbols);
    assert_eq!(
        rows.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
        [
            "Reader", "value", "read", "Base", "run", "field", "State", "Ready", "Done", "fetch",
            "Inner", "top", "view"
        ]
    );
    assert_eq!(symbols[0].kind, SymbolKind::Interface);
    assert_eq!(symbols[0].children[0].kind, SymbolKind::Property);
    assert!(symbols[0].children[1].is_callable());
    assert_eq!(symbols[2].kind, SymbolKind::Enum);
    assert_eq!(symbols[2].children.len(), 2);
    assert!(symbols[2]
        .children
        .iter()
        .all(|case| case.children.is_empty()));
    assert_eq!(&source[symbols[2].children[0].range.clone()], "Ready");
    assert_eq!(symbols[4].kind, SymbolKind::Module);
    let structure = "interface Reader {\n value: number;\n}\ntype Shape = {\n field: string;\n};\nenum State {\n Ready,\n Done\n}\nnamespace Inner {\n export const top = 1;\n}\n";
    for path in ["reader.ts", "reader.tsx"] {
        let buffer = SourceBuffer::from_str(path, structure);
        let syntax = DocumentSyntax::analyze(&buffer);
        assert!(syntax.parse_error.is_none() && syntax.fold_error.is_none());
        assert_eq!(
            syntax
                .folds
                .iter()
                .filter(|fold| fold.kind == crate::syntax::FoldKind::Type)
                .count(),
            4
        );
    }
    let syntax = DocumentSyntax::analyze(&SourceBuffer::from_str("reader.tsx", source));
    assert!(syntax.parse_error.is_none() && syntax.fold_error.is_none());
}

#[test]
fn go_queries_preserve_package_interface_and_file_scope_declarations() {
    let source = "package reader\ntype Reader interface { Read() int }\ntype Store struct { Value int }\ntype Count = int\nconst Limit = 1\nvar Value = 2\nfunc outer() { var local = 3; _ = local }\n";
    let (_, symbols) = declarations("reader.go", source);
    let rows = flattened(&symbols);
    assert_eq!(
        rows.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
        ["reader", "Reader", "Read", "Store", "Count", "Limit", "Value", "outer"]
    );
    assert_eq!(symbols[0].kind, SymbolKind::Module);
    assert_eq!(symbols[1].kind, SymbolKind::Interface);
    assert_eq!(symbols[2].kind, SymbolKind::Struct);
    assert_eq!(symbols[3].kind, SymbolKind::TypeAlias);
}

#[test]
fn python_queries_include_module_values_and_type_aliases() {
    let source =
        "top = 1\ntype Count = int\nclass Reader:\n def read(self):\n  local = 1\n  return local\n";
    let (_, symbols) = declarations("reader.py", source);
    assert_eq!(
        flattened(&symbols)
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>(),
        ["top", "Count", "Reader", "read"]
    );
    assert_eq!(symbols[0].kind, SymbolKind::Variable);
    assert_eq!(symbols[1].kind, SymbolKind::TypeAlias);
}

#[test]
fn kotlin_compact_members_keep_clean_trees_and_exact_declaration_ranges() {
    for (source, expected) in [
        (
            "interface Reader { fun read(value: Int): Int }",
            vec!["Reader", "read"],
        ),
        (
            "class Reader { fun read(value: Int): Int { return value } }",
            vec!["Reader", "read"],
        ),
        ("class Reader { val café = 1 }", vec!["Reader", "café"]),
        (
            "enum class Reader { A; fun read() = 1 }",
            vec!["Reader", "A", "read"],
        ),
        (
            "class Reader { class Inner { val value = 1 } }",
            vec!["Reader", "Inner", "value"],
        ),
        (
            "interface Reader { fun read(): Int; fun two(): Int }",
            vec!["Reader", "read", "two"],
        ),
        (
            "class Reader { val café = 1 /* final member */ }",
            vec!["Reader", "café"],
        ),
        (
            "class Reader { fun run() {\n call()\n} }",
            vec!["Reader", "run"],
        ),
    ] {
        for crlf in [false, true] {
            let source = if crlf {
                source.replace('\n', "\r\n")
            } else {
                source.to_owned()
            };
            let (buffer, symbols) = declarations("Reader.kt", &source);
            assert_eq!(
                flattened(&symbols)
                    .iter()
                    .map(|symbol| symbol.name.as_str())
                    .collect::<Vec<_>>(),
                expected
            );
            let syntax = DocumentSyntax::analyze(&buffer);
            assert!(!syntax.calls.parse_errors && syntax.folds_supported);
            if source.contains("call()") {
                assert_eq!(syntax.calls.sites.len(), 1);
                assert!(syntax
                    .folds
                    .iter()
                    .any(|fold| fold.kind == crate::syntax::FoldKind::Callable));
            }
        }
    }
}

#[test]
fn kotlin_compact_member_recovery_still_reports_actual_syntax_errors() {
    for source in [
        "interface Reader { fun read(value: ): Int }",
        "interface Reader { fun read(value: Int): Int",
        "class Reader { val value = }",
        "class Reader { fun run() { call( } }",
        "interface Reader { fun read(): Int fun two(): Int }",
    ] {
        let buffer = SourceBuffer::from_str("Broken.kt", source);
        let syntax = DocumentSyntax::analyze(&buffer);
        assert!(
            syntax.calls.parse_errors,
            "must retain diagnostics for {source}"
        );
    }
}

#[test]
fn kotlin_pinned_upstream_corpus_keeps_its_expected_named_trees() {
    let header = regex::Regex::new(r"(?m)^={3,}\n([^\n]+)\n={3,}\n").unwrap();
    let field = regex::Regex::new(r"\b[a-z_]+:\s+").unwrap();
    let compact = |value: &str| {
        value
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
    };
    let mut count = 0;
    for (name, fixture) in [
        (
            "declarations",
            include_str!("../../../vendor/tree-sitter-kotlin-ng/test/corpus/declarations.txt"),
        ),
        (
            "expressions",
            include_str!("../../../vendor/tree-sitter-kotlin-ng/test/corpus/expressions.txt"),
        ),
        (
            "semi",
            include_str!("../../../vendor/tree-sitter-kotlin-ng/test/corpus/semi.txt"),
        ),
    ] {
        let cases = header.captures_iter(fixture).collect::<Vec<_>>();
        assert!(!cases.is_empty(), "{name}");
        for (index, case) in cases.iter().enumerate() {
            let end = cases
                .get(index + 1)
                .map_or(fixture.len(), |next| next.get(0).unwrap().start());
            let (source, expected) = fixture[case.get(0).unwrap().end()..end]
                .split_once("\n---\n")
                .unwrap();
            for crlf in [false, true] {
                let source = if crlf {
                    source.replace('\n', "\r\n")
                } else {
                    source.to_owned()
                };
                let tree = crate::syntax::SupportedGrammar::Kotlin
                    .parse(source.as_bytes())
                    .unwrap();
                let actual = tree.root_node().to_sexp();
                assert_eq!(
                    compact(&field.replace_all(&actual, "")),
                    compact(expected),
                    "{name}/{} CRLF={crlf}",
                    &case[1]
                );
            }
            count += 1;
        }
    }
    assert_eq!(count, 22, "pinned upstream corpus must retain every case");
    println!("accepted {count} pinned Kotlin corpus cases in LF and CRLF");
}
