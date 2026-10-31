//! Source-bound outline selection, hierarchical disclosure, and virtual rows.

use crate::{
    pull_request::diff::FileLineChanges,
    source::{test_code::TestCodePresentation, SourceBuffer},
    syntax::outline::{outline_rows, DocumentSymbol, SymbolKind},
    theme::AppTheme,
};
use egui::{Context, Id, Key, Sense, Ui};
use std::{collections::HashSet, ops::Range};

/// An exact declaration identity in one immutable source revision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineTarget {
    document: (String, String),
    selection: Range<usize>,
}

/// Outline previews and commits have separate navigation semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutlineAction {
    /// Reveals the declaration without moving the source caret or history.
    Preview(OutlineTarget),
    /// Commits the exact declaration name through normal navigation history.
    Commit(OutlineTarget),
    /// Commits a pointer activation with transient destination feedback.
    Activate(OutlineTarget),
}

struct Entry {
    selection: Range<usize>,
    range: Range<usize>,
    name: String,
    kind: SymbolKind,
    callable: bool,
    depth: usize,
    lines: usize,
    has_children: bool,
    hidden: bool,
    changed: bool,
}

/// Outline state belongs to one exact document, separate from source selection.
#[derive(Default)]
pub struct OutlineList {
    document: Option<(String, String)>,
    entries: Vec<Entry>,
    visible: Vec<usize>,
    selected: Option<Range<usize>>,
    active: Option<Range<usize>>,
    collapsed: HashSet<usize>,
    test_generation: Option<u64>,
    changes: Option<FileLineChanges>,
    reveal: bool,
}

impl OutlineList {
    pub(crate) const KEYS: &[Key] = &[
        Key::ArrowUp,
        Key::ArrowDown,
        Key::Home,
        Key::End,
        Key::ArrowLeft,
        Key::ArrowRight,
        Key::Enter,
        Key::Escape,
    ];

    pub(crate) fn focus_id() -> Id {
        Id::new("outline_list_focus")
    }

    pub(crate) fn has_focus(ctx: &Context) -> bool {
        ctx.memory(|memory| memory.has_focus(Self::focus_id()))
    }

    pub(crate) fn sync(
        &mut self,
        buffer: &SourceBuffer,
        symbols: &[DocumentSymbol],
        tests: &TestCodePresentation,
        caret: usize,
        changes: Option<&FileLineChanges>,
    ) {
        let document = (buffer.position_key(), buffer.content_digest().to_owned());
        if self.document.as_ref() != Some(&document) {
            *self = Self {
                document: Some(document),
                ..Self::default()
            };
            self.entries = outline_rows(symbols, buffer)
                .into_iter()
                .map(|row| Entry {
                    selection: row.symbol.selection_range.clone(),
                    range: row.symbol.range.clone(),
                    name: row.symbol.name.clone(),
                    kind: row.symbol.kind,
                    callable: row.symbol.is_callable(),
                    depth: row.depth,
                    lines: row.line_count,
                    has_children: false,
                    hidden: false,
                    changed: false,
                })
                .collect();
            for index in 0..self.entries.len().saturating_sub(1) {
                self.entries[index].has_children = !self.entries[index].callable
                    && self.entries[index + 1].depth > self.entries[index].depth;
            }
        }
        self.sync_changes(buffer, changes);
        if self.test_generation != Some(tests.generation()) {
            for entry in &mut self.entries {
                entry.hidden = tests.hides_declaration(&entry.range);
            }
            self.test_generation = Some(tests.generation());
            self.project();
        }
        let active = self
            .entries
            .iter()
            .filter(|entry| entry.range.contains(&caret))
            .min_by_key(|entry| entry.range.len())
            .map(|entry| entry.selection.clone());
        if active != self.active {
            self.active = active.clone();
            self.selected = active;
            if let Some(index) = self.selected_entry() {
                self.expand_ancestors(index);
                self.project();
            }
            self.reveal = true;
        }
    }

    fn sync_changes(&mut self, buffer: &SourceBuffer, changes: Option<&FileLineChanges>) {
        let changes = changes.filter(|_| buffer.revision.is_some());
        if self.changes.as_ref() == changes {
            return;
        }
        let bytes: Vec<_> = buffer
            .revision
            .zip(changes)
            .filter(|(_, changes)| !changes.is_binary)
            .into_iter()
            .flat_map(|(revision, changes)| changes.changed_lines(revision))
            .filter_map(|lines| buffer.line_span_byte_range(lines))
            .collect();
        for entry in &mut self.entries {
            entry.changed = entry.callable
                && bytes
                    .iter()
                    .any(|range| range.start < entry.range.end && entry.range.start < range.end);
        }
        self.changes = changes.cloned();
    }

