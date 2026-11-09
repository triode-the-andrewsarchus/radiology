use super::*;
use crate::semantic::indexing::personality::{Personality, Settings};

fn disable_tools(app: &mut RadiologyApp) {
    app.index_tools = crate::semantic::indexing::tools::ToolDirectory {
        managed: None,
        home: None,
        search_path: vec![],
    };
}

#[test]
fn personality_selection_persists_across_alias_relaunch_and_pr_without_reader_changes() {
    let (owner, mut app) = opened_repository();
    disable_tools(&mut app);
    app.open_file("b.rs");
    settle(&mut app);
    app.jump_to_location("a.rs", 2, 5);
    settle(&mut app);
    let buffer = app.open_buffer.clone().unwrap();
    let caret = (app.caret_line, app.caret_col);
    let history = format!("{:?}", app.history);
    let root = app.repository_root.clone().unwrap();
    let ctx = Context::default();
    app.execute_command(
        &ctx,
        ToolbarAction::SelectIndexingPersonality(Personality::Sel4),
    );
    settle(&mut app);
    assert_eq!(app.indexing_personality(), Personality::Sel4);
    assert_eq!(
        app.generation_report().unwrap().plan.selected[0].indexer,
        crate::semantic::indexing::Indexer::Sel4
    );
    assert!(app.generation_report().unwrap().outcomes[0]
        .failure
        .is_some());
    assert_eq!((app.caret_line, app.caret_col), caret);
    assert_eq!(format!("{:?}", app.history), history);
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    let alias = owner.path().join("alias");
    std::os::unix::fs::symlink(&root, &alias).unwrap();
    let session = SessionStore::load_from(owner.path().join("settings/session.json"));
    assert_eq!(session.indexing_personality(&alias), Personality::Sel4);
    let mut restarted = RadiologyApp::from_session(session);
    disable_tools(&mut restarted);
    restarted.open_repository(&alias);
    settle(&mut restarted);
    assert_eq!(restarted.indexing_personality(), Personality::Sel4);
    let next = owner.path().join("next");
    repository(&next);
    restarted.open_repository(&next);
    settle(&mut restarted);
    assert_eq!(restarted.indexing_personality(), Personality::Default);
    restarted.open_repository(&root);
    settle(&mut restarted);
    assert_eq!(restarted.indexing_personality(), Personality::Sel4);

    let (_source, attached, review) = crate::pull_request::review::tests::indexing_fixture();
    app.open_repository(&attached);
    settle(&mut app);
    app.execute_command(
        &ctx,
        ToolbarAction::SelectIndexingPersonality(Personality::Sel4),
    );
    settle(&mut app);
    app.accept_review((*review).clone());
    settle(&mut app);
    assert_eq!(app.indexing_personality(), Personality::Sel4);
    assert_eq!(
        app.index_plan().selected[0].indexer,
        crate::semantic::indexing::Indexer::Sel4
    );
    app.execute_command(&ctx, ToolbarAction::ExitPRReview);
    settle(&mut app);
    assert_eq!(app.indexing_personality(), Personality::Sel4);
}

