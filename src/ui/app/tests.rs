use super::*;
use crate::semantic::scip_proto::{ScipDocument, ScipIndex, ScipOccurrence, ScipRange};
use crate::syntax::outline::SymbolExtractor;

fn reader() -> (tempfile::TempDir, RadiologyApp) {
    let dir = tempfile::tempdir().unwrap();
    let session = SessionStore::load_from(dir.path().join("settings/session.json"));
    (dir, RadiologyApp::from_session(session))
}

fn settle(app: &mut RadiologyApp) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let ctx = Context::default();
    while app.is_source_loading() {
        app.poll_source(&ctx);
        assert!(
            std::time::Instant::now() < deadline,
            "source preparation timed out"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn repository(root: &Path) {
    std::fs::create_dir_all(root).unwrap();
    std::fs::write(root.join("a.rs"), "fn alpha() {\n    let café = 1;\n}\n").unwrap();
    std::fs::write(root.join("b.rs"), "fn beta() {\n    alpha();\n}\n").unwrap();
}

#[test]
fn cross_file_navigation_preserves_departure_column_without_file_start_stop() {
    let (dir, mut app) = reader();
    repository(&dir.path().join("repo"));
    app.open_repository(&dir.path().join("repo"));
    settle(&mut app);
    app.jump_to_location("a.rs", 1, 5);
    settle(&mut app);
    app.jump_to_location("b.rs", 2, 0);
    settle(&mut app);
    app.execute_command(&Context::default(), ToolbarAction::NavigateBack);
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
    assert_eq!(app.caret_col, 5);
    app.execute_command(&Context::default(), ToolbarAction::NavigateForward);
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
    assert_eq!(app.caret_line, 2);
}

#[test]
fn navigation_reveals_fold_and_failed_open_preserves_current_document() {
    let (dir, mut app) = reader();
    repository(dir.path());
    app.open_repository(dir.path());
    settle(&mut app);
    app.fold_store.toggle(1);
    app.jump_to_location("a.rs", 2, 0);
    settle(&mut app);
    assert!(!app.fold_store.is_line_hidden(2, &app.fold_ranges));
    app.open_file("missing.rs");
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
    assert_eq!(app.caret_line, 2);
    assert!(app.error.as_ref().unwrap().contains("missing.rs"));
}

#[test]
fn repository_transition_clears_prior_index_pins_review_history_and_search() {
    let (dir, mut app) = reader();
    repository(&dir.path().join("first"));
    std::fs::create_dir(dir.path().join("empty")).unwrap();
    app.open_repository(&dir.path().join("first"));
    settle(&mut app);
    app.semantic_index = Some(SemanticIndex::default());
    app.pin_store
        .toggle_pin("alpha", std::iter::once(3..8).collect());
    app.active_revision = Some(ReviewRevision::Before);
    app.repository_search.running = true;
    app.open_repository(&dir.path().join("empty"));
    settle(&mut app);
    assert!(app.open_buffer.is_none());
    assert!(app.semantic_index.is_none());
    assert!(app.pin_store.is_empty());
    assert!(app.active_revision.is_none());
    assert!(app.history.current().is_none());
    assert!(!app.repository_search.running);
    assert!(app.repository_search.matches.is_empty());
}

#[test]
fn bookmark_toolbar_mutations_persist_and_navigation_relocates() {
    let (dir, mut app) = reader();
    let root = dir.path().join("repo");
    repository(&root);
    app.open_repository(&root);
    settle(&mut app);
    app.jump_to_location("a.rs", 2, 0);
    settle(&mut app);
    let ctx = Context::default();
    app.execute_command(&ctx, ToolbarAction::ToggleBookmark);
    settle(&mut app);
    let scope = app.bookmark_scope().unwrap();
    assert_eq!(
        BookmarkStore::load_in(&dir.path().join("settings"), &scope)
            .bookmarks
            .len(),
        1
    );
    std::fs::write(
        root.join("a.rs"),
        "// inserted\nfn alpha() {\n    let café = 1;\n}\n",
    )
    .unwrap();
    app.execute_command(&ctx, ToolbarAction::RefreshPR);
    settle(&mut app);
    app.execute_command(&ctx, ToolbarAction::OpenBookmark(0));
    settle(&mut app);
    assert_eq!(app.caret_line, 3);
    app.execute_command(&ctx, ToolbarAction::ToggleBookmark);
    settle(&mut app);
    // A relocated bookmark must be removed at its current location, not duplicated.
    assert!(app.bookmarks.bookmarks.is_empty());
    assert!(BookmarkStore::load_in(&dir.path().join("settings"), &scope)
        .bookmarks
        .is_empty());
}

#[test]
fn local_utf16_definition_uses_document_identity_and_exact_column() {
    let (dir, mut app) = reader();
    std::fs::write(dir.path().join("a.rs"), "😀 val val\n").unwrap();
    app.open_repository(dir.path());
    settle(&mut app);
    let mut document = ScipDocument {
        relative_path: "a.rs".into(),
        position_encoding: 2,
        ..Default::default()
    };
    for (start, end, roles) in [(3, 6, 1), (7, 10, 0)] {
        document.occurrences.push(ScipOccurrence {
            symbol: "local 1".into(),
            symbol_roles: roles,
            range: Some(ScipRange {
                start_line: 0,
                start_col: start,
                end_line: 0,
                end_col: end,
            }),
            ..Default::default()
        });
    }
    app.semantic_index = Some(SemanticIndex::from_scip(ScipIndex {
        documents: vec![document],
        ..Default::default()
    }));
    app.jump_to_location("a.rs", 1, 9);
    settle(&mut app);
    app.execute_command(&Context::default(), ToolbarAction::GoToDefinition);
    settle(&mut app);
    assert_eq!(app.caret_col, 2);
    assert_eq!(app.history.current().unwrap().byte_offset, 5);
    let buffer = app.open_buffer.as_ref().unwrap();
    assert!(app
        .semantic_index
        .as_ref()
        .unwrap()
        .occurrence_at_byte(buffer, 8)
        .is_none());
}

#[test]
fn filter_and_wrap_commands_persist_without_another_repository_open() {
    let (dir, mut app) = reader();
    let ctx = Context::default();
    app.execute_command(&ctx, ToolbarAction::ToggleWrapLines);
    settle(&mut app);
    app.execute_command(&ctx, ToolbarAction::ToggleHideTests);
    settle(&mut app);
    let saved = SessionStore::load_from(dir.path().join("settings/session.json"));
    assert!(saved.wrap_lines);
    assert!(saved.source_filters.hides_tests);
}

#[test]
fn review_navigation_reads_pinned_revisions_and_restores_checkout_scope_on_exit() {
    let (dir, root, snapshot) = crate::pull_request::review::tests::fixture();
    let mut app = RadiologyApp::from_session(SessionStore::load_from(
        dir.path().join("settings/session.json"),
    ));
    app.open_repository(&root);
    settle(&mut app);
    app.open_file("a file.rs");
    settle(&mut app);
    let ctx = Context::default();
    app.execute_command(&ctx, ToolbarAction::ToggleBookmark);
    settle(&mut app);
    assert_eq!(app.bookmarks.bookmarks.len(), 1);
    app.accept_review(snapshot);
    settle(&mut app);
    assert!(app.bookmarks.bookmarks.is_empty());
    assert_eq!(
        app.open_buffer.as_ref().unwrap().text,
        "one\nchanged\nthree\n"
    );
    app.jump_to_location("a file.rs", 2, 0);
    settle(&mut app);
    app.execute_command(&ctx, ToolbarAction::ToggleRevision(ReviewRevision::Before));
    settle(&mut app);
    assert_eq!(app.open_buffer.as_ref().unwrap().text, "one\ntwo\nthree\n");
    assert_eq!(app.caret_line, 2);
    app.execute_command(&ctx, ToolbarAction::NavigateBack);
    settle(&mut app);
    assert_eq!(
        app.open_buffer.as_ref().unwrap().revision,
        Some(ReviewRevision::After)
    );
    app.open_file("removed.rs");
    settle(&mut app);
    assert_eq!(
        app.open_buffer.as_ref().unwrap().revision,
        Some(ReviewRevision::Before)
    );
    app.open_file("added.rs");
    settle(&mut app);
    assert_eq!(
        app.open_buffer.as_ref().unwrap().revision,
        Some(ReviewRevision::After)
    );
    app.execute_command(&ctx, ToolbarAction::ToggleBookmark);
    settle(&mut app);
    app.execute_command(&ctx, ToolbarAction::ExitPRReview);
    settle(&mut app);
    assert!(app.review.is_none());
    assert_eq!(app.bookmarks.bookmarks.len(), 1);
    app.open_file("a file.rs");
    settle(&mut app);
    assert_eq!(app.open_buffer.as_ref().unwrap().text, "checkout only\n");
    assert!(
        SessionStore::load_from(dir.path().join("settings/session.json"))
            .active_pr
            .is_none()
    );
}

#[test]
fn explicit_launch_root_supersedes_saved_review_without_starting_it() {
    let (dir, mut app) = reader();
    repository(&dir.path().join("saved"));
    repository(&dir.path().join("explicit"));
    app.session
        .record_repository(dir.path().join("saved").display().to_string());
    app.session.set_active_pr(Some(123));
    app.launch(Some(&dir.path().join("explicit")), None, Some("b.rs"));
    settle(&mut app);
    assert_eq!(app.repository_root, Some(dir.path().join("explicit")));
    assert!(app.pending_pr.is_none());
    assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
}

#[test]
fn source_zoom_commands_persist_without_changing_other_ui_fonts() {
    let (dir, mut app) = reader();
    let ctx = Context::default();
    let body = ctx.style().text_styles[&egui::TextStyle::Body].clone();
    for _ in 0..3 {
        app.execute_command(&ctx, ToolbarAction::IncreaseSourceFont);
        settle(&mut app);
    }
    assert_eq!(app.session.source_font_size, 17.0);
    assert_eq!(
        SessionStore::load_from(dir.path().join("settings/session.json")).source_font_size,
        17.0
    );
    app.execute_command(&ctx, ToolbarAction::ResetSourceFont);
    settle(&mut app);
    assert_eq!(app.session.source_font_size, 14.0);
    assert_eq!(ctx.style().text_styles[&egui::TextStyle::Body], body);
}

#[test]
fn lexical_pins_use_whole_identifiers_and_the_enclosing_function() {
    let (_dir, mut app) = reader();
    let text = "fn first() { let value = 1; let values = value; }\nfn second() { let value = 2; }";
    let buffer = SourceBuffer::from_str("pins.rs", text);
    app.outline_symbols = SymbolExtractor::extract(&buffer);
    app.open_buffer = Some(Arc::new(buffer));
    app.pin_at_byte(text.find("value").unwrap());
    let ranges = &app.pin_store.groups[0].ranges;
    assert_eq!(ranges.len(), 2);
    assert!(ranges.iter().all(|range| &text[range.clone()] == "value"));
    assert!(ranges
        .iter()
        .all(|range| range.end < text.find("fn second").unwrap()));
    app.pin_at_byte(text.rfind("value").unwrap());
    assert_eq!(app.pin_store.groups.len(), 2);
}

#[test]
fn hide_tests_preserves_manual_folds_and_explicit_navigation_reveals_source() {
    let (dir, mut app) = reader();
    let text = "fn production() {\n 1;\n}\n#[cfg(test)]\nmod checks {\n fn hidden_only() {}\n}\nfn tail() {}\n";
    std::fs::write(dir.path().join("mixed.rs"), text).unwrap();
    app.open_repository(dir.path());
    settle(&mut app);
    app.open_file("mixed.rs");
    settle(&mut app);
    app.fold_store.toggle(1);
    let ctx = Context::default();
    app.execute_command(&ctx, ToolbarAction::ToggleHideTests);
    settle(&mut app);
    let buffer = app.open_buffer.as_ref().unwrap().clone();
    let hidden = text.find("fn hidden_only").unwrap();
    assert!(app
        .test_code
        .hides_declaration(&(hidden..hidden + "fn hidden_only() {}".len())));
    assert_eq!(app.fold_store.folded_lines, HashSet::from([1]));
    app.navigate(buffer.location_at_byte(hidden + 3), true);
    settle(&mut app);
    assert_eq!(app.caret_line, 6);
    assert_eq!(app.caret_col, 4);
    assert!(!app
        .test_code
        .hides_declaration(&(hidden..hidden + "fn hidden_only() {}".len())));
    assert_eq!(app.fold_store.folded_lines, HashSet::from([1]));
    app.execute_command(&ctx, ToolbarAction::ToggleHideTests);
    settle(&mut app);
    app.execute_command(&ctx, ToolbarAction::ToggleHideTests);
    settle(&mut app);
    assert!(app.test_code.status().is_some());
    assert_eq!(app.fold_store.folded_lines, HashSet::from([1]));
}

#[test]
fn pending_source_keeps_the_reader_and_history_until_success_or_cancellation() {
    let (dir, mut app) = reader();
    repository(dir.path());
    app.open_repository(dir.path());
    settle(&mut app);
    app.jump_to_location("a.rs", 1, 5);
    let original = app.open_buffer.as_ref().unwrap().clone();
    let history = app.history.current().cloned();
    app.open_file("b.rs");
    assert!(app.is_source_loading());
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &original));
    assert_eq!(app.history.current(), history.as_ref());
    app.cancel_source_load();
    app.poll_source(&Context::default());
    assert!(!app.is_source_loading());
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &original));
    assert_eq!(app.history.current(), history.as_ref());
    app.open_file("b.rs");
    app.caret_col = 7; // The current source remains interactive while reading b.rs.
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
    std::fs::remove_file(dir.path().join("a.rs")).unwrap();
    let before_back = app.history.current().cloned();
    app.execute_command(&Context::default(), ToolbarAction::NavigateBack);
    assert_eq!(app.history.current(), before_back.as_ref());
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
    assert_eq!(app.history.current(), before_back.as_ref());
    assert!(app.error.is_some());
    repository(dir.path());
    app.execute_command(&Context::default(), ToolbarAction::NavigateBack);
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
    assert_eq!(app.caret_col, 7);
}

