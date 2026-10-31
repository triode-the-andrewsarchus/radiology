use super::*;
use crate::semantic::scip_proto::{ScipDocument, ScipIndex, ScipOccurrence, ScipRange};

fn reader() -> (tempfile::TempDir, RadiologyApp) {
    let dir = tempfile::tempdir().unwrap();
    let session = SessionStore::load_from(dir.path().join("settings/session.json"));
    (dir, RadiologyApp::from_session(session))
}

fn settle(app: &mut RadiologyApp) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let ctx = Context::default();
    while app.pending_pr.is_some()
        || app.is_repository_opening()
        || app.is_git_refreshing()
        || app.is_source_loading()
        || app.is_index_loading()
        || app.is_critical_loading()
        || app.is_marks_loading()
        || app.is_reference_loading()
    {
        app.poll_review(&ctx);
        app.poll_repository(&ctx);
        app.poll_git_refresh(&ctx);
        app.poll_source(&ctx);
        app.poll_index(&ctx);
        assert!(
            std::time::Instant::now() < deadline,
            "source preparation timed out"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn settle_repository(app: &mut RadiologyApp) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while app.is_repository_opening() {
        app.poll_repository(&Context::default());
        assert!(
            std::time::Instant::now() < deadline,
            "repository discovery timed out"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn repository(root: &Path) {
    std::fs::create_dir_all(root).unwrap();
    std::fs::write(root.join("a.rs"), "fn alpha() {\n    let café = 1;\n}\n").unwrap();
    std::fs::write(root.join("b.rs"), "fn beta() {\n    alpha();\n}\n").unwrap();
}

fn opened_repository() -> (tempfile::TempDir, RadiologyApp) {
    let (dir, mut app) = reader();
    repository(dir.path());
    app.open_repository(dir.path());
    settle(&mut app);
    (dir, app)
}

#[cfg(unix)]
fn rust_indexer_fixture(settings: &Path, document: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    assert!(document.len() < 126);
    let executable = settings.join("settings/tools/bin/rust-analyzer");
    std::fs::create_dir_all(executable.parent().unwrap()).unwrap();
    let payload = [
        vec![0x12, (document.len() + 2) as u8, 0x0a, document.len() as u8],
        document.as_bytes().to_vec(),
    ]
    .concat();
    std::fs::write(executable.with_file_name("fixture.scip"), payload).unwrap();
    std::fs::write(
        &executable,
        r#"#!/bin/sh
if [ "${1:-}" = --version ]; then printf 'rust-analyzer fixture\n'; exit 0; fi
while [ "$#" -gt 0 ]; do case "$1" in --output) shift; output=$1;; esac; shift; done
printf artifact > artifact.bin
/bin/cat "$(/usr/bin/dirname "$0")/fixture.scip" > "$output"
"#,
    )
    .unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
    executable
}

fn assert_folded_headers(app: &RadiologyApp, expected: &[usize]) {
    assert_eq!(
        app.fold_store.collapsed_headers(&app.fold_ranges),
        expected.iter().copied().collect()
    );
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
    let (_dir, mut app) = opened_repository();
    assert!(app.fold_store.is_header_folded(1, &app.fold_ranges));
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
    app.semantic_index = Some(Arc::new(SemanticIndex::default()));
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
    app.semantic_index = Some(Arc::new(SemanticIndex::from_scip(ScipIndex {
        documents: vec![document],
        ..Default::default()
    })));
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
    let syntax = crate::syntax::DocumentSyntax::analyze(&buffer);
    app.outline_symbols = syntax.symbols;
    app.syntactic_occurrences = syntax.occurrences;
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
    app.fold_store.unfold_all();
    app.fold_store.toggle_header(1, &app.fold_ranges);
    let ctx = Context::default();
    app.execute_command(&ctx, ToolbarAction::ToggleHideTests);
    settle(&mut app);
    let buffer = app.open_buffer.as_ref().unwrap().clone();
    let hidden = text.find("fn hidden_only").unwrap();
    assert!(app
        .test_code
        .hides_declaration(&(hidden..hidden + "fn hidden_only() {}".len())));
    assert_folded_headers(&app, &[1]);
    app.navigate(buffer.location_at_byte(hidden + 3), true);
    settle(&mut app);
    assert_eq!(app.caret_line, 6);
    assert_eq!(app.caret_col, 4);
    assert!(!app
        .test_code
        .hides_declaration(&(hidden..hidden + "fn hidden_only() {}".len())));
    assert_folded_headers(&app, &[1]);
    app.execute_command(&ctx, ToolbarAction::ToggleHideTests);
    settle(&mut app);
    app.execute_command(&ctx, ToolbarAction::ToggleHideTests);
    settle(&mut app);
    assert!(app.test_code.status().is_some());
    assert_folded_headers(&app, &[1]);
}

#[test]
fn pending_source_keeps_the_reader_and_history_until_success_or_cancellation() {
    let (dir, mut app) = opened_repository();
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
    let (dir, mut app) = opened_repository();
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
    assert!(app.fold_store.is_header_folded(1, &app.fold_ranges));
    app.fold_store.toggle_header(1, &app.fold_ranges);
    let location = app.history.current().cloned();
    std::fs::remove_file(dir.path().join("b.rs")).unwrap();
    app.execute_command(&Context::default(), ToolbarAction::ToggleTheme);
    settle(&mut app);
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    assert!(
        !app.fold_store.is_header_folded(1, &app.fold_ranges),
        "theme recolor must preserve explicit expansion"
    );
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

#[test]
fn imported_indexes_publish_asynchronously_and_failed_reload_keeps_prior_data() {
    use crate::semantic::loader::IndexImportStatus;
    let (dir, mut app) = reader();
    let first = dir.path().join("first");
    let second = dir.path().join("second");
    repository(&first);
    repository(&second);
    let path = first.join("index.scip");
    // A SCIP document with the repository-relative path a.rs.
    std::fs::write(&path, [0x12, 6, 0x0a, 4, b'a', b'.', b'r', b's']).unwrap();
    let ctx = Context::default();
    app.open_repository(&first);
    assert!(app.semantic_index.is_none());
    assert!(app.is_repository_opening());
    settle_repository(&mut app);
    assert!(app.is_index_loading());
    settle(&mut app);
    assert!(app
        .semantic_index
        .as_ref()
        .unwrap()
        .documents
        .contains_key("a.rs"));
    assert!(matches!(
        app.index_import_status(),
        IndexImportStatus::Imported { documents: 1, .. }
    ));

    std::fs::write(&path, [0xff]).unwrap();
    app.execute_command(&ctx, ToolbarAction::ReloadIndexImport);
    settle(&mut app);
    assert!(matches!(
        app.index_import_status(),
        IndexImportStatus::Failed(_)
    ));
    assert!(app
        .semantic_index
        .as_ref()
        .unwrap()
        .documents
        .contains_key("a.rs"));
    app.execute_command(&ctx, ToolbarAction::ReloadIndexImport);
    app.execute_command(&ctx, ToolbarAction::CancelIndexImport);
    assert_eq!(
        app.index_cache_status(),
        &crate::semantic::indexing::service::CacheLoadStatus::Cancelled
    );
    settle(&mut app);
    assert!(app.semantic_index.is_some());

    app.execute_command(&ctx, ToolbarAction::ReloadIndexImport);
    app.open_repository(&second);
    settle(&mut app);
    assert_eq!(app.index_import_status(), &IndexImportStatus::Missing);
    assert!(app.semantic_index.is_none());
}

#[test]
fn index_planning_uses_unfiltered_inventory_and_retires_previous_repository() {
    let (dir, mut app) = reader();
    let first = dir.path().join("rust");
    let second = dir.path().join("go");
    repository(&first);
    std::fs::write(
        first.join("Cargo.toml"),
        "[package]\nname = 'fixture'\nversion = '0.1.0'\n",
    )
    .unwrap();
    std::fs::create_dir(first.join("tests")).unwrap();
    std::fs::write(first.join("tests/unit.rs"), "#[test] fn works() {}\n").unwrap();
    std::fs::create_dir(&second).unwrap();
    std::fs::write(second.join("go.mod"), "module fixture\n").unwrap();
    std::fs::write(second.join("main.go"), "package main\n").unwrap();
    app.open_repository(&first);
    settle(&mut app);
    let plan = app.index_plan().clone();
    assert_eq!(plan.selected[0].source_files, 3);
    app.execute_command(&Context::default(), ToolbarAction::ToggleHideTests);
    assert_eq!(app.index_plan(), &plan);
    app.inspect_index_projects();
    app.open_repository(&second);
    settle(&mut app);
    assert_eq!(app.index_plan().selected.len(), 1);
    assert_eq!(app.index_plan().selected[0].id(), "scip-go");
    assert_eq!(app.index_plan().selected[0].source_files, 1);
}

#[cfg(unix)]
#[test]
fn tool_probes_publish_real_versions_and_are_retired_on_repository_changes() {
    use crate::semantic::indexing::{
        tools::{ToolAvailability, ToolDirectory},
        Indexer,
    };
    use std::os::unix::fs::PermissionsExt;
    let (dir, mut app) = reader();
    let first = dir.path().join("first");
    let next = dir.path().join("next");
    repository(&first);
    repository(&next);
    std::fs::write(
        first.join("Cargo.toml"),
        "[package]\nname='fixture'\nversion='0.1.0'\n",
    )
    .unwrap();
    let tools = ToolDirectory {
        managed: None,
        home: None,
        search_path: vec![dir.path().join("bin")],
    };
    std::fs::create_dir(&tools.search_path[0]).unwrap();
    let binary = tools.search_path[0].join("rust-analyzer");
    std::fs::write(&binary, "#!/bin/sh\nprintf 'rust-analyzer fixture-1'\n").unwrap();
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755)).unwrap();
    app.open_repository(&first);
    settle(&mut app);
    assert!(app.tool_availability(Indexer::Rust).is_none());
    app.probe_index_tools(tools.clone(), first.clone());
    settle(&mut app);
    assert!(
        matches!(app.tool_availability(Indexer::Rust), Some(ToolAvailability::Available { version, .. }) if version == "rust-analyzer fixture-1")
    );
    app.probe_index_tools(tools.clone(), first.clone());
    app.execute_command(&Context::default(), ToolbarAction::CancelIndexToolCheck);
    assert!(app.tool_availability(Indexer::Rust).is_some());
    app.probe_index_tools(tools, first);
    app.open_repository(&next);
    settle(&mut app);
    assert!(app.tool_availability(Indexer::Rust).is_none());
}

#[cfg(unix)]
#[test]
fn generate_command_publishes_source_provenance_preserves_prior_data_and_retires_on_switch() {
    let (dir, mut app) = reader();
    let root = dir.path().join("repo");
    let next = dir.path().join("next");
    repository(&root);
    repository(&next);
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='fixture'\nversion='0.1.0'\n",
    )
    .unwrap();
    let executable = rust_indexer_fixture(dir.path(), "a.rs");
    let ctx = Context::default();
    app.open_repository(&root);
    settle(&mut app);
    app.execute_command(&ctx, ToolbarAction::GenerateIndex);
    assert!(app.index_generator.is_running());
    assert!(app.semantic_index.is_none());
    settle(&mut app);
    assert_eq!(app.generation_report().unwrap().documents, 1);
    let digest = app.semantic_index.as_ref().unwrap().source_digests["a.rs"].clone();
    assert!(!root.join("artifact.bin").exists());
    // Reopening must restore persisted data without even a usable indexer.
    std::fs::remove_file(&executable).unwrap();
    app.open_repository(&root);
    assert!(
        app.semantic_index.is_some(),
        "accepted index stays usable during discovery"
    );
    settle_repository(&mut app);
    assert!(app.semantic_index.is_none());
    settle(&mut app);
    assert!(app.generation_report().unwrap().cached);
    assert_eq!(
        app.index_cache_status(),
        &crate::semantic::indexing::service::CacheLoadStatus::Loaded
    );
    assert_eq!(
        app.semantic_index.as_ref().unwrap().source_digests["a.rs"],
        digest
    );
    assert!(!executable.exists());
    std::fs::write(root.join("a.rs"), "fn changed() {}\n").unwrap();
    app.execute_command(&ctx, ToolbarAction::ReloadSource);
    settle(&mut app);
    assert!(!app
        .semantic_index
        .as_ref()
        .unwrap()
        .matches_buffer(app.open_buffer.as_ref().unwrap()));
    app.open_repository(&root);
    settle(&mut app);
    assert!(app.generation_report().unwrap().cached);
    assert!(app.generation_report().unwrap().stale_reason.is_some());
    assert!(!app
        .semantic_index
        .as_ref()
        .unwrap()
        .matches_buffer(app.open_buffer.as_ref().unwrap()));
    std::fs::write(&executable, "#!/bin/sh\nif [ \"${1:-}\" = --version ]; then printf 'rust-analyzer fixture\\n'; exit 0; fi\nprintf failure >&2; exit 9\n").unwrap();
    app.execute_command(&ctx, ToolbarAction::GenerateIndex);
    settle(&mut app);
    assert_eq!(app.generation_report().unwrap().documents, 0);
    assert_eq!(
        app.semantic_index.as_ref().unwrap().source_digests["a.rs"],
        digest
    );
    app.execute_command(&ctx, ToolbarAction::GenerateIndex);
    app.execute_command(&ctx, ToolbarAction::CancelIndexGeneration);
    assert!(app.index_generator.was_cancelled());
    assert!(app.semantic_index.is_some());
    app.execute_command(&ctx, ToolbarAction::GenerateIndex);
    app.open_repository(&next);
    settle(&mut app);
    assert!(app.semantic_index.is_none());
    assert!(app.generation_report().is_none());

    // Cancelling an automatic load must not launch a fallback or publish its result.
    app.open_repository(&root);
    settle_repository(&mut app);
    app.execute_command(&ctx, ToolbarAction::CancelIndexImport);
    settle(&mut app);
    assert_eq!(
        app.index_cache_status(),
        &crate::semantic::indexing::service::CacheLoadStatus::Cancelled
    );
    assert!(app.semantic_index.is_none());
    app.open_repository(&root);
    app.open_repository(&next);
    settle(&mut app);
    assert!(app.semantic_index.is_none());

    // Corrupt cache diagnostics survive a successful explicitly unverified import.
    let cache = app.session.repository_cache(&root).unwrap().join("indexes");
    let entry = std::fs::read_dir(cache)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    std::fs::write(entry, b"{truncated").unwrap();
    std::fs::write(
        root.join("index.scip"),
        [0x12, 6, 0x0a, 4, b'b', b'.', b'r', b's'],
    )
    .unwrap();
    app.open_repository(&root);
    settle(&mut app);
    assert!(matches!(
        app.index_cache_status(),
        crate::semantic::indexing::service::CacheLoadStatus::Failed(_)
    ));
    assert!(matches!(
        app.index_import_status(),
        crate::semantic::loader::IndexImportStatus::Imported { documents: 1, .. }
    ));
    assert!(app.generation_report().is_none());
    let imported = app.semantic_index.as_ref().unwrap();
    assert!(imported.documents.contains_key("b.rs"));
    assert!(imported.source_digests.is_empty());
}

#[cfg(target_os = "linux")]
#[test]
fn idle_reader_construction_does_not_create_default_configuration() {
    const MARKER: &str = "RADIOLOGY_TEST_IDLE_CONFIG";
    if let Some(directory) = std::env::var_os(MARKER) {
        let directory = PathBuf::from(directory);
        let app = RadiologyApp::from_session(SessionStore::default());
        assert!(app.repository_root.is_none());
        assert_eq!(
            app.index_tools.managed,
            Some(directory.join("radiology/tools"))
        );
        assert!(!directory.exists());
        return;
    }
    let owner = tempfile::tempdir().unwrap();
    let directory = owner.path().join("uncreated-config");
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "ui::app::tests::idle_reader_construction_does_not_create_default_configuration",
            "--nocapture",
        ])
        .env("XDG_CONFIG_HOME", &directory)
        .env(MARKER, &directory)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!directory.exists());
}

