//! Semantic import publication and its repository-scoped status controls.

use super::*;
use crate::semantic::indexing::tools::{ToolAvailability, ToolDirectory};
use crate::semantic::indexing::{IndexPlan, Indexer};
use crate::semantic::loader::IndexImportStatus;

impl RadiologyApp {
    pub(super) fn reset_index_state(&mut self) {
        self.index_importer = Default::default();
        self.index_plan_job.stop();
        self.tool_probe_job.stop();
        self.index_plan = Default::default();
        self.tool_availability.clear();
        self.semantic_index = None;
    }

    pub(super) fn check_index_tools(&mut self) {
        let Some(root) = self.search_root() else {
            return;
        };
        let tools = ToolDirectory::from_environment(
            self.session
                .config_directory()
                .map(|path| path.join("tools")),
        );
        self.probe_index_tools(tools, root);
    }

    pub(super) fn probe_index_tools(&mut self, tools: ToolDirectory, root: PathBuf) {
        let indexers: std::collections::BTreeSet<_> = self
            .index_plan
            .selected
            .iter()
            .map(|strategy| strategy.indexer)
            .collect();
        self.tool_probe_job.spawn(move |cancel| {
            indexers
                .into_iter()
                .take_while(|_| !cancel.load(std::sync::atomic::Ordering::Relaxed))
                .map(|indexer| {
                    let status =
                        tools.probe(indexer, &root, cancel, std::time::Duration::from_secs(10));
                    (indexer, status)
                })
                .collect()
        });
    }

    /// Returns the most recent executable probe for one detected indexing tool.
    #[must_use]
    pub fn tool_availability(&self, indexer: Indexer) -> Option<&ToolAvailability> {
        self.tool_availability.get(&indexer)
    }

    pub(super) fn inspect_index_projects(&mut self) {
        let paths: Vec<_> = self
            .all_files
            .iter()
            .map(|file| file.path.clone())
            .collect();
        self.index_plan_job
            .spawn(move |_| IndexPlan::inspect(paths.iter().map(String::as_str)));
    }

    /// Returns the unfiltered repository's detected indexing projects.
    #[must_use]
    pub fn index_plan(&self) -> &IndexPlan {
        &self.index_plan
    }

    /// Publishes background index imports and refreshes the active symbol context.
    /// Embedded hosts may drive this independently of the eframe loop.
    pub fn poll_index(&mut self, ctx: &Context) {
        if let Some(index) = self.index_importer.poll() {
            self.semantic_index = Some(index);
            self.refresh_symbol_context();
        }
        match self.index_plan_job.poll() {
            Ok(Some(plan)) => self.index_plan = plan,
            Ok(None) => {}
            Err(_) => self.error = Some("Index project inspection stopped unexpectedly".into()),
        }
        match self.tool_probe_job.poll() {
            Ok(Some(availability)) => self.tool_availability = availability,
            Ok(None) => {}
            Err(_) => self.error = Some("Indexer tool checks stopped unexpectedly".into()),
        }
        if self.is_index_loading() {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
        }
    }

    /// Returns whether import, project inspection or tool probing is running.
    #[must_use]
    pub fn is_index_loading(&self) -> bool {
        self.index_importer.is_running()
            || self.index_plan_job.is_running()
            || self.tool_probe_job.is_running()
    }

    /// Returns lifecycle and diagnostic information for the current import.
    #[must_use]
    pub fn index_import_status(&self) -> &IndexImportStatus {
        self.index_importer.status()
    }

