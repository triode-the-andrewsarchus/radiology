//! Exercises actual app source/history transactions for every seL4 trace step.
//! Usage: cargo run --example code_path_app_probe -- /path/to/seL4
use radiology::{
    code_paths::CodePath, session::store::SessionStore, source::SourceLocation,
    ui::app::RadiologyApp,
};
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

fn settle(app: &mut RadiologyApp, ctx: &egui::Context) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(60);
    while app.is_repository_opening()
        || app.is_source_loading()
        || app.is_index_loading()
        || app.is_git_refreshing()
        || app.is_reference_loading()
        || app.is_critical_loading()
    {
        app.poll_repository(ctx);
        app.poll_source(ctx);
        app.poll_git_refresh(ctx);
        app.poll_index(ctx);
        if Instant::now() >= deadline {
            return Err("App source transaction timed out".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    Ok(())
}

fn location(app: &RadiologyApp) -> Result<SourceLocation, String> {
    let buffer = app.open_buffer.as_ref().ok_or("No source was published")?;
    Ok(buffer.location_at_byte(buffer.byte_offset(app.caret_line, app.caret_col)))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root: PathBuf = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("Pass the seL4 checkout root")?
        .canonicalize()?;
    let owner = tempfile::tempdir()?;
    let mut app =
        RadiologyApp::from_session(SessionStore::load_from(owner.path().join("session.json")));
    let ctx = egui::Context::default();
    app.open_repository(&root);
    settle(&mut app, &ctx)?;
    let trace = Arc::new(CodePath::parse(include_str!(
        "../Tests/RadiologyTests/Fixtures/code-paths/sel4-arm-boot.txt"
    ))?);
    if !app.select_code_path(Some(trace.clone())) {
        return Err("App refused trace selection".into());
    }
    let mut results = Vec::new();
    for (i, step) in trace.steps.iter().enumerate() {
        results.push(visit_step(&mut app, &ctx, step, i + 1)?);
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"root":root,"accepted":results.len(),"total":trace.steps.len(),"results":results})
        )?
    );
    Ok(())
}

fn visit_step(
    app: &mut RadiologyApp,
    ctx: &egui::Context,
    step: &radiology::code_paths::CodePathStep,
    position: usize,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Exercise the normal source-loader departure path, then every conditional variant.
    app.jump_to_location(&step.path, 1, 0);
    settle(app, ctx)?;
    let departure = location(app)?;
    app.go_to_code_path_step(step.id, 0);
    settle(app, ctx)?;
    let destinations = app.code_path_destinations().to_vec();
    if destinations.is_empty() {
        return Err(format!("Step {}: {:?}", position, app.code_path_error()).into());
    }
    let mut variants = Vec::new();
    for (variant, expected) in destinations.iter().enumerate() {
        variants.push(visit_variant(app, ctx, step, variant, expected)?);
    }
    // Send the same Back shortcut as the native reader, including exact modifiers.
    for _ in 0..destinations.len() {
        let _ = ctx.run(
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::OpenBracket,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..Default::default()
                    },
                }],
                ..Default::default()
            },
            |ctx| app.handle_shortcuts(ctx),
        );
        settle(app, ctx)?;
    }
    if location(app)? != departure {
        return Err(format!("Step {} did not restore exact departure", position).into());
    }
    Ok(serde_json::json!({"step":position,"variants":variants,"back":departure}))
}

fn visit_variant(
    app: &mut RadiologyApp,
    ctx: &egui::Context,
    step: &radiology::code_paths::CodePathStep,
    variant: usize,
    expected: &radiology::code_paths::resolver::CodePathDestination,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    if variant > 0 {
        app.go_to_code_path_step(step.id, variant);
        settle(app, ctx)?;
    }
    let actual = location(app)?;
    let buffer = app.open_buffer.as_ref().ok_or("Lost source")?;
    if app.code_path_error().is_some()
        || actual != expected.location
        || buffer.content_digest() != expected.content_digest
        || app.selected_code_path_step() != Some(step.id)
    {
        return Err(format!(
            "Step {} variant {} failed its exact destination",
            step.id, variant
        )
        .into());
    }
    Ok(serde_json::json!({"location":actual,"line":expected.line,"digest":buffer.content_digest()}))
}
