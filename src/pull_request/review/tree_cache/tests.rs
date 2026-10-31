use super::*;
use crate::pull_request::{github::PullRequestSummary, review::ReviewSnapshot};
use crate::repository::test_support::git;
use crate::source::ReviewRevision;

fn text(root: &Path, arguments: &[&str]) -> String {
    String::from_utf8(
        ReviewControl::new(&AtomicBool::new(false))
            .git(root)
            .required(arguments)
            .unwrap(),
    )
    .unwrap()
    .trim()
    .into()
}

fn new_head(root: &Path, value: &str, template: &PullRequestSummary) -> PullRequestSummary {
    git(root, &["restore", "."]);
    git(root, &["checkout", "review"]);
    fs::write(root.join("a file.rs"), value).unwrap();
    git(root, &["add", "a file.rs"]);
    git(root, &["commit", "-m", "next"]);
    let head = text(root, &["rev-parse", "HEAD"]);
    git(root, &["checkout", "main"]);
    fs::write(root.join("a file.rs"), "checkout only\n").unwrap();
    let mut summary = template.clone();
    summary.head_ref_oid = Some(head);
    summary
}

fn paths(cache: &Path) -> Vec<PathBuf> {
    let revisions = cache.join("revisions");
    walkdir::WalkDir::new(&revisions)
        .max_depth(2)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.depth() == 2
                && entry.file_type().is_dir()
                && store::owned_tree(entry.path(), &revisions)
        })
        .map(|entry| entry.into_path())
        .collect()
}

#[test]
fn exact_reuse_rejects_changes_hidden_from_the_git_index_and_preserves_leased_old_bytes() {
    let (owner, root, first) = super::super::tests::fixture();
    let cache = owner.path().join("cache");
    let second = ReviewSnapshot::prepare(&root, first.summary.clone(), &cache).unwrap();
    assert_eq!(first.head_root, second.head_root);
    assert_eq!(paths(&cache).len(), 1);
    let clone = first.clone();
    assert!(Arc::ptr_eq(
        first._tree_lease.as_ref().unwrap(),
        clone._tree_lease.as_ref().unwrap()
    ));
    git(
        &first.head_root,
        &["update-index", "--assume-unchanged", "--", "a file.rs"],
    );
    fs::write(first.head_root.join("a file.rs"), "one\nALTERED\nthree\n").unwrap();
    git(&first.head_root, &["diff-files", "--quiet"]);
    let fresh = ReviewSnapshot::prepare(&root, first.summary.clone(), &cache).unwrap();
    assert_ne!(fresh.head_root, first.head_root);
    assert_eq!(
        fs::read_to_string(fresh.head_root.join("a file.rs")).unwrap(),
        "one\nchanged\nthree\n"
    );
    assert_eq!(
        fs::read_to_string(first.head_root.join("a file.rs")).unwrap(),
        "one\nALTERED\nthree\n"
    );
    assert_eq!(
        first.read("a file.rs", ReviewRevision::After).unwrap().text,
        "one\nchanged\nthree\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("a file.rs")).unwrap(),
        "checkout only\n"
    );
}

#[test]
fn active_snapshots_and_clones_survive_pressure_then_release_converges_to_three_lru_trees() {
    let (owner, root, first) = super::super::tests::fixture();
    let cache = owner.path().join("cache");
    let template = first.summary.clone();
    let mut snapshots = vec![first];
    for number in 0..4 {
        let summary = new_head(&root, &format!("version {number}\n"), &template);
        snapshots.push(ReviewSnapshot::prepare(&root, summary, &cache).unwrap());
    }
    let retained: Vec<_> = snapshots
        .iter()
        .map(|snapshot| snapshot.head_root.clone())
        .collect();
    assert_eq!(
        paths(&cache).len(),
        5,
        "active readers can exceed the idle cache budget"
    );
    assert!(retained.iter().all(|path| path.exists()));
    let last = snapshots.pop().unwrap();
    let clone = snapshots[0].clone();
    drop(snapshots);
    // Drop-time maintenance may defer when another maintenance writer is busy.
    // The next preparation guarantees a serialized sweep of all released leases.
    let reuse = ReviewSnapshot::prepare(&root, last.summary.clone(), &cache).unwrap();
    assert!(paths(&cache).len() <= 3);
    assert!(
        retained[0].exists(),
        "a snapshot clone still protects the oldest tree"
    );
    assert!(last.head_root.exists());
    assert_eq!(reuse.head_root, last.head_root);
    drop(clone);
    // Releasing a protected old tree requests maintenance; the next accepted
    // preparation also guarantees a bounded sweep under the writer lease.
    let _ = ReviewSnapshot::prepare(&root, last.summary.clone(), &cache).unwrap();
    assert!(paths(&cache).len() <= 3);
    assert!(
        !retained[1].exists(),
        "least-recent unleased content is evicted"
    );
}

