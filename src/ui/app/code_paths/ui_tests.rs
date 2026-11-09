use super::*;
use crate::ui::{
    app::tests::{
        focus::{frame, key},
        settle,
    },
    test_support::{click, text_center, text_shapes},
};

fn app() -> (tempfile::TempDir, RadiologyApp, Context) {
    let (owner, mut app) = crate::ui::app::tests::isolated_source(
        "entry.c",
        "#ifdef A\nvoid boot() {}\n#else\nvoid boot() {}\n#endif\n",
    );
    let ctx = Context::default();
    frame(&mut app, &ctx, vec![]);
    (owner, app, ctx)
}

fn import(app: &mut RadiologyApp, ctx: &Context, text: &str) {
    app.execute_command(ctx, ToolbarAction::ImportCodePath);
    frame(app, ctx, vec![]);
    if let ActiveModal::CodePathImport(modal) = &mut app.modal {
        modal.text = text.into();
    } else {
        panic!("Import command did not create its sheet");
    }
    let output = frame(app, ctx, vec![]);
    frame(app, ctx, click(text_center(&output, "Import")));
    settle(app);
    frame(app, ctx, vec![]);
}

#[test]
fn import_click_steps_variants_back_copy_delete_and_relaunch_share_saved_identities() {
    for scale in [1.0, 2.0] {
        let (owner, mut app, ctx) = app();
        ctx.set_pixels_per_point(scale);
        app.show_sidebar = false;
        app.show_inspector = false;
        let departure = app.history.current().cloned().unwrap();
        let text =
            "Boot café 🦀\nentry.c:boot:Boots: the kernel\nentry.c:boot:Second authored step";
        import(&mut app, &ctx, text);
        assert!(matches!(app.modal, ActiveModal::None));
        assert!(app.show_sidebar && !app.show_inspector);
        assert_eq!(app.sidebar_tab, SidebarTab::CodePaths);
        assert_eq!(app.saved_code_paths().len(), 1);
        let id = app.saved_code_paths()[0].id;
        let step_id = app.saved_code_paths()[0].steps[0].id;
        let output = frame(&mut app, &ctx, vec![]);
        frame(
            &mut app,
            &ctx,
            click(text_center(&output, "Boots: the kernel")),
        );
        settle(&mut app);
        assert_eq!(app.selected_code_path_step(), Some(step_id));
        assert_eq!(app.code_path_destinations().len(), 2);
        let first = app.history.current().cloned().unwrap();
        let output = frame(&mut app, &ctx, vec![]);
        frame(
            &mut app,
            &ctx,
            click(text_center(&output, "2 conditional definitions")),
        );
        let output = frame(&mut app, &ctx, vec![]);
        frame(
            &mut app,
            &ctx,
            click(text_center(&output, "Definition 2 · line 4")),
        );
        settle(&mut app);
        assert_ne!(app.history.current(), Some(&first));
        app.execute_command(&ctx, ToolbarAction::NavigateBack);
        settle(&mut app);
        assert_eq!(app.history.current(), Some(&first));
        app.execute_command(&ctx, ToolbarAction::NavigateBack);
        settle(&mut app);
        assert_eq!(app.history.current(), Some(&departure));
        let output = frame(&mut app, &ctx, vec![]);
        frame(&mut app, &ctx, click(text_center(&output, "…")));
        let output = frame(&mut app, &ctx, vec![]);
        let output = frame(
            &mut app,
            &ctx,
            click(text_center(&output, "Copy Code Path")),
        );
        assert_eq!(output.platform_output.copied_text, text);
        let mut relaunched = RadiologyApp::from_session(SessionStore::load_from(
            owner.path().join("settings/session.json"),
        ));
        relaunched.open_repository(&owner.path().join("checkout"));
        settle(&mut relaunched);
        assert_eq!(relaunched.selected_code_path().unwrap().id, id);
        assert_eq!(
            relaunched.selected_code_path().unwrap().steps[0].id,
            step_id
        );
        // A second trace, failed atomic import and selected deletion preserve
        // both persisted identities and the reader, rather than just one row.
        let source = app.open_buffer.clone().unwrap();
        import(
            &mut app,
            &ctx,
            "Incoming request\nentry.c:boot:Other authored trace",
        );
        let second = app.selected_code_path().unwrap().id;
        assert_ne!(second, id);
        let accepted = app.saved_code_paths().to_vec();
        assert_eq!(accepted.len(), 2);
        assert!(app.import_code_path_text("Bad\nentry.c:boot:Valid prefix\ninvalid".into()));
        settle(&mut app);
        assert!(app.code_path_storage_error().unwrap().contains("Line 3:"));
        assert_eq!(app.saved_code_paths(), accepted);
        assert_eq!(app.selected_code_path().unwrap().id, second);
        assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &source));
        assert!(app.delete_code_path(second));
        settle(&mut app);
        assert_eq!(app.saved_code_paths().len(), 1);
        assert_eq!(app.selected_code_path().unwrap().id, id);
        assert_eq!(app.selected_code_path().unwrap().steps[0].id, step_id);
        assert!(app.delete_code_path(id));
        settle(&mut app);
        assert!(app.saved_code_paths().is_empty() && app.selected_code_path().is_none());
        assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &source));
    }
}

