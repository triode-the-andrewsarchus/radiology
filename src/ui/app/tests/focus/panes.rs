//! App-owned pane persistence, input retirement and recovery from empty layouts.
use super::*;
use crate::session::inspector::{InspectorPane, InspectorPanes};
use crate::ui::test_support::{click, text_center, text_shapes};

fn has(output: &egui::FullOutput, label: &str) -> bool {
    text_shapes(output).any(|text| text.galley.job.text == label)
}

pub(super) fn set_mask(app: &mut RadiologyApp, ctx: &Context, mask: usize) {
    for (bit, pane) in InspectorPane::ALL.into_iter().enumerate() {
        if app.session.inspector_panes.enabled(pane) != (mask & (1 << bit) != 0) {
            app.execute_command(ctx, ToolbarAction::ToggleInspectorPane(pane));
        }
    }
}

#[test]
fn inspector_pane_matrix_preserves_source_and_context_while_reclaiming_regions() {
    for scale in [1.0, 2.0] {
        let (_owner, mut app, ctx) = super::references::caller_reader();
        ctx.set_pixels_per_point(scale);
        frame(&mut app, &ctx, vec![]);
        app.handle_reference_action(crate::ui::reference_list::ReferenceAction::Select(1));
        app.toggle_current_bookmark();
        let bookmark = app.bookmarks.bookmarks[0].id;
        app.bookmark_list.selected = Some(bookmark);
        let source = app.open_buffer.clone().unwrap();
        let history = format!("{:?}", app.history);
        let caret = (app.caret_line, app.caret_col);
        let target = app.reference_context.explicit().cloned();
        let references = app.reference_context.results.hits.clone();
        for mask in 0..16 {
            set_mask(&mut app, &ctx, mask);
            frame(&mut app, &ctx, vec![]);
            let output = frame(&mut app, &ctx, vec![]);
            let panes = app.session.inspector_panes;
            assert!(has(&output, "Panes"));
            assert_eq!(has(&output, "Bookmarks"), panes.bookmarks);
            assert_eq!(
                text_shapes(&output).any(|text| text.galley.job.text.starts_with("References")),
                panes.references
            );
            assert_eq!(
                has(&output, "Outline") || has(&output, "Definition"),
                panes.definition,
                "mask {mask} scale {scale}: {:?}",
                text_shapes(&output)
                    .filter(|t| t.pos.x > 780.0)
                    .map(|t| (&t.galley.job.text, t.pos))
                    .collect::<Vec<_>>()
            );
            if panes.visible(false).is_empty() {
                assert!(has(&output, "Show All Panes"));
                assert!(has(
                    &output,
                    if panes.implementations {
                        "No Context Panes Available"
                    } else {
                        "No Panes Enabled"
                    }
                ));
            }
            if panes.visible(false) == [InspectorPane::Bookmarks] {
                let rect = ctx
                    .read_response(crate::ui::bookmark_list::BookmarkList::focus_id())
                    .unwrap()
                    .rect;
                assert!(
                    rect.height() > 500.0,
                    "single bookmark pane did not fill: {rect:?}"
                );
            }
            assert!(Arc::ptr_eq(&source, app.open_buffer.as_ref().unwrap()));
            assert_eq!((app.caret_line, app.caret_col), caret);
            assert_eq!(format!("{:?}", app.history), history);
            assert_eq!(app.reference_context.selected, Some(1));
            assert_eq!(app.reference_context.explicit(), target.as_ref());
            assert_eq!(app.reference_context.results.hits.len(), references.len());
            assert!(app.reference_context.disclosures[&1].callers().is_some());
            assert_eq!(app.bookmark_list.selected, Some(bookmark));
            assert_eq!(app.bookmarks.bookmarks[0].id, bookmark);
        }
        app.execute_command(&ctx, ToolbarAction::ShowAllInspectorPanes);
        assert!(app.session.inspector_panes.all_enabled());
        let persisted =
            SessionStore::load_from(app.session.config_directory().unwrap().join("session.json"));
        assert!(persisted.inspector_panes.all_enabled());
    }
}