    pub(super) fn show_index_status(&mut self, ctx: &Context) {
        if self.repository_root.is_none() {
            return;
        }
        let mut command = None;
        TopBottomPanel::bottom("semantic_index_status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                match self.index_importer.status() {
                    IndexImportStatus::Loading => {
                        ui.spinner();
                        ui.label("Loading semantic index…");
                    }
                    IndexImportStatus::Imported { path, documents } => {
                        ui.label(format!(
                            "Imported index · {documents} documents · freshness unknown"
                        ))
                        .on_hover_text(path.display().to_string());
                    }
                    IndexImportStatus::Failed(error) => {
                        ui.colored_label(self.theme.text_secondary, "Index import failed")
                            .on_hover_text(error);
                        if ui.small_button("Details").clicked() {
                            self.error = Some(error.clone());
                        }
                    }
                    IndexImportStatus::Cancelled => {
                        ui.label("Index import cancelled");
                    }
                    IndexImportStatus::Idle | IndexImportStatus::Missing => {
                        ui.label(if self.review.is_some() {
                            "Review has no generated semantic index"
                        } else {
                            "No semantic index · source navigation available"
                        });
                    }
                }
                if ui.small_button("Index Projects").clicked() {
                    self.index_projects_open = true;
                }
                if self.index_importer.is_running() {
                    if ui.small_button("Cancel").clicked() {
                        command = Some(ToolbarAction::CancelIndexImport);
                    }
                } else if self.review.is_none() && ui.small_button("Reload Index").clicked() {
                    command = Some(ToolbarAction::ReloadIndexImport);
                }
                if self.semantic_index.is_some()
                    && !matches!(
                        self.index_importer.status(),
                        IndexImportStatus::Imported { .. }
                    )
                {
                    ui.label("Previous imported data retained · freshness unknown");
                }
            });
        });
        if self.index_projects_open {
            let mut open = true;
            egui::Window::new("Index Projects")
                .open(&mut open)
                .min_width(360.0)
                .default_width(420.0)
                .show(ctx, |ui| {
                    if let Some(action) = self.show_index_projects(ui) {
                        command = Some(action);
                    }
                });
            self.index_projects_open = open;
        }
        if let Some(command) = command {
            self.execute_command(ctx, command);
        }
    }

    fn show_index_projects(&self, ui: &mut egui::Ui) -> Option<ToolbarAction> {
        if self.index_plan_job.is_running() {
            ui.label("Inspecting build files…");
            return None;
        }
        if self.index_plan.selected.is_empty() {
            ui.label("No supported build files found");
            return None;
        }
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                for strategy in &self.index_plan.selected {
                    let project = if strategy.directory.is_empty() {
                        "."
                    } else {
                        &strategy.directory
                    };
                    ui.label(
                        egui::RichText::new(format!("{} · {project}", strategy.indexer.id()))
                            .strong(),
                    );
                    ui.label(format!(
                        "{} · {} source file{}",
                        strategy.indexer.languages().join(", "),
                        strategy.source_files,
                        if strategy.source_files == 1 { "" } else { "s" }
                    ));
                    ui.label(strategy.evidence.join(", "));
                    match self.tool_availability.get(&strategy.indexer) {
                        Some(ToolAvailability::Available {
                            executable,
                            version,
                        }) => {
                            ui.label(version)
                                .on_hover_text(executable.display().to_string());
                        }
                        Some(ToolAvailability::Unavailable {
                            reason,
                            diagnostics,
                            ..
                        }) => {
                            ui.label(reason).on_hover_text(diagnostics);
                        }
                        None => {
                            ui.label("Tool availability unchecked");
                        }
                    }
                    ui.separator();
                }
                if !self.index_plan.skipped.is_empty() {
                    ui.label(format!(
                        "{} additional projects exceed the 32-project run limit",
                        self.index_plan.skipped.len()
                    ));
                }
            });
        if self.tool_probe_job.is_running() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Checking tool versions…");
            });
            if ui.button("Cancel Checks").clicked() {
                return Some(ToolbarAction::CancelIndexToolCheck);
            }
        } else if ui.button("Check Tools").clicked() {
            return Some(ToolbarAction::CheckIndexTools);
        }
        ui.label("Index generation is unavailable in this build.");
        None
    }
}