fn indexed_target(text: &str) -> (SemanticIndex, SemanticSymbolID, ScipRange) {
    let byte = text.find("target").unwrap();
    let column = text[..byte].encode_utf16().count() as i32;
    let range = ScipRange::from_packed(&[0, column, column + 6]).unwrap();
    let id = SemanticSymbolID::new("scip test fixture 1 target().", "");
    let mut index = SemanticIndex::from_scip(ScipIndex {
        documents: vec![ScipDocument {
            relative_path: "b.ts".into(),
            position_encoding: 2,
            occurrences: vec![ScipOccurrence {
                symbol: id.symbol.clone(),
                symbol_roles: 1,
                range: Some(range.clone()),
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    });
    index.source_digests.insert(
        "b.ts".into(),
        SourceBuffer::from_str("b.ts", text).content_digest().into(),
    );
    (index, id, range)
}

#[test]
fn indexed_symbol_and_reference_navigation_share_encoding_fresh_reads_and_provenance() {
    let (dir, mut app) = reader();
    repository(dir.path());
    let original = "const note = \"😀\"; function target() {}\n";
    std::fs::write(dir.path().join("b.ts"), original).unwrap();
    app.open_repository(dir.path());
    settle(&mut app);
    let (index, id, range) = indexed_target(original);
    app.semantic_index = Some(Arc::new(index));
    app.navigate_to_indexed_symbol(&id);
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("b.ts"));
    assert_eq!(
        app.caret_col,
        original[..original.find("target").unwrap()].chars().count()
    );
    app.navigate_to_indexed_range("b.ts", &range);
    settle(&mut app);
    assert_eq!(app.open_buffer.as_ref().unwrap().text, original);

    // A new index can describe newer bytes than the currently cached same-path buffer.
    let newer = "const note = \"😀 newer\"; function target() {}\n";
    std::fs::write(dir.path().join("b.ts"), newer).unwrap();
    let (index, _, range) = indexed_target(newer);
    app.semantic_index = Some(Arc::new(index));
    app.navigate_to_indexed_symbol(&id);
    assert!(app.is_source_loading());
    settle(&mut app);
    assert_eq!(app.open_buffer.as_ref().unwrap().text, newer);
    assert_eq!(
        app.caret_col,
        newer[..newer.find("target").unwrap()].chars().count()
    );

    app.open_file("a.rs");
    settle(&mut app);
    let history = format!("{:?}", app.history);
    std::fs::write(dir.path().join("b.ts"), "changed again").unwrap();
    app.navigate_to_indexed_range("b.ts", &range);
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
    assert_eq!(format!("{:?}", app.history), history);
    assert!(app.error.as_ref().unwrap().contains("bytes changed"));
    Arc::make_mut(app.semantic_index.as_mut().unwrap())
        .source_digests
        .clear();
    Arc::make_mut(app.semantic_index.as_mut().unwrap())
        .source_digests
        .insert("other.ts".into(), "missing target identity".into());
    app.navigate_to_indexed_range("b.ts", &range);
    assert!(!app.is_source_loading());
    assert!(app.error.as_ref().unwrap().contains("no source identity"));
    app.active_revision = Some(ReviewRevision::Before);
    app.navigate_to_indexed_symbol(&id);
    assert!(!app.is_source_loading());
    assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
}

#[test]
fn occurrence_pins_restore_on_return_and_invalidate_only_after_successful_changed_source() {
    let (dir, mut app) = reader();
    let root = dir.path().join("repo");
    repository(&root);
    let text =
        "fn alpha() { let one = 1; let two = 2; let three = 3; let four = 4; let five = 5; }\n";
    std::fs::write(root.join("a.rs"), text).unwrap();
    app.open_repository(&root);
    settle(&mut app);
    for word in ["one", "two", "three", "four", "five"] {
        app.pin_at_byte(text.find(word).unwrap());
    }
    assert_eq!(app.pin_store.groups.len(), 5);
    assert_eq!(app.pin_store.groups[0].label, "one");
    let original = app.pin_store.groups.clone();
    app.open_file("b.rs");
    settle(&mut app);
    assert!(app.pin_store.groups.is_empty());
    app.pin_at_byte("fn ".len());
    assert_eq!(app.pin_store.groups.len(), 1);
    app.open_file("a.rs");
    settle(&mut app);
    assert_eq!(app.pin_store.groups, original);
    app.open_file("missing.rs");
    settle(&mut app);
    assert_eq!(app.pin_store.groups, original);
    std::fs::write(root.join("a.rs"), text.replace("one", "new")).unwrap();
    app.execute_command(&Context::default(), ToolbarAction::ReloadSource);
    settle(&mut app);
    assert!(app.pin_store.groups.is_empty());
    app.open_file("b.rs");
    settle(&mut app);
    assert_eq!(app.pin_store.groups[0].label, "beta");
    app.pin_store.clear_current();
    app.open_file("a.rs");
    settle(&mut app);
    app.open_file("b.rs");
    settle(&mut app);
    assert!(app.pin_store.groups.is_empty());
}

#[test]
fn critical_mode_uses_background_calls_and_rebuilds_when_an_index_arrives() {
    let (dir, mut app) = reader();
    let root = dir.path().join("repo");
    repository(&root);
    let text = "fn demo() { clone(); unknown(); }";
    std::fs::write(root.join("a.rs"), text).unwrap();
    app.open_repository(&root);
    settle(&mut app);
    let ctx = Context::default();
    app.execute_command(&ctx, ToolbarAction::ToggleMode(SourceMode::Critical));
    assert!(app.is_critical_loading());
    assert_eq!(app.critical_status(), Some("Preparing Critical Lens…"));
    settle(&mut app);
    let projected = app.critical_projection().unwrap();
    assert!(!projected.is_receded(text.find("clone").unwrap()));
    assert!(projected.is_receded(text.find("demo").unwrap()));
    assert_eq!(app.critical_status(), Some("Call-focused"));

    // Import real SCIP wire data after the syntax-only presentation was published.
    write_critical_index(&root, text);
    let start = text.find("clone").unwrap();
    app.execute_command(&ctx, ToolbarAction::ReloadIndexImport);
    settle(&mut app);
    assert!(app.critical_projection().unwrap().is_receded(start));
    assert!(!app
        .critical_projection()
        .unwrap()
        .is_receded(text.find("unknown").unwrap()));
    app.set_source_mode(SourceMode::Normal);
    assert!(app.critical_projection().is_none());
    app.set_source_mode(SourceMode::Critical);
    app.set_source_mode(SourceMode::Normal);
    settle(&mut app);
    assert!(app.critical_projection().is_none());
    assert!(app.critical_status().is_none());
    app.set_source_mode(SourceMode::Critical);
    app.open_file("b.rs");
    settle(&mut app);
    assert!(app
        .critical_projection()
        .unwrap()
        .matches(app.open_buffer.as_ref().unwrap()));
    let next = dir.path().join("empty");
    std::fs::create_dir(&next).unwrap();
    app.open_repository(&next);
    settle(&mut app);
    assert!(app.critical_projection().is_none());
    assert!(!app.is_critical_loading());
}

fn write_critical_index(root: &Path, text: &str) {
    let symbol = b"rust-analyzer cargo core 0 clone().";
    let byte = text.find("clone").unwrap();
    let line = text[..byte].bytes().filter(|&b| b == b'\n').count() as u8;
    let line_start = text[..byte].rfind('\n').map_or(0, |p| p + 1);
    let start = (byte - line_start) as u8;
    let mut occurrence = vec![0x0a, 3, line, start, start + 5, 0x12, symbol.len() as u8];
    occurrence.extend_from_slice(symbol);
    let mut document = vec![
        0x0a,
        4,
        b'a',
        b'.',
        b'r',
        b's',
        0x12,
        occurrence.len() as u8,
    ];
    document.extend(occurrence);
    let mut index = vec![0x12, document.len() as u8];
    index.extend(document);
    std::fs::write(root.join("index.scip"), index).unwrap();
}

#[test]
fn critical_marks_refresh_without_reader_changes_and_restore_after_restart() {
    use crate::critical::overrides::{MarkEdit, MarkTarget};
    use crate::critical::Criticality;
    let (dir, mut app) = reader();
    let root = dir.path().join("repo");
    repository(&root);
    let text = "fn demo() { clone(); unknown(); }";
    std::fs::write(root.join("a.rs"), text).unwrap();
    write_critical_index(&root, text);
    app.open_repository(&root);
    settle(&mut app);
    let start = text.find("clone").unwrap();
    app.jump_to_location("a.rs", 1, start);
    settle(&mut app);
    app.pin_at_byte(start);
    app.set_source_mode(SourceMode::Critical);
    settle(&mut app);
    let source = app.open_buffer.clone();
    let history = format!("{:?}", app.history);
    let folds = format!("{:?}", app.fold_store);
    let pins = app.pin_store.groups.clone();
    let symbol = MarkTarget::Symbol("rust-analyzer cargo core 0 clone().".into());
    let package = MarkTarget::Package("rust-analyzer cargo core".into());
    for (target, decision, receded) in [
        (package.clone(), Some(Criticality::Critical), false),
        (symbol.clone(), Some(Criticality::Ordinary), true),
        (symbol.clone(), None, false),
        (package, None, true),
        (symbol.clone(), Some(Criticality::Critical), false),
    ] {
        assert!(app.apply_critical_mark(MarkEdit { target, decision }));
        settle(&mut app);
        assert_eq!(
            app.critical_projection().unwrap().is_receded(start),
            receded
        );
        assert!(Arc::ptr_eq(
            source.as_ref().unwrap(),
            app.open_buffer.as_ref().unwrap()
        ));
        assert_eq!((app.caret_line, app.caret_col), (1, start));
        assert_eq!(format!("{:?}", app.history), history);
        assert_eq!(format!("{:?}", app.fold_store), folds);
        assert_eq!(app.pin_store.groups, pins);
    }
    assert!(!app.apply_critical_mark(MarkEdit {
        target: MarkTarget::Symbol("rust-analyzer cargo core 0 wrong().".into()),
        decision: Some(Criticality::Critical),
    }));
    app.active_revision = Some(ReviewRevision::Before);
    // Validation is based on the immutable source identity, not just toolbar state.
    Arc::make_mut(app.open_buffer.as_mut().unwrap()).revision = Some(ReviewRevision::Before);
    assert!(!app.apply_critical_mark(MarkEdit {
        target: symbol.clone(),
        decision: None
    }));
    app.flush_critical_marks(std::time::Duration::from_secs(5))
        .unwrap();
    drop(app);
    let session = SessionStore::load_from(dir.path().join("settings/session.json"));
    let mut reopened = RadiologyApp::from_session(session);
    reopened.open_repository(&root);
    reopened.set_source_mode(SourceMode::Critical);
    settle(&mut reopened);
    assert_eq!(
        reopened
            .critical_overrides()
            .symbols
            .get("rust-analyzer cargo core 0 clone()."),
        Some(&Criticality::Critical)
    );
    assert!(!reopened.critical_projection().unwrap().is_receded(start));
}

#[test]
fn critical_fold_counts_refresh_with_policy_and_clear_on_mode_source_index_and_repository_changes()
{
    use crate::critical::{
        overrides::{MarkEdit, MarkTarget},
        Criticality,
    };
    let (dir, mut app) = reader();
    let root = dir.path().join("repo");
    repository(&root);
    let text = "fn demo() {\n    clone(); unknown();\n}\n";
    std::fs::write(root.join("a.rs"), text).unwrap();
    write_critical_index(&root, text);
    app.open_repository(&root);
    settle(&mut app);
    app.jump_to_location("a.rs", 2, text.find("clone").unwrap());
    settle(&mut app);
    app.fold_store.toggle_header(1, &app.fold_ranges);
    app.set_source_mode(SourceMode::Critical);
    settle(&mut app);
    let label = |app: &RadiologyApp| {
        app.critical_annotations()
            .unwrap()
            .at_line(app.open_buffer.as_ref().unwrap(), 1)
            .unwrap()
            .count
            .label()
    };
    assert_eq!(label(&app), "0+");
    let source = app.open_buffer.clone().unwrap();
    let history = format!("{:?}", app.history);
    assert!(app.apply_critical_mark(MarkEdit {
        target: MarkTarget::Symbol("rust-analyzer cargo core 0 clone().".into()),
        decision: Some(Criticality::Critical)
    }));
    assert!(app.critical_annotations().is_none());
    settle(&mut app);
    assert_eq!(label(&app), "1+");
    assert!(app.fold_store.is_header_folded(1, &app.fold_ranges));
    assert!(Arc::ptr_eq(&source, app.open_buffer.as_ref().unwrap()));
    assert_eq!(format!("{:?}", app.history), history);
    app.set_source_mode(SourceMode::Normal);
    assert!(app.critical_annotations().is_none());
    app.set_source_mode(SourceMode::Critical);
    settle(&mut app);
    assert_eq!(label(&app), "1+");
    let mut wire = std::fs::read(root.join("index.scip")).unwrap();
    let offset = wire.windows(5).position(|bytes| bytes == b"clone").unwrap();
    wire[offset..offset + 5].copy_from_slice(b"other");
    std::fs::write(root.join("index.scip"), wire).unwrap();
    app.execute_command(&Context::default(), ToolbarAction::ReloadIndexImport);
    // The current index remains usable until its replacement publishes.
    assert_eq!(label(&app), "1+");
    settle(&mut app);
    assert_eq!(label(&app), "0+");
    Arc::make_mut(app.open_buffer.as_mut().unwrap()).revision = Some(ReviewRevision::Before);
    app.refresh_critical_projection();
    settle(&mut app);
    assert!(app.critical_annotations().is_none());
    app.open_file("b.rs");
    settle(&mut app);
    assert!(app
        .critical_annotations()
        .unwrap()
        .matches(app.open_buffer.as_ref().unwrap()));
    let empty = dir.path().join("empty");
    std::fs::create_dir(&empty).unwrap();
    app.open_repository(&empty);
    settle(&mut app);
    assert!(app.critical_annotations().is_none());
    assert!(!app.is_critical_loading());
}

#[test]
fn fold_choices_restore_on_return_and_reload_but_reveal_destinations_and_reject_changed_bytes() {
    let (dir, mut app) = reader();
    let root = dir.path().join("repo");
    repository(&root);
    let text = "fn first() {\n one();\n}\nfn second() {\n two();\n}\n";
    std::fs::write(root.join("a.rs"), text).unwrap();
    app.open_repository(&root);
    settle(&mut app);
    assert_folded_headers(&app, &[1, 4]);
    app.open_file("b.rs");
    settle(&mut app);
    assert_folded_headers(&app, &[1]);
    app.fold_store.toggle_header(1, &app.fold_ranges);
    app.open_file("a.rs");
    settle(&mut app);
    assert_folded_headers(&app, &[1, 4]);
    let source = app.open_buffer.clone().unwrap();
    app.open_file("missing.rs");
    settle(&mut app);
    assert!(Arc::ptr_eq(&source, app.open_buffer.as_ref().unwrap()));
    assert_folded_headers(&app, &[1, 4]);
    app.execute_command(&Context::default(), ToolbarAction::ReloadSource);
    settle(&mut app);
    assert_folded_headers(&app, &[1, 4]);
    app.jump_to_location("a.rs", 2, text.find("one()").unwrap());
    settle(&mut app);
    assert_folded_headers(&app, &[4]);
    app.open_file("b.rs");
    settle(&mut app);
    assert!(app
        .fold_store
        .collapsed_headers(&app.fold_ranges)
        .is_empty());
    app.open_file("a.rs");
    settle(&mut app);
    assert_folded_headers(&app, &[4]);
    std::fs::write(root.join("a.rs"), text.replace("second", "newest")).unwrap();
    app.execute_command(&Context::default(), ToolbarAction::ReloadSource);
    settle(&mut app);
    assert_folded_headers(&app, &[1, 4]);
    app.fold_store.toggle_header(4, &app.fold_ranges);
    let other = dir.path().join("other");
    repository(&other);
    std::fs::write(other.join("a.rs"), text).unwrap();
    app.open_repository(&other);
    settle(&mut app);
    assert_folded_headers(&app, &[1, 4]);
}

#[test]
fn pinned_review_initial_folds_expose_changed_bodies_without_replaying_over_manual_choices() {
    let before = "fn first() {\n old();\n}\nfn second() {\n untouched();\n}\n";
    let after = before.replace("old()", "new()");
    let (dir, root, snapshot) =
        crate::pull_request::review::tests::fixture_with_sources(before, &after);
    let mut app = RadiologyApp::from_session(SessionStore::load_from(
        dir.path().join("settings/session.json"),
    ));
    app.open_repository(&root);
    settle(&mut app);
    app.open_file("a file.rs");
    settle(&mut app);
    app.accept_review(snapshot);
    settle(&mut app);
    assert_eq!(app.open_buffer.as_ref().unwrap().text, after);
    assert_folded_headers(&app, &[4]);
    app.fold_store.toggle_header(1, &app.fold_ranges);
    app.open_file("added.rs");
    settle(&mut app);
    app.open_file("a file.rs");
    settle(&mut app);
    assert_folded_headers(&app, &[1, 4]);
    app.execute_command(
        &Context::default(),
        ToolbarAction::ToggleRevision(ReviewRevision::Before),
    );
    settle(&mut app);
    assert_eq!(app.open_buffer.as_ref().unwrap().text, before);
    assert_folded_headers(&app, &[4]);
    assert!(app
        .line_changes
        .as_ref()
        .is_some_and(|changes| changes.total_deletions() == 1));
}

#[test]
fn fold_to_definitions_command_exposes_method_and_variant_headers_and_preserves_reader_state() {
    let (dir, mut app) = reader();
    let root = dir.path().join("repo");
    repository(&root);
    let text = crate::syntax::fold_queries::tests::DEFINITIONS;
    std::fs::write(root.join("a.rs"), text).unwrap();
    app.open_repository(&root);
    settle(&mut app);
    let source = app.open_buffer.clone().unwrap();
    let history = format!("{:?}", app.history);
    let caret = (app.caret_line, app.caret_col);
    app.fold_store.toggle_header(5, &app.fold_ranges);
    app.fold_store.toggle_header(14, &app.fold_ranges);
    app.execute_command(&Context::default(), ToolbarAction::FoldToDefinitions);
    assert_folded_headers(&app, &[1, 6, 15]);
    assert!(Arc::ptr_eq(&source, app.open_buffer.as_ref().unwrap()));
    assert_eq!(format!("{:?}", app.history), history);
    assert_eq!((app.caret_line, app.caret_col), caret);
    app.open_file("b.rs");
    settle(&mut app);
    app.open_file("a.rs");
    settle(&mut app);
    assert_folded_headers(&app, &[1, 6, 15]);
    assert!(app.fold_error.is_none());
}

fn reference_index() -> Arc<SemanticIndex> {
    let occurrence = |symbol: &str, role, coordinates: &[i32]| ScipOccurrence {
        symbol: format!("scip test fixture 1 {symbol}()."),
        symbol_roles: role,
        range: ScipRange::from_packed(coordinates),
        ..Default::default()
    };
    Arc::new(SemanticIndex::from_scip(ScipIndex {
        documents: vec![
            ScipDocument {
                relative_path: "a.rs".into(),
                occurrences: vec![occurrence("alpha", 1, &[0, 3, 8])],
                ..Default::default()
            },
            ScipDocument {
                relative_path: "b.rs".into(),
                occurrences: vec![
                    occurrence("beta", 1, &[0, 3, 7]),
                    occurrence("alpha", 0, &[1, 4, 9]),
                ],
                ..Default::default()
            },
        ],
        ..Default::default()
    }))
}

#[test]
fn explicit_references_remain_independent_of_navigation_and_follow_latest_declaration() {
    let (dir, mut app) = reader_with_reference_index();
    let root = dir.path().join("repo");
    app.jump_to_location("b.rs", 2, "fn beta() {\n    ".len() + 1);
    settle(&mut app);
    assert_eq!(app.reference_context.target().unwrap().name, "beta");
    let buffer = app.open_buffer.clone().unwrap();
    let location = app.history.current().cloned();
    let can_back = app.history.can_go_back();
    app.show_inspector = false;
    app.execute_command(&Context::default(), ToolbarAction::FindReferences);
    settle(&mut app);
    assert!(app.show_inspector);
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    assert_eq!((app.caret_line, app.caret_col), (2, 5));
    assert_eq!(app.history.current(), location.as_ref());
    assert_eq!(app.history.can_go_back(), can_back);
    assert_eq!(app.reference_context.explicit().unwrap().name, "alpha");
    assert_eq!(app.reference_context.results.reference_count, 1);
    assert_eq!(app.reference_context.results.definition_count, 1);
    assert_eq!(app.reference_context.results.hits[1].path, "b.rs");
    // The enclosing declaration changes independently of explicit focus.
    app.jump_to_location("a.rs", 2, "fn alpha() {\n    ".len());
    settle(&mut app);
    assert_eq!(app.reference_context.target().unwrap().name, "alpha");
    app.jump_to_location("b.rs", 1, 4);
    settle(&mut app);
    assert_eq!(app.reference_context.target().unwrap().name, "alpha");
    let location = app.history.current().cloned();
    app.follow_reference_caret();
    settle(&mut app);
    assert!(app.reference_context.explicit().is_none());
    assert_eq!(app.reference_context.target().unwrap().name, "beta");
    assert_eq!(app.reference_context.results.reference_count, 0);
    assert_eq!(app.history.current(), location.as_ref());
    app.find_references_at_byte(buffer.text.find("alpha").unwrap());
    settle(&mut app);
    assert_eq!(app.reference_context.explicit().unwrap().name, "alpha");
    app.open_repository(&root);
    settle(&mut app);
    assert!(app.reference_context.explicit().is_none());
    assert!(app.reference_context.results.hits.is_empty());
}

#[test]
fn reference_commands_reject_stale_and_before_sources_without_replacing_pinned_focus() {
    let (_dir, mut app) = reader_with_reference_index();
    app.find_references_at_caret();
    settle(&mut app);
    let target = app.reference_context.explicit().cloned();
    Arc::make_mut(app.semantic_index.as_mut().unwrap())
        .source_digests
        .insert("a.rs".into(), "different bytes".into());
    app.find_references_at_caret();
    assert!(app
        .error
        .as_ref()
        .unwrap()
        .contains("No exact indexed symbol"));
    assert_eq!(app.reference_context.explicit(), target.as_ref());
    Arc::make_mut(app.open_buffer.as_mut().unwrap()).revision = Some(ReviewRevision::Before);
    app.refresh_symbol_context();
    app.find_references_at_caret();
    assert!(!app.is_reference_loading());
    assert!(app.reference_context.results.hits.is_empty());
    assert!(!app.reference_context.available());
    assert_eq!(app.reference_context.explicit(), target.as_ref());
}

#[test]
fn definition_previews_use_real_source_and_suppress_local_definition_fetches() {
    use crate::source::preview::PreviewPhase;
    let (_dir, mut app) = reader_with_reference_index();
    app.jump_to_location("b.rs", 2, "fn beta() {\n    ".len() + 1);
    settle(&mut app);
    let buffer = app.open_buffer.clone().unwrap();
    let location = app.history.current().cloned();
    let PreviewPhase::Loaded(preview) = app.definition_preview.phase() else {
        panic!("{:?}", app.definition_preview.phase());
    };
    assert_eq!(preview.path, "a.rs");
    assert_eq!(preview.lines.len(), 3);
    assert_eq!(preview.lines[0].text, "fn alpha() {");
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    assert_eq!(app.history.current(), location.as_ref());
    app.definition_expanded = true;
    app.caret_col += 1;
    app.refresh_symbol_context();
    assert!(
        app.definition_expanded,
        "moving within a token changed disclosure"
    );
    assert!(!app.definition_preview.is_running());
    app.jump_to_location("a.rs", 1, 4);
    settle(&mut app);
    assert_eq!(app.definition_preview.phase(), &PreviewPhase::None);
    assert!(!app.definition_expanded);
}

#[test]
fn metadata_jobs_preserve_token_disclosure_and_retire_replaced_indexes() {
    let (_dir, mut app) = reader_with_reference_index();
    let original = app.under_caret.clone().unwrap();
    app.definition_expanded = true;
    app.caret_col += 1;
    app.refresh_symbol_context();
    assert_eq!(app.under_caret.as_ref(), Some(&original));
    assert!(app.definition_expanded && !app.under_caret_job.is_running());
    for signature in ["old metadata", "new metadata"] {
        let mut index = (**app.semantic_index.as_ref().unwrap()).clone();
        let id = original.id.clone();
        index.symbols.get_mut(&id).unwrap().signature = signature.into();
        app.semantic_index = Some(Arc::new(index));
        app.refresh_symbol_context();
        assert!(app.under_caret.is_none() && app.under_caret_job.is_running());
    }
    settle(&mut app);
    assert_eq!(app.under_caret.as_ref().unwrap().signature, "new metadata");
    assert!(!app.definition_expanded);
    app.refresh_symbol_context();
    app.semantic_index = None;
    app.refresh_symbol_context();
    settle(&mut app);
    assert!(app.under_caret.is_none() && !app.under_caret_job.is_running());
}

#[test]
fn pinning_after_pointer_caret_movement_refreshes_metadata_and_current_history() {
    let (_dir, mut app) = reader_with_reference_index();
    app.jump_to_location("b.rs", 1, 4);
    settle(&mut app);
    assert_eq!(app.under_caret.as_ref().unwrap().name, "beta");
    let previous = (app.caret_line, app.caret_col);
    let byte = app
        .open_buffer
        .as_ref()
        .unwrap()
        .text
        .find("alpha")
        .unwrap()
        + 1;
    // The viewport mutates the caret before a double-click replaces its
    // CaretMoved action with PinAtByte.
    app.caret_line = 2;
    app.caret_col = 5;
    app.pin_at_byte(byte);
    app.finish_source_interaction(previous);
    settle(&mut app);
    assert_eq!(app.under_caret.as_ref().unwrap().name, "alpha");
    assert_eq!(app.history.current().unwrap().byte_offset, byte);
    app.definition_expanded = true;
    app.finish_source_interaction((2, 5));
    assert!(app.definition_expanded && !app.under_caret_job.is_running());
}

#[test]
fn local_metadata_uses_displayed_unicode_bytes_without_a_definition_fetch() {
    let (dir, mut app) = reader_with_reference_index();
    let text = "fn alpha() { let café = \"😀\"; café; }\n";
    std::fs::write(dir.path().join("repo/a.rs"), text).unwrap();
    app.open_file("b.rs");
    settle(&mut app);
    app.open_file("a.rs");
    settle(&mut app);
    let buffer = app.open_buffer.clone().unwrap();
    let first = text.find("café").unwrap();
    let last = text.rfind("café").unwrap();
    let occurrences = [(first, true), (last, false)]
        .into_iter()
        .map(|(byte, definition)| {
            crate::semantic::test_support::occurrence(
                &buffer,
                byte..byte + "café".len(),
                "local 1",
                definition,
            )
        })
        .collect();
    app.semantic_index = Some(Arc::new(SemanticIndex::from_scip(ScipIndex {
        documents: vec![ScipDocument {
            relative_path: "a.rs".into(),
            position_encoding: 2,
            occurrences,
            ..Default::default()
        }],
        ..Default::default()
    })));
    app.caret_col = buffer.column_at_byte(last);
    app.refresh_symbol_context();
    settle(&mut app);
    let context = app.under_caret.as_ref().unwrap();
    assert_eq!(
        (context.name.as_str(), context.file_occurrence_count),
        ("café", 2)
    );
    let line = context.local_definition_line.as_ref().unwrap();
    assert_eq!(&line.text[line.emphasis.clone().unwrap()], "café");
    assert_eq!(
        app.definition_preview.phase(),
        &crate::source::preview::PreviewPhase::None
    );
    assert!(Arc::ptr_eq(&buffer, app.open_buffer.as_ref().unwrap()));
}

#[test]
fn definition_here_previews_and_opens_another_indexed_definition() {
    let (dir, mut app) = reader_with_reference_index();
    std::fs::write(dir.path().join("repo/b.rs"), "fn alpha() {}\n").unwrap();
    let index = app.semantic_index.as_ref().unwrap();
    let mut documents = index.documents.values().cloned().collect::<Vec<_>>();
    let definition = index.documents["a.rs"].occurrences[0].clone();
    documents
        .iter_mut()
        .find(|doc| doc.relative_path == "b.rs")
        .unwrap()
        .occurrences = vec![definition];
    app.semantic_index = Some(Arc::new(SemanticIndex::from_scip(ScipIndex {
        documents,
        ..Default::default()
    })));
    app.refresh_symbol_context();
    settle(&mut app);
    let context = app.under_caret.as_ref().unwrap();
    assert!(context.is_on_definition);
    assert_eq!(context.definition_count, 2);
    assert_eq!(context.destination.as_ref().unwrap().path, "b.rs");
    app.execute_command(&Context::default(), ToolbarAction::GoToDefinition);
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
    assert_eq!((app.caret_line, app.caret_col), (1, 3));
    assert!(app.history.can_go_back());
}

fn reference_key(app: &mut RadiologyApp, ctx: &Context, key: egui::Key) -> bool {
    let mut consumed = false;
    let _ = ctx.run(
        egui::RawInput {
            events: vec![crate::ui::commands::test_key(key)],
            ..Default::default()
        },
        |ctx| {
            ctx.memory_mut(|memory| {
                memory.request_focus(crate::ui::reference_list::ReferenceViewport::focus_id())
            });
            app.handle_shortcuts(ctx);
            consumed = !ctx.input(|input| input.key_pressed(key));
        },
    );
    consumed
}

#[test]
fn reference_keyboard_selection_and_disclosure_preserve_source_until_return_commit() {
    use crate::source::preview::PreviewPhase;
    let (_dir, mut app) = reader_with_reference_index();
    app.find_references_at_caret();
    settle(&mut app);
    let buffer = app.open_buffer.clone().unwrap();
    let history = app.history.current().cloned();
    let ctx = Context::default();
    assert!(reference_key(&mut app, &ctx, egui::Key::ArrowDown));
    assert!(reference_key(&mut app, &ctx, egui::Key::ArrowDown));
    assert_eq!(app.reference_context.selected, Some(1));
    assert!(app.reference_context.disclosures.is_empty());
    assert!(reference_key(&mut app, &ctx, egui::Key::ArrowRight));
    settle(&mut app);
    let PreviewPhase::Loaded(preview) = app.reference_context.disclosures[&1].phase() else {
        panic!("reference preview did not load");
    };
    assert_eq!(preview.path, "b.rs");
    assert!(preview
        .lines
        .iter()
        .any(|line| line.text.contains("alpha();")));
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    assert_eq!(app.history.current(), history.as_ref());
    assert!(reference_key(&mut app, &ctx, egui::Key::ArrowLeft));
    assert!(app.reference_context.disclosures.is_empty());
    assert!(reference_key(&mut app, &ctx, egui::Key::Escape));
    assert_eq!(app.reference_context.selected, None);
    assert!(reference_key(&mut app, &ctx, egui::Key::ArrowUp));
    assert!(reference_key(&mut app, &ctx, egui::Key::Enter));
    settle(&mut app);
    assert_eq!(
        app.selected_path.as_deref(),
        Some("b.rs"),
        "Return ran under-caret definition navigation instead of reference commit"
    );
    assert_eq!((app.caret_line, app.caret_col), (2, 4));
    assert!(app.history.can_go_back());
}

#[test]
fn caller_navigation_validates_trace_source_and_commits_history_only_on_success() {
    use crate::ui::reference_list::ReferenceAction;
    let (dir, mut app) = reader_with_reference_index();
    app.find_references_at_caret();
    settle(&mut app);
    app.handle_reference_action(ReferenceAction::SetExpanded(0, true));
    settle(&mut app);
    let rows = &app.reference_context.disclosures[&0]
        .callers()
        .unwrap()
        .rows;
    assert_eq!(
        rows.iter().map(|row| row.name.as_str()).collect::<Vec<_>>(),
        ["alpha", "beta"]
    );
    let original = app.open_buffer.clone().unwrap();
    let history = app.history.current().cloned();
    std::fs::write(dir.path().join("repo/b.rs"), "fn beta() {}\n").unwrap();
    app.handle_reference_action(ReferenceAction::CommitCaller(0, 1));
    settle(&mut app);
    assert!(Arc::ptr_eq(&original, app.open_buffer.as_ref().unwrap()));
    assert_eq!(app.history.current(), history.as_ref());
    assert!(app
        .error
        .as_ref()
        .is_some_and(|error| error.contains("changed")));
    std::fs::write(
        dir.path().join("repo/b.rs"),
        "fn beta() {\n    alpha();\n}\n",
    )
    .unwrap();
    app.handle_reference_action(ReferenceAction::CommitCaller(0, 1));
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
    assert_eq!((app.caret_line, app.caret_col), (2, 4));
    assert!(app.history.can_go_back());
}

#[test]
fn modal_input_cannot_disclose_or_commit_a_reference_behind_the_sheet() {
    let (_dir, mut app) = reader();
    app.reference_context.selected = Some(0);
    app.modal = ActiveModal::GoToLine(GoToLineModal::default());
    let ctx = Context::default();
    assert!(!reference_key(&mut app, &ctx, egui::Key::Enter));
    assert!(!reference_key(&mut app, &ctx, egui::Key::ArrowRight));
    assert_eq!(app.reference_context.selected, Some(0));
    assert!(!app.is_source_loading());
}

fn reader_with_reference_index() -> (tempfile::TempDir, RadiologyApp) {
    let (dir, mut app) = reader();
    let root = dir.path().join("repo");
    repository(&root);
    app.open_repository(&root);
    settle(&mut app);
    app.semantic_index = Some(reference_index());
    app.jump_to_location("a.rs", 1, 4);
    settle(&mut app);
    (dir, app)
}

#[test]
fn explicit_reference_selection_and_disclosures_survive_unchanged_file_navigation() {
    let (_dir, mut app) = reader_with_reference_index();
    app.find_references_at_caret();
    settle(&mut app);
    app.reference_context.expand(1, true);
    settle(&mut app);
    let original = app.reference_context.disclosures[&1].phase().clone();
    app.open_file("b.rs");
    settle(&mut app);
    assert_eq!(app.reference_context.selected, Some(1));
    assert_eq!(app.reference_context.disclosures[&1].phase(), &original);
    assert!(!app.reference_context.is_running());
    app.open_file("a.rs");
    settle(&mut app);
    assert_eq!(app.reference_context.disclosures[&1].phase(), &original);
    app.handle_reference_action(crate::ui::reference_list::ReferenceAction::Commit(None));
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
    assert_eq!(app.reference_context.selected, Some(1));
    assert_eq!(app.reference_context.disclosures[&1].phase(), &original);
}

#[test]
fn changed_preview_destination_invalidates_disclosures_on_first_reader_visit() {
    let (dir, mut app) = reader_with_reference_index();
    app.find_references_at_caret();
    settle(&mut app);
    app.reference_context.expand(1, true);
    settle(&mut app);
    assert!(matches!(
        app.reference_context.disclosures[&1].phase(),
        crate::source::preview::PreviewPhase::Loaded(_)
    ));
    // b.rs has been read by the preview worker, but never opened by the reader.
    std::fs::write(
        dir.path().join("repo/b.rs"),
        "fn beta() {\n    changed();\n}\n",
    )
    .unwrap();
    app.open_file("b.rs");
    settle(&mut app);
    assert!(app.reference_context.disclosures.is_empty());
    assert_eq!(app.reference_context.selected, None);
    assert_eq!(app.reference_context.explicit().unwrap().name, "alpha");
}

#[test]
fn inspector_focus_remains_eligible_for_global_reader_definition_shortcuts() {
    let (_dir, mut app) = reader_with_reference_index();
    app.jump_to_location("b.rs", 2, "fn beta() {\n    ".len() + 1);
    settle(&mut app);
    let ctx = Context::default();
    assert!(reference_key(&mut app, &ctx, egui::Key::F12));
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
    assert_eq!((app.caret_line, app.caret_col), (1, 3));
}

#[test]
fn caller_controls_leave_global_definition_shortcuts_available() {
    let (_dir, mut app) = reader_with_reference_index();
    app.jump_to_location("b.rs", 2, "fn beta() {\n    ".len() + 1);
    settle(&mut app);
    app.find_references_at_caret();
    settle(&mut app);
    app.handle_reference_action(crate::ui::reference_list::ReferenceAction::SetExpanded(
        1, true,
    ));
    settle(&mut app);
    let ctx = Context::default();
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            app.reference_viewport.show(
                ui,
                &app.theme,
                &app.reference_context.results,
                Some(1),
                &app.reference_context.disclosures,
            );
        });
    });
    ctx.memory_mut(|memory| {
        memory.request_focus(crate::ui::reference_callers::control_id(1, Some(0)))
    });
    let _ = ctx.run(
        egui::RawInput {
            events: vec![crate::ui::commands::test_key(egui::Key::F12)],
            ..Default::default()
        },
        |ctx| app.handle_shortcuts(ctx),
    );
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
    assert_eq!((app.caret_line, app.caret_col), (1, 3));
}

