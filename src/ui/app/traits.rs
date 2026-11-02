//! Trait context binding and guarded source navigation through the shared loader.
use super::*;
use crate::semantic::{
    trait_context::{trait_at_byte, TraitStructure},
    trait_impls::TraitImplementation,
};
use crate::ui::trait_implementations::{TraitAction, TraitImplementationList};
use std::sync::atomic::AtomicBool;

#[derive(Clone)]
pub(super) struct TraitNavigationOwner {
    root: PathBuf,
    index: Arc<SemanticIndex>,
    row: TraitImplementation,
}

impl RadiologyApp {
    pub(super) fn retire_hidden_trait_focus(&mut self, ctx: &Context) {
        let hidden_trait = TraitImplementationList::has_focus(ctx)
            && (!self.show_inspector || !self.trait_pane_visible());
        let hidden_definition = (crate::ui::outline_list::OutlineList::has_focus(ctx)
            || ctx.memory(|memory| memory.has_focus(ContextInspector::definition_focus_id())))
            && (!self.show_inspector || !self.definition_pane_visible());
        if hidden_trait || hidden_definition {
            self.source_viewport.request_focus(ctx);
        }
    }

    pub(super) fn refresh_trait_context(&mut self) {
        let target = self
            .open_buffer
            .as_ref()
            .zip(self.semantic_index.as_ref())
            .and_then(|(buffer, index)| {
                trait_at_byte(
                    buffer,
                    TraitStructure {
                        symbols: &self.outline_symbols,
                        implementations: &self.rust_trait_impls,
                    },
                    index,
                    buffer.byte_offset(self.caret_line, self.caret_col),
                    &AtomicBool::new(false),
                )
                .ok()
                .flatten()
            });
        if self.trait_context.update(
            self.repository_root.as_deref(),
            self.semantic_index.clone(),
            target,
        ) {
            self.trait_list = Default::default();
            self.retire_trait_navigation();
        }
    }

    pub(super) fn trait_pane_visible(&self) -> bool {
        self.session.inspector_panes.implementations && self.trait_context.target().is_some()
    }

    pub(super) fn definition_pane_visible(&self) -> bool {
        self.session.inspector_panes.definition && !self.trait_pane_visible()
    }

    pub(super) fn retire_trait_navigation(&mut self) {
        let retired = self
            .pending_navigation
            .as_ref()
            .and_then(|pending| pending.trait_owner.as_ref())
            .is_some_and(|owner| {
                self.repository_root.as_ref() != Some(&owner.root)
                    || !self
                        .trait_context
                        .index()
                        .is_some_and(|index| Arc::ptr_eq(index, &owner.index))
                    || self
                        .trait_context
                        .selected
                        .and_then(|row| self.trait_context.rows().get(row))
                        != Some(&owner.row)
            });
        if retired {
            self.cancel_source_load();
        }
    }

    pub(super) fn handle_trait_key(
        &mut self,
        ctx: &Context,
        key: egui::Key,
        modifiers: egui::Modifiers,
    ) -> bool {
        if !self.trait_pane_visible()
            || !crate::ui::commands::focused_key(
                ctx,
                TraitImplementationList::focus_id(),
                key,
                modifiers,
                TraitImplementationList::KEYS,
            )
        {
            return false;
        }
        if let Some(action) = self.trait_list.key(key, &self.trait_context) {
            self.handle_trait_action(action);
        }
        true
    }

    pub(super) fn handle_trait_action(&mut self, action: TraitAction) {
        if !self.repository_interaction_ready() || !self.trait_pane_visible() {
            return;
        }
        match action {
            TraitAction::Select(row) => {
                self.trait_context.select(row);
                self.trait_list.reveal_selection();
                self.retire_trait_navigation();
            }
            TraitAction::Commit(row) => {
                self.trait_context.select(Some(row));
                let Some(item) = self.trait_context.rows().get(row).cloned() else {
                    return;
                };
                let (Some(root), Some(index)) =
                    (self.repository_root.clone(), self.semantic_index.clone())
                else {
                    return;
                };
                let revision = self.review.as_ref().map(|_| ReviewRevision::After);
                self.request_navigation(
                    SourceLocation::with_revision(&item.path, item.byte_range.start, revision),
                    crate::source::loader::SourceTarget::ByteRange {
                        range: item.byte_range.clone(),
                        content_digest: item.source_digest.clone(),
                    },
                    source::NavigationIntent {
                        record_history: true,
                        reveal: true,
                        flash_destination: true,
                        force_read: true,
                        ..Default::default()
                    },
                );
                if let Some(pending) = &mut self.pending_navigation {
                    pending.trait_owner = Some(TraitNavigationOwner {
                        root,
                        index,
                        row: item,
                    });
                }
            }
        }
    }
}
