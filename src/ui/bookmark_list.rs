//! Identity-based bookmark selection, ordering, and bounded list presentation.

use crate::bookmarks::{bookmark::SourceBookmark, store::BookmarkStore};
use crate::theme::AppTheme;
use egui::{Context, Key, RichText, Ui};
use uuid::Uuid;

/// Bookmark commands carry stable identities rather than mutable shortcut slots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BookmarkAction {
    /// Opens the identity through the source anchor resolver.
    Open(Uuid),
    /// Removes one persisted identity.
    Remove(Uuid),
    /// Assigns an identity to a final slot, preserving other entries' order.
    Move {
        /// Identity to move.
        id: Uuid,
        /// Final zero-based slot.
        slot: usize,
    },
    /// Toggles the current source line through the shared reader command.
    ToggleCurrent,
}

/// Selection and reveal state for one checkout or pull-request bookmark scope.
#[derive(Default)]
pub struct BookmarkList {
    pub(super) selected: Option<Uuid>,
    reveal: bool,
}

struct BookmarkDrag(Uuid);

impl BookmarkList {
    pub(crate) const KEYS: &[Key] = &[
        Key::ArrowUp,
        Key::ArrowDown,
        Key::Home,
        Key::End,
        Key::Enter,
        Key::Delete,
        Key::Backspace,
        Key::Escape,
    ];

    pub(super) fn focus_id() -> egui::Id {
        egui::Id::new("bookmark_list_focus")
    }

    pub(super) fn has_focus(ctx: &Context) -> bool {
        ctx.memory(|memory| memory.has_focus(Self::focus_id()))
    }

    pub(super) fn key(&mut self, key: Key, store: &BookmarkStore) -> Option<BookmarkAction> {
        let index = self.selected.and_then(|id| store.index_of(id));
        match key {
            Key::Enter => return index.map(|i| BookmarkAction::Open(store.bookmarks[i].id)),
            Key::Delete | Key::Backspace => {
                return index.map(|i| BookmarkAction::Remove(store.bookmarks[i].id));
            }
            Key::Escape => self.selected = None,
            Key::Home | Key::End | Key::ArrowUp | Key::ArrowDown => {
                let next = super::commands::list_selection(key, index, store.bookmarks.len())?;
                self.selected = Some(store.bookmarks[next].id);
                self.reveal = true;
            }
            _ => {}
        }
        None
    }

    pub(super) fn removed(&mut self, id: Uuid, old_index: usize, store: &BookmarkStore) {
        if self.selected == Some(id) {
            self.selected = store
                .bookmarks
                .get(old_index.min(store.bookmarks.len().saturating_sub(1)))
                .map(|bookmark| bookmark.id);
            self.reveal = true;
        }
    }