#[test]
fn bookmark_keys_follow_identities_save_in_order_and_retire_on_scope_changes() {
    use crate::ui::bookmark_list::{BookmarkAction, BookmarkList};
    use eframe::App;
    let (dir, mut app) = opened_repository();
    for (path, line) in [("a.rs", 1), ("a.rs", 2), ("b.rs", 1)] {
        app.jump_to_location(path, line, 0);
        settle(&mut app);
        app.toggle_current_bookmark();
    }
    let ids: Vec<_> = app
        .bookmarks
        .bookmarks
        .iter()
        .map(|bookmark| bookmark.id)
        .collect();
    let ctx = Context::default();
    ctx.memory_mut(|memory| memory.request_focus(BookmarkList::focus_id()));
    let mut input =
        crate::ui::commands::test_input(&[egui::Key::Home, egui::Key::Delete, egui::Key::Enter]);
    app.raw_input_hook(&ctx, &mut input);
    assert!(input.events.is_empty());
    settle(&mut app);
    assert_eq!(app.bookmark_list.selected, Some(ids[1]));
    assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
    assert_eq!(app.caret_line, 2);
    app.handle_bookmark_action(
        &ctx,
        BookmarkAction::Move {
            id: ids[1],
            slot: 1,
        },
    );
    assert_eq!(app.bookmark_list.selected, Some(ids[1]));
    assert_eq!(app.bookmarks.bookmarks[1].id, ids[1]);
    let scope = app.bookmark_scope().unwrap();
    let saved = BookmarkStore::load_in(&dir.path().join("settings"), &scope);
    assert_eq!(saved.bookmarks, app.bookmarks.bookmarks);
    app.handle_bookmark_action(&ctx, BookmarkAction::Open(ids[0])); // Removed identity cannot target its old slot.
    assert!(!app.is_source_loading());
    assert_eq!(app.caret_line, 2);
    app.pending_pr = Some(42);
    app.handle_bookmark_action(&ctx, BookmarkAction::Remove(ids[1]));
    assert_eq!(app.bookmarks.bookmarks.len(), 2);
    app.pending_pr = None;
    app.active_pr = Some(PullRequestSummary {
        number: 42,
        title: "Other scope".into(),
        state: "OPEN".into(),
        base_ref_oid: None,
        updated_at: String::new(),
        head_ref_name: "feature".into(),
        base_ref_name: "main".into(),
        head_ref_oid: None,
    });
    app.load_bookmarks();
    assert!(app.bookmarks.bookmarks.is_empty());
    assert!(app.bookmark_list.selected.is_none());
    app.handle_bookmark_action(&ctx, BookmarkAction::Open(ids[1]));
    assert!(!app.is_source_loading());
}