#[test]
fn return_edits_text_import_errors_keep_sheet_and_source_and_show_shortcut_only_reveals_left() {
    let (_owner, mut app, ctx) = app();
    let original = app.open_buffer.clone().unwrap();
    app.execute_command(&ctx, ToolbarAction::ImportCodePath);
    frame(&mut app, &ctx, vec![]);
    frame(
        &mut app,
        &ctx,
        vec![
            egui::Event::Text("Boot".into()),
            key(egui::Key::Enter, egui::Modifiers::NONE),
            egui::Event::Text("invalid".into()),
        ],
    );
    assert!(
        matches!(&app.modal, ActiveModal::CodePathImport(modal) if modal.text == "Boot\ninvalid")
    );
    let output = frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, click(text_center(&output, "Import")));
    settle(&mut app);
    let output = frame(&mut app, &ctx, vec![]);
    assert!(
        matches!(&app.modal, ActiveModal::CodePathImport(modal) if modal.text == "Boot\ninvalid")
    );
    assert!(app.saved_code_paths().is_empty());
    assert!(text_shapes(&output).any(|shape| shape.galley.job.text.contains("2")));
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &original));
    frame(
        &mut app,
        &ctx,
        vec![key(egui::Key::Escape, egui::Modifiers::NONE)],
    );
    app.show_sidebar = false;
    app.show_inspector = false;
    frame(
        &mut app,
        &ctx,
        vec![key(
            egui::Key::P,
            egui::Modifiers {
                ctrl: true,
                alt: true,
                ..Default::default()
            },
        )],
    );
    assert!(app.show_sidebar && !app.show_inspector);
    assert_eq!(app.sidebar_tab, SidebarTab::CodePaths);
}

#[test]
fn list_keys_select_without_navigation_and_commit_exact_step_after_tab_switch() {
    let (_owner, mut app, ctx) = app();
    import(
        &mut app,
        &ctx,
        "Boot\nentry.c:boot:First step\nentry.c:boot:Second step",
    );
    let origin = app.history.current().cloned();
    ctx.memory_mut(|memory| memory.request_focus(crate::ui::code_paths::CodePathList::focus_id()));
    frame(
        &mut app,
        &ctx,
        vec![key(egui::Key::End, egui::Modifiers::NONE)],
    );
    assert_eq!(app.history.current(), origin.as_ref());
    let step = app.selected_code_path().unwrap().steps[1].id;
    frame(
        &mut app,
        &ctx,
        vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
    );
    settle(&mut app);
    assert_eq!(app.selected_code_path_step(), Some(step));
    app.sidebar_tab = SidebarTab::Files;
    frame(&mut app, &ctx, vec![]);
    app.sidebar_tab = SidebarTab::CodePaths;
    frame(&mut app, &ctx, vec![]);
    assert_eq!(app.selected_code_path_step(), Some(step));
}

