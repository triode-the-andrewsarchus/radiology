use super::*;
use crate::semantic::indexing::personality::{Personality, Settings};

fn settings(personality: Personality, from: &str, to: &str) -> Arc<Settings> {
    let text = std::str::from_utf8(personality.template())
        .unwrap()
        .replace(from, to);
    Arc::new(Settings::parse(personality, "captured.toml".into(), text.as_bytes()).unwrap())
}

fn load(request: &GenerationRequest) -> Result<GeneratedIndex, String> {
    crate::semantic::indexing::cache::load(request, &AtomicBool::new(false))?
        .ok_or("No cache".into())
}

#[test]
fn personality_planning_filters_before_caps_and_pins_selected_and_skipped_identity() {
    let settings = settings(
        Personality::Default,
        "indexers = []\nproject_limit = 32",
        "indexers = ['scip-go']\nproject_limit = 1",
    );
    let plan = settings.plan([
        "Cargo.toml",
        "lib.rs",
        "main.c",
        "a/go.mod",
        "a/a.go",
        "b/go.mod",
        "b/b.go",
        "c/go.mod",
        "c/c.go",
    ]);
    assert_eq!(plan.selected.len(), 1);
    assert_eq!(plan.selected[0].directory, "a");
    assert_eq!(plan.skipped.len(), 2);
    assert!(plan
        .selected
        .iter()
        .chain(&plan.skipped)
        .all(|strategy| strategy.indexer == Indexer::Go
            && strategy.personality_digest.as_deref() == Some(settings.digest())));
    assert!(plan.uncovered_languages.contains(&"Rust".into()));
    assert!(plan.uncovered_languages.contains(&"C".into()));
    let sel4 = settings_for_sel4();
    let plan = sel4.plan(["main.c", "include/types.h", "/outside.c"]);
    assert_eq!(plan.selected[0].indexer, Indexer::Sel4);
    assert_eq!(plan.selected[0].source_files, 2);
    assert_eq!(plan.selected[0].evidence, ["CMakeLists.txt"]);
}

fn settings_for_sel4() -> Arc<Settings> {
    settings(Personality::Sel4, "bootstrap_python = [\"python3.11\", \"python3.12\", \"python3.13\", \"python3.14\", \"python3\"]", "bootstrap_python = ['missing-python', 'python3']")
}

#[test]
fn captured_default_settings_drive_clang_jobs_and_reject_cache_and_reuse_after_edits() {
    let fixture = Fixture::new(&[("main.c", "int main(void) { return 0; }\n")]);
    let counter = fixture.bin.join("runs");
    fixture.tool(
        Indexer::Clang,
        &["main.c"],
        &format!("printf 'run\\n' >> {}", quote(&counter)),
    );
    let mut request = fixture.request();
    request.settings = Some(settings(
        Personality::Default,
        "clang_jobs = 4",
        "clang_jobs = 7",
    ));
    let generated = run(request.clone());
    assert_eq!(generated.report.documents, 1);
    assert!(fs::read_to_string(fixture.bin.join("scip-clang.args"))
        .unwrap()
        .contains("--jobs\n7\n"));
    let original = load(&request).unwrap();
    assert!(original.report.cached && original.report.stale_reason.is_none());
    request.reuse = true;
    assert_eq!(
        run(request.clone()).report.outcomes[0]
            .reused_from
            .as_deref(),
        Some("checkout")
    );
    assert_eq!(fs::read_to_string(&counter).unwrap().lines().count(), 1);
    let saved = request.settings.clone();
    request.settings = Some(settings(
        Personality::Default,
        "name = \"Default\"",
        "name = \"Edited defaults\"",
    ));
    assert!(load(&request)
        .err()
        .unwrap()
        .contains("personality settings"));
    let updated = run(request.clone());
    assert!(updated.report.outcomes[0].reused_from.is_none());
    assert_eq!(fs::read_to_string(counter).unwrap().lines().count(), 2);
    assert!(fs::read_to_string(fixture.bin.join("scip-clang.args"))
        .unwrap()
        .contains("--jobs\n4\n"));
    request.settings = saved;
    assert!(
        load(&request).is_err(),
        "the latest default cache cannot impersonate prior settings"
    );
    assert_eq!(
        fs::read_to_string(fixture.source.join("main.c")).unwrap(),
        "int main(void) { return 0; }\n"
    );
}

