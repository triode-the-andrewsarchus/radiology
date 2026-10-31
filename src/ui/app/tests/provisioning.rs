use super::*;
use crate::semantic::indexing::{provisioning::tests::Fixture, tools::ToolAvailability, Indexer};

fn project() -> (Fixture, tempfile::TempDir, RadiologyApp) {
    let fixture = Fixture::new();
    fixture.npm();
    std::fs::write(fixture.source.join("package.json"), "{}").unwrap();
    std::fs::write(fixture.source.join("a.ts"), "const answer = 1;\n").unwrap();
    std::fs::write(fixture.bin.join("fixture.scip"), b"\x12\x06\x0a\x04a.ts").unwrap();
    let (settings, mut app) = reader();
    app.index_tools = fixture.tools.clone();
    app.open_repository(&fixture.source);
    settle(&mut app);
    app.execute_command(&Context::default(), ToolbarAction::CheckIndexTools);
    settle(&mut app);
    assert!(matches!(
        app.tool_availability(Indexer::TypeScript),
        Some(ToolAvailability::Unavailable { .. })
    ));
    (fixture, settings, app)
}

#[test]
fn explicit_install_verifies_the_tool_then_automatically_generates_the_current_index() {
    let (fixture, _settings, mut app) = project();
    let previous = std::fs::read(fixture.source.join("a.ts")).unwrap();
    app.execute_command(
        &Context::default(),
        ToolbarAction::InstallIndexer(Indexer::TypeScript),
    );
    assert!(app.tool_provisioner.is_running());
    settle(&mut app);
    assert!(
        matches!(app.tool_availability(Indexer::TypeScript), Some(ToolAvailability::Available { version, .. }) if version == "scip-typescript 0.4.0")
    );
    assert_eq!(
        app.generation_report().unwrap().documents,
        1,
        "{:?}",
        app.generation_report()
    );
    assert!(app
        .semantic_index
        .as_ref()
        .unwrap()
        .documents
        .contains_key("a.ts"));
    assert_eq!(
        std::fs::read(fixture.source.join("a.ts")).unwrap(),
        previous
    );
    assert!(!fixture.source.join("node_modules").exists());
    assert!(app.error.is_none(), "{:?}", app.error);
}

#[test]
fn cancel_and_repository_replacement_retire_installation_without_late_generation() {
    let (fixture, settings, mut app) = project();
    std::fs::write(fixture.bin.join("pause"), "").unwrap();
    let prior = Arc::new(SemanticIndex::default());
    app.semantic_index = Some(prior.clone());
    let next = settings.path().join("second");
    repository(&next);
    for replace_repository in [false, true] {
        let _ = std::fs::remove_file(fixture.bin.join("paused.pid"));
        app.execute_command(
            &Context::default(),
            ToolbarAction::InstallIndexer(Indexer::TypeScript),
        );
        crate::pull_request::test_support::wait_for_child(&fixture.bin.join("paused.pid"));
        if replace_repository {
            app.open_repository(&next);
            settle(&mut app);
            assert_eq!(app.repository_root.as_deref(), Some(next.as_path()));
            assert!(!app.tool_provisioner.is_running());
            assert!(app.generation_report().is_none());
            assert!(app.tool_availability(Indexer::TypeScript).is_none());
            assert!(app.semantic_index.is_none());
        } else {
            app.execute_command(&Context::default(), ToolbarAction::CancelToolInstallation);
            app.poll_index(&Context::default());
            assert!(app.tool_provisioner.was_cancelled());
            assert!(Arc::ptr_eq(app.semantic_index.as_ref().unwrap(), &prior));
            assert!(app.generation_report().is_none());
        }
    }
}