#[test]
fn import_ime_return_and_tab_submission_keep_background_source_unchanged() {
    let (_owner, mut app, ctx) = app();
    let source = app.open_buffer.clone().unwrap();
    frame(
        &mut app,
        &ctx,
        vec![key(
            egui::Key::P,
            egui::Modifiers {
                ctrl: true,
                shift: true,
                ..Default::default()
            },
        )],
    );
    frame(&mut app, &ctx, vec![]);
    frame(
        &mut app,
        &ctx,
        vec![
            egui::Event::CompositionStart,
            egui::Event::CompositionUpdate("にほん".into()),
            egui::Event::CompositionEnd("Boot 日本".into()),
        ],
    );
    frame(
        &mut app,
        &ctx,
        vec![
            key(egui::Key::Enter, egui::Modifiers::NONE),
            egui::Event::Text("entry.c:boot:Enter".into()),
        ],
    );
    assert!(
        matches!(&app.modal, ActiveModal::CodePathImport(modal) if modal.text == "Boot 日本\nentry.c:boot:Enter")
    );
    for _ in 0..3 {
        frame(
            &mut app,
            &ctx,
            vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
        );
    }
    frame(
        &mut app,
        &ctx,
        vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
    );
    settle(&mut app);
    frame(&mut app, &ctx, vec![]);
    assert!(matches!(app.modal, ActiveModal::None));
    assert_eq!(app.selected_code_path().unwrap().title, "Boot 日本");
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &source));
}

#[test]
fn sidebar_variants_errors_and_import_render_within_narrow_widths_in_both_themes() {
    let (_owner, mut app, ctx) = app();
    let root = app.repository_root.clone().unwrap();
    for (path, text) in [
        ("arch/x86/start.S", "_start:\n nop\n"),
        ("kernel/boot.cc", "namespace Kernel { void boot() {} }\n"),
        (
            "kernel/scheduler.cc",
            "#if SMP\nvoid start_scheduler() {}\n#else\nvoid start_scheduler() {}\n#endif\n",
        ),
    ] {
        std::fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
        std::fs::write(root.join(path), text).unwrap();
    }
    let text = "Boot sequence — x86-64\narch/x86/start.S:_start:Establishes the initial stack and enters C startup.\nkernel/boot.cc:Kernel\\:\\:boot:Initializes memory: calls allocator setup before scheduling.\nkernel/scheduler.cc:start_scheduler:If SMP is enabled, wakes secondary processors before entering the first userspace thread.";
    import(&mut app, &ctx, text);
    let first = app.saved_code_paths()[0].clone();
    import(
        &mut app,
        &ctx,
        "Incoming request\nentry.c:boot:Registers the request callback.",
    );
    app.select_code_path(Some(first.clone()));
    let render = |app: &mut RadiologyApp, width| {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, 480.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| app.show_code_paths(ui));
            },
        )
    };
    for state in 0..3 {
        if state == 1 {
            app.go_to_code_path_step(first.steps[2].id, 0);
            settle(&mut app);
            assert_eq!(app.code_path_destinations().len(), 2);
        } else if state == 2 {
            std::fs::write(root.join("arch/x86/start.S"), "renamed_entry:\n nop\n").unwrap();
            app.go_to_code_path_step(first.steps[0].id, 0);
            settle(&mut app);
            assert!(app.code_path_error().unwrap().contains("not found"));
        }
        for scale in [1.0, 2.0] {
            ctx.set_pixels_per_point(scale);
            for dark in [true, false] {
                app.theme = if dark {
                    AppTheme::dark()
                } else {
                    AppTheme::light()
                };
                ctx.set_visuals(app.theme.visuals());
                for width in [220.0, 280.0, 420.0] {
                    render(&mut app, width);
                    let output = render(&mut app, width);
                    text_center(&output, "2 code paths");
                    text_center(&output, "3 steps");
                    if state == 1 {
                        text_center(&output, "2 conditional definitions");
                    }
                    let description = text_shapes(&output)
                        .find(|shape| shape.galley.job.text == first.steps[0].description)
                        .unwrap();
                    if width == 220.0 {
                        assert!(description.galley.rows.len() > 1);
                    }
                    for shape in text_shapes(&output) {
                        assert!(
                            shape.pos.x >= 0.0
                                && shape.pos.x + shape.galley.size().x <= width + 1.0,
                            "text outside pane at {width}/{scale}/{dark}: {} {:?}",
                            shape.galley.job.text,
                            shape.pos
                        );
                    }
                    assert_eq!(app.saved_code_paths().len(), 2);
                    assert_eq!(app.selected_code_path(), Some(first.as_ref()));
                }
                app.execute_command(&ctx, ToolbarAction::ImportCodePath);
                frame(&mut app, &ctx, vec![]);
                let output = frame(&mut app, &ctx, vec![]);
                text_center(&output, "Import Code Path");
                text_center(&output, "Choose Text File…");
                text_center(&output, "Cancel");
                frame(
                    &mut app,
                    &ctx,
                    vec![key(egui::Key::Escape, egui::Modifiers::NONE)],
                );
            }
        }
    }
}
