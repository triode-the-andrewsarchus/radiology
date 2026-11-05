use super::*;
use crate::pull_request::review::ReviewSnapshot;
use crate::repository::test_support::git;

pub(super) fn advance(
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

pub(in crate::ui::app) fn open_review(
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

#[test]
fn integrated_changed_files_refresh_and_cache_pressure_preserve_pinned_reader_transactions() {
    use crate::ui::app::tests::focus::{frame, key};
    use egui::{Event, Key, Modifiers};
    let original = format!(
        "fn demo() {{\n{}    before();\n}}\n",
        "    // stable caller context\n".repeat(8)
    );
    let after = original.replace("before()", "after()");
    let (owner, root, first) =
        crate::pull_request::review::tests::fixture_with_sources(&original, &after);
    let summary = first.summary.clone();
    let first_tree = first.head_root.clone();
    let (_settings, mut app) = open_review(&root, first.clone(), ReviewRevision::After);
    let ctx = Context::default();
    for (path, revision, text) in [
        ("added.rs", ReviewRevision::After, "new\n"),
        ("removed.rs", ReviewRevision::Before, "old\n"),
        ("renamed new.rs", ReviewRevision::After, "keep\n"),
    ] {
        app.open_file(path);
        settle(&mut app);
        app.source_viewport.request_focus(&ctx);
        frame(&mut app, &ctx, vec![]);
        assert_eq!(app.active_revision, Some(revision));
        assert_eq!(
            frame(
                &mut app,
                &ctx,
                vec![key(Key::A, Modifiers::CTRL), Event::Copy]
            )
            .platform_output
            .copied_text,
            text
        );
    }
    app.execute_command(&ctx, ToolbarAction::ToggleRevision(ReviewRevision::Before));
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("renamed old.rs"));
    assert_eq!(app.open_buffer.as_ref().unwrap().text, "keep\n");
    let retained = app.open_buffer.clone().unwrap();
    let history = app.history.current().cloned();
    app.open_file("binary.dat");
    settle(&mut app);
    assert!(app
        .error
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("binary"));
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &retained));
    assert_eq!(app.history.current(), history.as_ref());
    app.navigate(
        SourceLocation::with_revision(
            "a file.rs",
            after.find("after()").unwrap(),
            Some(ReviewRevision::After),
        ),
        true,
    );
    settle(&mut app);
    let line = app.caret_line;
    let column = app.caret_col;
    for step in 0..5 {
        let current = app.review.as_ref().unwrap().clone();
        let fresh = advance(&root, owner.path(), &current, |root| {
            if step == 0 {
                std::fs::rename(root.join("a file.rs"), root.join("moved.rs")).unwrap();
                std::fs::write(root.join("moved.rs"), format!("// moved\n{after}")).unwrap();
            } else {
                std::fs::write(root.join("stamp.txt"), format!("cache pressure {step}\n")).unwrap();
            }
        });
        app.accept_review(fresh);
        settle(&mut app);
        frame(&mut app, &ctx, vec![]);
        assert_eq!(app.selected_path.as_deref(), Some("moved.rs"));
        assert_eq!((app.caret_line, app.caret_col), (line + 1, column));
        assert_eq!(
            app.open_buffer.as_ref().unwrap().text,
            format!("// moved\n{after}")
        );
        assert_eq!(
            first.read("a file.rs", ReviewRevision::After).unwrap().text,
            after
        );
        assert!(
            first_tree.exists(),
            "a live snapshot lease protects its old tree"
        );
    }
    drop(first);
    let cache = owner.path().join("cache");
    // Releasing a lease alone leaves three retained trees within the budget.
    // A new head supplies actual pressure and must evict the oldest idle tree.
    let current = app.review.as_ref().unwrap().clone();
    let fresh = advance(&root, owner.path(), &current, |root| {
        std::fs::write(root.join("stamp.txt"), "final cache pressure\n").unwrap();
    });
    app.accept_review(fresh);
    settle(&mut app);
    assert!(
        !first_tree.exists(),
        "released oldest tree is evicted under pressure"
    );
    let reopened = ReviewSnapshot::prepare(&root, summary, &cache).unwrap();
    assert_ne!(reopened.head_root, first_tree);
    let (_reopened_settings, mut reopened_app) =
        open_review(&root, reopened, ReviewRevision::After);
    frame(&mut reopened_app, &Context::default(), vec![]);
    assert_eq!(reopened_app.open_buffer.as_ref().unwrap().text, after);
    assert_eq!(app.selected_path.as_deref(), Some("moved.rs"));
    assert_eq!(
        std::fs::read_to_string(root.join("a file.rs")).unwrap(),
        "checkout only\n"
    );
}

