use super::*;
use crate::pull_request::test_support::wait_for_child;
use crate::repository::test_support::fake_git;
use std::{
    sync::{atomic::Ordering, mpsc},
    thread,
};

#[test]
fn writer_lease_waits_with_cancellation_and_deadline_and_keeps_its_stable_inode() {
    use std::os::unix::fs::MetadataExt;
    let owner = tempfile::tempdir().unwrap();
    let cancel = AtomicBool::new(false);
    let control = ReviewControl::new(&cancel);
    let first = StoreLease::acquire(owner.path(), &control).unwrap();
    let inode = first.file().metadata().unwrap().ino();
    thread::scope(|scope| {
        let (began, started) = mpsc::channel();
        let path = owner.path();
        let worker = scope.spawn(move || {
            began.send(()).unwrap();
            StoreLease::acquire(path, &control).err().unwrap()
        });
        started.recv_timeout(Duration::from_secs(2)).unwrap();
        thread::sleep(Duration::from_millis(30));
        cancel.store(true, Ordering::Relaxed);
        assert!(worker.join().unwrap().contains("cancelled"));
    });
    cancel.store(false, Ordering::Relaxed);
    let short = ReviewControl {
        deadline: Instant::now() + Duration::from_millis(50),
        ..control
    };
    assert!(StoreLease::acquire(owner.path(), &short)
        .err()
        .unwrap()
        .contains("timed out"));
    drop(first);
    let fresh = StoreLease::acquire(owner.path(), &control).unwrap();
    assert_eq!(fresh.file().metadata().unwrap().ino(), inode);
}

#[test]
fn cancelled_preparer_releases_its_writer_and_successor_repairs_a_locked_missing_tree() {
    let (owner, root, accepted) = super::tests::fixture();
    let cache = owner.path().join("shared-cache");
    let script = fake_git(
        owner.path(),
        r#"
case "$5:$6" in
worktree:add)
 /usr/bin/git "$@" || exit
 printf initializing > "worktrees/${8##*/}/locked"
 printf '%s' "$$" > paused.pid
 exec /bin/sleep 30 ;;
esac
exec /usr/bin/git "$@"
"#,
    );
    let cancel = AtomicBool::new(false);
    let first = ReviewControl {
        git: &script,
        ..ReviewControl::new(&cancel)
    };
    let fresh_cancel = AtomicBool::new(false);
    let fresh = ReviewControl {
        deadline: Instant::now() + Duration::from_secs(5),
        ..ReviewControl::new(&fresh_cancel)
    };
    thread::scope(|scope| {
        let old = scope.spawn(|| {
            ReviewSnapshot::prepare_cancellable(&root, accepted.summary.clone(), &cache, &first)
        });
        let pid = wait_for_child(&cache.join("review.git/paused.pid"));
        let (sender, completion) = mpsc::channel();
        let next_root = &root;
        let next_cache = &cache;
        let next_summary = accepted.summary.clone();
        let next = scope.spawn(move || {
            sender
                .send(ReviewSnapshot::prepare_cancellable(
                    next_root,
                    next_summary,
                    next_cache,
                    &fresh,
                ))
                .unwrap()
        });
        assert!(
            matches!(
                completion.recv_timeout(Duration::from_millis(100)),
                Err(mpsc::RecvTimeoutError::Timeout)
            ),
            "replacement cannot mutate a store with a live writer"
        );
        cancel.store(true, Ordering::Relaxed);
        assert!(old.join().unwrap().unwrap_err().contains("cancelled"));
        assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
        let recovered = completion
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            .unwrap();
        next.join().unwrap();
        assert_eq!(recovered.head, accepted.head);
        assert_eq!(
            recovered
                .read("a file.rs", ReviewRevision::After)
                .unwrap()
                .text,
            "one\nchanged\nthree\n"
        );
        let listing = fresh
            .git(&recovered.object_dir)
            .required(&["worktree", "list", "--porcelain"])
            .unwrap();
        assert!(!String::from_utf8_lossy(&listing).contains("\nlocked"));
        assert_eq!(
            std::fs::read_dir(cache.join("revisions").join(&accepted.head))
                .unwrap()
                .count(),
            1
        );
    });
    assert_eq!(
        accepted
            .read("a file.rs", ReviewRevision::After)
            .unwrap()
            .text,
        "one\nchanged\nthree\n"
    );
    println!("PR store: replacement waits for live writer, Cancel reaps child, locked missing worktree repairs, pinned retry succeeds");
}

