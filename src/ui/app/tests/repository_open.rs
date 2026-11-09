use super::*;
use crate::repository::discovery::DiscoveredRepository;

#[test]
fn plain_repository_discovery_waits_for_file_choice_and_rejects_a_late_root() {
    let (dir, mut app) = reader();
    let old = dir.path().join("old");
    repository(&old);
    let current = dir.path().join("current");
    std::fs::create_dir(&current).unwrap();
    std::fs::write(current.join("README.md"), "# Current repository\n").unwrap();
    std::fs::write(current.join("last.swift"), "struct Current {}\n").unwrap();
    app.open_repository(&old);
    let (release, gate) = std::sync::mpsc::channel();
    let (finished, done) = std::sync::mpsc::channel();
    let (started, running) = std::sync::mpsc::channel();
    app.repository_open.spawn(move |_| {
        started.send(()).unwrap();
        gate.recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        let _ = finished.send(());
        Ok(DiscoveredRepository {
            root: old,
            files: vec![],
            tree: vec![],
            filters: Default::default(),
        })
    });
    running
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    app.open_repository(&current);
    settle(&mut app);
    release.send(()).unwrap();
    done.recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    app.poll_repository(&Context::default());
    assert_eq!(app.repository_root.as_deref(), Some(current.as_path()));
    assert_eq!(app.all_files.len(), 2);
    assert!(app.open_buffer.is_none() && app.selected_path.is_none());
    assert!(app.history.current().is_none());
    assert!(!app.is_source_loading());
    assert!(app.repository_error.is_none());
    for scale in [1.0, 2.0] {
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        rendered_frame(&mut app, &ctx);
        let output = rendered_frame(&mut app, &ctx);
        assert!(output.shapes.iter().any(|shape| matches!(&shape.shape,
            egui::Shape::Text(text) if text.galley.job.text == "No File Selected")));
    }
    app.open_file("last.swift");
    settle(&mut app);
    assert_eq!(
        app.open_buffer.as_ref().unwrap().text,
        "struct Current {}\n"
    );
    app.open_repository(&current);
    settle(&mut app);
    assert!(app.open_buffer.is_none() && app.history.current().is_none());
}

#[test]
fn repository_failure_cancel_and_retry_keep_the_accepted_reader_and_session() {
    let (dir, mut app) = opened_repository();
    let inventory = app.all_files.clone();
    let root = app.repository_root.clone();
    let buffer = app.open_buffer.clone().unwrap();
    let history = app.history.current().cloned();
    let saved = app.session.last_repository.clone();
    let next = dir.path().join("next");
    app.open_repository(&next);
    assert!(app.is_repository_opening());
    assert_eq!(app.repository_root, root);
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    app.cancel_repository_open();
    settle(&mut app);
    assert_eq!(app.history.current(), history.as_ref());
    assert_eq!(app.session.last_repository, saved);
    app.open_repository(&next);
    settle(&mut app);
    assert!(app.repository_error.is_some());
    assert_eq!(app.repository_root, root);
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    assert_eq!(app.session.last_repository, saved);
    assert!(Arc::ptr_eq(&app.all_files, &inventory));
    repository(&next);
    let retry = app.pending_repository.clone().unwrap();
    app.request_repository(retry);
    settle(&mut app);
    assert_eq!(app.repository_root.as_deref(), Some(next.as_path()));
    assert!(!Arc::ptr_eq(&app.all_files, &inventory));
    assert!(app.repository_error.is_none());
}

