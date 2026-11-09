use super::*;

pub(crate) fn git(root: &Path, args: &[&str]) -> String {
    String::from_utf8(
        ReviewControl::new(&AtomicBool::new(false))
            .git(root)
            .required(args)
            .unwrap(),
    )
    .unwrap()
    .trim()
    .to_owned()
}

pub(crate) fn fixture() -> (tempfile::TempDir, PathBuf, ReviewSnapshot) {
    fixture_with_sources("one\ntwo\nthree\n", "one\nchanged\nthree\n")
}

pub(crate) fn fixture_with_sources(
    before: &str,
    after: &str,
) -> (tempfile::TempDir, PathBuf, ReviewSnapshot) {
    fixture_with_tree_edits(before, after, |_| {}, |_| {})
}

pub(crate) fn fixture_with_tree_edits(
    before: &str,
    after: &str,
    edit_base: impl FnOnce(&Path),
    edit_head: impl FnOnce(&Path),
) -> (tempfile::TempDir, PathBuf, ReviewSnapshot) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("attached");
    std::fs::create_dir(&root).unwrap();
    git(&root, &["init", "-b", "main"]);
    git(&root, &["config", "user.email", "test@example.invalid"]);
    git(&root, &["config", "user.name", "Test"]);
    std::fs::write(root.join("a file.rs"), before).unwrap();
    std::fs::write(root.join("removed.rs"), "old\n").unwrap();
    std::fs::write(root.join("renamed old.rs"), "keep\n").unwrap();
    edit_base(&root);
    git(&root, &["add", "."]);
    git(&root, &["commit", "-m", "base"]);
    let base = git(&root, &["rev-parse", "HEAD"]);
    git(&root, &["checkout", "-b", "review"]);
    std::fs::write(root.join("a file.rs"), after).unwrap();
    std::fs::write(root.join("added.rs"), "new\n").unwrap();
    std::fs::remove_file(root.join("removed.rs")).unwrap();
    std::fs::rename(root.join("renamed old.rs"), root.join("renamed new.rs")).unwrap();
    std::fs::write(root.join("binary.dat"), [1, 0, 2]).unwrap();
    edit_head(&root);
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-m", "head"]);
    let head = git(&root, &["rev-parse", "HEAD"]);
    git(&root, &["checkout", "main"]);
    std::fs::write(root.join("a file.rs"), "checkout only\n").unwrap();
    git(&root, &["remote", "add", "origin", root.to_str().unwrap()]);
    let status = git(&root, &["status", "--porcelain"]);
    let summary: PullRequestSummary = serde_json::from_value(serde_json::json!({
        "number": 7, "title": "Review", "state": "OPEN", "headRefOid": head, "baseRefOid": base
    }))
    .unwrap();
    let review = ReviewSnapshot::prepare(&root, summary, &temp.path().join("cache")).unwrap();
    assert_eq!(git(&root, &["status", "--porcelain"]), status);
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), base);
    (temp, root, review)
}

pub(crate) fn indexing_fixture() -> (tempfile::TempDir, PathBuf, std::sync::Arc<ReviewSnapshot>) {
    let (owner, root, old) = fixture();
    git(&root, &["restore", "."]);
    git(&root, &["checkout", "review"]);
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='fixture'\nversion='0.1.0'\n",
    )
    .unwrap();
    git(&root, &["add", "Cargo.toml"]);
    git(&root, &["commit", "-m", "build inputs"]);
    let mut summary = old.summary.clone();
    summary.head_ref_oid = Some(git(&root, &["rev-parse", "HEAD"]));
    git(&root, &["checkout", "main"]);
    std::fs::write(root.join("a file.rs"), "checkout only\n").unwrap();
    let review = ReviewSnapshot::prepare(&root, summary, &owner.path().join("cache")).unwrap();
    (owner, root, std::sync::Arc::new(review))
}

