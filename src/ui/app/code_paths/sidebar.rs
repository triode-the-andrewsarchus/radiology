//! Saved-trace controls in the left navigation peer tab.
use super::*;
use crate::ui::{
    code_paths::CodePathList,
    context_menu::{Item, MenuBar},
};

#[derive(Clone)]
enum Action {
    Import,
    Select(Uuid),
    Copy,
    Delete,
    Variant(Uuid, usize),
    Retry,
}

impl RadiologyApp {
    pub(in crate::ui::app) fn show_code_paths(&mut self, ui: &mut egui::Ui) {
        if self.sidebar_tab != SidebarTab::CodePaths {
            return;
        }
        let trace = self.code_paths.selected.clone();
        let enabled = self.repository_interaction_ready() && !self.is_repository_opening();
        let mutable = self.can_edit_code_paths();
        let mut action = None;
        egui::TopBottomPanel::bottom("code_paths_footer").show_inside(ui, |ui| {
            let count = self.saved_code_paths().len();
            ui.colored_label(
                self.theme.text_secondary,
                format!("{count} code {}", if count == 1 { "path" } else { "paths" }),
            );
        });
        let mut menus = MenuBar::default();
        ui.horizontal(|ui| {
            ui.strong("Code Paths");
            if ui
                .add_enabled(mutable, egui::Button::new("+"))
                .on_hover_text("Import Code Path (Ctrl+Shift+P)")
                .clicked()
            {
                action = Some(Action::Import);
            }
            menus.button(
                ui,
                "…",
                "code_path_actions",
                vec![
                    Item::action("Copy Code Path", Action::Copy).enabled(trace.is_some()),
                    Item::action("Delete Code Path", Action::Delete)
                        .enabled(mutable && trace.is_some()),
                    Item::separator(),
                    Item::action("Reload Saved Code Paths", Action::Retry).enabled(mutable),
                ],
            );
        });
        if self.is_code_path_storage_busy() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Updating code paths…");
            });
        }
        if let Some(error) = self.code_path_storage_error().or(self.code_path_error()) {
            crate::ui::text_fit::explanation(ui, error, ui.visuals().error_fg_color);
        }
        let selected = trace.as_ref().map(|trace| trace.id);
        let title = trace
            .as_ref()
            .map_or("Choose a code path", |trace| trace.title.as_str());
        let title = crate::ui::text_fit::middle(
            ui,
            title,
            egui::TextStyle::Button.resolve(ui.style()),
            self.theme.text_primary,
            (ui.available_width() - 24.0).max(1.0),
        );
        menus
            .button(
                ui,
                title,
                ("code_path_picker", selected),
                self.saved_code_paths()
                    .iter()
                    .map(|path| {
                        Item::action(path.title.clone(), Action::Select(path.id))
                            .checked(selected == Some(path.id))
                    })
                    .collect(),
            )
            .on_hover_text(
                trace
                    .as_ref()
                    .map_or("Choose a code path", |trace| trace.title.as_str()),
            );
        if self.code_path_destinations().len() > 1 {
            let step = self.selected_code_path_step();
            let items = self
                .code_path_destinations()
                .iter()
                .enumerate()
                .filter_map(|(variant, destination)| {
                    step.map(|step| {
                        Item::action(
                            format!("Definition {} · line {}", variant + 1, destination.line),
                            Action::Variant(step, variant),
                        )
                    })
                })
                .collect();
            menus.button(ui, format!("{} conditional definitions", self.code_path_destinations().len()), "code_path_variants", items)
                .on_hover_text("The first source definition opens initially. Choose another; no build configuration is implied.");
        }
        action = menus.show().or(action);
        if let Some(trace) = &trace {
            ui.colored_label(
                self.theme.text_secondary,
                format!("{} steps", trace.steps.len()),
            );
            ui.add_enabled_ui(enabled, |ui| {
                if let Some(step) =
                    self.code_path_list
                        .show(ui, &self.theme, trace, self.code_paths.selected_step)
                {
                    self.go_to_code_path_step(step, 0);
                }
            });
        } else {
            ui.colored_label(self.theme.text_muted, "No Code Paths");
            ui.label("Import a source trace with Ctrl+Shift+P, then follow its steps.");
        }
        match action {
            Some(Action::Import) => self.execute_command(ui.ctx(), ToolbarAction::ImportCodePath),
            Some(Action::Select(id)) => {
                let selected = self
                    .saved_code_paths()
                    .iter()
                    .find(|trace| trace.id == id)
                    .cloned();
                self.select_code_path(selected);
            }
            Some(Action::Copy) => {
                if let Some(trace) = trace {
                    ui.output_mut(|output| output.copied_text = trace.text());
                }
            }
            Some(Action::Delete) => {
                if let Some(trace) = trace {
                    self.delete_code_path(trace.id);
                }
            }
            Some(Action::Variant(step, variant)) => self.go_to_code_path_step(step, variant),
            Some(Action::Retry) => self.code_path_store.reload(),
            None => {}
        }
    }

    pub(in crate::ui::app) fn handle_code_path_key(
        &mut self,
        ctx: &Context,
        key: egui::Key,
        modifiers: egui::Modifiers,
    ) -> bool {
        if !self.show_sidebar
            || self.sidebar_tab != SidebarTab::CodePaths
            || !crate::ui::commands::focused_key(
                ctx,
                CodePathList::focus_id(),
                key,
                modifiers,
                CodePathList::KEYS,
            )
        {
            return false;
        }
        if let Some(trace) = self.code_paths.selected.clone() {
            if let Some(step) = self.code_path_list.key(key, &trace) {
                self.go_to_code_path_step(step, 0);
            }
        }
        true
    }
}
