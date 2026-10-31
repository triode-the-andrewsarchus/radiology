use super::*;
use crate::source::SourceBuffer;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

mod real_tools;
mod reuse;

#[test]
fn adapter_columns_override_file_encoding_and_explicit_document_columns_win() {
    let fixture = Fixture::new(&[("a.ts", "/* 🦀 */ const value = 1;\r\n")]);
    let request = fixture.request();
    let (inventory, baseline) = request.prepare(&AtomicBool::new(false)).unwrap();
    let strategy = IndexingStrategy {
        indexer: Indexer::TypeScript,
        directory: String::new(),
        evidence: Vec::new(),
        source_files: 1,
    };
    for file_encoding in [0, 1, 2, 3] {
        for (explicit, expected) in [(0, 2), (1, 1), (2, 2), (3, 3)] {
            let scip = ScipIndex {
                text_document_encoding: file_encoding,
                documents: vec![crate::semantic::ScipDocument {
                    relative_path: "a.ts".into(),
                    position_encoding: explicit,
                    ..Default::default()
                }],
                ..Default::default()
            };
            let normalized = normalize(scip, &strategy, &inventory, &baseline).unwrap();
            assert_eq!(normalized.documents[0].position_encoding, expected);
        }
    }
}

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
            attached_root: self.source.clone(),
            reuse: false,
            review: None,
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
while [ "$#" -gt 0 ]; do case "$1" in --output|--index-output-path) shift; output=$1;; --compdb-path) shift; database=$1;; esac; shift; done
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
    let persisted: Vec<_> = fs::read_dir(&fixture.cache).unwrap().collect();
    assert_eq!(persisted.len(), 2); // merged cache and latest per-project records
    assert!(fixture.cache.join("projects").is_dir());
    assert!(cache_file(&fixture).is_file());
    assert!(generated.report.cache_warning.is_none());
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

fn cache_file(fixture: &Fixture) -> PathBuf {
    fs::read_dir(&fixture.cache)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .expect("persisted index")
}

#[test]
fn cache_round_trip_preserves_partial_coverage_and_original_source_provenance_without_tools() {
    let fixture = Fixture::new(&[
        ("Cargo.toml", "[package]\nname='fixture'"),
        ("lib.rs", "a"),
        ("orphan.rs", "b"),
    ]);
    fixture.tool(Indexer::Rust, &["lib.rs"], "printf compiler-note >&2");
    let generated = run(fixture.request());
    assert!(generated.report.cache_warning.is_none());
    let original_bytes = fs::read(cache_file(&fixture)).unwrap();
    fs::remove_dir_all(&fixture.bin).unwrap();
    let mut generator = IndexGenerator::default();
    generator.start_cached(fixture.request());
    let cached = settle(&mut generator);
    assert_eq!(generator.cache_status(), &CacheLoadStatus::Loaded);
    assert!(cached.report.cached);
    assert!(cached.report.is_partial());
    assert!(cached.report.summary().starts_with("Partial cached index"));
    assert_eq!(cached.report.fingerprint, generated.report.fingerprint);
    let outcome = &cached.report.outcomes[0];
    assert_eq!(outcome.version.as_deref(), Some("rust-analyzer fixture-1"));
    assert_eq!(outcome.missing_paths, ["orphan.rs"]);
    assert!(outcome.diagnostics.contains("compiler-note"));
    let index = cached.index.unwrap();
    assert_eq!(index.definitions.len(), 1);
    assert_eq!(
        index.source_digests,
        generated.index.unwrap().source_digests
    );
    assert!(index.matches_buffer(&SourceBuffer::from_str("lib.rs", "a")));
    assert_eq!(fs::read(cache_file(&fixture)).unwrap(), original_bytes);
    assert_eq!(fs::read_dir(&fixture.cache).unwrap().count(), 2);

    // A different root sharing the same cache never borrows another repository's data.
    let other = Fixture::new(&[("Cargo.toml", "[package]\nname='fixture'"), ("lib.rs", "a")]);
    let mut request = other.request();
    request.cache = fixture.cache.clone();
    assert!(super::super::cache::load(&request, &AtomicBool::new(false))
        .unwrap()
        .is_none());
}

