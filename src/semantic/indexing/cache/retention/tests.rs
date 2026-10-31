use super::*;
use crate::semantic::indexing::cache::tests::{entry, request};
use std::time::{Duration, SystemTime};

struct Fixture {
    _owner: tempfile::TempDir,
    request: GenerationRequest,
    entry: StoredEntry,
    current: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let (owner, attached, review) = crate::pull_request::review::tests::indexing_fixture();
        let mut request = request(attached, owner.path().join("semantic-cache"));
        request.root = review.head_root.clone();
        request.review = Some(review);
        fs::create_dir(&request.cache).unwrap();
        let mut entry = entry(
            request.identity_root().unwrap(),
            "retained Unicode 🦀".into(),
        );
        entry.report.revision = request.revision().map(str::to_owned);
        let current = path(&request).unwrap();
        fs::write(&current, serde_json::to_vec(&entry).unwrap()).unwrap();
        Self {
            _owner: owner,
            request,
            entry,
            current,
        }
    }

    fn history(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for number in 1..=6 {
            let mut value = serde_json::to_value(&self.entry).unwrap();
            let revision = format!("{number:040x}");
            value["report"]["revision"] = revision.clone().into();
            let path = self
                .request
                .cache
                .join(key(&self.entry.root, Some(&revision)));
            fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
            fs::File::open(&path)
                .unwrap()
                .set_times(
                    fs::FileTimes::new()
                        .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(number)),
                )
                .unwrap();
            paths.push(path);
        }
        // Current publication wins even when timestamps put it before every old record.
        fs::File::open(&self.current)
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(SystemTime::UNIX_EPOCH))
            .unwrap();
        paths
    }

    fn maintain(&self, cancel: &AtomicBool) -> Result<bool, String> {
        maintain(&self.request, &self.entry.report, &self.current, cancel)
    }
    fn finish(&self) {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while !self.maintain(&AtomicBool::new(false)).unwrap() {
            assert!(
                std::time::Instant::now() < deadline,
                "maintenance never became idle"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

#[test]
fn bounds_owned_revisions_preserving_current_checkout_foreign_unknown_and_symlinks() {
    let fixture = Fixture::new();
    let history = fixture.history();
    let parent = &fixture.request.cache;
    let mut checkout = serde_json::to_value(&fixture.entry).unwrap();
    checkout["report"]["revision"] = serde_json::Value::Null;
    let checkout_path = parent.join(key(&fixture.entry.root, None));
    fs::write(&checkout_path, serde_json::to_vec(&checkout).unwrap()).unwrap();
    let foreign_root = fixture.entry.root.join("foreign");
    let mut foreign = serde_json::to_value(&fixture.entry).unwrap();
    foreign["root"] = foreign_root.to_str().unwrap().into();
    let foreign_path = parent.join(key(&foreign_root, fixture.request.revision()));
    fs::write(&foreign_path, serde_json::to_vec(&foreign).unwrap()).unwrap();
    let malformed = parent.join(format!("{}.index.json", "a".repeat(64)));
    fs::write(&malformed, "{truncated").unwrap();
    let unknown = parent.join("notes.index.json");
    fs::write(&unknown, serde_json::to_vec(&fixture.entry).unwrap()).unwrap();
    let mut unsupported = serde_json::to_value(&fixture.entry).unwrap();
    unsupported["format"] = 0.into();
    unsupported["report"]["revision"] = "c".repeat(40).into();
    let unsupported_path = parent.join(key(&fixture.entry.root, Some(&"c".repeat(40))));
    fs::write(&unsupported_path, serde_json::to_vec(&unsupported).unwrap()).unwrap();
    let link = parent.join(format!("{}.index.json", "b".repeat(64)));
    std::os::unix::fs::symlink(&history[0], &link).unwrap();
    let stage = parent.join(".radiology-index-stage-abandoned");
    fs::write(&stage, "retired stage").unwrap();
    let kept: Vec<_> = [
        &fixture.current,
        &checkout_path,
        &foreign_path,
        &malformed,
        &unknown,
        &unsupported_path,
    ]
    .into_iter()
    .map(|path| (path.clone(), fs::read(path).unwrap()))
    .collect();
    fixture.finish();
    assert!(history[..4].iter().all(|path| !path.exists()));
    assert!(history[4..].iter().all(|path| path.exists()));
    assert!(!stage.exists());
    assert!(fs::symlink_metadata(link).unwrap().file_type().is_symlink());
    for (path, bytes) in kept {
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
}

#[test]
fn live_writer_defers_cleanup_then_idle_retry_prunes_and_open_readers_still_decode() {
    let fixture = Fixture::new();
    let history = fixture.history();
    let open = fs::File::open(&history[0]).unwrap();
    let stage = fixture.request.cache.join(".radiology-index-stage-active");
    fs::write(&stage, "active").unwrap();
    drop(projects::lease_file(&fixture.request).unwrap());
    let ready = fixture.request.cache.join("holder.pid");
    let command = crate::process::ProcessCommand {
        executable: "/usr/bin/flock".into(),
        arguments: vec![
            "--shared".into(),
            "--no-fork".into(),
            fixture.request.cache.join(".project-cache.lock").into(),
            "/bin/sh".into(),
            "-c".into(),
            "printf '%s' \"$$\" > \"$1\"; exec /bin/sleep 30".into(),
            "holder".into(),
            ready.clone().into(),
        ],
        directory: fixture.request.cache.clone(),
        environment: Default::default(),
        resources: vec![],
    };
    let cancel = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let child = scope.spawn(|| crate::process::run(&command, &cancel, Duration::from_secs(5)));
        let pid = crate::pull_request::test_support::wait_for_child(&ready);
        assert!(!fixture.maintain(&AtomicBool::new(false)).unwrap());
        assert!(stage.exists() && history.iter().all(|path| path.exists()));
        cancel.store(true, Ordering::Relaxed);
        assert!(child.join().unwrap().is_err());
        assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
    });
    fixture.finish();
    assert!(!stage.exists() && !history[0].exists());
    let restored: StoredEntry = serde_json::from_reader(open).unwrap();
    assert_eq!(
        restored.report.revision.as_deref(),
        Some("0000000000000000000000000000000000000001")
    );
}

#[test]
fn failed_stale_and_cancelled_runs_leave_history_until_successful_publication() {
    let mut fixture = Fixture::new();
    let history = fixture.history();
    fixture.entry.report.stale_reason = Some("changed inputs".into());
    fixture.maintain(&AtomicBool::new(false)).unwrap();
    assert!(history.iter().all(|path| path.exists()));
    fixture.entry.report.stale_reason = None;
    fixture
        .entry
        .report
        .outcomes
        .push(crate::semantic::indexing::service::StrategyOutcome {
            strategy: crate::semantic::indexing::IndexPlan::inspect(["Cargo.toml", "lib.rs"])
                .selected
                .remove(0),
            version: Some("fixture".into()),
            reused_from: None,
            cache_warning: None,
            documents: 1,
            missing_count: 0,
            missing_paths: vec![],
            failure: Some("compiler failed".into()),
            diagnostics: String::new(),
            partial_build: false,
            elapsed: Duration::ZERO,
        });
    fixture.maintain(&AtomicBool::new(false)).unwrap();
    assert!(history.iter().all(|path| path.exists()));
    fixture.entry.report.outcomes[0].failure = None;
    fixture.entry.report.outcomes[0].partial_build = true;
    fixture.maintain(&AtomicBool::new(false)).unwrap();
    assert!(history.iter().all(|path| path.exists()));
    fixture.entry.report.outcomes[0].partial_build = false;
    assert!(fixture
        .maintain(&AtomicBool::new(true))
        .unwrap_err()
        .contains("cancelled"));
    assert!(history.iter().all(|path| path.exists()));
    save(
        &fixture.request,
        &fixture.entry.inventory,
        &fixture.entry.scip,
        &fixture.entry.source_digests,
        &fixture.entry.report,
        &AtomicBool::new(false),
    )
    .unwrap();
    fixture.finish();
    assert!(fixture.current.exists() && !history[0].exists());
}
