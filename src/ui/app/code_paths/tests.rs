//! App ownership and exact-history tests, including completions already queued.
use super::*;
use crate::ui::app::tests::{opened_repository, reader, settle};

fn trace(app: &mut RadiologyApp, text: &str) -> Arc<CodePath> {
    let trace = Arc::new(CodePath::parse(text).unwrap());
    assert!(app.select_code_path(Some(trace.clone())));
    trace
}

fn caret(app: &RadiologyApp) -> SourceLocation {
    let buffer = app.open_buffer.as_ref().unwrap();
    buffer.location_at_byte(buffer.byte_offset(app.caret_line, app.caret_col))
}

/// Replace the worker with an already-queued error, retaining its real app owner.
/// The receipt proves the result is terminal/queued before a competing action.
fn queue_error(app: &mut RadiologyApp) {
    let (done, receipt) = std::sync::mpsc::channel();
    app.code_paths.job.spawn_with_sender(move |_, sender| {
        sender.send(Err("retired error".into())).unwrap();
        done.send(()).unwrap();
    });
    receipt
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
}

#[test]
fn opens_steps_reuses_authoritative_source_and_restores_exact_departure() {
    let (_dir, mut app) = opened_repository();
    app.open_file("a.rs");
    settle(&mut app);
    app.jump_to_location("a.rs", 2, 8);
    settle(&mut app);
    let departure = caret(&app);
    let selected = trace(
        &mut app,
        "Boot\nb.rs:beta:Enter beta\na.rs:alpha:Return alpha",
    );
    app.go_to_code_path_step(selected.steps[0].id, 0);
    assert!(app.is_code_path_loading());
    assert_eq!(caret(&app), departure);
    settle(&mut app);
    assert_eq!(caret(&app), SourceLocation::new("b.rs", 3));
    assert_eq!(app.selected_code_path_step(), Some(selected.steps[0].id));
    assert_eq!(app.code_path_destinations().len(), 1);
    assert!(app.code_path_error().is_none());
    let fold =
        crate::syntax::folds::enclosing_function_fold(3, &app.outline_symbols, &app.fold_ranges)
            .unwrap();
    assert!(!app.fold_store.is_folded(&fold.byte_range));
    app.execute_command(&Context::default(), ToolbarAction::NavigateBack);
    settle(&mut app);
    assert_eq!(caret(&app), departure);
    let original = app.open_buffer.clone().unwrap();
    // Unsaved/external checkout edits cannot replace the matching displayed source.
    std::fs::write(
        app.repository_root.as_ref().unwrap().join("a.rs"),
        "fn renamed() {}\n",
    )
    .unwrap();
    app.go_to_code_path_step(selected.steps[1].id, 0);
    settle(&mut app);
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &original));
    assert_eq!(caret(&app).byte_offset, 3);
    assert_eq!(app.selected_code_path_step(), Some(selected.steps[1].id));
    assert!(app.code_path_error().is_none());
}

#[test]
fn opens_first_conditional_then_explicit_variant_and_reports_invalid_variant_atomically() {
    let (_dir, mut app) = opened_repository();
    let source = "#ifdef A\nvoid start() {}\n#else\nvoid start() {}\n#endif\n";
    std::fs::write(app.repository_root.as_ref().unwrap().join("boot.c"), source).unwrap();
    let selected = trace(&mut app, "Boot\nboot.c:start:Choose entry");
    let id = selected.steps[0].id;
    app.go_to_code_path_step(id, 0);
    settle(&mut app);
    assert_eq!(caret(&app).byte_offset, source.find("start").unwrap());
    assert_eq!(app.code_path_destinations().len(), 2);
    app.go_to_code_path_step(id, 1);
    settle(&mut app);
    let second = caret(&app);
    assert_eq!(second.byte_offset, source.rfind("start").unwrap());
    app.go_to_code_path_step(id, 2);
    settle(&mut app);
    assert_eq!(caret(&app), second);
    assert_eq!(app.history.current(), Some(&second));
    assert_eq!(app.selected_code_path_step(), Some(id));
    assert!(app.code_path_destinations().is_empty());
    assert!(app
        .code_path_error()
        .unwrap()
        .contains("no longer available"));
}

