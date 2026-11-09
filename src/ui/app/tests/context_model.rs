//! Real Swift fixture checks reuse through Context's actual metadata publication.
use super::*;
use crate::source::preview::PreviewPhase;

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
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while app.under_caret_job.is_running() {
            app.poll_under_caret_context(&ctx);
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
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
