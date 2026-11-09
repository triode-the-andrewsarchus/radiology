//! Bookmark presentation and numbered-command boundaries from the Swift app.
use super::{focus::frame, *};
use crate::ui::bookmark_list::BookmarkAction;

#[test]
fn bookmark_labels_use_the_innermost_declaration_and_plain_filename() {
    for scale in [1.0, 2.0] {
        let text = "// outside\nfn outer() {\n    fn inner() {\n        let café = 1;\n    }\n}\n";
        let (owner, mut app) = isolated_source("src/nested.rs", text);
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        for (needle, expected) in [
            ("café", "inner"),
            ("fn inner", "inner"),
            ("// outside", "nested.rs"),
        ] {
            let byte = text.find(needle).unwrap();
            app.navigate(SourceLocation::new("src/nested.rs", byte), true);
            frame(&mut app, &ctx, vec![]);
            app.execute_command(&ctx, ToolbarAction::ToggleBookmark);
            assert_eq!(app.bookmarks.bookmarks.last().unwrap().label, expected);
        }
        std::fs::write(owner.path().join("checkout/NOTES.md"), "# notes\r\n").unwrap();
        app.open_file("NOTES.md");
        settle(&mut app);
        app.execute_command(&ctx, ToolbarAction::ToggleBookmark);
        assert_eq!(app.bookmarks.bookmarks.last().unwrap().label, "NOTES.md");
        let saved = BookmarkStore::load_in(
            &owner.path().join("settings"),
            &app.bookmark_scope().unwrap(),
        );
        assert_eq!(saved.bookmarks, app.bookmarks.bookmarks);
        let output = frame(&mut app, &ctx, vec![]);
        assert!(
            crate::ui::test_support::text_shapes(&output).any(|shape| shape
                .galley
                .job
                .text
                .contains("inner"))
        );
    }
}

#[test]
fn only_ten_numbered_bookmark_commands_are_enabled_but_every_identity_can_open() {
    let text: String = (0..12).map(|n| format!("// point {n}\n")).collect();
    let (owner, mut app) = isolated_source("locations.rs", &text);
    let ctx = Context::default();
    for line in 1..=12 {
        app.jump_to_location("locations.rs", line, 0);
        settle(&mut app);
        app.execute_command(&ctx, ToolbarAction::ToggleBookmark);
    }
    let last = app.bookmarks.bookmarks[11].clone();
    app.handle_bookmark_action(
        &ctx,
        BookmarkAction::Move {
            id: last.id,
            slot: 0,
        },
    );
    let expected = app.bookmarks.bookmarks.clone();
    for slot in 0..10 {
        assert!(app.command_enabled(&ToolbarAction::OpenBookmark(slot)));
    }
    for slot in [10, 11, usize::MAX] {
        assert!(!app.command_enabled(&ToolbarAction::OpenBookmark(slot)));
        let before = format!("{:?}", app.history);
        app.execute_command(&ctx, ToolbarAction::OpenBookmark(slot));
        assert!(!app.is_source_loading());
        assert_eq!(format!("{:?}", app.history), before);
    }
    app.execute_command(&ctx, ToolbarAction::OpenBookmark(9));
    settle(&mut app);
    assert_eq!(app.caret_line, expected[9].line);
    app.handle_bookmark_action(&ctx, BookmarkAction::Open(expected[11].id));
    settle(&mut app);
    assert_eq!(app.caret_line, expected[11].line);
    assert_eq!(app.bookmark_list.selected, Some(expected[11].id));
    assert_eq!(
        BookmarkStore::load_in(
            &owner.path().join("settings"),
            &app.bookmark_scope().unwrap(),
        )
        .bookmarks,
        expected
    );
}