#[test]
fn settings_errors_block_fallback_and_preserve_recovery_path_source_and_history() {
    for personality in [Personality::Default, Personality::Sel4] {
        let (_owner, mut app) = isolated_source("main.c", "int main(void) { return 0; }\n");
        disable_tools(&mut app);
        let ctx = Context::default();
        if personality != Personality::Default {
            app.execute_command(&ctx, ToolbarAction::SelectIndexingPersonality(personality));
            settle(&mut app);
        }
        let buffer = app.open_buffer.clone().unwrap();
        let history = format!("{:?}", app.history);
        let path = app.indexing_settings_path().unwrap();
        let original = std::fs::read(&path).unwrap();
        app.semantic_index = Some(Arc::new(SemanticIndex::default()));
        std::fs::write(&path, "invalid settings").unwrap();
        app.execute_command(&ctx, ToolbarAction::GenerateIndex);
        settle(&mut app);
        assert!(app
            .indexing_personality_error()
            .unwrap()
            .contains(&format!("{}.toml", personality.id())));
        assert!(app.indexing_personality_settings().is_none());
        assert!(app.semantic_index.is_none() && app.generation_report().is_none());
        assert_eq!(app.indexing_settings_path(), Some(path.clone()));
        assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
        assert_eq!(format!("{:?}", app.history), history);
        app.index_projects_open = true;
        rendered_frame(&mut app, &ctx);
        let output = rendered_frame(&mut app, &ctx);
        for label in ["Personality", "Show Settings File", "Reload Settings"] {
            crate::ui::test_support::text_center(&output, label);
        }
        let command = crate::desktop::prepare_reveal(
            path.parent().unwrap(),
            &format!("{}.toml", personality.id()),
        )
        .unwrap();
        assert!(command.arguments.iter().any(|argument| argument
            .to_string_lossy()
            .contains(&format!("{}.toml", personality.id()))));
        let edited = String::from_utf8(original).unwrap().replace(
            if personality == Personality::Default {
                "clang_jobs = 4"
            } else {
                "clang_jobs = 1"
            },
            "clang_jobs = 2",
        );
        std::fs::write(&path, &edited).unwrap();
        app.execute_command(&ctx, ToolbarAction::ReloadIndexImport);
        settle(&mut app);
        assert!(app.indexing_personality_error().is_none());
        assert_eq!(app.indexing_personality_settings().unwrap().clang_jobs(), 2);
        app.execute_command(&ctx, ToolbarAction::GenerateIndex);
        settle(&mut app);
        assert_eq!(
            app.generation_report().unwrap().plan.selected[0]
                .personality_digest
                .as_deref(),
            Some(app.indexing_personality_settings().unwrap().digest())
        );
        assert_eq!(std::fs::read_to_string(path).unwrap(), edited);
        assert_eq!(app.indexing_personality(), personality);
        if personality == Personality::Sel4 {
            assert_eq!(
                app.generation_report().unwrap().plan.selected[0].indexer,
                crate::semantic::indexing::Indexer::Sel4
            );
        }
    }
}

#[test]
fn failed_preference_persistence_keeps_the_selected_pipeline_and_reader() {
    let (owner, mut app) = opened_repository();
    let buffer = app.open_buffer.clone().unwrap();
    let root = app.repository_root.as_ref().unwrap();
    let forbidden = root.join("forbidden/session.json");
    let mut unsafe_session = SessionStore::load_from(&forbidden);
    assert!(unsafe_session
        .set_indexing_personality(root, Personality::Sel4)
        .unwrap_err()
        .contains("outside"));
    assert!(!forbidden.parent().unwrap().exists());
    let path = owner.path().join("settings/session.json");
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir(&path).unwrap();
    app.execute_command(
        &Context::default(),
        ToolbarAction::SelectIndexingPersonality(Personality::Sel4),
    );
    assert!(app
        .error
        .as_ref()
        .unwrap()
        .contains("Cannot save indexing personality"));
    assert_eq!(app.indexing_personality(), Personality::Default);
    assert_eq!(
        app.session
            .indexing_personality(app.repository_root.as_ref().unwrap()),
        Personality::Default
    );
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    assert!(!app.is_index_loading());
}