#[test]
fn outline_keys_preview_without_caret_history_changes_and_commit_exact_departure() {
    use crate::ui::outline_list::OutlineList;
    use eframe::App;
    for seed_history in [false, true] {
        let (dir, mut app) = opened_repository();
        let text =
            "// café 😀 origin\nfn large() {\n let x = 1;\n let y = 2;\n}\n\nfn small() {}\n";
        std::fs::write(dir.path().join("a.rs"), text).unwrap();
        app.execute_command(&Context::default(), ToolbarAction::ReloadSource);
        settle(&mut app);
        let origin = app
            .open_buffer
            .as_ref()
            .unwrap()
            .location_at_byte(text.find("origin").unwrap());
        app.navigate(origin.clone(), false);
        app.history.clear();
        if seed_history {
            app.history.visit(origin.clone());
        }
        let history_before = app.history.current().cloned();
        let caret = (app.caret_line, app.caret_col);
        let ctx = Context::default();
        ctx.memory_mut(|memory| memory.request_focus(OutlineList::focus_id()));
        let mut input = crate::ui::commands::test_input(&[egui::Key::Home, egui::Key::ArrowDown]);
        app.raw_input_hook(&ctx, &mut input);
        assert!(input.events.is_empty());
        assert_eq!((app.caret_line, app.caret_col), caret);
        assert_eq!(app.history.current(), history_before.as_ref());
        assert!(!app.history.can_go_back());
        assert!(app.under_caret.is_none());
        let stale = app.outline_list.key(egui::Key::Enter).unwrap();
        let mut input = egui::RawInput {
            events: vec![crate::ui::commands::test_key(egui::Key::Enter)],
            ..Default::default()
        };
        app.raw_input_hook(&ctx, &mut input);
        settle(&mut app);
        let small = text.find("small").unwrap();
        assert_eq!(app.history.current().unwrap().byte_offset, small);
        app.execute_command(&ctx, ToolbarAction::NavigateBack);
        settle(&mut app);
        assert_eq!(app.history.current(), Some(&origin));
        assert_eq!((app.caret_line, app.caret_col), caret);
        assert!(app.outline_list.key(egui::Key::Enter).is_none());
        app.open_file("b.rs");
        settle(&mut app);
        let destination = app.history.current().cloned();
        app.handle_outline_action(stale);
        assert_eq!(app.history.current(), destination.as_ref());
        assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
    }
}