#[test]
fn reordered_shortcuts_restore_exact_departures_and_refresh_relocated_unicode_bookmarks() {
    use super::focus::{key, settle_frame};
    use egui::{Key, Modifiers};
    for scale in [1.0, 2.0] {
        let text = "fn café() {\r\n    let s = \"🚀\"; target();\r\n}\r\n";
        let (owner, mut app) = isolated_source("unicode.rs", text);
        let root = owner.path().join("checkout");
        std::fs::write(root.join("other.rs"), "// origin\nfn other() {}\n").unwrap();
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        let byte = text.find("target").unwrap() + 2;
        app.navigate(SourceLocation::new("unicode.rs", byte), true);
        frame(&mut app, &ctx, vec![]);
        // Passive viewport reports must not replace the caret used by Ctrl+D.
        app.source_viewport
            .restore_top(app.open_buffer.as_ref().unwrap(), 0);
        app.source_viewport.request_focus(&ctx);
        frame(&mut app, &ctx, vec![key(Key::D, Modifiers::CTRL)]);
        let first = app.bookmarks.bookmarks[0].clone();
        assert_eq!(first.byte_offset, byte);
        assert!(app.current_bookmarked());
        app.open_file("other.rs");
        settle_frame(&mut app, &ctx);
        app.jump_to_location("other.rs", 2, 4);
        app.source_viewport.request_focus(&ctx);
        frame(&mut app, &ctx, vec![]);
        frame(&mut app, &ctx, vec![key(Key::D, Modifiers::CTRL)]);
        let second = app.bookmarks.bookmarks[1].clone();
        frame(&mut app, &ctx, vec![key(Key::Num0, Modifiers::CTRL)]);
        settle_frame(&mut app, &ctx);
        assert_bookmark_location(&app, &first);
        app.execute_command(&ctx, ToolbarAction::NavigateBack);
        settle_frame(&mut app, &ctx);
        assert_bookmark_location(&app, &second);
        app.handle_bookmark_action(
            &ctx,
            BookmarkAction::Move {
                id: second.id,
                slot: 0,
            },
        );
        for (slot, expected) in [(1, &first), (0, &second)] {
            app.execute_command(&ctx, ToolbarAction::OpenBookmark(slot));
            settle_frame(&mut app, &ctx);
            assert_bookmark_location(&app, expected);
        }
        let mut relaunched = RadiologyApp::from_session(SessionStore::load_from(
            owner.path().join("settings/session.json"),
        ));
        relaunched.open_repository(&root);
        settle(&mut relaunched);
        assert_eq!(
            relaunched.bookmarks.bookmarks,
            vec![second.clone(), first.clone()]
        );
        assert!(relaunched.open_buffer.is_none());
        std::fs::write(root.join("unicode.rs"), format!("// inserted\n{text}")).unwrap();
        app.execute_command(&ctx, ToolbarAction::OpenBookmark(1));
        settle_frame(&mut app, &ctx);
        let refreshed = app.bookmarks.bookmarks[1].clone();
        assert_eq!(
            (refreshed.id, refreshed.label.as_str()),
            (first.id, first.label.as_str())
        );
        assert_eq!(
            (refreshed.byte_offset, refreshed.line),
            (byte + 12, first.line + 1)
        );
        assert_bookmark_location(&app, &refreshed);
        assert!(app.current_bookmarked());
        assert_eq!(
            BookmarkStore::load_in(
                &owner.path().join("settings"),
                &app.bookmark_scope().unwrap()
            )
            .bookmarks,
            app.bookmarks.bookmarks
        );
        app.execute_command(&ctx, ToolbarAction::ToggleBookmark);
        assert_eq!(app.bookmarks.bookmarks, vec![second]);
        assert_eq!(
            std::fs::read_to_string(root.join("unicode.rs")).unwrap(),
            format!("// inserted\n{text}")
        );
    }
}

fn assert_bookmark_location(app: &RadiologyApp, expected: &crate::bookmarks::SourceBookmark) {
    let source = app.open_buffer.as_ref().unwrap();
    assert_eq!(
        source.location_at_byte(source.byte_offset(app.caret_line, app.caret_col)),
        expected.location()
    );
}

#[test]
fn checkout_two_pull_requests_and_another_root_restore_independent_bookmark_lists() {
    use crate::bookmarks::store::BookmarkScope;
    use crate::pull_request::review::{tests::fixture_with_sources, ReviewSnapshot};
    let before = "// 🚀 before\r\nfn café() {}\r\n";
    let after = "// 🚀 after\r\nfn café() { changed(); }\r\n";
    for scale in [1.0, 2.0] {
        let (repository, root, first) = fixture_with_sources(before, after);
        let (settings, mut app) = reader();
        open_reader(&mut app, &root, "a file.rs");
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        app.execute_command(&ctx, ToolbarAction::ToggleBookmark);
        let local = app.bookmarks.bookmarks[0].clone();
        app.accept_review(first.clone());
        settle(&mut app);
        app.open_file("a file.rs");
        settle(&mut app);
        assert_eq!(
            app.bookmark_scope(),
            Some(BookmarkScope::new(&root, Some(7)))
        );
        assert!(app.bookmarks.bookmarks.is_empty());
        assert_ne!(app.review.as_ref().unwrap().head_root, root);
        for revision in [ReviewRevision::After, ReviewRevision::Before] {
            app.execute_command(&ctx, ToolbarAction::ToggleRevision(revision));
            settle(&mut app);
            app.jump_to_location("a file.rs", 2, 4);
            settle(&mut app);
            frame(&mut app, &ctx, vec![]);
            app.execute_command(&ctx, ToolbarAction::ToggleBookmark);
        }
        let pair = app.bookmarks.bookmarks.clone();
        assert_eq!(pair.len(), 2);
        assert_eq!(pair[0].revision, Some(ReviewRevision::After));
        assert_eq!(pair[1].revision, Some(ReviewRevision::Before));
        let mut summary = first.summary.clone();
        summary.number = 8;
        let second =
            ReviewSnapshot::prepare(&root, summary, &repository.path().join("cache")).unwrap();
        app.accept_review(second.clone());
        settle(&mut app);
        assert!(app.bookmarks.bookmarks.is_empty());
        app.open_file("added.rs");
        settle(&mut app);
        app.execute_command(&ctx, ToolbarAction::ToggleBookmark);
        let other_pr = app.bookmarks.bookmarks.clone();
        app.accept_review(first.clone());
        settle(&mut app);
        assert_eq!(app.bookmarks.bookmarks, pair);
        for slot in [1, 0] {
            app.execute_command(&ctx, ToolbarAction::OpenBookmark(slot));
            settle(&mut app);
            assert_bookmark_location(&app, &pair[slot]);
            assert_eq!(
                app.open_buffer.as_ref().unwrap().text,
                if slot == 1 { before } else { after }
            );
        }
        app.execute_command(&ctx, ToolbarAction::NavigateBack);
        settle(&mut app);
        // History restores the source coordinate independently of list selection.
        let source = app.open_buffer.as_ref().unwrap();
        assert_eq!(
            source.location_at_byte(source.byte_offset(app.caret_line, app.caret_col)),
            pair[1].location()
        );
        app.execute_command(&ctx, ToolbarAction::ExitPRReview);
        settle(&mut app);
        assert_eq!(app.bookmarks.bookmarks, vec![local.clone()]);
        let other_root = settings.path().join("other-root");
        open_empty_repository(&mut app, &other_root);
        assert!(app.bookmarks.bookmarks.is_empty());
        app.open_repository(&root);
        settle(&mut app);
        assert_eq!(app.bookmarks.bookmarks, vec![local]);
        app.accept_review(second);
        settle(&mut app);
        assert_eq!(app.bookmarks.bookmarks, other_pr);
        assert_eq!(
            std::fs::read_to_string(root.join("a file.rs")).unwrap(),
            "checkout only\n"
        );
    }
}