#[test]
fn stale_cache_retains_generation_identity_and_refuses_changed_source_destinations() {
    let fixture = Fixture::new(&[("Cargo.toml", "[package]\nname='fixture'"), ("lib.rs", "a")]);
    fixture.tool(Indexer::Rust, &["lib.rs"], "");
    let generated = run(fixture.request());
    let pristine = fs::read(cache_file(&fixture)).unwrap();
    // Build inputs count even when the source document itself has not changed.
    fs::write(
        fixture.source.join("Cargo.toml"),
        "[package]\nname='changed'",
    )
    .unwrap();
    let cached = super::super::cache::load(&fixture.request(), &AtomicBool::new(false))
        .unwrap()
        .unwrap();
    assert!(cached.report.summary().starts_with("Stale cached index"));
    assert_eq!(cached.report.fingerprint, generated.report.fingerprint);
    assert!(cached
        .index
        .unwrap()
        .matches_buffer(&SourceBuffer::from_str("lib.rs", "a")));
    fs::write(fixture.source.join("lib.rs"), "z").unwrap(); // same size as the original
    fs::write(fixture.source.join("added.rs"), "c").unwrap();
    let cached = super::super::cache::load(&fixture.request(), &AtomicBool::new(false))
        .unwrap()
        .unwrap();
    let index = cached.index.unwrap();
    assert!(!index.matches_buffer(&SourceBuffer::from_str("lib.rs", "z")));
    assert!(index.matches_buffer(&SourceBuffer::from_str("lib.rs", "a")));
    assert!(!index.matches_buffer(&SourceBuffer::from_str("added.rs", "c")));
    assert_eq!(fs::read(cache_file(&fixture)).unwrap(), pristine);
}

#[test]
fn invalid_cache_formats_ranges_paths_and_provenance_never_publish() {
    let fixture = Fixture::new(&[("Cargo.toml", "[package]\nname='fixture'"), ("lib.rs", "a")]);
    fixture.tool(Indexer::Rust, &["lib.rs"], "");
    run(fixture.request());
    let path = cache_file(&fixture);
    let pristine: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    for (pointer, replacement) in [
        ("/format", serde_json::json!(1)), // old coordinate normalization is unsafe
        ("/format", serde_json::json!(999)),
        ("/root", serde_json::json!("/different/root")),
        (
            "/scip/documents/0/relative_path",
            serde_json::json!("../escape.rs"),
        ),
        (
            "/scip/documents/0/occurrences/0/range/start_col",
            serde_json::json!(-1),
        ),
        ("/scip/documents/0/position_encoding", serde_json::json!(99)),
        ("/source_digests/lib.rs", serde_json::json!("invalid")),
        ("/report/documents", serde_json::json!(99)),
    ] {
        let mut invalid = pristine.clone();
        *invalid.pointer_mut(pointer).expect(pointer) = replacement;
        fs::write(&path, serde_json::to_vec(&invalid).unwrap()).unwrap();
        assert!(
            super::super::cache::load(&fixture.request(), &AtomicBool::new(false)).is_err(),
            "{pointer}"
        );
    }
    fs::write(&path, b"{truncated").unwrap();
    assert!(super::super::cache::load(&fixture.request(), &AtomicBool::new(false)).is_err());
    fs::remove_file(&path).unwrap();
    std::os::unix::fs::symlink(fixture.source.join("lib.rs"), &path).unwrap();
    assert!(super::super::cache::load(&fixture.request(), &AtomicBool::new(false)).is_err());
    fs::remove_file(&path).unwrap();
    let sparse = fs::File::create(&path).unwrap();
    sparse.set_len(256 * 1024 * 1024 + 1).unwrap();
    assert!(super::super::cache::load(&fixture.request(), &AtomicBool::new(false)).is_err());
}

#[test]
fn failed_generation_and_cancelled_cache_load_preserve_the_last_persisted_index() {
    let fixture = Fixture::new(&[("Cargo.toml", "[package]\nname='fixture'"), ("lib.rs", "a")]);
    fixture.tool(Indexer::Rust, &["lib.rs"], "");
    run(fixture.request());
    let path = cache_file(&fixture);
    let pristine = fs::read(&path).unwrap();
    fixture.tool(Indexer::Rust, &["lib.rs"], "exit 1");
    assert!(run(fixture.request()).index.is_none());
    assert_eq!(fs::read(&path).unwrap(), pristine);
    assert!(super::super::cache::load(&fixture.request(), &AtomicBool::new(true)).is_err());
    assert_eq!(fs::read(&path).unwrap(), pristine);
    // Cache failures must not discard usable freshly generated data.
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    fixture.tool(Indexer::Rust, &["lib.rs"], "");
    let generated = run(fixture.request());
    assert!(generated.index.is_some());
    assert!(generated.report.cache_warning.is_some());
    assert!(path.is_dir());
}

