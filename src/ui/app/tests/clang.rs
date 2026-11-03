//! Best-effort assumptions remain visible before a tool runs.
use super::*;

#[test]
fn clang_source_only_plan_explains_defaults_and_cpp_language_in_actual_app_frames() {
    let (_owner, mut app) =
        isolated_source("main.C", "class Kernel {};\nint main() { return 0; }\n");
    let ctx = Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while app.index_plan.selected.is_empty() {
        rendered_frame(&mut app, &ctx);
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert!(app.index_plan.selected[0].is_best_effort());
    app.index_projects_open = true;
    rendered_frame(&mut app, &ctx);
    let output = rendered_frame(&mut app, &ctx);
    let labels: Vec<_> = crate::ui::test_support::text_shapes(&output)
        .map(|shape| shape.galley.job.text.as_str())
        .collect();
    assert!(labels
        .iter()
        .any(|label| label.contains("Best effort · GNU defaults")));
    assert!(labels
        .iter()
        .any(|label| label.contains("Missing headers, defines, targets")));
    assert_eq!(
        crate::syntax::LanguageRegistry::for_path("main.C").display_name,
        "C++"
    );
}

#[test]
fn nongit_index_controls_find_hidden_database_without_adding_build_outputs_to_files() {
    let (_owner, mut app) = isolated_source("native/main.cpp", "int main() {}\n");
    let root = app.repository_root.clone().unwrap();
    assert!(!root.join(".git").exists());
    std::fs::write(root.join("native/CMakeLists.txt"), "# actual project\n").unwrap();
    std::fs::create_dir_all(root.join("native/build")).unwrap();
    let database = root.join("native/build/compile_commands.json");
    std::fs::write(&database, "[]").unwrap();
    std::fs::write(root.join("native/build/private.o"), "untouched object").unwrap();
    let buffer = app.open_buffer.clone().unwrap();
    let history = format!("{:?}", app.history);
    app.inspect_index_projects();
    settle(&mut app);
    let strategy = app
        .index_plan()
        .selected
        .iter()
        .find(|strategy| strategy.indexer == crate::semantic::indexing::Indexer::Clang)
        .unwrap();
    assert_eq!(strategy.evidence, ["native/build/compile_commands.json"]);
    assert!(!strategy.is_best_effort());
    assert!(!app
        .all_files
        .iter()
        .any(|file| file.path.contains("/build/")));
    app.index_projects_open = true;
    let ctx = Context::default();
    rendered_frame(&mut app, &ctx);
    let output = rendered_frame(&mut app, &ctx);
    crate::ui::test_support::text_center(&output, "native/build/compile_commands.json");
    assert!(
        !crate::ui::test_support::text_shapes(&output).any(|text| text
            .galley
            .job
            .text
            .contains("Best effort · GNU defaults"))
    );
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    assert_eq!(format!("{:?}", app.history), history);
    std::fs::remove_file(database).unwrap();
    app.inspect_index_projects();
    settle(&mut app);
    assert!(app
        .index_plan()
        .selected
        .iter()
        .find(|strategy| strategy.indexer == crate::semantic::indexing::Indexer::Clang)
        .unwrap()
        .is_best_effort());
}
