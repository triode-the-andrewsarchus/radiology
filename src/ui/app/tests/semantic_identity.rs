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
