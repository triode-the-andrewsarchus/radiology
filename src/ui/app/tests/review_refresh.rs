use super::*;
use crate::pull_request::review::ReviewSnapshot;
use crate::repository::test_support::git;

fn advance(
    root: &Path,
    owner: &Path,
    template: &ReviewSnapshot,
    edit: impl FnOnce(&Path),
) -> ReviewSnapshot {
    git(root, &["restore", "."]);
    git(root, &["checkout", "review"]);
    edit(root);
    git(root, &["add", "-A"]);
    git(root, &["commit", "--allow-empty", "-m", "refresh"]);
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .unwrap();
    let mut summary = template.summary.clone();
    summary.head_ref_oid = Some(String::from_utf8(output.stdout).unwrap().trim().into());
    git(root, &["checkout", "main"]);
    std::fs::write(root.join("a file.rs"), "checkout only\n").unwrap();
    ReviewSnapshot::prepare(root, summary, &owner.join("cache")).unwrap()
}

fn open_review(
    root: &Path,
    review: ReviewSnapshot,
    revision: ReviewRevision,
) -> (tempfile::TempDir, RadiologyApp) {
    let (settings, mut app) = reader();
    app.open_repository(root);
    settle(&mut app);
    app.pending_open_file = Some("a file.rs".into());
    app.accept_review(review);
    settle(&mut app);
    if revision == ReviewRevision::Before {
        app.execute_command(&Context::default(), ToolbarAction::ToggleRevision(revision));
        settle(&mut app);
    }
    (settings, app)
}

#[test]
fn refresh_tracks_rename_preserves_latest_caret_and_exact_pins_and_resets_old_fold_memory() {
    let text = "fn demo() {\n    let value = 1;\n    value;\n}\n\nfn other() {\n    helper();\n}\n";
    let (owner, root, first) = crate::pull_request::review::tests::fixture_with_sources(text, text);
    let (_settings, mut app) = open_review(&root, first.clone(), ReviewRevision::After);
    app.fold_store.folded_ranges.clear();
    app.jump_to_location("a file.rs", 2, 0);
    settle(&mut app);
    app.pin_store.toggle_pin("value", vec![20..25, 35..40]);
    let fresh = advance(&root, owner.path(), &first, |root| {
        git(root, &["mv", "a file.rs", "moved café.rs"]);
    });
    let old_buffer = app.open_buffer.clone().unwrap();
    app.accept_review(fresh.clone());
    app.poll_review(&Context::default()); // Worker starts, but accepted state is unchanged.
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &old_buffer));
    assert_eq!(app.review.as_ref().unwrap().head, first.head);
    app.caret_line = 3;
    app.caret_col = 7;
    app.pin_store
        .toggle_pin("latest", std::iter::once(61..67).collect());
    let groups = app.pin_store.groups.clone();
    app.sidebar_tab = SidebarTab::Files;
    settle(&mut app);
    assert_eq!(app.review.as_ref().unwrap().head, fresh.head);
    assert_eq!(app.selected_path.as_deref(), Some("moved café.rs"));
    assert_eq!((app.caret_line, app.caret_col), (3, 7));
    assert_eq!(app.pin_store.groups, groups);
    assert_eq!(app.sidebar_tab, SidebarTab::Files);
    assert!(app.pin_store.ephemeral.is_none());
    assert!(
        app.fold_store.is_header_folded(6, &app.fold_ranges),
        "fresh defaults replace manual expansion"
    );
    assert!(!app.history.can_go_back());
    assert!(app.error.is_none(), "{:?}", app.error);
    assert_eq!(
        std::fs::read_to_string(root.join("a file.rs")).unwrap(),
        "checkout only\n"
    );
}

#[test]
fn refresh_maps_inserted_lines_keeps_only_unchanged_columns_and_drops_changed_pins() {
    let before = "fn demo() {\n    let café = 1;\n    use_it(café);\n}\n";
    let (owner, root, first) =
        crate::pull_request::review::tests::fixture_with_sources(before, before);
    let (_settings, mut app) = open_review(&root, first.clone(), ReviewRevision::After);
    app.jump_to_location("a file.rs", 3, 0);
    settle(&mut app);
    app.caret_col = 9;
    app.pin_store
        .toggle_pin("café", std::iter::once(20..25).collect());
    let fresh = advance(&root, owner.path(), &first, |root| {
        std::fs::write(root.join("a file.rs"), format!("// inserted 🦀\n{before}")).unwrap();
    });
    app.accept_review(fresh.clone());
    settle(&mut app);
    assert_eq!((app.caret_line, app.caret_col), (4, 9));
    assert!(app.pin_store.is_empty());
    let fresh = advance(&root, owner.path(), &fresh, |root| {
        std::fs::write(
            root.join("a file.rs"),
            "// inserted 🦀\nfn demo() {\n    let café = 1;\n    replaced();\n}\n",
        )
        .unwrap();
    });
    app.accept_review(fresh);
    settle(&mut app);
    assert_eq!((app.caret_line, app.caret_col), (4, 0));
}

