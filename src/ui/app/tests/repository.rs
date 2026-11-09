//! Repository discovery, filtering and source-loading parity contracts.
use super::{focus::frame, *};

#[cfg(unix)]
#[test]
fn external_checkout_links_cannot_replace_the_reader_through_file_or_bookmark_navigation() {
    use std::os::unix::fs::symlink;
    for scale in [1.0, 2.0] {
        for bookmark in [false, true] {
            let (owner, mut app, id) = reader_with_bookmark();
            let root = app.repository_root.clone().unwrap();
            let path = root.join("a.rs");
            let original = std::fs::read(&path).unwrap();
            let outside = owner.path().join("outside.rs");
            std::fs::write(&outside, &original).unwrap();
            app.open_file("b.rs");
            settle(&mut app);
            let ctx = Context::default();
            ctx.set_pixels_per_point(scale);
            frame(&mut app, &ctx, vec![]);
            let accepted = app.open_buffer.clone().unwrap();
            let caret = (app.caret_line, app.caret_col);
            let history = format!("{:?}", app.history);
            std::fs::remove_file(&path).unwrap();
            symlink(&outside, &path).unwrap();
            if bookmark {
                app.execute_command(&ctx, ToolbarAction::OpenBookmark(0));
            } else {
                app.open_file("a.rs");
            }
            settle(&mut app);
            assert!(Arc::ptr_eq(&accepted, app.open_buffer.as_ref().unwrap()));
            assert_eq!((app.caret_line, app.caret_col), caret);
            assert_eq!(format!("{:?}", app.history), history);
            assert_eq!(app.bookmarks.bookmarks[0].id, id);
            let error = if bookmark {
                assert!(app.error.is_none());
                &app.bookmark_list.navigation_error.as_ref().unwrap().1
            } else {
                app.error.as_ref().unwrap()
            };
            assert!(error.contains("leaves the repository"));
            assert_eq!(std::fs::read(&outside).unwrap(), original);
        }
    }
}

#[test]
fn filters_preserve_reader_preferences_and_manual_directory_expansion_through_refresh() {
    use crate::repository::SourceFilters;
    for scale in [1.0, 2.0] {
        let (owner, mut app) = reader();
        let root = owner.path().join("filtered-root");
        for (path, text) in [
            ("README.md", "# Notes\n"),
            ("src/app.rs", "fn main() {}\n"),
            ("src/contest.rs", "fn contest() {}\n"),
            ("tests/integration.rs", "#[test]\nfn t() {}\n"),
            ("docs/guide.md", "guide\n"),
        ] {
            let file = root.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, text).unwrap();
        }
        open_reader(&mut app, &root, "README.md");
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        app.expanded_dirs.extend(["src".into(), "docs".into()]);
        let inventory = app.all_files.clone();
        let accepted = app.open_buffer.clone().unwrap();
        let history = format!("{:?}", app.history);
        app.execute_command(&ctx, ToolbarAction::ToggleSourceFilesOnly);
        assert_eq!(
            app.session
                .source_filters
                .filter_files(&app.all_files)
                .len(),
            3
        );
        assert!(Arc::ptr_eq(&accepted, app.open_buffer.as_ref().unwrap()));
        assert_eq!(app.selected_path.as_deref(), Some("README.md"));
        assert!(!app.session.source_filters.includes_path("README.md"));
        assert_eq!(format!("{:?}", app.history), history);
        assert!(Arc::ptr_eq(&inventory, &app.all_files));
        app.execute_command(&ctx, ToolbarAction::ToggleHideTests);
        assert_eq!(
            app.session
                .source_filters
                .filter_files(&app.all_files)
                .len(),
            2
        );
        app.session.source_filters.path_query = "rs".into();
        app.filters_changed();
        assert_eq!(
            app.session
                .source_filters
                .filter_files(&app.all_files)
                .iter()
                .map(|file| file.path.as_str())
                .collect::<Vec<_>>(),
            ["src/app.rs", "src/contest.rs"]
        );
        app.execute_command(&ctx, ToolbarAction::ToggleSourceFilesOnly);
        assert_eq!(
            app.session
                .source_filters
                .filter_files(&app.all_files)
                .len(),
            2
        );
        assert!(app.expanded_dirs.contains("src") && app.expanded_dirs.contains("docs"));
        assert!(Arc::ptr_eq(&accepted, app.open_buffer.as_ref().unwrap()));
        assert_eq!(format!("{:?}", app.history), history);
        let restored = RadiologyApp::from_session(SessionStore::load_from(
            owner.path().join("settings/session.json"),
        ));
        assert_eq!(
            restored.session.source_filters,
            SourceFilters {
                hides_tests: true,
                ..Default::default()
            }
        );
        app.execute_command(&ctx, ToolbarAction::ShowAllFiles);
        assert_eq!(app.session.source_filters, SourceFilters::default());
        assert_eq!(
            SessionStore::load_from(owner.path().join("settings/session.json")).source_filters,
            SourceFilters::default()
        );
        assert_eq!(
            app.session
                .source_filters
                .filter_files(&app.all_files)
                .len(),
            5
        );
        app.execute_command(&ctx, ToolbarAction::ToggleSourceFilesOnly);
        std::fs::write(root.join("src/added.rs"), "fn added() {}\n").unwrap();
        app.execute_command(&ctx, ToolbarAction::Refresh);
        settle(&mut app);
        frame(&mut app, &ctx, vec![]);
        assert_eq!(app.all_files.len(), 6);
        assert_eq!(
            app.session
                .source_filters
                .filter_files(&app.all_files)
                .len(),
            4
        );
        assert_eq!(app.selected_path.as_deref(), Some("README.md"));
        assert_eq!(app.open_buffer.as_ref().unwrap().text, accepted.text);
        assert!(
            app.expanded_dirs.contains("src"),
            "refresh must retain manually expanded folders unrelated to the open file"
        );
        assert!(
            app.expanded_dirs.contains("docs"),
            "a temporarily filtered-out disclosure must survive refresh"
        );
        assert!(
            !app.expanded_dirs.contains("tests"),
            "collapsed folders stay collapsed"
        );
        app.execute_command(&ctx, ToolbarAction::ShowAllFiles);
        frame(&mut app, &ctx, vec![]);
        let output = frame(&mut app, &ctx, vec![]);
        assert!(crate::ui::test_support::text_shapes(&output)
            .any(|text| text.galley.job.text == "guide.md"));
        app.open_repository(&root);
        settle(&mut app);
        assert!(
            app.open_buffer.is_none(),
            "plain opens still wait for a file choice"
        );
        assert!(app.expanded_dirs.contains("src") && app.expanded_dirs.contains("docs"));
        let other = owner.path().join("other");
        open_empty_repository(&mut app, &other);
        assert!(
            app.expanded_dirs.is_empty(),
            "a different root must not inherit disclosures"
        );
    }
}

