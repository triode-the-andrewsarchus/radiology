//! The checked-in Swift language corpus passes through source publication and real app frames.
use super::focus::{frame, key, settle_frame};
use super::*;
use egui::{Event, Key, Modifiers};

#[test]
fn swift_source_corpus_keeps_reader_bytes_folds_copy_and_history_through_recovery() {
    for scale in [1.0, 2.0] {
        let (owner, mut app) = reader();
        let root = owner.path().join("checkout");
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("anchor.txt"), "anchor café 🦀\r\n").unwrap();
        let mut cases = Vec::new();
        for &(name, source, callable) in crate::syntax::test_fixtures::SWIFT_SOURCE_FIXTURES {
            for crlf in [false, true] {
                for broken in [false, true] {
                    let path = format!("{crlf}-{broken}-{name}");
                    let mut text = source.to_owned();
                    if broken {
                        text.push_str("\n<<<\n");
                    }
                    if crlf {
                        text = text.replace('\n', "\r\n");
                    }
                    std::fs::write(root.join(&path), &text).unwrap();
                    cases.push((path, text, callable, broken));
                }
            }
        }
        app.open_repository(&root);
        settle(&mut app);
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        for (path, text, callable, broken) in cases {
            app.open_file(&path);
            settle_frame(&mut app, &ctx);
            let buffer = app.open_buffer.clone().unwrap();
            assert_eq!(buffer.text, text, "{path}/{scale}");
            assert!(
                app.syntax_error.is_none() && app.fold_error.is_none(),
                "{path}"
            );
            assert!(app.call_sites.supported, "{path}");
            assert_eq!(app.call_sites.parse_errors, broken, "{path}");
            assert!(app.native_highlights.matches(&buffer), "{path}");
            let rows = crate::syntax::outline::outline_rows(&app.outline_symbols, &buffer);
            for row in &rows {
                assert_eq!(
                    buffer.text.get(row.symbol.selection_range.clone()),
                    Some(row.symbol.name.as_str()),
                    "{path}"
                );
            }
            let (byte, body) = rows
                .iter()
                .filter(|row| row.symbol.name == callable)
                .find_map(|row| {
                    crate::syntax::folds::enclosing_function_fold(
                        row.symbol.selection_range.start,
                        &app.outline_symbols,
                        &app.fold_ranges,
                    )
                    .map(|fold| (row.symbol.selection_range.start, fold.byte_range.clone()))
                })
                .unwrap_or_else(|| {
                    panic!(
                        "missing {callable} body in {path}: {:?}",
                        app.outline_symbols
                    )
                });
            app.navigate(SourceLocation::new(&path, byte), true);
            settle(&mut app);
            app.source_viewport.request_focus(&ctx);
            frame(&mut app, &ctx, vec![]);
            assert_eq!(
                buffer.byte_offset(app.caret_line, app.caret_col),
                byte,
                "{path}"
            );
            assert!(
                !app.fold_store.is_folded(&body),
                "committed destination: {path}"
            );
            frame(&mut app, &ctx, vec![key(Key::X, Modifiers::NONE)]);
            assert!(app.fold_store.is_folded(&body), "current function: {path}");
            frame(&mut app, &ctx, vec![key(Key::X, Modifiers::NONE)]);
            assert!(!app.fold_store.is_folded(&body));
            app.source_viewport.select(byte..byte + callable.len());
            assert_eq!(
                frame(&mut app, &ctx, vec![Event::Copy])
                    .platform_output
                    .copied_text,
                callable,
                "{path}"
            );
            app.execute_command(&ctx, ToolbarAction::ToggleWrapLines);
            app.execute_command(&ctx, ToolbarAction::ToggleTheme);
            settle(&mut app);
            assert_eq!(
                frame(&mut app, &ctx, vec![Event::Copy])
                    .platform_output
                    .copied_text,
                callable,
                "recolor/wrap: {path}"
            );
            let departure = buffer.byte_offset(app.caret_line, app.caret_col);
            app.navigate(SourceLocation::new("anchor.txt", 7), true);
            settle_frame(&mut app, &ctx);
            app.execute_command(&ctx, ToolbarAction::NavigateBack);
            settle_frame(&mut app, &ctx);
            assert_eq!(app.selected_path.as_deref(), Some(path.as_str()));
            assert_eq!(
                buffer.byte_offset(app.caret_line, app.caret_col),
                departure,
                "Back: {path}"
            );
            assert_eq!(app.open_buffer.as_ref().unwrap().text, text);
            assert_eq!(std::fs::read(root.join(&path)).unwrap(), text.as_bytes());
        }
    }
}
