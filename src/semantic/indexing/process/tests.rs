use super::*;
use std::collections::BTreeMap;
use std::path::Path;

fn shell(root: &Path, script: &str) -> IndexCommand {
    IndexCommand {
        executable: "/bin/sh".into(),
        arguments: vec![
            "-c".into(),
            script.into(),
            "indexer-test".into(),
            root.into(),
        ],
        directory: root.into(),
        environment: BTreeMap::new(),
        resources: Vec::new(),
    }
}

#[test]
fn preserves_exit_status_streams_and_literal_arguments() {
    let root = tempfile::tempdir().unwrap();
    let command = shell(
        root.path(),
        "printf '%s' \"$1\"; printf 'diagnostic' >&2; exit 7",
    );
    let output = run(&command, &AtomicBool::new(false), Duration::from_secs(5)).unwrap();
    assert_eq!(output.exit_code, Some(7));
    assert!(!output.succeeded());
    assert_eq!(output.stdout.text, root.path().to_str().unwrap());
    assert_eq!(output.stderr.text, "diagnostic");
    assert!(!output.stdout.truncated);
}

#[test]
fn drains_large_streams_without_deadlock_and_retains_first_and_last_bytes() {
    let root = tempfile::tempdir().unwrap();
    let command = shell(root.path(), "(printf HEAD; head -c 1000000 /dev/zero; printf TAIL) & (printf FIRST; head -c 1000000 /dev/zero; printf LAST) >&2 & wait");
    let output = run(&command, &AtomicBool::new(false), Duration::from_secs(10)).unwrap();
    assert!(output.succeeded());
    for (capture, first, last, count) in [
        (&output.stdout, "HEAD", "TAIL", 1_000_008),
        (&output.stderr, "FIRST", "LAST", 1_000_009),
    ] {
        assert!(capture.truncated);
        assert_eq!(capture.total_bytes, count);
        assert!(capture.text.starts_with(first));
        assert!(capture.text.ends_with(last));
        assert!(capture.text.len() < CAPTURE_PART * 2 + 100);
    }
}

#[test]
fn cancellation_and_zero_deadlines_do_not_launch_retired_commands() {
    let root = tempfile::tempdir().unwrap();
    let command = shell(root.path(), "touch \"$1/launched\"");
    let failure = run(&command, &AtomicBool::new(true), Duration::from_secs(1)).unwrap_err();
    assert_eq!(failure.kind, RunFailureKind::Cancelled);
    let failure = run(&command, &AtomicBool::new(false), Duration::ZERO).unwrap_err();
    assert_eq!(failure.kind, RunFailureKind::TimedOut);
    assert!(!root.path().join("launched").exists());
}

#[test]
fn cancellation_reaches_a_running_child_and_preserves_diagnostics() {
    let root = tempfile::tempdir().unwrap();
    let command = shell(
        root.path(),
        "printf ready; printf error >&2; touch \"$1/ready\"; while :; do sleep 1; done",
    );
    let cancelled = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| run(&command, &cancelled, Duration::from_secs(10)));
        let deadline = Instant::now() + Duration::from_secs(5);
        while !root.path().join("ready").exists() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        cancelled.store(true, Ordering::Relaxed);
        let failure = worker.join().unwrap().unwrap_err();
        assert_eq!(failure.kind, RunFailureKind::Cancelled);
        assert_eq!(failure.stdout.text, "ready");
        assert_eq!(failure.stderr.text, "error");
    });
}

#[test]
fn spawn_failure_and_signalled_exit_are_distinct() {
    let root = tempfile::tempdir().unwrap();
    let mut command = shell(root.path(), "kill -TERM $$");
    let output = run(&command, &AtomicBool::new(false), Duration::from_secs(5)).unwrap();
    assert_eq!(output.exit_code, None);
    assert_eq!(output.signal, Some(libc::SIGTERM));
    command.executable = root.path().join("missing-tool");
    assert!(matches!(
        run(&command, &AtomicBool::new(false), Duration::from_secs(5))
            .unwrap_err()
            .kind,
        RunFailureKind::Io(_)
    ));
}

#[cfg(target_os = "linux")]
fn assert_child_stopped(root: &Path) {
    let pid: u32 = std::fs::read_to_string(root.join("child.pid"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat"));
        if stat.is_err() || stat.unwrap().rsplit_once(") ").unwrap().1.starts_with('Z') {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "descendant {pid} survived group termination"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[cfg(target_os = "linux")]
#[test]
fn deadlines_kill_descendants_that_ignore_term() {
    let root = tempfile::tempdir().unwrap();
    let command = shell(
        root.path(),
        "trap '' TERM; (trap '' TERM; sleep 30) & echo $! > \"$1/child.pid\"; printf started; wait",
    );
    let failure = run(
        &command,
        &AtomicBool::new(false),
        Duration::from_millis(150),
    )
    .unwrap_err();
    assert_eq!(failure.kind, RunFailureKind::TimedOut);
    assert_eq!(failure.stdout.text, "started");
    assert_child_stopped(root.path());
}

#[cfg(target_os = "linux")]
#[test]
fn normal_exit_cleans_up_descendants_holding_output_pipes() {
    let root = tempfile::tempdir().unwrap();
    let command = shell(
        root.path(),
        "sleep 30 & echo $! > \"$1/child.pid\"; printf done; exit 0",
    );
    let started = Instant::now();
    let output = run(&command, &AtomicBool::new(false), Duration::from_secs(5)).unwrap();
    assert!(started.elapsed() < Duration::from_secs(2));
    assert!(output.succeeded());
    assert_eq!(output.stdout.text, "done");
    assert_child_stopped(root.path());
}

#[test]
fn observed_stream_is_complete_even_when_diagnostics_truncate() {
    let root = tempfile::tempdir().unwrap();
    let command = shell(root.path(), "head -c 200000 /dev/zero; printf end");
    let mut count = 0;
    let output = run_observed(
        &command,
        &AtomicBool::new(false),
        Duration::from_secs(5),
        |stream, bytes| {
            if stream == OutputStream::Stdout {
                count += bytes.len();
            }
            Ok(())
        },
    )
    .unwrap();
    assert!(output.succeeded());
    assert_eq!(count, 200003);
    assert_eq!(output.stdout.total_bytes, count as u64);
    assert!(output.stdout.truncated);
    let failure = run_observed(
        &shell(root.path(), "printf ready; sleep 30"),
        &AtomicBool::new(false),
        Duration::from_secs(5),
        |_, _| Err(io::Error::other("consumer refused output")),
    )
    .unwrap_err();
    assert!(
        matches!(failure.kind, RunFailureKind::Io(message) if message.contains("consumer refused"))
    );
}
