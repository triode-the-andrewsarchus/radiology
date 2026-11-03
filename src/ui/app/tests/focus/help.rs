use super::*;
use crate::ui::test_support::{click, text_center};

#[test]
fn keyboard_reference_owns_input_and_returns_exact_reader_selection_and_history() {
    let (_owner, mut app, ctx) = source_reader();
    let buffer = app.open_buffer.clone().unwrap();
    let selected = buffer.line_byte_range(2);
    app.source_viewport.select(selected.clone());
    app.source_viewport.request_focus(&ctx);
    frame(&mut app, &ctx, vec![]);
    let caret = (app.caret_line, app.caret_col);
    let history = format!("{:?}", app.history);
    frame(
        &mut app,
        &ctx,
        vec![
            key(Key::Slash, Modifiers::CTRL),
            key(Key::D, Modifiers::CTRL),
            Event::Text("x".into()),
        ],
    );
    assert!(matches!(app.modal, ActiveModal::KeyboardShortcuts(_)));
    frame(&mut app, &ctx, vec![]);
    let output = frame(&mut app, &ctx, vec![]);
    text_center(&output, "Keyboard Shortcuts");
    text_center(&output, "Done");
    assert!(app.bookmarks.bookmarks.is_empty());
    assert_eq!((app.caret_line, app.caret_col), caret);
    frame(
        &mut app,
        &ctx,
        vec![
            key(Key::PageDown, Modifiers::NONE),
            key(Key::End, Modifiers::NONE),
            Event::Copy,
        ],
    );
    let output = frame(
        &mut app,
        &ctx,
        vec![
            key(Key::Escape, Modifiers::NONE),
            key(Key::ArrowDown, Modifiers::NONE),
            key(Key::D, Modifiers::CTRL),
        ],
    );
    assert!(matches!(app.modal, ActiveModal::None));
    assert!(output.platform_output.copied_text.is_empty());
    assert!(app.bookmarks.bookmarks.is_empty());
    assert!(app.source_viewport.has_focus(&ctx));
    assert_eq!(format!("{:?}", app.history), history);
    assert_eq!((app.caret_line, app.caret_col), caret);
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    assert_eq!(
        frame(&mut app, &ctx, vec![Event::Copy])
            .platform_output
            .copied_text,
        buffer.text[selected]
    );
}

#[test]
fn help_menu_welcome_copy_keyboard_done_and_narrow_scrolling_work_at_both_scales() {
    for scale in [1.0, 2.0] {
        let (_owner, mut app, ctx, output) = welcome_frame(scale);
        let point = text_center(&output, "Help");
        frame(&mut app, &ctx, click(point));
        frame(&mut app, &ctx, vec![]);
        frame(&mut app, &ctx, vec![key(Key::ArrowDown, Modifiers::NONE)]);
        frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
        frame(&mut app, &ctx, vec![]);
        assert!(matches!(app.modal, ActiveModal::KeyboardShortcuts(_)));
        let output = frame(&mut app, &ctx, vec![]);
        let screen = ctx.screen_rect();
        assert!(screen.contains(text_center(&output, "Done")));
        // Done initially owns focus; Tab wraps to the document's Copy control.
        frame(&mut app, &ctx, vec![key(Key::Tab, Modifiers::NONE)]);
        let output = frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
        assert_eq!(
            output.platform_output.copied_text,
            include_str!("../../../../../plans/keyboard-shortcuts-linux.md")
        );
        assert!(matches!(app.modal, ActiveModal::KeyboardShortcuts(_)));
        for key_code in [Key::End, Key::PageUp, Key::Home] {
            let output = frame_sized(
                &mut app,
                &ctx,
                vec![key(key_code, Modifiers::NONE)],
                egui::vec2(640.0, 480.0),
                |_| {},
            );
            assert!(ctx.screen_rect().contains(text_center(&output, "Done")));
            if key_code != Key::PageUp {
                // A visible footer alone would miss an invalid/blank scroll offset.
                let prefix = if key_code == Key::End {
                    "Window-management shortcuts"
                } else {
                    "Open this reference"
                };
                let output = frame_sized(&mut app, &ctx, vec![], egui::vec2(640.0, 480.0), |_| {});
                assert!(output.shapes.iter().any(|shape| {
                    matches!(&shape.shape, egui::Shape::Text(text)
                        if text.galley.job.text.starts_with(prefix)
                        && shape.clip_rect.contains_rect(egui::Rect::from_min_size(text.pos, text.galley.size())))
                }), "Home/End must reveal the complete boundary paragraph");
            }
        }
        frame(&mut app, &ctx, vec![key(Key::Tab, Modifiers::NONE)]);
        frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
        assert!(matches!(app.modal, ActiveModal::None));
        assert!(app.repository_root.is_none());
    }
}