#[test]
fn review_revision_choices_follow_aliases_bookmarks_refresh_and_scope_without_failed_writes() {
    let (owner, root, first) = crate::pull_request::review::tests::fixture();
    let (_settings, mut app) = open_review(&root, first.clone(), ReviewRevision::Before);
    let ctx = Context::default();
    app.open_file("renamed new.rs");
    settle(&mut app);
    app.execute_command(&ctx, ToolbarAction::ToggleRevision(ReviewRevision::Before));
    settle(&mut app);
    app.open_file("a file.rs");
    settle(&mut app);
    assert_eq!(app.active_revision, Some(ReviewRevision::Before));
    app.execute_command(&ctx, ToolbarAction::ToggleBookmark);
    settle(&mut app);
    app.execute_command(&ctx, ToolbarAction::ToggleRevision(ReviewRevision::After));
    settle(&mut app);
    app.execute_command(&ctx, ToolbarAction::OpenBookmark(0));
    settle(&mut app);
    assert_eq!(app.active_revision, Some(ReviewRevision::Before));
    app.open_file("added.rs");
    settle(&mut app);
    assert_eq!(app.active_revision, Some(ReviewRevision::After));
    app.open_file("renamed new.rs");
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("renamed old.rs"));
    assert_eq!(app.active_revision, Some(ReviewRevision::Before));
    let choices = app.review_revision_choices.clone();
    let buffer = app.open_buffer.clone().unwrap();
    app.request_navigation(
        SourceLocation::with_revision("binary.dat", 0, Some(ReviewRevision::After)),
        crate::source::loader::SourceTarget::Byte(0),
        super::super::source::NavigationIntent {
            remember_revision: true,
            record_history: true,
            ..Default::default()
        },
    );
    settle(&mut app);
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    assert_eq!(app.review_revision_choices, choices);
    app.open_file("a file.rs");
    settle(&mut app);
    app.execute_command(&ctx, ToolbarAction::ToggleRevision(ReviewRevision::After));
    settle(&mut app);
    app.open_file("added.rs");
    settle(&mut app);
    let fresh = advance(&root, owner.path(), &first, |root| {
        std::fs::remove_file(root.join("a file.rs")).unwrap();
    });
    app.accept_review(fresh);
    settle(&mut app);
    assert!(
        !app.review_revision_choices.contains_key("a file.rs"),
        "unavailable choices retire on refresh"
    );
    app.open_file("renamed new.rs");
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("renamed old.rs"));
    app.open_file("a file.rs");
    settle(&mut app);
    assert_eq!(app.active_revision, Some(ReviewRevision::Before));
    let mut summary = app.review.as_ref().unwrap().summary.clone();
    summary.number += 1;
    let other = ReviewSnapshot::prepare(&root, summary, &owner.path().join("cache")).unwrap();
    app.accept_review(other);
    settle(&mut app);
    app.open_file("renamed new.rs");
    settle(&mut app);
    assert_eq!(app.active_revision, Some(ReviewRevision::After));
    app.execute_command(&ctx, ToolbarAction::ExitPRReview);
    settle(&mut app);
    assert!(app.review_revision_choices.is_empty());
}
