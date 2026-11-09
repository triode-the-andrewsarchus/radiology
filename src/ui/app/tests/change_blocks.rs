//! Real pinned change navigation across hidden tests, passive scrolling and EOF.
use super::focus::{frame, key, settle_frame};
use super::*;
use egui::{Key, Modifiers};

#[test]
fn pinned_change_keys_reveal_tests_keep_exact_departures_and_reach_only_the_eof_anchor() {
    let mut lines: Vec<String> = [
        "fn production() {",
        "    let value = 1;",
        "    consume(value);",
        "}",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    lines.extend((5..20).map(|line| format!("// stable {line}")));
    lines.extend(
        ["#[cfg(test)]", "fn helper() {", "    let value = 1;", "}"]
            .into_iter()
            .map(str::to_owned),
    );
    // More unchanged tail than Swift's fixture permits an actual passive scroll
    // in the complete app window, rather than assigning a synthetic viewport.
    lines.extend((24..=195).map(|line| format!("// stable {line}")));
    let before = lines.join("\n");
    lines[1] = "    let value = 2;".into();
    lines[21] = "    let value = 2;".into();
    lines.pop();
    let after = lines.join("\n") + "\n";
    let (_owner, root, review) = crate::pull_request::review::tests::fixture_with_tree_edits(
        &before,
        &after,
        |root| {
            std::fs::write(
                root.join("unchanged.rs"),
                "fn quiet() {\n    untouched();\n}\n",
            )
            .unwrap()
        },
        |_| {},
    );
    for scale in [1.0, 2.0] {
        let (_settings, mut app, ctx) =
            super::review_refresh::review_context(&root, review.clone(), scale);
        app.execute_command(&ctx, ToolbarAction::ToggleHideTests);
        app.source_viewport.request_focus(&ctx);
        frame(&mut app, &ctx, vec![]);
        let source = app.open_buffer.clone().unwrap();
        let test_byte = source.byte_offset(22, 0);
        assert!(app
            .test_code
            .hidden_ranges()
            .iter()
            .any(|range| range.contains(&test_byte)));
        let previous = (app.caret_line, app.caret_col);
        app.caret_line = 3;
        app.caret_col = 5;
        app.finish_source_interaction(previous);
        let expected = app.change_navigation();
        assert_eq!(
            (expected.previous, expected.next),
            (Some(source.byte_offset(2, 0)), Some(test_byte))
        );
        frame(&mut app, &ctx, vec![key(Key::ArrowRight, Modifiers::NONE)]);
        assert_eq!((app.caret_line, app.caret_col), (3, 6));
        assert_eq!(app.change_navigation(), expected);
        let previous = (app.caret_line, app.caret_col);
        app.caret_col = 5;
        app.finish_source_interaction(previous);
        app.scroll_to_line = None;
        let passive = source.byte_offset(30, 0);
        app.source_viewport.restore_top(&source, passive);
        frame(&mut app, &ctx, vec![]);
        assert_eq!(app.reading_positions.current_byte(), Some(passive));
        assert_eq!(app.change_navigation(), expected);
        frame(&mut app, &ctx, vec![key(Key::ArrowDown, Modifiers::ALT)]);
        settle_frame(&mut app, &ctx);
        assert_eq!((app.caret_line, app.caret_col), (22, 0));
        assert!(app.session.source_filters.hides_tests);
        assert!(!app
            .test_code
            .hidden_ranges()
            .iter()
            .any(|range| range.contains(&test_byte)));
        assert!(!app.fold_store.is_line_hidden(22, &app.fold_ranges));
        app.execute_command(&ctx, ToolbarAction::NavigateBack);
        settle_frame(&mut app, &ctx);
        assert_eq!((app.caret_line, app.caret_col), (3, 5));
        let previous = (app.caret_line, app.caret_col);
        app.caret_line = 2;
        app.finish_source_interaction(previous);
        assert!(app.change_navigation().previous.is_none());
        frame(&mut app, &ctx, vec![key(Key::ArrowDown, Modifiers::ALT)]);
        settle_frame(&mut app, &ctx);
        assert_eq!(app.caret_line, 22);
        frame(&mut app, &ctx, vec![key(Key::ArrowDown, Modifiers::ALT)]);
        settle_frame(&mut app, &ctx);
        assert_eq!(
            source.byte_offset(app.caret_line, app.caret_col),
            source.byte_len()
        );
        assert_eq!(app.change_navigation().previous, Some(test_byte));
        assert!(app.change_navigation().next.is_none());
        let valid = app.line_changes.take().unwrap();
        assert!(!valid
            .changed_lines(ReviewRevision::After)
            .iter()
            .any(|range| range.contains(&source.line_count())));
        assert_eq!(valid.hunks.len(), 3);
        let history = format!("{:?}", app.history);
        for unavailable in [
            None,
            Some(crate::pull_request::FileLineChanges {
                is_binary: true,
                ..valid.clone()
            }),
        ] {
            app.line_changes = unavailable;
            assert_eq!(app.change_navigation(), Default::default());
            frame(
                &mut app,
                &ctx,
                vec![
                    key(Key::ArrowDown, Modifiers::ALT),
                    key(Key::ArrowUp, Modifiers::ALT),
                ],
            );
            settle_frame(&mut app, &ctx);
            assert!(Arc::ptr_eq(&source, app.open_buffer.as_ref().unwrap()));
            assert_eq!(
                source.byte_offset(app.caret_line, app.caret_col),
                source.byte_len()
            );
            assert_eq!(format!("{:?}", app.history), history);
        }
        app.line_changes = Some(valid);
        frame(&mut app, &ctx, vec![key(Key::ArrowDown, Modifiers::ALT)]);
        assert_eq!(format!("{:?}", app.history), history);
        app.execute_command(&ctx, ToolbarAction::ToggleRevision(ReviewRevision::Before));
        settle_frame(&mut app, &ctx);
        let base = app.open_buffer.clone().unwrap();
        assert_eq!(base.text, before);
        assert_eq!(base.revision, Some(ReviewRevision::Before));
        let previous = (app.caret_line, app.caret_col);
        app.caret_line = 3;
        app.caret_col = 5;
        app.finish_source_interaction(previous);
        frame(&mut app, &ctx, vec![key(Key::ArrowDown, Modifiers::ALT)]);
        settle_frame(&mut app, &ctx);
        assert_eq!((app.caret_line, app.caret_col), (22, 0));
        assert_eq!(app.active_revision, Some(ReviewRevision::Before));
        assert_eq!(base.text, before);
        assert_eq!(source.text, after);
        app.open_file("unchanged.rs");
        settle_frame(&mut app, &ctx);
        let unchanged = app.open_buffer.clone().unwrap();
        assert_eq!(unchanged.revision, Some(ReviewRevision::After));
        assert_eq!(unchanged.path, "unchanged.rs");
        assert!(app.line_changes.is_none());
        assert_eq!(app.change_navigation(), Default::default());
        let history = format!("{:?}", app.history);
        frame(&mut app, &ctx, vec![key(Key::ArrowDown, Modifiers::ALT)]);
        assert!(Arc::ptr_eq(&unchanged, app.open_buffer.as_ref().unwrap()));
        assert_eq!(format!("{:?}", app.history), history);
    }
    assert_eq!(
        std::fs::read(root.join("a file.rs")).unwrap(),
        b"checkout only\n"
    );
}