#[test]
fn pinned_review_uses_merge_base_and_never_checkout_bytes() {
    let (_temp, root, review) = fixture();
    let base = review.base.clone();
    let status = git(&root, &["status", "--porcelain"]);
    assert_eq!(
        review
            .read("a file.rs", ReviewRevision::Before)
            .unwrap()
            .text,
        "one\ntwo\nthree\n"
    );
    assert_eq!(
        review
            .read("a file.rs", ReviewRevision::After)
            .unwrap()
            .text,
        "one\nchanged\nthree\n"
    );
    assert_eq!(
        review.diffs["a file.rs"].changed_lines(ReviewRevision::After),
        std::iter::once(2..3).collect::<Vec<_>>()
    );
    assert_eq!(review.diffs["a file.rs"].total_additions(), 1);
    assert!(!review.diffs["a file.rs"].is_binary);
    assert!(review.diffs["binary.dat"].is_binary);
    for change in &review.changes {
        for (revision, commit, actual) in [
            (ReviewRevision::Before, &review.base, &change.before_blob),
            (ReviewRevision::After, &review.head, &change.after_blob),
        ] {
            let expected = change.path_for_revision(revision).map_or_else(
                || "0".repeat(commit.len()),
                |path| git(&root, &["rev-parse", &format!("{commit}:{path}")]),
            );
            assert_eq!(*actual, expected, "{} {revision:?}", change.path);
        }
    }
    let deleted = review
        .resolve(SourceLocation::new("removed.rs", 0))
        .unwrap();
    assert_eq!(deleted.revision, Some(ReviewRevision::Before));
    assert!(review
        .resolve(SourceLocation::with_revision(
            "added.rs",
            0,
            Some(ReviewRevision::Before)
        ))
        .is_err());
    let renamed = review
        .resolve(SourceLocation::with_revision(
            "renamed new.rs",
            0,
            Some(ReviewRevision::Before),
        ))
        .unwrap();
    assert_eq!(renamed.path, "renamed old.rs");
    assert_eq!(
        review
            .read(&renamed.path, ReviewRevision::Before)
            .unwrap()
            .text,
        "keep\n"
    );
    assert!(review.read("../outside", ReviewRevision::After).is_err());
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), base);
    assert_eq!(git(&root, &["status", "--porcelain"]), status);
    assert_eq!(
        std::fs::read_to_string(root.join("a file.rs")).unwrap(),
        "checkout only\n"
    );
}

