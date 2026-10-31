use super::references::caller_reader;
use super::*;

#[test]
fn modal_prefix_prevents_later_reference_or_caller_navigation() {
    for caller in [false, true] {
        let (_dir, mut app, ctx) = caller_reader();
        if caller {
            frame(&mut app, &ctx, vec![key(Key::ArrowRight, Modifiers::NONE)]);
        }
        let original = app.open_buffer.clone().unwrap();
        let history = app.history.current().cloned();
        frame(
            &mut app,
            &ctx,
            vec![
                key(Key::L, Modifiers::CTRL),
                key(Key::ArrowDown, Modifiers::NONE),
                key(Key::Enter, Modifiers::NONE),
            ],
        );
        frame(&mut app, &ctx, vec![]);
        settle(&mut app);
        assert!(matches!(app.modal, ActiveModal::GoToLine(_)));
        assert!(
            Arc::ptr_eq(&original, app.open_buffer.as_ref().unwrap()),
            "caller={caller}"
        );
        assert_eq!(app.history.current(), history.as_ref());
    }
}

#[test]
fn opening_file_modal_accepts_the_following_text_and_return_in_order() {
    let (_dir, mut app, ctx) = caller_reader();
    frame(
        &mut app,
        &ctx,
        vec![
            key(Key::K, Modifiers::CTRL),
            Event::Text("b.rs".into()),
            key(Key::Enter, Modifiers::NONE),
        ],
    );
    frame(&mut app, &ctx, vec![]); // Complete the modal's registration pass and replay its queued input.
    settle(&mut app);
    assert!(matches!(app.modal, ActiveModal::None));
    assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
    assert_eq!((app.caret_line, app.caret_col), (1, 0));
    app.execute_command(&ctx, ToolbarAction::NavigateBack);
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
    assert_eq!((app.caret_line, app.caret_col), (1, 4));
}

#[test]
fn find_prefix_routes_text_and_return_to_the_query_without_committing_a_reference() {
    let (_dir, mut app, ctx) = caller_reader();
    frame(
        &mut app,
        &ctx,
        vec![
            key(Key::ArrowRight, Modifiers::NONE),
            key(Key::F, Modifiers::CTRL),
            Event::Text("alpha".into()),
            key(Key::Enter, Modifiers::NONE),
        ],
    );
    settle(&mut app);
    assert_eq!(app.source_find.search.options.query, "alpha");
    assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
    assert_eq!(app.reference_context.selected, Some(1));
    assert!(app.reference_context.disclosures.contains_key(&1));
    assert!(ctx.wants_keyboard_input());
}

#[test]
fn repository_search_shortcut_focuses_and_accepts_the_query_in_its_opening_batch() {
    let (_dir, mut app, ctx) = caller_reader();
    frame(
        &mut app,
        &ctx,
        vec![
            key(Key::F, Modifiers::CTRL | Modifiers::SHIFT),
            Event::Text("needle".into()),
        ],
    );
    assert_eq!(app.sidebar_tab, SidebarTab::Search);
    assert_eq!(app.search_options.query, "needle");
    assert!(ctx.wants_keyboard_input());
}