#[test]
fn type_mode_inventory_and_head_corruption_cannot_be_reused_or_overwrite_a_live_tree() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let (owner, root, initial) = super::super::tests::fixture();
    let cache = owner.path().join("cache");
    let template = initial.summary.clone();
    let outside = owner.path().join("outside");
    fs::write(&outside, "outside bytes").unwrap();
    let mut previous = initial;
    for kind in ["mode", "link", "extra", "missing", "head"] {
        match kind {
            "mode" => fs::set_permissions(
                previous.head_root.join("a file.rs"),
                fs::Permissions::from_mode(0o755),
            )
            .unwrap(),
            "link" => {
                fs::remove_file(previous.head_root.join("a file.rs")).unwrap();
                symlink(&outside, previous.head_root.join("a file.rs")).unwrap();
            }
            "extra" => fs::write(previous.head_root.join("untracked.rs"), "extra").unwrap(),
            "missing" => fs::remove_file(previous.head_root.join("a file.rs")).unwrap(),
            "head" => git(
                &previous.head_root,
                &["checkout", "--detach", &previous.base],
            ),
            _ => unreachable!(),
        }
        let fresh = ReviewSnapshot::prepare(&root, template.clone(), &cache).unwrap();
        assert_ne!(fresh.head_root, previous.head_root, "{kind}");
        assert!(
            previous.head_root.exists(),
            "active {kind} tree must be retained"
        );
        assert_eq!(
            fresh.read("a file.rs", ReviewRevision::After).unwrap().text,
            "one\nchanged\nthree\n"
        );
        previous = fresh;
    }
    assert_eq!(fs::read_to_string(outside).unwrap(), "outside bytes");
}

#[test]
fn uncommitted_stage_is_removed_and_unrecognized_metadata_is_preserved_without_reuse() {
    let (owner, root, first) = super::super::tests::fixture();
    let cache = owner.path().join("cache");
    let cancel = AtomicBool::new(false);
    let control = ReviewControl::new(&cancel);
    let mut state = scan(&cache, &control).unwrap().pop().unwrap();
    let metadata = state.metadata(&cache);
    fs::write(metadata.join("state.json"), "{unrecognized").unwrap();
    let fresh = ReviewSnapshot::prepare(&root, first.summary.clone(), &cache).unwrap();
    assert_ne!(fresh.head_root, first.head_root);
    assert_eq!(
        fs::read_to_string(metadata.join("state.json")).unwrap(),
        "{unrecognized"
    );
    state = scan(&cache, &control).unwrap().pop().unwrap();
    state.complete = false;
    write_state(&state, &cache).unwrap();
    let writer = StoreLease::acquire(&cache, &control).unwrap();
    let control = ReviewControl {
        lease: Some(writer.file()),
        ..control
    };
    let store = GitStore(control.git(&fresh.object_dir));
    let pending = PreparedTree::prepare(&store, &fresh.head, &control).unwrap();
    let pending_path = pending.path.clone();
    let pending_metadata = pending.state.metadata(&cache);
    drop(pending);
    assert!(!pending_path.exists());
    assert!(!pending_metadata.exists());
    assert!(first.head_root.exists());
    assert!(fresh.head_root.exists());
}

