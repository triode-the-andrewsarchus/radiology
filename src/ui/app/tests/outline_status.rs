use super::*;

fn frame(app: &mut RadiologyApp, ctx: &Context) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1100.0, 800.0),
            )),
            ..Default::default()
        },
        |ctx| app.render(ctx),
    )
}

#[test]
fn outline_explanations_follow_the_current_language_and_test_projection() {
    let (dir, mut app) = opened_repository();
    let ctx = Context::default();
    for (path, text, explanation) in [
        (
            "empty.rs",
            "// No declarations\n",
            "No declarations were found in this Rust file.",
        ),
        (
            "README.md",
            "# Notes\n```rust\nfn phantom() {}\n```\ndef example():\n",
            "No structural parser is available for this file.",
        ),
        (
            "empty.kt",
            "// No declarations\n",
            "No declarations were found in this Kotlin file.",
        ),
        (
            "hidden.rs",
            "// Origin\n#[test]\nfn hidden() {}\n",
            "Every declaration in this file is hidden test code.",
        ),
    ] {
        std::fs::write(dir.path().join(path), text).unwrap();
        app.session.source_filters.hides_tests = true;
        app.open_file(path);
        settle(&mut app);
        frame(&mut app, &ctx);
        let output = frame(&mut app, &ctx);
        let painted = crate::ui::test_support::text_shapes(&output)
            .map(|t| t.galley.job.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(painted.contains(explanation), "{path}: {painted}");
        if path == "hidden.rs" {
            assert!(painted.contains("Ctrl+Alt+H"));
            app.execute_command(&ctx, ToolbarAction::ToggleHideTests);
            let output = frame(&mut app, &ctx);
            assert!(crate::ui::test_support::text_shapes(&output)
                .any(|t| t.galley.job.text == "hidden"));
            assert!(!crate::ui::test_support::text_shapes(&output).any(|t| t
                .galley
                .job
                .text
                .contains(explanation)));
        }
    }
}

#[test]
fn parser_failure_is_published_with_source_and_clears_on_success_and_repository_reset() {
    let (dir, mut app) = opened_repository();
    let text = "// Empty structure\n";
    std::fs::write(dir.path().join("empty.rs"), text).unwrap();
    app.open_file("empty.rs");
    app.source_loader.stop(); // Deliver the native-parser failure outcome at the normal publication boundary.
    let buffer = Arc::new(SourceBuffer::from_str("empty.rs", text));
    let mut syntax = crate::syntax::DocumentSyntax::analyze(&buffer);
    let message = "Could not read Rust structure. Source remains available.";
    syntax.parse_error = Some(message.into());
    app.publish_source(crate::source::loader::PreparedSource {
        destination: buffer.location_at_byte(0),
        buffer: buffer.clone(),
        syntax: Some(syntax),
        initial_folds: None,
        changes: None,
        highlights: Vec::new(),
        dark_mode: app.theme.is_dark,
    });
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    assert_eq!(app.syntax_error.as_deref(), Some(message));
    let ctx = Context::default();
    frame(&mut app, &ctx);
    let output = frame(&mut app, &ctx);
    assert!(crate::ui::test_support::text_shapes(&output).any(|t| t.galley.job.text == message));
    assert!(!crate::ui::test_support::text_shapes(&output).any(|t| t
        .galley
        .job
        .text
        .contains("No declarations were found")));
    app.execute_command(&ctx, ToolbarAction::ToggleTheme);
    settle(&mut app);
    assert_eq!(app.syntax_error.as_deref(), Some(message));
    app.open_file("missing.rs");
    settle(&mut app);
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    assert_eq!(app.syntax_error.as_deref(), Some(message));
    app.execute_command(&ctx, ToolbarAction::ReloadSource);
    settle(&mut app);
    assert_eq!(app.open_buffer.as_ref().unwrap().text, text);
    assert!(app.syntax_error.is_none());
    app.syntax_error = Some(message.into());
    app.reset_repository_state();
    assert!(app.syntax_error.is_none());
}