#[test]
fn clang_generation_uses_a_private_rebased_database_and_keeps_original_inputs() {
    let fixture = Fixture::new(&[
        ("build/compile_commands.json", "[]"),
        ("native.c", "int f(void) { return 0; }\n"),
        ("include/header.h", "#define VALUE 1\n"),
        ("flags.rsp", "-I../include -c ../native.c -o native.o"),
    ]);
    let database = fixture.source.join("build/compile_commands.json");
    let original = serde_json::to_vec(&serde_json::json!([{
        "directory": fixture.source.join("build"), "file": "../native.c",
        "arguments": ["clang", "@../flags.rsp"], "output": "native.o"
    }]))
    .unwrap();
    fs::write(&database, &original).unwrap();
    // Full inventories retain tracked build inputs even though the browser hides build/.
    for arguments in [
        vec!["init", "-q"],
        vec!["add", "-f", "build/compile_commands.json"],
    ] {
        assert!(std::process::Command::new("git")
            .args(arguments)
            .current_dir(&fixture.source)
            .status()
            .unwrap()
            .success());
    }
    let witness = fixture.bin.join("rebased-database.json");
    fixture.tool(
        Indexer::Clang,
        &["native.c"],
        &format!(
            "test -d build\n/bin/cat \"$database\" > {}",
            quote(&witness)
        ),
    );
    let generated = run(fixture.request());
    assert!(generated.report.cache_warning.is_none());
    assert_eq!(generated.report.documents, 1, "{:?}", generated.report);
    let index = generated.index.unwrap();
    assert_eq!(
        index.column_encoding("native.c"),
        Some(crate::source::ColumnEncoding::Utf8)
    );
    assert!(index.matches_buffer(&SourceBuffer::from_str(
        "native.c",
        "int f(void) { return 0; }\n"
    )));
    let arguments = fs::read_to_string(fixture.bin.join("scip-clang.args")).unwrap();
    let arguments: Vec<_> = arguments.lines().collect();
    assert_eq!(arguments[0], "--compdb-path");
    let prepared = Path::new(arguments[1]);
    assert!(prepared.starts_with(&fixture.cache));
    assert_eq!(prepared.file_name().unwrap(), "compile_commands.json");
    assert!(!prepared.exists()); // Disposable run removed after success.
    let commands: serde_json::Value = serde_json::from_slice(&fs::read(witness).unwrap()).unwrap();
    let copied = prepared.parent().unwrap().parent().unwrap();
    assert_eq!(
        commands[0]["directory"],
        serde_json::json!(copied.join("build"))
    );
    assert_eq!(
        commands[0]["file"],
        serde_json::json!(copied.join("native.c"))
    );
    assert_eq!(
        commands[0]["arguments"][1],
        format!("-I{}", copied.join("include").display())
    );
    assert_eq!(
        commands[0]["output"],
        serde_json::json!(copied.join("build/native.o"))
    );
    assert_eq!(fs::read(database).unwrap(), original);
    assert!(!fixture.source.join("build/native.o").exists());
    assert!(
        super::super::cache::load(&fixture.request(), &AtomicBool::new(false))
            .unwrap()
            .unwrap()
            .report
            .cached
    );
}

#[test]
fn malformed_clang_inputs_fail_before_indexer_execution_without_writing_inputs_or_cache() {
    let fixture = Fixture::new(&[
        ("compile_commands.json", "not JSON"),
        ("native.c", "int x;"),
    ]);
    fixture.tool(Indexer::Clang, &["native.c"], "");
    let generated = run(fixture.request());
    assert!(generated.index.is_none());
    assert!(generated.report.outcomes[0].failure.is_some());
    assert!(!fixture.bin.join("scip-clang.args").exists());
    assert_eq!(
        fs::read_to_string(fixture.source.join("compile_commands.json")).unwrap(),
        "not JSON"
    );
    assert!(
        super::super::cache::load(&fixture.request(), &AtomicBool::new(false))
            .unwrap()
            .is_none()
    );
}

fn pinned_request(fixture: &Fixture) -> (tempfile::TempDir, PathBuf, GenerationRequest) {
    let (owner, root, review) = crate::pull_request::review::tests::indexing_fixture();
    let mut request = fixture.request();
    request.root = review.head_root.clone();
    request.attached_root = root.clone();
    request.review = Some(review);
    (owner, root, request)
}

