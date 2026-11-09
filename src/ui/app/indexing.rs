//! Semantic import publication and its repository-scoped status controls.

use super::*;
use crate::semantic::indexing::service::{CacheLoadStatus, GenerationReport, GenerationRequest};
use crate::semantic::indexing::tools::{ToolAvailability, ToolDirectory};
use crate::semantic::indexing::{IndexPlan, Indexer};
use crate::semantic::loader::IndexImportStatus;

pub(super) mod personality;
use personality::Intent;

impl RadiologyApp {
    pub(super) fn reset_index_state(&mut self) {
        self.critical_lens.clear();
        self.critical_annotations.stop();
        self.critical_graph_inputs = None;
        self.index_importer = Default::default();
        self.index_generator = Default::default();
        self.tool_provisioner = Default::default();
        self.generation_report = None;
        self.index_personality = Default::default();
        self.index_plan_job.stop();
        self.tool_probe_job.stop();
        self.index_plan = Default::default();
        self.index_plan_error = None;
        self.tool_availability.clear();
        self.semantic_index = None;
        self.refresh_symbol_context();
    }

    pub(super) fn generate_index(&mut self, reuse: bool) {
        if !self.repository_interaction_ready()
            || self.index_generator.is_running()
            || self.tool_provisioner.is_running()
        {
            return;
        }
        self.prepare_index_personality(Intent::Generate(reuse));
    }

