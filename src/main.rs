//! Executable entry point for the Radiology desktop application.
//!
//! Handles command-line arguments, native viewport configuration via [`eframe`],
//! logging initialization, and bootstrapping the [`radiology::RadiologyApp`] state.
//!
//! # CLI Usage
//!
//! ```bash
//! # Open current directory
//! radiology .
//!
//! # Open a specific repository path
//! radiology --repository /path/to/repo
//!
//! # Open repository and immediately review a GitHub pull request
//! radiology /path/to/repo --pull-request 42 --file src/main.rs
//!
//! # Verify embedded resources and syntaxes
//! radiology --verify-resources
//! ```

use eframe::egui::Vec2;
use radiology::RadiologyApp;
use std::path::PathBuf;

fn verify_resources() -> bool {
    let mut ok = true;
    let highlighter = radiology::syntax::highlighter::SyntaxHighlighter::new();
    println!("Checking registered syntax grammars...");
    let grammars = [
        ("rust", "rs"),
        ("python", "py"),
        ("javascript", "js"),
        ("typescript", "ts"),
        ("tsx", "tsx"),
        ("c", "c"),
        ("cpp", "cpp"),
        ("go", "go"),
        ("html", "html"),
        ("json", "json"),
        ("markdown", "md"),
        ("yaml", "yaml"),
        ("toml", "toml"),
    ];

    for (name, ext) in grammars {
        let fake_path = format!("test.{ext}");
        if highlighter.find_syntax_for_path(&fake_path).is_some() {
            println!("ok   {name} (.{ext})");
        } else {
            println!("FAIL {name} (.{ext})");
            ok = false;
        }
    }

    if ok {
        println!("All registered syntax grammars verified successfully.");
    } else {
        println!("Some syntax grammars failed verification.");
    }
    ok
}

/// Application entry point configuring the native window and running the main event loop.
fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--verify-resources") {
        let success = verify_resources();
        std::process::exit(if success { 0 } else { 1 });
    }

    let mut initial_path: Option<PathBuf> = None;
    let mut initial_pr: Option<usize> = None;
    let mut initial_file: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--repository" | "-r" | "--open" | "-o" => {
                if i + 1 < args.len() {
                    initial_path = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--pull-request" | "--pr" | "-p" => {
                if i + 1 < args.len() {
                    if let Ok(num) = args[i + 1].parse::<usize>() {
                        initial_pr = Some(num);
                    }
                    i += 1;
                }
            }
            "--file" | "-f" => {
                if i + 1 < args.len() {
                    initial_file = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            arg if !arg.starts_with('-') => {
                initial_path = Some(PathBuf::from(arg));
            }
            _ => {}
        }
        i += 1;
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Radiology")
            .with_inner_size(Vec2::new(1400.0, 900.0))
            .with_min_inner_size(Vec2::new(800.0, 500.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Radiology",
        native_options,
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            let mut app = RadiologyApp::new(cc);
            app.launch(initial_path.as_deref(), initial_pr, initial_file.as_deref());
            Box::new(app)
        }),
    )
}
