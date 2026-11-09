//! Real Swift fixture checks reuse through Context's actual metadata publication.
use super::*;
use crate::source::preview::PreviewPhase;

// Publish symbol facts only: an unexpected excerpt worker must remain observable.
fn settle_metadata(app: &mut RadiologyApp, ctx: &Context) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while app.under_caret_job.is_running() {
        app.poll_under_caret_context(ctx);
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

#[test]
fn context_reuses_previews_and_declaration_rows_across_identity_changes() {
    let owner = crate::semantic::test_support::scip_rust_fixture();
    for scale in [1.0, 2.0] {
        let (_settings, mut app, ctx) = checkout_context(owner.path(), "src/lib.rs", scale);
        app.caret_line = 5;
        app.caret_col = 27;
        app.refresh_symbol_context();
        focus::settle_frame(&mut app, &ctx);
        let circle = app.under_caret.clone().unwrap();
        let first_preview = app.definition_preview.phase().clone();
        let PreviewPhase::Loaded(preview) = &first_preview else {
            panic!("missing Circle preview");
        };
        assert_eq!(preview.lines[0].text, "pub struct Circle {");
        let declarations = app.outline_symbols.clone();
        let rows = app.reference_context.results.clone();
        app.definition_expanded = true;
        for column in 28..33 {
            app.caret_col = column;
            app.refresh_symbol_context();
            assert_eq!(app.under_caret.as_ref(), Some(&circle));
            assert!(app.definition_expanded);
            assert!(!app.under_caret_job.is_running() && !app.definition_preview.is_running());
            assert!(Arc::ptr_eq(&app.outline_symbols, &declarations));
            assert!(Arc::ptr_eq(&app.reference_context.results, &rows));
        }
        app.caret_line = 8;
        app.caret_col = 21;
        app.refresh_symbol_context();
        focus::settle_frame(&mut app, &ctx);
        let PreviewPhase::Loaded(preview) = app.definition_preview.phase() else {
            panic!("missing area preview");
        };
        assert_eq!(
            preview
                .lines
                .iter()
                .map(|line| line.number)
                .collect::<Vec<_>>(),
            [7, 8, 9, 10]
        );
        assert!(!preview.truncated);
        assert!(Arc::ptr_eq(&app.reference_context.results, &rows));
        app.caret_line = 5;
        app.caret_col = 30;
        app.refresh_symbol_context();
        settle_metadata(&mut app, &ctx);
        assert_eq!(app.definition_preview.phase(), &first_preview);
        assert!(
            !app.definition_preview.is_running(),
            "cached Circle must publish without starting an excerpt worker"
        );
        app.open_file("src/shapes.rs");
        focus::settle_frame(&mut app, &ctx);
        app.caret_line = 8;
        app.caret_col = 12;
        app.refresh_symbol_context();
        focus::settle_frame(&mut app, &ctx);
        let area_rows = app.reference_context.results.clone();
        assert_eq!(
            (area_rows.reference_count, area_rows.definition_count),
            (1, 1)
        );
        app.reference_context.select(0);
        for (line, column) in [(8, 16), (8, 22), (9, 26), (9, 2), (8, 13)] {
            app.caret_line = line;
            app.caret_col = column;
            app.refresh_symbol_context();
            focus::settle_frame(&mut app, &ctx);
            assert_eq!(app.reference_context.selected, Some(0));
            assert!(Arc::ptr_eq(&app.reference_context.results, &area_rows));
        }
        app.caret_line = 6;
        app.caret_col = 40;
        app.refresh_symbol_context();
        focus::settle_frame(&mut app, &ctx);
        assert_eq!(app.reference_context.target().unwrap().name, "new");
        assert_eq!(app.reference_context.selected, None);
        app.caret_line = 8;
        app.caret_col = 12;
        app.refresh_symbol_context();
        assert!(!app.reference_context.is_query_running());
        assert!(Arc::ptr_eq(&app.reference_context.results, &area_rows));
        assert_eq!(app.reference_context.selected, None);
        focus::settle_frame(&mut app, &ctx);
    }
}

#[test]
fn cold_same_file_unicode_preview_is_ready_at_metadata_publication() {
    use crate::semantic::{
        test_support::{document, occurrence},
        ScipIndex, ScipSymbolInformation,
    };
    let text =
        "fn café() -> u32 { 1 }\nfn main() {\n    let s = \"🚀🚀\"; let total = café();\n}\n";
    for scale in [1.0, 2.0] {
        let (owner, mut app) = isolated_source("src/uni.rs", text);
        let source = app.open_buffer.clone().unwrap();
        let symbol = "rust-analyzer cargo uni 0.1.0 café().";
        let definition = text.find("café").unwrap();
        let call = text.rfind("café").unwrap();
        let total = text.find("total").unwrap();
        let mut doc = document(
            &source,
            vec![
                occurrence(&source, definition..definition + "café".len(), symbol, true),
                occurrence(&source, call..call + "café".len(), symbol, false),
                occurrence(&source, total..total + 5, "local 1", true),
            ],
        );
        doc.symbols = vec![
            ScipSymbolInformation {
                symbol: symbol.into(),
                display_name: "café".into(),
                kind: 17,
                signature: "fn café() -> u32".into(),
                ..Default::default()
            },
            ScipSymbolInformation {
                symbol: "local 1".into(),
                display_name: "total".into(),
                kind: 61,
                ..Default::default()
            },
        ];
        app.semantic_index = Some(Arc::new(SemanticIndex::from_scip(ScipIndex {
            documents: vec![doc],
            ..Default::default()
        })));
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        let history = format!("{:?}", app.history);
        let column = source.text[source.line_starts[2]..call]
            .encode_utf16()
            .count();
        for offset in 0..4 {
            let byte = source
                .byte_offset_encoded(3, column + offset, crate::source::ColumnEncoding::Utf16)
                .unwrap();
            app.caret_line = 3;
            app.caret_col = source.column_at_byte(byte);
            app.refresh_symbol_context();
            if offset > 0 {
                assert!(!app.under_caret_job.is_running());
                assert!(app.definition_expanded);
            }
            settle_metadata(&mut app, &ctx);
            let under_caret = app.under_caret.as_ref().unwrap();
            assert_eq!(
                (under_caret.name.as_str(), under_caret.signature.as_str()),
                ("café", "fn café() -> u32")
            );
            assert_eq!(
                app.enclosing_source_context().unwrap().last().unwrap().name,
                "main"
            );
            let PreviewPhase::Loaded(preview) = app.definition_preview.phase() else {
                panic!(
                    "cold same-file preview required an excerpt worker: {:?}",
                    app.definition_preview.phase()
                );
            };
            assert_eq!(preview.lines.len(), 1);
            assert_eq!(preview.lines[0].number, 1);
            assert_eq!(preview.lines[0].text, "fn café() -> u32 { 1 }");
            assert!(!app.definition_preview.is_running());
            assert_eq!(format!("{:?}", app.history), history);
            assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &source));
            app.definition_expanded = true;
        }
        let rocket = text.find("🚀").unwrap();
        let rocket_column = text[source.line_starts[2]..rocket].encode_utf16().count();
        assert!(source
            .byte_offset_encoded(3, rocket_column + 1, crate::source::ColumnEncoding::Utf16)
            .is_none());
        app.caret_col = source.column_at_byte(rocket);
        app.refresh_symbol_context();
        assert!(app.under_caret.is_none());
        assert_eq!(app.definition_preview.phase(), &PreviewPhase::None);
        app.caret_col = source.column_at_byte(total + 2);
        app.refresh_symbol_context();
        settle_metadata(&mut app, &ctx);
        assert_eq!(app.under_caret.as_ref().unwrap().name, "total");
        assert!(app.under_caret.as_ref().unwrap().id.is_local());
        assert_eq!(app.definition_preview.phase(), &PreviewPhase::None);
        assert!(!app.definition_preview.is_running());
        let site = app
            .semantic_index
            .as_ref()
            .unwrap()
            .occurrence_at_byte(&source, call)
            .unwrap();
        let range = app
            .semantic_index
            .as_ref()
            .unwrap()
            .byte_range(&source, site.range.as_ref().unwrap())
            .unwrap();
        assert_eq!(&source.text[range], "café");
        let path = owner.path().join("checkout/src/uni.rs");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
        let edited = text.replace("{ 1 }", "{ 2 }");
        std::fs::write(&path, &edited).unwrap();
        app.execute_command(&ctx, ToolbarAction::ReloadSource);
        settle(&mut app);
        let fresh = app.open_buffer.clone().unwrap();
        app.caret_line = 3;
        app.caret_col = fresh.column_at_byte(call);
        app.refresh_symbol_context();
        settle_metadata(&mut app, &ctx);
        let PreviewPhase::Loaded(preview) = app.definition_preview.phase() else {
            panic!("reload did not publish a memory preview");
        };
        assert_eq!(preview.lines[0].text, "fn café() -> u32 { 2 }");
        assert_eq!(preview.content_digest, fresh.content_digest());
        assert!(!app.definition_preview.is_running());
        assert_eq!(format!("{:?}", app.history), history);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), edited);
    }
}