    pub(super) fn show(
        &mut self,
        ui: &mut Ui,
        theme: &AppTheme,
        store: &BookmarkStore,
        scope: &str,
        current_bookmarked: Option<bool>,
    ) -> Option<BookmarkAction> {
        if self.selected.is_some_and(|id| store.index_of(id).is_none()) {
            self.selected = None;
        }
        let mut action = None;
        ui.horizontal(|ui| {
            ui.label(RichText::new("Bookmarks").strong());
            ui.colored_label(theme.text_muted, scope);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = if current_bookmarked == Some(true) {
                    "−"
                } else {
                    "+"
                };
                let help = if current_bookmarked == Some(true) {
                    "Remove current bookmark (Ctrl+D)"
                } else {
                    "Bookmark current location (Ctrl+D)"
                };
                if ui
                    .add_enabled(current_bookmarked.is_some(), egui::Button::new(label))
                    .on_hover_text(help)
                    .clicked()
                {
                    action = Some(BookmarkAction::ToggleCurrent);
                }
            });
        });
        let region = ui.available_rect_before_wrap();
        ui.interact(
            region,
            Self::focus_id(),
            egui::Sense::focusable_noninteractive(),
        )
        .widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Other,
                "Bookmarks. Up and Down select; Return opens; Delete removes. Drag to reorder.",
            )
        });
        super::commands::retain_navigation_keys(ui.ctx(), Self::focus_id());
        let row_height = ui.text_style_height(&egui::TextStyle::Body)
            + ui.text_style_height(&egui::TextStyle::Small) * 2.0
            + 10.0;
        let mut scroll = egui::ScrollArea::vertical()
            .id_source("reader_bookmarks")
            .auto_shrink([false, false]);
        let reveal = std::mem::take(&mut self.reveal)
            .then(|| self.selected.and_then(|id| store.index_of(id)))
            .flatten();
        scroll = super::commands::reveal_list_row(
            ui,
            scroll,
            egui::Id::new("reader_bookmarks"),
            reveal,
            row_height,
        );
        scroll.show_rows(ui, row_height, store.bookmarks.len(), |ui, rows| {
            if store.bookmarks.is_empty() {
                ui.colored_label(theme.text_muted, "Ctrl+D to bookmark this location.");
            }
            for index in rows {
                if let Some(next) = self.row(ui, theme, store, index, row_height) {
                    action = Some(next);
                }
            }
        });
        action
    }

    fn row(
        &mut self,
        ui: &mut Ui,
        theme: &AppTheme,
        store: &BookmarkStore,
        index: usize,
        height: f32,
    ) -> Option<BookmarkAction> {
        let bookmark = &store.bookmarks[index];
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), height),
            egui::Sense {
                click: true,
                drag: true,
                focusable: false,
            },
        );
        let selected = self.selected == Some(bookmark.id);
        if selected || response.hovered() {
            let fill = if selected {
                ui.visuals().selection.bg_fill
            } else {
                ui.visuals().widgets.hovered.bg_fill
            };
            ui.painter().rect_filled(rect, 3.0, fill);
        }
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::SelectableLabel,
                selected,
                format!(
                    "{}, {}:{}, column {}. {}",
                    bookmark.label,
                    bookmark.path,
                    bookmark.line,
                    bookmark.column + 1,
                    if index < 10 {
                        format!("Ctrl+{index}")
                    } else {
                        "Drag to assign a shortcut".into()
                    }
                ),
            )
        });
        self.paint_row(ui, theme, rect, bookmark, index);
        let mut action = None;
        if response.clicked() || response.drag_started() || response.secondary_clicked() {
            self.selected = Some(bookmark.id);
        }
        if response.clicked() {
            ui.memory_mut(|memory| memory.request_focus(Self::focus_id()));
            action = Some(BookmarkAction::Open(bookmark.id));
        }
        response.dnd_set_drag_payload(BookmarkDrag(bookmark.id));
        if let Some(payload) = response.dnd_hover_payload::<BookmarkDrag>() {
            if let Some(from) = store.index_of(payload.0) {
                let below = ui
                    .input(|input| input.pointer.hover_pos())
                    .is_some_and(|p| p.y > rect.center().y);
                let gap = index + usize::from(below);
                let slot = gap - usize::from(from < gap);
                let y = if below { rect.bottom() } else { rect.top() };
                ui.painter()
                    .hline(rect.x_range(), y, ui.visuals().selection.stroke);
                if response.dnd_release_payload::<BookmarkDrag>().is_some() {
                    action = Some(BookmarkAction::Move {
                        id: payload.0,
                        slot,
                    });
                }
            }
        }
        let response = response.on_hover_text(format!(
            "{}:{}, column {}\nDrag to reorder bookmarks.",
            bookmark.path,
            bookmark.line,
            bookmark.column + 1
        ));
        super::commands::context_menu(&response, |ui| {
            action = bookmark_menu(ui, store, index).or(action.take());
        });
        action
    }

    fn paint_row(
        &self,
        ui: &mut Ui,
        theme: &AppTheme,
        rect: egui::Rect,
        bookmark: &SourceBookmark,
        index: usize,
    ) {
        let mut row = ui.child_ui(rect.shrink(5.0), egui::Layout::top_down(egui::Align::Min));
        row.spacing_mut().item_spacing.y = 0.0;
        row.horizontal(|ui| {
            let shortcut = (index < 10).then(|| format!("Ctrl+{index}"));
            let slot_width = shortcut.as_ref().map_or(0.0, |text| {
                ui.painter()
                    .layout_no_wrap(
                        text.clone(),
                        egui::TextStyle::Small.resolve(ui.style()),
                        theme.text_muted,
                    )
                    .size()
                    .x
                    + ui.spacing().item_spacing.x
            });
            let label_width = (ui.available_width() - slot_width).max(0.0);
            ui.allocate_ui_with_layout(
                egui::vec2(label_width, ui.text_style_height(&egui::TextStyle::Body)),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.set_min_width(label_width);
                    ui.add(egui::Label::new(&bookmark.label).truncate(true));
                },
            );
            if let Some(shortcut) = shortcut {
                ui.label(RichText::new(shortcut).small().color(theme.text_muted));
            }
        });
        row.add(
            egui::Label::new(
                RichText::new(format!("{}:{}", bookmark.path, bookmark.line))
                    .small()
                    .color(theme.text_secondary),
            )
            .truncate(true),
        );
        if let Some(revision) = bookmark.revision {
            row.label(
                RichText::new(revision.title())
                    .small()
                    .color(theme.text_muted),
            );
        }
    }
}