#[test]
fn inspector_pane_menus_recover_empty_layout_and_reopen_from_view_without_source() {
    let (_owner, mut app) = reader();
    let ctx = Context::default();
    let output = frame(&mut app, &ctx, vec![]);
    let panes = text_center(&output, "Panes");
    frame(&mut app, &ctx, click(panes));
    let output = frame(&mut app, &ctx, vec![]);
    assert!(has(&output, "✓ References"));
    frame(
        &mut app,
        &ctx,
        vec![
            key(Key::Home, Modifiers::NONE),
            key(Key::Enter, Modifiers::NONE),
        ],
    );
    assert!(!app.session.inspector_panes.references);
    for pane in [
        InspectorPane::Definition,
        InspectorPane::Implementations,
        InspectorPane::Bookmarks,
    ] {
        app.execute_command(&ctx, ToolbarAction::ToggleInspectorPane(pane));
    }
    let output = frame(&mut app, &ctx, vec![]);
    assert!(has(&output, "No Panes Enabled"));
    let restore = text_center(&output, "Show All Panes");
    frame(&mut app, &ctx, click(restore));
    frame(&mut app, &ctx, vec![]);
    assert!(app.session.inspector_panes.all_enabled());
    app.execute_command(
        &ctx,
        ToolbarAction::ToggleInspectorPane(InspectorPane::Bookmarks),
    );
    app.show_inspector = false;
    let output = frame(&mut app, &ctx, vec![]);
    let view = text_center(&output, "View");
    frame(&mut app, &ctx, click(view));
    let output = frame(&mut app, &ctx, vec![]);
    let submenu = text_center(&output, "Context Panes");
    frame(&mut app, &ctx, click(submenu));
    let output = frame(&mut app, &ctx, vec![]);
    let bookmarks = text_center(&output, "Bookmarks");
    frame(&mut app, &ctx, click(bookmarks));
    frame(&mut app, &ctx, vec![]);
    assert!(app.show_inspector && app.session.inspector_panes.bookmarks);
    assert!(app.open_buffer.is_none());
}

#[test]
fn hiding_focused_reference_pane_retires_its_keys_but_preserves_selection() {
    let (owner, mut app, ctx) = super::references::caller_reader();
    app.handle_reference_action(crate::ui::reference_list::ReferenceAction::Select(1));
    ctx.memory_mut(|memory| {
        memory.request_focus(crate::ui::reference_list::ReferenceViewport::focus_id())
    });
    app.execute_command(
        &ctx,
        ToolbarAction::ToggleInspectorPane(InspectorPane::References),
    );
    assert!(app.source_viewport.has_focus(&ctx));
    frame(&mut app, &ctx, vec![key(Key::ArrowDown, Modifiers::NONE)]);
    assert_eq!(app.reference_context.selected, Some(1));
    app.execute_command(
        &ctx,
        ToolbarAction::ToggleInspectorPane(InspectorPane::References),
    );
    assert!(app.session.inspector_panes.references);
    assert_eq!(app.reference_context.selected, Some(1));
    app.execute_command(
        &ctx,
        ToolbarAction::ToggleInspectorPane(InspectorPane::Definition),
    );
    let saved = app.session.inspector_panes;
    let other = owner.path().join("another");
    std::fs::create_dir(&other).unwrap();
    app.open_repository(&other);
    settle(&mut app);
    assert_eq!(app.session.inspector_panes, saved);
    assert!(!saved.definition);
    assert_ne!(InspectorPanes::default(), saved);
}

#[test]
fn inspector_bookmark_scroll_survives_split_and_single_pane_layouts() {
    let (_owner, mut app, ctx) = scaled_reader("fn alpha() {}\n", 1.0);
    let source = app.open_buffer.clone().unwrap();
    app.bookmarks.bookmarks = (0..120)
        .map(|index| {
            crate::bookmarks::bookmark::SourceBookmark::new(&source, 0, format!("saved-{index:03}"))
        })
        .collect();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    let region = ctx
        .read_response(crate::ui::bookmark_list::BookmarkList::focus_id())
        .unwrap()
        .rect;
    frame(
        &mut app,
        &ctx,
        vec![
            Event::PointerMoved(region.center()),
            Event::Scroll(egui::vec2(0.0, -1000.0)),
        ],
    );
    for _ in 0..24 {
        frame(&mut app, &ctx, vec![]);
    }
    let first = |output: &egui::FullOutput, region: egui::Rect| {
        text_shapes(output)
            .filter(|text| {
                text.galley.job.text.starts_with("saved-")
                    && text.pos.y >= region.top()
                    && text.pos.y < region.bottom()
            })
            .min_by(|a, b| a.pos.y.total_cmp(&b.pos.y))
            .map(|text| text.galley.job.text.clone())
            .unwrap()
    };
    let output = frame(&mut app, &ctx, vec![]);
    let before = first(&output, region);
    assert_ne!(before, "saved-000");
    app.execute_command(
        &ctx,
        ToolbarAction::ToggleInspectorPane(InspectorPane::References),
    );
    app.execute_command(
        &ctx,
        ToolbarAction::ToggleInspectorPane(InspectorPane::Definition),
    );
    frame(&mut app, &ctx, vec![]);
    let output = frame(&mut app, &ctx, vec![]);
    let larger = ctx
        .read_response(crate::ui::bookmark_list::BookmarkList::focus_id())
        .unwrap()
        .rect;
    assert!(larger.height() > region.height() * 2.0);
    assert_eq!(first(&output, larger), before);
    app.execute_command(&ctx, ToolbarAction::ShowAllInspectorPanes);
    frame(&mut app, &ctx, vec![]);
    let output = frame(&mut app, &ctx, vec![]);
    let restored = ctx
        .read_response(crate::ui::bookmark_list::BookmarkList::focus_id())
        .unwrap()
        .rect;
    assert_eq!(first(&output, restored), before);
}
