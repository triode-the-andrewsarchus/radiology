use super::*;

#[test]
fn shorthand_keeps_semantic_field_target_and_preserves_spelling_fallback_at_both_scales() {
    let owner = crate::semantic::test_support::scip_rust_fixture();
    for scale in [1.0, 2.0] {
        let (_settings, mut app, ctx) = checkout_context(owner.path(), "src/shapes.rs", scale);
        let source = app.open_buffer.clone().unwrap();
        let byte = source.text.find("Circle { radius }").unwrap() + "Circle { ".len();
        app.navigate(source.location_at_byte(byte + 1), false);
        focus::settle_frame(&mut app, &ctx);
        let semantic = app.under_caret.as_ref().unwrap();
        assert_eq!(
            semantic.id.symbol,
            "rust-analyzer cargo ra_fixture 0.1.0 shapes/Circle#radius."
        );
        assert_eq!(
            (semantic.reference_count, semantic.file_occurrence_count),
            (2, 3)
        );
        assert_eq!(semantic.destination.as_ref().unwrap().range.start_line, 1);
        // Swift deliberately keeps a function-local spelling decoration when
        // the semantic field set does not cover the same-spelled parameter.
        app.select_occurrences_at_byte(byte + 1);
        let selected = app.pin_store.ephemeral.clone().unwrap();
        assert!(selected.symbol_name.starts_with("syntactic:"));
        assert_eq!(selected.ranges.len(), 2);
        let local_parameter = source.text.find("new(radius").unwrap() + "new(".len();
        assert!(selected
            .ranges
            .iter()
            .any(|range| range.contains(&local_parameter)));
        app.pin_at_byte(byte + 1);
        assert!(app.pin_store.ephemeral.is_none());
        assert_eq!(app.pin_store.groups[0].symbol_name, selected.symbol_name);
        assert_eq!(app.pin_store.groups[0].ranges, selected.ranges);
        app.select_occurrences_at_byte(source.text.find("impl Circle").unwrap() - 1);
        assert!(app.pin_store.ephemeral.is_none());
        app.select_occurrences_at_byte(byte + 1);
        assert!(app.pin_store.ephemeral.is_some());
        app.open_file("src/lib.rs");
        focus::settle_frame(&mut app, &ctx);
        assert!(app.pin_store.ephemeral.is_none());
        let lib = app.open_buffer.clone().unwrap();
        let circle_end = lib.text.find("[Circle]").unwrap() + "[Circle".len();
        app.select_occurrences_at_byte(circle_end);
        assert!(
            app.pin_store.ephemeral.is_none(),
            "punctuation must not select a whole-file module"
        );
        assert_eq!(
            std::fs::read_to_string(owner.path().join("src/shapes.rs")).unwrap(),
            source.text
        );
    }
}

#[test]
fn semantic_selection_preserves_unicode_bytes_and_external_operator_identity() {
    use crate::semantic::test_support::{document, occurrence};
    use crate::semantic::{ScipIndex, SemanticIndex};
    let fixture = crate::semantic::test_support::scip_rust_fixture();
    for (scale, encoding) in [(1.0, 1), (1.0, 2), (2.0, 1), (2.0, 2)] {
        let (_settings, mut app, ctx) = checkout_context(fixture.path(), "src/lib.rs", scale);
        let lib = app.open_buffer.clone().unwrap();
        app.select_occurrences_at_byte(lib.byte_offset(8, 12));
        let external = app.pin_store.ephemeral.as_ref().unwrap();
        assert!(external.symbol_name.starts_with("rust-analyzer cargo core"));
        assert!(!external.ranges.is_empty());
        app.select_occurrences_at_byte(lib.byte_offset(3, 0));
        assert!(app.pin_store.ephemeral.is_none());
        let text = "let 🚀café = 1;\nuse(🚀café);\n";
        std::fs::write(fixture.path().join("unicode.rs"), text).unwrap();
        app.open_file("unicode.rs");
        focus::settle_frame(&mut app, &ctx);
        let source = app.open_buffer.clone().unwrap();
        let symbol = "rust-analyzer cargo unicode 0.1.0 crate/café.";
        let mut doc = document(
            &source,
            vec![
                occurrence(&source, 4..13, symbol, true),
                occurrence(&source, 23..32, symbol, false),
            ],
        );
        if encoding == 1 {
            doc.position_encoding = 1;
            for (line, occurrence) in doc.occurrences.iter_mut().enumerate() {
                occurrence.range = crate::semantic::ScipRange::from_packed(&[line as i32, 4, 13]);
            }
        }
        app.semantic_index = Some(std::sync::Arc::new(SemanticIndex::from_scip(ScipIndex {
            documents: vec![doc],
            ..Default::default()
        })));
        app.select_occurrences_at_byte(6);
        let selection = app.pin_store.ephemeral.clone().unwrap();
        assert_eq!(selection.symbol_name, symbol);
        assert_eq!(selection.active, 4..13);
        assert_eq!(selection.ranges, [4..13, 23..32]);
        let coordinates = reader_text::CoordinateIndex::new(&source.text);
        let utf16 = selection
            .ranges
            .iter()
            .map(|range| {
                let start = coordinates
                    .units_at_byte(
                        &source.text,
                        range.start,
                        crate::source::ColumnEncoding::Utf16,
                    )
                    .unwrap();
                let end = coordinates
                    .units_at_byte(
                        &source.text,
                        range.end,
                        crate::source::ColumnEncoding::Utf16,
                    )
                    .unwrap();
                (start, end - start)
            })
            .collect::<Vec<_>>();
        assert_eq!(utf16, [(4, 6), (20, 6)]);
        app.pin_at_byte(6);
        assert!(app.pin_store.ephemeral.is_none());
        assert_eq!(
            app.pin_store.groups.last().unwrap().ranges,
            selection.ranges
        );
        assert_eq!(
            std::fs::read_to_string(fixture.path().join("unicode.rs")).unwrap(),
            text
        );
    }
}
