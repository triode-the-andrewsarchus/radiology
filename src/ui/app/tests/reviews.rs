use super::*;
use crate::pull_request::test_support::await_retirement;
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[test]
fn cancelled_and_failed_review_jobs_keep_the_accepted_reader_and_drop_launch_intent() {
    let (_dir, mut app) = opened_repository();
    let buffer = app.open_buffer.clone().unwrap();
    let history = app.history.current().cloned();
    let saved = app.session.last_repository.clone();
    let (started, begin) = mpsc::channel();
    let (retired, done) = mpsc::channel();
    app.review_job.spawn(move |cancel| {
        await_retirement(cancel, started, retired);
        Err("retired review must never publish".into())
    });
    app.pending_pr = Some(7);
    app.pending_open_file = Some("stale.rs".into());
    begin.recv_timeout(Duration::from_secs(5)).unwrap();
    app.cancel_review_preparation();
    done.recv_timeout(Duration::from_secs(5)).unwrap();
    app.poll_review(&Context::default());
    assert!(app.error.is_none());
    assert!(app.pending_pr.is_none());
    assert!(app.pending_open_file.is_none());
    app.pending_pr = Some(8);
    app.pending_open_file = Some("failed.rs".into());
    app.review_job.spawn(|_| Err("fixture timeout".into()));
    let deadline = Instant::now() + Duration::from_secs(5);
    while app.pending_pr.is_some() {
        app.poll_review(&Context::default());
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(app.error.as_ref().unwrap().contains("fixture timeout"));
    assert!(app.pending_open_file.is_none());
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    assert_eq!(app.history.current(), history.as_ref());
    assert_eq!(app.session.last_repository, saved);
    assert!(app.review.is_none());
    assert!(app.active_pr.is_none());
}

#[test]
fn switching_pull_requests_with_identical_commits_changes_bookmark_and_session_scope() {
    let (_repository, root, snapshot) = crate::pull_request::review::tests::fixture();
    let (settings, mut app) = reader();
    app.open_repository(&root);
    settle(&mut app);
    app.accept_review(snapshot.clone());
    settle(&mut app);
    app.execute_command(&Context::default(), ToolbarAction::ToggleBookmark);
    assert_eq!(app.bookmarks.bookmarks.len(), 1);
    let mut other = snapshot;
    other.summary.number = 8;
    other.summary.title = "Same commits in another PR".into();
    app.accept_review(other);
    settle(&mut app);
    assert_eq!(app.active_pr.as_ref().unwrap().number, 8);
    assert_eq!(app.session.active_pr, Some(8));
    assert!(
        app.bookmarks.bookmarks.is_empty(),
        "another PR must not inherit the previous PR's bookmarks"
    );
    assert_eq!(
        SessionStore::load_from(settings.path().join("settings/session.json")).active_pr,
        Some(8)
    );
}

#[test]
fn accepted_review_retires_checkout_and_previous_pr_searches_but_reuse_keeps_current_results() {
    let (_repository, root, snapshot) = crate::pull_request::review::tests::fixture();
    let (_settings, mut app) = reader();
    app.open_repository(&root);
    settle(&mut app);
    app.repository_search.error = Some("checkout search".into());
    app.accept_review(snapshot.clone());
    settle(&mut app);
    assert!(app.repository_search.error.is_none());
    app.repository_search.error = Some("current search".into());
    app.accept_review(snapshot.clone());
    assert_eq!(
        app.repository_search.error.as_deref(),
        Some("current search")
    );
    let mut replacement = snapshot.clone();
    replacement.head_root = replacement.head_root.with_file_name("replacement-tree");
    app.accept_review(replacement);
    assert!(app.repository_search.error.is_none());
    app.repository_search.error = Some("previous PR search".into());
    let mut other = snapshot;
    other.summary.number = 8;
    app.accept_review(other);
    assert!(app.repository_search.error.is_none());
    settle(&mut app);
}

#[cfg(unix)]
#[test]
fn live_tool_probe_keeps_its_review_snapshot_after_the_reader_releases_it() {
    use crate::semantic::indexing::{
        tools::{ToolAvailability, ToolDirectory},
        IndexPlan, Indexer,
    };
    use std::os::unix::fs::PermissionsExt;
    let (_repository, _, snapshot) = crate::pull_request::review::tests::fixture();
    let (settings, mut app) = reader();
    let bin = settings.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    let executable = bin.join("rust-analyzer");
    std::fs::write(&executable, "#!/bin/sh\nprintf ready > probe-started\nwhile [ ! -f probe-release ]; do /bin/sleep 0.02; done\nprintf 'rust-analyzer fixture-version'\n").unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
    let root = snapshot.head_root.clone();
    let snapshot = Arc::new(snapshot);
    let weak = Arc::downgrade(&snapshot);
    app.review = Some(snapshot);
    app.index_plan = IndexPlan::inspect(["Cargo.toml", "a file.rs"]);
    app.probe_index_tools(
        ToolDirectory {
            managed: None,
            home: None,
            search_path: vec![bin],
        },
        root.clone(),
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while !root.join("probe-started").exists() {
        assert!(Instant::now() < deadline, "probe did not start");
        std::thread::sleep(Duration::from_millis(5));
    }
    app.review = None;
    assert!(
        weak.upgrade().is_some(),
        "live worker must retain the tree lease"
    );
    std::fs::write(root.join("probe-release"), "").unwrap();
    settle(&mut app);
    assert!(
        matches!(app.tool_availability(Indexer::Rust), Some(ToolAvailability::Available { version, .. }) if version == "rust-analyzer fixture-version")
    );
    assert!(
        weak.upgrade().is_none(),
        "finished probe releases the snapshot"
    );
}

#[cfg(unix)]
#[test]
fn review_index_commands_generate_after_and_reload_its_cache_without_checkout_imports() {
    let (_owner, root, review) = crate::pull_request::review::tests::indexing_fixture();
    let (settings, mut app) = reader();
    // This index is valid but belongs only to the attached checkout.
    std::fs::write(root.join("index.scip"), b"\x12\x0b\x0a\x09a file.rs").unwrap();
    app.open_repository(&root);
    settle(&mut app);
    assert!(app.semantic_index.is_some());
    app.accept_review((*review).clone());
    settle(&mut app);
    assert!(
        app.semantic_index.is_none(),
        "missing PR cache must never fall back to checkout import"
    );
    let executable = rust_indexer_fixture(settings.path(), "a file.rs");
    let ctx = Context::default();
    app.execute_command(&ctx, ToolbarAction::GenerateIndex);
    settle(&mut app);
    let report = app.generation_report().unwrap();
    assert_eq!(report.documents, 1, "{report:?}");
    assert_eq!(report.revision.as_deref(), Some(review.head.as_str()));
    assert!(app
        .semantic_index
        .as_ref()
        .unwrap()
        .matches_buffer(&review.read("a file.rs", ReviewRevision::After).unwrap()));
    assert!(!root.join("artifact.bin").exists());
    assert!(!review.head_root.join("artifact.bin").exists());
    std::fs::remove_file(executable).unwrap();
    app.execute_command(&ctx, ToolbarAction::ReloadIndexImport);
    settle(&mut app);
    assert!(app.generation_report().unwrap().cached);
    app.execute_command(&ctx, ToolbarAction::ToggleRevision(ReviewRevision::Before));
    settle(&mut app);
    assert_eq!(
        app.generation_report().unwrap().revision.as_deref(),
        Some(review.head.as_str())
    );
    for (path, revision) in [
        ("a file.rs", ReviewRevision::Before),
        ("Cargo.toml", ReviewRevision::After),
    ] {
        app.navigate(
            SourceLocation::with_revision(path, 0, Some(revision)),
            false,
        );
        settle(&mut app);
        let text = crate::ui::test_support::painted_text(&ctx, egui::vec2(1200.0, 100.0), |ui| {
            app.show_index_status(ui.ctx())
        });
        assert!(
            !text
                .iter()
                .any(|text| text == "Open source differs from index"),
            "{path}: {text:?}"
        );
        assert!(!text.iter().any(|text| text == "Reload Source"));
        assert!(text.iter().any(|text| text == "After index"));
    }
    assert_eq!(
        std::fs::read_to_string(root.join("a file.rs")).unwrap(),
        "checkout only\n"
    );
}

#[test]
fn unavailable_revision_commands_keep_the_reader_without_starting_a_failed_read() {
    let (_owner, root, snapshot) = crate::pull_request::review::tests::fixture();
    let (_settings, mut app) = reader();
    app.open_repository(&root);
    settle(&mut app);
    app.accept_review(snapshot);
    settle(&mut app);
    for (path, current, unavailable) in [
        ("added.rs", ReviewRevision::After, ReviewRevision::Before),
        ("removed.rs", ReviewRevision::Before, ReviewRevision::After),
    ] {
        app.open_file(path);
        settle(&mut app);
        assert_eq!(app.active_revision, Some(current));
        let buffer = app.open_buffer.clone().unwrap();
        let history = app.history.current().cloned();
        app.execute_command(
            &Context::default(),
            ToolbarAction::ToggleRevision(unavailable),
        );
        app.execute_command(&Context::default(), ToolbarAction::ToggleCurrentRevision);
        assert!(!app.is_source_loading());
        assert!(app.error.is_none());
        assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
        assert_eq!(app.history.current(), history.as_ref());
    }
}
