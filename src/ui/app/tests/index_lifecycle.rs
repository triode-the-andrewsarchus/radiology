//! Automatic open/refresh reuses existing workers and retains prior publications.
use super::*;
use crate::semantic::indexing::service::CacheLoadStatus;
use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};

fn project() -> (tempfile::TempDir, PathBuf, RadiologyApp, PathBuf) {
    let (owner, mut app) = reader();
    let root = owner.path().join("checkout");
    fs::create_dir_all(root.join("src")).unwrap();
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("Tests/RadiologyTests/Fixtures/scip-rust");
    for path in ["Cargo.toml", "src/lib.rs", "src/shapes.rs"] {
        fs::copy(fixture.join(path), root.join(path)).unwrap();
    }
    let executable = rust_indexer_fixture(owner.path(), "src/lib.rs");
    fs::copy(
        fixture.join("index.scip"),
        executable.with_file_name("fixture.scip"),
    )
    .unwrap();
    let script = fs::read_to_string(&executable).unwrap().replace("printf artifact", r#"printf 'run\n' >> "$0.runs"
if [ -e "$0.fail" ]; then echo 'fixture refresh failed' >&2; exit 9; fi
if [ -e "$0.pause" ]; then printf '%s' "$$" > "$0.pid"; while [ ! -e "$0.release" ]; do sleep 0.02; done; fi
printf artifact"#);
    fs::write(&executable, script).unwrap();
    app.index_tools = crate::semantic::indexing::tools::ToolDirectory {
        managed: None,
        home: None,
        search_path: vec![
            executable.parent().unwrap().to_owned(),
            "/usr/bin".into(),
            "/bin".into(),
        ],
    };
    (owner, root, app, executable)
}

fn assert_repaired_index(app: &mut RadiologyApp, source: &Arc<SourceBuffer>) {
    app.load_repository_index();
    settle(app);
    assert!(app.indexing_personality_error().is_none());
    let report = app.generation_report().unwrap();
    assert_eq!(report.documents, 2);
    assert!(report
        .outcomes
        .iter()
        .all(|outcome| outcome.failure.is_none()));
    assert!(Arc::ptr_eq(source, app.open_buffer.as_ref().unwrap()));
}

#[test]
fn automatic_open_generates_once_and_current_cache_needs_no_executable() {
    let (_owner, root, mut app, tool) = project();
    open_reader(&mut app, &root, "src/lib.rs");
    let report = app.generation_report().unwrap();
    assert_eq!(report.documents, 2);
    assert!(!report.cached && report.stale_reason.is_none());
    assert_eq!(
        report.outcomes[0].version.as_deref(),
        Some("rust-analyzer fixture")
    );
    assert!(report.revision.is_none());
    assert_eq!(
        fs::read_to_string(tool.with_extension("runs")).unwrap(),
        "run\n"
    );
    assert!(!root.join("index.scip").exists() && !root.join("artifact.bin").exists());
    let buffer = app.open_buffer.clone().unwrap();
    let history = format!("{:?}", app.history);
    fs::remove_file(&tool).unwrap();
    app.load_repository_index();
    settle(&mut app);
    assert!(app.generation_report().unwrap().cached);
    assert_eq!(app.generation_report().unwrap().documents, 2);
    assert!(Arc::ptr_eq(&buffer, app.open_buffer.as_ref().unwrap()));
    assert_eq!(format!("{:?}", app.history), history);
    app.open_repository(&root);
    settle(&mut app);
    assert!(app.generation_report().unwrap().cached);
    assert_eq!(app.semantic_index.as_ref().unwrap().documents.len(), 2);
}

#[test]
fn automatic_stale_refresh_publishes_cached_data_before_failure_and_keeps_disk_cache() {
    let (_owner, root, mut app, tool) = project();
    open_reader(&mut app, &root, "src/lib.rs");
    let directory = app.session.repository_cache(&root).unwrap().join("indexes");
    let cache = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .ends_with(".index.json")
        })
        .unwrap();
    let pristine = fs::read(&cache).unwrap();
    let shapes = root.join("src/shapes.rs");
    let changed = fs::read_to_string(&shapes).unwrap() + "\n// changed dependency\n";
    fs::write(&shapes, &changed).unwrap();
    fs::write(tool.with_extension("fail"), "").unwrap();
    app.load_repository_index();
    let ctx = Context::default();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !app
        .generation_report()
        .is_some_and(|report| report.cached && report.stale_reason.is_some())
    {
        app.poll_index(&ctx);
        assert!(Instant::now() < deadline, "stale publication missing");
        std::thread::sleep(Duration::from_millis(1));
    }
    let retained = app.semantic_index.clone().unwrap();
    assert_eq!(retained.documents.len(), 2);
    assert!(!retained.matches_buffer(&SourceBuffer::from_str("src/shapes.rs", &changed)));
    assert!(retained.matches_buffer(app.open_buffer.as_ref().unwrap()));
    assert!(app.index_generator.is_running());
    settle(&mut app);
    assert!(Arc::ptr_eq(&retained, app.semantic_index.as_ref().unwrap()));
    let outcome = &app.generation_report().unwrap().outcomes[0];
    assert!(outcome.failure.as_deref().unwrap().contains('9'));
    assert!(outcome.diagnostics.contains("fixture refresh failed"));
    assert_eq!(fs::read(&cache).unwrap(), pristine);
    assert_eq!(
        fs::read_to_string(tool.with_extension("runs")).unwrap(),
        "run\nrun\n"
    );
    assert_eq!(fs::read_to_string(&shapes).unwrap(), changed);
    fs::remove_file(tool.with_extension("fail")).unwrap();
    app.load_repository_index();
    settle(&mut app);
    let report = app.generation_report().unwrap();
    assert!(!report.cached && report.stale_reason.is_none());
    assert_eq!(report.documents, 2);
    assert!(app
        .semantic_index
        .as_ref()
        .unwrap()
        .matches_buffer(&SourceBuffer::from_str("src/shapes.rs", &changed)));
    assert_ne!(fs::read(&cache).unwrap(), pristine);
}

