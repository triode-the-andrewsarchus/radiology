//! Context previews are passive; explicit inspector actions record exact carets.
use super::*;
use crate::ui::test_support::{click, text_shapes};

#[test]
fn inspector_definition_and_declaration_links_return_to_carets_after_passive_scroll() {
    let owner = crate::semantic::test_support::scip_rust_fixture();
    let path = owner.path().join("src/lib.rs");
    let original = std::fs::read_to_string(&path).unwrap();
    let text = format!(
        "{original}{}",
        "// distant reading position café 🦀\n".repeat(300)
    );
    std::fs::write(&path, &text).unwrap();
    for scale in [1.0, 2.0] {
        let (_settings, mut app, ctx) = checkout_context(owner.path(), "src/lib.rs", scale);
        let source = app.open_buffer.clone().unwrap();
        app.caret_line = 8;
        app.caret_col = 21;
        app.refresh_symbol_context();
        focus::settle_frame(&mut app, &ctx);
        app.scroll_to_line = None;
        app.source_viewport
            .restore_top(&source, source.byte_offset(120, 0));
        focus::frame(&mut app, &ctx, vec![]);
        app.handle_inspector_action(
            &ctx,
            crate::ui::context_inspector::InspectorAction::GoToDefinition,
        );
        focus::settle_frame(&mut app, &ctx);
        assert_eq!(
            (app.selected_path.as_deref(), app.caret_line, app.caret_col),
            (Some("src/shapes.rs"), 7, 11)
        );
        app.execute_command(&ctx, ToolbarAction::NavigateBack);
        focus::settle_frame(&mut app, &ctx);
        assert_eq!(
            app.history.current(),
            Some(&source.location_at_byte(source.byte_offset(8, 21)))
        );
        assert_eq!((app.caret_line, app.caret_col), (8, 21));
        app.caret_line = 7;
        app.caret_col = 10;
        app.refresh_symbol_context();
        focus::settle_frame(&mut app, &ctx);
        let departure = source.location_at_byte(source.byte_offset(7, 10));
        app.scroll_to_line = None;
        let distant = source.byte_offset(120, 0);
        app.source_viewport.restore_top(&source, distant);
        focus::frame(&mut app, &ctx, vec![]);
        let output = focus::frame(&mut app, &ctx, vec![]);
        assert_eq!(app.source_viewport.top_byte(&source), Some(distant));
        assert_eq!((app.caret_line, app.caret_col), (7, 10));
        assert_eq!(
            app.enclosing_source_context().unwrap().last().unwrap().name,
            "total_area"
        );
        let label = text_shapes(&output)
            .find(|shape| shape.pos.x > 780.0 && shape.galley.job.text == "total_area")
            .unwrap();
        focus::frame(&mut app, &ctx, click(label.pos + label.galley.size() / 2.0));
        focus::settle_frame(&mut app, &ctx);
        assert_eq!((app.caret_line, app.caret_col), (5, 7));
        assert_eq!(
            app.history.current(),
            Some(&source.location_at_byte(source.byte_offset(5, 7)))
        );
        app.execute_command(&ctx, ToolbarAction::NavigateBack);
        focus::settle_frame(&mut app, &ctx);
        assert_eq!(app.history.current(), Some(&departure));
        assert_eq!((app.caret_line, app.caret_col), (7, 10));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
    }
}