    pub(super) fn install_indexer(&mut self, indexer: Indexer) {
        let Some(root) = self.repository_root.clone() else {
            return;
        };
        self.error = None;
        self.index_personality.cancel();
        self.index_personality.cancelled = false;
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
                self.generate_index(false);
            }
            Some(Err(error)) => self.error = Some(format!("Tool installation failed: {error}")),
            None => {}
        }
    }

    fn index_request(&self) -> Option<GenerationRequest> {
        let attached = self.repository_root.as_ref()?;
        let cache = self.session.repository_cache(attached)?.join("indexes");
        Some(GenerationRequest {
            settings: Some(self.index_personality.settings.clone()?),
            attached_root: attached.clone(),
            reuse: false,
            reuse_sources: Vec::new(),
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
        self.prepare_index_personality(Intent::Load);
    }

    fn import_repository_index(&mut self) {
        if let Some(root) = &self.repository_root {
            self.index_importer.start_with_cache(
                root.clone(),
                self.session
                    .repository_cache(root)
                    .map(|cache| cache.join("indexes")),
            );
        }
    }

    /// Returns cache loading status, including an import fallback's original error.
    #[must_use]
    pub fn index_cache_status(&self) -> &CacheLoadStatus {
        self.index_personality
            .cache_status()
            .unwrap_or_else(|| self.index_generator.cache_status())
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
        if self.index_personality.job.is_running() || self.index_personality.error.is_some() {
            return;
        }
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

    /// Reloads personality settings and inspects projects on workers without generating an index.
    pub fn inspect_index_projects(&mut self) {
        self.prepare_index_personality(Intent::Inspect);
    }

    fn inspect_captured_index_projects(&mut self) {
        self.index_plan_error = None;
        let Some(root) = self.search_root() else {
            self.index_plan_job.stop();
            self.index_plan = Default::default();
            return;
        };
        let review = self.review.clone();
        let settings = self.index_personality.settings.clone();
        self.index_plan_job.spawn(move |cancel| {
            crate::semantic::indexing::service::inspect_inputs(&root, review.as_deref(), cancel)
                .map(|inputs| {
                    settings.as_ref().map_or_else(
                        || IndexPlan::inspect(inputs.paths.iter().map(String::as_str)),
                        |settings| settings.plan(inputs.paths.iter().map(String::as_str)),
                    )
                })
        });
    }

    /// Returns the unfiltered repository's detected indexing projects.
    #[must_use]
    pub fn index_plan(&self) -> &IndexPlan {
        &self.index_plan
    }

    /// Publishes background index imports and refreshes the active symbol context.
    /// Embedded hosts may drive this independently of the eframe loop.
    pub fn poll_index(&mut self, ctx: &Context) {
        self.poll_index_personality();
        if let Some(index) = self.index_importer.poll() {
            self.semantic_index = Some(Arc::new(index));
            self.refresh_symbol_context();
            self.refresh_critical_projection();
        }
        let inspected = match self.index_plan_job.poll() {
            Ok(result) => result,
            Err(_) => Some(Err("Inspection worker stopped unexpectedly".into())),
        };
        if let Some(result) = inspected {
            match result {
                Ok(plan) => {
                    self.index_plan = plan;
                    self.index_plan_error = None;
                }
                Err(error) => {
                    self.index_plan = Default::default();
                    self.error = Some(format!("Cannot inspect build projects: {error}"));
                    self.index_plan_error = Some(error);
                }
            }
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
        if let Some(mut generated) = self.index_generator.poll() {
            if !generated.report.cached {
                if let Some(warning) = self.index_personality.cache_warning.take() {
                    generated.report.cache_warning = Some(match generated.report.cache_warning {
                        Some(other) => format!("{warning}\n{other}"),
                        None => warning,
                    });
                }
            }
            if let Some(index) = generated.index {
                self.semantic_index = Some(Arc::new(index));
                self.refresh_symbol_context();
                self.refresh_critical_projection();
            }
            if !generated.report.cached {
                self.index_plan = generated.report.plan.clone();
                self.index_plan_error = None;
            }
            self.generation_report = Some(generated.report);
        }
        self.advance_index_open();
    }

    /// Returns whether import, project inspection or tool probing is running.
    #[must_use]
    pub fn is_index_loading(&self) -> bool {
        self.index_personality.job.is_running()
            || self.tool_provisioner.is_running()
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
                command = self.show_index_status_actions(ui);
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

    fn show_index_status_actions(&mut self, ui: &mut egui::Ui) -> Option<ToolbarAction> {
        let mut command = None;
        self.show_generation_status(ui);
        if ui.small_button("Index Projects").clicked() {
            self.index_projects_open = true;
        }
        if self.tool_provisioner.is_running() {
            if ui.small_button("Cancel Installation").clicked() {
                command = Some(ToolbarAction::CancelToolInstallation);
            }
        } else if self.index_importer.is_running()
            || self.index_personality.job.is_running()
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
        command
    }

    fn show_generation_status(&mut self, ui: &mut egui::Ui) {
        if self.show_personality_status(ui) {
            return;
        }
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
        if let std::ops::ControlFlow::Break(action) = self.show_personality_controls(ui) {
            return action;
        }
        if self.index_plan_job.is_running() {
            ui.label("Inspecting build files…");
            return None;
        }
        if self.index_plan_error.is_some() {
            ui.label("Build project inspection failed. Reload the repository to retry.");
            return None;
        }
        let install = self.show_index_project_list(ui);
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
            ui.label("Index available · cache warning")
                .on_hover_text(warning);
        }
        if let Some(warning) = self
            .generation_report
            .as_ref()
            .and_then(GenerationReport::reuse_warning)
        {
            ui.label(warning);
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
        if let Some(action) = self.show_generation_controls(ui) {
            return Some(action);
        }
        let command = self.show_tool_check_controls(ui);
        if self.review.is_some() {
            ui.label("Indexing uses the pinned After revision. Before remains source-only.");
        }
        command
    }

    fn show_index_project_list(&self, ui: &mut egui::Ui) -> Option<ToolbarAction> {
        if self.index_plan.selected.is_empty() {
            ui.label("No build projects detected in the current inventory.");
        }
        let mut install = None;
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                for strategy in &self.index_plan.selected {
                    install = self.show_index_project(ui, strategy).or(install.take());
                }
                if !self.index_plan.uncovered_languages.is_empty() {
                    crate::ui::text_fit::explanation(
                        ui,
                        &format!(
                            "Languages without a selected indexing project: {}",
                            self.index_plan.uncovered_languages.join(", ")
                        ),
                        self.theme.text_secondary,
                    );
                }
                if !self.index_plan.skipped.is_empty() {
                    ui.label(format!(
                        "{} additional projects exceed the {}-project run limit",
                        self.index_plan.skipped.len(),
                        self.index_personality
                            .settings
                            .as_ref()
                            .map_or(32, |settings| settings.project_limit())
                    ));
                }
            });
        install
    }

    fn show_generation_controls(&self, ui: &mut egui::Ui) -> Option<ToolbarAction> {
        let label = match (self.review.is_some(), self.generation_report.is_some()) {
            (true, true) => "Update After Index",
            (true, false) => "Generate After Index",
            (false, true) => "Update Index",
            (false, false) => "Generate Index",
        };
        let mut action = None;
        ui.horizontal(|ui| {
            if ui.button(label).on_hover_text("Reuse projects with exact inputs and matching tool versions; generate the rest.").clicked() {
                action = Some(ToolbarAction::UpdateIndex);
            }
            if self.generation_report.is_some()
                && ui.button("Reindex").on_hover_text("Regenerate every project, bypassing reuse (Ctrl+Shift+I).").clicked()
            {
                action = Some(ToolbarAction::GenerateIndex);
            }
        });
        action
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
            if (!matches!(indexer, Indexer::Clang | Indexer::Sel4)
                || cfg!(all(target_os = "linux", target_arch = "x86_64")))
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
            if strategy.is_best_effort() {
                ui.label(
                    egui::RichText::new("Best effort · GNU defaults")
                        .small()
                        .color(self.theme.text_secondary),
                );
                crate::ui::text_fit::explanation(
                    ui,
                    crate::semantic::indexing::clang_fallback::EXPLANATION,
                    self.theme.text_secondary,
                );
            } else {
                ui.label(strategy.evidence.join(", "));
            }
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
                Self::show_project_outcome(ui, outcome);
            }
            ui.separator();
            action
        })
        .inner
    }
    fn show_project_outcome(
        ui: &mut egui::Ui,
        outcome: &crate::semantic::indexing::service::StrategyOutcome,
    ) {
        if let Some(origin) = &outcome.reused_from {
            ui.label(format!("Reused from {origin}"));
        }
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
        Self::show_project_details(ui, outcome);
    }

    fn show_project_details(
        ui: &mut egui::Ui,
        outcome: &crate::semantic::indexing::service::StrategyOutcome,
    ) {
        if !outcome.diagnostics.is_empty()
            || !outcome.missing_paths.is_empty()
            || outcome.cache_warning.is_some()
            || outcome.command_line.is_some()
        {
            ui.collapsing("Generation Details", |ui| {
                if let Some(command) = &outcome.command_line {
                    if ui
                        .small_button("Copy Command")
                        .on_hover_text(
                            "Copy the recorded invocation; private input files may no longer exist",
                        )
                        .clicked()
                    {
                        ui.output_mut(|output| output.copied_text = command.clone());
                    }
                }
                egui::ScrollArea::both().max_height(160.0).show(ui, |ui| {
                    if let Some(directory) = &outcome.working_directory {
                        ui.label(format!("Working directory: {}", directory.display()));
                    }
                    if let Some(command) = &outcome.command_line {
                        ui.add(
                            egui::Label::new(egui::RichText::new(command).monospace()).wrap(false),
                        );
                    }
                    if let Some(warning) = &outcome.cache_warning {
                        ui.label(format!("Cache: {warning}"));
                    }
                    for path in &outcome.missing_paths {
                        ui.label(format!("Not indexed: {path}"));
                    }
                    ui.add(
                        egui::Label::new(egui::RichText::new(&outcome.diagnostics).monospace())
                            .wrap(false),
                    );
                });
            });
        }
    }
}
