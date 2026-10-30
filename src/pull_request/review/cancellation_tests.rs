use super::*;
use crate::pull_request::github::GitHubClient;
use crate::repository::test_support::fake_git;

use crate::pull_request::test_support::{interrupt_gh, interrupted};

#[test]
fn github_metadata_children_cancel_timeout_and_recover_without_accepting_bad_output() {
    let owner = tempfile::tempdir().unwrap();
    let root = owner.path();
    interrupt_gh(root, |control| {
        GitHubClient::view_cancellable(root, 7, control)
    });
    let gh = fake_git(root, "printf '%s' '{\"number\":7,\"title\":\"Fresh\",\"state\":\"OPEN\",\"headRefOid\":\"abc\"}'");
    let cancel = AtomicBool::new(false);
    let fresh = ReviewControl {
        gh: &gh,
        ..ReviewControl::new(&cancel)
    };
    assert_eq!(
        GitHubClient::view_cancellable(root, 7, &fresh)
            .unwrap()
            .title,
        "Fresh"
    );
    fake_git(root, "printf '{invalid'");
    assert!(GitHubClient::view_cancellable(root, 7, &fresh)
        .unwrap_err()
        .contains("Invalid GitHub JSON"));
    fake_git(root, "printf 'denied' >&2; exit 1");
    assert!(GitHubClient::view_cancellable(root, 7, &fresh)
        .unwrap_err()
        .contains("denied"));
    fake_git(root, "head -c 17000000 /dev/zero");
    assert!(GitHubClient::view_cancellable(root, 7, &fresh)
        .unwrap_err()
        .contains("exceeds 16 MiB"));
    println!("PR metadata: live cancellation/timeout reap children; fresh/malformed/failed/oversized output checked");
}

#[test]
fn fetch_materialization_and_patch_interruption_preserve_accepted_snapshots_and_allow_retry() {
    let (owner, root, accepted) = super::tests::fixture();
    let attached_bytes = std::fs::read(root.join("a file.rs")).unwrap();
    let accepted_bytes = std::fs::read(accepted.head_root.join("a file.rs")).unwrap();
    for phase in [
        "*\"init --bare\"*",
        "*\"fetch --no-tags\"*",
        "*\"worktree add\"*",
        "*\"diff --no-ext-diff\"*",
    ] {
        for timeout in [false, true] {
            let cache = tempfile::tempdir_in(owner.path()).unwrap();
            let script = format!(
                r#"
case "$*" in
  {phase})
    printf '%s\n' "$5" >> phase-attempts
    case "$*" in *"worktree add"*) /usr/bin/git "$@" || exit ;; esac
    printf '%s' "$$" > paused.pid
    exec /bin/sleep 30 ;;
esac
exec /usr/bin/git "$@"
"#
            );
            let git = fake_git(owner.path(), &script);
            let cancel = AtomicBool::new(false);
            let control = ReviewControl {
                cancel: &cancel,
                deadline: Instant::now() + Duration::from_secs(2),
                git: &git,
                gh: Path::new("gh"),
                lease: None,
            };
            let phase_root = if phase.contains("init") {
                cache.path().to_owned()
            } else {
                cache.path().join("review.git")
            };
            interrupted(&control, &phase_root.join("paused.pid"), timeout, || {
                ReviewSnapshot::prepare_cancellable(
                    &root,
                    accepted.summary.clone(),
                    cache.path(),
                    &control,
                )
            });
            assert_eq!(
                std::fs::read_to_string(phase_root.join("phase-attempts"))
                    .unwrap()
                    .lines()
                    .count(),
                1,
                "no fallback/retry after interruption"
            );
            let revision = cache.path().join("revisions").join(&accepted.head);
            if revision.exists() {
                assert_eq!(
                    std::fs::read_dir(&revision).unwrap().count(),
                    0,
                    "incomplete tree must be removed"
                );
            }
            assert_eq!(
                std::fs::read(root.join("a file.rs")).unwrap(),
                attached_bytes
            );
            assert_eq!(
                std::fs::read(accepted.head_root.join("a file.rs")).unwrap(),
                accepted_bytes
            );
            let fresh =
                ReviewSnapshot::prepare(&root, accepted.summary.clone(), cache.path()).unwrap();
            assert_eq!(fresh.head, accepted.head);
            assert_eq!(fresh.base, accepted.base);
            assert_eq!(
                fresh
                    .read("a file.rs", ReviewRevision::After)
                    .unwrap()
                    .text
                    .as_bytes(),
                accepted_bytes
            );
            assert_eq!(fresh.changes.len(), accepted.changes.len());
        }
    }
    println!("PR snapshots: fetch/worktree/patch live Cancel + timeout reap children, clean partial trees, retain accepted bytes and retry successfully");
}
