//! Assertion-level publication and activation contracts from the Swift app model.
use super::{
    focus::{frame, key, settle_frame},
    *,
};
use eframe::App;
use egui::{Event, Key, Modifiers};

#[test]
fn latest_navigation_restores_reported_positions_and_retires_another_pending_file() {
    for scale in [1.0, 2.0] {
        let text = "first\nsecond\nthird";
        let (owner, mut app) = isolated_source("A.swift", text);
        std::fs::write(owner.path().join("checkout/B.swift"), "other").unwrap();
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        app.source_viewport.request_focus(&ctx);
        frame(&mut app, &ctx, vec![]);
        frame(
            &mut app,
            &ctx,
            vec![
                key(Key::ArrowDown, Modifiers::NONE),
                key(Key::ArrowDown, Modifiers::NONE),
            ],
        );
        assert_eq!((app.caret_line, app.caret_col), (3, 0));
        app.open_file("B.swift");
        settle_frame(&mut app, &ctx);
        app.execute_command(&ctx, ToolbarAction::NavigateBack);
        settle_frame(&mut app, &ctx);
        let accepted = app.open_buffer.clone().unwrap();
        assert_eq!(accepted.path, "A.swift");
        assert_eq!(accepted.byte_offset(app.caret_line, app.caret_col), 13);
        assert_eq!(app.caret_line, 3);
        app.open_file("B.swift");
        assert!(app.is_source_loading());
        app.navigate(SourceLocation::new("A.swift", 6), true);
        assert!(!app.is_source_loading());
        assert!(app.pending_navigation.is_none());
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![]);
        }
        assert!(Arc::ptr_eq(&accepted, app.open_buffer.as_ref().unwrap()));
        assert_eq!(accepted.byte_offset(app.caret_line, app.caret_col), 6);
        assert_eq!(app.caret_line, 2);
        assert_eq!(
            app.history.current(),
            Some(&SourceLocation::new("A.swift", 6))
        );
        assert_eq!(accepted.text, text);
    }
}

#[test]
fn superseded_swift_preparation_and_language_changes_cannot_reuse_derived_facts() {
    for scale in [1.0, 2.0] {
        let (owner, mut app) = isolated_source("Scan.swift", "struct Scan {}\n");
        let root = owner.path().join("checkout");
        let old: String = (0..2000)
            .map(|n| format!("func old{n}() {{ print({n}) }}\n"))
            .collect();
        for (path, text) in [
            ("Old.swift", old.as_str()),
            ("Current.swift", "struct Current {}\n"),
            ("main.rs", "struct T;\ntrait Action { fn run(&self); }\nimpl Action for T { fn run(&self) {} }\nfn main() { println!(\"hi\"); }\n"),
            ("NOTES.md", "# notes\n"),
        ] { std::fs::write(root.join(path), text).unwrap(); }
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        app.open_file("Old.swift");
        // Hold a genuinely completed parse at the existing publication boundary.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let retired = loop {
            if let Some(result) = app.source_loader.poll() {
                break result.unwrap();
            }
            assert!(
                std::time::Instant::now() < deadline,
                "old Swift preparation did not finish"
            );
            std::thread::sleep(std::time::Duration::from_millis(1));
        };
        assert!(retired
            .syntax
            .as_ref()
            .unwrap()
            .symbols
            .iter()
            .any(|symbol| symbol.name == "old1999"));
        app.open_file("Old.swift");
        app.open_file("Current.swift");
        settle_frame(&mut app, &ctx);
        let current = app.open_buffer.clone().unwrap();
        let colors = app.native_highlights.clone();
        let folds = format!("{:?}", app.fold_ranges);
        assert_eq!(current.path, "Current.swift");
        assert_eq!(app.outline_symbols[0].name, "Current");
        assert!(colors.matches(&current));
        app.publish_source(retired);
        frame(&mut app, &ctx, vec![]);
        assert!(Arc::ptr_eq(&current, app.open_buffer.as_ref().unwrap()));
        assert!(Arc::ptr_eq(&colors, &app.native_highlights));
        assert_eq!(format!("{:?}", app.fold_ranges), folds);
        assert_eq!(app.outline_symbols.len(), 1);
        assert_eq!(app.outline_symbols[0].name, "Current");
        app.open_file("main.rs");
        settle_frame(&mut app, &ctx);
        let rust = app.open_buffer.clone().unwrap();
        assert_eq!(rust.path, "main.rs");
        assert!(app
            .outline_symbols
            .iter()
            .any(|symbol| symbol.name == "main"));
        assert!(!app
            .outline_symbols
            .iter()
            .any(|symbol| symbol.name == "Current"));
        assert_eq!(app.rust_trait_impls.implementations.len(), 1);
        assert!(app.native_highlights.matches(&rust));
        assert!(!colors.matches(&rust));
        app.syntax_error = Some("retired structure failure".into());
        app.fold_error = Some("retired fold failure".into());
        app.open_file("NOTES.md");
        settle_frame(&mut app, &ctx);
        let plain = app.open_buffer.clone().unwrap();
        assert_eq!(plain.path, "NOTES.md");
        assert_eq!(plain.text, "# notes\n");
        assert!(app.syntax_error.is_none() && app.fold_error.is_none());
        assert!(app.outline_symbols.is_empty() && app.fold_ranges.is_empty());
        assert!(app.rust_trait_impls.implementations.is_empty());
        assert!(!app.native_highlights.matches(&plain));
        let output = frame(&mut app, &ctx, vec![]);
        assert!(crate::ui::test_support::text_shapes(&output)
            .any(|text| text.galley.job.text == "No Outline"));
        app.source_viewport.request_focus(&ctx);
        let copied = frame(
            &mut app,
            &ctx,
            vec![key(Key::A, Modifiers::CTRL), Event::Copy],
        );
        assert_eq!(copied.platform_output.copied_text, plain.text);
        assert_eq!(
            std::fs::read_to_string(root.join("Old.swift")).unwrap(),
            old
        );
    }
}

