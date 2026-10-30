use super::*;
use crate::pull_request::test_support::await_retirement;
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[test]
fn cancelled_and_failed_review_jobs_keep_the_accepted_reader_and_drop_launch_intent() {
    let (_dir, mut app) = opened_repository();
    let buffer = app.open_buffer.clone().unwrap();
    let history = app.history.current().cloned();
    let saved = app.session.last_repository.clone();
    let (started, begin) = mpsc::channel();
    let (retired, done) = mpsc::channel();
    app.review_job.spawn(move |cancel| {
        await_retirement(cancel, started, retired);
        Err("retired review must never publish".into())
    });
    app.pending_pr = Some(7);
    app.pending_open_file = Some("stale.rs".into());
    begin.recv_timeout(Duration::from_secs(5)).unwrap();
    app.cancel_review_preparation();
    done.recv_timeout(Duration::from_secs(5)).unwrap();
    app.poll_review(&Context::default());
    assert!(app.error.is_none());
    assert!(app.pending_pr.is_none());
    assert!(app.pending_open_file.is_none());
    app.pending_pr = Some(8);
    app.pending_open_file = Some("failed.rs".into());
    app.review_job.spawn(|_| Err("fixture timeout".into()));
    let deadline = Instant::now() + Duration::from_secs(5);
    while app.pending_pr.is_some() {
        app.poll_review(&Context::default());
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(app.error.as_ref().unwrap().contains("fixture timeout"));
    assert!(app.pending_open_file.is_none());
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    assert_eq!(app.history.current(), history.as_ref());
    assert_eq!(app.session.last_repository, saved);
    assert!(app.review.is_none());
    assert!(app.active_pr.is_none());
}

#[test]
fn switching_pull_requests_with_identical_commits_changes_bookmark_and_session_scope() {
    let (_repository, root, snapshot) = crate::pull_request::review::tests::fixture();
    let (settings, mut app) = reader();
    app.open_repository(&root);
    settle(&mut app);
    app.accept_review(snapshot.clone());
    settle(&mut app);
    app.execute_command(&Context::default(), ToolbarAction::ToggleBookmark);
    assert_eq!(app.bookmarks.bookmarks.len(), 1);
    let mut other = snapshot;
    other.summary.number = 8;
    other.summary.title = "Same commits in another PR".into();
    app.accept_review(other);
    settle(&mut app);
    assert_eq!(app.active_pr.as_ref().unwrap().number, 8);
    assert_eq!(app.session.active_pr, Some(8));
    assert!(
        app.bookmarks.bookmarks.is_empty(),
        "another PR must not inherit the previous PR's bookmarks"
    );
    assert_eq!(
        SessionStore::load_from(settings.path().join("settings/session.json")).active_pr,
        Some(8)
    );
}