#[test]
fn failed_automatic_trait_refresh_excludes_changed_coordinates_and_repairs_the_catalog() {
    let (_owner, root, mut app, tool) = project();
    let reference = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("Tests/RadiologyTests/Fixtures/scip-rust-traits");
    fs::remove_file(root.join("src/shapes.rs")).unwrap();
    for path in ["Cargo.toml", "src/lib.rs", "src/implementations.rs"] {
        fs::copy(reference.join(path), root.join(path)).unwrap();
    }
    fs::copy(
        reference.join("index.scip"),
        tool.with_file_name("fixture.scip"),
    )
    .unwrap();
    open_reader(&mut app, &root, "src/lib.rs");
    let source = app.open_buffer.clone().unwrap();
    let accepted = &app.semantic_index.as_ref().unwrap().trait_implementations;
    assert!(accepted.is_prepared && !accepted.unverified_semantics);
    assert_eq!(accepted.by_trait.values().map(Vec::len).sum::<usize>(), 5);
    let implementations = root.join("src/implementations.rs");
    let original = fs::read_to_string(&implementations).unwrap();
    fs::write(&implementations, original.replace("Alpha", "Gamma")).unwrap();
    fs::write(tool.with_extension("fail"), "").unwrap();
    app.load_repository_index();
    settle(&mut app);
    assert!(app.generation_report().unwrap().outcomes[0]
        .failure
        .as_deref()
        .unwrap()
        .contains('9'));
    let stale = &app.semantic_index.as_ref().unwrap().trait_implementations;
    assert!(stale.by_trait.is_empty());
    assert_eq!(stale.excluded_paths, ["src/implementations.rs"]);
    assert!(stale.has_gaps());
    assert!(Arc::ptr_eq(&source, app.open_buffer.as_ref().unwrap()));
    fs::write(implementations, original).unwrap();
    fs::remove_file(tool.with_extension("fail")).unwrap();
    assert_repaired_index(&mut app, &source);
    assert_eq!(
        app.semantic_index
            .as_ref()
            .unwrap()
            .trait_implementations
            .by_trait
            .values()
            .map(Vec::len)
            .sum::<usize>(),
        5
    );
}

#[test]
fn automatic_missing_tool_reports_failure_and_reload_recovers_without_installation() {
    let (_owner, root, mut app, tool) = project();
    let script = fs::read(&tool).unwrap();
    fs::remove_file(&tool).unwrap();
    open_reader(&mut app, &root, "src/lib.rs");
    assert!(app.semantic_index.is_none());
    assert!(app.generation_report().unwrap().outcomes[0]
        .failure
        .is_some());
    assert!(!app.tool_provisioner.is_running());
    assert!(!Path::new(&tool.with_extension("runs")).exists());
    app.index_projects_open = true;
    let ctx = Context::default();
    for scale in [1.0, 2.0] {
        ctx.set_pixels_per_point(scale);
        for _ in 0..3 {
            rendered_frame(&mut app, &ctx);
        }
        let output = rendered_frame(&mut app, &ctx);
        assert!(crate::ui::test_support::text_shapes(&output)
            .any(|shape| shape.galley.job.text == "No tool version recorded"));
        assert!(!crate::ui::test_support::text_shapes(&output)
            .any(|shape| shape.galley.job.text == "Tool availability unchecked"));
    }
    let source = app.open_buffer.clone().unwrap();
    fs::write(&tool, script).unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).unwrap();
    assert_repaired_index(&mut app, &source);
}

