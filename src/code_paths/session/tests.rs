use super::*;

fn fixture() -> (tempfile::TempDir, PathBuf, CodePathStore, CodePathSession) {
    let owner = tempfile::tempdir().unwrap();
    let root = owner.path().join("checkout");
    std::fs::create_dir(&root).unwrap();
    let store = CodePathStore::new(owner.path().join("settings/code-paths"));
    let session = CodePathSession::new(Some(store.clone()));
    (owner, root, store, session)
}

fn loaded_fixture() -> (tempfile::TempDir, PathBuf, CodePathStore, CodePathSession) {
    let (owner, root, store, mut session) = fixture();
    session.open(root.clone());
    session.flush(Duration::from_secs(5)).unwrap();
    (owner, root, store, session)
}

#[test]
fn import_delete_and_relaunch_publish_only_after_durable_completion() {
    let (_owner, root, store, mut session) = fixture();
    session.open(root.clone());
    assert!(session.is_busy());
    assert!(!session.import_text("Early\na.rs:boot:Early".into()));
    session.flush(Duration::from_secs(5)).unwrap();
    assert!(session.import_text("Boot\na.rs:boot:Enter".into()));
    assert!(session.paths().is_empty());
    assert!(!session.import_text("Second\na.rs:boot:Enter".into()));
    session.flush(Duration::from_secs(5)).unwrap();
    let saved = session.paths()[0].clone();
    assert_eq!(store.load(&root).unwrap()[0], *saved);
    let mut relaunch = CodePathSession::new(Some(store.clone()));
    relaunch.open(root.clone());
    relaunch.flush(Duration::from_secs(5)).unwrap();
    assert_eq!(relaunch.paths()[0].id, saved.id);
    assert!(session.delete(saved.id));
    assert_eq!(session.paths()[0].id, saved.id);
    session.flush(Duration::from_secs(5)).unwrap();
    assert!(session.paths().is_empty() && store.load(&root).unwrap().is_empty());
}

#[test]
fn scope_switch_never_cancels_an_accepted_save_or_races_a_reopened_load() {
    let (owner, root, store, mut session) = loaded_fixture();
    let other = owner.path().join("other");
    std::fs::create_dir(&other).unwrap();
    assert!(session.import_text("Boot\na.rs:boot:Enter".into()));
    session.open(other.clone());
    assert!(session.paths().is_empty());
    session.open(root.clone());
    session.flush(Duration::from_secs(5)).unwrap();
    assert_eq!(session.paths().len(), 1);
    assert_eq!(session.paths()[0].title, "Boot");
    assert_eq!(store.load(&root).unwrap().len(), 1);
    assert!(store.load(&other).unwrap().is_empty());
    session.open(other);
    session.flush(Duration::from_secs(5)).unwrap();
    assert!(session.paths().is_empty());
}

#[test]
fn malformed_import_and_corrupt_save_preserve_collection_and_file() {
    let (owner, root, store, mut session) = loaded_fixture();
    assert!(session.import_text("Boot\na.rs:boot:Enter".into()));
    session.flush(Duration::from_secs(5)).unwrap();
    let accepted = session.paths()[0].clone();
    assert!(session.import_text("Invalid\na.c:f:Valid prefix\nmalformed".into()));
    assert!(session
        .flush(Duration::from_secs(5))
        .unwrap_err()
        .contains("Line 3:"));
    assert_eq!(session.paths()[0], accepted);
    let archive = std::fs::read_dir(owner.path().join("settings/code-paths"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    std::fs::write(&archive, "broken archive").unwrap();
    assert!(session.delete(accepted.id));
    assert!(session.flush(Duration::from_secs(5)).is_err());
    assert_eq!(session.paths()[0], accepted);
    assert_eq!(std::fs::read_to_string(&archive).unwrap(), "broken archive");
    assert!(session.import_text("Must not overwrite\nb.rs:boot:Another trace".into()));
    assert!(session.flush(Duration::from_secs(5)).is_err());
    assert_eq!(session.paths(), std::slice::from_ref(&accepted));
    assert_eq!(std::fs::read_to_string(&archive).unwrap(), "broken archive");
    let mut corrupt = CodePathSession::new(Some(store.clone()));
    corrupt.open(root.clone());
    assert!(corrupt.flush(Duration::from_secs(5)).is_err());
    assert!(corrupt.paths().is_empty() && corrupt.error().is_some());
    assert!(corrupt.import_text("Blocked import\nb.rs:boot:Keep corrupt data".into()));
    assert!(corrupt.flush(Duration::from_secs(5)).is_err());
    assert!(corrupt.paths().is_empty());
    assert_eq!(std::fs::read_to_string(&archive).unwrap(), "broken archive");
    // Repair is explicit; retry a load without replacing source data or records.
    std::fs::remove_file(&archive).unwrap();
    store.save(&root, &[(*accepted).clone()]).unwrap();
    session.reload();
    session.flush(Duration::from_secs(5)).unwrap_err(); // Failed write still reportable.
    assert!(session.import_text("Recovery\nb.rs:boot:Resume".into()));
    session.flush(Duration::from_secs(5)).unwrap();
    assert_eq!(session.paths().len(), 2);
}

#[test]
fn stale_scope_results_cannot_publish_rows_and_background_write_errors_remain_visible() {
    let (owner, root, _store, mut session) = loaded_fixture();
    let other = owner.path().join("other");
    std::fs::create_dir(&other).unwrap();
    assert!(session.import_text("Boot\na.rs:boot:Enter".into()));
    // Replace the operation with a deterministically queued old-scope failure.
    let (done, receipt) = std::sync::mpsc::channel();
    session.job.spawn_with_sender(move |_, sender| {
        assert!(sender.send(Err("write refused".into())).is_ok());
        done.send(()).unwrap();
    });
    receipt.recv_timeout(Duration::from_secs(5)).unwrap();
    session.open(other);
    assert!(session
        .flush(Duration::from_secs(5))
        .unwrap_err()
        .contains("write refused"));
    assert!(session.paths().is_empty());
    assert!(session.error().unwrap().contains(root.to_str().unwrap()));
}

#[test]
fn closing_retains_the_receipt_until_flush_and_no_storage_is_a_visible_failure() {
    let (_owner, root, store, mut session) = loaded_fixture();
    assert!(session.import_text("Boot\na.rs:boot:Enter".into()));
    session.close();
    assert!(!session.import_text("No repository\na.rs:boot:Cannot import".into()));
    session.flush(Duration::from_secs(5)).unwrap();
    assert!(session.paths().is_empty());
    assert_eq!(store.load(&root).unwrap().len(), 1);
    let mut missing = CodePathSession::new(None);
    missing.open(root);
    assert!(!missing.is_busy());
    assert!(!missing.import_text("Boot\na.rs:boot:Enter".into()));
    assert!(missing.error().is_some());
}