#[test]
fn natural_file_order_reaches_discovery_filtered_rows_and_exact_picker_navigation() {
    use super::focus::{finish_file_picker, key, settle_frame};
    use egui::{Event, Key, Modifiers};
    for scale in [1.0, 2.0] {
        let (owner, mut app) = reader();
        let root = owner.path().join("numbered");
        std::fs::create_dir(&root).unwrap();
        for (path, text) in [
            ("file10.rs", "// ten\r\n"),
            ("file2.rs", "// two café\r\n"),
            ("file2x.rs", "// two x\r\n"),
        ] {
            std::fs::write(root.join(path), text).unwrap();
        }
        app.open_repository(&root);
        settle(&mut app);
        assert_eq!(
            app.all_files
                .iter()
                .map(|file| file.path.as_str())
                .collect::<Vec<_>>(),
            ["file2.rs", "file2x.rs", "file10.rs"]
        );
        app.session.source_filters.path_query = "file".into();
        app.filters_changed();
        assert_eq!(
            app.file_tree
                .iter()
                .map(|node| node.path.as_str())
                .collect::<Vec<_>>(),
            ["file2.rs", "file2x.rs", "file10.rs"]
        );
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        frame(&mut app, &ctx, vec![]);
        app.execute_command(&ctx, ToolbarAction::OpenQuickFilePicker);
        frame(&mut app, &ctx, vec![]);
        frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
        finish_file_picker(&mut app, &ctx);
        settle_frame(&mut app, &ctx);
        assert_eq!(app.selected_path.as_deref(), Some("file2.rs"));
        assert_eq!(app.open_buffer.as_ref().unwrap().text, "// two café\r\n");
        app.source_viewport.request_focus(&ctx);
        let output = frame(
            &mut app,
            &ctx,
            vec![key(Key::A, Modifiers::CTRL), Event::Copy],
        );
        assert_eq!(output.platform_output.copied_text, "// two café\r\n");
    }
    let (owner, root, first) = crate::pull_request::review::tests::fixture();
    let review = super::review_refresh::advance(&root, owner.path(), &first, |root| {
        std::fs::write(root.join("file10.rs"), "// ten PR\n").unwrap();
        std::fs::write(root.join("file2.rs"), "// two PR\n").unwrap();
    });
    let (_settings, mut app) =
        super::review_refresh::open_review(&root, review, ReviewRevision::After);
    assert_eq!(
        app.all_files
            .iter()
            .filter(|file| file.path.starts_with("file"))
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>(),
        ["file2.rs", "file10.rs"]
    );
    let ctx = Context::default();
    app.execute_command(&ctx, ToolbarAction::OpenQuickFilePicker);
    frame(&mut app, &ctx, vec![]);
    frame(
        &mut app,
        &ctx,
        vec![Event::Text("file".into()), key(Key::Enter, Modifiers::NONE)],
    );
    finish_file_picker(&mut app, &ctx);
    settle_frame(&mut app, &ctx);
    assert_eq!(app.selected_path.as_deref(), Some("file2.rs"));
    assert_eq!(app.active_revision, Some(ReviewRevision::After));
    assert_eq!(app.open_buffer.as_ref().unwrap().text, "// two PR\n");
}