#[test]
fn newer_repository_choice_retires_launch_intent_and_repeated_launch_is_ignored() {
    let (dir, mut app) = reader();
    let old = dir.path().join("old");
    repository(&old);
    let next = dir.path().join("next");
    repository(&next);
    let (old_review, receiver) = std::sync::mpsc::channel();
    app.review_job.spawn(move |_| {
        let _ = receiver.recv_timeout(std::time::Duration::from_secs(5));
        Err("retired review".into())
    });
    app.pending_pr = Some(99);
    app.launch(Some(&old), Some(123), Some("stale.rs"));
    let _ = old_review.send(());
    assert!(!app.review_job.is_running());
    assert!(app.pending_pr.is_none(), "PR work must wait for its root");
    assert!(matches!(
        app.pending_repository.as_ref().unwrap().destination,
        super::super::repository::RepositoryDestination::Review { number: 123, .. }
    ));
    let (release, gate) = std::sync::mpsc::channel();
    let retired_root = old.clone();
    app.repository_open.spawn(move |_| {
        gate.recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        Ok(DiscoveredRepository {
            root: retired_root,
            files: vec![],
            tree: vec![],
            filters: Default::default(),
        })
    });
    app.request_repository(super::super::repository::RepositoryRequest {
        path: next.clone(),
        destination: super::super::repository::RepositoryDestination::Source {
            location: SourceLocation::new("b.rs", 0),
            target: crate::source::loader::SourceTarget::Byte(0),
        },
    });
    let _ = release.send(());
    settle(&mut app);
    assert_eq!(app.repository_root.as_deref(), Some(next.as_path()));
    assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
    assert!(app.pending_pr.is_none());
    assert!(app.review.is_none());
    assert!(app.error.is_none());
    app.launch(Some(&old), Some(123), Some("stale.rs"));
    assert!(!app.is_repository_opening());
    assert_eq!(app.repository_root.as_deref(), Some(next.as_path()));
    assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
    assert!(app.pending_pr.is_none() && app.review.is_none());
}

#[test]
fn refresh_restores_live_reader_position_after_discovery_not_the_request_time_caret() {
    let (_dir, mut app) = opened_repository();
    app.jump_to_location("b.rs", 2, 0);
    settle(&mut app);
    app.execute_command(&Context::default(), ToolbarAction::RefreshPR);
    assert!(app.is_repository_opening());
    assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
    app.caret_line = 1;
    app.caret_col = 4;
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
    assert_eq!((app.caret_line, app.caret_col), (1, 4));
    assert_eq!(app.history.current().unwrap().path, "b.rs");
    app.execute_command(&Context::default(), ToolbarAction::RefreshPR);
    app.jump_to_location("a.rs", 2, 0);
    settle_repository(&mut app);
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
    assert_eq!(app.caret_line, 2);
}

#[test]
fn discovery_reprepares_its_tree_when_filters_change_before_publication() {
    let (dir, mut app) = opened_repository();
    let next = dir.path().join("filtered");
    repository(&next);
    std::fs::write(next.join("a_test.rs"), "fn test_one() {}\n").unwrap();
    std::fs::write(next.join("notes.txt"), "notes\n").unwrap();
    let discovered = RepositoryDiscovery::discover(
        &next,
        &Default::default(),
        &std::sync::atomic::AtomicBool::new(false),
    )
    .unwrap();
    app.open_repository(&next);
    app.repository_open.spawn(move |_| Ok(discovered));
    app.session.source_filters.hides_tests = true;
    app.session.source_filters.source_files_only = true;
    settle(&mut app);
    assert_eq!(app.repository_root.as_deref(), Some(next.as_path()));
    assert_eq!(
        app.all_files.len(),
        4,
        "inventory remains independent of filters"
    );
    assert_eq!(
        app.file_tree
            .iter()
            .map(|node| node.path.as_str())
            .collect::<Vec<_>>(),
        ["a.rs", "b.rs"]
    );
}