#[test]
fn noisy_compiler_completion_stays_terminal_after_more_polls_and_frames() {
    let (_owner, root, mut app, tool) = project();
    let script = fs::read_to_string(&tool).unwrap().replace(
        "printf artifact",
        "i=0; while [ $i -lt 2000 ]; do echo progress-$i >&2; i=$((i + 1)); done; printf artifact",
    );
    fs::write(&tool, script).unwrap();
    open_reader(&mut app, &root, "src/lib.rs");
    let summary = app.generation_report().unwrap().summary();
    assert_eq!(app.generation_report().unwrap().documents, 2);
    assert!(app.generation_report().unwrap().outcomes[0]
        .diagnostics
        .contains("progress-1999"));
    let index = app.semantic_index.clone().unwrap();
    let ctx = Context::default();
    let deadline = Instant::now() + Duration::from_millis(200);
    while Instant::now() < deadline {
        app.poll_index(&ctx);
        let output = rendered_frame(&mut app, &ctx);
        assert!(!app.index_generator.is_running());
        assert_eq!(app.generation_report().unwrap().summary(), summary);
        assert!(Arc::ptr_eq(&index, app.semantic_index.as_ref().unwrap()));
        assert!(crate::ui::test_support::text_shapes(&output)
            .any(|shape| shape.galley.job.text == summary));
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn automatic_generation_cancellation_and_root_replacement_retire_late_results() {
    for replace in [false, true] {
        let (owner, root, mut app, tool) = project();
        fs::write(tool.with_extension("pause"), "").unwrap();
        app.open_repository(&root);
        settle_repository(&mut app);
        app.open_file("src/lib.rs");
        let ctx = Context::default();
        let deadline = Instant::now() + Duration::from_secs(10);
        while !Path::new(&tool.with_extension("pid")).exists() || app.open_buffer.is_none() {
            app.poll_index(&ctx);
            app.poll_source(&ctx);
            assert!(Instant::now() < deadline, "automatic runner did not start");
            std::thread::sleep(Duration::from_millis(1));
        }
        let source = app.open_buffer.clone().unwrap();
        if replace {
            let next = owner.path().join("next");
            repository(&next);
            app.open_repository(&next);
        } else {
            app.execute_command(&ctx, ToolbarAction::CancelIndexGeneration);
        }
        settle(&mut app);
        assert!(app.semantic_index.is_none());
        if !replace {
            assert!(app.index_generator.was_cancelled());
            assert!(Arc::ptr_eq(&source, app.open_buffer.as_ref().unwrap()));
        }
        assert!(!root.join("artifact.bin").exists());
    }
}

#[test]
fn cancelling_cache_or_legacy_import_cannot_continue_into_automatic_generation() {
    for import in [false, true] {
        let (_owner, root, mut app, tool) = project();
        app.open_repository(&root);
        settle_repository(&mut app);
        let ctx = Context::default();
        let deadline = Instant::now() + Duration::from_secs(10);
        if import {
            while !app.index_importer.is_running() {
                app.poll_index(&ctx);
                assert!(Instant::now() < deadline);
            }
        } else {
            while app.index_generator.cache_status() != &CacheLoadStatus::Loading {
                app.poll_index_personality();
                assert!(Instant::now() < deadline);
            }
        }
        app.execute_command(&ctx, ToolbarAction::CancelIndexImport);
        settle(&mut app);
        assert!(!app.index_generator.is_running());
        assert!(app.generation_report().is_none() && app.semantic_index.is_none());
        assert!(!Path::new(&tool.with_extension("runs")).exists());
        assert_eq!(app.index_cache_status(), &CacheLoadStatus::Cancelled);
    }
}

#[test]
fn current_partial_cache_retries_failed_projects_and_reuses_successful_ones() {
    let (_owner, root, mut app, rust) = project();
    fs::create_dir(root.join("web")).unwrap();
    fs::write(root.join("web/package.json"), "{}").unwrap();
    fs::write(root.join("web/a.ts"), "const a = 1;\n").unwrap();
    app.open_repository(&root);
    settle(&mut app);
    assert_eq!(app.generation_report().unwrap().documents, 2);
    assert!(app
        .generation_report()
        .unwrap()
        .outcomes
        .iter()
        .any(|outcome| outcome.failure.is_some()));
    let ts = rust.with_file_name("scip-typescript");
    let script = fs::read_to_string(&rust)
        .unwrap()
        .replace("rust-analyzer fixture", "typescript fixture")
        .replace("fixture.scip", "ts.scip");
    fs::write(&ts, script).unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&ts, fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(
        ts.with_file_name("ts.scip"),
        [0x12, 6, 0x0a, 4, b'a', b'.', b't', b's'],
    )
    .unwrap();
    app.open_repository(&root);
    settle(&mut app);
    let report = app.generation_report().unwrap();
    assert_eq!(report.documents, 3);
    assert!(report
        .outcomes
        .iter()
        .all(|outcome| outcome.failure.is_none()));
    let reused = report
        .outcomes
        .iter()
        .find(|outcome| outcome.strategy.indexer == crate::semantic::indexing::Indexer::Rust)
        .unwrap();
    assert_eq!(reused.reused_from.as_deref(), Some("checkout"));
    assert_eq!(
        fs::read_to_string(rust.with_extension("runs")).unwrap(),
        "run\n"
    );
    assert_eq!(
        fs::read_to_string(ts.with_extension("runs")).unwrap(),
        "run\n"
    );
    fs::remove_file(&rust).unwrap();
    fs::remove_file(&ts).unwrap();
    app.open_repository(&root);
    settle(&mut app);
    assert!(app.generation_report().unwrap().cached);
    assert_eq!(app.semantic_index.as_ref().unwrap().documents.len(), 3);
}

#[test]
fn explicit_install_retires_the_pending_automatic_open() {
    let (_owner, root, mut app, tool) = project();
    app.open_repository(&root);
    settle_repository(&mut app);
    // No managed directory: reject before launching any installer process.
    app.execute_command(
        &Context::default(),
        ToolbarAction::InstallIndexer(crate::semantic::indexing::Indexer::Rust),
    );
    settle(&mut app);
    assert!(app
        .error
        .as_deref()
        .unwrap()
        .contains("Tool installation failed"));
    assert!(app.semantic_index.is_none() && app.generation_report().is_none());
    assert!(!tool.with_extension("runs").exists());
}

#[test]
fn unknown_adapters_fail_before_automatic_execution_and_preserve_repairable_settings() {
    use crate::semantic::indexing::personality::Personality;
    let (owner, root, mut app, tool) = project();
    let path = owner
        .path()
        .join("settings/indexing-personalities/default.toml");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let original = std::str::from_utf8(Personality::Default.template()).unwrap();
    let invalid = original.replace("indexers = []", "indexers = ['ghost', 'scip-clang']");
    fs::write(&path, &invalid).unwrap();
    open_reader(&mut app, &root, "src/lib.rs");
    assert!(app
        .indexing_personality_error()
        .unwrap()
        .contains("Unknown personality indexers: ghost."));
    assert!(app.index_plan().selected.is_empty());
    assert!(app.semantic_index.is_none() && app.generation_report().is_none());
    assert!(!tool.with_extension("runs").exists());
    assert_eq!(fs::read_to_string(&path).unwrap(), invalid);
    let source = app.open_buffer.clone().unwrap();
    fs::write(&path, original).unwrap();
    assert_repaired_index(&mut app, &source);
}

#[test]
fn failed_invocation_details_copy_exact_command_without_moving_or_running_the_reader() {
    use crate::ui::test_support::{click, text_center, text_shapes};
    for scale in [1.0, 2.0] {
        let (_owner, root, mut app, tool) = project();
        fs::write(tool.with_extension("fail"), "").unwrap();
        open_reader(&mut app, &root, "src/lib.rs");
        let expected = app.generation_report().unwrap().outcomes[0]
            .command_line
            .clone()
            .unwrap();
        let source = app.open_buffer.clone().unwrap();
        let history = format!("{:?}", app.history);
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        app.index_projects_open = true;
        for _ in 0..3 {
            rendered_frame(&mut app, &ctx);
        }
        let output = rendered_frame(&mut app, &ctx);
        focus::frame(
            &mut app,
            &ctx,
            click(text_center(&output, "Generation Details")),
        );
        let output = rendered_frame(&mut app, &ctx);
        assert!(text_shapes(&output).any(|text| text
            .galley
            .job
            .text
            .starts_with("Working directory:")));
        let copied = focus::frame(&mut app, &ctx, click(text_center(&output, "Copy Command")));
        assert_eq!(copied.platform_output.copied_text, expected);
        assert!(Arc::ptr_eq(&source, app.open_buffer.as_ref().unwrap()));
        assert_eq!(format!("{:?}", app.history), history);
        assert_eq!(
            fs::read_to_string(tool.with_extension("runs")).unwrap(),
            "run\n"
        );
        app.generation_report.as_mut().unwrap().outcomes[0].command_line = None;
        let output = rendered_frame(&mut app, &ctx);
        assert!(!text_shapes(&output).any(|text| text.galley.job.text == "Copy Command"));
    }
}
