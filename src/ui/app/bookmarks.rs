//! Scoped bookmark persistence and identity-based application commands.

use super::*;
use crate::ui::bookmark_list::{BookmarkAction, BookmarkList};

impl RadiologyApp {
    pub(super) fn bookmark_scope(&self) -> Option<BookmarkScope> {
        self.repository_root
            .as_ref()
            .map(|root| BookmarkScope::new(root, self.active_pr.as_ref().map(|pr| pr.number)))
    }

    pub(super) fn load_bookmarks(&mut self) {
        self.bookmark_list = Default::default();
        if let (Some(dir), Some(scope)) = (self.session.config_directory(), self.bookmark_scope()) {
            self.bookmarks = BookmarkStore::load_in(&dir, &scope);
        }
    }

    pub(super) fn save_bookmarks(&mut self) {
        if let (Some(dir), Some(scope)) = (self.session.config_directory(), self.bookmark_scope()) {
            if let Err(error) = self.bookmarks.save_in(&dir, &scope) {
                self.error = Some(format!("Bookmarks could not be saved: {error}"));
            }
        }
    }

    pub(super) fn open_bookmark(&mut self, index: usize) {
        if !self.repository_interaction_ready() {
            return;
        }
        let Some(bookmark) = self.bookmarks.get_by_index(index).cloned() else {
            return;
        };
        self.bookmark_list.selected = Some(bookmark.id);
        let location =
            SourceLocation::with_revision(&bookmark.path, bookmark.byte_offset, bookmark.revision);
        self.request_navigation(
            location,
            crate::source::loader::SourceTarget::Bookmark(bookmark),
            source::NavigationIntent {
                record_history: true,
                flash_destination: true,
                reveal: true,
                force_read: true,
                ..Default::default()
            },
        );
    }
}

impl RadiologyApp {
    pub(super) fn toggle_current_bookmark(&mut self) {
        if !self.source_location_ready() {
            return;
        }
        let Some(buffer) = &self.open_buffer else {
            return;
        };
        let byte = buffer.byte_offset(self.caret_line, self.caret_col);
        let existing = self
            .bookmarks
            .at_location(buffer, byte)
            .and_then(|bookmark| {
                self.bookmarks
                    .index_of(bookmark.id)
                    .map(|index| (bookmark.id, index))
            });
        if self.bookmarks.toggle(buffer, byte) {
            self.bookmark_list.selected =
                self.bookmarks.bookmarks.last().map(|bookmark| bookmark.id);
        } else if let Some((id, index)) = existing {
            self.bookmark_list.removed(id, index, &self.bookmarks);
        }
        self.save_bookmarks();
    }

    pub(super) fn current_bookmarked(&self) -> bool {
        self.open_buffer.as_ref().is_some_and(|buffer| {
            self.bookmarks
                .at_location(buffer, buffer.byte_offset(self.caret_line, self.caret_col))
                .is_some()
        })
    }

    pub(super) fn handle_bookmark_keys(&mut self, ctx: &Context, input: &mut egui::RawInput) {
        if !self.repository_interaction_ready() {
            return;
        }
        for key in crate::ui::commands::take_focused_keys(
            ctx,
            input,
            BookmarkList::focus_id(),
            BookmarkList::KEYS,
        ) {
            if let Some(action) = self.bookmark_list.key(key, &self.bookmarks) {
                self.handle_bookmark_action(ctx, action);
            }
        }
    }

    pub(super) fn handle_bookmark_action(&mut self, ctx: &Context, action: BookmarkAction) {
        if !self.repository_interaction_ready() {
            return;
        }
        match action {
            BookmarkAction::Open(id) => {
                if let Some(index) = self.bookmarks.index_of(id) {
                    self.bookmark_list.selected = Some(id);
                    self.open_bookmark(index);
                }
            }
            BookmarkAction::Remove(id) => {
                if let Some(index) = self.bookmarks.index_of(id) {
                    self.bookmarks.remove(id);
                    self.bookmark_list.removed(id, index, &self.bookmarks);
                    self.save_bookmarks();
                }
            }
            BookmarkAction::Move { id, slot } => {
                if self.bookmarks.move_to(id, slot) {
                    self.save_bookmarks();
                }
            }
            BookmarkAction::ToggleCurrent => {
                self.execute_command(ctx, ToolbarAction::ToggleBookmark)
            }
        }
    }
}
