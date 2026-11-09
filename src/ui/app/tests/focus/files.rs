//! Files follow accepted navigation without taking ownership of the source.
use super::*;
use crate::ui::test_support::{click, text_shapes};

fn file_row(output: &egui::FullOutput, name: &str) -> Option<egui::Pos2> {
    text_shapes(output)
        .filter(|text| text.pos.x < 260.0 && text.galley.job.text == name)
        .map(|text| text.pos + text.galley.size() / 2.0)
        .next()
}

#[test]
fn files_reveal_centers_committed_jumps_and_preserves_passive_scroll_and_caret() {
    for scale in [1.0, 2.0] {
        let (dir, mut app) = reader();
        let root = dir.path().join("checkout");
        for branch in ["b", "z"] {
            let folder = root.join(branch).join("deep");
            std::fs::create_dir_all(&folder).unwrap();
            for n in 0..130 {
                std::fs::write(
                    folder.join(format!("file{n:03}.rs")),
                    "// exact café\r\nfn work() {}\r\n",
                )
                .unwrap();
            }
        }
        std::fs::create_dir(root.join("other")).unwrap();
        std::fs::write(root.join("other/keep.rs"), "fn keep() {}\n").unwrap();
        std::fs::write(root.join("top.rs"), "fn top() {}\n").unwrap();
        open_reader(&mut app, &root, "z/deep/file055.rs");
        app.navigate(SourceLocation::new("z/deep/file055.rs", 5), true);
        settle(&mut app);
        app.toggle_current_bookmark();
        app.open_file("other/keep.rs");
        settle(&mut app);
        app.expanded_dirs.clear();
        app.expanded_dirs.insert("other".into());
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        // egui applies a requested zoom on its next frame. Establish the
        // native viewport before measuring the first mounted Files list.
        let _ = ctx.run(egui::RawInput::default(), |_| {});
        app.execute_command(&ctx, ToolbarAction::OpenBookmark(0));
        settle(&mut app);
        frame(&mut app, &ctx, vec![]);
        let output = frame(&mut app, &ctx, vec![]);
        let point = file_row(&output, "file055.rs").unwrap_or_else(|| {
            panic!(
                "scale {scale}, rows {:?}",
                text_shapes(&output)
                    .filter(|t| t.pos.x < 260.0)
                    .map(|t| (&t.galley.job.text, t.pos))
                    .collect::<Vec<_>>()
            )
        });
        assert!(
            point.y > 280.0 && point.y < 500.0,
            "not centered: {point:?}"
        );
        assert!(app.expanded_dirs.contains("z") && app.expanded_dirs.contains("z/deep"));
        assert!(app.expanded_dirs.contains("other"));
        assert!(!app.expanded_dirs.contains("z/deep/file055.rs"));
        assert_eq!(
            app.open_buffer
                .as_ref()
                .unwrap()
                .byte_offset(app.caret_line, app.caret_col),
            5
        );
        let source = app.open_buffer.clone().unwrap();
        let history = format!("{:?}", app.history);
        let caret = (app.caret_line, app.caret_col);
        frame(&mut app, &ctx, click(point));
        frame(&mut app, &ctx, vec![]);
        assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &source));
        assert_eq!((app.caret_line, app.caret_col), caret);
        assert_eq!(format!("{:?}", app.history), history);
        frame(
            &mut app,
            &ctx,
            vec![
                Event::PointerMoved(point),
                Event::Scroll(egui::vec2(0.0, -240.0)),
            ],
        );
        for _ in 0..20 {
            frame(&mut app, &ctx, vec![]);
        }
        let scrolled = frame(&mut app, &ctx, vec![]);
        let after_scroll = file_row(&scrolled, "file055.rs");
        assert!(after_scroll.is_none_or(|row| (row.y - point.y).abs() > 150.0));
        app.source_viewport.request_focus(&ctx);
        frame(&mut app, &ctx, vec![key(Key::ArrowRight, Modifiers::NONE)]);
        let output = frame(&mut app, &ctx, vec![]);
        assert_eq!(file_row(&output, "file055.rs"), after_scroll);
        let byte = source.byte_offset(app.caret_line, app.caret_col);
        app.navigate(source.location_at_byte(byte), true);
        frame(&mut app, &ctx, vec![]);
        let output = frame(&mut app, &ctx, vec![]);
        let recentered = file_row(&output, "file055.rs").unwrap();
        assert!(
            (recentered.y - point.y).abs() < 6.0,
            "initial {point:?}, recentered {recentered:?}"
        );
        app.expanded_dirs.remove("z/deep");
        let output = frame(&mut app, &ctx, vec![]);
        assert!(file_row(&output, "file055.rs").is_none());
        assert!(!app.expanded_dirs.contains("z/deep"));
        app.sidebar_tab = SidebarTab::Search;
        app.search_options.query = "work".into();
        app.search_options.include_glob = "b/deep/file044.rs".into();
        app.start_repository_search();
        crate::search::session::finish_search(&mut app.repository_search);
        assert!(app.repository_search.error.is_none());
        assert_eq!(app.repository_search.matches.len(), 1);
        assert_eq!(app.repository_search.matches[0].byte_offset, 19);
        let output = frame(&mut app, &ctx, vec![]);
        let point = crate::ui::test_support::text_center(&output, "b/deep/file044.rs:2");
        frame(&mut app, &ctx, click(point));
        settle_frame(&mut app, &ctx);
        let accepted = app.open_buffer.clone().unwrap();
        let history = format!("{:?}", app.history);
        assert_eq!(accepted.path, "b/deep/file044.rs");
        assert_eq!(accepted.byte_offset(app.caret_line, app.caret_col), 19);
        app.sidebar_tab = SidebarTab::Files;
        frame(&mut app, &ctx, vec![]);
        let output = frame(&mut app, &ctx, vec![]);
        let row = file_row(&output, "file044.rs").unwrap();
        assert!(
            row.y > 280.0 && row.y < 500.0,
            "search destination not centered: {row:?}"
        );
        assert!(app.expanded_dirs.contains("b") && app.expanded_dirs.contains("b/deep"));
        assert!(app.expanded_dirs.contains("other"));
        assert!(!app.expanded_dirs.contains("z/deep"));
        assert!(Arc::ptr_eq(&accepted, app.open_buffer.as_ref().unwrap()));
        assert_eq!(format!("{:?}", app.history), history);
        assert_eq!(accepted.byte_offset(app.caret_line, app.caret_col), 19);
        let expansion = app.expanded_dirs.clone();
        app.navigate(SourceLocation::new("top.rs", 0), true);
        settle_frame(&mut app, &ctx);
        assert_eq!(
            app.expanded_dirs, expansion,
            "a top-level leaf has no ancestors to expand"
        );
    }
}