#[test]
fn retired_settings_load_cannot_publish_after_selection_or_repository_replacement() {
    use std::{sync::mpsc, time::Duration};
    let (owner, mut app) = opened_repository();
    disable_tools(&mut app);
    let next = owner.path().join("next");
    repository(&next);
    for replace_repository in [false, true] {
        let (started, ready) = mpsc::channel();
        let (retired, done) = mpsc::channel();
        let settings = Arc::new(
            Settings::parse(
                Personality::Default,
                "retired.toml".into(),
                &Personality::Default
                    .template()
                    .iter()
                    .copied()
                    .chain(b"\n# retired result".iter().copied())
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        );
        app.index_personality.job.spawn(move |cancel| {
            crate::pull_request::test_support::await_retirement(cancel, started, retired);
            Ok(settings)
        });
        ready.recv_timeout(Duration::from_secs(5)).unwrap();
        if replace_repository {
            app.open_repository(&next);
        } else {
            app.execute_command(
                &Context::default(),
                ToolbarAction::SelectIndexingPersonality(Personality::Sel4),
            );
        }
        settle(&mut app);
        done.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(!app
            .indexing_personality_settings()
            .unwrap()
            .bytes()
            .ends_with(b"retired result"));
        assert_eq!(
            app.indexing_personality(),
            if replace_repository {
                Personality::Default
            } else {
                Personality::Sel4
            }
        );
    }
}

#[test]
fn actual_picker_arrows_return_and_escape_preserve_source_and_history() {
    use crate::ui::test_support::{click, text_center};
    use egui::{Key, Modifiers};
    let (_owner, mut app) = opened_repository();
    disable_tools(&mut app);
    let buffer = app.open_buffer.clone().unwrap();
    let history = format!("{:?}", app.history);
    let ctx = Context::default();
    app.index_projects_open = true;
    for _ in 0..3 {
        focus::frame(&mut app, &ctx, vec![]);
    }
    let output = focus::frame(&mut app, &ctx, vec![]);
    focus::frame(&mut app, &ctx, click(text_center(&output, "Default")));
    focus::frame(
        &mut app,
        &ctx,
        vec![focus::key(Key::ArrowDown, Modifiers::NONE)],
    );
    focus::frame(
        &mut app,
        &ctx,
        vec![focus::key(Key::Enter, Modifiers::NONE)],
    );
    settle(&mut app);
    assert_eq!(app.indexing_personality(), Personality::Sel4);
    let output = focus::frame(&mut app, &ctx, vec![]);
    focus::frame(&mut app, &ctx, click(text_center(&output, "seL4")));
    focus::frame(
        &mut app,
        &ctx,
        vec![
            focus::key(Key::ArrowDown, Modifiers::NONE),
            focus::key(Key::Escape, Modifiers::NONE),
        ],
    );
    assert_eq!(app.indexing_personality(), Personality::Sel4);
    assert!(!crate::ui::context_menu::is_open(&ctx));
    let output = focus::frame(&mut app, &ctx, vec![]);
    focus::frame(&mut app, &ctx, click(text_center(&output, "View")));
    focus::frame(
        &mut app,
        &ctx,
        vec![focus::key(Key::ArrowRight, Modifiers::NONE)],
    );
    let output = focus::frame(&mut app, &ctx, vec![]);
    text_center(&output, "Find in File (Ctrl+F)");
    focus::frame(
        &mut app,
        &ctx,
        vec![focus::key(Key::Escape, Modifiers::NONE)],
    );
    assert_eq!(format!("{:?}", app.history), history);
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
}

#[test]
fn rapid_personality_switches_keep_the_reader_and_bounded_projects_window() {
    for scale in [1.0, 2.0] {
        for dark in [false, true] {
            let (_owner, mut app) = opened_repository();
            disable_tools(&mut app);
            let ctx = Context::default();
            ctx.set_pixels_per_point(scale);
            if app.theme.is_dark != dark {
                app.execute_command(&ctx, ToolbarAction::ToggleTheme);
                settle(&mut app);
            }
            let source = app.open_buffer.clone().unwrap();
            let caret = (app.caret_line, app.caret_col);
            let history = format!("{:?}", app.history);
            for selected in [Personality::Sel4, Personality::Default, Personality::Sel4] {
                app.execute_command(&ctx, ToolbarAction::SelectIndexingPersonality(selected));
            }
            settle(&mut app);
            assert_eq!(app.indexing_personality(), Personality::Sel4);
            assert_eq!(
                app.indexing_personality_settings().unwrap().personality(),
                Personality::Sel4
            );
            assert_eq!(
                app.generation_report().unwrap().plan.selected[0].indexer,
                crate::semantic::indexing::Indexer::Sel4
            );
            app.index_projects_open = true;
            for _ in 0..3 {
                rendered_frame(&mut app, &ctx);
            }
            let output = rendered_frame(&mut app, &ctx);
            for label in ["Personality", "seL4", "Show Settings File"] {
                crate::ui::test_support::text_center(&output, label);
            }
            let window = ctx
                .memory(|memory| memory.area_rect(egui::Id::new("Index Projects")))
                .unwrap();
            assert!(
                window.width() >= 360.0 && window.width() < 600.0,
                "{window:?}"
            );
            assert!(window.height() < 750.0, "{window:?}");
            assert!(Arc::ptr_eq(&source, app.open_buffer.as_ref().unwrap()));
            assert_eq!((app.caret_line, app.caret_col), caret);
            assert_eq!(format!("{:?}", app.history), history);
            app.execute_command(&ctx, ToolbarAction::GenerateIndex);
            settle(&mut app);
            assert_eq!(
                app.generation_report().unwrap().plan.selected[0].indexer,
                crate::semantic::indexing::Indexer::Sel4
            );
            assert!(Arc::ptr_eq(&source, app.open_buffer.as_ref().unwrap()));
        }
    }
}
