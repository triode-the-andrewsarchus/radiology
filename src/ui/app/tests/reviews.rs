use super::*;
use std::sync::{atomic::Ordering, mpsc};
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
        started.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !cancel.load(Ordering::Relaxed) {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        retired.send(()).unwrap();
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
