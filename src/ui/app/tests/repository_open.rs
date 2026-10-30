use super::*;
use crate::repository::discovery::DiscoveredRepository;

#[test]
fn repository_failure_cancel_and_retry_keep_the_accepted_reader_and_session() {
    let (dir, mut app) = opened_repository();
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
    repository(&next);
    let retry = app.pending_repository.clone().unwrap();
    app.request_repository(retry);
    settle(&mut app);
    assert_eq!(app.repository_root.as_deref(), Some(next.as_path()));
    assert!(app.repository_error.is_none());
}

#[test]
fn replacing_discovery_retires_the_previous_root_and_its_launch_intent() {
    let (dir, mut app) = opened_repository();
    let old = dir.path().to_path_buf();
    let next = old.join("next");
    repository(&next);
    let (old_review, receiver) = std::sync::mpsc::channel();
    app.review_rx = Some(receiver);
    app.pending_pr = Some(99);
    app.launch(Some(&old), Some(123), Some("stale.rs"));
    assert!(old_review.send(Err("retired review".into())).is_err());
    assert!(app.pending_pr.is_none(), "PR work must wait for its root");
    assert!(matches!(
        app.pending_repository.as_ref().unwrap().destination,
        super::super::repository::RepositoryDestination::Review { number: 123, .. }
    ));
    let (release, gate) = std::sync::mpsc::channel();
    app.repository_open.spawn(move |_| {
        gate.recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        Ok(DiscoveredRepository {
            root: old,
            files: vec![],
            tree: vec![],
            filters: Default::default(),
        })
    });
    app.launch(Some(&next), None, Some("b.rs"));
    let _ = release.send(());
    settle(&mut app);
    assert_eq!(app.repository_root.as_deref(), Some(next.as_path()));
    assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
    assert!(app.pending_pr.is_none());
    assert!(app.review.is_none());
    assert!(app.error.is_none());
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
