//! Reading anchors remain independent of exact-caret history.
use super::focus::{frame, key, settle_frame};
use super::*;
use egui::{Key, Modifiers};

fn reopen_via(app: &mut RadiologyApp, ctx: &Context, other: &str, path: &str) {
    app.open_file(other);
    settle_frame(app, ctx);
    app.open_file(path);
    settle_frame(app, ctx);
}

#[test]
fn implicit_file_reopens_restore_reported_reading_positions_without_replacing_exact_back() {
    for scale in [1.0, 2.0] {
        let text: String = (1..=300)
            .map(|n| format!("// row {n:03} café 🦀 reading anchor\n"))
            .collect();
        let (owner, mut app) = isolated_source("a.rs", &text);
        let root = owner.path().join("checkout");
        std::fs::write(root.join("b.rs"), &text).unwrap();
        app.open_file("a.rs");
        settle(&mut app);
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        app.jump_to_location("a.rs", 12, 0);
        settle(&mut app);
        app.source_viewport.request_focus(&ctx);
        frame(&mut app, &ctx, vec![]);
        frame(&mut app, &ctx, vec![key(Key::ArrowRight, Modifiers::NONE)]);
        let source = app.open_buffer.clone().unwrap();
        let caret = source.byte_offset(app.caret_line, app.caret_col);
        reopen_via(&mut app, &ctx, "b.rs", "a.rs");
        assert_eq!(
            source.byte_offset(app.caret_line, app.caret_col),
            caret,
            "caret position is the latest report before scrolling"
        );
        // A passive scroll replaces the reading anchor, without moving the caret.
        app.scroll_to_line = None;
        let top = source.byte_offset(151, 0);
        app.source_viewport.restore_top(&source, top);
        frame(&mut app, &ctx, vec![]);
        assert_eq!(app.source_viewport.top_byte(&source), Some(top));
        assert_eq!(source.byte_offset(app.caret_line, app.caret_col), caret);
        app.navigate(SourceLocation::new("b.rs", 0), true);
        settle_frame(&mut app, &ctx);
        app.execute_command(&ctx, ToolbarAction::NavigateBack);
        settle_frame(&mut app, &ctx);
        assert_eq!(
            source.byte_offset(app.caret_line, app.caret_col),
            caret,
            "Back restores the departure caret rather than the passive viewport anchor"
        );
        app.scroll_to_line = None;
        app.source_viewport.restore_top(&source, top);
        frame(&mut app, &ctx, vec![]);
        for _ in 0..3 {
            reopen_via(&mut app, &ctx, "b.rs", "a.rs");
            assert_eq!(source.byte_offset(app.caret_line, app.caret_col), top);
            assert_eq!(
                app.source_viewport.top_byte(&source),
                Some(top),
                "restoration must not drift by one line on each return"
            );
        }
        app.source_viewport.restore_top(&source, 0);
        frame(&mut app, &ctx, vec![]);
        assert_eq!(app.source_viewport.top_byte(&source), Some(0));
        assert_eq!(source.byte_offset(app.caret_line, app.caret_col), top);
        assert_eq!(std::fs::read_to_string(root.join("a.rs")).unwrap(), text);
    }
}

#[test]
fn failed_canceled_changed_and_other_repository_sources_cannot_reuse_a_stale_anchor() {
    let text = "// café reading\n".repeat(160);
    let (owner, mut app) = isolated_source("a.rs", &text);
    let root = owner.path().join("checkout");
    std::fs::write(root.join("b.rs"), "// other\n").unwrap();
    std::fs::write(root.join("binary.dat"), [0, 1, 2]).unwrap();
    app.open_file("a.rs");
    settle(&mut app);
    let ctx = Context::default();
    app.jump_to_location("a.rs", 60, 0);
    settle_frame(&mut app, &ctx);
    let source = app.open_buffer.clone().unwrap();
    let anchor = source.byte_offset(60, 0);
    app.open_file("binary.dat");
    settle(&mut app);
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &source));
    app.open_file("b.rs");
    settle_frame(&mut app, &ctx);
    app.open_file("a.rs");
    app.cancel_source_load();
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
    app.open_file("a.rs");
    settle_frame(&mut app, &ctx);
    assert_eq!(source.byte_offset(app.caret_line, app.caret_col), anchor);
    app.open_file("b.rs");
    settle(&mut app);
    std::fs::write(root.join("a.rs"), "// changed\n").unwrap();
    app.open_file("a.rs");
    settle_frame(&mut app, &ctx);
    assert_eq!((app.caret_line, app.caret_col), (1, 0));
    app.jump_to_location("b.rs", 1, 5);
    settle_frame(&mut app, &ctx);
    let next = owner.path().join("other-checkout");
    std::fs::create_dir(&next).unwrap();
    std::fs::write(next.join("b.rs"), "// other\n").unwrap();
    app.open_repository(&next);
    settle(&mut app);
    app.open_file("b.rs");
    settle_frame(&mut app, &ctx);
    assert_eq!((app.caret_line, app.caret_col), (1, 0));
}