#[test]
fn current_document_navigation_and_repository_changes_retire_stale_reads() {
    let (dir, mut app) = reader();
    repository(&dir.path().join("first"));
    std::fs::create_dir(dir.path().join("second")).unwrap();
    std::fs::write(dir.path().join("second/new.rs"), "fn new_repository() {}\n").unwrap();
    app.open_repository(&dir.path().join("first"));
    settle(&mut app);
    app.open_file("b.rs");
    app.jump_to_location("a.rs", 2, 0);
    assert!(!app.is_source_loading());
    assert_eq!(app.caret_line, 2);
    app.open_file("b.rs");
    app.open_repository(&dir.path().join("second"));
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("new.rs"));
    assert_eq!(
        app.open_buffer.as_ref().unwrap().text,
        "fn new_repository() {}\n"
    );
    assert_eq!(app.history.current().unwrap().path, "new.rs");
}

#[test]
fn appearance_changes_keep_pending_navigation_and_recolor_immutable_current_bytes() {
    let (dir, mut app) = reader();
    repository(dir.path());
    app.open_repository(dir.path());
    settle(&mut app);
    app.open_file("b.rs");
    app.execute_command(&Context::default(), ToolbarAction::ToggleTheme);
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
    assert!(!app.theme.is_dark);
    let buffer = app.open_buffer.as_ref().unwrap().clone();
    let expected = app.highlighter.highlight_document(
        app.highlighter.find_syntax_for_path("b.rs"),
        &buffer.text,
        false,
    );
    assert_eq!(app.highlighted_lines, expected);
    app.fold_store.toggle(1);
    let location = app.history.current().cloned();
    std::fs::remove_file(dir.path().join("b.rs")).unwrap();
    app.execute_command(&Context::default(), ToolbarAction::ToggleTheme);
    settle(&mut app);
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    assert!(app.fold_store.is_folded(1));
    assert_eq!(app.history.current(), location.as_ref());
    assert!(app.error.is_none());
}

#[test]
fn a_default_file_open_keeps_first_test_region_hidden_until_explicit_navigation() {
    let (dir, mut app) = reader();
    std::fs::write(
        dir.path().join("mixed.rs"),
        "#[test]\nfn check() { assert!(true); }\nfn production() {}\n",
    )
    .unwrap();
    app.session.source_filters.hides_tests = true;
    app.open_repository(dir.path());
    settle(&mut app);
    assert_eq!(
        app.test_code.status().as_deref(),
        Some("1 test region hidden")
    );
    app.jump_to_location("mixed.rs", 1, 0);
    assert!(app.test_code.status().is_none());
}
