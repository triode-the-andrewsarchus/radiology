//! Shared PR fixtures and live-child interruption checks.

use super::control::ReviewControl;
use super::github::PullRequestSummary;
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

pub(crate) fn summary(number: usize) -> PullRequestSummary {
    serde_json::from_value(
        serde_json::json!({"number":number,"title":format!("Review {number}"),"state":"OPEN"}),
    )
    .unwrap()
}

pub(crate) fn await_retirement(
    cancel: &AtomicBool,
    started: mpsc::Sender<()>,
    done: mpsc::Sender<()>,
) {
    started.send(()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !cancel.load(Ordering::Relaxed) {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    done.send(()).unwrap();
}

pub(crate) fn wait_for_child(path: &Path) -> i32 {
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

pub(crate) fn interrupted<T: Send>(
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

pub(crate) fn interrupt_gh<T: Send>(
    root: &Path,
    request: impl Fn(&ReviewControl<'_>) -> Result<T, String> + Sync,
) {
    for timeout in [false, true] {
        let _ = std::fs::remove_file(root.join("paused.pid"));
        let gh = crate::repository::test_support::fake_git(
            root,
            "printf '%s' \"$$\" > paused.pid; exec /bin/sleep 30",
        );
        let cancel = AtomicBool::new(false);
        let control = ReviewControl {
            gh: &gh,
            deadline: Instant::now() + Duration::from_secs(2),
            ..ReviewControl::new(&cancel)
        };
        interrupted(&control, &root.join("paused.pid"), timeout, || {
            request(&control)
        });
    }
}