#[test]
fn pr_refresh_seeds_the_mapped_viewport_anchor_and_retires_other_snapshot_positions() {
    use super::review_refresh::{advance, open_review};
    let text = "// café reading\n".repeat(160);
    let after = format!("// head\n{text}");
    let (owner, root, first) =
        crate::pull_request::review::tests::fixture_with_sources(&text, &after);
    let (_settings, mut app) = open_review(&root, first.clone(), ReviewRevision::After);
    let ctx = Context::default();
    app.open_file("added.rs");
    settle(&mut app);
    app.jump_to_location("added.rs", 1, 1);
    settle_frame(&mut app, &ctx);
    app.open_file("a file.rs");
    settle(&mut app);
    let caret = app.open_buffer.as_ref().unwrap().byte_offset(100, 1);
    app.jump_to_location("a file.rs", 100, caret);
    settle_frame(&mut app, &ctx);
    let source = app.open_buffer.clone().unwrap();
    app.scroll_to_line = None;
    app.source_viewport
        .restore_top(&source, source.byte_offset(30, 0));
    frame(&mut app, &ctx, vec![]);
    let fresh = advance(&root, owner.path(), &first, |root| {
        std::fs::write(root.join("a file.rs"), format!("// refreshed\n{after}")).unwrap();
    });
    app.accept_review(fresh);
    settle_frame(&mut app, &ctx);
    assert_eq!(app.caret_line, 101);
    let fresh_source = app.open_buffer.clone().unwrap();
    assert_eq!(
        app.source_viewport.top_byte(&fresh_source),
        Some(fresh_source.byte_offset(31, 0))
    );
    app.open_file("added.rs");
    settle_frame(&mut app, &ctx);
    assert_eq!((app.caret_line, app.caret_col), (1, 0));
    app.open_file("a file.rs");
    settle_frame(&mut app, &ctx);
    assert_eq!((app.caret_line, app.caret_col), (31, 0));
    assert_eq!(
        app.source_viewport.top_byte(&fresh_source),
        Some(fresh_source.byte_offset(31, 0))
    );
}

#[test]
fn wrapped_reading_positions_restore_the_visible_visual_row_inside_a_source_line() {
    for (scale, requested) in [(1.0, 1000), (2.0, 1000), (1.0, 5000), (2.0, 5000)] {
        let text = format!("// {}\n{}", "café 🦀 ".repeat(900), "// tail\n".repeat(120));
        let (owner, mut app) = isolated_source("a.rs", &text);
        let root = owner.path().join("checkout");
        std::fs::write(root.join("b.rs"), "// other\n").unwrap();
        app.open_file("a.rs");
        settle(&mut app);
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        app.execute_command(&ctx, ToolbarAction::ToggleWrapLines);
        frame(&mut app, &ctx, vec![]);
        let source = app.open_buffer.clone().unwrap();
        let inside = source.clamp_to_char_boundary(requested);
        app.scroll_to_line = None;
        app.source_viewport.restore_top(&source, inside);
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![]);
        }
        let top = app.source_viewport.top_byte(&source).unwrap();
        assert!(
            top > 0 && top <= inside,
            "the anchor must identify the visible wrapped row rather than the whole source line"
        );
        assert_eq!(app.caret_line, 1);
        app.open_file("b.rs");
        settle_frame(&mut app, &ctx);
        app.open_file("a.rs");
        settle(&mut app);
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![]);
        }
        assert_eq!(source.byte_offset(app.caret_line, app.caret_col), top);
        assert_eq!(app.source_viewport.top_byte(&source), Some(top));
    }
}

#[test]
fn committed_structural_stops_survive_caret_motion_and_keep_exact_departures() {
    for scale in [1.0, 2.0] {
        let text = "// origin\nfn café() {\n work();\n}\n";
        let (owner, mut app) = isolated_source("a.rs", text);
        std::fs::write(owner.path().join("checkout/b.rs"), "fn other() {}\n").unwrap();
        app.open_file("a.rs");
        settle(&mut app);
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        let anchor = SourceLocation::new("a.rs", text.find("café").unwrap());
        app.navigate(anchor.clone(), true);
        app.source_viewport.request_focus(&ctx);
        frame(&mut app, &ctx, vec![]);
        frame(&mut app, &ctx, vec![key(Key::ArrowRight, Modifiers::NONE)]);
        let buffer = app.open_buffer.clone().unwrap();
        let departure = buffer.location_at_byte(buffer.byte_offset(app.caret_line, app.caret_col));
        assert_ne!(departure, anchor);
        assert_eq!(
            app.history.current(),
            Some(&anchor),
            "caret motion must not erase a committed structural stop"
        );
        let history = format!("{:?}", app.history);
        app.navigate(SourceLocation::new("missing.rs", 0), true);
        settle_frame(&mut app, &ctx);
        assert!(app.error.is_some());
        assert_eq!(format!("{:?}", app.history), history);
        app.navigate(SourceLocation::new("b.rs", 3), true);
        app.cancel_source_load();
        assert_eq!(format!("{:?}", app.history), history);
        app.navigate(SourceLocation::new("b.rs", 3), true);
        settle_frame(&mut app, &ctx);
        app.execute_command(&ctx, ToolbarAction::NavigateBack);
        settle_frame(&mut app, &ctx);
        assert_eq!(app.history.current(), Some(&departure));
        app.execute_command(&ctx, ToolbarAction::NavigateBack);
        settle_frame(&mut app, &ctx);
        assert_eq!(app.history.current(), Some(&anchor));
        app.execute_command(&ctx, ToolbarAction::NavigateForward);
        settle_frame(&mut app, &ctx);
        assert_eq!(app.history.current(), Some(&departure));
        app.execute_command(&ctx, ToolbarAction::NavigateForward);
        settle_frame(&mut app, &ctx);
        assert_eq!(app.history.current(), Some(&SourceLocation::new("b.rs", 3)));
        assert_eq!(buffer.text, text);
    }
}
