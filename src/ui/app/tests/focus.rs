//! Real app frames exercise modal/canvas focus rather than a parallel test renderer.
use super::*;
mod ownership;
mod references;
use egui::{Event, Key, Modifiers};

fn key(key: Key, modifiers: Modifiers) -> Event {
    let mut event = crate::ui::commands::test_key(key);
    if let Event::Key {
        modifiers: actual, ..
    } = &mut event
    {
        *actual = modifiers;
    }
    event
}

fn frame(app: &mut RadiologyApp, ctx: &Context, events: Vec<Event>) -> egui::FullOutput {
    use eframe::App;
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1100.0, 800.0),
        )),
        events,
        ..Default::default()
    };
    app.raw_input_hook(ctx, &mut input);
    ctx.run(input, |ctx| app.render(ctx))
}

fn source_reader() -> (tempfile::TempDir, RadiologyApp, Context) {
    let (dir, mut app) = opened_repository();
    app.open_file("a.rs");
    settle(&mut app);
    let ctx = Context::default();
    frame(&mut app, &ctx, vec![]);
    (dir, app, ctx)
}

fn assert_source_line_copy(
    app: &mut RadiologyApp,
    ctx: &Context,
    caret: (usize, usize),
    expected: &str,
) {
    assert!(app.source_viewport.has_focus(ctx));
    assert_eq!((app.caret_line, app.caret_col), caret);
    let output = frame(app, ctx, vec![key(Key::End, Modifiers::SHIFT), Event::Copy]);
    assert_eq!(output.platform_output.copied_text, expected);
}

fn open_toolbar_menu(app: &mut RadiologyApp, ctx: &Context, label: &str) {
    let output = frame(app, ctx, vec![]);
    app.source_viewport.request_focus(ctx);
    frame(
        app,
        ctx,
        crate::ui::test_support::click(crate::ui::test_support::text_center(&output, label)),
    );
}

#[test]
fn committed_line_modal_returns_source_keys_without_a_click_and_owns_closing_batch() {
    let (_dir, mut app, ctx) = source_reader();
    app.source_viewport.request_focus(&ctx);
    frame(&mut app, &ctx, vec![key(Key::L, Modifiers::CTRL)]);
    frame(&mut app, &ctx, vec![Event::Text("2".into())]);
    frame(
        &mut app,
        &ctx,
        vec![
            key(Key::Enter, Modifiers::NONE),
            key(Key::ArrowDown, Modifiers::NONE),
        ],
    );
    assert!(matches!(app.modal, ActiveModal::None));
    assert_source_line_copy(&mut app, &ctx, (2, 0), "    let café = 1;");
}

#[test]
fn cancelled_modal_restores_the_live_find_query_and_source_cancellation_restores_canvas() {
    let (_dir, mut app, ctx) = source_reader();
    frame(&mut app, &ctx, vec![key(Key::F, Modifiers::CTRL)]);
    frame(&mut app, &ctx, vec![]);
    let query_focus = ctx.memory(|memory| memory.focused()).unwrap();
    assert!(!app.source_viewport.has_focus(&ctx));
    frame(&mut app, &ctx, vec![key(Key::L, Modifiers::CTRL)]);
    frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(query_focus));
    frame(&mut app, &ctx, vec![Event::Text("café".into())]);
    assert_eq!(app.source_find.search.options.query, "café");
    assert_eq!((app.caret_line, app.caret_col), (1, 0));

    app.source_viewport.request_focus(&ctx);
    frame(&mut app, &ctx, vec![key(Key::T, Modifiers::CTRL)]);
    frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    assert!(app.source_viewport.has_focus(&ctx));
    frame(&mut app, &ctx, vec![key(Key::End, Modifiers::NONE)]);
    assert!(app.caret_col > 0);
}

