use super::*;

fn git(root: &Path, args: &[&str]) -> String {
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
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("attached");
    std::fs::create_dir(&root).unwrap();
    git(&root, &["init", "-b", "main"]);
    git(&root, &["config", "user.email", "test@example.invalid"]);
    git(&root, &["config", "user.name", "Test"]);
    std::fs::write(root.join("a file.rs"), before).unwrap();
    std::fs::write(root.join("removed.rs"), "old\n").unwrap();
    std::fs::write(root.join("renamed old.rs"), "keep\n").unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-m", "base"]);
    let base = git(&root, &["rev-parse", "HEAD"]);
    git(&root, &["checkout", "-b", "review"]);
    std::fs::write(root.join("a file.rs"), after).unwrap();
    std::fs::write(root.join("added.rs"), "new\n").unwrap();
    std::fs::remove_file(root.join("removed.rs")).unwrap();
    std::fs::rename(root.join("renamed old.rs"), root.join("renamed new.rs")).unwrap();
    std::fs::write(root.join("binary.dat"), [1, 0, 2]).unwrap();
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
