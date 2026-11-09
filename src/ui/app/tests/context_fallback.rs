//! Swift ContextModel's implementation, outline, local and external fallbacks.
use super::context_model::settle_metadata;
use super::*;
use crate::source::preview::PreviewPhase;
use crate::ui::test_support::assert_visible_text;

// Reuse the existing real-fixture 1x/2x app matrix and its accepted shapes source.
pub(super) fn assert_fallbacks(app: &mut RadiologyApp, ctx: &Context) {
    let source = app.open_buffer.clone().unwrap();
    let history = format!("{:?}", app.history);
    app.caret_line = 11;
    app.caret_col = 0;
    app.refresh_symbol_context();
    settle_metadata(app, ctx);
    let declaration = app.enclosing_source_context().unwrap().pop().unwrap();
    assert_eq!(declaration.kind, crate::syntax::SymbolKind::Extension);
    assert_eq!(
        app.reference_context.target().unwrap().id,
        SemanticSymbolID::new(
            "rust-analyzer cargo ra_fixture 0.1.0 shapes/Circle#",
            "src/shapes.rs"
        )
    );
    assert!(app.under_caret.is_none());
    assert_eq!(app.definition_preview.phase(), &PreviewPhase::None);

    app.caret_line = 6;
    app.caret_col = 39;
    app.refresh_symbol_context();
    settle_metadata(app, ctx);
    assert_eq!(app.under_caret.as_ref().unwrap().name, "Circle");
    app.caret_line = 4;
    app.caret_col = 0;
    app.refresh_symbol_context();
    assert!(app.under_caret.is_none());
    assert_eq!(app.definition_preview.phase(), &PreviewPhase::None);
    assert!(app.enclosing_source_context().unwrap().is_empty());
    assert!(app.reference_context.target().is_none());
    let painted = focus::frame(app, ctx, vec![]);
    assert_visible_text(
        &painted,
        "Place the caret inside a function, type, or other declaration.",
    );

    app.caret_line = 8;
    app.caret_col = 9;
    app.refresh_symbol_context();
    assert!(app.under_caret.is_none());
    assert_eq!(
        app.enclosing_source_context().unwrap().last().unwrap().name,
        "area"
    );
    assert_eq!(app.reference_context.target().unwrap().name, "area");
    // Match Swift's nil-syntax input while retaining exact indexed source.
    app.outline_symbols = Default::default();
    app.syntax_error = Some("Controlled structure unavailable".into());
    app.caret_col = 12;
    app.refresh_symbol_context();
    settle_metadata(app, ctx);
    assert!(app.enclosing_source_context().unwrap().is_empty());
    assert!(app.reference_context.target().is_none());
    let local = app.under_caret.as_ref().unwrap();
    assert!(local.id.is_local());
    assert_eq!(local.name, "r");
    let painted = focus::frame(app, ctx, vec![]);
    assert_visible_text(&painted, "This file has no parsed structure.");
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &source));
    assert_eq!(format!("{:?}", app.history), history);
    // Recover real prepared syntax after the deliberately absent-structure input.
    app.execute_command(ctx, ToolbarAction::ReloadSource);
    focus::settle_frame(app, ctx);
    app.caret_line = 9;
    app.caret_col = 35;
    app.refresh_symbol_context();
    settle_metadata(app, ctx);
    let local = app.under_caret.as_ref().unwrap();
    assert!(local.id.is_local());
    assert_eq!(local.name, "r");
    assert_eq!(local.file_occurrence_count, 3);
    let definition = local.local_definition_line.as_ref().unwrap();
    assert_eq!(
        (definition.number, definition.text.as_str()),
        (8, "        let r = self.radius;")
    );
    let painted = disclose_without_preview(app, ctx);
    assert!(
        crate::ui::test_support::text_shapes(&painted).any(|text| text
            .galley
            .job
            .text
            .contains("3 occurrences in this file"))
    );

    app.open_file("src/lib.rs");
    focus::settle_frame(app, ctx);
    let source = app.open_buffer.clone().unwrap();
    let history = format!("{:?}", app.history);
    app.caret_line = 8;
    app.caret_col = 12;
    app.refresh_symbol_context();
    settle_metadata(app, ctx);
    let external = app.under_caret.as_ref().unwrap();
    assert!(!external.id.is_local());
    assert!(external.destination.is_none());
    assert_eq!(external.definition_count, 0);
    assert!(external
        .external_package
        .as_ref()
        .unwrap()
        .starts_with("core"));
    let painted = disclose_without_preview(app, ctx);
    assert_visible_text(&painted, "No definition indexed in this repository.");
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &source));
    assert_eq!(format!("{:?}", app.history), history);
}

// Locals and external identities must disclose facts without starting source I/O.
fn disclose_without_preview(app: &mut RadiologyApp, ctx: &Context) -> egui::FullOutput {
    assert_eq!(app.definition_preview.phase(), &PreviewPhase::None);
    assert!(!app.definition_preview.is_running());
    app.definition_expanded = true;
    focus::frame(app, ctx, vec![])
}