#[test]
fn breadcrumb_commit_returns_source_keys_after_pointer_focus() {
    let (_dir, mut app, ctx) = source_reader();
    app.fold_store.unfold_all();
    app.jump_to_location("a.rs", 2, 0);
    frame(&mut app, &ctx, vec![]);
    let output = frame(&mut app, &ctx, vec![]);
    let symbol = output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == "alpha" && text.pos.y < 65.0 => {
                Some(text.pos + text.galley.size() / 2.0)
            }
            _ => None,
        })
        .expect("source breadcrumb");
    frame(&mut app, &ctx, crate::ui::test_support::click(symbol));
    assert_eq!(app.caret_line, 1);
    assert!(app.source_viewport.has_focus(&ctx));
    let output = frame(
        &mut app,
        &ctx,
        vec![
            key(Key::Home, Modifiers::NONE),
            key(Key::End, Modifiers::SHIFT),
            Event::Copy,
        ],
    );
    assert_eq!(output.platform_output.copied_text, "fn alpha() {");
}

#[test]
fn first_file_modal_keeps_canvas_focus_while_loading_without_stealing_later_find_focus() {
    let (_dir, mut app) = opened_repository();
    let ctx = Context::default();
    frame(&mut app, &ctx, vec![key(Key::K, Modifiers::CTRL)]);
    frame(&mut app, &ctx, vec![Event::Text("a.rs".into())]);
    frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
    assert!(app.source_viewport.has_focus(&ctx));
    settle(&mut app);
    frame(&mut app, &ctx, vec![]);
    assert!(app.source_viewport.has_focus(&ctx));
    let output = frame(
        &mut app,
        &ctx,
        vec![key(Key::End, Modifiers::SHIFT), Event::Copy],
    );
    assert_eq!(
        output.platform_output.copied_text,
        "fn alpha() {\n    let café = 1;\n}"
    );
    frame(&mut app, &ctx, vec![key(Key::F, Modifiers::CTRL)]);
    let query_focus = ctx.memory(|memory| memory.focused()).unwrap();
    app.open_file("b.rs");
    settle(&mut app);
    frame(&mut app, &ctx, vec![]);
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(query_focus));
    frame(&mut app, &ctx, vec![Event::Text("beta".into())]);
    assert_eq!(app.source_find.search.options.query, "beta");
}

#[test]
fn repository_reset_retires_modal_input_and_its_return_owner() {
    use eframe::App;
    let (_dir, mut app, ctx) = source_reader();
    app.source_viewport.request_focus(&ctx);
    frame(&mut app, &ctx, vec![key(Key::L, Modifiers::CTRL)]);
    let mut input = egui::RawInput {
        events: vec![key(Key::Enter, Modifiers::NONE)],
        ..Default::default()
    };
    app.raw_input_hook(&ctx, &mut input);
    assert!(input.events.is_empty());
    app.reset_repository_state();
    assert!(app.modal_focus.is_none());
    app.modal = ActiveModal::GoToLine(GoToLineModal {
        line_str: "1".into(),
    });
    frame(&mut app, &ctx, vec![]);
    assert!(matches!(app.modal, ActiveModal::GoToLine(_)));
    assert!(!app.source_viewport.has_focus(&ctx));
}

#[test]
fn view_menu_keyboard_activation_restores_source_selection_without_an_extra_click() {
    let (_dir, mut app, ctx) = source_reader();
    app.fold_store.unfold_all();
    app.jump_to_location("a.rs", 2, 0);
    let original_wrap = app.session.wrap_lines;
    open_toolbar_menu(&mut app, &ctx, "View");
    frame(
        &mut app,
        &ctx,
        vec![
            key(Key::Home, Modifiers::NONE),
            key(Key::ArrowDown, Modifiers::NONE),
            key(Key::Enter, Modifiers::NONE),
        ],
    );
    assert_eq!(app.session.wrap_lines, !original_wrap);
    assert_source_line_copy(&mut app, &ctx, (2, 0), "    let café = 1;");
}