#[test]
fn files_current_exception_preserves_inventory_filters_and_exact_source() {
    let (dir, mut app) = reader();
    for (path, text) in [
        ("src/main.swift", "func main() {}\n"),
        ("tests/nested/Spec.swift", "// source café\r\n"),
        ("docs/guide.md", "guide\n"),
    ] {
        let file = dir.path().join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, text).unwrap();
    }
    app.open_repository(dir.path());
    settle(&mut app);
    let inventory = app.all_files.clone();
    app.session.source_filters = crate::repository::SourceFilters {
        source_files_only: true,
        hides_tests: true,
        path_query: "no-match".into(),
    };
    let filters = app.session.source_filters.clone();
    app.navigate(SourceLocation::new("tests/nested/Spec.swift", 7), true);
    settle(&mut app);
    assert!(Arc::ptr_eq(&inventory, &app.all_files));
    assert_eq!(app.session.source_filters, filters);
    assert_eq!(app.file_tree.len(), 1);
    assert_eq!(app.file_tree[0].name, "tests/nested/Spec.swift");
    assert!(!app.file_tree[0].is_directory);
    assert!(filters.filter_files(&inventory).is_empty());
    app.session.source_filters.path_query.clear();
    app.filters_changed();
    assert_eq!(
        app.file_tree
            .iter()
            .map(|node| node.path.as_str())
            .collect::<Vec<_>>(),
        ["src", "tests"]
    );
    assert_eq!(
        app.file_tree[1].children[0].children[0].path,
        "tests/nested/Spec.swift"
    );
    assert_eq!(
        app.session
            .source_filters
            .filter_files(&inventory)
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>(),
        ["src/main.swift"]
    );
    app.session.source_filters = filters.clone();
    app.filters_changed();
    std::fs::write(dir.path().join("new.txt"), "new\r\n").unwrap();
    app.open_file("new.txt");
    settle(&mut app);
    assert_eq!(app.file_tree.len(), 1);
    assert_eq!(app.file_tree[0].path, "new.txt");
    assert!(!inventory.iter().any(|file| file.path == "new.txt"));
    let buffer = app.open_buffer.clone().unwrap();
    app.open_file("missing.txt");
    settle(&mut app);
    assert!(Arc::ptr_eq(&buffer, app.open_buffer.as_ref().unwrap()));
    assert_eq!(app.file_tree[0].path, "new.txt");
    assert_eq!(app.session.source_filters, filters);
}

