//! Compact trait rows with independent list focus and explicit source activation.
use crate::{
    semantic::{trait_context::TraitImplementationContext, trait_impls::TraitImplementation},
    theme::AppTheme,
};
use egui::{Key, Ui};

/// An identity-scoped list operation; selection alone never navigates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraitAction {
    /// Selects a source-sorted row, or clears selection.
    Select(Option<usize>),
    /// Selects and opens a listed implementation through source verification.
    Commit(usize),
}

/// Retained scrolling intent; the semantic context owns selection and rows.
#[derive(Default)]
pub struct TraitImplementationList {
    reveal: bool,
}

impl TraitImplementationList {
    /// Navigation keys handled by the host before egui's focus traversal.
    pub const KEYS: &'static [Key] = &[
        Key::ArrowUp,
        Key::ArrowDown,
        Key::Home,
        Key::End,
        Key::Enter,
        Key::Escape,
    ];
    /// Returns the list's stable keyboard owner.
    pub fn focus_id() -> egui::Id {
        egui::Id::new("trait_implementations_focus")
    }
    /// Returns whether the list owns ordinary navigation keys.
    pub fn has_focus(ctx: &egui::Context) -> bool {
        ctx.memory(|memory| memory.has_focus(Self::focus_id()))
    }
    /// Requests minimal scrolling to reveal the context's selected row.
    pub fn reveal_selection(&mut self) {
        self.reveal = true;
    }
    /// Maps a focused key to an action without moving the source caret.
    /// The host applies the action to the same accepted context before replacing it.
    pub fn key(&mut self, key: Key, context: &TraitImplementationContext) -> Option<TraitAction> {
        if let Some(row) =
            super::commands::list_selection(key, context.selected, context.rows().len())
        {
            return Some(TraitAction::Select(Some(row)));
        }
        match key {
            Key::Enter => context.selected.map(TraitAction::Commit),
            Key::Escape => Some(TraitAction::Select(None)),
            _ => None,
        }
    }

    /// Renders one compact source-sorted list. The host applies returned actions.
    pub fn show(
        &mut self,
        ui: &mut Ui,
        theme: &AppTheme,
        context: &TraitImplementationContext,
    ) -> Option<TraitAction> {
        let catalog = &context.index()?.trait_implementations;
        let rows = context.rows();
        let negative = rows.iter().filter(|row| row.is_negative).count();
        let name = context
            .target()
            .map(|id| context.index().unwrap().display_name(id))
            .unwrap_or_default();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Trait Implementations").strong());
            if catalog.is_prepared {
                ui.colored_label(theme.text_secondary, rows.len().to_string());
            }
            if catalog.has_gaps() {
                ui.colored_label(theme.text_secondary, "⚠")
                    .on_hover_text(catalog.coverage_detail());
            }
        })
        .response
        .on_hover_text(format!(
            "{name}: {} indexed implementations, {negative} negative impls. {}",
            rows.len() - negative,
            catalog.coverage_detail()
        ));
        ui.interact(
            ui.available_rect_before_wrap(),
            Self::focus_id(),
            egui::Sense::focusable_noninteractive(),
        );
        super::commands::retain_navigation_keys(ui.ctx(), Self::focus_id());
        let mut action = None;
        ui.spacing_mut().item_spacing.y = 0.0;
        let id = egui::Id::new("trait_implementations_rows");
        let height = 22.0_f32.max(ui.text_style_height(&egui::TextStyle::Body) + 4.0);
        let scroll = egui::ScrollArea::vertical()
            .id_source(id)
            .auto_shrink([false, false])
            .animated(false);
        let scroll = super::commands::reveal_list_row(
            ui,
            scroll,
            id,
            self.reveal.then_some(context.selected).flatten(),
            height,
        );
        self.reveal = false;
        if let Some(error) = &context.navigation_error {
            ui.colored_label(theme.text_secondary, "Implementation unavailable")
                .on_hover_text(error);
        }
        if !catalog.is_prepared {
            ui.colored_label(theme.text_muted, "Waiting for index");
        } else if rows.is_empty() {
            ui.colored_label(theme.text_muted, "No indexed implementations")
                .on_hover_text(catalog.coverage_detail());
        }
        scroll.show_rows(ui, height, rows.len(), |ui, visible| {
            for position in visible {
                if let Some(next) = row(
                    ui,
                    theme,
                    &rows[position],
                    position,
                    context.selected == Some(position),
                    height,
                ) {
                    action = Some(next);
                }
            }
        });
        action
    }
}

fn row(
    ui: &mut Ui,
    theme: &AppTheme,
    item: &TraitImplementation,
    position: usize,
    selected: bool,
    height: f32,
) -> Option<TraitAction> {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::hover(),
    );
    let response = ui
        .interact(
            rect,
            ui.id().with(("trait_impl", position)),
            egui::Sense::click(),
        )
        .on_hover_text(format!("{}:{}\n{}", item.path, item.line, item.signature));
    if selected {
        ui.painter()
            .rect_filled(rect, 2.0, ui.visuals().selection.bg_fill);
    } else if response.hovered() {
        ui.painter()
            .rect_filled(rect, 2.0, ui.visuals().widgets.hovered.bg_fill);
    }
    let metadata = format!(
        "{}:{}",
        item.path.rsplit('/').next().unwrap_or(&item.path),
        item.line
    );
    let meta_width = (rect.width() * 0.4).min(96.0);
    let metadata = super::text_fit::middle(
        ui,
        &metadata,
        egui::TextStyle::Small.resolve(ui.style()),
        theme.text_secondary,
        meta_width,
    );
    let negative_width = if item.is_negative { 52.0 } else { 0.0 };
    let name = super::text_fit::lines(
        ui,
        &item.target_type,
        egui::TextStyle::Monospace.resolve(ui.style()),
        theme.text_primary,
        (rect.width() - meta_width - negative_width - 12.0).max(0.0),
        1,
    );
    ui.painter().galley(
        egui::pos2(rect.left() + 3.0, rect.center().y - name.size().y * 0.5),
        name,
        theme.text_primary,
    );
    if item.is_negative {
        ui.painter().text(
            egui::pos2(rect.right() - meta_width - 5.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            "negative",
            egui::TextStyle::Small.resolve(ui.style()),
            theme.text_secondary,
        );
    }
    ui.painter().galley(
        egui::pos2(
            rect.right() - metadata.size().x - 3.0,
            rect.center().y - metadata.size().y * 0.5,
        ),
        metadata,
        theme.text_secondary,
    );
    let action = if super::commands::row_activated(&response) {
        ui.memory_mut(|memory| memory.request_focus(TraitImplementationList::focus_id()));
        Some(TraitAction::Commit(position))
    } else if response.secondary_clicked() {
        ui.memory_mut(|memory| memory.request_focus(TraitImplementationList::focus_id()));
        Some(TraitAction::Select(Some(position)))
    } else {
        None
    };
    super::commands::context_menu(&response, ("trait_impl", position), || {
        vec![super::commands::MenuItem::action(
            "Go to Trait Implementation",
            TraitAction::Commit(position),
        )]
    })
    .or(action)
}

#[cfg(test)]
mod tests;
