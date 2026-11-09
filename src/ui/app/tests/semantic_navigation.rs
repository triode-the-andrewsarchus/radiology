//! Generated semantic data uses the same exact-caret navigation transaction as imports.
use super::*;
use crate::ui::reference_list::ReferenceAction;

#[cfg(unix)]
#[test]
fn generated_swift_index_drives_definition_reference_search_and_repository_retirement() {
    let owner = crate::semantic::test_support::scip_rust_fixture();
    let wire = std::fs::read(owner.path().join("index.scip")).unwrap();
    std::fs::remove_file(owner.path().join("index.scip")).unwrap();
    for scale in [1.0, 2.0] {
        let (settings, mut app) = reader();
        let tool = rust_indexer_fixture(settings.path(), "src/lib.rs");
        std::fs::write(tool.with_file_name("fixture.scip"), &wire).unwrap();
        open_reader(&mut app, owner.path(), "src/lib.rs");
        assert!(app.semantic_index.is_none());
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        app.execute_command(&ctx, ToolbarAction::GenerateIndex);
        focus::settle_frame(&mut app, &ctx);
        assert_eq!(app.generation_report().unwrap().documents, 2);
        assert_eq!(app.semantic_index.as_ref().unwrap().source_digests.len(), 2);
        assert!(!owner.path().join("artifact.bin").exists());
        let lib = app.open_buffer.clone().unwrap();
        let click = lib.byte_offset(5, 29);
        let departure = lib.location_at_byte(click);
        app.caret_line = 5;
        app.caret_col = 29;
        app.refresh_symbol_context();
        focus::settle_frame(&mut app, &ctx);
        let history = format!("{:?}", app.history);
        // A restoration report must not displace the semantic departure caret.
        app.reading_positions.observe(&lib, Some(0), None);
        app.reading_positions
            .observe(&lib, Some(lib.byte_offset(9, 0)), None);
        assert_eq!((app.caret_line, app.caret_col), (5, 29));
        app.find_references_at_byte(click);
        focus::settle_frame(&mut app, &ctx);
        let results = &app.reference_context.results;
        assert_eq!(
            (
                results.hits.len(),
                results.reference_count,
                results.definition_count
            ),
            (5, 4, 1)
        );
        assert_eq!(
            results
                .groups
                .iter()
                .map(|group| group.path.as_str())
                .collect::<Vec<_>>(),
            ["src/lib.rs", "src/shapes.rs"]
        );
        let lib_hits = &results.hits[results.groups[0].hits.clone()];
        assert_eq!(
            lib_hits
                .iter()
                .map(|hit| hit.source_line.number)
                .collect::<Vec<_>>(),
            [2, 5]
        );
        assert_eq!(lib_hits[0].source_line.text, "pub use shapes::Circle;");
        assert!(results.hits[results.groups[1].hits.start].is_definition);
        assert_eq!(format!("{:?}", app.history), history);
        assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &lib));
        app.execute_command(&ctx, ToolbarAction::GoToDefinition);
        focus::settle_frame(&mut app, &ctx);
        let shapes = app.open_buffer.clone().unwrap();
        assert_eq!(
            (shapes.path.as_str(), app.caret_line, app.caret_col),
            ("src/shapes.rs", 1, 11)
        );
        assert!(app.under_caret.as_ref().unwrap().is_on_definition);
        let destination = shapes.location_at_byte(shapes.byte_offset(1, 11));
        assert_eq!(app.history.current(), Some(&destination));
        for command in [
            ToolbarAction::NavigateBack,
            ToolbarAction::NavigateForward,
            ToolbarAction::NavigateBack,
        ] {
            let back = matches!(command, ToolbarAction::NavigateBack);
            app.execute_command(&ctx, command);
            focus::settle_frame(&mut app, &ctx);
            assert_eq!(
                app.history.current(),
                Some(if back { &departure } else { &destination })
            );
            assert_eq!(
                (app.caret_line, app.caret_col),
                if back { (5, 29) } else { (1, 11) }
            );
        }
        let matches = app
            .semantic_index
            .as_ref()
            .unwrap()
            .search_symbols("circ", 200);
        assert_eq!((matches[0].name.as_str(), matches[0].kind), ("Circle", 49));
        app.navigate_to_indexed_symbol(&matches[0].id);
        focus::settle_frame(&mut app, &ctx);
        assert_eq!(app.history.current(), Some(&destination));
        let reference = app
            .reference_context
            .results
            .hits
            .iter()
            .position(|hit| hit.path == "src/lib.rs" && hit.source_line.number == 5)
            .unwrap();
        let target = app.reference_context.results.hits[reference]
            .byte_range
            .start;
        app.handle_reference_action(ReferenceAction::Commit(Some(reference)));
        focus::settle_frame(&mut app, &ctx);
        assert_eq!(app.open_buffer.as_ref().unwrap().path, "src/lib.rs");
        assert_eq!(lib.byte_offset(app.caret_line, app.caret_col), target);
        let plain = settings.path().join("plain");
        std::fs::create_dir(&plain).unwrap();
        std::fs::write(plain.join("a.swift"), "let x = 1\n").unwrap();
        app.open_repository(&plain);
        focus::settle_frame(&mut app, &ctx);
        assert!(app.semantic_index.is_none() && app.under_caret.is_none());
        assert!(app.semantic_inputs.is_none() && app.definition_identity.is_none());
        assert!(app.pin_store.ephemeral.is_none());
        assert!(app.reference_context.results.hits.is_empty());
        assert!(app.reference_context.explicit().is_none());
        assert!(app.history.current().is_none());
        assert!(app
            .index_plan
            .uncovered_languages
            .contains(&"Swift".to_owned()));
    }
}

#[test]
fn unavailable_definition_reports_why_without_changing_reader_or_history() {
    let owner = crate::semantic::test_support::scip_rust_fixture();
    for (scale, metadata_ready) in [(1.0, false), (1.0, true), (2.0, false), (2.0, true)] {
        let (_settings, mut app, ctx) = checkout_context(owner.path(), "src/lib.rs", scale);
        for (line, column, reason) in [(8, 12, "outside this repository"), (5, 8, "defined here")] {
            app.caret_line = line;
            app.caret_col = column;
            app.refresh_symbol_context();
            if metadata_ready {
                focus::settle_frame(&mut app, &ctx);
            } else {
                assert!(app.under_caret.is_none());
            }
            app.error = None;
            let buffer = app.open_buffer.clone().unwrap();
            let history = format!("{:?}", app.history);
            assert!(app.command_enabled(&ToolbarAction::GoToDefinition));
            app.execute_command(&ctx, ToolbarAction::GoToDefinition);
            focus::settle_frame(&mut app, &ctx);
            assert!(
                app.error
                    .as_deref()
                    .is_some_and(|error| error.contains(reason)),
                "{:?}",
                app.error
            );
            assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
            assert_eq!(format!("{:?}", app.history), history);
            assert_eq!((app.caret_line, app.caret_col), (line, column));
        }
    }
}