#[test]
fn toolbar_left_reaches_pr_dialog_and_escape_returns_to_source() {
    let (_dir, mut app, ctx) = source_reader();
    open_toolbar_menu(&mut app, &ctx, "View");
    frame(
        &mut app,
        &ctx,
        vec![
            key(Key::ArrowLeft, Modifiers::NONE),
            key(Key::Home, Modifiers::NONE),
            key(Key::Enter, Modifiers::NONE),
        ],
    );
    frame(&mut app, &ctx, vec![]);
    assert!(matches!(app.modal, ActiveModal::PullRequest(_)));
    frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    frame(&mut app, &ctx, vec![]); // Replay input held during the window's sizing pass.
    assert!(matches!(app.modal, ActiveModal::None));
    assert!(app.source_viewport.has_focus(&ctx));
    assert!(app.pending_pr.is_none());
}

#[test]
fn focused_toolbar_menu_keeps_global_definition_shortcuts_available() {
    let (_dir, mut app) = reader_at_indexed_call();
    let ctx = Context::default();
    open_toolbar_menu(&mut app, &ctx, "View");
    assert!(crate::ui::context_menu::is_open(&ctx));
    frame(
        &mut app,
        &ctx,
        vec![key(Key::J, Modifiers::CTRL | Modifiers::ALT)],
    );
    settle(&mut app);
    frame(&mut app, &ctx, vec![]);
    assert!(!crate::ui::context_menu::is_open(&ctx));
    assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
    assert_eq!((app.caret_line, app.caret_col), (1, 3));
    assert!(app.source_viewport.has_focus(&ctx));
}

#[test]
fn source_commands_preserve_coalesced_bookmark_move_and_copy_order() {
    for scale in [1.0, 1.5, 2.0] {
        for focused in [false, true] {
            let (_dir, mut app) = reader_with_source("// é🦀漢字 SAME_LINE\r\n");
            app.jump_to_location("a.rs", 1, 3);
            settle(&mut app);
            let ctx = Context::default();
            ctx.set_pixels_per_point(scale);
            frame(&mut app, &ctx, vec![]);
            if focused {
                app.source_viewport.request_focus(&ctx);
            } else if let Some(id) = ctx.memory(|memory| memory.focused()) {
                ctx.memory_mut(|memory| memory.surrender_focus(id));
            }
            frame(
                &mut app,
                &ctx,
                vec![
                    key(Key::D, Modifiers::CTRL),
                    key(Key::ArrowRight, Modifiers::NONE),
                    key(Key::D, Modifiers::CTRL),
                    key(Key::End, Modifiers::SHIFT),
                    Event::Copy,
                ],
            );
            assert_eq!(app.bookmarks.bookmarks.len(), 1);
            frame(&mut app, &ctx, vec![]);
            assert_eq!(
                app.bookmarks
                    .bookmarks
                    .iter()
                    .map(|entry| entry.byte_offset)
                    .collect::<Vec<_>>(),
                vec![3, 5],
                "scale={scale}, focused={focused}"
            );
            let output = frame(&mut app, &ctx, vec![]);
            assert_eq!(output.platform_output.copied_text, "🦀漢字 SAME_LINE");
            assert!(
                !app.current_bookmarked(),
                "selection ends beyond both saved columns"
            );
        }
    }
}

fn source_at_indexed_gap() -> (tempfile::TempDir, RadiologyApp, Context) {
    let (dir, mut app) = reader_at_indexed_call();
    let ctx = Context::default();
    app.caret_line = 2;
    app.caret_col = 3;
    frame(&mut app, &ctx, vec![]);
    app.source_viewport.request_focus(&ctx);
    (dir, app, ctx)
}

#[test]
fn source_commands_resolve_references_after_preceding_movement() {
    let (_dir, mut app, ctx) = source_at_indexed_gap();
    frame(
        &mut app,
        &ctx,
        vec![
            key(Key::ArrowRight, Modifiers::NONE),
            key(Key::R, Modifiers::CTRL | Modifiers::ALT),
        ],
    );
    assert_eq!((app.caret_line, app.caret_col), (2, 4));
    assert_eq!(app.reference_context.target().unwrap().name, "alpha");
    assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
}

