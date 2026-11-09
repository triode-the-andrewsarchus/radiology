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