#[test]
fn outline_change_dots_use_pinned_revision_source_and_clear_on_checkout() {
    let before = "// café 😀\nfn removed_only() {\n old();\n}\n\nfn edited() {\n old();\n}\n\nfn untouched() {\n same();\n}\n";
    let after = "// café 😀\nfn edited() {\n new();\n}\n\nfn untouched() {\n same();\n}\n\nfn added_only() {\n new();\n}\n";
    let (dir, root, review) =
        crate::pull_request::review::tests::fixture_with_sources(before, after);
    let mut app = RadiologyApp::from_session(SessionStore::load_from(
        dir.path().join("settings/session.json"),
    ));
    app.open_repository(&root);
    settle(&mut app);
    app.accept_review(review);
    settle(&mut app);
    app.open_file("a file.rs");
    settle(&mut app);
    let marked = |app: &mut RadiologyApp| {
        app.sync_outline();
        let ctx = Context::default();
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(300.0, 400.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    app.outline_list.show(ui, &app.theme);
                });
            },
        );
        let dots: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Circle(circle)
                    if circle.radius == 3.0 && circle.fill == app.theme.accent =>
                {
                    Some(circle.center.y)
                }
                _ => None,
            })
            .collect();
        let mut names: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Text(text)
                    if [
                        "removed_only",
                        "edited",
                        "untouched",
                        "added_only",
                        "checkout_only",
                    ]
                    .contains(&text.galley.job.text.as_str())
                        && dots.iter().any(|y| {
                            (*y - text.pos.y - text.galley.size().y / 2.0).abs() < 0.5
                        }) =>
                {
                    Some(text.galley.job.text.clone())
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            dots.len(),
            names.len(),
            "every visible marker belongs to a callable label"
        );
        names.sort();
        names
    };
    assert_eq!(app.open_buffer.as_ref().unwrap().text, after);
    assert_eq!(marked(&mut app), ["added_only", "edited"]);
    let ctx = Context::default();
    assert_eq!(app.caret_line, 1);
    app.execute_command(&ctx, ToolbarAction::ToggleRevision(ReviewRevision::Before));
    settle(&mut app);
    assert_eq!(app.open_buffer.as_ref().unwrap().text, before);
    assert_eq!(app.caret_line, 1);
    assert_eq!(marked(&mut app), ["edited", "removed_only"]);
    app.execute_command(&ctx, ToolbarAction::ToggleRevision(ReviewRevision::After));
    settle(&mut app);
    assert_eq!(marked(&mut app), ["added_only", "edited"]);
    assert_eq!(app.caret_line, 1);
    // A real dirty checkout callable must not inherit the pinned review markers.
    let checkout = "fn checkout_only() {\n dirty();\n}\n";
    std::fs::write(root.join("a file.rs"), checkout).unwrap();
    app.execute_command(&ctx, ToolbarAction::ExitPRReview);
    settle(&mut app);
    app.open_file("a file.rs");
    settle(&mut app);
    assert_eq!(app.open_buffer.as_ref().unwrap().text, checkout);
    assert!(marked(&mut app).is_empty());
}

