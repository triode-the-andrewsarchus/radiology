use super::*;
use crate::semantic::scip_proto::{ScipDocument, ScipIndex, ScipOccurrence, ScipRange};

fn reader() -> (tempfile::TempDir, RadiologyApp) {
    let dir = tempfile::tempdir().unwrap();
    let session = SessionStore::load_from(dir.path().join("settings/session.json"));
    (dir, RadiologyApp::from_session(session))
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
    app.jump_to_location("a.rs", 1, 5);
    app.jump_to_location("b.rs", 2, 0);
    app.execute_command(&Context::default(), ToolbarAction::NavigateBack);
    assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
    assert_eq!(app.caret_col, 5);
    app.execute_command(&Context::default(), ToolbarAction::NavigateForward);
    assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
    assert_eq!(app.caret_line, 2);
}

#[test]
fn navigation_reveals_fold_and_failed_open_preserves_current_document() {
    let (dir, mut app) = reader();
    repository(dir.path());
    app.open_repository(dir.path());
    app.fold_store.toggle(1);
    app.jump_to_location("a.rs", 2, 0);
    assert!(!app.fold_store.is_line_hidden(2, &app.fold_ranges));
    app.open_file("missing.rs");
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
    app.semantic_index = Some(SemanticIndex::default());
    app.pin_store
        .toggle_pin("alpha", std::iter::once(3..8).collect());
    app.active_revision = Some(ReviewRevision::Before);
    app.repository_search.running = true;
    app.open_repository(&dir.path().join("empty"));
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
    app.jump_to_location("a.rs", 2, 0);
    let ctx = Context::default();
    app.execute_command(&ctx, ToolbarAction::ToggleBookmark);
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
    app.open_file_internal("a.rs", false);
    app.execute_command(&ctx, ToolbarAction::OpenBookmark(0));
    assert_eq!(app.caret_line, 3);
    app.execute_command(&ctx, ToolbarAction::ToggleBookmark);
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
    app.execute_command(&Context::default(), ToolbarAction::GoToDefinition);
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
    app.execute_command(&ctx, ToolbarAction::ToggleHideTests);
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
    app.open_file("a file.rs");
    let ctx = Context::default();
    app.execute_command(&ctx, ToolbarAction::ToggleBookmark);
    assert_eq!(app.bookmarks.bookmarks.len(), 1);
    app.accept_review(snapshot);
    assert!(app.bookmarks.bookmarks.is_empty());
    assert_eq!(
        app.open_buffer.as_ref().unwrap().text,
        "one\nchanged\nthree\n"
    );
    app.jump_to_location("a file.rs", 2, 0);
    app.execute_command(&ctx, ToolbarAction::ToggleRevision(ReviewRevision::Before));
    assert_eq!(app.open_buffer.as_ref().unwrap().text, "one\ntwo\nthree\n");
    assert_eq!(app.caret_line, 2);
    app.execute_command(&ctx, ToolbarAction::NavigateBack);
    assert_eq!(
        app.open_buffer.as_ref().unwrap().revision,
        Some(ReviewRevision::After)
    );
    app.open_file("removed.rs");
    assert_eq!(
        app.open_buffer.as_ref().unwrap().revision,
        Some(ReviewRevision::Before)
    );
    app.open_file("added.rs");
    assert_eq!(
        app.open_buffer.as_ref().unwrap().revision,
        Some(ReviewRevision::After)
    );
    app.execute_command(&ctx, ToolbarAction::ToggleBookmark);
    app.execute_command(&ctx, ToolbarAction::ExitPRReview);
    assert!(app.review.is_none());
    assert_eq!(app.bookmarks.bookmarks.len(), 1);
    app.open_file("a file.rs");
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
    assert_eq!(app.repository_root, Some(dir.path().join("explicit")));
    assert!(app.pending_pr.is_none());
    assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
}