#[test]
fn source_command_navigation_retires_the_remaining_source_batch() {
    let (_dir, mut app, ctx) = source_at_indexed_gap();
    frame(
        &mut app,
        &ctx,
        vec![
            key(Key::ArrowRight, Modifiers::NONE),
            key(Key::J, Modifiers::CTRL | Modifiers::ALT),
            key(Key::D, Modifiers::CTRL),
        ],
    );
    settle(&mut app);
    frame(&mut app, &ctx, vec![]);
    assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
    assert_eq!((app.caret_line, app.caret_col), (1, 3));
    assert!(
        app.bookmarks.bookmarks.is_empty(),
        "navigation must retire later source commands"
    );
}

#[test]
fn escape_cancels_a_bookmark_drag_without_navigation_or_reordering() {
    use crate::ui::test_support::{click, text_center};
    let (_dir, mut app, ctx) = source_reader();
    let buffer = app.open_buffer.clone().unwrap();
    app.bookmarks.bookmarks = (0..30)
        .map(|index| {
            crate::bookmarks::bookmark::SourceBookmark::new(&buffer, 0, format!("drag-{index:02}"))
        })
        .collect();
    let original = app.bookmarks.bookmarks.clone();
    let history = format!("{:?}", app.history);
    frame(&mut app, &ctx, vec![]);
    let output = frame(&mut app, &ctx, vec![]);
    let from = text_center(&output, "drag-00");
    let to = text_center(&output, "drag-01");
    let mut press = click(from);
    press.pop();
    frame(&mut app, &ctx, press);
    frame(&mut app, &ctx, vec![Event::PointerMoved(to)]);
    assert!(crate::ui::bookmark_list::BookmarkList::has_focus(&ctx));
    assert!(egui::DragAndDrop::has_any_payload(&ctx));
    frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    assert!(!egui::DragAndDrop::has_any_payload(&ctx));
    assert!(app.bookmark_list.selected.is_none());
    frame(&mut app, &ctx, vec![click(to).pop().unwrap()]);
    assert_eq!(app.bookmarks.bookmarks, original);
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    assert_eq!(format!("{:?}", app.history), history);
}

#[test]
fn bookmark_empty_states_follow_repository_source_and_last_entry_transitions() {
    use crate::ui::test_support::{text_center, text_shapes};
    for scale in [1.0, 1.5, 2.0] {
        let (dir, mut app) = reader();
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        frame(&mut app, &ctx, vec![]);
        let output = frame(&mut app, &ctx, vec![]);
        text_center(&output, "No Repository");
        assert!(!text_shapes(&output).any(|text| text.galley.job.text == "Checkout"));
        assert!(!app.command_enabled(&ToolbarAction::ToggleBookmark));
        frame(&mut app, &ctx, vec![key(Key::D, Modifiers::CTRL)]);
        assert!(app.bookmarks.bookmarks.is_empty());
        open_empty_repository(&mut app, &dir.path().join("empty"));
        assert!(app.open_buffer.is_none());
        let output = frame(&mut app, &ctx, vec![]);
        text_center(&output, "Checkout");
        text_center(&output, "No Bookmarks");
        assert!(!app.command_enabled(&ToolbarAction::ToggleBookmark));
        let populated = dir.path().join("populated");
        repository(&populated);
        app.open_repository(&populated);
        settle(&mut app);
        let output = frame(&mut app, &ctx, vec![]);
        text_center(&output, "No Bookmarks");
        assert!(app.command_enabled(&ToolbarAction::ToggleBookmark));
        frame(&mut app, &ctx, vec![key(Key::D, Modifiers::CTRL)]);
        let output = frame(&mut app, &ctx, vec![]);
        assert_eq!(app.bookmarks.bookmarks.len(), 1);
        assert!(!text_shapes(&output).any(|text| text.galley.job.text == "No Bookmarks"));
        frame(&mut app, &ctx, vec![key(Key::D, Modifiers::CTRL)]);
        let output = frame(&mut app, &ctx, vec![]);
        text_center(&output, "No Bookmarks");
        assert!(app.bookmarks.bookmarks.is_empty());
    }
}