#[test]
fn dropped_paths_use_repository_transactions_and_only_the_first_local_item() {
    let (dir, mut app) = opened_repository();
    let next = dir.path().join("drop 漢字 ; folder");
    repository(&next);
    std::fs::write(next.join("README.md"), "# Dropped repository\n").unwrap();
    let ctx = Context::default();
    let drop = |app: &mut RadiologyApp, paths: Vec<Option<PathBuf>>| {
        let _ = ctx.run(
            egui::RawInput {
                dropped_files: paths
                    .into_iter()
                    .map(|path| egui::DroppedFile {
                        path,
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            },
            |ctx| app.render(ctx),
        );
    };
    let previous = app.repository_root.clone();
    let buffer = app.open_buffer.clone().unwrap();
    drop(
        &mut app,
        vec![Some(next.join("b.rs")), Some(dir.path().into())],
    );
    assert!(
        app.is_repository_opening() || app.repository_root.as_ref() == Some(&next),
        "native dropped-file input was ignored"
    );
    if app.is_repository_opening() {
        assert_eq!(app.repository_root, previous);
        assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    }
    settle(&mut app);
    assert_eq!(app.repository_root.as_deref(), Some(next.as_path()));
    assert!(app.selected_path.is_none() && app.open_buffer.is_none());
    assert!(app.history.current().is_none());
    assert_eq!(app.session.last_repository.as_deref(), next.to_str());
    assert!(!app.history.can_go_back());
    assert!(app.review.is_none() && app.active_revision.is_none());
    drop(&mut app, vec![None, Some(dir.path().into())]);
    assert!(
        !app.is_repository_opening(),
        "byte-only drops must not open another item"
    );
    drop(&mut app, vec![Some(dir.path().into())]);
    settle(&mut app);
    assert_eq!(app.repository_root.as_deref(), Some(dir.path()));
    let _ = rendered_frame(&mut app, &ctx);
    assert!(
        !app.is_repository_opening(),
        "a consumed drop must not replay"
    );
}

#[test]
fn failed_and_superseded_drops_keep_the_accepted_repository_and_source() {
    let (dir, mut app) = opened_repository();
    let root = app.repository_root.clone();
    let buffer = app.open_buffer.clone().unwrap();
    let saved = app.session.last_repository.clone();
    app.open_dropped_path(&dir.path().join("missing.rs"));
    settle(&mut app);
    assert!(app.repository_error.is_some());
    assert_eq!(app.repository_root, root);
    assert_eq!(app.session.last_repository, saved);
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    let next = dir.path().join("next");
    repository(&next);
    app.open_dropped_path(&next);
    app.cancel_repository_open();
    settle(&mut app);
    assert_eq!(app.repository_root, root);
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    app.open_dropped_path(&next);
    app.open_repository(dir.path());
    settle(&mut app);
    assert_eq!(app.repository_root.as_deref(), Some(dir.path()));
    assert!(app.repository_error.is_none());
}

#[test]
fn refresh_availability_blocks_replacement_of_pending_repository_or_review_work() {
    let (_settings, empty) = reader();
    assert!(!empty.command_enabled(&ToolbarAction::Refresh));
    assert!(!empty.command_enabled(&ToolbarAction::RefreshPR));
    let (_dir, mut app) = opened_repository();
    let ctx = Context::default();
    assert!(app.command_enabled(&ToolbarAction::Refresh));
    app.execute_command(&ctx, ToolbarAction::Refresh);
    assert!(app.is_repository_opening());
    assert!(!app.command_enabled(&ToolbarAction::Refresh));
    app.cancel_repository_open();
    app.pending_pr = Some(7);
    assert!(!app.command_enabled(&ToolbarAction::Refresh));
    assert!(!app.command_enabled(&ToolbarAction::RefreshPR));
    app.execute_command(&ctx, ToolbarAction::RefreshPR);
    assert_eq!(app.pending_pr, Some(7));
    assert!(!app.is_repository_opening());
    app.pending_pr = None;
    assert!(app.command_enabled(&ToolbarAction::Refresh));
}

#[test]
fn missing_saved_repository_keeps_recovery_identity_and_invalid_sessions_remain_empty() {
    let (owner, mut app) = reader();
    let root = owner.path().join("saved");
    repository(&root);
    app.session.record_repository(root.display().to_string());
    app.session.set_active_pr(Some(758));
    let settings = owner.path().join("settings/session.json");
    let remembered = app.session.last_repository.clone();
    std::fs::rename(&root, owner.path().join("unavailable")).unwrap();
    let mut restored = RadiologyApp::from_session(SessionStore::load_from(&settings));
    assert_eq!(restored.session.last_repository, remembered);
    restored.restore_session();
    settle(&mut restored);
    assert!(restored.repository_root.is_none());
    assert!(restored.repository_error.is_some());
    assert_eq!(SessionStore::load_from(&settings).active_pr, Some(758));
    let other = owner.path().join("other");
    repository(&other);
    restored.open_repository(&other);
    settle(&mut restored);
    assert_eq!(restored.repository_root.as_deref(), Some(other.as_path()));
    assert!(restored.repository_error.is_none());
    assert_eq!(SessionStore::load_from(&settings).active_pr, None);
    let invalid_payloads = [("relative", Some(758)), ("/tmp", Some(0))].map(|(path, pr)| {
        let mut invalid = SessionStore::default();
        invalid.last_repository = Some(path.into());
        invalid.active_pr = pr;
        serde_json::to_vec(&invalid).unwrap()
    });
    for payload in invalid_payloads.into_iter().chain([b"broken".to_vec()]) {
        std::fs::write(&settings, payload).unwrap();
        let mut empty = RadiologyApp::from_session(SessionStore::load_from(&settings));
        empty.restore_session();
        settle(&mut empty);
        assert!(empty.repository_root.is_none() && empty.repository_error.is_none());
        assert_eq!(empty.session.active_pr, None);
    }
}

#[test]
fn saved_review_identity_survives_checkout_preparation_failure_cancel_and_fresh_restore() {
    let (_repository, root, mut snapshot) = crate::pull_request::review::tests::fixture();
    snapshot.summary.number = 758;
    let (owner, mut app) = reader();
    app.session.record_repository(root.display().to_string());
    app.session.set_active_pr(Some(758));
    let settings = owner.path().join("settings/session.json");
    let ctx = Context::default();
    let mut restored = RadiologyApp::from_session(SessionStore::load_from(&settings));
    restored.restore_session();
    settle_repository(&mut restored);
    assert_eq!(SessionStore::load_from(&settings).active_pr, Some(758));
    restored.review_job.spawn(|_| Err("fixture offline".into()));
    settle(&mut restored);
    assert!(
        restored.review.is_none() && restored.review_error().unwrap().contains("fixture offline")
    );
    restored.show_pull_request_prompt(None);
    assert!(matches!(&restored.modal, ActiveModal::PullRequest(prompt) if prompt.pr_str == "758"));
    let mut fresh = RadiologyApp::from_session(SessionStore::load_from(&settings));
    fresh.restore_session();
    settle_repository(&mut fresh);
    assert_eq!(fresh.pending_pr, Some(758));
    fresh.cancel_review_preparation();
    fresh.poll_review(&ctx);
    assert_eq!(SessionStore::load_from(&settings).active_pr, Some(758));
    fresh.open_pull_request(758);
    fresh.review_job.spawn(move |_| Ok(snapshot));
    settle(&mut fresh);
    assert_eq!(fresh.active_pr.as_ref().unwrap().number, 758);
    assert_eq!(fresh.repository_root.as_deref(), Some(root.as_path()));
    assert_ne!(fresh.review.as_ref().unwrap().head_root, root);
    assert_eq!(
        SessionStore::load_from(&settings)
            .last_repository
            .as_deref(),
        root.to_str()
    );
    fresh.open_pull_request(999);
    fresh
        .review_job
        .spawn(|_| Err("unavailable replacement".into()));
    settle(&mut fresh);
    assert_eq!(fresh.active_pr.as_ref().unwrap().number, 758);
    assert_eq!(SessionStore::load_from(&settings).active_pr, Some(758));
    fresh.open_pull_request(888);
    fresh.cancel_review_preparation();
    settle(&mut fresh);
    assert_eq!(fresh.active_pr.as_ref().unwrap().number, 758);
    assert_eq!(SessionStore::load_from(&settings).active_pr, Some(758));
    fresh.execute_command(&ctx, ToolbarAction::ExitPRReview);
    settle(&mut fresh);
    assert_eq!(SessionStore::load_from(&settings).active_pr, None);
    let mut local = RadiologyApp::from_session(SessionStore::load_from(&settings));
    local.restore_session();
    settle(&mut local);
    assert_eq!(local.repository_root.as_deref(), Some(root.as_path()));
    assert!(local.review.is_none() && local.pending_pr.is_none());
}
