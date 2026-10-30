use super::*;
use crate::pull_request::github::GitHubClient;
use crate::repository::test_support::fake_git;
use std::sync::atomic::Ordering;

fn wait_for_child(path: &Path) -> i32 {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Ok(pid) = std::fs::read_to_string(path) {
            if let Ok(pid) = pid.trim().parse() {
                return pid;
            }
        }
        assert!(
            Instant::now() < deadline,
            "child never reached {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn interrupted<T: Send>(
    control: &ReviewControl<'_>,
    pid_path: &Path,
    timeout: bool,
    work: impl FnOnce() -> Result<T, String> + Send,
) -> String {
    std::thread::scope(|scope| {
        let worker = scope.spawn(work);
        let pid = wait_for_child(pid_path);
        if !timeout {
            control.cancel.store(true, Ordering::Relaxed);
        }
        let started = Instant::now();
        let error = worker
            .join()
            .unwrap()
            .err()
            .expect("interrupted operation must fail");
        assert!(started.elapsed() < Duration::from_secs(3));
        assert!(
            error.contains(if timeout { "timed out" } else { "cancelled" }),
            "{error}"
        );
        assert_eq!(
            unsafe { libc::kill(pid, 0) },
            -1,
            "child {pid} must be reaped"
        );
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
        error
    })
}

#[test]
fn github_metadata_children_cancel_timeout_and_recover_without_accepting_bad_output() {
    let owner = tempfile::tempdir().unwrap();
    let root = owner.path();
    for timeout in [false, true] {
        let _ = std::fs::remove_file(root.join("paused.pid"));
        let gh = fake_git(root, "printf '%s' \"$$\" > paused.pid; exec /bin/sleep 30");
        let cancel = AtomicBool::new(false);
        let control = ReviewControl {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(2),
            git: Path::new("git"),
            gh: &gh,
        };
        interrupted(&control, &root.join("paused.pid"), timeout, || {
            GitHubClient::view_cancellable(root, 7, &control)
        });
        cancel.store(false, Ordering::Relaxed);
        let fresh = ReviewControl {
            deadline: Instant::now() + Duration::from_secs(5),
            ..control
        };
        fake_git(root, "printf '%s' '{\"number\":7,\"title\":\"Fresh\",\"state\":\"OPEN\",\"headRefOid\":\"abc\"}'");
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
    }
    println!("PR metadata: live cancellation/timeout reap children; fresh/malformed/failed/oversized output checked");
}

#[test]
fn fetch_materialization_and_patch_interruption_preserve_accepted_snapshots_and_allow_retry() {
    let (owner, root, accepted) = super::tests::fixture();
    let attached_bytes = std::fs::read(root.join("a file.rs")).unwrap();
    let accepted_bytes = std::fs::read(accepted.head_root.join("a file.rs")).unwrap();
    for phase in ["fetch:*", "worktree:add", "diff:--no-ext-diff"] {
        for timeout in [false, true] {
            let cache = tempfile::tempdir_in(owner.path()).unwrap();
            let script = format!(
                r#"
case "$5:$6" in
  {phase})
    printf '%s\n' "$5" >> phase-attempts
    if [ "$5" = worktree ]; then /usr/bin/git "$@" || exit; fi
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
            };
            interrupted(
                &control,
                &cache.path().join("review.git/paused.pid"),
                timeout,
                || {
                    ReviewSnapshot::prepare_cancellable(
                        &root,
                        accepted.summary.clone(),
                        cache.path(),
                        &control,
                    )
                },
            );
            assert_eq!(
                std::fs::read_to_string(cache.path().join("review.git/phase-attempts"))
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