#[test]
fn syntactic_pins_include_cfg_omissions_but_prefer_complete_semantic_identity() {
    let (dir, mut app) = reader();
    let text = "fn demo() { clone(); #[cfg(other)] clone(); let s = \"clone\"; /* clone */ }";
    std::fs::write(dir.path().join("a.rs"), text).unwrap();
    write_critical_index(dir.path(), text);
    app.open_repository(dir.path());
    settle(&mut app);
    app.select_occurrences_at_byte(text.find("clone").unwrap());
    assert_eq!(app.pin_store.ephemeral.as_ref().unwrap().ranges.len(), 2);
    app.pin_at_byte(text.find("clone").unwrap());
    assert_eq!(app.pin_store.groups.len(), 1);
    let group = &app.pin_store.groups[0];
    assert_eq!(group.label, "clone");
    assert!(group.symbol_name.starts_with("syntactic:"));
    assert_eq!(group.ranges.len(), 2);
    app.pin_at_byte(text.find("\"clone\"").unwrap() + 1);
    assert_eq!(
        app.pin_store.groups.len(),
        1,
        "string clicks must not create pins"
    );
    app.pin_at_byte(text.find("#[cfg(other)] clone").unwrap() + "#[cfg(other)] ".len());
    assert!(
        app.pin_store.groups.is_empty(),
        "clicking another occurrence unpins the same group"
    );
    let mut index = (*app.semantic_index.as_ref().unwrap().as_ref()).clone();
    let second = text.find("#[cfg(other)] clone").unwrap() + "#[cfg(other)] ".len();
    let column = text[..second].chars().count() as i32;
    let mut occurrence = index.documents["a.rs"].occurrences[0].clone();
    occurrence.range = crate::semantic::ScipRange::from_packed(&[0, column, column + 5]);
    index
        .documents
        .get_mut("a.rs")
        .unwrap()
        .occurrences
        .push(occurrence);
    app.semantic_index = Some(Arc::new(index));
    app.select_occurrences_at_byte(text.find("clone").unwrap());
    assert_eq!(
        app.pin_store.ephemeral.as_ref().unwrap().symbol_name,
        "rust-analyzer cargo core 0 clone()."
    );
    app.pin_at_byte(text.find("clone").unwrap());
    assert_eq!(
        app.pin_store.groups[0].symbol_name,
        "rust-analyzer cargo core 0 clone()."
    );
    assert_eq!(app.pin_store.groups[0].ranges.len(), 2);
}