#[test]
fn passive_context_and_reference_previews_only_navigate_on_explicit_activation() {
    use crate::source::preview::PreviewPhase;
    use crate::ui::reference_list::ReferenceAction;
    let owner = crate::semantic::test_support::scip_rust_fixture();
    for scale in [1.0, 2.0] {
        let (_settings, mut app, ctx) = checkout_context(owner.path(), "src/lib.rs", scale);
        let lib = app.open_buffer.clone().unwrap();
        let history = format!("{:?}", app.history);
        let top = app.source_viewport.top_byte(&lib);
        let position = app.reading_positions.current_byte();
        for (line, column) in [(5, 27), (5, 29), (5, 19), (8, 21), (6, 0), (7, 9), (5, 30)] {
            app.caret_line = line;
            app.caret_col = column;
            app.refresh_symbol_context();
            focus::settle_frame(&mut app, &ctx);
            assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &lib));
            assert_eq!(format!("{:?}", app.history), history);
            assert_eq!(app.source_viewport.top_byte(&lib), top);
            assert_eq!(app.reading_positions.current_byte(), position);
        }
        assert_eq!(
            app.enclosing_source_context().unwrap().last().unwrap().name,
            "total_area"
        );
        assert_eq!(app.under_caret.as_ref().unwrap().name, "Circle");
        let PreviewPhase::Loaded(preview) = app.definition_preview.phase() else {
            panic!("missing Circle preview");
        };
        assert_eq!(preview.path, "src/shapes.rs");
        app.handle_inspector_action(
            &ctx,
            crate::ui::context_inspector::InspectorAction::ExpandDefinition(true),
        );
        assert_eq!(format!("{:?}", app.history), history);
        app.open_file("src/shapes.rs");
        focus::settle_frame(&mut app, &ctx);
        let shapes = app.open_buffer.clone().unwrap();
        let departure = shapes.location_at_byte(shapes.byte_offset(8, 12));
        for activation in ["named", "keyboard", "caller"] {
            app.caret_line = 8;
            app.caret_col = 12;
            app.refresh_symbol_context();
            focus::settle_frame(&mut app, &ctx);
            assert_eq!(
                app.enclosing_source_context().unwrap().last().unwrap().name,
                "area"
            );
            let shapes = app.open_buffer.clone().unwrap();
            let results = &app.reference_context.results;
            let call = results
                .hits
                .iter()
                .position(|hit| hit.path == "src/lib.rs")
                .unwrap();
            let definition = results
                .hits
                .iter()
                .position(|hit| hit.is_definition)
                .unwrap();
            let destination = lib.location_at_byte(results.hits[call].byte_range.start);
            let history = format!("{:?}", app.history);
            app.reading_positions.observe(&shapes, Some(1), None);
            app.reading_positions.observe(&shapes, Some(0), None);
            app.handle_reference_action(ReferenceAction::Select(definition));
            app.handle_reference_action(ReferenceAction::Select(call));
            app.handle_reference_action(ReferenceAction::SetExpanded(call, true));
            focus::settle_frame(&mut app, &ctx);
            let disclosure = &app.reference_context.disclosures[&call];
            let PreviewPhase::Loaded(preview) = disclosure.phase() else {
                panic!("missing reference preview");
            };
            assert!(preview
                .lines
                .iter()
                .any(|line| line.number == 8 && line.text.contains("item.area()")));
            assert_eq!(disclosure.callers().unwrap().rows[0].name, "total_area");
            assert_eq!(format!("{:?}", app.history), history);
            assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &shapes));
            assert_eq!((app.caret_line, app.caret_col), (8, 12));
            match activation {
                "named" => {
                    app.handle_reference_action(ReferenceAction::Select(definition));
                    app.handle_reference_action(ReferenceAction::Commit(Some(usize::MAX)));
                    assert_eq!(format!("{:?}", app.history), history);
                    assert!(!app.is_source_loading());
                    app.handle_reference_action(ReferenceAction::Commit(Some(call)));
                }
                "keyboard" => {
                    assert!(reference_key(&mut app, &ctx, egui::Key::Enter));
                }
                "caller" => {
                    app.handle_reference_action(ReferenceAction::CommitCaller(call, 0));
                }
                _ => unreachable!(),
            }
            focus::settle_frame(&mut app, &ctx);
            assert_eq!(app.history.current(), Some(&destination), "{activation}");
            assert_eq!(app.open_buffer.as_ref().unwrap().path, "src/lib.rs");
            app.execute_command(&ctx, ToolbarAction::NavigateBack);
            focus::settle_frame(&mut app, &ctx);
            assert_eq!(app.history.current(), Some(&departure), "{activation}");
            assert_eq!((app.caret_line, app.caret_col), (8, 12));
            app.execute_command(&ctx, ToolbarAction::NavigateForward);
            focus::settle_frame(&mut app, &ctx);
            assert_eq!(app.history.current(), Some(&destination));
            app.execute_command(&ctx, ToolbarAction::NavigateBack);
            focus::settle_frame(&mut app, &ctx);
            assert_eq!(app.history.current(), Some(&departure));
        }
    }
}