fn bookmark_menu(ui: &mut Ui, store: &BookmarkStore, index: usize) -> Option<BookmarkAction> {
    let id = store.bookmarks[index].id;
    let mut action = None;
    for (label, enabled, next) in [
        ("Go to Bookmark", true, BookmarkAction::Open(id)),
        (
            "Move Up",
            index > 0,
            BookmarkAction::Move {
                id,
                slot: index.saturating_sub(1),
            },
        ),
        (
            "Move Down",
            index + 1 < store.bookmarks.len(),
            BookmarkAction::Move {
                id,
                slot: index + 1,
            },
        ),
    ] {
        if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
            action = Some(next);
            ui.close_menu();
        }
    }
    ui.menu_button("Move to Shortcut", |ui| {
        for slot in 0..store.bookmarks.len().min(10) {
            if ui
                .add_enabled(slot != index, egui::Button::new(format!("Ctrl+{slot}")))
                .clicked()
            {
                action = Some(BookmarkAction::Move { id, slot });
                ui.close_menu();
            }
        }
    });
    ui.separator();
    if ui.button("Remove Bookmark").clicked() {
        action = Some(BookmarkAction::Remove(id));
        ui.close_menu();
    }
    action
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bookmarks::bookmark::SourceBookmark, source::SourceBuffer};
    use egui::Modifiers;

    #[test]
    fn list_keys_require_focus_and_leave_modified_global_keys_and_releases_available() {
        let ctx = Context::default();
        let mut input = egui::RawInput {
            events: vec![
                super::super::commands::test_key(Key::ArrowDown),
                super::super::commands::test_key(Key::F12),
                egui::Event::Key {
                    key: Key::ArrowUp,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::CTRL,
                },
                egui::Event::Key {
                    key: Key::Enter,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..Default::default()
        };
        let original = input.events.clone();
        assert!(super::super::commands::take_focused_keys(
            &ctx,
            &mut input,
            BookmarkList::focus_id(),
            BookmarkList::KEYS
        )
        .is_empty());
        assert_eq!(input.events, original);
        ctx.memory_mut(|memory| memory.request_focus(BookmarkList::focus_id()));
        assert_eq!(
            super::super::commands::take_focused_keys(
                &ctx,
                &mut input,
                BookmarkList::focus_id(),
                BookmarkList::KEYS
            ),
            [Key::ArrowDown]
        );
        assert_eq!(input.events, original[1..]);
    }

    #[test]
    fn large_lists_only_paint_visible_rows_and_reveal_keyboard_selection() {
        let ctx = Context::default();
        let buffer = SourceBuffer::from_str("very/long/path/a.rs", "fn alpha() {}\n");
        let store = BookmarkStore {
            bookmarks: (0..1000)
                .map(|index| SourceBookmark::new(&buffer, 0, format!("bookmark-{index:04}")))
                .collect(),
        };
        let mut list = BookmarkList::default();
        let render = |list: &mut BookmarkList| {
            crate::ui::test_support::painted_text(&ctx, egui::vec2(300.0, 240.0), |ui| {
                list.show(ui, &AppTheme::dark(), &store, "Checkout", Some(false));
            })
        };
        // The first layout discovers the viewport size; subsequent layouts use that geometry.
        render(&mut list);
        let visible = render(&mut list);
        assert!(visible.iter().any(|text| text == "bookmark-0000"));
        assert!(
            visible
                .iter()
                .filter(|text| text.starts_with("bookmark-"))
                .count()
                < 12
        );
        list.key(Key::End, &store);
        render(&mut list);
        let visible = render(&mut list);
        assert!(
            visible.iter().any(|text| text == "bookmark-0999"),
            "{visible:?}"
        );
        list.key(Key::Home, &store);
        render(&mut list);
        assert!(render(&mut list).iter().any(|text| text == "bookmark-0000"));
    }
}
