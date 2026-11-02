//! C++ grammar/query facts reach the actual reader and exact outline commits.
use super::*;

#[test]
fn cpp_reader_publishes_scopes_name_bytes_and_committed_function_folds() {
    let (owner, mut app) = reader();
    let text = include_str!("../../../../Tests/RadiologyTests/Fixtures/code-paths/cpp-reader.cpp");
    std::fs::write(owner.path().join("boot.cpp"), text).unwrap();
    app.open_repository(owner.path());
    settle(&mut app);
    app.open_file("boot.cpp");
    settle(&mut app);
    assert!(app.syntax_error.is_none() && app.fold_error.is_none());
    assert!(app.call_sites.supported && app.call_sites.scopes.iter().any(|scope| scope.closure));
    assert_eq!(app.outline_symbols[0].name, "Boot");
    let class = &app.outline_symbols[0].children[0];
    assert_eq!(class.name, "Kernel");
    assert_eq!(class.kind, crate::syntax::SymbolKind::Class);
    assert!(class
        .children
        .iter()
        .all(|method| &text[method.selection_range.clone()] == "start"));
    let ctx = Context::default();
    let output = rendered_frame(&mut app, &ctx);
    assert!(crate::ui::test_support::text_shapes(&output)
        .any(|shape| shape.galley.job.text == "Outline"));
    app.sync_outline();
    let row = crate::syntax::outline::outline_rows(
        &app.outline_symbols,
        app.open_buffer.as_ref().unwrap(),
    )
    .iter()
    .position(|row| row.symbol.name == "main")
    .unwrap();
    let first = app.outline_list.key(egui::Key::Home).unwrap();
    app.handle_outline_action(first);
    for _ in 0..row {
        let next = app.outline_list.key(egui::Key::ArrowDown).unwrap();
        app.handle_outline_action(next);
    }
    let commit = app.outline_list.key(egui::Key::Enter).unwrap();
    let debug = format!(
        "preview/commit: {commit:?}, symbols: {:?}",
        app.outline_symbols
    );
    app.handle_outline_action(commit);
    assert_eq!(
        app.open_buffer
            .as_ref()
            .unwrap()
            .byte_offset(app.caret_line, app.caret_col),
        text.find("main").unwrap(),
        "{debug}"
    );
    let fold = crate::syntax::folds::enclosing_function_fold(
        text.find("main").unwrap(),
        &app.outline_symbols,
        &app.fold_ranges,
    )
    .unwrap();
    assert!(!app.fold_store.is_folded(&fold.byte_range));
    assert_eq!(app.open_buffer.as_ref().unwrap().text, text);
}