#[test]
fn retries_fetch_new_objects_without_rewriting_or_deleting_legacy_git_locks() {
    let (owner, root, accepted) = super::tests::fixture();
    let cache = owner.path().join("legacy-locks");
    let cancel = AtomicBool::new(false);
    let control = ReviewControl::new(&cancel);
    let object_dir = {
        let lease = StoreLease::acquire(&cache, &control).unwrap();
        let locked = ReviewControl {
            lease: Some(lease.file()),
            ..control
        };
        GitStore::initialize(lease.cache(), &locked).unwrap()
    };
    for name in ["config.lock", "FETCH_HEAD.lock", "FETCH_HEAD"] {
        std::fs::write(object_dir.join(name), format!("preserve legacy {name}\n")).unwrap();
    }
    let snapshot = ReviewSnapshot::prepare(&root, accepted.summary.clone(), &cache).unwrap();
    assert_eq!(snapshot.head, accepted.head);
    assert_eq!(snapshot.base, accepted.base);
    for name in ["config.lock", "FETCH_HEAD.lock", "FETCH_HEAD"] {
        assert_eq!(
            std::fs::read_to_string(object_dir.join(name)).unwrap(),
            format!("preserve legacy {name}\n")
        );
    }
    assert_eq!(snapshot.changes.len(), accepted.changes.len());
    println!("PR store: real local fetch succeeds with legacy config/FETCH_HEAD locks and preserves their bytes");
}

#[test]
fn interrupted_legacy_initialization_is_preserved_while_a_complete_store_is_published() {
    let (owner, root, accepted) = super::tests::fixture();
    let cache = owner.path().join("interrupted-init");
    std::fs::create_dir_all(cache.join("review.git/objects")).unwrap();
    std::fs::write(
        cache.join("review.git/objects/sentinel"),
        "preserve unknown data",
    )
    .unwrap();
    let recovered = ReviewSnapshot::prepare(&root, accepted.summary.clone(), &cache).unwrap();
    assert!(recovered.object_dir.join("HEAD").is_file());
    let quarantine = std::fs::read_dir(&cache)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("review-recovery-")
        })
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(quarantine.join("review.git/objects/sentinel")).unwrap(),
        "preserve unknown data"
    );
    assert!(!std::fs::read_dir(&cache).unwrap().any(|entry| entry
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with("review-init-")));
}

#[test]
fn fallback_fetch_uses_a_request_scoped_ref_and_rejects_a_moved_head_without_fetch_head() {
    let (owner, root, accepted) = super::tests::fixture();
    crate::repository::test_support::git(
        &root,
        &["update-ref", "refs/pull/7/head", &accepted.head],
    );
    let script = format!(
        r#"
case "$*" in
 *"fetch --no-tags"*" {}") exit 1 ;;
esac
exec /usr/bin/git "$@"
"#,
        accepted.head
    );
    let git = fake_git(owner.path(), &script);
    let cancel = AtomicBool::new(false);
    let control = ReviewControl {
        git: &git,
        ..ReviewControl::new(&cancel)
    };
    let cache = owner.path().join("fallback");
    let snapshot =
        ReviewSnapshot::prepare_cancellable(&root, accepted.summary.clone(), &cache, &control)
            .unwrap();
    assert_eq!(snapshot.head, accepted.head);
    assert!(!snapshot.object_dir.join("FETCH_HEAD").exists());
    assert!(control
        .git(&snapshot.object_dir)
        .required(&["for-each-ref", "refs/radiology/requests/"])
        .unwrap()
        .is_empty());
    // The direct SHA remains refused, while fallback now points at a different commit.
    crate::repository::test_support::git(
        &root,
        &["update-ref", "refs/pull/7/head", &accepted.base],
    );
    let other_cache = owner.path().join("moved-fallback");
    let error = ReviewSnapshot::prepare_cancellable(
        &root,
        accepted.summary.clone(),
        &other_cache,
        &control,
    )
    .unwrap_err();
    assert!(error.contains("moved during fetch"), "{error}");
    assert!(snapshot.head_root.is_dir());
    assert_eq!(
        snapshot
            .read("a file.rs", ReviewRevision::After)
            .unwrap()
            .text,
        "one\nchanged\nthree\n"
    );
    println!("PR fallback: request-scoped ref preserves exact pinned head, rejects movement and never writes FETCH_HEAD");
}
