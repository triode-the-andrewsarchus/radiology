use super::references::{caller_reader, reference_click_and_right};
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

#[test]
fn pointer_handoff_preserves_queued_keys_ahead_of_new_input_and_retires_at_boundaries() {
    use crate::ui::reference_list::ReferenceAction;
    use eframe::App;
    for boundary in ["new-key", "window", "repository"] {
        let (dir, mut app, ctx) = caller_reader();
        app.handle_reference_action(ReferenceAction::SetExpanded(1, false));
        let events = reference_click_and_right(&mut app, &ctx, true);
        frame(&mut app, &ctx, events);
        assert!(!app.deferred_input.is_empty());
        match boundary {
            "new-key" => {
                frame(&mut app, &ctx, vec![key(Key::ArrowLeft, Modifiers::NONE)]);
            }
            "window" => {
                let mut input = egui::RawInput {
                    focused: false,
                    ..Default::default()
                };
                app.raw_input_hook(&ctx, &mut input);
            }
            "repository" => {
                app.open_repository(dir.path());
                settle(&mut app);
            }
            _ => unreachable!(),
        }
        assert!(app.deferred_input.is_empty(), "{boundary}");
        assert!(
            !app.reference_context.disclosures.contains_key(&1),
            "{boundary}"
        );
    }
}

#[test]
fn floating_editor_over_source_receives_click_then_select_all_and_text() {
    for scale in [1.0, 1.5, 2.0] {
        let (_dir, mut app, ctx) = source_reader();
        ctx.set_pixels_per_point(scale);
        let mut value = "original".to_string();
        let field = std::cell::Cell::new(egui::Rect::NOTHING);
        let source = app.open_buffer.clone().unwrap();
        let caret = (app.caret_line, app.caret_col);
        let mut overlay = |ctx: &Context| {
            let source = ctx
                .read_response(crate::ui::source_view::SourceViewport::focus_id())
                .unwrap();
            egui::Area::new(egui::Id::new("floating_editor"))
                .order(egui::Order::Foreground)
                .fixed_pos(source.rect.center())
                .show(ctx, |ui| {
                    field.set(
                        ui.add(egui::TextEdit::singleline(&mut value).id_source("field"))
                            .rect,
                    );
                });
        };
        frame_with_overlay(&mut app, &ctx, vec![], &mut overlay);
        app.source_viewport.request_focus(&ctx);
        frame_with_overlay(&mut app, &ctx, vec![], &mut overlay);
        let source_response = ctx
            .read_response(crate::ui::source_view::SourceViewport::focus_id())
            .unwrap();
        let point = field.get().center();
        assert!(source_response.rect.contains(point));
        assert_ne!(ctx.layer_id_at(point), Some(source_response.layer_id));
        let mut events = crate::ui::test_support::click(point);
        events.extend([
            key(
                Key::A,
                Modifiers {
                    command: true,
                    ..Modifiers::CTRL
                },
            ),
            Event::Text("replacement".into()),
        ]);
        frame_with_overlay(&mut app, &ctx, events, &mut overlay);
        frame_with_overlay(&mut app, &ctx, vec![], &mut overlay);
        assert_eq!(value, "replacement", "{scale}x");
        assert_eq!((app.caret_line, app.caret_col), caret);
        assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &source));
        assert!(app.deferred_input.is_empty());
    }
}
