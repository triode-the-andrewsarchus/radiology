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
        let folder = dir.path().join("z/deep");
        std::fs::create_dir_all(&folder).unwrap();
        for n in 0..130 {
            std::fs::write(
                folder.join(format!("file{n:03}.rs")),
                "// exact café\r\nfn work() {}\r\n",
            )
            .unwrap();
        }
        app.open_repository(dir.path());
        settle(&mut app);
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        // egui applies a requested zoom on its next frame. Establish the
        // native viewport before measuring the first mounted Files list.
        let _ = ctx.run(egui::RawInput::default(), |_| {});
        app.navigate(SourceLocation::new("z/deep/file055.rs", 5), true);
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
        frame(&mut app, &ctx, vec![]);
        app.navigate(source.location_at_byte(9), true);
        app.sidebar_tab = SidebarTab::Files;
        frame(&mut app, &ctx, vec![]);
        let output = frame(&mut app, &ctx, vec![]);
        assert!(file_row(&output, "file055.rs").is_some());
        assert_eq!(source.byte_offset(app.caret_line, app.caret_col), 9);
    }
}

#[test]
fn files_current_exception_preserves_inventory_filters_and_exact_source() {
    let (dir, mut app) = reader();
    std::fs::create_dir_all(dir.path().join("tests")).unwrap();
    std::fs::write(dir.path().join("tests/notes.md"), "// source café\r\n").unwrap();
    std::fs::write(dir.path().join("other.rs"), "fn other() {}\n").unwrap();
    app.open_repository(dir.path());
    settle(&mut app);
    let inventory = app.all_files.clone();
    app.session.source_filters = crate::repository::SourceFilters {
        source_files_only: true,
        hides_tests: true,
        path_query: "no-match".into(),
    };
    let filters = app.session.source_filters.clone();
    app.navigate(SourceLocation::new("tests/notes.md", 7), true);
    settle(&mut app);
    assert!(Arc::ptr_eq(&inventory, &app.all_files));
    assert_eq!(app.session.source_filters, filters);
    assert_eq!(app.file_tree.len(), 1);
    assert_eq!(app.file_tree[0].name, "tests/notes.md");
    assert!(!app.file_tree[0].is_directory);
    assert!(filters.filter_files(&inventory).is_empty());
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
        let (owner, root, snapshot) = crate::pull_request::review::tests::fixture();
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
            SourceLocation::with_revision("removed.rs", 1, Some(ReviewRevision::Before)),
            true,
        );
        settle(&mut app);
        let filters = app.session.source_filters.clone();
        assert!(!app.all_files.iter().any(|file| file.path == "removed.rs"));
        let source = app.open_buffer.clone().unwrap();
        assert_eq!(source.text, "old\n");
        let history = format!("{:?}", app.history);
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        app.sidebar_tab = SidebarTab::Files;
        frame(&mut app, &ctx, vec![]);
        let output = frame(&mut app, &ctx, vec![]);
        let point = file_row(&output, "removed.rs").unwrap();
        assert!(file_row(&output, "Before").is_some());
        frame(&mut app, &ctx, click(point));
        frame(&mut app, &ctx, vec![]);
        assert!(Arc::ptr_eq(&source, app.open_buffer.as_ref().unwrap()));
        assert_eq!(app.active_revision, Some(ReviewRevision::Before));
        assert_eq!(source.byte_offset(app.caret_line, app.caret_col), 1);
        assert_eq!(app.session.source_filters, filters);
        assert_eq!(format!("{:?}", app.history), history);
    }
}