#[test]
fn repeated_pointer_outline_activation_restarts_feedback_and_jump_without_history_duplicates() {
    use crate::ui::outline_list::OutlineAction;
    for scale in [1.0, 2.0] {
        let text = "fn target() {}\n";
        let (_owner, mut app) = isolated_source("A.rs", text);
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        let render = |app: &mut RadiologyApp, now, events| {
            let mut raw = egui::RawInput {
                time: Some(now),
                events,
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1100.0, 800.0),
                )),
                ..Default::default()
            };
            app.raw_input_hook(&ctx, &mut raw);
            ctx.run(raw, |ctx| app.render(ctx))
        };
        let accent = app.theme.accent;
        let bands = |output: &egui::FullOutput, alpha| {
            output.shapes.iter().filter(|shape| matches!(&shape.shape,
            egui::Shape::Rect(rect) if rect.fill == accent.gamma_multiply(alpha) && rect.rect.width() > 200.0)).count()
        };
        render(&mut app, 1.0, vec![]);
        let target = first_outline_target(&mut app);
        app.handle_outline_action(OutlineAction::Activate(target.clone()));
        let history = format!("{:?}", app.history);
        assert_eq!(bands(&render(&mut app, 1000.0, vec![]), 0.18), 1);
        assert_eq!(bands(&render(&mut app, 1001.005, vec![]), 0.09), 1);
        let expired = render(&mut app, 1001.2, vec![]);
        assert_eq!(bands(&expired, 0.18), 0);
        app.source_viewport.request_focus(&ctx);
        app.source_viewport.select(0..2);
        assert_eq!(
            render(&mut app, 1001.3, vec![Event::Copy])
                .platform_output
                .copied_text,
            "fn"
        );
        app.handle_outline_action(OutlineAction::Activate(target.clone()));
        let restarted = render(&mut app, 2000.0, vec![Event::Copy]);
        assert!(
            restarted.platform_output.copied_text.is_empty(),
            "repeated activation must place the jump and clear the old selection"
        );
        assert_eq!(bands(&restarted, 0.18), 1);
        assert_eq!(format!("{:?}", app.history), history);
        assert_eq!(bands(&render(&mut app, 2001.005, vec![]), 0.09), 1);
        app.handle_outline_action(OutlineAction::Activate(target.clone()));
        assert_eq!(bands(&render(&mut app, 3000.0, vec![]), 0.18), 1);
        assert_eq!(format!("{:?}", app.history), history);
        // Swift's Return commit is intentionally different from pointer activation.
        app.handle_outline_action(OutlineAction::Commit(target));
        assert_eq!(bands(&render(&mut app, 4000.0, vec![]), 0.18), 0);
        assert_eq!(app.open_buffer.as_ref().unwrap().text, text);
    }
}