fn sel4_fixture() -> (Fixture, GenerationRequest) {
    let fixture = Fixture::new(&[
        ("CMakeLists.txt", "# fixture\n"),
        ("main.c", "int main(void) { return 0; }\n"),
    ]);
    fixture.tool(Indexer::Clang, &["main.c", "generated/hidden.h"], "");
    let script = format!(
        r#"#!/bin/sh
set -eu
printf '%s\n' "$@" > {arguments}
printf 'run\n' >> {runs}
pipeline=$2
while [ "$#" -gt 0 ]; do case "$1" in --repository) shift; repository=$1;; --configuration) shift; configuration=$1;; --output) shift; output=$1;; --artifacts) shift; artifacts=$1;; esac; shift; done
case "$configuration" in "$repository"/*) echo 'run leaked into source inventory' >&2; exit 17;; esac
/bin/cp "$configuration" {configuration_copy}
for script in "$(/usr/bin/dirname "$pipeline")"/*.py; do /bin/cp "$script" {copies}/; done
printf '%s\n' "$PYTHONDONTWRITEBYTECODE" "$PYTHONUNBUFFERED" "$TMPDIR" > {environment}
if [ -f {pause} ]; then printf '%s' "$$" > {pid}; exec /bin/sleep 30; fi
/bin/mkdir -p "$artifacts"
printf 'generated' > "$artifacts/headers.h"
/bin/cat {payload} > "$output"
"#,
        arguments = quote(&fixture.bin.join("python.args")),
        runs = quote(&fixture.bin.join("python.runs")),
        configuration_copy = quote(&fixture.bin.join("captured.toml")),
        copies = quote(&fixture.bin),
        environment = quote(&fixture.bin.join("python.env")),
        pause = quote(&fixture.bin.join("pause")),
        pid = quote(&fixture.bin.join("paused.pid")),
        payload = quote(&fixture.bin.join("scip-clang.scip"))
    );
    fs::write(fixture.bin.join("python3"), script).unwrap();
    fs::set_permissions(
        fixture.bin.join("python3"),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let mut request = fixture.request();
    request.settings = Some(settings_for_sel4());
    request.tools.managed = Some(fixture.bin.join("managed"));
    (fixture, request)
}

#[test]
fn sel4_runs_shared_pinned_resources_outside_source_and_isolates_default_cache() {
    let (fixture, request) = sel4_fixture();
    let runs = fixture.bin.join("python.runs");
    let mut automatic = request.clone();
    automatic.settings = None;
    assert_eq!(run(automatic.clone()).report.documents, 1);
    let default_path = crate::semantic::indexing::cache::path(&automatic).unwrap();
    let default_bytes = fs::read(&default_path).unwrap();
    let generated = run(request.clone());
    assert_eq!(
        generated.report.documents, 1,
        "generated headers must not become destinations"
    );
    assert_eq!(generated.report.outcomes[0].strategy.indexer, Indexer::Sel4);
    assert!(generated.index.unwrap().documents.contains_key("main.c"));
    let expected = request.settings.as_ref().unwrap();
    assert_eq!(
        fs::read(fixture.bin.join("captured.toml")).unwrap(),
        expected.bytes()
    );
    for (name, bytes) in crate::semantic::indexing::personality::pipeline_resources() {
        assert_eq!(fs::read(fixture.bin.join(name)).unwrap(), *bytes);
    }
    let env = fs::read_to_string(fixture.bin.join("python.env")).unwrap();
    assert!(env.starts_with("1\n1\n"));
    let run_directory = Path::new(env.lines().nth(2).unwrap());
    assert!(run_directory.starts_with(&fixture.cache) && !run_directory.exists());
    let path = crate::semantic::indexing::cache::path(&request).unwrap();
    assert!(path.starts_with(fixture.cache.join("personalities/sel4")));
    assert_eq!(fs::read(default_path).unwrap(), default_bytes);
    let mut reuse = request.clone();
    reuse.reuse = true;
    assert_eq!(
        run(reuse).report.outcomes[0].reused_from.as_deref(),
        Some("checkout")
    );
    assert_eq!(fs::read_to_string(&runs).unwrap(), "run\n");
    assert!(load(&automatic).unwrap().report.cached);
    let mut warm = request.clone();
    warm.tools.search_path.clear();
    warm.tools.managed = None;
    assert!(load(&warm).unwrap().report.cached);
    warm.settings = Some(settings(
        Personality::Sel4,
        "clang_jobs = 1",
        "clang_jobs = 2",
    ));
    assert!(load(&warm).is_err());
    // A settings edit must regenerate the selected pipeline, never fall back.
    warm.tools = request.tools.clone();
    let edited = run(warm.clone());
    let outcome = &edited.report.outcomes[0];
    assert_eq!(outcome.strategy.indexer, Indexer::Sel4);
    assert!(outcome.reused_from.is_none() && outcome.failure.is_none());
    assert_eq!(
        outcome.strategy.personality_digest.as_deref(),
        Some(warm.settings.as_ref().unwrap().digest())
    );
    assert_eq!(
        fs::read(fixture.bin.join("captured.toml")).unwrap(),
        warm.settings.as_ref().unwrap().bytes()
    );
    assert_eq!(fs::read_to_string(&runs).unwrap(), "run\nrun\n");
    assert!(load(&warm).unwrap().report.cached);
    assert!(
        load(&request).is_err(),
        "old settings cannot read the regenerated cache"
    );
    let default_path = crate::semantic::indexing::cache::path(&automatic).unwrap();
    assert_eq!(fs::read(default_path).unwrap(), default_bytes);
    assert!(!fixture.source.join("__pycache__").exists());
    assert_eq!(fs::read_dir(&fixture.source).unwrap().count(), 2);
}

#[test]
fn sel4_live_cancel_reaps_runner_and_cleans_private_inputs_without_losing_cache() {
    let (fixture, request) = sel4_fixture();
    run(request.clone());
    let accepted = fs::read(crate::semantic::indexing::cache::path(&request).unwrap()).unwrap();
    fs::write(fixture.bin.join("pause"), "").unwrap();
    let marker = fixture.bin.join("paused.pid");
    let cancel = AtomicBool::new(false);
    let control = crate::pull_request::control::ReviewControl::new(&cancel);
    crate::pull_request::test_support::interrupted(&control, &marker, false, || {
        generate(
            request.clone(),
            &cancel,
            &Mutex::new(GenerationProgress::default()),
        )
    });
    assert_eq!(
        fs::read(crate::semantic::indexing::cache::path(&request).unwrap()).unwrap(),
        accepted
    );
    assert!(walkdir::WalkDir::new(&fixture.cache)
        .into_iter()
        .map(Result::unwrap)
        .all(|entry| !entry
            .file_name()
            .to_string_lossy()
            .starts_with(".radiology-index-run-")
            && !entry
                .file_name()
                .to_string_lossy()
                .starts_with("radiology-build-inputs-")));
    fs::remove_file(fixture.bin.join("pause")).unwrap();
    assert_eq!(run(request).report.documents, 1);
}

#[test]
fn sel4_missing_bootstrap_and_unsafe_tools_fail_before_runner_side_effects() {
    let (fixture, mut request) = sel4_fixture();
    request.tools.managed = Some(fixture.source.join("tools"));
    let generated = run(request.clone());
    assert!(generated.report.outcomes[0]
        .failure
        .as_ref()
        .unwrap()
        .contains("outside viewed"));
    assert!(!fixture.source.join("tools").exists());
    request.tools.managed = Some(fixture.bin.join("managed"));
    fs::remove_file(fixture.bin.join("python3")).unwrap();
    assert!(run(request).report.outcomes[0]
        .failure
        .as_ref()
        .unwrap()
        .contains("bootstrap Python"));
    assert!(!fixture.bin.join("python.args").exists());
}

#[test]
fn sel4_cache_namespace_cannot_escape_through_a_symlink() {
    let (fixture, request) = sel4_fixture();
    fs::create_dir(&fixture.cache).unwrap();
    std::os::unix::fs::symlink(&fixture.source, fixture.cache.join("personalities")).unwrap();
    let error = generate(
        request,
        &AtomicBool::new(false),
        &Mutex::new(GenerationProgress::default()),
    )
    .err()
    .unwrap();
    assert!(error.contains("cache escapes"));
    assert!(!fixture.source.join("sel4").exists());
    assert!(!fixture.bin.join("python.args").exists());
}
