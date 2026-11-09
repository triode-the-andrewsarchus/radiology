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

#[test]
fn real_swift_fixture_retains_tool_metadata_before_and_after_normalization() {
    let owner = scip_rust_fixture();
    let raw = ScipIndex::decode(&std::fs::read(owner.path().join("index.scip")).unwrap()).unwrap();
    let mut paths = raw
        .documents
        .iter()
        .map(|document| document.relative_path.as_str())
        .collect::<Vec<_>>();
    paths.sort();
    assert_eq!(paths, ["src/lib.rs", "src/shapes.rs"]);
    assert!(raw
        .documents
        .iter()
        .all(|document| document.position_encoding == 1));
    let encoded = serde_json::to_value(&raw).unwrap();
    assert_eq!(encoded["sources"][0]["tool_name"], "rust-analyzer");
    assert!(encoded["sources"][0]["project_root"]
        .as_str()
        .unwrap()
        .starts_with("file://"));
    assert_eq!(encoded["sources"][0]["document_count"], 2);
    let expected = raw.sources.clone();
    let index = SemanticIndex::from_scip(raw);
    assert_eq!(index.sources, expected);
    let total = index
        .symbols
        .values()
        .find(|symbol| symbol.display_name == "total_area")
        .unwrap();
    assert_eq!(total.kind, 17);
    assert_eq!(
        total.signature,
        "pub fn total_area(items: &[Circle]) -> f64"
    );
    assert!(total
        .documentation
        .iter()
        .any(|paragraph| paragraph.contains("café 🚀")));

    assert_eq!(
        index.column_encoding("src/shapes.rs"),
        Some(crate::source::ColumnEncoding::Utf8)
    );
}

#[test]
fn real_fixture_definitions_context_search_and_stale_ranges_match_swift() {
    use crate::semantic::context::UnderCaretContext;
    use std::sync::atomic::AtomicBool;
    let owner = scip_rust_fixture();
    let index = SemanticIndex::load_file(&owner.path().join("index.scip")).unwrap();
    let lib = SourceBuffer::from_bytes(
        "src/lib.rs",
        std::fs::read(owner.path().join("src/lib.rs")).unwrap(),
        None,
    )
    .unwrap();
    assert!(index.documents[&lib.path]
        .occurrences
        .iter()
        .all(|occurrence| {
            occurrence
                .range
                .as_ref()
                .is_some_and(|range| index.byte_range(&lib, range).is_some())
        }));
    let circle = SemanticSymbolID::new(
        "rust-analyzer cargo ra_fixture 0.1.0 shapes/Circle#",
        &lib.path,
    );
    assert!(!circle.is_local());
    let definitions = index.definition_locations_for_id(&circle);
    assert_eq!(definitions.len(), 1);
    assert_eq!(
        definitions[0],
        (
            "src/shapes.rs",
            &ScipRange::from_packed(&[0, 11, 17]).unwrap()
        )
    );
    let locals = ["src/lib.rs", "src/shapes.rs"].map(|path| SemanticSymbolID::new("local 0", path));
    assert_ne!(locals[0], locals[1]);
    for (id, path, name) in [
        (&locals[0], "src/lib.rs", "items"),
        (&locals[1], "src/shapes.rs", "radius"),
    ] {
        assert_eq!(index.find_definition_for_id(id).unwrap().0, path);
        assert_eq!(index.display_name(id), name);
    }
    for (line, col, name, kind, definition, destination) in [
        (5, 29, "Circle", "Struct", false, Some(0)),
        (5, 8, "total_area", "Function", true, None),
        (8, 21, "area", "Method", false, Some(6)),
    ] {
        let context = UnderCaretContext::prepare(
            &lib,
            &index,
            lib.byte_offset(line, col),
            &AtomicBool::new(false),
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            (
                context.name.as_str(),
                context.kind.as_str(),
                context.is_on_definition
            ),
            (name, kind, definition)
        );
        assert_eq!(
            context
                .destination
                .as_ref()
                .map(|target| target.range.start_line),
            destination
        );
        if let Some(target) = context.destination {
            assert_eq!(target.path, "src/shapes.rs");
        }
    }
    let external = UnderCaretContext::prepare(
        &lib,
        &index,
        lib.byte_offset(8, 12),
        &AtomicBool::new(false),
    )
    .unwrap()
    .unwrap();
    assert!(external.id.symbol.starts_with("rust-analyzer cargo core"));
    assert_eq!(external.definition_count, 0);
    assert!(external.destination.is_none());
    assert!(external.file_occurrence_count > 0);
    assert!(index
        .occurrence_at_byte(&lib, lib.byte_offset(3, 0))
        .is_none());
    let names = index
        .search_entries
        .iter()
        .map(|entry| entry.name.as_str())
        .collect::<Vec<_>>();
    for name in ["Circle", "total_area", "area", "new"] {
        assert!(names.contains(&name));
    }
    assert!(!names.contains(&"items"));
    let shortened = SourceBuffer::from_str("src/shapes.rs", "pub struct Circle {\n}\n");
    let occurrences = &index.documents[&shortened.path].occurrences;
    let mapped = occurrences
        .iter()
        .filter_map(|occurrence| {
            occurrence
                .range
                .as_ref()
                .and_then(|range| index.byte_range(&shortened, range))
        })
        .collect::<Vec<_>>();
    assert!(mapped.len() < occurrences.len());
    assert!(mapped.iter().all(|range| range.end <= shortened.text.len()));
    assert!(ScipIndex::decode(&[0x0a, 0xff, 0xff]).is_err());
    assert!(ScipIndex::decode(&[]).unwrap().documents.is_empty());
}

