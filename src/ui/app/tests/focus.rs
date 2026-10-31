//! Real app frames exercise modal/canvas focus rather than a parallel test renderer.
use super::*;
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