#[test]
fn files_deleted_before_row_keeps_revision_caret_and_filters_when_clicked() {
    for scale in [1.0, 2.0] {
        use crate::pull_request::review::{tests::git, ReviewSnapshot};
        let (owner, root, initial) = crate::pull_request::review::tests::fixture();
        git(&root, &["restore", "."]);
        std::fs::create_dir(root.join("src")).unwrap();
        std::fs::rename(root.join("removed.rs"), root.join("src/old.rs")).unwrap();
        git(&root, &["add", "-A"]);
        git(&root, &["commit", "-qm", "nested base"]);
        let mut summary = initial.summary.clone();
        summary.base_ref_oid = Some(git(&root, &["rev-parse", "HEAD"]));
        std::fs::remove_file(root.join("src/old.rs")).unwrap();
        git(&root, &["add", "-A"]);
        git(&root, &["commit", "-qm", "delete nested source"]);
        summary.head_ref_oid = Some(git(&root, &["rev-parse", "HEAD"]));
        let snapshot =
            ReviewSnapshot::prepare(&root, summary, &owner.path().join("cache")).unwrap();
        let checkout_head = git(&root, &["rev-parse", "HEAD"]);
        let checkout_status = git(&root, &["status", "--porcelain"]);
        let mut app = RadiologyApp::from_session(SessionStore::load_from(
            owner.path().join("settings/session.json"),
        ));
        app.open_repository(&root);
        settle(&mut app);
        app.accept_review(snapshot);
        settle(&mut app);
        app.session.source_filters.path_query = "not-listed".into();
        app.session.source_filters.hides_tests = true;
        app.session.source_filters.source_files_only = true;
        app.navigate(
            SourceLocation::with_revision("src/old.rs", 1, Some(ReviewRevision::Before)),
            true,
        );
        settle(&mut app);
        assert!(!app.all_files.iter().any(|file| file.path == "src/old.rs"));
        let source = app.open_buffer.clone().unwrap();
        assert_eq!(source.text, "old\n");
        let history = format!("{:?}", app.history);
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        app.sidebar_tab = SidebarTab::Files;
        for (query, label) in [("not-listed", "src/old.rs"), ("", "old.rs")] {
            app.session.source_filters.path_query = query.into();
            app.filters_changed();
            let filters = app.session.source_filters.clone();
            frame(&mut app, &ctx, vec![]);
            let output = frame(&mut app, &ctx, vec![]);
            let point = file_row(&output, label).unwrap();
            assert!(file_row(&output, "Before").is_some());
            assert!(app.expanded_dirs.contains("src"));
            frame(&mut app, &ctx, click(point));
            frame(&mut app, &ctx, vec![]);
            assert!(Arc::ptr_eq(&source, app.open_buffer.as_ref().unwrap()));
            assert_eq!(app.active_revision, Some(ReviewRevision::Before));
            assert_eq!(source.byte_offset(app.caret_line, app.caret_col), 1);
            assert_eq!(app.session.source_filters, filters);
            assert_eq!(format!("{:?}", app.history), history);
        }
        assert_eq!(git(&root, &["rev-parse", "HEAD"]), checkout_head);
        assert_eq!(git(&root, &["status", "--porcelain"]), checkout_status);
        assert!(!root.join("src/old.rs").exists());
    }
}