#[test]
fn swift_search_matrix_preserves_exact_prefix_initial_and_subsequence_order() {
    let entries = [
        "Circle",
        "CircleArea",
        "total_area",
        "tally",
        "SnapshotBootstrapGlobals",
    ]
    .into_iter()
    .enumerate()
    .map(|(offset, name)| SymbolSearchEntry {
        id: SemanticSymbolID::new(format!("s {name}"), ""),
        name: name.into(),
        kind: 17,
        path: format!("f{offset}.rs"),
        line: offset + 1,
        definition_count: 1,
    })
    .collect::<Vec<_>>();
    for (query, expected) in [
        ("circle", vec![("Circle", 0), ("CircleArea", 1)]),
        ("sbg", vec![("SnapshotBootstrapGlobals", 2)]),
        (
            "ta",
            vec![
                ("tally", 1),
                ("total_area", 2),
                ("SnapshotBootstrapGlobals", 3),
            ],
        ),
        ("tlar", vec![("total_area", 3)]),
        ("zzz", vec![]),
    ] {
        let matches = SymbolSearch::rank(query, &entries, 200);
        assert_eq!(
            matches
                .iter()
                .map(|hit| (hit.name.as_str(), hit.tier))
                .collect::<Vec<_>>(),
            expected,
            "{query}"
        );
    }
}

#[test]
fn encoded_ranges_use_exact_scalar_boundaries_and_reject_invalid_positions() {
    use crate::source::ColumnEncoding::{Utf16, Utf32, Utf8};
    let buffer = SourceBuffer::from_str("u.rs", "let 🚀café = 1;\nnext\n");
    for (encoding, column, expected) in [
        (Utf8, 4, Some(4)),
        (Utf8, 8, Some(8)),
        (Utf8, 6, None),
        (Utf16, 6, Some(8)),
        (Utf16, 5, None),
        (Utf32, 5, Some(8)),
        (Utf32, 9, Some(13)),
        (Utf32, 14, Some(18)),
        (Utf32, 15, None),
        (Utf8, 99, None),
    ] {
        assert_eq!(buffer.byte_offset_encoded(1, column, encoding), expected);
    }
    assert_eq!(buffer.byte_offset_encoded(6, 0, Utf8), None);
    let coordinates = reader_text::CoordinateIndex::new(&buffer.text);
    assert_eq!(coordinates.units_at_byte(&buffer.text, 8, Utf16), Some(6));
    assert_eq!(coordinates.units_at_byte(&buffer.text, 8, Utf32), Some(5));
    assert_eq!(buffer.line_at_byte(22), 2);
    assert_eq!(22 - buffer.line_starts[1], 3);
    let mut doc = document(&buffer, vec![]);
    doc.position_encoding = 1;
    let index = SemanticIndex::from_scip(ScipIndex {
        documents: vec![doc],
        ..Default::default()
    });
    assert_eq!(
        index.byte_range(&buffer, &ScipRange::from_packed(&[0, 4, 1, 2]).unwrap()),
        Some(4..21)
    );
    assert!(ScipRange::from_packed(&[0, -1, 4]).is_none());
    for packed in [vec![0, 4, 3, 0], vec![0, 6, 8]] {
        assert!(index
            .byte_range(&buffer, &ScipRange::from_packed(&packed).unwrap())
            .is_none());
    }
}