    fn selected_entry(&self) -> Option<usize> {
        self.entries
            .iter()
            .position(|entry| Some(&entry.selection) == self.selected.as_ref())
    }

    fn expand_ancestors(&mut self, index: usize) {
        let mut depth = self.entries[index].depth;
        for entry in self.entries[..index].iter().rev() {
            if entry.depth < depth {
                self.collapsed.remove(&entry.selection.start);
                depth = entry.depth;
            }
        }
    }

    fn project(&mut self) {
        self.visible.clear();
        let mut collapsed_depth = None;
        for (index, entry) in self.entries.iter().enumerate() {
            if collapsed_depth.is_some_and(|depth| entry.depth > depth) {
                continue;
            }
            collapsed_depth = None;
            if !entry.hidden {
                self.visible.push(index);
                if self.collapsed.contains(&entry.selection.start) {
                    collapsed_depth = Some(entry.depth);
                }
            }
        }
        if self
            .selected_entry()
            .is_some_and(|index| !self.visible.contains(&index))
        {
            self.selected = None;
        }
    }

    fn target(&self, index: usize) -> Option<OutlineTarget> {
        Some(OutlineTarget {
            document: self.document.clone()?,
            selection: self.entries[index].selection.clone(),
        })
    }

    /// Validates source identity and current visibility before resolving an action.
    pub(crate) fn destination(
        &self,
        buffer: &SourceBuffer,
        target: &OutlineTarget,
    ) -> Option<usize> {
        if target.document.0 != buffer.position_key()
            || target.document.1 != buffer.content_digest()
            || self.document.as_ref() != Some(&target.document)
        {
            return None;
        }
        self.visible
            .iter()
            .any(|&index| self.entries[index].selection == target.selection)
            .then_some(target.selection.start)
    }

    pub(crate) fn key(&mut self, key: Key) -> Option<OutlineAction> {
        let selected = self.selected_entry();
        let position = selected.and_then(|index| self.visible.iter().position(|&row| row == index));
        if let Some(next) = super::commands::list_selection(key, position, self.visible.len()) {
            self.select(self.visible[next]);
        } else {
            match key {
                Key::Enter => return Some(OutlineAction::Commit(self.target(selected?)?)),
                Key::Escape => {
                    self.selected = None;
                    return None;
                }
                Key::ArrowRight => self.right(selected?),
                Key::ArrowLeft => self.left(selected?),
                _ => return None,
            }
        }
        Some(OutlineAction::Preview(self.target(self.selected_entry()?)?))
    }

    fn select(&mut self, index: usize) {
        self.selected = Some(self.entries[index].selection.clone());
        self.reveal = true;
    }

    fn right(&mut self, index: usize) {
        if !self.entries[index].has_children {
            return;
        }
        if self.collapsed.remove(&self.entries[index].selection.start) {
            self.project();
        } else if let Some(child) = self.visible.iter().copied().find(|&row| row > index) {
            if self.entries[child].depth > self.entries[index].depth {
                self.select(child);
            }
        }
    }

