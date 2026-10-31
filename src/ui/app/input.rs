//! Ordered keyboard ownership across inspector controls and global commands.
use super::*;
use egui::{Event, Key, Modifiers};

impl RadiologyApp {
    pub(super) fn inspector_has_focus(&self, ctx: &Context) -> bool {
        self.show_inspector
            && (self.reference_viewport.has_focus(ctx)
                || crate::ui::bookmark_list::BookmarkList::has_focus(ctx)
                || crate::ui::outline_list::OutlineList::has_focus(ctx)
                || ctx.memory(|memory| memory.has_focus(ContextInspector::definition_focus_id())))
    }

    pub(super) fn capture_inspector_input(&mut self, ctx: &Context, input: &mut egui::RawInput) {
        if !matches!(self.modal, ActiveModal::None)
            || crate::ui::commands::context_menu_is_open(ctx)
            || !self.inspector_has_focus(ctx)
        {
            return;
        }
        self.inspector_input
            .extend(crate::ui::commands::take_input_actions(
                &mut input.events,
                |event| {
                    // Tab stays with egui's native traversal; owned keys cannot move focus first.
                    (crate::ui::commands::keyboard_event(event)
                        && !matches!(event, Event::Key { key: Key::Tab, .. }))
                    .then(|| event.clone())
                },
            ));
    }

    pub(super) fn dispatch_key_events(&mut self, ctx: &Context, events: &mut Vec<Event>) {
        crate::ui::commands::take_input_actions(events, |event| {
            if !matches!(self.modal, ActiveModal::None) {
                return None;
            }
            let menu_open = crate::ui::commands::context_menu_is_open(ctx);
            if !menu_open {
                if let Event::Key {
                    key,
                    modifiers,
                    pressed: true,
                    ..
                } = event
                {
                    if self.inspector_key(ctx, *key, *modifiers) {
                        return Some(());
                    }
                }
            }
            let reader_focused =
                self.source_viewport.has_focus(ctx) || self.inspector_has_focus(ctx);
            let typing = ctx.wants_keyboard_input() && !reader_focused && !menu_open;
            let command = crate::ui::commands::shortcut_event(event, typing, menu_open)?;
            crate::ui::commands::close_context_menu(ctx);
            let previous_focus = ctx.memory(|memory| memory.focused());
            self.execute_command(ctx, command);
            if !matches!(self.modal, ActiveModal::None)
                || ctx.memory(|memory| memory.focused()) != previous_focus
            {
                self.reference_viewport.cancel_focus_request();
            }
            Some(())
        });
    }

    fn inspector_key(&mut self, ctx: &Context, key: Key, modifiers: Modifiers) -> bool {
        if !self.show_inspector {
            return false;
        }
        if self.handle_bookmark_key(ctx, key, modifiers)
            || self.handle_outline_key(ctx, key, modifiers)
        {
            return true;
        }
        if let Some(action) = self.reference_viewport.action_for_key(ctx, key, modifiers) {
            self.handle_reference_action(action);
            self.reference_viewport.publish_focus_request(ctx);
            return true;
        }
        if let Some(action) = ContextInspector::definition_key(ctx, key, modifiers) {
            self.handle_inspector_action(ctx, action);
            return true;
        }
        false
    }

    pub(super) fn handle_inspector_action(&mut self, ctx: &Context, action: InspectorAction) {
        match action {
            InspectorAction::Reference(action) => self.handle_reference_action(action),
            InspectorAction::FollowCaret => self.follow_reference_caret(),
            InspectorAction::ExpandDefinition(expanded) => self.definition_expanded = expanded,
            InspectorAction::GoToDefinition => {
                self.execute_command(ctx, ToolbarAction::GoToDefinition)
            }
            InspectorAction::JumpToIndexedLocation { path, range } => {
                self.navigate_to_indexed_range(&path, &range);
            }
            InspectorAction::JumpToSymbol(byte) => {
                if let Some(buffer) = &self.open_buffer {
                    self.navigate_target(
                        buffer.location_at_byte(byte),
                        None,
                        source::NavigationIntent {
                            record_history: true,
                            reveal: true,
                            flash_destination: true,
                            ..Default::default()
                        },
                    );
                }
            }
            InspectorAction::Outline(action) => self.handle_outline_action(action),
            InspectorAction::Bookmark(action) => self.handle_bookmark_action(ctx, action),
        }
    }
}
