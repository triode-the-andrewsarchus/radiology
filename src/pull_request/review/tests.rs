use super::*;

fn git(root: &Path, args: &[&str]) -> String {
    git_at(root, args).unwrap()
}

pub(crate) fn fixture() -> (tempfile::TempDir, PathBuf, ReviewSnapshot) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("attached");
    std::fs::create_dir(&root).unwrap();
    git(&root, &["init", "-b", "main"]);
    git(&root, &["config", "user.email", "test@example.invalid"]);
    git(&root, &["config", "user.name", "Test"]);
    std::fs::write(root.join("a file.rs"), "one\ntwo\nthree\n").unwrap();
    std::fs::write(root.join("removed.rs"), "old\n").unwrap();
    std::fs::write(root.join("renamed old.rs"), "keep\n").unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-m", "base"]);
    let base = git(&root, &["rev-parse", "HEAD"]);
    git(&root, &["checkout", "-b", "review"]);
    std::fs::write(root.join("a file.rs"), "one\nchanged\nthree\n").unwrap();
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
