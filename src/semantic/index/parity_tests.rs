use super::*;
use crate::semantic::test_support::{document, occurrence, scip_rust_fixture};
use crate::source::SourceBuffer;

#[test]
fn real_swift_fixture_prefers_field_identity_and_ignores_whole_file_caret_hits() {
    let owner = scip_rust_fixture();
    let raw = ScipIndex::decode(&std::fs::read(owner.path().join("index.scip")).unwrap()).unwrap();
    let index = SemanticIndex::from_scip(raw);
    let shapes = SourceBuffer::from_bytes(
        "src/shapes.rs",
        std::fs::read(owner.path().join("src/shapes.rs")).unwrap(),
        None,
    )
    .unwrap();
    let shorthand = shapes.text.find("Circle { radius }").unwrap() + "Circle { ".len();
    let field = index.occurrence_at_byte(&shapes, shorthand + 1).unwrap();
    assert_eq!(
        field.symbol,
        "rust-analyzer cargo ra_fixture 0.1.0 shapes/Circle#radius."
    );
    let id = SemanticSymbolID::new(&field.symbol, &shapes.path);
    assert_eq!(index.find_references_for_id(&id).len(), 2);
    assert_eq!(index.find_definition_for_id(&id).unwrap().1.start_line, 1);
    let lib = SourceBuffer::from_bytes(
        "src/lib.rs",
        std::fs::read(owner.path().join("src/lib.rs")).unwrap(),
        None,
    )
    .unwrap();
    let circle = lib.text.find("[Circle]").unwrap() + 1;
    assert_eq!(
        index.occurrence_at_byte(&lib, circle + 2).unwrap().symbol,
        "rust-analyzer cargo ra_fixture 0.1.0 shapes/Circle#"
    );
    assert!(
        index.occurrence_at_byte(&lib, circle + 6).is_none(),
        "the closing bracket is not a whole-file module caret target"
    );
}

#[test]
fn equal_range_caret_targets_prefer_navigation_then_global_identity_in_any_order() {
    let source = SourceBuffer::from_str("a.rs", "same other\n");
    let global = "rust-analyzer cargo app 1 field.";
    for (local_definition, global_definition, expected) in [
        (false, false, global),
        (true, false, "local 0"),
        (false, true, global),
        (true, true, global),
    ] {
        for reverse in [false, true] {
            let mut hits = vec![
                occurrence(&source, 0..4, "local 0", false),
                occurrence(&source, 0..4, global, false),
            ];
            if local_definition {
                hits.push(occurrence(&source, 5..10, "local 0", true));
            }
            if global_definition {
                hits.push(occurrence(&source, 5..10, global, true));
            }
            if reverse {
                hits.reverse();
            }
            let index = SemanticIndex::from_scip(ScipIndex {
                documents: vec![document(&source, hits)],
                ..Default::default()
            });
            assert_eq!(
                index.occurrence_at_byte(&source, 2).unwrap().symbol,
                expected
            );
        }
    }
    let index = SemanticIndex::from_scip(ScipIndex {
        documents: vec![document(
            &source,
            vec![
                occurrence(&source, 0..10, global, false),
                occurrence(&source, 1..3, "local 0", false),
                occurrence(&source, 5..10, global, true),
            ],
        )],
        ..Default::default()
    });
    assert_eq!(
        index.occurrence_at_byte(&source, 2).unwrap().symbol,
        "local 0",
        "narrower ranges precede navigation preference"
    );
}

#[test]
fn invalid_document_paths_cannot_publish_symbols_definitions_or_search_entries() {
    let source = SourceBuffer::from_str("a.rs", "thing\n");
    for path in [
        "../outside.rs",
        "/outside.rs",
        "",
        "a/../b.rs",
        "./a.rs",
        "a//b.rs",
    ] {
        let mut bad = document(
            &source,
            vec![occurrence(
                &source,
                0..5,
                "rust-analyzer cargo app 1 thing.",
                true,
            )],
        );
        bad.relative_path = path.into();
        let index = SemanticIndex::from_scip(ScipIndex {
            documents: vec![bad],
            ..Default::default()
        });
        assert!(index.documents.is_empty(), "{path:?}");
        assert!(
            index.symbols.is_empty() && index.definitions.is_empty() && index.references.is_empty()
        );
        assert!(index.search_entries.is_empty() && index.repository_packages.is_empty());
    }
}
