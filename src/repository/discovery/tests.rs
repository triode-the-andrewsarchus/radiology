use super::*;

#[test]
fn test_excluded_paths() {
    assert!(RepositoryDiscovery::is_excluded_path("target/debug/build"));
    assert!(RepositoryDiscovery::is_excluded_path(".git/config"));
    assert!(RepositoryDiscovery::is_excluded_path(
        "node_modules/pkg/index.js"
    ));
    assert!(!RepositoryDiscovery::is_excluded_path(
        "src/target_detector.rs"
    ));
    assert!(!RepositoryDiscovery::is_excluded_path("src/main.rs"));
}

#[test]
fn real_git_preserves_root_whitespace_and_worktree_literal_paths() {
    use crate::repository::test_support::{git, repository};
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("repo café \n");
    std::fs::create_dir(&root).unwrap();
    repository(&root);
    std::fs::create_dir(root.join("nested")).unwrap();
    std::fs::write(root.join("nested/a\tline\n.rs"), "fn one() {}\n").unwrap();
    std::fs::write(root.join("deleted.rs"), "gone\n").unwrap();
    std::fs::write(root.join(".gitignore"), "ignored.rs\n").unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-qm", "initial"]);
    std::fs::remove_file(root.join("deleted.rs")).unwrap();
    std::fs::write(root.join("ignored.rs"), "ignored\n").unwrap();
    std::fs::write(root.join("new café.rs"), "new\n").unwrap();
    let cancel = AtomicBool::new(false);
    let found =
        RepositoryDiscovery::discover(&root.join("nested"), &SourceFilters::default(), &cancel)
            .unwrap();
    assert_eq!(found.root, root);
    assert_eq!(
        found.files,
        [".gitignore", "nested/a\tline\n.rs", "new café.rs"].map(RepositoryFile::new)
    );
    let worktree = temp.path().join("worktree");
    git(
        &root,
        &[
            "worktree",
            "add",
            "--detach",
            worktree.to_str().unwrap(),
            "HEAD",
        ],
    );
    let found =
        RepositoryDiscovery::discover(&worktree.join("nested"), &SourceFilters::default(), &cancel)
            .unwrap();
    assert_eq!(found.root, worktree);
    assert!(found.root.join(".git").is_file());
    assert!(found
        .files
        .iter()
        .any(|file| file.path == "nested/a\tline\n.rs"));
}

#[test]
fn missing_git_uses_walk_exclusions_and_parent_marker_without_following_links() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    for directory in ["nested", ".git", "target", "node_modules", "build"] {
        std::fs::create_dir(root.join(directory)).unwrap();
        std::fs::write(root.join(directory).join("source.rs"), "fn x() {}\n").unwrap();
    }
    std::fs::write(root.join("a.rs"), "fn alpha() {}\n").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(root.join("nested"), root.join("link")).unwrap();
    let found = RepositoryDiscovery::discover_with(
        &root.join("nested"),
        &SourceFilters::default(),
        &AtomicBool::new(false),
        &root.join("missing-git"),
        Duration::from_secs(5),
    )
    .unwrap();
    assert_eq!(found.root, root);
    assert_eq!(
        found.files,
        ["a.rs", "nested/source.rs"].map(RepositoryFile::new)
    );
    assert!(RepositoryDiscovery::discover(
        &root.join("missing"),
        &SourceFilters::default(),
        &AtomicBool::new(false)
    )
    .is_err());
    assert!(RepositoryDiscovery::discover(
        &root.join("a.rs"),
        &SourceFilters::default(),
        &AtomicBool::new(false)
    )
    .is_err());
}

#[cfg(unix)]
#[test]
fn cancelled_deadline_oversized_and_partial_git_reads_never_fall_back_to_success() {
    use crate::repository::test_support::fake_git;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    for (body, cancelled, timeout, message) in [
        (
            "touch launched; exec sleep 30",
            true,
            Duration::from_secs(5),
            "cancelled",
        ),
        (
            "exec sleep 30",
            false,
            Duration::from_millis(100),
            "timed out",
        ),
        (
            "head -c 17000000 /dev/zero",
            false,
            Duration::from_secs(5),
            "exceeds 16 MiB",
        ),
        (
            "if [ \"$5\" = rev-parse ]; then exit 1; fi; printf 'partial.rs'",
            false,
            Duration::from_secs(5),
            "Incomplete NUL",
        ),
        (
            "if [ \"$5\" = rev-parse ]; then exit 1; fi; printf '../escape.rs\\0'",
            false,
            Duration::from_secs(5),
            "Invalid Git source path",
        ),
    ] {
        let script = fake_git(root, body);
        let error = RepositoryDiscovery::discover_with(
            root,
            &SourceFilters::default(),
            &AtomicBool::new(cancelled),
            &script,
            timeout,
        )
        .err()
        .unwrap();
        assert!(error.to_string().contains(message), "{error}");
        assert!(
            !root.join("launched").exists(),
            "retired work must not launch"
        );
    }
}
