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
