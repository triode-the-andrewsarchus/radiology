//! Shared pane menu and composition; hidden state remains with the host.
use super::*;
use crate::session::inspector::{InspectorPane, InspectorPanes};
use crate::ui::commands::{MenuItem, ReaderCommand};

pub(crate) fn menu(panes: InspectorPanes) -> Vec<MenuItem<ReaderCommand>> {
    let mut items: Vec<_> = InspectorPane::ALL
        .into_iter()
        .map(|pane| {
            MenuItem::action(pane.title(), ReaderCommand::ToggleInspectorPane(pane))
                .checked(panes.enabled(pane))
        })
        .collect();
    items.push(MenuItem::separator());
    items.push(
        MenuItem::action("Show All Panes", ReaderCommand::ShowAllInspectorPanes)
            .enabled(!panes.all_enabled()),
    );
    items
}

impl ContextInspector {
    /// Renders the existing reference, definition/outline and bookmark regions.
    #[allow(clippy::too_many_arguments)]
    pub fn show_with_panes(
        ui: &mut Ui,
        theme: &AppTheme,
        content: InspectorContent<'_>,
        references: &mut crate::ui::reference_list::ReferenceViewport,
        bookmarks: &mut BookmarkList,
        outline: &mut OutlineList,
        declaration: &mut DeclarationSummary,
        panes: &InspectorPanes,
    ) -> Option<InspectorAction> {
        Self::show_with_traits(
            ui,
            theme,
            content,
            references,
            bookmarks,
            outline,
            declaration,
            panes,
            None,
        )
    }

    /// Renders only enabled regions, retaining each host-owned viewport and selection.
    ///
    /// Returns shared commands for the host to persist. A supplied trait context
    /// occupies the existing middle slot; hidden state remains owned by the host.
    #[allow(clippy::too_many_arguments)]
    pub fn show_with_traits(
        ui: &mut Ui,
        theme: &AppTheme,
        mut content: InspectorContent<'_>,
        references: &mut crate::ui::reference_list::ReferenceViewport,
        bookmarks: &mut BookmarkList,
        outline: &mut OutlineList,
        declaration: &mut DeclarationSummary,
        panes: &InspectorPanes,
        mut traits: Option<(
            &crate::semantic::trait_context::TraitImplementationContext,
            &mut crate::ui::trait_implementations::TraitImplementationList,
        )>,
    ) -> Option<InspectorAction> {
        let mut action = header(ui, theme, *panes);
        if !panes.references {
            content.summary_identity = None;
        }
        let visible = panes.visible(
            traits
                .as_ref()
                .is_some_and(|(context, _)| context.target().is_some()),
        );
        if visible.is_empty() {
            return empty_panes(ui, theme, *panes).or(action);
        }
        let height = ui.available_height();
        let mut remaining = visible;
        if remaining.len() > 1 && panes.bookmarks {
            egui::TopBottomPanel::bottom("bookmarks_region")
                .resizable(true)
                .default_height(height * 0.25)
                .min_height(100.0)
                .max_height((height * 0.5).max(100.0))
                .show_inside(ui, |ui| {
                    if let Some(next) = region(ui, InspectorPane::Bookmarks, |ui| {
                        Self::render_bookmarks(ui, theme, &content, bookmarks)
                    }) {
                        action = Some(next);
                    }
                });
            remaining.retain(|pane| *pane != InspectorPane::Bookmarks);
        }
        if remaining.len() > 1 {
            egui::TopBottomPanel::top("declaration_region")
                .resizable(true)
                .default_height(height * 0.38)
                .min_height(160.0)
                .max_height((ui.available_height() - 120.0).max(160.0))
                .show_inside(ui, |ui| {
                    if let Some(next) = region(ui, InspectorPane::References, |ui| {
                        Self::render_declaration_context(
                            ui,
                            theme,
                            &content,
                            references,
                            declaration,
                        )
                    }) {
                        action = Some(next);
                    }
                });
            remaining.remove(0);
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show_inside(ui, |ui| {
                if let Some(next) = region(ui, remaining[0], |ui| match remaining[0] {
                    InspectorPane::References => Self::render_declaration_context(
                        ui,
                        theme,
                        &content,
                        references,
                        declaration,
                    ),
                    InspectorPane::Bookmarks => {
                        Self::render_bookmarks(ui, theme, &content, bookmarks)
                    }
                    InspectorPane::Definition => {
                        Self::render_under_caret_context(ui, theme, &content, outline)
                    }
                    InspectorPane::Implementations => traits
                        .as_mut()
                        .and_then(|(context, list)| list.show(ui, theme, context))
                        .map(InspectorAction::Implementation),
                }) {
                    action = Some(next);
                }
            });
        action
    }

    fn render_bookmarks(
        ui: &mut Ui,
        theme: &AppTheme,
        content: &InspectorContent<'_>,
        bookmarks: &mut BookmarkList,
    ) -> Option<InspectorAction> {
        ui.add_enabled_ui(
            content.bookmarks_enabled || content.bookmarks.bookmarks.is_empty(),
            |ui| {
                bookmarks.show(
                    ui,
                    theme,
                    content.bookmarks,
                    content.scope,
                    (content.bookmarks_enabled && content.can_toggle_bookmark)
                        .then_some(content.current_bookmarked),
                )
            },
        )
        .inner
        .map(InspectorAction::Bookmark)
    }
}

fn header(ui: &mut Ui, theme: &AppTheme, panes: InspectorPanes) -> Option<InspectorAction> {
    let mut action = None;
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Context")
                .small()
                .strong()
                .color(theme.text_secondary),
        );
        let response = ui.button("Panes");
        let keyboard = response.has_focus()
            && ui.input_mut(|input| {
                [
                    egui::Key::Enter,
                    egui::Key::Space,
                    egui::Key::ArrowDown,
                    egui::Key::ArrowUp,
                ]
                .into_iter()
                .any(|key| input.consume_key(egui::Modifiers::NONE, key))
            });
        super::super::commands::retain_navigation_keys(ui.ctx(), response.id);
        if let Some(command) = super::super::context_menu::show_at(
            &response,
            "context_panes",
            (response.clicked() || keyboard).then_some(response.rect.left_bottom()),
            || menu(panes),
        ) {
            action = Some(InspectorAction::Command(command));
        }
    });
    ui.separator();
    action
}

fn empty_panes(ui: &mut Ui, theme: &AppTheme, panes: InspectorPanes) -> Option<InspectorAction> {
    let mut action = None;
    ui.vertical_centered(|ui| {
                ui.heading(if panes.implementations { "No Context Panes Available" } else { "No Panes Enabled" });
                super::super::text_fit::explanation(ui, "Enable a pane from Panes or View > Context Panes. Trait implementations appear when a Rust trait context is available.", theme.text_secondary);
                if ui.button("Show All Panes").clicked() { action=Some(InspectorAction::Command(ReaderCommand::ShowAllInspectorPanes)); }
            });
    action
}
