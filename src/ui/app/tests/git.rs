use super::*;
use crate::{
    pull_request::{change::FileChange, diff::UnifiedDiffParser},
    repository::git_refresh::GitSnapshot,
    source::buffer::SourceIdentity,
};

fn snapshot(app: &RadiologyApp) -> GitSnapshot {
    GitSnapshot {
        root: app.repository_root.clone().unwrap(),
        branch: Some("refreshed".into()),
        changes: FileChange::parse_name_status(b"M\0a.rs\0").unwrap(),
        source: app
            .open_buffer
            .as_ref()
            .map(|buffer| SourceIdentity::new(buffer)),
        lines: Some(UnifiedDiffParser::parse("@@ -1 +1,7 @@\n")),
    }
}

#[test]
fn git_failure_and_cancel_preserve_accepted_reader_and_changes() {
    let (_dir, mut app) = opened_repository();
    app.branch_name = Some("accepted".into());
    app.pr_changes = FileChange::parse_name_status(b"M\0b.rs\0").unwrap();
    let buffer = app.open_buffer.clone().unwrap();
    let caret = (app.caret_line, app.caret_col);
    app.git_refresh.spawn(|_| Err("fixture failure".into()));
    settle(&mut app);
    assert_eq!(app.git_error.as_deref(), Some("fixture failure"));
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    assert_eq!(app.branch_name.as_deref(), Some("accepted"));
    assert_eq!(app.pr_changes[0].path, "b.rs");
    app.refresh_git_changes();
    app.cancel_git_refresh();
    settle(&mut app);
    assert!(!app.is_git_refreshing());
    assert_eq!(app.branch_name.as_deref(), Some("accepted"));
    assert_eq!(app.pr_changes[0].path, "b.rs");
    assert_eq!((app.caret_line, app.caret_col), caret);
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
}

#[test]
fn git_publication_checks_checkout_and_displayed_source_independently() {
    let (_dir, mut app) = opened_repository();
    app.open_file("a.rs");
    settle(&mut app);
    let result = snapshot(&app);
    app.open_file("b.rs");
    settle(&mut app);
    app.line_changes = None;
    let buffer = app.open_buffer.clone().unwrap();
    app.git_refresh.spawn(move |_| Ok(result));
    settle(&mut app);
    assert_eq!(app.branch_name.as_deref(), Some("refreshed"));
    assert_eq!(app.pr_changes[0].path, "a.rs");
    assert!(
        app.line_changes.is_none(),
        "old document hunks must not publish"
    );
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));

    let result = snapshot(&app);
    app.git_refresh.spawn(move |_| Ok(result));
    settle(&mut app);
    assert_eq!(app.line_changes.as_ref().unwrap().total_additions(), 7);

    let mut foreign = snapshot(&app);
    foreign.root.push("foreign");
    foreign.branch = Some("foreign".into());
    app.git_refresh.spawn(move |_| Ok(foreign));
    settle(&mut app);
    assert_eq!(app.branch_name.as_deref(), Some("refreshed"));
}

#[test]
fn repository_transition_retires_git_publication() {
    let (dir, mut app) = opened_repository();
    let result = snapshot(&app);
    let (release, gate) = std::sync::mpsc::channel();
    app.git_refresh.spawn(move |_| {
        gate.recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        Ok(result)
    });
    let next = dir.path().join("next");
    repository(&next);
    app.open_repository(&next);
    let _ = release.send(());
    settle(&mut app);
    assert_eq!(app.repository_root.as_deref(), Some(next.as_path()));
    assert_ne!(app.branch_name.as_deref(), Some("refreshed"));
    assert!(app.pr_changes.is_empty());
    assert!(app.git_error.is_none());
}
