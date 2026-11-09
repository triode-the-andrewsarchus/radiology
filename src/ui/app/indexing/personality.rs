use super::*;
use crate::semantic::indexing::personality::{Personality, PersonalityStore, Settings};

#[derive(Clone, Copy)]
pub(in crate::ui::app) enum Intent {
    Load,
    Import,
    Inspect,
    Generate(bool),
}

#[derive(Default)]
pub(in crate::ui::app) struct State {
    pub selected: Personality,
    pub settings: Option<Arc<Settings>>,
    pub error: Option<String>,
    pub cancelled: bool,
    pub cache_warning: Option<String>,
    cancelled_load: bool,
    pub job: crate::background::LatestRead<Result<Arc<Settings>, String>>,
    intent: Option<Intent>,
}

impl State {
    pub fn cancel(&mut self) {
        if self.job.is_running() || matches!(self.intent, Some(Intent::Load | Intent::Import)) {
            self.cancelled_load = matches!(self.intent, Some(Intent::Load | Intent::Import));
            self.job.stop();
            self.intent = None;
            self.cancelled = true;
        }
    }

    pub fn cache_status(&self) -> Option<&CacheLoadStatus> {
        if self.job.is_running() && matches!(self.intent, Some(Intent::Load)) {
            Some(&CacheLoadStatus::Loading)
        } else if self.cancelled && self.cancelled_load {
            Some(&CacheLoadStatus::Cancelled)
        } else {
            None
        }
    }
}

impl RadiologyApp {
    /// Returns the selected pipeline for the canonical attached checkout and its PRs.
    #[must_use]
    pub fn indexing_personality(&self) -> Personality {
        self.index_personality.selected
    }

    /// Returns the latest validated settings capture; pending/failed loads have none.
    #[must_use]
    pub fn indexing_personality_settings(&self) -> Option<&Settings> {
        self.index_personality.settings.as_deref()
    }

    /// Returns a settings failure independently of compilation and cache errors.
    #[must_use]
    pub fn indexing_personality_error(&self) -> Option<&str> {
        self.index_personality.error.as_deref()
    }

    /// Returns the editable file path even when its current TOML cannot be parsed.
    #[must_use]
    pub fn indexing_settings_path(&self) -> Option<PathBuf> {
        self.personality_store()
            .map(|store| store.path(self.indexing_personality()))
    }

    fn personality_store(&self) -> Option<PersonalityStore> {
        self.session
            .config_directory()
            .map(|directory| PersonalityStore::new(directory.join("indexing-personalities")))
    }

    pub(in crate::ui::app) fn prepare_index_personality(&mut self, intent: Intent) {
        let Some(root) = self.repository_root.clone() else {
            return;
        };
        self.index_generator.stop();
        self.index_importer.stop();
        self.index_plan_job.stop();
        self.tool_probe_job.stop();
        self.tool_provisioner = Default::default();
        self.index_personality.selected = self.session.indexing_personality(&root);
        self.index_personality.settings = None;
        self.index_personality.error = None;
        self.index_personality.cache_warning = None;
        self.index_personality.cancelled = false;
        self.index_personality.intent = Some(intent);
        self.error = None;
        let Some(store) = self.personality_store() else {
            self.fail_index_personality(
                "No private indexing settings directory is available".into(),
            );
            return;
        };
        let personality = self.index_personality.selected;
        let review = self.review.clone();
        let mut protected = vec![root];
        if let Some(review) = &review {
            protected.push(review.head_root.clone());
        }
        self.index_personality.job.spawn(move |cancel| {
            let result = store.load(personality, &protected, cancel).map(Arc::new);
            drop(review);
            result
        });
    }

    pub(in crate::ui::app) fn poll_index_personality(&mut self) {
        let loaded = match self.index_personality.job.poll() {
            Ok(loaded) => loaded,
            Err(_) => Some(Err("Indexing settings worker stopped unexpectedly".into())),
        };
        let Some(loaded) = loaded else {
            return;
        };
        match loaded {
            Err(error) => self.fail_index_personality(error),
            Ok(settings) => {
                self.index_personality.settings = Some(settings);
                let intent = self.index_personality.intent.take();
                match intent {
                    Some(Intent::Generate(reuse)) => {
                        if let Some(mut request) = self.index_request() {
                            request.reuse = reuse;
                            self.start_index_generation(request);
                        } else {
                            self.fail_index_personality(
                                "No private index cache is available".into(),
                            );
                        }
                    }
                    Some(Intent::Load) => {
                        self.index_personality.intent = Some(Intent::Load);
                        self.inspect_captured_index_projects();
                        if let Some(request) = self.index_request() {
                            self.index_generator.start_cached(request);
                        } else if self.review.is_none()
                            && self.indexing_personality() == Personality::Default
                        {
                            self.index_personality.intent = Some(Intent::Import);
                            self.import_repository_index();
                        } else {
                            self.index_personality.intent = None;
                        }
                    }
                    Some(Intent::Inspect) => self.inspect_captured_index_projects(),
                    Some(Intent::Import) | None => {}
                }
            }
        }
    }