#[test]
fn pinned_generation_uses_exact_after_inputs_and_cache_survives_tree_replacement() {
    let fixture = Fixture::new(&[]);
    let (owner, root, request) = pinned_request(&fixture);
    let witness = fixture.bin.join("working-directory");
    fixture.tool(
        Indexer::Rust,
        &["a file.rs"],
        &format!("pwd > {}\nprintf artifact > artifact.bin", quote(&witness)),
    );
    let generated = run(request.clone());
    assert_eq!(generated.report.revision.as_deref(), request.revision());
    assert_eq!(generated.report.documents, 1, "{:?}", generated.report);
    assert!(
        generated.report.cache_warning.is_none(),
        "{:?}",
        generated.report
    );
    let index = generated.index.unwrap();
    assert!(index.matches_buffer(&SourceBuffer::from_str(
        "a file.rs",
        "one\nchanged\nthree\n"
    )));
    assert!(!index.matches_buffer(&SourceBuffer::from_str("a file.rs", "checkout only\n")));
    let working = fs::read_to_string(witness).unwrap();
    assert!(Path::new(working.trim()).starts_with(&fixture.cache));
    assert!(!request.root.join("artifact.bin").exists());
    assert!(!root.join("artifact.bin").exists());
    assert_eq!(
        fs::read_to_string(root.join("a file.rs")).unwrap(),
        "checkout only\n"
    );
    let cached = super::super::cache::load(&request, &AtomicBool::new(false))
        .unwrap()
        .unwrap();
    assert!(cached.report.cached);
    fs::write(request.root.join("a file.rs"), "tampered tree\n").unwrap();
    let mut replacement = request.clone();
    replacement.review = Some(Arc::new(
        crate::pull_request::review::ReviewSnapshot::prepare(
            &root,
            request.review.as_ref().unwrap().summary.clone(),
            &owner.path().join("cache"),
        )
        .unwrap(),
    ));
    replacement.root = replacement.review.as_ref().unwrap().head_root.clone();
    assert_ne!(replacement.root, request.root);
    fs::remove_file(fixture.bin.join("rust-analyzer")).unwrap();
    let cached = super::super::cache::load(&replacement, &AtomicBool::new(false))
        .unwrap()
        .unwrap();
    assert!(cached.report.cached);
    assert!(cached.report.stale_reason.is_none());
    assert_eq!(cached.report.revision.as_deref(), replacement.revision());
    assert!(super::super::cache::load(&request, &AtomicBool::new(false)).is_err());
    let mut checkout = replacement.clone();
    checkout.review = None;
    assert!(
        super::super::cache::load(&checkout, &AtomicBool::new(false))
            .unwrap()
            .is_none()
    );
}

#[test]
fn pinned_generation_rejects_hidden_modification_and_mismatched_roots_before_tools_run() {
    let fixture = Fixture::new(&[]);
    let (_owner, _root, request) = pinned_request(&fixture);
    fixture.tool(Indexer::Rust, &["a file.rs"], "");
    let cancel = AtomicBool::new(false);
    let progress = Mutex::new(GenerationProgress::default());
    let mut wrong = request.clone();
    wrong.root = fixture.source.clone();
    assert!(generate(wrong, &cancel, &progress)
        .err()
        .unwrap()
        .contains("differs"));
    crate::repository::test_support::git(
        &request.root,
        &["update-index", "--assume-unchanged", "--", "a file.rs"],
    );
    fs::write(request.root.join("a file.rs"), "modified\n").unwrap();
    assert!(generate(request, &cancel, &progress)
        .err()
        .unwrap()
        .contains("pinned"));
    assert!(!fixture.bin.join("rust-analyzer.args").exists());
}

#[test]
fn cancelled_pinned_generation_releases_its_live_snapshot_and_publishes_no_cache() {
    let fixture = Fixture::new(&[]);
    let (_owner, _root, request) = pinned_request(&fixture);
    let marker = fixture.bin.join("started");
    fixture.tool(
        Indexer::Rust,
        &["a file.rs"],
        &format!("printf ready > {}\n/bin/sleep 40", quote(&marker)),
    );
    let weak = Arc::downgrade(request.review.as_ref().unwrap());
    let mut generator = IndexGenerator::default();
    generator.start(request);
    wait_for(&marker);
    assert!(weak.upgrade().is_some());
    generator.stop();
    let deadline = Instant::now() + Duration::from_secs(5);
    while weak.upgrade().is_some() {
        assert!(
            Instant::now() < deadline,
            "retired generation retained its snapshot"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(generator.poll().is_none());
    assert!(generator.was_cancelled());
    assert!(fs::read_dir(&fixture.cache).unwrap().all(|entry| !entry
        .unwrap()
        .path()
        .extension()
        .is_some_and(|extension| extension == "json")));
}
