use super::*;
use crate::source::SourceBuffer;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

struct Fixture {
    _owner: tempfile::TempDir,
    source: PathBuf,
    cache: PathBuf,
    bin: PathBuf,
}
impl Fixture {
    fn new(files: &[(&str, &str)]) -> Self {
        let owner = tempfile::tempdir().unwrap();
        let source = owner.path().join("source with spaces;literal");
        let cache = owner.path().join("cache");
        let bin = owner.path().join("tools");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&bin).unwrap();
        for (path, text) in files {
            let path = source.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        Self {
            _owner: owner,
            source,
            cache,
            bin,
        }
    }
    fn request(&self) -> GenerationRequest {
        GenerationRequest {
            root: self.source.clone(),
            cache: self.cache.clone(),
            tools: ToolDirectory {
                managed: None,
                home: None,
                search_path: vec![self.bin.clone()],
            },
            timeout: Duration::from_secs(5),
            limits: Default::default(),
        }
    }
    fn tool(&self, indexer: Indexer, documents: &[&str], before_output: &str) {
        let payload = self.bin.join(format!("{}.scip", indexer.id()));
        let mut index = Vec::new();
        for path in documents {
            let occurrence = [field(1, &[0, 0, 1]), field(2, b"local 0"), vec![0x18, 1]].concat();
            let document = [field(1, path.as_bytes()), field(2, &occurrence)].concat();
            index.extend(field(2, &document));
        }
        fs::write(&payload, index).unwrap();
        let executable = self.bin.join(indexer.id());
        let script = format!(
            r#"#!/bin/sh
if [ "${{1:-}}" = --version ]; then printf '{} fixture-1\n'; exit 0; fi
printf '%s\n' "$@" > {}
output=''
while [ "$#" -gt 0 ]; do case "$1" in --output) shift; output=$1;; esac; shift; done
{before_output}
/bin/cat {} > "$output"
"#,
            indexer.id(),
            quote(&self.bin.join(format!("{}.args", indexer.id()))),
            quote(&payload)
        );
        fs::write(&executable, script).unwrap();
        fs::set_permissions(executable, fs::Permissions::from_mode(0o755)).unwrap();
    }
}
fn quote(path: &Path) -> String {
    format!("'{}'", path.to_str().unwrap().replace('\'', "'\\''"))
}
fn field(number: u8, bytes: &[u8]) -> Vec<u8> {
    assert!(bytes.len() < 128);
    [vec![number << 3 | 2, bytes.len() as u8], bytes.to_vec()].concat()
}
fn run(request: GenerationRequest) -> GeneratedIndex {
    generate(
        request,
        &AtomicBool::new(false),
        &Mutex::new(GenerationProgress::default()),
    )
    .unwrap()
}
fn wait_for(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !path.exists() {
        assert!(Instant::now() < deadline, "fixture tool did not start");
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn settle(generator: &mut IndexGenerator) -> GeneratedIndex {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(result) = generator.poll() {
            return result;
        }
        assert!(generator.error().is_none(), "{:?}", generator.error());
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn generates_isolated_projects_with_encoded_scoped_documents_and_partial_failures() {
    let fixture = Fixture::new(&[
        ("rust/Cargo.toml", "[package]\nname='fixture'\n"),
        ("rust/lib.rs", "a\r\n"),
        ("web/package.json", "{}"),
        ("web/app.ts", "b\r\n"),
        ("other/go.mod", "module fixture\n"),
        ("other/main.go", "package main\n"),
    ]);
    fixture.tool(
        Indexer::Rust,
        &["lib.rs", "generated.rs"],
        "printf artifact > artifact.bin",
    );
    fixture.tool(
        Indexer::TypeScript,
        &["app.ts"],
        "printf artifact > artifact.bin",
    );
    let generated = run(fixture.request());
    let index = generated.index.unwrap();
    assert_eq!(index.documents.len(), 2);
    assert_eq!(
        index.column_encoding("rust/lib.rs"),
        Some(crate::source::ColumnEncoding::Utf8)
    );
    assert_eq!(
        index.column_encoding("web/app.ts"),
        Some(crate::source::ColumnEncoding::Utf16)
    );
    assert_eq!(index.definitions.len(), 2); // local 0 stays document scoped.
    assert!(index.matches_buffer(&SourceBuffer::from_str("web/app.ts", "b\r\n")));
    assert!(!index.matches_buffer(&SourceBuffer::from_str("web/app.ts", "changed\n")));
    assert!(generated.report.is_partial());
    assert_eq!(generated.report.documents, 2);
    assert!(generated
        .report
        .outcomes
        .iter()
        .any(|outcome| outcome.strategy.indexer == Indexer::Go && outcome.failure.is_some()));
    assert!(!fixture.source.join("rust/artifact.bin").exists());
    assert!(!fixture.source.join("web/artifact.bin").exists());
    assert_eq!(fs::read_dir(&fixture.cache).unwrap().count(), 0);
    assert!(generated.report.stale_reason.is_none());
}

#[test]
fn rewritten_inputs_and_invalid_outputs_do_not_publish_semantics() {
    let fixture = Fixture::new(&[("Cargo.toml", "[package]\nname='fixture'"), ("lib.rs", "a")]);
    for (documents, action, expected) in [
        (&["lib.rs"][..], "printf changed > lib.rs", "rewritten"),
        (&["../outside.rs"][..], "", "invalid repository-relative"),
        (
            &["lib.rs"][..],
            "/bin/ln -s /dev/null \"$output\"",
            "regular file",
        ),
    ] {
        fixture.tool(Indexer::Rust, documents, action);
        let generated = run(fixture.request());
        assert!(generated.index.is_none());
        assert!(generated.report.outcomes[0]
            .failure
            .as_ref()
            .unwrap()
            .contains(expected));
        assert_eq!(
            fs::read_to_string(fixture.source.join("lib.rs")).unwrap(),
            "a"
        );
    }
}

#[test]
fn edited_or_added_viewed_inputs_are_reported_stale_without_rewriting_snapshot_coordinates() {
    for edit_existing in [true, false] {
        let fixture = Fixture::new(&[("Cargo.toml", "[package]\nname='fixture'"), ("lib.rs", "a")]);
        let ready = fixture.bin.join("ready");
        let release = fixture.bin.join("release");
        fixture.tool(Indexer::Rust, &["lib.rs"], &format!("touch_path={}; printf ready > \"$touch_path\"; while [ ! -e {} ]; do /bin/sleep 0.01; done", quote(&ready), quote(&release)));
        let mut generator = IndexGenerator::default();
        generator.start(fixture.request());
        wait_for(&ready);
        if edit_existing {
            fs::write(fixture.source.join("lib.rs"), "b").unwrap();
        } else {
            fs::write(fixture.source.join("new.rs"), "new").unwrap();
        }
        fs::write(release, "go").unwrap();
        let generated = settle(&mut generator);
        assert!(generated.report.stale_reason.is_some());
        let index = generated.index.unwrap();
        assert!(index.matches_buffer(&SourceBuffer::from_str("lib.rs", "a")));
        assert!(!index.matches_buffer(&SourceBuffer::from_str("lib.rs", "b")));
    }
}

#[test]
fn replacement_cancels_processes_and_only_the_current_request_publishes() {
    let old = Fixture::new(&[("Cargo.toml", "[package]\nname='old'"), ("lib.rs", "old")]);
    let ready = old.bin.join("ready");
    old.tool(
        Indexer::Rust,
        &["lib.rs"],
        &format!("printf ready > {}; /bin/sleep 30", quote(&ready)),
    );
    let next = Fixture::new(&[("Cargo.toml", "[package]\nname='next'"), ("lib.rs", "new")]);
    next.tool(Indexer::Rust, &["lib.rs"], "");
    let mut generator = IndexGenerator::default();
    generator.start(old.request());
    wait_for(&ready);
    generator.start(next.request());
    let generated = settle(&mut generator);
    let index = generated.index.unwrap();
    assert!(index.matches_buffer(&SourceBuffer::from_str("lib.rs", "new")));
    assert!(!index.matches_buffer(&SourceBuffer::from_str("lib.rs", "old")));
    generator.start(old.request());
    generator.stop();
    assert!(generator.was_cancelled());
    assert!(generator.poll().is_none());
}

#[test]
fn missing_source_coverage_and_partial_build_markers_survive_bounded_diagnostics() {
    let fixture = Fixture::new(&[
        ("Cargo.toml", "[package]\nname='fixture'"),
        ("lib.rs", "a"),
        ("orphan.rs", "b"),
    ]);
    fixture.tool(Indexer::Rust, &["lib.rs"], "/usr/bin/head -c 70000 /dev/zero >&2; printf RADIOLOGY_KOTLIN_PARTIAL_BUILD >&2; /usr/bin/head -c 70000 /dev/zero >&2");
    let generated = run(fixture.request());
    let outcome = &generated.report.outcomes[0];
    assert_eq!(outcome.missing_count, 1);
    assert_eq!(outcome.missing_paths, ["orphan.rs"]);
    assert!(outcome.partial_build);
    assert!(generated.report.is_partial());
    assert!(!outcome
        .diagnostics
        .contains("RADIOLOGY_KOTLIN_PARTIAL_BUILD"));
    assert!(outcome.diagnostics.len() < 140000);
}

#[test]
fn an_indexer_deadline_is_a_project_failure_with_bounded_diagnostics() {
    let fixture = Fixture::new(&[("Cargo.toml", "[package]\nname='fixture'"), ("lib.rs", "a")]);
    fixture.tool(
        Indexer::Rust,
        &["lib.rs"],
        "printf waiting >&2; /bin/sleep 30",
    );
    let mut request = fixture.request();
    request.timeout = Duration::from_millis(100);
    let generated = run(request);
    assert!(generated.index.is_none());
    assert!(generated.report.outcomes[0]
        .failure
        .as_ref()
        .unwrap()
        .contains("timed out"));
    assert!(generated.report.outcomes[0].diagnostics.contains("waiting"));
    assert_eq!(fs::read_dir(fixture.cache).unwrap().count(), 0);
}

#[test]
fn python_project_identity_uses_original_name_instead_of_random_workspace_name() {
    let fixture = Fixture::new(&[
        ("pyproject.toml", "[project]\nname='fixture'"),
        ("main.py", "a"),
    ]);
    fixture.tool(Indexer::Python, &["main.py"], "");
    let generated = run(fixture.request());
    assert_eq!(generated.report.documents, 1);
    let arguments = fs::read_to_string(fixture.bin.join("scip-python.args")).unwrap();
    let arguments: Vec<_> = arguments.lines().collect();
    assert_eq!(
        arguments[3],
        fixture.source.file_name().unwrap().to_str().unwrap()
    );
    assert!(!arguments[3].starts_with("radiology-build-inputs-"));
}