#[test]
fn missing_bookmark_and_retired_preparation_preserve_the_current_reader_and_new_scope() {
    for scale in [1.0, 2.0] {
        let (owner, mut app, id) = reader_with_missing_bookmark();
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        let accepted = app.open_buffer.clone().unwrap();
        let history = format!("{:?}", app.history);
        let caret = (app.caret_line, app.caret_col);
        app.execute_command(&ctx, ToolbarAction::OpenBookmark(0));
        settle(&mut app);
        assert!(app.bookmark_list.navigation_error.is_some());
        assert_eq!(app.bookmarks.bookmarks[0].id, id);
        assert!(Arc::ptr_eq(&accepted, app.open_buffer.as_ref().unwrap()));
        assert_eq!(format!("{:?}", app.history), history);
        assert_eq!((app.caret_line, app.caret_col), caret);
        // A failed request superseded by root discovery cannot leak an old error.
        app.execute_command(&ctx, ToolbarAction::OpenBookmark(0));
        let next = owner.path().join("next");
        open_empty_repository(&mut app, &next);
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![]);
        }
        assert_empty_bookmark_reader(&app, &next);
        // Separately hold an actual successful old bookmark preparation, change
        // scope, then exercise the publication guard with that completed result.
        let original_root = owner.path().join("repo");
        repository(&original_root);
        open_reader(&mut app, &original_root, "b.rs");
        app.execute_command(&ctx, ToolbarAction::OpenBookmark(0));
        let retired = take_prepared_source(&mut app);
        assert_eq!(retired.buffer.path, "a.rs");
        app.open_repository(&next);
        settle(&mut app);
        app.publish_source(retired);
        frame(&mut app, &ctx, vec![]);
        assert!(app.pending_navigation.is_none());
        assert_empty_bookmark_reader(&app, &next);
    }
}

fn assert_empty_bookmark_reader(app: &RadiologyApp, root: &Path) {
    assert!(app.bookmarks.bookmarks.is_empty());
    assert!(app.bookmark_list.navigation_error.is_none());
    assert!(app.open_buffer.is_none());
    assert!(app.history.current().is_none());
    assert_eq!(app.repository_root.as_deref(), Some(root));
    assert!(app.error.is_none());
}

#[test]
fn deleting_the_selected_bookmark_clears_selection_without_opening_its_neighbor() {
    use super::focus::key;
    use crate::ui::bookmark_list::BookmarkList;
    use egui::{Key, Modifiers};
    for scale in [1.0, 2.0] {
        let (_owner, mut app) = opened_repository();
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        for line in [1, 2] {
            app.jump_to_location("a.rs", line, 0);
            settle(&mut app);
            app.execute_command(&ctx, ToolbarAction::ToggleBookmark);
        }
        frame(&mut app, &ctx, vec![]);
        ctx.memory_mut(|memory| memory.request_focus(BookmarkList::focus_id()));
        let source = app.open_buffer.clone().unwrap();
        let history = format!("{:?}", app.history);
        frame(
            &mut app,
            &ctx,
            vec![
                key(Key::Home, Modifiers::NONE),
                key(Key::Delete, Modifiers::NONE),
                key(Key::Enter, Modifiers::NONE),
            ],
        );
        assert!(app.bookmark_list.selected.is_none());
        assert!(!app.is_source_loading());
        assert_eq!(app.bookmarks.bookmarks.len(), 1);
        assert!(Arc::ptr_eq(&source, app.open_buffer.as_ref().unwrap()));
        assert_eq!(format!("{:?}", app.history), history);
    }
}
