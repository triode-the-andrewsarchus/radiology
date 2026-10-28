//! Semantic import publication and its repository-scoped status controls.

use super::*;
use crate::semantic::loader::IndexImportStatus;

impl RadiologyApp {
    /// Publishes background index imports and refreshes the active symbol context.
    /// Embedded hosts may drive this independently of the eframe loop.
    pub fn poll_index(&mut self, ctx: &Context) {
        if let Some(index) = self.index_importer.poll() {
            self.semantic_index = Some(index);
            self.refresh_symbol_context();
        }
        if self.index_importer.is_running() {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
        }
    }

    /// Returns whether the current repository import is still running.
    #[must_use]
    pub fn is_index_loading(&self) -> bool {
        self.index_importer.is_running()
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
}