#[test]
fn missing_ambiguous_and_cancelled_steps_preserve_source_caret_and_history() {
    let (dir, mut app) = opened_repository();
    std::fs::write(dir.path().join("bad.c"), "void f() {}\nvoid f(int n) {}\n").unwrap();
    let selected = trace(&mut app, "Boot\na.rs:missing:Missing\nbad.c:f:Ambiguous");
    let original = app.open_buffer.clone().unwrap();
    let departure = caret(&app);
    let history = app.history.current().cloned();
    for step in &selected.steps {
        app.go_to_code_path_step(step.id, 0);
        settle(&mut app);
        assert!(app.code_path_error().is_some());
        assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &original));
        assert_eq!(caret(&app), departure);
        assert_eq!(app.history.current(), history.as_ref());
        assert_eq!(app.selected_code_path_step(), None);
    }
    app.go_to_code_path_step(selected.steps[0].id, 0);
    queue_error(&mut app);
    app.cancel_source_load();
    app.poll_source(&Context::default());
    assert!(!app.is_code_path_loading());
    assert!(app.code_path_error().is_none());
    assert_eq!(caret(&app), departure);
}

#[test]
fn queued_errors_retire_on_caret_round_trip_navigation_and_same_id_trace_replacement() {
    let (_dir, mut app) = opened_repository();
    for action in 0..4 {
        let selected = trace(&mut app, "Boot\nb.rs:beta:Enter beta");
        app.go_to_code_path_step(selected.steps[0].id, 0);
        queue_error(&mut app);
        match action {
            0 => {
                let previous = (app.caret_line, app.caret_col);
                app.caret_col += 1;
                app.finish_source_interaction(previous);
                let changed = (app.caret_line, app.caret_col);
                (app.caret_line, app.caret_col) = previous;
                app.finish_source_interaction(changed);
            }
            1 => app.open_file("b.rs"),
            2 => {
                assert!(app.select_code_path(Some(Arc::new((*selected).clone()))));
            }
            _ => {
                assert!(app.select_code_path(None));
            }
        }
        settle(&mut app);
        assert!(app.code_path_error().is_none());
        assert!(app.selected_code_path_step().is_none());
    }
}

#[test]
fn queued_success_cannot_overwrite_a_changed_origin_or_new_navigation() {
    for navigate in [false, true] {
        let (_dir, mut app) = opened_repository();
        let selected = trace(&mut app, "Boot\nb.rs:beta:Enter beta");
        app.go_to_code_path_step(selected.steps[0].id, 0);
        // Obtain the actual worker result without letting the app publish it.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let ready = loop {
            if let Some(ready) = app.code_paths.job.poll().unwrap() {
                break ready.unwrap();
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        };
        let (done, receipt) = std::sync::mpsc::channel();
        app.code_paths.job.spawn_with_sender(move |_, sender| {
            assert!(sender.send(Ok(ready)).is_ok());
            done.send(()).unwrap();
        });
        receipt
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        if navigate {
            app.jump_to_location("a.rs", 2, 4);
        } else {
            // Even equal bytes in a different source object retire the request.
            app.open_buffer = app
                .open_buffer
                .as_ref()
                .map(|buffer| Arc::new((**buffer).clone()));
        }
        let before = caret(&app);
        settle(&mut app);
        assert_eq!(caret(&app), before);
        assert_eq!(app.selected_code_path_step(), None);
        assert!(app.code_path_destinations().is_empty());
    }
}

