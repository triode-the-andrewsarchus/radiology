//! Semantic import publication and its repository-scoped status controls.

use super::*;
use crate::semantic::indexing::service::{CacheLoadStatus, GenerationReport, GenerationRequest};
use crate::semantic::indexing::tools::{ToolAvailability, ToolDirectory};
use crate::semantic::indexing::{IndexPlan, Indexer};
use crate::semantic::loader::IndexImportStatus;

impl RadiologyApp {
    pub(super) fn reset_index_state(&mut self) {
        self.critical_lens.stop();
        self.critical_annotations.stop();
        self.critical_graph_inputs = None;
        self.index_importer = Default::default();
        self.index_generator = Default::default();
        self.tool_provisioner = Default::default();
        self.generation_report = None;
        self.index_plan_job.stop();
        self.tool_probe_job.stop();
        self.index_plan = Default::default();
        self.tool_availability.clear();
        self.semantic_index = None;
        self.refresh_symbol_context();
    }

    pub(super) fn generate_index(&mut self) {
        if self.tool_provisioner.is_running() {
            return;
        }
        let Some(request) = self.index_request() else {
            self.error = Some("No private index cache is available for this repository".into());
            return;
        };
        self.start_index_generation(request);
    }

    pub(super) fn install_indexer(&mut self, indexer: Indexer) {
        let Some(root) = self.repository_root.clone() else {
            return;
        };
        self.error = None;
        self.index_generator.stop();
        self.index_importer.stop();
        self.tool_probe_job.stop();
        let mut protected_roots = vec![root];
        if let Some(review) = &self.review {
            protected_roots.push(review.head_root.clone());
        }
        self.tool_provisioner.start(
            crate::semantic::indexing::provisioning::ProvisioningRequest {
                indexer,
                tools: self.index_tools.clone(),
                protected_roots,
                timeout: std::time::Duration::from_secs(1800),
            },
        );
    }

    fn poll_tool_installation(&mut self) {
        match self.tool_provisioner.poll() {
            Some(Ok(installed)) => {
                self.tool_availability
                    .insert(installed.indexer, installed.availability);
                self.generate_index();
            }
            Some(Err(error)) => self.error = Some(format!("Tool installation failed: {error}")),
            None => {}
        }
    }

    fn index_request(&self) -> Option<GenerationRequest> {
        let attached = self.repository_root.as_ref()?;
        let cache = self.session.repository_cache(attached)?.join("indexes");
        Some(GenerationRequest {
            root: self
                .review
                .as_ref()
                .map_or_else(|| attached.clone(), |review| review.head_root.clone()),
            review: self.review.clone(),
            cache,
            tools: self.index_tools.clone(),
            timeout: std::time::Duration::from_secs(600),
            limits: Default::default(),
        })
    }

    pub(super) fn load_repository_index(&mut self) {
        self.tool_provisioner = Default::default();
        self.index_importer.stop();
        self.index_generator.stop();
        self.generation_report = None;
        if let Some(request) = self.index_request() {
            self.index_generator.start_cached(request);
        } else if self.review.is_none() {
            if let Some(root) = &self.repository_root {
                self.index_importer.start(root.clone());
            }
        }
    }

    /// Returns cache loading status, including an import fallback's original error.
    #[must_use]
    pub fn index_cache_status(&self) -> &CacheLoadStatus {
        self.index_generator.cache_status()
    }

    pub(super) fn start_index_generation(&mut self, request: GenerationRequest) {
        self.tool_provisioner = Default::default();
        self.index_importer.stop();
        self.index_plan_job.stop();
        self.tool_probe_job.stop();
        self.index_generator.start(request);
    }

    /// Returns coverage, diagnostics and provenance for the last completed run.
    #[must_use]
    pub fn generation_report(&self) -> Option<&GenerationReport> {
        self.generation_report.as_ref()
    }

    pub(super) fn check_index_tools(&mut self) {
        let Some(root) = self.search_root() else {
            return;
        };
        let tools = self.index_tools.clone();
        self.probe_index_tools(tools, root);
    }

