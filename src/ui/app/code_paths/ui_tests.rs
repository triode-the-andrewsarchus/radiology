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
        assert!(app.delete_code_path(id));
        settle(&mut app);
        assert!(app.saved_code_paths().is_empty() && app.selected_code_path().is_none());
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
