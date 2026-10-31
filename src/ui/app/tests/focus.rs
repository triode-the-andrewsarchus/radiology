//! Real app frames exercise modal/canvas focus rather than a parallel test renderer.
use super::*;
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
    assert!(app.source_viewport.has_focus(&ctx));
    assert_eq!((app.caret_line, app.caret_col), (2, 0));
    let output = frame(
        &mut app,
        &ctx,
        vec![key(Key::End, Modifiers::SHIFT), Event::Copy],
    );
    assert_eq!(output.platform_output.copied_text, "    let café = 1;");
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
