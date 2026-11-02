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
    let symbols = SymbolExtractor::extract(&buffer);
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