#[test]
fn review_opens_pinned_after_and_back_restores_exact_before_departure() {
    let before = "// before café 🦀\nfn entry() {}\n";
    let after = "// after café 🦀\n\nfn entry() {}\n";
    let (_owner, root, review) =
        crate::pull_request::review::tests::fixture_with_sources(before, after);
    let (_settings, mut app) =
        crate::ui::app::tests::review_refresh::open_review(&root, review, ReviewRevision::Before);
    app.jump_to_location("a file.rs", 1, 8);
    settle(&mut app);
    let departure = caret(&app);
    assert_eq!(departure.revision, Some(ReviewRevision::Before));
    let selected = trace(&mut app, "Entry\na file.rs:entry:Begin");
    app.go_to_code_path_step(selected.steps[0].id, 0);
    settle(&mut app);
    assert_eq!(app.open_buffer.as_ref().unwrap().text, after);
    assert_eq!(caret(&app).revision, Some(ReviewRevision::After));
    assert_eq!(caret(&app).byte_offset, after.find("entry").unwrap());
    app.execute_command(&Context::default(), ToolbarAction::NavigateBack);
    settle(&mut app);
    assert_eq!(caret(&app), departure);
    assert_eq!(app.open_buffer.as_ref().unwrap().text, before);
    // New snapshot identity retires a queued error, even if its Git heads are equal.
    app.go_to_code_path_step(selected.steps[0].id, 0);
    queue_error(&mut app);
    app.review = app
        .review
        .as_ref()
        .map(|review| Arc::new((**review).clone()));
    app.poll_source(&Context::default());
    assert!(app.code_path_error().is_none());
    assert_eq!(caret(&app), departure);
}

#[test]
fn theme_change_recolors_without_replacing_selected_trace_or_prepared_source() {
    let (_dir, mut app) = opened_repository();
    let selected = trace(&mut app, "Boot\nb.rs:beta:Enter beta");
    app.go_to_code_path_step(selected.steps[0].id, 0);
    app.theme = AppTheme::light();
    app.queue_rehighlight();
    settle(&mut app);
    assert_eq!(caret(&app), SourceLocation::new("b.rs", 3));
    assert_eq!(app.selected_code_path_step(), Some(selected.steps[0].id));
    assert_eq!(
        app.highlighted_lines,
        app.native_highlights
            .recolor(app.open_buffer.as_ref().unwrap(), false, || false)
            .unwrap()
    );
    assert!(app.code_path_error().is_none());
}

#[test]
fn repository_transition_clears_trace_and_idle_model_rejects_selection() {
    let (_settings, mut idle) = reader();
    let selected = Arc::new(CodePath::parse("Boot\na.rs:alpha:Enter").unwrap());
    assert!(!idle.select_code_path(Some(selected.clone())));
    let (dir, mut app) = opened_repository();
    assert!(app.select_code_path(Some(selected.clone())));
    app.go_to_code_path_step(selected.steps[0].id, 0);
    queue_error(&mut app);
    let other = dir.path().join("other");
    std::fs::create_dir(&other).unwrap();
    std::fs::write(other.join("c.rs"), "fn other() {}\n").unwrap();
    app.open_repository(&other);
    settle(&mut app);
    assert!(app.selected_code_path().is_none());
    assert!(app.code_path_error().is_none());
    assert_eq!(app.repository_root.as_ref(), Some(&other));
}

#[test]
fn a_real_keyboard_batch_that_returns_to_its_origin_retires_the_step_job() {
    use crate::ui::app::tests::focus::{frame, key};
    for scale in [1.0, 2.0] {
        let (_dir, mut app) = opened_repository();
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        frame(&mut app, &ctx, vec![]);
        frame(&mut app, &ctx, vec![]);
        app.source_viewport.request_focus(&ctx);
        let selected = trace(&mut app, "Boot\nb.rs:beta:Enter beta");
        app.go_to_code_path_step(selected.steps[0].id, 0);
        // Keep completion gated until the real source widget has processed both keys.
        let (release, gate) = std::sync::mpsc::channel();
        let (observed, receipt) = std::sync::mpsc::channel();
        let (started, running) = std::sync::mpsc::channel();
        app.code_paths.job.spawn_with_sender(move |cancel, sender| {
            started.send(()).unwrap();
            gate.recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
            observed
                .send(cancel.load(std::sync::atomic::Ordering::Relaxed))
                .unwrap();
            let _ = sender.send(Err("late batch error".into()));
        });
        running
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        let departure = caret(&app);
        frame(
            &mut app,
            &ctx,
            vec![
                key(egui::Key::ArrowRight, egui::Modifiers::NONE),
                key(egui::Key::ArrowLeft, egui::Modifiers::NONE),
            ],
        );
        assert_eq!(caret(&app), departure);
        assert!(!app.is_code_path_loading());
        release.send(()).unwrap();
        assert!(receipt
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap());
        app.poll_source(&ctx);
        assert!(app.code_path_error().is_none());
    }
}
