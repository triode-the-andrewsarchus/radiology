use super::*;

#[test]
fn outline_explanations_follow_the_current_language_and_test_projection() {
    let (_dir, mut app) = opened_repository();
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
        std::fs::write(app.repository_root.as_ref().unwrap().join(path), text).unwrap();
        app.session.source_filters.hides_tests = true;
        app.open_file(path);
        settle(&mut app);
        rendered_frame(&mut app, &ctx);
        let output = rendered_frame(&mut app, &ctx);
        let painted = crate::ui::test_support::text_shapes(&output)
            .map(|t| t.galley.job.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(painted.contains(explanation), "{path}: {painted}");
        crate::ui::test_support::text_center(&output, "No Declaration");
        let declaration_hint = if path == "README.md" {
            "This file has no parsed structure."
        } else {
            "Place the caret inside a function, type, or other declaration."
        };
        assert!(painted.contains(declaration_hint), "{path}: {painted}");
        assert!(
            !crate::ui::test_support::text_shapes(&output).any(|text| text
                .galley
                .job
                .text
                .starts_with("References"))
        );
        if path == "hidden.rs" {
            assert!(painted.contains("Ctrl+Alt+H"));
            app.execute_command(&ctx, ToolbarAction::ToggleHideTests);
            let output = rendered_frame(&mut app, &ctx);
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
    let (_dir, mut app) = opened_repository();
    let text = "// Empty structure\n";
    std::fs::write(app.repository_root.as_ref().unwrap().join("empty.rs"), text).unwrap();
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
    rendered_frame(&mut app, &ctx);
    let output = rendered_frame(&mut app, &ctx);
    assert!(crate::ui::test_support::text_shapes(&output).any(|t| t.galley.job.text == message));
    assert!(!crate::ui::test_support::text_shapes(&output).any(|t| t
        .galley
        .job
        .text
        .contains("No declarations were found")));
    crate::ui::test_support::text_center(&output, "No Declaration");
    assert!(crate::ui::test_support::text_shapes(&output)
        .any(|text| text.galley.job.text == "This file has no parsed structure."));
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
    let output = rendered_frame(&mut app, &ctx);
    crate::ui::test_support::text_center(&output, "No File");
    assert!(
        !crate::ui::test_support::text_shapes(&output).any(|text| text
            .galley
            .job
            .text
            .starts_with("References"))
    );
}

#[test]
fn semantic_explanations_follow_index_coverage_source_identity_and_revision() {
    let (_dir, mut app) = reader_with_reference_index();
    let ctx = Context::default();
    app.show_inspector = true;
    let original = app.semantic_index.clone().unwrap();
    let rendered_text = |app: &mut RadiologyApp| {
        rendered_frame(app, &ctx);
        let output = rendered_frame(app, &ctx);
        crate::ui::test_support::text_shapes(&output)
            .map(|t| t.galley.job.text.clone())
            .collect::<Vec<_>>()
    };
    for (index, revision, expected) in [
        (None, None, "No semantic index"),
        (
            Some(Arc::new(crate::semantic::SemanticIndex::default())),
            None,
            "No semantic index",
        ),
        (
            Some(Arc::new(crate::semantic::SemanticIndex::from_scip(
                ScipIndex {
                    documents: vec![ScipDocument {
                        relative_path: "other.rs".into(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            ))),
            None,
            "File not indexed",
        ),
        (
            {
                let mut stale = (*original).clone();
                stale
                    .source_digests
                    .insert("a.rs".into(), "different bytes".into());
                Some(Arc::new(stale))
            },
            None,
            "Source changed",
        ),
        (
            Some(original.clone()),
            Some(ReviewRevision::Before),
            "Before not indexed",
        ),
    ] {
        app.semantic_index = index;
        Arc::make_mut(app.open_buffer.as_mut().unwrap()).revision = revision;
        app.refresh_symbol_context();
        settle(&mut app);
        let texts = rendered_text(&mut app);
        assert_eq!(
            texts.iter().filter(|t| t.as_str() == expected).count(),
            2,
            "{texts:?}"
        );
        assert!(
            !texts.iter().any(|t| t.contains("No references shown")),
            "{texts:?}"
        );
    }
    app.semantic_index = Some(original);
    Arc::make_mut(app.open_buffer.as_mut().unwrap()).revision = None;
    app.refresh_symbol_context();
    settle(&mut app);
    let texts = rendered_text(&mut app);
    assert!(
        !texts.iter().any(
            |t| ["No semantic index", "Source changed", "Before not indexed"].contains(&t.as_str())
        ),
        "{texts:?}"
    );
    assert!(app.reference_context.target().is_some());
}

#[test]
fn recoverable_structure_warnings_follow_the_displayed_source_and_clear_on_repair() {
    let (_dir, mut app) = opened_repository();
    let ctx = Context::default();
    for &(path, original) in crate::syntax::test_fixtures::MALFORMED_SOURCE_FIXTURES {
        // Retain the app-specific proof that recovery keeps an earlier good item.
        let text = if path == "broken.rs" {
            "fn preserved() {}\nfn broken( { let x = ; }"
        } else {
            original
        };
        std::fs::write(app.repository_root.as_ref().unwrap().join(path), text).unwrap();
        app.open_file(path);
        settle(&mut app);
        assert_eq!(app.open_buffer.as_ref().unwrap().text, text);
        assert!(
            app.syntax_error.is_none(),
            "recoverable parse is not a failed analysis: {path}"
        );
        assert!(
            app.call_sites.parse_errors,
            "missing recovered-error fact for {path}"
        );
        rendered_frame(&mut app, &ctx);
        let output = rendered_frame(&mut app, &ctx);
        crate::ui::test_support::assert_visible_text(&output, "Partial structure");
        assert!(!crate::ui::test_support::text_shapes(&output)
            .any(|text| text.galley.job.text == "Structure unavailable"));
        if path == "broken.rs" {
            assert!(app
                .outline_symbols
                .iter()
                .any(|symbol| symbol.name == "preserved"));
        }
    }
    let calls = app.call_sites.clone();
    let assert_retained_warning = |app: &mut RadiologyApp| {
        settle(app);
        assert!(Arc::ptr_eq(&calls, &app.call_sites));
        let output = rendered_frame(app, &ctx);
        crate::ui::test_support::assert_visible_text(&output, "Partial structure");
    };
    app.execute_command(&ctx, ToolbarAction::ToggleTheme);
    assert_retained_warning(&mut app);
    app.open_file("missing.rs");
    assert_retained_warning(&mut app);
    app.syntax_error = Some("The outline query could not complete.".into());
    let output = rendered_frame(&mut app, &ctx);
    crate::ui::test_support::assert_visible_text(&output, "Structure unavailable");
    assert!(!crate::ui::test_support::text_shapes(&output)
        .any(|text| text.galley.job.text == "Partial structure"));
    std::fs::write(
        app.repository_root.as_ref().unwrap().join("broken.kts"),
        "fun repaired() {}\n",
    )
    .unwrap();
    app.execute_command(&ctx, ToolbarAction::ReloadSource);
    settle(&mut app);
    assert!(!app.call_sites.parse_errors && app.syntax_error.is_none());
    let output = rendered_frame(&mut app, &ctx);
    assert!(!crate::ui::test_support::text_shapes(&output).any(|text| [
        "Partial structure",
        "Structure unavailable"
    ]
    .contains(&text.galley.job.text.as_str())));
    app.reset_repository_state();
    assert!(!app.call_sites.parse_errors && app.syntax_error.is_none());
}