    pub(super) fn probe_index_tools(&mut self, tools: ToolDirectory, root: PathBuf) {
        let indexers: std::collections::BTreeSet<_> = self
            .index_plan
            .selected
            .iter()
            .map(|strategy| strategy.indexer)
            .collect();
        let review = self
            .review
            .clone()
            .filter(|review| review.head_root == root);
        self.tool_probe_job.spawn(move |cancel| {
            let _review = review;
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
            self.semantic_index = Some(Arc::new(index));
            self.refresh_symbol_context();
            self.refresh_critical_projection();
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
        self.poll_tool_installation();
        self.poll_generated_index();
        if self.is_index_loading() {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
        }
    }

    fn poll_generated_index(&mut self) {
        let loading_cache = self.index_cache_status() == &CacheLoadStatus::Loading;
        if let Some(generated) = self.index_generator.poll() {
            if let Some(index) = generated.index {
                self.semantic_index = Some(Arc::new(index));
                self.refresh_symbol_context();
                self.refresh_critical_projection();
            }
            self.index_plan = generated.report.plan.clone();
            self.generation_report = Some(generated.report);
        }
        if loading_cache
            && self.review.is_none()
            && matches!(
                self.index_cache_status(),
                CacheLoadStatus::Missing | CacheLoadStatus::Failed(_)
            )
        {
            if let Some(root) = &self.repository_root {
                self.index_importer.start(root.clone());
            }
        }
    }

    /// Returns whether import, project inspection or tool probing is running.
    #[must_use]
    pub fn is_index_loading(&self) -> bool {
        self.tool_provisioner.is_running()
            || self.index_importer.is_running()
            || self.index_plan_job.is_running()
            || self.tool_probe_job.is_running()
            || self.index_generator.is_running()
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
                self.show_generation_status(ui);
                if ui.small_button("Index Projects").clicked() {
                    self.index_projects_open = true;
                }
                if self.tool_provisioner.is_running() {
                    if ui.small_button("Cancel Installation").clicked() {
                        command = Some(ToolbarAction::CancelToolInstallation);
                    }
                } else if self.index_importer.is_running()
                    || self.index_cache_status() == &CacheLoadStatus::Loading
                {
                    if ui.small_button("Cancel").clicked() {
                        command = Some(ToolbarAction::CancelIndexImport);
                    }
                } else if ui
                    .add_enabled(
                        !self.index_generator.is_running(),
                        egui::Button::new("Reload Index").small(),
                    )
                    .clicked()
                {
                    command = Some(ToolbarAction::ReloadIndexImport);
                }
                if self.semantic_index.is_some()
                    && self.generation_report.is_none()
                    && !matches!(
                        self.index_importer.status(),
                        IndexImportStatus::Imported { .. }
                    )
                {
                    ui.label("Previous semantic data retained · freshness unknown");
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

    fn show_generation_status(&mut self, ui: &mut egui::Ui) {
        if self.show_installation_status(ui) {
            return;
        }
        if self.index_generator.is_running() {
            let progress = self.index_generator.progress();
            ui.spinner();
            let label = if self.index_cache_status() == &CacheLoadStatus::Loading {
                "Loading cached index…".to_owned()
            } else {
                format!(
                    "Indexing {}/{} · {}",
                    progress.step,
                    progress.total,
                    progress.project.as_deref().unwrap_or("repository")
                )
            };
            ui.label(label).on_hover_text(progress.message);
        } else if let CacheLoadStatus::Failed(error) = self.index_cache_status() {
            ui.label("Cached index unavailable").on_hover_text(error);
            self.show_import_status(ui);
        } else if let Some(error) = self.index_generator.error() {
            ui.label("Index generation failed").on_hover_text(error);
            if ui.small_button("Details").clicked() {
                self.error = Some(error.to_owned());
            }
        } else if self.index_generator.was_cancelled() {
            let operation = if self.index_cache_status() == &CacheLoadStatus::Cancelled {
                "Index loading cancelled"
            } else {
                "Index generation cancelled"
            };
            ui.label(if self.semantic_index.is_some() {
                format!("{operation} · previous data retained")
            } else {
                operation.to_owned()
            });
        } else if let Some(report) = &self.generation_report {
            if report.documents == 0 && self.semantic_index.is_some() {
                ui.label("Previous semantic data retained");
            }
            if let Some(revision) = &report.revision {
                ui.label("After index").on_hover_text(revision);
            }
            ui.label(report.summary()).on_hover_text(
                report.stale_reason.as_deref().unwrap_or(
                    "Source coverage and tool diagnostics are available in Index Projects",
                ),
            );
            self.show_index_source_status(ui);
        } else {
            self.show_import_status(ui);
        }
    }

    fn show_installation_status(&self, ui: &mut egui::Ui) -> bool {
        if self.tool_provisioner.is_running() {
            ui.spinner();
            ui.label("Installing indexer…")
                .on_hover_text(self.tool_provisioner.progress());
            true
        } else if self.tool_provisioner.was_cancelled() {
            ui.label(if self.semantic_index.is_some() {
                "Tool installation cancelled · previous semantic data retained"
            } else {
                "Tool installation cancelled"
            });
            true
        } else {
            false
        }
    }

    fn show_index_source_status(&mut self, ui: &mut egui::Ui) {
        if self
            .open_buffer
            .as_ref()
            .zip(self.semantic_index.as_ref())
            .is_some_and(|(buffer, index)| {
                buffer.revision != Some(ReviewRevision::Before)
                    && index.source_digests.contains_key(&buffer.path)
                    && !index.matches_buffer(buffer)
            })
        {
            ui.label("Open source differs from index");
            if ui.small_button("Reload Source").clicked() {
                self.execute_command(ui.ctx(), ToolbarAction::ReloadSource);
            }
        }
    }

    fn show_import_status(&mut self, ui: &mut egui::Ui) {
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
    }

    fn show_index_projects(&self, ui: &mut egui::Ui) -> Option<ToolbarAction> {
        if self.index_plan_job.is_running() {
            ui.label("Inspecting build files…");
            return None;
        }
        if self.index_plan.selected.is_empty() {
            ui.label("Generate Index to check for supported build projects.");
        }
        let mut install = None;
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                for strategy in &self.index_plan.selected {
                    install = self.show_index_project(ui, strategy).or(install.take());
                }
                if !self.index_plan.skipped.is_empty() {
                    ui.label(format!(
                        "{} additional projects exceed the 32-project run limit",
                        self.index_plan.skipped.len()
                    ));
                }
            });
        if self.tool_provisioner.is_running() {
            ui.label(self.tool_provisioner.progress());
            return ui
                .button("Cancel Installation")
                .clicked()
                .then_some(ToolbarAction::CancelToolInstallation);
        }
        if install.is_some() {
            return install;
        }
        if let Some(warning) = self
            .generation_report
            .as_ref()
            .and_then(|report| report.cache_warning.as_deref())
        {
            ui.label("Index is available for this session; saving its cache failed.")
                .on_hover_text(warning);
        }
        if self.index_generator.is_running() {
            ui.label(self.index_generator.progress().message);
            let label = if self.index_cache_status() == &CacheLoadStatus::Loading {
                "Cancel Load"
            } else {
                "Cancel Generation"
            };
            return ui
                .button(label)
                .clicked()
                .then_some(ToolbarAction::CancelIndexGeneration);
        }
        if ui
            .button(if self.review.is_some() {
                "Generate After Index"
            } else if self.generation_report.is_some() {
                "Reindex"
            } else {
                "Generate Index"
            })
            .clicked()
        {
            return Some(ToolbarAction::GenerateIndex);
        }
        let command = self.show_tool_check_controls(ui);
        if self.review.is_some() {
            ui.label("Indexing uses the pinned After revision. Before remains source-only.");
        }
        command
    }
    fn show_tool_check_controls(&self, ui: &mut egui::Ui) -> Option<ToolbarAction> {
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
        None
    }

    fn show_indexer_install(
        &self,
        ui: &mut egui::Ui,
        indexer: Indexer,
        outcome: Option<&crate::semantic::indexing::service::StrategyOutcome>,
    ) -> Option<ToolbarAction> {
        let mut action = None;
        let missing = matches!(
            self.tool_availability.get(&indexer),
            Some(ToolAvailability::Unavailable { .. })
        ) || outcome
            .is_some_and(|outcome| outcome.version.is_none() && outcome.failure.is_some());
        if missing {
            ui.label(indexer.installation_hint());
            if indexer != Indexer::Clang
                && ui
                    .add_enabled(
                        !self.tool_provisioner.is_running()
                            && !self.index_generator.is_running()
                            && self.index_tools.managed.is_some(),
                        egui::Button::new("Install and Index"),
                    )
                    .clicked()
            {
                action = Some(ToolbarAction::InstallIndexer(indexer));
            }
        }
        action
    }

    fn show_index_project(
        &self,
        ui: &mut egui::Ui,
        strategy: &crate::semantic::indexing::IndexingStrategy,
    ) -> Option<ToolbarAction> {
        ui.push_id(strategy.id(), |ui| {
            let project = if strategy.directory.is_empty() {
                "."
            } else {
                &strategy.directory
            };
            ui.label(
                egui::RichText::new(format!("{} · {project}", strategy.indexer.id())).strong(),
            );
            ui.label(format!(
                "{} · {} source file{}",
                strategy.indexer.languages().join(", "),
                strategy.source_files,
                if strategy.source_files == 1 { "" } else { "s" }
            ));
            ui.label(strategy.evidence.join(", "));
            let outcome = self.generation_report.as_ref().and_then(|report| {
                report
                    .outcomes
                    .iter()
                    .find(|outcome| outcome.strategy == *strategy)
            });
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
                None if outcome.is_none_or(|outcome| outcome.version.is_none()) => {
                    ui.label("Tool availability unchecked");
                }
                None => {}
            }
            let action = self.show_indexer_install(ui, strategy.indexer, outcome);
            if let Some(outcome) = outcome {
                ui.label(format!(
                    "Last run: {} document{} · {} source file{} missing",
                    outcome.documents,
                    if outcome.documents == 1 { "" } else { "s" },
                    outcome.missing_count,
                    if outcome.missing_count == 1 { "" } else { "s" }
                ));
                if let Some(version) = &outcome.version {
                    ui.label(format!("Index tool: {version}"));
                }
                if let Some(failure) = &outcome.failure {
                    ui.label(failure);
                }
                if !outcome.diagnostics.is_empty() || !outcome.missing_paths.is_empty() {
                    ui.collapsing("Generation Details", |ui| {
                        egui::ScrollArea::both().max_height(160.0).show(ui, |ui| {
                            for path in &outcome.missing_paths {
                                ui.label(format!("Not indexed: {path}"));
                            }
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&outcome.diagnostics).monospace(),
                                )
                                .wrap(false),
                            );
                        });
                    });
                }
            }
            ui.separator();
            action
        })
        .inner
    }
}
