use super::*;
use std::sync::{atomic::AtomicBool, mpsc};
use std::time::{Duration, Instant};

use crate::pull_request::test_support::{await_retirement, interrupt_gh, summary};

fn settle(list: &mut RecentPullRequests) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while list.is_loading() {
        list.poll();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn publish(list: &mut RecentPullRequests, result: Result<Option<Vec<PullRequestSummary>>, String>) {
    list.phase = PullRequestListPhase::Loading;
    list.job.spawn(move |_| result);
    settle(list);
}

#[test]
fn refresh_failure_cancellation_empty_results_and_repository_changes_preserve_exact_ownership() {
    let owner = tempfile::tempdir().unwrap();
    let mut list = RecentPullRequests::default();
    list.open(owner.path().into());
    settle(&mut list);
    assert!(matches!(list.phase(), PullRequestListPhase::Unavailable(_)));
    publish(&mut list, Ok(Some(vec![summary(7), summary(3)])));
    let original = list.items().to_vec();
    let (began, started) = mpsc::channel();
    let (finished, done) = mpsc::channel();
    list.phase = PullRequestListPhase::Loading;
    list.job.spawn(move |cancel| {
        await_retirement(cancel, began, finished);
        Ok(Some(vec![summary(99)]))
    });
    started.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(list.items(), original);
    list.cancel();
    done.recv_timeout(Duration::from_secs(5)).unwrap();
    list.poll();
    assert_eq!(list.phase(), &PullRequestListPhase::Cancelled);
    assert_eq!(list.items(), original);
    publish(&mut list, Err("authentication required".into()));
    assert_eq!(list.items(), original);
    assert!(
        matches!(list.phase(), PullRequestListPhase::Failed(error) if error.contains("authentication"))
    );
    list.open(owner.path().into());
    assert_eq!(
        list.items(),
        original,
        "leaving review preserves this repository's list"
    );
    let mut updated = summary(7);
    updated.title = "Fresh metadata".into();
    list.remember(&updated);
    assert_eq!(list.items()[0], updated);
    list.remember(&summary(88));
    assert_eq!(
        list.items().len(),
        2,
        "active PRs outside the list use a separate menu fallback"
    );
    publish(&mut list, Ok(Some(vec![])));
    assert!(list.items().is_empty());
    assert_eq!(list.phase(), &PullRequestListPhase::Loaded);
    publish(&mut list, Ok(Some(original)));
    let (release, gate) = mpsc::channel();
    list.job.spawn(move |_| {
        let _ = gate.recv_timeout(Duration::from_secs(5));
        Ok(Some(vec![summary(66)]))
    });
    let other = tempfile::tempdir().unwrap();
    list.open(other.path().into());
    let _ = release.send(());
    settle(&mut list);
    assert!(
        list.items().is_empty(),
        "old-root rows must never enter a new repository"
    );
}

#[cfg(unix)]
#[test]
fn list_requests_are_bounded_cancellable_and_validate_cli_output_before_publication() {
    use crate::repository::test_support::{fake_git, git, repository};
    let owner = tempfile::tempdir().unwrap();
    let root = owner.path();
    repository(root);
    git(root, &["remote", "add", "origin", root.to_str().unwrap()]);
    interrupt_gh(root, |control| RecentPullRequests::load(root, control));
    let gh = fake_git(root, "printf '%s\\n' \"$@\" > listing.args; cat list.json");
    let cancel = AtomicBool::new(false);
    let control = ReviewControl {
        gh: &gh,
        ..ReviewControl::new(&cancel)
    };
    let values = vec![summary(7), summary(3)];
    std::fs::write(root.join("list.json"), serde_json::to_vec(&values).unwrap()).unwrap();
    assert_eq!(
        RecentPullRequests::load(root, &control).unwrap().unwrap(),
        values
    );
    let arguments = std::fs::read_to_string(root.join("listing.args")).unwrap();
    assert!(arguments.contains("--state\nall\n--limit\n25\n--json\n"));
    for invalid in [
        vec![summary(0)],
        vec![summary(7), summary(7)],
        (1..=26).map(summary).collect(),
    ] {
        std::fs::write(
            root.join("list.json"),
            serde_json::to_vec(&invalid).unwrap(),
        )
        .unwrap();
        assert!(RecentPullRequests::load(root, &control).is_err());
    }
    std::fs::write(root.join("list.json"), "[{bad").unwrap();
    assert!(RecentPullRequests::load(root, &control).is_err());
    println!("Recent PRs: live list Cancel/timeout reap children; 25-row request, ordering, invalid/duplicate/overflow/malformed results verified");
}
