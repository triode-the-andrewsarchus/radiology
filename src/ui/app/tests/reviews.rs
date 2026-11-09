use super::*;
use crate::pull_request::test_support::await_retirement;
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[test]
fn cancelled_and_failed_review_jobs_keep_the_accepted_reader_and_drop_launch_intent() {
    let (settings, mut app) = opened_repository();
    let buffer = app.open_buffer.clone().unwrap();
    let history = app.history.current().cloned();
    let saved = app.session.last_repository.clone();
    for exit in [false, true] {
        app.session.set_active_pr(Some(7));
        let (started, begin) = mpsc::channel();
        let (retired, done) = mpsc::channel();
        app.review_job.spawn(move |cancel| {
            await_retirement(cancel, started, retired);
            Err("retired review must never publish".into())
        });
        app.pending_pr = Some(7);
        app.pending_open_file = Some("stale.rs".into());
        begin.recv_timeout(Duration::from_secs(5)).unwrap();
        if exit {
            app.execute_command(&Context::default(), ToolbarAction::ExitPRReview);
        } else {
            app.cancel_review_preparation();
        }
        done.recv_timeout(Duration::from_secs(5)).unwrap();
        app.poll_review(&Context::default());
        assert!(app.error.is_none());
        assert!(app.review_error().is_none());
        assert!(app.pending_pr.is_none());
        assert!(app.pending_open_file.is_none());
        settle(&mut app);
        assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
        assert_eq!(app.history.current(), history.as_ref());
        let expected_pr = if exit { None } else { Some(7) };
        assert_eq!(app.session.active_pr, expected_pr);
        assert_eq!(
            SessionStore::load_from(settings.path().join("settings/session.json")).active_pr,
            expected_pr
        );
    }
    app.pending_pr = Some(8);
    app.pending_open_file = Some("failed.rs".into());
    app.review_job.spawn(|_| Err("fixture timeout".into()));
    let deadline = Instant::now() + Duration::from_secs(5);
    while app.pending_pr.is_some() {
        app.poll_review(&Context::default());
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(app.review_error().unwrap().contains("fixture timeout"));
    assert!(app.pending_open_file.is_none());
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    assert_eq!(app.history.current(), history.as_ref());
    assert_eq!(app.session.last_repository, saved);
    assert!(app.review.is_none());
    assert!(app.active_pr.is_none());

    // The same exit command also clears an already published failure.
    app.execute_command(&Context::default(), ToolbarAction::ExitPRReview);
    settle(&mut app);
    assert!(app.review_error().is_none());
}

#[test]
fn switching_pull_requests_with_identical_commits_changes_bookmark_and_session_scope() {
    let (_repository, root, snapshot) = crate::pull_request::review::tests::fixture();
    let (settings, mut app) = reader();
    app.open_repository(&root);
    settle(&mut app);
    app.accept_review(snapshot.clone());
    settle(&mut app);
    app.open_file("a file.rs");
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
    app.open_file("a file.rs");
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
    app.execute_command(&ctx, ToolbarAction::UpdateIndex);
    settle(&mut app);
    assert_eq!(
        app.generation_report().unwrap().outcomes[0]
            .reused_from
            .as_deref(),
        Some(review.head.as_str())
    );
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
    let (_owner, root, snapshot) = crate::pull_request::review::tests::fixture_with_tree_edits(
        "one\ntwo\nthree\n",
        "one\nchanged\nthree\n",
        |root| std::fs::write(root.join("unchanged.rs"), "fn stable() {}\n").unwrap(),
        |_| {},
    );
    let (_settings, mut app) = reader();
    app.open_repository(&root);
    settle(&mut app);
    app.accept_review(snapshot);
    settle(&mut app);
    for (path, current, unavailable) in [
        ("added.rs", ReviewRevision::After, ReviewRevision::Before),
        ("removed.rs", ReviewRevision::Before, ReviewRevision::After),
        (
            "unchanged.rs",
            ReviewRevision::After,
            ReviewRevision::Before,
        ),
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
        let blocked = SourceLocation::with_revision(path, 0, Some(unavailable));
        assert!(app
            .review
            .as_ref()
            .unwrap()
            .resolve(blocked.clone())
            .is_err());
        app.navigate(blocked, true);
        assert!(!app.is_source_loading());
        assert!(app.error.is_some());
        assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
        assert_eq!(app.history.current(), history.as_ref());
    }
}

#[test]
fn empty_change_side_reaches_unterminated_eof_once_without_inventing_a_changed_line() {
    use super::focus::{frame, key, settle_frame};
    use crate::pull_request::diff::UnifiedDiffParser;
    use egui::{Key, Modifiers};
    let (_owner, mut app) = reader_with_source("one\ntwo");
    let ctx = Context::default();
    frame(&mut app, &ctx, vec![]);
    app.line_changes = Some(UnifiedDiffParser::parse("@@ -3,2 +2,0 @@\n-three\n-four\n").unwrap());
    app.source_viewport.request_focus(&ctx);
    frame(&mut app, &ctx, vec![key(Key::ArrowDown, Modifiers::ALT)]);
    settle_frame(&mut app, &ctx);
    let source = app.open_buffer.clone().unwrap();
    assert_eq!(
        source.byte_offset(app.caret_line, app.caret_col),
        source.byte_len()
    );
    assert_eq!(app.change_navigation(), Default::default());
    assert!(app
        .line_changes
        .as_ref()
        .unwrap()
        .changed_lines(ReviewRevision::After)
        .is_empty());
    let history = app.history.current().cloned();
    frame(&mut app, &ctx, vec![key(Key::ArrowDown, Modifiers::ALT)]);
    settle_frame(&mut app, &ctx);
    assert_eq!(app.history.current(), history.as_ref());
    app.execute_command(&ctx, ToolbarAction::NavigateBack);
    settle_frame(&mut app, &ctx);
    assert_eq!((app.caret_line, app.caret_col), (1, 0));
}

#[test]
fn revision_switches_land_on_first_changed_lines_and_after_deleted_gaps() {
    let before =
        "one\r\nremoved café\r\nremoved two\r\nshared\r\nold a\r\nold b\r\nold c\r\ntail\r\n";
    let after = "one\r\nshared\r\nnew a\r\nnew b\r\nnew c\r\nnew d\r\ntail\r\n";
    let (_owner, root, review) =
        crate::pull_request::review::tests::fixture_with_sources(before, after);
    let head = crate::pull_request::review::tests::git(&root, &["rev-parse", "HEAD"]);
    let status = crate::pull_request::review::tests::git(&root, &["status", "--porcelain"]);
    for scale in [1.0, 2.0] {
        let (_settings, mut app, ctx) =
            super::review_refresh::review_context(&root, review.clone(), scale);
        for (from, line, to, expected_line) in [
            (ReviewRevision::Before, 3, ReviewRevision::After, 2),
            (ReviewRevision::Before, 7, ReviewRevision::After, 3),
            (ReviewRevision::After, 5, ReviewRevision::Before, 5),
        ] {
            let source = SourceBuffer::from_str(
                "a file.rs",
                if from == ReviewRevision::Before {
                    before
                } else {
                    after
                },
            );
            app.navigate(
                SourceLocation::with_revision("a file.rs", source.byte_offset(line, 0), Some(from)),
                true,
            );
            settle(&mut app);
            app.execute_command(&ctx, ToolbarAction::ToggleRevision(to));
            settle(&mut app);
            super::focus::settle_frame(&mut app, &ctx);
            let buffer = app.open_buffer.as_ref().unwrap();
            assert_eq!(buffer.revision, Some(to));
            assert_eq!(
                buffer.text,
                if to == ReviewRevision::Before {
                    before
                } else {
                    after
                }
            );
            assert_eq!(
                (app.caret_line, app.caret_col),
                (expected_line, 0),
                "{scale}× {from:?}:{line}"
            );
            assert!(app.error.is_none(), "{:?}", app.error);
        }
    }
    assert_eq!(
        crate::pull_request::review::tests::git(&root, &["rev-parse", "HEAD"]),
        head
    );
    assert_eq!(
        crate::pull_request::review::tests::git(&root, &["status", "--porcelain"]),
        status
    );
    assert_eq!(
        std::fs::read_to_string(root.join("a file.rs")).unwrap(),
        "checkout only\n"
    );
}

#[test]
fn changes_selection_follows_both_rename_revisions_without_moving_the_reader() {
    use crate::ui::test_support::text_shapes;
    let (_owner, root, review) = crate::pull_request::review::tests::fixture();
    for scale in [1.0, 2.0] {
        let (_settings, mut app) =
            super::review_refresh::open_review(&root, review.clone(), ReviewRevision::After);
        app.sidebar_tab = SidebarTab::Changes;
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        for (revision, path) in [
            (ReviewRevision::Before, "renamed old.rs"),
            (ReviewRevision::After, "renamed new.rs"),
        ] {
            app.navigate(SourceLocation::with_revision(path, 0, Some(revision)), true);
            super::focus::settle_frame(&mut app, &ctx);
            let accepted = app.open_buffer.clone().unwrap();
            let history = format!("{:?}", app.history);
            let output = super::focus::frame(&mut app, &ctx, vec![]);
            let caption = text_shapes(&output)
                .find(|text| text.galley.job.text == "renamed new.rs")
                .expect("rename row");
            let point = caption.pos + caption.galley.size() / 2.0;
            assert!(output.shapes.iter().any(|shape| matches!(&shape.shape,
                egui::Shape::Rect(rect) if rect.rect.contains(point) && rect.rect.height() < 80.0
                    && rect.rect.width() > 100.0 && rect.stroke.color == app.theme.border && rect.stroke.width == 1.0
            )), "{scale}× {revision:?}: selected rename row must be highlighted");
            assert!(Arc::ptr_eq(&accepted, app.open_buffer.as_ref().unwrap()));
            assert_eq!(format!("{:?}", app.history), history);
            assert_eq!(app.selected_path.as_deref(), Some(path));
        }
    }
}

#[test]
fn changes_filter_reset_restores_visible_totals_without_reader_departure_and_empty_pr_has_no_reset()
{
    use crate::pull_request::review::{
        tests::{fixture_with_tree_edits, git},
        ReviewSnapshot,
    };
    use crate::ui::test_support::{click, text_center, text_shapes};
    let (owner, root, review) = fixture_with_tree_edits(
        "unchanged\n",
        "unchanged\n",
        |_| {},
        |root| {
            for path in ["added.rs", "binary.dat", "renamed new.rs"] {
                std::fs::remove_file(root.join(path)).unwrap();
            }
            git(
                root,
                &["restore", "--source=main", "--worktree", "--staged", "."],
            );
            std::fs::write(root.join("README.md"), "# Notes\n").unwrap();
        },
    );
    assert_eq!(review.changes.len(), 1);
    let canonical = review.changes.clone();
    let mut summary = review.summary.clone();
    summary.head_ref_oid = Some(review.base.clone());
    let empty = ReviewSnapshot::prepare(&root, summary, &owner.path().join("cache")).unwrap();
    for scale in [1.0, 2.0] {
        let (_settings, mut app) =
            super::review_refresh::open_review(&root, review.clone(), ReviewRevision::After);
        app.navigate(
            SourceLocation::with_revision("README.md", 0, Some(ReviewRevision::After)),
            true,
        );
        settle(&mut app);
        app.sidebar_tab = SidebarTab::Changes;
        app.session.source_filters.path_query = "does-not-match-any-path".into();
        app.filters_changed();
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        let accepted = app.open_buffer.clone().unwrap();
        let history = format!("{:?}", app.history);
        let output = super::focus::frame(&mut app, &ctx, vec![]);
        assert!(text_shapes(&output).any(|t| t.galley.job.text == "README.md"));
        assert!(text_shapes(&output).any(|t| t.galley.job.text == "1 of 1 files"));
        assert!(text_shapes(&output).any(|t| t.galley.job.text == "+1"));
        app.execute_command(&ctx, ToolbarAction::ToggleSourceFilesOnly);
        let output = super::focus::frame(&mut app, &ctx, vec![]);
        assert!(text_shapes(&output).any(|t| t.galley.job.text == "0 of 1 files"));
        assert!(text_shapes(&output).any(|t| t.galley.job.text == "+0"));
        assert!(text_shapes(&output)
            .any(|t| t.galley.job.text == "No files match the current filters."));
        assert!(!text_shapes(&output).any(|t| t.galley.job.text == "No changed files."));
        let reset = text_center(&output, "Show All Files");
        super::focus::frame(&mut app, &ctx, click(reset));
        let output = super::focus::frame(&mut app, &ctx, vec![]);
        assert_eq!(app.session.source_filters, Default::default());
        assert!(text_shapes(&output).any(|t| t.galley.job.text == "README.md"));
        assert!(text_shapes(&output).any(|t| t.galley.job.text == "+1"));
        assert!(Arc::ptr_eq(&accepted, app.open_buffer.as_ref().unwrap()));
        assert_eq!(format!("{:?}", app.history), history);
        assert_eq!(app.pr_changes, canonical);
        app.accept_review(empty.clone());
        settle(&mut app);
        app.sidebar_tab = SidebarTab::Changes;
        app.execute_command(&ctx, ToolbarAction::ToggleSourceFilesOnly);
        let output = super::focus::frame(&mut app, &ctx, vec![]);
        assert!(text_shapes(&output).any(|t| t.galley.job.text == "No changed files."));
        assert!(!text_shapes(&output).any(|t| t.galley.job.text == "Show All Files"));
        assert!(app.pr_changes.is_empty());
    }
}

#[test]
fn review_fold_choices_survive_late_appearance_and_retired_reload_but_reset_for_new_scopes() {
    use super::focus::{frame, settle_frame};
    let before = format!(
        "fn changed() {{\n    old();\n}}\nfn quiet() {{\n    untouched();\n}}\n{}",
        "// tail café reading anchor\n".repeat(160)
    );
    let after = before.replace("old()", "new()");
    let (_owner, root, review) =
        crate::pull_request::review::tests::fixture_with_sources(&before, &after);
    for scale in [1.0, 2.0] {
        let (_settings, mut app, ctx) =
            super::review_refresh::review_context(&root, review.clone(), scale);
        assert_folded_headers(&app, &[4]);
        app.fold_store.toggle_header(4, &app.fold_ranges);
        app.fold_store.toggle_header(1, &app.fold_ranges);
        assert_folded_headers(&app, &[1]);
        let accepted = app.open_buffer.clone().unwrap();
        app.scroll_to_line = None;
        let top = accepted.byte_offset(30, 0);
        app.source_viewport.restore_top(&accepted, top);
        frame(&mut app, &ctx, vec![]);
        let history = format!("{:?}", app.history);
        app.execute_command(&ctx, ToolbarAction::ToggleTheme);
        let appearance = take_prepared_source(&mut app);
        assert!(appearance.syntax.is_none() && appearance.initial_folds.is_none());
        assert!(app.command_enabled(&ToolbarAction::CollapseFunctions));
        app.execute_command(&ctx, ToolbarAction::CollapseFunctions);
        assert_folded_headers(&app, &[1, 4]);
        app.publish_source(appearance);
        frame(&mut app, &ctx, vec![]);
        assert_folded_headers(&app, &[1, 4]);
        assert_eq!(app.source_viewport.top_byte(&accepted), Some(top));
        assert_eq!((app.caret_line, app.caret_col), (1, 0));
        assert_eq!(format!("{:?}", app.history), history);
        assert!(Arc::ptr_eq(&accepted, app.open_buffer.as_ref().unwrap()));
        // Revision navigation reveals its destination; use an unrelated tail row
        // so that body-reveal policy does not mask whether choices were restored.
        app.navigate(accepted.location_at_byte(accepted.byte_offset(30, 0)), true);
        settle_frame(&mut app, &ctx);
        app.execute_command(&ctx, ToolbarAction::ToggleRevision(ReviewRevision::Before));
        settle_frame(&mut app, &ctx);
        assert_eq!(app.open_buffer.as_ref().unwrap().text, before);
        assert_folded_headers(&app, &[4]);
        app.execute_command(&ctx, ToolbarAction::FoldToDefinitions);
        assert_folded_headers(&app, &[1, 4]);
        app.execute_command(&ctx, ToolbarAction::ToggleRevision(ReviewRevision::After));
        settle_frame(&mut app, &ctx);
        assert_folded_headers(&app, &[1, 4]);
        app.execute_command(&ctx, ToolbarAction::ReloadSource);
        let obsolete = take_prepared_source(&mut app);
        assert!(obsolete.initial_folds.is_some());
        let source = app.open_buffer.clone().unwrap();
        let jump = source.location_at_byte(source.byte_offset(100, 2));
        app.navigate(jump.clone(), true);
        settle_frame(&mut app, &ctx);
        let newer_top = app.source_viewport.top_byte(&source);
        let newer_history = format!("{:?}", app.history);
        app.publish_source(obsolete);
        frame(&mut app, &ctx, vec![]);
        assert_eq!((app.caret_line, app.caret_col), (100, 2));
        assert_eq!(app.source_viewport.top_byte(&source), newer_top);
        assert_ne!(newer_top, Some(top));
        assert_eq!(format!("{:?}", app.history), newer_history);
        assert!(Arc::ptr_eq(&source, app.open_buffer.as_ref().unwrap()));
        assert_folded_headers(&app, &[1, 4]);
        let mut other = review.clone();
        other.summary.number = 8;
        app.accept_review(other);
        settle_frame(&mut app, &ctx);
        app.open_file("a file.rs");
        settle_frame(&mut app, &ctx);
        assert_eq!(app.open_buffer.as_ref().unwrap().text, after);
        assert_folded_headers(&app, &[4]);
        assert_eq!(app.active_pr.as_ref().unwrap().number, 8);
    }
}

#[test]
fn fresh_review_entry_waits_for_file_choice_and_explicit_destinations_still_open() {
    let (_owner, root, snapshot) = crate::pull_request::review::tests::fixture();
    for scale in [1.0, 2.0] {
        let (_settings, mut app, ctx) = checkout_context(&root, "a file.rs", scale);
        app.session.source_filters.path_query = "checkout-only query".into();
        app.session.source_filters.source_files_only = true;
        app.filters_changed();
        app.accept_review(snapshot.clone());
        super::focus::settle_frame(&mut app, &ctx);
        assert_eq!(app.sidebar_tab, SidebarTab::Changes);
        assert!(app.session.source_filters.path_query.is_empty());
        assert!(app.session.source_filters.source_files_only);
        assert!(
            app.open_buffer.is_none(),
            "a fresh review waits for an explicit source choice"
        );
        assert!(app.selected_path.is_none() && app.history.current().is_none());
        assert!(app.line_changes.is_none() && !app.is_source_loading());
        assert!(app.active_revision.is_none());
        assert_eq!(app.repository_root.as_ref(), Some(&root));
        assert_eq!(app.review.as_ref().unwrap().head, snapshot.head);
        let output = super::focus::frame(&mut app, &ctx, vec![]);
        assert!(crate::ui::test_support::text_shapes(&output).any(|t| t
            .galley
            .job
            .text
            .starts_with("Select a file from the sidebar")));
        let row = crate::ui::test_support::text_center(&output, "a file.rs");
        super::focus::frame(&mut app, &ctx, crate::ui::test_support::click(row));
        super::focus::settle_frame(&mut app, &ctx);
        assert_eq!(
            app.open_buffer.as_ref().unwrap().text,
            "one\nchanged\nthree\n"
        );
        assert!(!app.history.can_go_back());
        let accepted = app.open_buffer.clone().unwrap();
        let history = format!("{:?}", app.history);
        app.session.source_filters.path_query = "review query".into();
        app.filters_changed();
        app.accept_review(snapshot.clone());
        super::focus::settle_frame(&mut app, &ctx);
        assert!(Arc::ptr_eq(&accepted, app.open_buffer.as_ref().unwrap()));
        assert_eq!(format!("{:?}", app.history), history);
        assert_eq!(app.session.source_filters.path_query, "review query");
        let mut other = snapshot.clone();
        other.summary.number = 8;
        app.accept_review(other);
        super::focus::settle_frame(&mut app, &ctx);
        assert!(app.open_buffer.is_none() && app.history.current().is_none());
        assert!(app.session.source_filters.path_query.is_empty());
        app.pending_open_file = Some("removed.rs".into());
        app.accept_review(snapshot.clone());
        super::focus::settle_frame(&mut app, &ctx);
        assert_eq!(app.open_buffer.as_ref().unwrap().text, "old\n");
        assert_eq!(app.active_revision, Some(ReviewRevision::Before));
        assert!(!app.history.can_go_back());
    }
}

#[test]
fn pinned_browsing_keeps_snapshot_inventory_revision_history_rename_filters_search_and_exit_in_agreement(
) {
    use super::focus::{frame, settle_frame};
    use crate::pull_request::review::tests::{fixture_with_tree_edits, git};
    use crate::repository::filters::SourceFilters;
    use crate::source::ReviewRevision::{After, Before};
    use crate::ui::test_support::{click, text_center};
    use std::collections::BTreeSet;
    let tail = "// passive reading tail café\n".repeat(160);
    let before =
        format!("pub fn a() -> i32 {{\n    1\n}}\n\npub fn b() -> i32 {{\n    2\n}}\n{tail}");
    let after = format!("pub fn a() -> i32 {{\n    1\n}}\n\npub fn b() -> i32 {{\n    let extra = 40;\n    extra + 2\n}}\n\npub fn c() -> i32 {{\n    3\n}}\n{tail}");
    let markdown = "# Readme\n\nOne\nTwo\nThree\n";
    let edited_markdown = format!("{markdown}Four\n");
    let (_owner, root, review) = fixture_with_tree_edits(
        &before,
        &after,
        |root| {
            std::fs::write(
                root.join("Cargo.toml"),
                "[package]\nname='fixture'\nversion='0.1.0'\n[lib]\npath='a file.rs'\n",
            )
            .unwrap();
            std::fs::create_dir(root.join("docs")).unwrap();
            std::fs::write(root.join("docs/readme.md"), markdown).unwrap();
        },
        |root| {
            git(root, &["mv", "docs/readme.md", "docs/README.md"]);
            std::fs::write(root.join("docs/README.md"), &edited_markdown).unwrap();
        },
    );
    let head = git(&root, &["rev-parse", "HEAD"]);
    let status = git(&root, &["status", "--porcelain"]);
    let source_only = SourceFilters {
        source_files_only: true,
        ..Default::default()
    };
    for scale in [1.0, 2.0] {
        let (_settings, mut app, ctx) = checkout_context(&root, "a file.rs", scale);
        app.execute_command(&ctx, ToolbarAction::ToggleSourceFilesOnly);
        assert_eq!(
            app.file_tree
                .iter()
                .map(|f| f.path.as_str())
                .collect::<BTreeSet<_>>(),
            ["a file.rs", "removed.rs", "renamed old.rs"]
                .into_iter()
                .collect()
        );
        crate::pull_request::recent::tests::publish(
            &mut app.recent_pull_requests,
            Ok(Some(vec![review.summary.clone()])),
        );
        app.accept_review(review.clone());
        settle_frame(&mut app, &ctx);
        assert!(app.open_buffer.is_none() && app.history.current().is_none());
        assert_eq!(app.sidebar_tab, SidebarTab::Changes);
        assert_eq!(app.repository_root.as_ref(), Some(&root));
        assert_eq!(app.search_root().as_ref(), Some(&review.head_root));
        assert_eq!(app.review.as_ref().unwrap().base, review.base);
        assert_eq!(
            app.file_tree
                .iter()
                .map(|f| f.path.as_str())
                .collect::<BTreeSet<_>>(),
            ["a file.rs", "added.rs", "renamed new.rs"]
                .into_iter()
                .collect()
        );
        for (path, has_before, has_after) in [
            ("a file.rs", true, true),
            ("removed.rs", true, false),
            ("added.rs", false, true),
            ("Cargo.toml", false, true),
        ] {
            for (revision, available) in [(Before, has_before), (After, has_after)] {
                assert_eq!(
                    app.resolve_location(SourceLocation::with_revision(path, 0, Some(revision)))
                        .is_ok(),
                    available,
                    "{path} {revision:?}"
                );
            }
        }
        app.open_file("a file.rs");
        settle_frame(&mut app, &ctx);
        assert_eq!(app.open_buffer.as_ref().unwrap().text, after);
        assert_eq!(
            app.line_changes.as_ref().unwrap().changed_lines(After),
            std::iter::once(6..12).collect::<Vec<_>>()
        );
        assert_eq!(
            marked_outline_names(&mut app, &["a", "b", "c"], scale),
            ["b", "c"]
        );
        for (command, departure_line, expected_revision) in [
            (ToolbarAction::NextChangeBlock, 1, After),
            (ToolbarAction::ToggleRevision(Before), 10, Before),
        ] {
            let previous = (app.caret_line, app.caret_col);
            app.caret_line = departure_line;
            app.caret_col = 2;
            app.finish_source_interaction(previous);
            let source = app.open_buffer.clone().unwrap();
            app.scroll_to_line = None;
            let top = source.byte_offset(80, 0);
            app.source_viewport.restore_top(&source, top);
            frame(&mut app, &ctx, vec![]);
            assert_eq!(app.reading_positions.current_byte(), Some(top));
            app.execute_command(&ctx, command);
            settle_frame(&mut app, &ctx);
            assert_eq!(app.caret_line, 6);
            assert_eq!(app.active_revision, Some(expected_revision));
            if expected_revision == Before {
                assert_eq!(app.open_buffer.as_ref().unwrap().text, before);
                assert_eq!(
                    app.line_changes.as_ref().unwrap().changed_lines(Before),
                    std::iter::once(6..7).collect::<Vec<_>>()
                );
                assert_eq!(
                    crate::semantic::availability::SemanticAvailability::from_services(
                        app.open_buffer.as_deref(),
                        app.semantic_index.as_deref(),
                        &app.index_generator,
                        &app.index_importer
                    ),
                    crate::semantic::availability::SemanticAvailability::RevisionNotIndexed
                );
                assert!(!app.command_enabled(&ToolbarAction::GoToDefinition));
            }
            app.execute_command(&ctx, ToolbarAction::NavigateBack);
            settle_frame(&mut app, &ctx);
            assert_eq!(app.active_revision, Some(After));
            assert_eq!((app.caret_line, app.caret_col), (departure_line, 2));
            assert_eq!(app.history.current().unwrap().revision, Some(After));
        }
        app.open_file("removed.rs");
        settle_frame(&mut app, &ctx);
        assert_eq!(app.active_revision, Some(Before));
        assert_eq!(app.open_buffer.as_ref().unwrap().text, "old\n");
        assert_eq!(
            app.review
                .as_ref()
                .unwrap()
                .change("removed.rs")
                .unwrap()
                .status
                .title(),
            "Deleted"
        );
        app.open_file("docs/README.md");
        settle_frame(&mut app, &ctx);
        app.execute_command(&ctx, ToolbarAction::ToggleRevision(Before));
        settle_frame(&mut app, &ctx);
        let accepted = app.open_buffer.clone().unwrap();
        assert_eq!(
            (accepted.path.as_str(), accepted.text.as_str()),
            ("docs/readme.md", markdown)
        );
        let history = format!("{:?}", app.history);
        let changes = app.pr_changes.clone();
        app.execute_command(&ctx, ToolbarAction::ToggleHideTests);
        let projection = crate::pull_request::ReviewChangesPresentation::new(
            &app.pr_changes,
            &app.session.source_filters,
        );
        assert_eq!(projection.selection_id(&accepted.path, Before), None);
        app.execute_command(&ctx, ToolbarAction::ShowAllFiles);
        assert!(Arc::ptr_eq(&accepted, app.open_buffer.as_ref().unwrap()));
        assert_eq!(format!("{:?}", app.history), history);
        assert_eq!(app.pr_changes, changes);
        app.sidebar_tab = SidebarTab::Search;
        let result = one_search_result(&mut app, "Four", "docs/README.md");
        assert_eq!(result.byte_offset, edited_markdown.find("Four").unwrap());
        let output = frame(&mut app, &ctx, vec![]);
        let row = text_center(&output, "docs/README.md:6");
        frame(&mut app, &ctx, click(row));
        settle_frame(&mut app, &ctx);
        let found = app.open_buffer.as_ref().unwrap();
        assert_eq!(
            (found.path.as_str(), found.revision, found.text.as_str()),
            ("docs/README.md", Some(After), edited_markdown.as_str())
        );
        assert_eq!(
            found.byte_offset(app.caret_line, app.caret_col),
            edited_markdown.find("Four").unwrap()
        );
        app.execute_command(&ctx, ToolbarAction::ExitPRReview);
        settle_frame(&mut app, &ctx);
        assert!(
            app.review.is_none() && app.line_changes.is_none() && app.active_revision.is_none()
        );
        assert_eq!(app.sidebar_tab, SidebarTab::Files);
        assert_eq!(app.repository_root.as_ref(), Some(&root));
        assert_eq!(
            source_only
                .filter_files(&app.all_files)
                .iter()
                .map(|f| f.path.clone())
                .collect::<BTreeSet<_>>(),
            ["a file.rs", "removed.rs", "renamed old.rs"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        );
        assert_eq!(
            app.recent_pull_requests.items(),
            std::slice::from_ref(&review.summary)
        );
    }
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), head);
    assert_eq!(git(&root, &["status", "--porcelain"]), status);
    assert_eq!(
        std::fs::read(root.join("a file.rs")).unwrap(),
        b"checkout only\n"
    );
}
