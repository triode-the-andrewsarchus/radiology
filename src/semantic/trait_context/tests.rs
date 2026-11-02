use super::*;
use crate::semantic::{ScipDocument, ScipIndex, ScipSymbolInformation};
use std::sync::atomic::Ordering;

fn resolve(buffer: &SourceBuffer, index: &SemanticIndex, byte: usize) -> Option<SemanticSymbolID> {
    trait_at_byte(
        buffer,
        &DocumentSyntax::analyze(buffer),
        index,
        byte,
        &AtomicBool::new(false),
    )
    .unwrap()
}

#[test]
fn compiler_fixture_context_follows_traits_members_alias_and_qualified_impl_bodies() {
    let (_owner, storage, raw, _) = crate::semantic::trait_impls::tests::fixture();
    let crate::source::loader::SourceRead::Checkout(root) = storage else {
        unreachable!()
    };
    let index = SemanticIndex::from_scip(raw);
    let lib = SourceBuffer::from_str(
        "src/lib.rs",
        &std::fs::read_to_string(root.join("src/lib.rs")).unwrap(),
    );
    let implementations = SourceBuffer::from_str(
        "src/implementations.rs",
        &std::fs::read_to_string(root.join("src/implementations.rs")).unwrap(),
    );
    for (buffer, text, suffix) in [
        (&lib, "Render<A", "left/Render#"),
        (&lib, "fn render", "left/Render#"),
        (&lib, "{ 1 }", "left/Render#"),
        (&lib, "Empty {}", "left/Empty#"),
        (&implementations, "Alias for", "left/Render#"),
        (&implementations, "{ 2 }", "left/Render#"),
        (&implementations, "where T", "left/Render#"),
        (&implementations, "Other {}", "right/Render#"),
        (&implementations, "Negative {}", "left/Marker#"),
    ] {
        let id = resolve(buffer, &index, buffer.text.find(text).unwrap()).unwrap();
        assert!(id.symbol.ends_with(suffix), "{text}: {}", id.symbol);
    }
    for text in ["pub struct Alpha", "pub fn bounds", "fn inherent"] {
        assert!(
            resolve(&lib, &index, lib.text.find(text).unwrap()).is_none(),
            "{text}"
        );
    }
}

#[test]
fn caret_trait_wins_over_enclosing_trait_without_spelling_joins() {
    let text = "// 🦀\ntrait Outer { fn method<T: Inner>(); }\ntrait Inner {}\n";
    let buffer = SourceBuffer::from_str("a.rs", text);
    let outer = text.find("Outer").unwrap();
    let inner = text.find("Inner").unwrap();
    let occurrence = |start, symbol, definition| {
        crate::semantic::test_support::occurrence(&buffer, start..start + 5, symbol, definition)
    };
    let index = SemanticIndex::from_scip(ScipIndex {
        documents: vec![ScipDocument {
            relative_path: buffer.path.clone(),
            position_encoding: 2,
            occurrences: vec![
                occurrence(outer, "test Outer#", true),
                occurrence(inner, "test Inner#", false),
            ],
            symbols: vec![ScipSymbolInformation {
                symbol: "test Inner#".into(),
                kind: 53,
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    });
    assert_eq!(
        resolve(&buffer, &index, inner).unwrap().symbol,
        "test Inner#"
    );
    assert_eq!(
        resolve(&buffer, &index, text.find("method").unwrap())
            .unwrap()
            .symbol,
        "test Outer#"
    );
    let mut document = index.documents["a.rs"].clone();
    document.symbols[0].kind = 42;
    let raw = ScipIndex {
        documents: vec![document],
        ..Default::default()
    };
    assert_eq!(
        resolve(&buffer, &SemanticIndex::from_scip(raw), inner)
            .unwrap()
            .symbol,
        "test Inner#"
    );
}

#[test]
fn trait_context_rejects_before_stale_wrong_language_unresolved_and_cancelled_inputs() {
    let text = "trait Trait {}\nimpl Trait for Item { fn f() {} }\n";
    let mut buffer = SourceBuffer::from_str("a.rs", text);
    let start = text.rfind("Trait").unwrap();
    let mut raw = ScipIndex {
        documents: vec![ScipDocument {
            relative_path: buffer.path.clone(),
            position_encoding: 2,
            occurrences: vec![crate::semantic::test_support::occurrence(
                &buffer,
                start..start + 5,
                "test Trait#",
                false,
            )],
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut index = SemanticIndex::from_scip(raw.clone());
    let caret = text.find("fn f").unwrap();
    assert_eq!(
        resolve(&buffer, &index, caret).unwrap().symbol,
        "test Trait#"
    );
    buffer.revision = Some(ReviewRevision::Before);
    assert!(resolve(&buffer, &index, caret).is_none());
    buffer.revision = None;
    index
        .source_digests
        .insert(buffer.path.clone(), "stale".into());
    assert!(resolve(&buffer, &index, caret).is_none());
    index.source_digests.clear();
    let syntax = DocumentSyntax::analyze(&buffer);
    let cancel = AtomicBool::new(false);
    cancel.store(true, Ordering::Relaxed);
    assert!(trait_at_byte(&buffer, &syntax, &index, caret, &cancel).is_err());
    let other = SourceBuffer::from_str("a.swift", text);
    assert!(resolve(&other, &index, caret).is_none());
    let duplicate = raw.documents[0].occurrences[0].clone();
    raw.documents[0].occurrences.push(duplicate);
    assert!(resolve(&buffer, &SemanticIndex::from_scip(raw.clone()), caret).is_some());
    raw.documents[0].occurrences[1].symbol = "test Other#".into();
    raw.documents[0].symbols.push(ScipSymbolInformation {
        symbol: "test Trait#".into(),
        kind: 53,
        ..Default::default()
    });
    let ambiguous = SemanticIndex::from_scip(raw.clone());
    assert!(resolve(&buffer, &ambiguous, start).is_none());
    assert!(resolve(&buffer, &SemanticIndex::from_scip(raw.clone()), caret).is_none());
    raw.documents[0].occurrences.truncate(1);
    raw.documents[0].occurrences[0].symbol_roles = 1;
    assert!(resolve(&buffer, &SemanticIndex::from_scip(raw.clone()), caret).is_none());
    raw.documents[0].occurrences[0].symbol_roles = 0;
    raw.documents[0].occurrences[0].symbol = "local 0".into();
    assert!(resolve(&buffer, &SemanticIndex::from_scip(raw.clone()), caret).is_none());
    // A declaration can have a document-local identity, unlike an impl join.
    raw.documents[0].occurrences[0] =
        crate::semantic::test_support::occurrence(&buffer, 6..11, "local 0", true);
    assert!(resolve(&buffer, &SemanticIndex::from_scip(raw), 0)
        .unwrap()
        .is_local());
}
