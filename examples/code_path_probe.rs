//! Runs the upstream seL4 trace against an explicitly selected checkout.
//! Usage: cargo run --example code_path_probe -- /path/to/seL4
use radiology::{
    code_paths::{resolver::resolve_step, CodePath},
    source::loader::SourceRead,
};
use std::{path::PathBuf, sync::atomic::AtomicBool};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("Pass the seL4 checkout root")?,
    )
    .canonicalize()?;
    let trace = CodePath::parse(include_str!(
        "../Tests/RadiologyTests/Fixtures/code-paths/sel4-arm-boot.txt"
    ))?;
    let source = SourceRead::Checkout(root.clone());
    let mut accepted = 0;
    let mut results = Vec::new();
    for (position, step) in trace.steps.iter().enumerate() {
        let result = match resolve_step(step, &source, None, &AtomicBool::new(false)) {
            Ok(resolved) => {
                accepted += 1;
                let destinations: Vec<_> = resolved.destinations.iter().map(|destination| serde_json::json!({
                    "location": destination.location, "line": destination.line,
                    "source_line": resolved.buffer.line_text(destination.line), "digest": destination.content_digest,
                })).collect();
                serde_json::json!({"step": position+1, "path": step.path, "selector": step.function, "destinations": destinations})
            }
            Err(error) => {
                serde_json::json!({"step": position+1, "path": step.path, "selector": step.function, "error": error.to_string()})
            }
        };
        results.push(result);
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"root": root, "accepted": accepted, "total": trace.steps.len(), "results": results})
        )?
    );
    if accepted != trace.steps.len() {
        return Err("Not every authored step resolved".into());
    }
    Ok(())
}