#[test]
fn transient_occurrences_are_click_owned_clear_on_misses_pins_and_document_changes() {
    let (dir, mut app) = reader();
    let text = "fn demo() { let value = 1; value; let s = \"value\"; /* value */ }";
    std::fs::write(dir.path().join("a.rs"), text).unwrap();
    std::fs::write(dir.path().join("b.rs"), "fn other() {}\n").unwrap();
    app.open_repository(dir.path());
    settle(&mut app);
    let byte = text.find("value").unwrap();
    app.select_occurrences_at_byte(byte);
    let selected = app.pin_store.ephemeral.clone().unwrap();
    assert_eq!(selected.label, "value");
    assert_eq!(selected.active, byte..byte + 5);
    assert_eq!(selected.ranges.len(), 2);
    assert!(app.pin_store.groups.is_empty());
    let previous = (app.caret_line, app.caret_col);
    app.caret_col = text.find("let s").unwrap();
    app.finish_source_interaction(previous);
    assert_eq!(
        app.pin_store.ephemeral.as_ref(),
        Some(&selected),
        "caret refresh does not select another group"
    );
    app.select_occurrences_at_byte(text.find("\"value").unwrap() + 1);
    assert!(app.pin_store.ephemeral.is_none());
    app.select_occurrences_at_byte(byte);
    app.pin_at_byte(byte);
    assert!(app.pin_store.ephemeral.is_none());
    assert_eq!(app.pin_store.groups.len(), 1);
    app.select_occurrences_at_byte(byte);
    assert!(app.pin_store.ephemeral.is_some());
    app.pin_at_byte(byte);
    assert!(app.pin_store.ephemeral.is_none());
    assert!(app.pin_store.groups.is_empty());
    app.select_occurrences_at_byte(byte);
    app.open_file("missing.rs");
    settle(&mut app);
    assert_eq!(app.pin_store.ephemeral.as_ref(), Some(&selected));
    app.open_file("b.rs");
    settle(&mut app);
    assert!(app.pin_store.ephemeral.is_none());
    app.open_file("a.rs");
    settle(&mut app);
    assert!(
        app.pin_store.ephemeral.is_none(),
        "transient groups are not restored with pins"
    );
}

mod git;

mod repository_open;

mod reviews;

mod review_refresh;

mod provisioning;
