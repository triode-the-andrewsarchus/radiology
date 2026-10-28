//! Semantic import publication and its repository-scoped status controls.

use super::*;
use crate::semantic::indexing::IndexPlan;
use crate::semantic::loader::IndexImportStatus;

impl RadiologyApp {
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
        if self.is_index_loading() {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
        }
    }

    /// Returns whether the current repository import is still running.
    #[must_use]
    pub fn is_index_loading(&self) -> bool {
        self.index_importer.is_running() || self.index_plan_job.is_running()
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
                ui.menu_button("Index Projects", |ui| {
                    self.show_index_projects(ui);
                });
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
        if let Some(command) = command {
            self.execute_command(ctx, command);
        }
    }

    fn show_index_projects(&self, ui: &mut egui::Ui) {
        if self.index_plan_job.is_running() {
            ui.label("Inspecting build files…");
            return;
        }
        if self.index_plan.selected.is_empty() {
            ui.label("No supported build files found");
            return;
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
                        "{} · {} source files",
                        strategy.indexer.languages().join(", "),
                        strategy.source_files
                    ));
                    ui.label(strategy.evidence.join(", "));
                    ui.separator();
                }
                if !self.index_plan.skipped.is_empty() {
                    ui.label(format!(
                        "{} additional projects exceed the 32-project run limit",
                        self.index_plan.skipped.len()
                    ));
                }
            });
        ui.label("Index generation is unavailable in this build.");
    }
}