#[test]
fn accepted_source_changes_reset_context_and_unindexed_structure_keeps_the_live_caret() {
    let owner = crate::semantic::test_support::scip_rust_fixture();
    for scale in [1.0, 2.0] {
        let (settings, mut app, ctx) = checkout_context(owner.path(), "src/lib.rs", scale);
        app.caret_line = 5;
        app.caret_col = 28;
        app.refresh_symbol_context();
        focus::settle_frame(&mut app, &ctx);
        assert_eq!(app.under_caret.as_ref().unwrap().name, "Circle");
        let lib = app.open_buffer.clone().unwrap();
        app.open_file("src/shapes.rs");
        assert!(app.enclosing_source_context().is_none());
        let prepared = take_prepared_source(&mut app);
        assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &lib));
        assert!(app.enclosing_source_context().is_none());
        app.publish_source(prepared);
        focus::settle_frame(&mut app, &ctx);
        assert_eq!(
            app.enclosing_source_context().unwrap().last().unwrap().name,
            "Circle"
        );
        assert!(app.under_caret.is_none());
        app.open_file("Cargo.toml");
        focus::settle_frame(&mut app, &ctx);
        assert!(app.semantic_index.is_some());
        let output = focus::frame(&mut app, &ctx, vec![]);
        assert!(text_shapes(&output).any(|shape| shape.galley.job.text == "File not indexed"));
        assert!(app.under_caret.is_none());
        assert!(app.enclosing_source_context().unwrap().is_empty());
        let plain = settings.path().join("plain");
        std::fs::create_dir(&plain).unwrap();
        let text = format!(
            "fn only() {{\n{}}}\n",
            "    let note = \"🚀 café\";\n".repeat(300)
        );
        std::fs::write(plain.join("a.rs"), &text).unwrap();
        app.open_repository(&plain);
        assert!(app.enclosing_source_context().is_none());
        focus::settle_frame(&mut app, &ctx);
        assert!(app.under_caret.is_none() && app.semantic_inputs.is_none());
        assert!(app.reference_context.results.hits.is_empty());
        assert!(app.semantic_index.is_none());
        app.open_file("a.rs");
        focus::settle_frame(&mut app, &ctx);
        let source = app.open_buffer.clone().unwrap();
        let byte = source.byte_offset(4, 0) + "    let note = \"🚀 ".len();
        app.caret_line = 4;
        app.caret_col = source.column_at_byte(byte);
        app.refresh_symbol_context();
        app.source_viewport.request_focus(&ctx);
        focus::frame(&mut app, &ctx, vec![]);
        focus::frame(
            &mut app,
            &ctx,
            vec![focus::key(egui::Key::ArrowRight, egui::Modifiers::NONE)],
        );
        assert_eq!(source.byte_offset(app.caret_line, app.caret_col), byte + 1);
        let caret = (app.caret_line, app.caret_col);
        let history = format!("{:?}", app.history);
        app.scroll_to_line = None;
        let top = source.byte_offset(150, 0);
        app.source_viewport.restore_top(&source, top);
        focus::frame(&mut app, &ctx, vec![]);
        focus::frame(&mut app, &ctx, vec![]);
        assert_eq!(app.source_viewport.top_byte(&source), Some(top));
        assert_eq!(app.reading_positions.current_byte(), Some(top));
        assert_eq!((app.caret_line, app.caret_col), caret);
        assert_eq!(format!("{:?}", app.history), history);
        assert_eq!(
            app.enclosing_source_context().unwrap().last().unwrap().name,
            "only"
        );
        assert!(app.under_caret.is_none());
        assert!(!app.command_enabled(&ToolbarAction::GoToDefinition));
        assert!(app.reference_context.results.hits.is_empty());
        let output = focus::frame(&mut app, &ctx, vec![]);
        assert!(text_shapes(&output).any(|shape| shape.galley.job.text == "No semantic index"));
        assert_eq!(std::fs::read_to_string(plain.join("a.rs")).unwrap(), text);
    }
}