#[test]
fn checkout_ignores_smudge_hooks_and_eol_attributes_and_verifies_modes_links_and_gitlinks() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let (owner, root, first) = super::super::tests::fixture();
    git(&root, &["restore", "."]);
    git(&root, &["checkout", "review"]);
    fs::write(
        root.join(".gitattributes"),
        "*.txt text eol=crlf\n*.rs filter=fixture\n",
    )
    .unwrap();
    fs::write(root.join("raw.txt"), "raw LF\n").unwrap();
    fs::create_dir(root.join("nested")).unwrap();
    fs::write(root.join("nested/test.txt"), "nested LF\n").unwrap();
    fs::write(root.join("run.sh"), "#!/bin/sh\nprintf raw\n").unwrap();
    fs::set_permissions(root.join("run.sh"), fs::Permissions::from_mode(0o755)).unwrap();
    symlink("a file.rs", root.join("link.rs")).unwrap();
    git(&root, &["add", "."]);
    git(
        &root,
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{},module", first.head),
        ],
    );
    git(&root, &["commit", "-m", "typed tree"]);
    let mut summary = first.summary.clone();
    summary.head_ref_oid = Some(text(&root, &["rev-parse", "HEAD"]));
    git(&root, &["checkout", "main"]);
    fs::write(root.join("a file.rs"), "checkout only\n").unwrap();
    let marker = owner.path().join("unexpected-hook");
    let hook = owner.path().join("hooks");
    fs::create_dir(&hook).unwrap();
    fs::write(
        hook.join("post-checkout"),
        format!("#!/bin/sh\ntouch '{}'\n", marker.display()),
    )
    .unwrap();
    fs::set_permissions(
        hook.join("post-checkout"),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    git(
        &first.object_dir,
        &["config", "filter.fixture.smudge", "printf smudged"],
    );
    git(
        &first.object_dir,
        &["config", "filter.fixture.required", "true"],
    );
    git(
        &first.object_dir,
        &["config", "core.hooksPath", hook.to_str().unwrap()],
    );
    let snapshot = ReviewSnapshot::prepare(&root, summary, &owner.path().join("cache")).unwrap();
    assert_eq!(
        fs::read(snapshot.head_root.join("raw.txt")).unwrap(),
        b"raw LF\n"
    );
    assert_eq!(
        fs::read_link(snapshot.head_root.join("link.rs")).unwrap(),
        PathBuf::from("a file.rs")
    );
    assert!(snapshot.head_root.join("module").is_dir());
    assert_eq!(
        fs::read_dir(snapshot.head_root.join("module"))
            .unwrap()
            .count(),
        0
    );
    assert!(!marker.exists());
}

#[test]
fn sha256_snapshots_reuse_exact_trees_and_the_shared_workspace_blob_proof_accepts_them() {
    use crate::semantic::indexing::{inventory::InputInventory, workspace::IndexingWorkspace};
    let owner = tempfile::tempdir().unwrap();
    let root = owner.path().join("source");
    fs::create_dir(&root).unwrap();
    git(&root, &["init", "--object-format=sha256", "-b", "main"]);
    git(&root, &["config", "user.name", "Test"]);
    git(&root, &["config", "user.email", "test@example.invalid"]);
    fs::write(root.join("a.rs"), "before\n").unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-m", "base"]);
    let base = text(&root, &["rev-parse", "HEAD"]);
    fs::write(root.join("a.rs"), "after\n").unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-m", "head"]);
    let head = text(&root, &["rev-parse", "HEAD"]);
    git(&root, &["remote", "add", "origin", root.to_str().unwrap()]);
    let mut summary = crate::pull_request::test_support::summary(7);
    summary.base_ref_oid = Some(base);
    summary.head_ref_oid = Some(head.clone());
    let cache = owner.path().join("cache");
    let first = ReviewSnapshot::prepare(&root, summary.clone(), &cache).unwrap();
    let second = ReviewSnapshot::prepare(&root, summary, &cache).unwrap();
    assert_eq!(first.head_root, second.head_root);
    assert_eq!(first.head.len(), 64);
    let cancel = AtomicBool::new(false);
    let inventory = InputInventory::from_paths(["a.rs".into()]).unwrap();
    let workspace = IndexingWorkspace::create(
        &first.head_root,
        &inventory,
        &owner.path().join("workspaces"),
        Default::default(),
        &cancel,
    )
    .unwrap();
    workspace
        .verify_snapshot(&std::collections::BTreeMap::from([(
            "a.rs".into(),
            text(&root, &["rev-parse", &format!("{head}:a.rs")]),
        )]))
        .unwrap();
    println!("Tree cache: SHA-256 Git objects and shared workspace blob identities agree");
}