    fn left(&mut self, index: usize) {
        let entry = &self.entries[index];
        if entry.has_children && !self.collapsed.contains(&entry.selection.start) {
            self.collapsed.insert(entry.selection.start);
            self.project();
        } else if let Some(parent) = (0..index)
            .rev()
            .find(|&row| self.entries[row].depth < entry.depth)
        {
            self.select(parent);
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui, theme: &AppTheme) -> Option<OutlineAction> {
        ui.interact(
            ui.available_rect_before_wrap(),
            Self::focus_id(),
            Sense::focusable_noninteractive(),
        )
        .widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Other,
                "Outline. Arrows preview; Left and Right disclose declarations; Return navigates.",
            )
        });
        super::commands::retain_navigation_keys(ui.ctx(), Self::focus_id());
        if self.visible.is_empty() {
            ui.colored_label(theme.text_muted, "No Outline");
            ui.label(if self.entries.is_empty() {
                "No declarations were found in this file."
            } else {
                "Declarations are hidden by Hide Tests."
            });
            return None;
        }
        let heading = self
            .visible
            .iter()
            .position(|&row| !self.entries[row].callable);
        let height = ui.text_style_height(&egui::TextStyle::Body) + 8.0;
        let mut scroll = egui::ScrollArea::vertical()
            .id_source("reader_outline")
            .auto_shrink([false, false]);
        let reveal = std::mem::take(&mut self.reveal)
            .then(|| {
                let selected = self.selected_entry()?;
                let position = self.visible.iter().position(|&row| row == selected)?;
                Some(position + usize::from(heading.is_some_and(|heading| position >= heading)))
            })
            .flatten();
        scroll =
            super::commands::reveal_list_row(ui, scroll, Id::new("reader_outline"), reveal, height);
        let mut action = None;
        let mut toggle = None;
        scroll.show_rows(
            ui,
            height,
            self.visible.len() + usize::from(heading.is_some()),
            |ui, rows| {
                for row in rows {
                    if heading == Some(row) {
                        ui.label(egui::RichText::new("Declarations").strong());
                        continue;
                    }
                    let position = row - usize::from(heading.is_some_and(|heading| row > heading));
                    let index = self.visible[position];
                    let (commit, disclose) = self.paint_row(ui, theme, index, height);
                    if commit {
                        self.select(index);
                        ui.memory_mut(|memory| memory.request_focus(Self::focus_id()));
                        action = self.target(index).map(OutlineAction::Activate);
                    }
                    if disclose {
                        toggle = Some(index);
                    }
                }
            },
        );
        if let Some(index) = toggle {
            let start = self.entries[index].selection.start;
            if !self.collapsed.remove(&start) {
                self.collapsed.insert(start);
            }
            self.select(index);
            self.project();
            ui.memory_mut(|memory| memory.request_focus(Self::focus_id()));
        }
        action
    }

    fn paint_row(&self, ui: &mut Ui, theme: &AppTheme, index: usize, height: f32) -> (bool, bool) {
        let entry = &self.entries[index];
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), height), Sense::hover());
        let selected = self.selected.as_ref() == Some(&entry.selection);
        let response = ui.interact(
            rect,
            ui.id().with(("outline_row", entry.selection.start)),
            Sense {
                click: true,
                drag: false,
                focusable: false,
            },
        );
        if selected || response.hovered() {
            ui.painter().rect_filled(
                rect,
                3.0,
                if selected {
                    ui.visuals().selection.bg_fill
                } else {
                    ui.visuals().widgets.hovered.bg_fill
                },
            );
        }
        let x = rect.left() + 6.0 + (entry.depth as f32 * 14.0).min(rect.width() * 0.35);
        if entry.has_children {
            let icon = if self.collapsed.contains(&entry.selection.start) {
                super::icons::Icon::ChevronRight
            } else {
                super::icons::Icon::ChevronDown
            };
            icon.paint(
                ui.painter(),
                egui::Rect::from_center_size(
                    egui::pos2(x + 5.0, rect.center().y),
                    egui::vec2(10.0, 10.0),
                ),
                theme.text_secondary,
            );
        } else {
            ui.painter().text(
                egui::pos2(x, rect.center().y),
                egui::Align2::LEFT_CENTER,
                entry.kind.icon(),
                egui::TextStyle::Body.resolve(ui.style()),
                theme.text_secondary,
            );
        }
        let count = if entry.callable {
            format!("{} L", entry.lines)
        } else {
            String::new()
        };
        let count = ui.painter().layout_no_wrap(
            count,
            egui::TextStyle::Small.resolve(ui.style()),
            theme.text_secondary,
        );
        let right = rect.right() - 6.0 - count.size().x;
        ui.painter().galley(
            egui::pos2(right, rect.center().y - count.size().y / 2.0),
            count,
            theme.text_secondary,
        );
        let label_right = if entry.changed {
            ui.painter()
                .circle_filled(egui::pos2(right - 8.0, rect.center().y), 3.0, theme.accent);
            right - 16.0
        } else {
            right
        };
        let label = super::text_fit::middle(
            ui,
            &entry.name,
            egui::TextStyle::Body.resolve(ui.style()),
            theme.text_primary,
            (label_right - x - 24.0).max(0.0),
        );
        ui.painter()
            .with_clip_rect(rect.intersect(ui.clip_rect()))
            .galley(
                egui::pos2(x + 18.0, rect.center().y - label.size().y / 2.0),
                label,
                theme.text_primary,
            );
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::SelectableLabel,
                selected,
                format!(
                    "{} {}. {} lines. {} {}",
                    entry.kind.title(),
                    entry.name,
                    entry.lines,
                    if self.active.as_ref() == Some(&entry.selection) {
                        "Current declaration"
                    } else {
                        ""
                    },
                    if entry.changed {
                        "Changed in this pull request"
                    } else {
                        ""
                    }
                ),
            )
        });
        let clicked = response.clicked();
        let disclose = clicked
            && entry.has_children
            && response
                .interact_pointer_pos()
                .is_some_and(|point| point.x < x + 16.0);
        response.on_hover_text(format!(
            "{} · {} lines\nGo to {}{}",
            entry.kind.title(),
            entry.lines,
            super::text_fit::bounded(&entry.name),
            if entry.changed {
                "\nContains lines changed by the pull request"
            } else {
                ""
            }
        ));
        (clicked && !disclose, disclose)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::{outline::SymbolExtractor, test_detector::TestRegionDetector};

    fn changed_names(list: &OutlineList) -> Vec<&str> {
        list.entries
            .iter()
            .filter(|entry| entry.changed)
            .map(|entry| entry.name.as_str())
            .collect()
    }

    #[test]
    fn review_markers_follow_revision_half_open_bytes_and_live_diff_replacement() {
        use crate::{pull_request::diff::DiffHunk, source::ReviewRevision};
        let text = "// café 😀\r\nfn removed() {\r\n first();\r\n}\r\nfn added() {\r\n second();\r\n}\r\nstruct Plain;\r\n";
        let changes = FileLineChanges {
            hunks: vec![DiffHunk {
                before_start: 2,
                before_count: 3,
                after_start: 5,
                after_count: 3,
            }],
            is_binary: false,
        };
        let tests = TestCodePresentation::default();
        let mut list = OutlineList::default();
        for (revision, expected) in [
            (ReviewRevision::Before, "removed"),
            (ReviewRevision::After, "added"),
        ] {
            let buffer =
                SourceBuffer::from_bytes("a.rs", text.as_bytes().to_vec(), Some(revision)).unwrap();
            let symbols = SymbolExtractor::extract(&buffer);
            list.sync(&buffer, &symbols, &tests, 0, Some(&changes));
            assert_eq!(changed_names(&list), [expected]);
            list.key(Key::Home);
            let selected = list.selected.clone();
            let mut replacement = changes.clone();
            replacement.hunks[0].after_count = 10_000; // clip at EOF, never mark Plain
            list.sync(&buffer, &symbols, &tests, 0, Some(&replacement));
            assert_eq!(changed_names(&list), [expected]);
            assert_eq!(list.selected, selected);
            replacement.is_binary = true;
            list.sync(&buffer, &symbols, &tests, 0, Some(&replacement));
            assert!(changed_names(&list).is_empty());
            list.sync(&buffer, &symbols, &tests, 0, Some(&changes));
            assert_eq!(changed_names(&list), [expected]);
            list.sync(&buffer, &symbols, &tests, 0, None);
            assert!(changed_names(&list).is_empty());
        }
        let checkout = SourceBuffer::from_str("a.rs", text);
        list.sync(
            &checkout,
            &SymbolExtractor::extract(&checkout),
            &tests,
            0,
            Some(&changes),
        );
        assert!(changed_names(&list).is_empty());
    }

    #[test]
    fn nested_callable_markers_use_overlapping_spans_and_reject_empty_eof_changes() {
        use crate::{pull_request::diff::DiffHunk, source::ReviewRevision};
        let text = "fn outer() {\n fn inner() { changed(); }\n}\n";
        let buffer = SourceBuffer::from_bytes(
            "a.rs",
            text.as_bytes().to_vec(),
            Some(ReviewRevision::After),
        )
        .unwrap();
        let symbols = SymbolExtractor::extract(&buffer);
        let mut changes = FileLineChanges {
            hunks: vec![DiffHunk {
                before_start: 2,
                before_count: 1,
                after_start: 2,
                after_count: 1,
            }],
            is_binary: false,
        };
        let mut list = OutlineList::default();
        list.sync(&buffer, &symbols, &Default::default(), 0, Some(&changes));
        assert_eq!(changed_names(&list), ["outer", "inner"]);
        changes.hunks[0].after_start = buffer.line_count(); // empty trailing line
        list.sync(&buffer, &symbols, &Default::default(), 0, Some(&changes));
        assert!(changed_names(&list).is_empty());
        changes.hunks[0].after_start = 2;
        changes.hunks[0].after_count = 0; // navigation anchor is not an added source line
        list.sync(&buffer, &symbols, &Default::default(), 0, Some(&changes));
        assert!(changed_names(&list).is_empty());
    }

    #[test]
    fn hierarchical_selection_disclosure_and_active_caret_follow_source_identities() {
        let source = SourceBuffer::from_str("main.rs", "// origin\nfn small() {}\nfn large() {\n let x = 1;\n}\nmod outer {\n pub struct Inner;\n pub const VALUE: i32 = 1;\n}\n");
        let symbols = SymbolExtractor::extract(&source);
        let tests = TestCodePresentation::default();
        let mut list = OutlineList::default();
        list.sync(&source, &symbols, &tests, 0, None);
        let Some(OutlineAction::Preview(first)) = list.key(Key::Home) else {
            panic!()
        };
        assert_eq!(first.selection.start, source.text.find("large").unwrap());
        let Some(OutlineAction::Preview(last)) = list.key(Key::End) else {
            panic!()
        };
        assert_eq!(last.selection.start, source.text.find("VALUE").unwrap());
        list.key(Key::ArrowLeft); // Parent, then collapse it.
        assert_eq!(
            list.selected.as_ref().unwrap().start,
            source.text.find("outer").unwrap()
        );
        list.key(Key::ArrowLeft);
        assert!(list.destination(&source, &last).is_none());
        list.key(Key::ArrowRight); // Expand, then select its first child.
        list.key(Key::ArrowRight);
        assert_eq!(
            list.selected.as_ref().unwrap().start,
            source.text.find("Inner").unwrap()
        );
        list.key(Key::ArrowLeft);
        list.key(Key::ArrowLeft);
        // Actual source motion into a nested declaration reveals its ancestors.
        list.sync(
            &source,
            &symbols,
            &tests,
            source.text.find("VALUE").unwrap(),
            None,
        );
        assert!(list.destination(&source, &last).is_some());
        assert_eq!(list.key(Key::Enter), Some(OutlineAction::Commit(last)));
        list.key(Key::Escape);
        list.sync(
            &source,
            &symbols,
            &tests,
            source.text.find("VALUE").unwrap(),
            None,
        );
        assert!(list.key(Key::Enter).is_none());
    }

    #[test]
    fn hidden_and_stale_targets_do_not_resolve_to_replacement_rows() {
        let source = SourceBuffer::from_str(
            "main.rs",
            "// origin\nfn ordinary() {}\n#[test]\nfn hidden() {}\n",
        );
        let symbols = SymbolExtractor::extract(&source);
        let mut tests = TestCodePresentation::default();
        tests.load(&source, TestRegionDetector::detect(&source), false);
        let mut list = OutlineList::default();
        list.sync(&source, &symbols, &tests, 0, None);
        list.key(Key::Home);
        let Some(OutlineAction::Preview(target)) = list.key(Key::ArrowDown) else {
            panic!()
        };
        assert_eq!(target.selection.start, source.text.find("hidden").unwrap());
        tests.set_enabled(&source, true);
        list.sync(&source, &symbols, &tests, 0, None);
        assert!(list.destination(&source, &target).is_none());
        assert!(list.selected.is_none());
        tests.set_enabled(&source, false);
        list.sync(&source, &symbols, &tests, 0, None);
        assert!(list.destination(&source, &target).is_some());
        for replacement in [
            SourceBuffer::from_str("other.rs", &source.text),
            SourceBuffer::from_str(&source.path, &(source.text.clone() + "changed")),
            SourceBuffer::from_bytes(
                &source.path,
                source.data.clone(),
                Some(crate::source::ReviewRevision::Before),
            )
            .unwrap(),
        ] {
            assert!(list.destination(&replacement, &target).is_none());
            list.sync(
                &replacement,
                &SymbolExtractor::extract(&replacement),
                &tests,
                0,
                None,
            );
            assert!(list.destination(&replacement, &target).is_none());
        }
    }

    #[test]
    fn outline_virtualizes_large_documents_and_reveals_end_selection() {
        let ctx = Context::default();
        let text = (0..1000)
            .map(|i| format!("fn item_{i:04}() {{}}\n"))
            .collect::<String>();
        let source = SourceBuffer::from_str("main.rs", &text);
        let mut list = OutlineList::default();
        list.sync(
            &source,
            &SymbolExtractor::extract(&source),
            &Default::default(),
            0,
            None,
        );
        let render = |list: &mut OutlineList| {
            crate::ui::test_support::painted_text(&ctx, egui::vec2(300.0, 240.0), |ui| {
                list.show(ui, &AppTheme::dark());
            })
        };
        render(&mut list);
        let visible = render(&mut list);
        assert!(visible.iter().any(|text| text == "item_0000"));
        assert!(
            visible
                .iter()
                .filter(|text| text.starts_with("item_"))
                .count()
                < 20
        );
        list.key(Key::End);
        render(&mut list);
        assert!(render(&mut list).iter().any(|text| text == "item_0999"));
    }
}