#[test]
fn refresh_waits_for_current_navigation_and_reconciles_a_moving_reader() {
    let (owner, root, first) = crate::pull_request::review::tests::fixture();
    let (_settings, mut app) = open_review(&root, first.clone(), ReviewRevision::After);
    let fresh = advance(&root, owner.path(), &first, |root| {
        std::fs::write(root.join("a file.rs"), "one\nnew head\nthree\n").unwrap();
    });
    app.accept_review(fresh);
    app.poll_review(&Context::default());
    assert!(
        app.source_location_ready(),
        "accepted reader stays interactive during refresh"
    );
    app.open_file("added.rs");
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("added.rs"));
    assert_eq!(app.open_buffer.as_ref().unwrap().text, "new\n");
    assert!(app.error.is_none(), "{:?}", app.error);
}

#[test]
fn refresh_preserves_before_and_reports_revision_fallback_or_missing_source() {
    let (owner, root, first) = crate::pull_request::review::tests::fixture();
    let (_settings, mut app) = open_review(&root, first.clone(), ReviewRevision::Before);
    let fresh = advance(&root, owner.path(), &first, |_| {});
    app.accept_review(fresh.clone());
    settle(&mut app);
    assert_eq!(app.active_revision, Some(ReviewRevision::Before));
    assert_eq!(app.open_buffer.as_ref().unwrap().text, "one\ntwo\nthree\n");
    app.execute_command(
        &Context::default(),
        ToolbarAction::ToggleRevision(ReviewRevision::After),
    );
    settle(&mut app);
    let fresh = advance(&root, owner.path(), &fresh, |root| {
        std::fs::remove_file(root.join("a file.rs")).unwrap();
    });
    app.accept_review(fresh.clone());
    settle(&mut app);
    assert_eq!(app.active_revision, Some(ReviewRevision::Before));
    assert!(app
        .review_refresh
        .notice
        .as_ref()
        .unwrap()
        .contains("Before"));
    app.open_file("added.rs");
    settle(&mut app);
    let fresh = advance(&root, owner.path(), &fresh, |root| {
        std::fs::remove_file(root.join("added.rs")).unwrap();
    });
    app.accept_review(fresh);
    settle(&mut app);
    assert!(app.open_buffer.is_none());
    assert_eq!(app.sidebar_tab, SidebarTab::Changes);
    assert!(app
        .review_refresh
        .notice
        .as_ref()
        .unwrap()
        .contains("absent"));
}

#[test]
fn cancelling_a_staged_refresh_preserves_the_accepted_reader_and_same_commit_is_metadata_only() {
    let (owner, root, first) = crate::pull_request::review::tests::fixture();
    let (_settings, mut app) = open_review(&root, first.clone(), ReviewRevision::After);
    let fresh = advance(&root, owner.path(), &first, |_| {});
    let buffer = app.open_buffer.clone().unwrap();
    app.accept_review(fresh);
    app.poll_review(&Context::default());
    app.cancel_review_preparation();
    settle(&mut app);
    assert_eq!(app.review.as_ref().unwrap().head, first.head);
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    app.accept_review(first);
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    assert_eq!(
        app.review_refresh.notice.as_deref(),
        Some("PR is up to date.")
    );
}

#[test]
fn refresh_of_a_moving_merge_base_maps_the_before_reader() {
    let text = "fn demo() {\n    helper();\n}\n";
    let (owner, root, first) = crate::pull_request::review::tests::fixture_with_sources(text, text);
    let (_settings, mut app) = open_review(&root, first.clone(), ReviewRevision::Before);
    app.jump_to_location("a file.rs", 2, 0);
    settle(&mut app);
    app.caret_col = 5;
    git(&root, &["restore", "."]);
    std::fs::write(root.join("a file.rs"), format!("// new base\n{text}")).unwrap();
    git(&root, &["add", "a file.rs"]);
    git(&root, &["commit", "-m", "advance base"]);
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    let mut template = first.clone();
    template.summary.base_ref_oid = Some(String::from_utf8(output.stdout).unwrap().trim().into());
    let fresh = advance(&root, owner.path(), &template, |root| {
        git(root, &["merge", "--no-edit", "main"]);
    });
    assert_ne!(fresh.base, first.base);
    app.accept_review(fresh);
    settle(&mut app);
    assert_eq!(app.active_revision, Some(ReviewRevision::Before));
    assert_eq!((app.caret_line, app.caret_col), (3, 5));
}

#[test]
fn failed_source_restoration_retains_the_entire_accepted_reader() {
    let (owner, root, first) = crate::pull_request::review::tests::fixture();
    let (_settings, mut app) = open_review(&root, first.clone(), ReviewRevision::After);
    let buffer = app.open_buffer.clone().unwrap();
    let history = app.history.current().cloned();
    let fresh = advance(&root, owner.path(), &first, |root| {
        std::fs::write(root.join("a file.rs"), [0, 1]).unwrap();
    });
    app.accept_review(fresh);
    settle(&mut app);
    assert_eq!(app.review.as_ref().unwrap().head, first.head);
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    assert_eq!(app.history.current(), history.as_ref());
    assert!(app
        .error
        .as_ref()
        .unwrap()
        .contains("Unable to refresh review"));
    assert!(app.pending_pr.is_none());
}
