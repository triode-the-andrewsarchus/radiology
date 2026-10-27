use eframe::egui::Vec2;
use radiology::RadiologyApp;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let mut initial_path: Option<PathBuf> = None;
    let mut initial_pr: Option<usize> = None;

    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--open" | "-o" => {
                if i + 1 < args.len() {
                    initial_path = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--pr" | "-p" => {
                if i + 1 < args.len() {
                    if let Ok(num) = args[i + 1].parse::<usize>() {
                        initial_pr = Some(num);
                    }
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
            let mut app = RadiologyApp::new(cc);
            if let Some(path) = initial_path {
                app.open_repository(&path);
            }
            if let Some(pr_num) = initial_pr {
                if let Some(root) = &app.repository_root {
                    if let Ok(pr) =
                        radiology::pull_request::GitHubClient::view_pull_request(root, pr_num)
                    {
                        app.active_pr = Some(pr);
                        if let Ok(diff) =
                            radiology::pull_request::GitHubClient::get_pr_diff(root, pr_num)
                        {
                            app.line_changes =
                                Some(radiology::pull_request::UnifiedDiffParser::parse(&diff));
                        }
                    }
                }
            }
            Box::new(app)
        }),
    )
}