#[cfg(unix)]
#[test]
fn pinned_blob_reads_are_bounded_cancellable_and_verify_size_metadata() {
    use crate::repository::test_support::fake_git;
    let (temp, _root, review) = fixture();
    let cancel = AtomicBool::new(false);
    let script = fake_git(
        temp.path(),
        "if [ \"$6\" = -s ]; then printf '4\\n'; else printf 'raw\\n'; fi",
    );
    let git = GitRead {
        root: &review.object_dir,
        executable: &script,
        cancel: &cancel,
        deadline: Instant::now() + Duration::from_secs(5),
        lease: None,
    };
    let buffer = review
        .read_from_git("a file.rs", ReviewRevision::After, &git)
        .unwrap();
    assert_eq!(buffer.text, "raw\n");
    assert_eq!(buffer.revision, Some(ReviewRevision::After));
    assert!(review
        .read_cancellable(
            "a file.rs",
            ReviewRevision::Before,
            &AtomicBool::new(true),
            git.deadline
        )
        .err()
        .unwrap()
        .contains("cancelled"));
    for (body, timeout, expected) in [
        (
            "if [ \"$6\" = -s ]; then printf '4\\n'; else printf 'raw'; fi",
            Duration::from_secs(5),
            "does not match",
        ),
        (
            "printf '16777217\\n'",
            Duration::from_secs(5),
            "maximum allowed size",
        ),
        (
            "if [ \"$6\" = -s ]; then printf '4\\n'; else head -c 17000000 /dev/zero; fi",
            Duration::from_secs(5),
            "exceeds 16 MiB",
        ),
        ("exec sleep 30", Duration::from_millis(100), "timed out"),
    ] {
        let script = fake_git(temp.path(), body);
        let git = GitRead {
            executable: &script,
            deadline: Instant::now() + timeout,
            ..git
        };
        let error = review
            .read_from_git("a file.rs", ReviewRevision::Before, &git)
            .err()
            .unwrap();
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn file_manager_before_tree_reuses_verified_base_and_retains_pinned_ownership() {
    let (_owner, root, review) = fixture();
    let status = git(&root, &["status", "--porcelain"]);
    let cancel = AtomicBool::new(false);
    let (before, lease) = review.reveal_tree(ReviewRevision::Before, &cancel).unwrap();
    assert!(lease.is_some());
    assert_eq!(std::fs::read(before.join("removed.rs")).unwrap(), b"old\n");
    assert_eq!(
        std::fs::read(before.join("a file.rs")).unwrap(),
        b"one\ntwo\nthree\n"
    );
    assert!(!before.join("added.rs").exists());
    let (again, second_lease) = review.reveal_tree(ReviewRevision::Before, &cancel).unwrap();
    assert_eq!(again, before);
    assert!(second_lease.is_some());
    crate::desktop::prepare_reveal(&before, "removed.rs").unwrap();
    std::fs::write(before.join("removed.rs"), "tampered\n").unwrap();
    let (repaired, repaired_lease) = review.reveal_tree(ReviewRevision::Before, &cancel).unwrap();
    assert_ne!(repaired, before);
    assert!(repaired_lease.is_some());
    assert_eq!(
        std::fs::read(repaired.join("removed.rs")).unwrap(),
        b"old\n"
    );
    let (after, _) = review.reveal_tree(ReviewRevision::After, &cancel).unwrap();
    assert_eq!(after, review.head_root);
    assert!(!after.join("removed.rs").exists());
    assert_eq!(git(&root, &["status", "--porcelain"]), status);
    let stopped = AtomicBool::new(true);
    assert!(review
        .reveal_tree(ReviewRevision::Before, &stopped)
        .unwrap_err()
        .contains("cancelled"));
}

#[cfg(unix)]
#[test]
fn malformed_patch_does_not_replace_pinned_changes_or_counts() {
    let (owner, _, mut review) = fixture();
    let script = crate::repository::test_support::fake_git(owner.path(), "printf '@@ -x +1 @@\\n'");
    let cancel = AtomicBool::new(false);
    let control = ReviewControl {
        git: &script,
        ..ReviewControl::new(&cancel)
    };
    let changes = review.changes.clone();
    let diffs = review.diffs.clone();
    let error = review
        .add_change(changes[0].clone(), "", &control)
        .unwrap_err();
    assert!(
        error.contains("Unrecognized diff hunk header: @@ -x +1 @@"),
        "{error}"
    );
    assert_eq!(review.changes, changes);
    assert_eq!(review.diffs, diffs);
}

#[test]
fn per_file_diffs_exclude_children_of_reused_paths_for_renames_additions_and_deletions() {
    let before = format!(
        "fn demo() {{\r\n{}}}\r\n",
        (0..20)
            .map(|n| format!("    let item{n:02} = \"café\";\r\n"))
            .collect::<String>()
    );
    for edited in [false, true] {
        let after = if edited {
            before.replace("item10 = \"café\"", "item10 = \"edited 🦀\"")
        } else {
            before.clone()
        };
        let (_owner, root, review) = fixture_with_tree_edits(
            &before,
            &after,
            |root| {
                std::fs::create_dir(root.join("was directory")).unwrap();
                std::fs::write(root.join("was directory/child.rs"), "nested before\n").unwrap();
            },
            |root| {
                git(root, &["mv", "a file.rs", "moved café.rs"]);
                std::fs::create_dir(root.join("a file.rs")).unwrap();
                std::fs::write(
                    root.join("a file.rs/unrelated.rs"),
                    "unrelated rename child\n",
                )
                .unwrap();
                std::fs::create_dir(root.join("removed.rs")).unwrap();
                std::fs::write(
                    root.join("removed.rs/child.rs"),
                    "unrelated deletion child\n",
                )
                .unwrap();
                std::fs::remove_dir_all(root.join("was directory")).unwrap();
                std::fs::write(root.join("was directory"), "new file 🦀\r\n").unwrap();
            },
        );
        let change = review.change("moved café.rs").unwrap();
        assert!(matches!(change.status, FileChangeStatus::Renamed { .. }));
        let diff = &review.diffs["moved café.rs"];
        let expected: Vec<_> = edited.then_some(12..13).into_iter().collect();
        assert_eq!(
            diff.changed_lines(ReviewRevision::After),
            expected,
            "edited={edited}"
        );
        assert_eq!(diff.changed_lines(ReviewRevision::Before), expected);
        assert_eq!(review.diffs["a file.rs"], *diff);
        assert_eq!(
            (change.additions, change.deletions),
            if edited { (1, 1) } else { (0, 0) }
        );
        for line in [1, 3, 20] {
            assert_eq!(diff.map_line(line, ReviewRevision::Before), line);
        }
        for (path, additions, deletions) in [
            ("a file.rs/unrelated.rs", 1, 0),
            ("removed.rs", 0, 1),
            ("removed.rs/child.rs", 1, 0),
            ("was directory", 1, 0),
            ("was directory/child.rs", 0, 1),
        ] {
            let change = review.change(path).unwrap();
            assert_eq!(
                (change.additions, change.deletions),
                (additions, deletions),
                "{path}"
            );
            let diff = &review.diffs[path];
            assert_eq!(diff.total_additions(), additions);
            assert_eq!(diff.total_deletions(), deletions);
        }
        assert_eq!(
            review
                .read("moved café.rs", ReviewRevision::After)
                .unwrap()
                .data,
            after.as_bytes()
        );
        assert_eq!(
            review
                .read("a file.rs", ReviewRevision::Before)
                .unwrap()
                .data,
            before.as_bytes()
        );
        assert_eq!(git(&root, &["rev-parse", "HEAD"]), review.base);
        assert_eq!(
            std::fs::read_to_string(root.join("a file.rs")).unwrap(),
            "checkout only\n"
        );
    }
}

#[test]
fn fetches_pinned_commits_missing_from_the_attached_clone_and_names_unavailable_ids() {
    let (owner, origin, template) = fixture();
    let attached = owner.path().join("older clone");
    git(
        owner.path(),
        &[
            "clone",
            "--no-local",
            "--single-branch",
            "--branch",
            "main",
            origin.to_str().unwrap(),
            attached.to_str().unwrap(),
        ],
    );
    let cancel = AtomicBool::new(false);
    let control = ReviewControl::new(&cancel);
    assert!(control
        .git(&attached)
        .optional(&["cat-file", "-e", &template.head])
        .unwrap()
        .is_none());
    std::fs::write(attached.join("a file.rs"), "staged local\n").unwrap();
    git(&attached, &["add", "a file.rs"]);
    std::fs::write(attached.join("untracked.rs"), "local untracked\n").unwrap();
    let status = git(&attached, &["status", "--porcelain", "--branch"]);
    let refs = git(&attached, &["for-each-ref"]);
    let index = std::fs::read(attached.join(".git/index")).unwrap();
    let head = git(&attached, &["rev-parse", "HEAD"]);
    let snapshot = ReviewSnapshot::prepare(
        &attached,
        template.summary.clone(),
        &owner.path().join("older-cache"),
    )
    .unwrap();
    assert_eq!(snapshot.head, template.head);
    assert_eq!(
        snapshot
            .read("a file.rs", ReviewRevision::After)
            .unwrap()
            .text,
        "one\nchanged\nthree\n"
    );
    let missing = "f".repeat(template.head.len());
    for before in [true, false] {
        let mut summary = template.summary.clone();
        if before {
            summary.base_ref_oid = Some(missing.clone());
        } else {
            summary.head_ref_oid = Some(missing.clone());
        }
        let error = ReviewSnapshot::prepare(&attached, summary, &owner.path().join("older-cache"))
            .unwrap_err();
        assert!(error.contains(&missing), "before={before}: {error}");
    }
    assert!(control
        .git(&attached)
        .optional(&["cat-file", "-e", &template.head])
        .unwrap()
        .is_none());
    assert_eq!(git(&attached, &["rev-parse", "HEAD"]), head);
    assert_eq!(git(&attached, &["for-each-ref"]), refs);
    assert_eq!(std::fs::read(attached.join(".git/index")).unwrap(), index);
    assert_eq!(
        git(&attached, &["status", "--porcelain", "--branch"]),
        status
    );
    assert_eq!(
        std::fs::read(attached.join("a file.rs")).unwrap(),
        b"staged local\n"
    );
    assert_eq!(
        std::fs::read(attached.join("untracked.rs")).unwrap(),
        b"local untracked\n"
    );
    assert_eq!(
        snapshot
            .read("a file.rs", ReviewRevision::After)
            .unwrap()
            .text,
        "one\nchanged\nthree\n"
    );
}

#[test]
fn three_dot_changes_keep_exact_crlf_unicode_and_edited_rename_blobs_without_later_main_files() {
    let before = "one\r\ntwo café\r\nthree\r\nfour\r\n";
    let after = "one\r\nchanged 🦀\r\nthree\r\nFOUR\r\n";
    let renamed = (0..20)
        .map(|n| format!("stable line {n}\n"))
        .collect::<String>();
    let edited = format!("{renamed}extra line\n");
    let (owner, root, first) = fixture_with_tree_edits(
        before,
        after,
        |root| std::fs::write(root.join("renamed old.rs"), &renamed).unwrap(),
        |root| std::fs::write(root.join("renamed new.rs"), &edited).unwrap(),
    );
    git(&root, &["restore", "."]);
    std::fs::write(root.join("later-main.rs"), "not in the PR\n").unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-m", "main advanced independently"]);
    let main = git(&root, &["rev-parse", "HEAD"]);
    std::fs::write(root.join("a file.rs"), "checkout only\n").unwrap();
    let status = git(&root, &["status", "--porcelain"]);
    let mut summary = first.summary.clone();
    summary.base_ref_oid = Some(main.clone());
    let review = ReviewSnapshot::prepare(&root, summary, &owner.path().join("cache")).unwrap();
    assert_eq!(review.base, first.base);
    assert_eq!(review.head, first.head);
    assert_eq!(
        review
            .changes
            .iter()
            .map(|c| c.path.as_str())
            .collect::<std::collections::BTreeSet<_>>(),
        [
            "a file.rs",
            "added.rs",
            "binary.dat",
            "removed.rs",
            "renamed new.rs"
        ]
        .into_iter()
        .collect()
    );
    assert_eq!(
        review.diffs["a file.rs"].changed_lines(ReviewRevision::After),
        vec![2..3, 4..5]
    );
    assert_eq!(
        review.diffs["a file.rs"].changed_lines(ReviewRevision::Before),
        vec![2..3, 4..5]
    );
    let rename = review.change("renamed new.rs").unwrap();
    assert!(matches!(
        rename.status,
        FileChangeStatus::Renamed {
            similarity: 50..=99
        }
    ));
    assert_eq!(rename.old_path.as_deref(), Some("renamed old.rs"));
    for (path, status, additions, deletions, binary) in [
        ("a file.rs", FileChangeStatus::Modified, 2, 2, false),
        ("added.rs", FileChangeStatus::Added, 1, 0, false),
        ("removed.rs", FileChangeStatus::Deleted, 0, 1, false),
        ("renamed new.rs", rename.status.clone(), 1, 0, false),
        ("binary.dat", FileChangeStatus::Added, 0, 0, true),
    ] {
        let change = review.change(path).unwrap();
        assert_eq!(
            (
                change.status.clone(),
                change.additions,
                change.deletions,
                change.is_binary
            ),
            (status, additions, deletions, binary),
            "{path}"
        );
    }
    for (path, revision, bytes) in [
        ("a file.rs", ReviewRevision::Before, before),
        ("a file.rs", ReviewRevision::After, after),
        ("removed.rs", ReviewRevision::Before, "old\n"),
        ("renamed old.rs", ReviewRevision::Before, renamed.as_str()),
        ("renamed new.rs", ReviewRevision::After, edited.as_str()),
    ] {
        assert_eq!(review.read(path, revision).unwrap().data, bytes.as_bytes());
    }
    assert_eq!(
        review.diffs["renamed new.rs"].changed_lines(ReviewRevision::After),
        std::iter::once(21..22).collect::<Vec<_>>()
    );
    assert!(review.diffs["renamed new.rs"]
        .changed_lines(ReviewRevision::Before)
        .is_empty());
    assert_eq!(
        review.diffs["removed.rs"].changed_lines(ReviewRevision::Before),
        std::iter::once(1..2).collect::<Vec<_>>()
    );
    assert!(review.diffs["removed.rs"]
        .changed_lines(ReviewRevision::After)
        .is_empty());
    let cancel = AtomicBool::new(false);
    let (base_tree, _lease) = review.reveal_tree(ReviewRevision::Before, &cancel).unwrap();
    assert_eq!(
        std::fs::read(base_tree.join("removed.rs")).unwrap(),
        b"old\n"
    );
    assert_eq!(
        std::fs::read(base_tree.join("renamed old.rs")).unwrap(),
        renamed.as_bytes()
    );
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), main);
    assert_eq!(git(&root, &["status", "--porcelain"]), status);
    assert_eq!(
        std::fs::read(root.join("a file.rs")).unwrap(),
        b"checkout only\n"
    );
}
