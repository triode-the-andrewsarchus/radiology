//! Large source transactions retain text owners and retire superseded publication.
use super::focus::{frame, key};
use super::*;
use egui::{Event, Key, Modifiers};

#[test]
fn large_source_publication_preserves_modal_text_and_canceled_work_cannot_replace_the_reader() {
    let text = format!(
        "// {} LONG_END\r\nint tail(int café) {{ return café; }}\r\n",
        "é🦀漢字 ".repeat(450_000)
    );
    let many = format!(
        "{}int many_tail() {{ return 1; }}\r\n",
        format!("// row café 🦀 {}\r\n", "value ".repeat(12)).repeat(60_000)
    );
    for (shape, text) in [("long", text), ("many", many)] {
        for scale in [1.0, 2.0] {
            let (_owner, mut app) = isolated_source("a.rs", "// anchor café\r\n");
            let root = app.repository_root.clone().unwrap();
            std::fs::write(root.join("large.cpp"), &text).unwrap();
            app.open_repository(&root);
            settle(&mut app);
            let ctx = Context::default();
            ctx.set_pixels_per_point(scale);
            for import in [false, true] {
                app.open_file("a.rs");
                settle(&mut app);
                app.source_viewport.request_focus(&ctx);
                frame(&mut app, &ctx, vec![]);
                let anchor = app.open_buffer.clone().unwrap();
                app.open_file("large.cpp");
                assert!(app.is_source_loading());
                assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &anchor));
                app.execute_command(
                    &ctx,
                    if import {
                        ToolbarAction::ImportCodePath
                    } else {
                        ToolbarAction::OpenQuickFilePicker
                    },
                );
                let started = std::time::Instant::now();
                frame(&mut app, &ctx, vec![Event::Text("owner café".into())]);
                let mut frames = vec![started.elapsed().as_micros()];
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
                while app.is_source_loading() {
                    let started = std::time::Instant::now();
                    frame(&mut app, &ctx, vec![]);
                    frames.push(started.elapsed().as_micros());
                    assert!(
                        std::time::Instant::now() < deadline,
                        "source publication timed out"
                    );
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                frame(&mut app, &ctx, vec![Event::Text(" suffix".into())]);
                match &app.modal {
                    ActiveModal::QuickFilePicker(picker) => {
                        assert_eq!(picker.query, "owner café suffix")
                    }
                    ActiveModal::CodePathImport(modal) => {
                        assert_eq!(modal.text, "owner café suffix")
                    }
                    _ => panic!("source publication closed its text owner"),
                }
                assert_eq!(app.open_buffer.as_ref().unwrap().text, text);
                assert!(app
                    .native_highlights
                    .matches(app.open_buffer.as_ref().unwrap()));
                frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
                frame(&mut app, &ctx, vec![]);
                assert!(matches!(app.modal, ActiveModal::None));
                assert!(app.source_viewport.has_focus(&ctx));
                let copy = frame(
                    &mut app,
                    &ctx,
                    vec![key(Key::A, Modifiers::CTRL), Event::Copy],
                );
                assert_eq!(copy.platform_output.copied_text, text);
                let history = app.history.current().cloned();
                app.execute_command(&ctx, ToolbarAction::ToggleWrapLines);
                frame(&mut app, &ctx, vec![]);
                assert_eq!(
                    frame(&mut app, &ctx, vec![Event::Copy])
                        .platform_output
                        .copied_text,
                    text
                );
                assert_eq!(app.history.current(), history.as_ref());
                app.open_file("a.rs");
                settle(&mut app);
                let retained = app.open_buffer.clone().unwrap();
                let history = app.history.current().cloned();
                app.open_file("large.cpp");
                assert!(app.is_source_loading());
                app.navigate(SourceLocation::new("a.rs", 3), false);
                assert!(!app.is_source_loading());
                for _ in 0..4 {
                    frame(&mut app, &ctx, vec![]);
                }
                assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &retained));
                assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
                assert_eq!(app.caret_col, 3);
                assert_eq!(
                    app.history.current().unwrap().path,
                    history.as_ref().unwrap().path
                );
                assert_eq!(
                    std::fs::read(root.join("large.cpp")).unwrap(),
                    text.as_bytes()
                );
                frames.sort_unstable();
                eprintln!(
                "stress shape={shape} scale={scale} import={import} bytes={} frames={} p95_us={} max_us={}",
                text.len(),
                frames.len(),
                frames[frames.len() * 95 / 100],
                frames.last().unwrap()
            );
            }
        }
    }
}
