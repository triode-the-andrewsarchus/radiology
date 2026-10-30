use super::*;
#[cfg(unix)]
use crate::repository::test_support::fake_git;
use crate::repository::test_support::{git, repository};

#[test]
fn snapshots_preserve_literal_paths_counts_binary_files_and_displayed_source_identity() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    repository(root);
    let path = "a café\tline\n.rs";
    std::fs::write(root.join(path), "old\n").unwrap();
    std::fs::write(root.join("removed.rs"), "gone\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "initial"]);
    std::fs::write(root.join(path), "new\nmore\n").unwrap();
    std::fs::remove_file(root.join("removed.rs")).unwrap();
    std::fs::write(root.join("new file é.rs"), "one\ntwo\n").unwrap();
    std::fs::write(root.join("binary.bin"), [0, 1, 2]).unwrap();
    let buffer = Arc::new(SourceBuffer::read(&root.join(path), path, None).unwrap());
    let snapshot = read(root.into(), Some(buffer.clone()), &AtomicBool::new(false)).unwrap();
    assert!(snapshot.branch.is_some());
    assert!(snapshot.source.as_ref().unwrap().matches(&buffer));
    let changed = snapshot
        .changes
        .iter()
        .find(|change| change.path == path)
        .unwrap();
    assert_eq!((changed.additions, changed.deletions), (2, 1));
    assert_eq!(snapshot.lines.as_ref().unwrap().total_additions(), 2);
    assert!(snapshot
        .changes
        .iter()
        .any(|change| change.path == "removed.rs" && change.status == FileChangeStatus::Deleted));
    assert!(snapshot
        .changes
        .iter()
        .any(|change| change.path == "new file é.rs" && change.additions == 2));
    assert!(snapshot
        .changes
        .iter()
        .any(|change| change.path == "binary.bin" && change.is_binary));
    std::fs::write(root.join(path), "different bytes\n").unwrap();
    let snapshot = read(root.into(), Some(buffer), &AtomicBool::new(false)).unwrap();
    assert!(
        snapshot.lines.is_none(),
        "fresh hunks must not annotate stale displayed bytes"
    );
}

#[test]
fn unborn_repositories_and_plain_folders_remain_browsable() {
    let dir = tempfile::tempdir().unwrap();
    assert!(read(dir.path().into(), None, &AtomicBool::new(false))
        .unwrap()
        .changes
        .is_empty());
    repository(dir.path());
    std::fs::write(dir.path().join("staged.rs"), "one\n").unwrap();
    git(dir.path(), &["add", "staged.rs"]);
    std::fs::write(dir.path().join("untracked.rs"), "two\n").unwrap();
    let snapshot = read(dir.path().into(), None, &AtomicBool::new(false)).unwrap();
    assert_eq!(snapshot.changes.len(), 2);
    assert!(snapshot
        .changes
        .iter()
        .all(|change| change.status == FileChangeStatus::Added && change.additions == 1));
}

#[test]
fn nul_record_parsing_handles_renames_and_rejects_partial_records() {
    let changes =
        FileChange::parse_name_status(b"R100\0old name.rs\0new\tname.rs\0M\0file\nname.rs\0")
            .unwrap();
    assert_eq!(changes[0].old_path.as_deref(), Some("old name.rs"));
    assert_eq!(changes[0].path, "new\tname.rs");
    assert_eq!(changes[1].path, "file\nname.rs");
    assert!(FileChange::parse_name_status(b"R100\0missing.rs\0").is_err());
    assert!(FileChange::parse_name_status(b"M\0partial.rs").is_err());
    assert!(paths(b"partial.rs").is_err());
    assert!(numstat(b"1\t0\tpartial.rs").is_err());
    let stats = numstat(b"2\t1\t\0old name.rs\0new\tname.rs\0-\t-\tbinary.bin\0").unwrap();
    assert_eq!(stats["new\tname.rs"], (2, 1, false));
    assert_eq!(stats["binary.bin"], (0, 0, true));
}

#[test]
fn new_source_hunks_cover_the_exact_displayed_untracked_file() {
    let dir = tempfile::tempdir().unwrap();
    repository(dir.path());
    git(dir.path(), &["commit", "--allow-empty", "-qm", "initial"]);
    let path = "new.rs";
    std::fs::write(dir.path().join(path), "one\ntwo\n").unwrap();
    let buffer = Arc::new(SourceBuffer::read(&dir.path().join(path), path, None).unwrap());
    let snapshot = read(dir.path().into(), Some(buffer), &AtomicBool::new(false)).unwrap();
    assert_eq!(snapshot.lines.unwrap().total_additions(), 2);
    let bare = tempfile::tempdir().unwrap();
    git(bare.path(), &["init", "--bare", "-q"]);
    assert!(read(bare.path().into(), None, &AtomicBool::new(false))
        .unwrap()
        .changes
        .is_empty());
}

#[cfg(unix)]
#[test]
fn git_refresh_rejects_retired_work_and_reaps_at_its_deadline() {
    let dir = tempfile::tempdir().unwrap();
    let script = fake_git(
        dir.path(),
        "echo $$ > child-pid; touch ready; exec sleep 30",
    );
    let error = read_with(
        dir.path().into(),
        None,
        &AtomicBool::new(true),
        &script,
        Duration::from_secs(10),
    )
    .err()
    .unwrap();
    assert!(error.contains("cancelled"));
    assert!(!dir.path().join("child-pid").exists());
    let started = Instant::now();
    let error = read_with(
        dir.path().into(),
        None,
        &AtomicBool::new(false),
        &script,
        Duration::from_millis(100),
    )
    .err()
    .unwrap();
    assert!(error.contains("timed out"), "{error}");
    assert!(started.elapsed() < Duration::from_secs(2));
    let pid: i32 = std::fs::read_to_string(dir.path().join("child-pid"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert_eq!(unsafe { libc::kill(pid, 0) }, -1, "deadline must reap Git");
}

#[cfg(unix)]
#[test]
fn oversized_git_stdout_is_rejected_instead_of_parsing_truncated_records() {
    let dir = tempfile::tempdir().unwrap();
    let script = fake_git(dir.path(), "head -c 17000000 /dev/zero; exec sleep 30");
    let started = Instant::now();
    let error = read_with(
        dir.path().into(),
        None,
        &AtomicBool::new(false),
        &script,
        Duration::from_secs(10),
    )
    .err()
    .unwrap();
    assert!(error.contains("exceeds 16 MiB"), "{error}");
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[cfg(unix)]
#[test]
fn moving_head_rejects_the_entire_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let script = fake_git(
        dir.path(),
        r#"
case "$5:$6" in
  rev-parse:--is-inside-work-tree) printf 'true\n' ;;
  symbolic-ref:*) printf 'main\n' ;;
  rev-parse:--verify)
    if [ -f head-read ]; then printf 'second\n'; else touch head-read; printf 'first\n'; fi ;;
esac
"#,
    );
    let error = read_with(
        dir.path().into(),
        None,
        &AtomicBool::new(false),
        &script,
        Duration::from_secs(5),
    )
    .err()
    .unwrap();
    assert!(error.contains("HEAD changed"), "{error}");
}