    /// Advances the accepted open through cache, legacy import and generation.
    pub(in crate::ui::app) fn advance_index_open(&mut self) {
        if self.index_personality.job.is_running()
            || self.index_generator.is_running()
            || self.index_plan_job.is_running()
        {
            return;
        }
        let refresh = match self.index_personality.intent {
            Some(Intent::Load) => match self.index_cache_status() {
                CacheLoadStatus::Idle | CacheLoadStatus::Loading => return,
                CacheLoadStatus::Cancelled => false,
                CacheLoadStatus::Loaded => self.generation_report.as_ref().is_some_and(|report| {
                    report.stale_reason.is_some()
                        || report
                            .outcomes
                            .iter()
                            .any(|outcome| outcome.failure.is_some())
                }),
                CacheLoadStatus::Missing | CacheLoadStatus::Failed(_) => {
                    if let CacheLoadStatus::Failed(error) = self.index_cache_status() {
                        self.index_personality.cache_warning =
                            Some(format!("Cached index: {error}"));
                    }
                    if self.review.is_none() && self.indexing_personality() == Personality::Default
                    {
                        self.index_personality.intent = Some(Intent::Import);
                        self.import_repository_index();
                        return;
                    }
                    true
                }
            },
            Some(Intent::Import) => {
                if self.index_importer.is_running() {
                    return;
                }
                !matches!(self.index_importer.status(), IndexImportStatus::Cancelled)
            }
            _ => return,
        };
        self.index_personality.intent = None;
        if refresh && self.index_plan_error.is_none() && !self.index_plan.selected.is_empty() {
            if let Some(mut request) = self.index_request() {
                request.reuse = true;
                self.start_index_generation(request);
            }
        }
    }

    fn fail_index_personality(&mut self, error: String) {
        self.index_personality.intent = None;
        self.index_personality.settings = None;
        self.index_personality.error = Some(error.clone());
        self.index_plan = Default::default();
        self.index_plan_error = None;
        self.semantic_index = None;
        self.generation_report = None;
        self.refresh_symbol_context();
        self.refresh_critical_projection();
        self.error = Some(format!("Indexing settings failed: {error}"));
    }

    pub(in crate::ui::app) fn select_indexing_personality(&mut self, personality: Personality) {
        if !self.repository_interaction_ready() {
            return;
        }
        let Some(root) = self.repository_root.clone() else {
            return;
        };
        if let Err(error) = self.session.set_indexing_personality(&root, personality) {
            self.error = Some(format!("Cannot save indexing personality: {error}"));
            return;
        }
        self.reset_index_state();
        self.prepare_index_personality(Intent::Generate(false));
    }

    pub(in crate::ui::app) fn cancel_index_generation(&mut self) {
        self.index_personality.cancel();
        self.index_importer.stop();
        self.index_generator.stop();
    }

    pub(in crate::ui::app) fn show_indexing_settings(&mut self) {
        let Some(path) = self.indexing_settings_path() else {
            return;
        };
        self.reveal_job.spawn(move |cancel| {
            let parent = path.parent().ok_or("Settings file has no parent")?;
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or("Invalid settings filename")?;
            let command = crate::desktop::prepare_reveal(parent, name)?;
            crate::desktop::show_items(&command, cancel, std::time::Duration::from_secs(5))
        });
    }

    pub(in crate::ui::app) fn show_personality_controls(
        &self,
        ui: &mut egui::Ui,
    ) -> std::ops::ControlFlow<Option<ToolbarAction>> {
        let selected = self.indexing_personality();
        let mut menu = crate::ui::context_menu::MenuBar::default();
        ui.horizontal(|ui| {
            ui.label("Personality");
            let items = [Personality::Default, Personality::Sel4]
                .into_iter()
                .map(|personality| {
                    crate::ui::context_menu::Item::action(
                        personality.name(),
                        ToolbarAction::SelectIndexingPersonality(personality),
                    )
                    .checked(personality == selected)
                    .enabled(personality != selected)
                })
                .collect();
            menu.button(
                ui,
                selected.name(),
                ("indexing_personality", &self.repository_root, selected.id()),
                items,
            );
        });
        if let Some(action) = menu.show() {
            return std::ops::ControlFlow::Break(Some(action));
        }
        if let Some(settings) = &self.index_personality.settings {
            crate::ui::text_fit::explanation(ui, settings.summary(), self.theme.text_secondary);
        }
        if ui.small_button("Show Settings File").clicked() {
            return std::ops::ControlFlow::Break(Some(ToolbarAction::ShowIndexingSettings));
        }
        ui.separator();
        if self.index_personality.job.is_running() {
            ui.spinner();
            ui.label("Loading indexing settings…");
            return std::ops::ControlFlow::Break(
                ui.button("Cancel Load")
                    .clicked()
                    .then_some(ToolbarAction::CancelIndexGeneration),
            );
        }
        if let Some(error) = &self.index_personality.error {
            ui.label(error);
            return std::ops::ControlFlow::Break(
                ui.button("Reload Settings")
                    .clicked()
                    .then_some(ToolbarAction::ReloadIndexImport),
            );
        }
        if self.index_personality.cancelled {
            ui.label("Index loading cancelled");
            return std::ops::ControlFlow::Break(
                ui.button("Reload Settings")
                    .clicked()
                    .then_some(ToolbarAction::ReloadIndexImport),
            );
        }
        std::ops::ControlFlow::Continue(())
    }

    pub(in crate::ui::app) fn show_personality_status(&self, ui: &mut egui::Ui) -> bool {
        if self.index_personality.job.is_running() {
            ui.spinner();
            ui.label("Loading indexing settings…");
        } else if let Some(error) = &self.index_personality.error {
            ui.label("Indexing settings failed").on_hover_text(error);
        } else if self.index_personality.cancelled {
            ui.label(if self.semantic_index.is_some() {
                "Index loading cancelled · previous data retained"
            } else {
                "Index loading cancelled"
            });
        } else {
            return false;
        }
        true
    }
}
