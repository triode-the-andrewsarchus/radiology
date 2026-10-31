use super::reuse::project_cache;
use super::*;
use crate::semantic::indexing::cache::projects::{maintain, reader_lease};

fn maintain_when_idle(fixture: &Fixture, report: &GenerationReport) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !maintain(&fixture.request(), report, &AtomicBool::new(false))? {
        assert!(
            Instant::now() < deadline,
            "project maintenance never became idle"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    Ok(())
}

#[test]
fn renamed_projects_retire_only_after_success_and_leave_foreign_unknown_and_linked_files() {
    let fixture = Fixture::new(&[("old/package.json", "{}"), ("old/app.ts", "a")]);
    fixture.tool(Indexer::TypeScript, &["app.ts"], "");
    run(fixture.request());
    let (old, _) = project_cache(&fixture).pop().unwrap();
    let namespace = old.parent().unwrap();
    let unknown = namespace.join("notes.json");
    fs::write(&unknown, "keep notes").unwrap();
    let link = namespace.join(format!("{}.json", "a".repeat(64)));
    std::os::unix::fs::symlink(fixture.source.join("old/app.ts"), &link).unwrap();
    let orphan = namespace.join(".radiology-index-stage-abandoned");
    fs::write(&orphan, "retired stage").unwrap();
    let other = Fixture::new(&[("package.json", "{}"), ("app.ts", "b")]);
    let mut other_request = other.request();
    other_request.cache = fixture.cache.clone();
    other_request.tools = fixture.request().tools;
    run(other_request);
    let foreign = project_cache(&fixture)
        .into_iter()
        .find(|(path, _)| path.parent() != old.parent())
        .unwrap();
    fs::rename(fixture.source.join("old"), fixture.source.join("new")).unwrap();
    fixture.tool(Indexer::TypeScript, &["app.ts"], "exit 9");
    assert!(run(fixture.request()).report.outcomes[0].failure.is_some());
    assert!(old.exists() && orphan.exists());
    fixture.tool(Indexer::TypeScript, &["app.ts"], "");
    let accepted = run(fixture.request());
    maintain_when_idle(&fixture, &accepted.report).unwrap();
    assert!(
        accepted.report.cache_warning.is_none(),
        "{:?}",
        accepted.report
    );
    assert!(!old.exists() && !orphan.exists());
    assert_eq!(fs::read_to_string(unknown).unwrap(), "keep notes");
    assert!(fs::symlink_metadata(link).unwrap().file_type().is_symlink());
    assert_eq!(fs::read(foreign.0).unwrap(), foreign.1);
    assert_eq!(
        fs::read_to_string(fixture.source.join("new/app.ts")).unwrap(),
        "a"
    );
}

#[test]
fn valid_legacy_records_reuse_and_migrate_without_claiming_corrupt_or_foreign_data() {
    let (fixture, _report, scoped) = cached_typescript_project();
    let legacy = fixture
        .cache
        .join("projects")
        .join(scoped.file_name().unwrap());
    fs::rename(&scoped, &legacy).unwrap();
    fs::remove_file(scoped.parent().unwrap().join(".legacy-migrated")).unwrap();
    let unknown = legacy.with_file_name(format!("{}.json", "e".repeat(64)));
    fs::write(&unknown, "{truncated").unwrap();
    let other = Fixture::new(&[("package.json", "{}"), ("app.ts", "b")]);
    let mut request = other.request();
    request.cache = fixture.cache.clone();
    request.tools = fixture.request().tools;
    run(request);
    let (foreign, bytes) = project_cache(&fixture)
        .into_iter()
        .find(|(path, _)| path.parent() != scoped.parent() && path.parent() != legacy.parent())
        .expect("foreign scoped record");
    let foreign_legacy = legacy.with_file_name(foreign.file_name().unwrap());
    fs::rename(foreign, &foreign_legacy).unwrap();
    fixture.tool(Indexer::TypeScript, &["app.ts"], "exit 9"); // actual generation would fail
    let mut request = fixture.request();
    request.reuse = true;
    let reused = run(request);
    maintain_when_idle(&fixture, &reused.report).unwrap();
    assert_eq!(
        reused.report.outcomes[0].reused_from.as_deref(),
        Some("checkout")
    );
    assert!(reused.report.cache_warning.is_none(), "{:?}", reused.report);
    assert!(scoped.exists() && !legacy.exists());
    assert_eq!(fs::read_to_string(unknown).unwrap(), "{truncated");
    assert_eq!(fs::read(foreign_legacy).unwrap(), bytes);
}

#[test]
fn skipped_projects_keep_their_records_beyond_the_run_cap() {
    let fixture = Fixture::new(&[("p99/package.json", "{}"), ("p99/app.ts", "a")]);
    fixture.tool(Indexer::TypeScript, &["app.ts"], "");
    run(fixture.request());
    let (retained, bytes) = project_cache(&fixture).pop().unwrap();
    for n in 0..32 {
        let project = fixture.source.join(format!("p{n:02}"));
        fs::create_dir(&project).unwrap();
        fs::write(project.join("package.json"), "{}").unwrap();
        fs::write(project.join("app.ts"), "a").unwrap();
    }
    let generated = run(fixture.request());
    assert_eq!(generated.report.plan.skipped.len(), 1);
    assert_eq!(generated.report.documents, 32);
    assert_eq!(fs::read(retained).unwrap(), bytes);
}

#[test]
fn cross_process_ownership_defers_cleanup_and_writer_wait_is_bounded_and_cancellable() {
    let (fixture, report, record) = cached_typescript_project();
    let bytes = fs::read(&record).unwrap();
    let stage = record.with_file_name(".radiology-index-stage-active");
    let ready = fixture.bin.join("lease.pid");
    for mode in ["shared", "exclusive"] {
        let _ = fs::remove_file(&ready);
        fs::write(&stage, "active").unwrap();
        let command = super::super::super::command::IndexCommand {
            executable: "/usr/bin/python3".into(),
            arguments: vec!["-c".into(), "import fcntl,os,pathlib,sys,time; f=open(sys.argv[1],'r+'); fcntl.flock(f,fcntl.LOCK_SH if sys.argv[3]=='shared' else fcntl.LOCK_EX); pathlib.Path(sys.argv[2]).write_text(str(os.getpid())); time.sleep(30)".into(), fixture.cache.join(".project-cache.lock").into(), ready.clone().into(), mode.into()],
            directory: fixture.bin.clone(), environment: Default::default(), resources: Vec::new(),
        };
        let cancel = AtomicBool::new(false);
        std::thread::scope(|scope| {
            let holder = scope.spawn(|| process::run(&command, &cancel, Duration::from_secs(10)));
            crate::pull_request::test_support::wait_for_child(&ready);
            assert!(!maintain(&fixture.request(), &report, &AtomicBool::new(false),).unwrap());
            assert!(stage.exists());
            if mode == "exclusive" {
                let mut request = fixture.request();
                request.timeout = Duration::from_millis(50);
                assert!(reader_lease(&request, &AtomicBool::new(false))
                    .unwrap_err()
                    .contains("busy"));
                assert!(reader_lease(&request, &AtomicBool::new(true))
                    .unwrap_err()
                    .contains("cancelled"));
            }
            cancel.store(true, std::sync::atomic::Ordering::Relaxed);
            assert!(holder.join().unwrap().is_err());
        });
        maintain_when_idle(&fixture, &report).unwrap();
        assert!(!stage.exists());
        assert_eq!(fs::read(&record).unwrap(), bytes);
    }
}

#[test]
fn migration_marker_symlinks_are_never_followed_for_writes() {
    let (fixture, report, record) = cached_typescript_project();
    let marker = record.parent().unwrap().join(".legacy-migrated");
    fs::remove_file(&marker).unwrap();
    let target = fixture.source.join("must-not-create");
    std::os::unix::fs::symlink(&target, marker).unwrap();
    let error = maintain_when_idle(&fixture, &report).unwrap_err();
    assert!(error.contains("regular file"));
    assert!(!target.exists());
}

fn cached_typescript_project() -> (Fixture, GenerationReport, PathBuf) {
    let fixture = Fixture::new(&[("package.json", "{}"), ("app.ts", "a")]);
    fixture.tool(Indexer::TypeScript, &["app.ts"], "");
    let generated = run(fixture.request());
    maintain_when_idle(&fixture, &generated.report).unwrap();
    let (record, _) = project_cache(&fixture).pop().unwrap();
    (fixture, generated.report, record)
}
