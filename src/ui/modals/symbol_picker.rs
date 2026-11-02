//! Background workspace/document symbol navigation over immutable snapshots.

use super::{
    picker::{self, PickerInput, PickerRow},
    ModalAction,
};
use crate::{
    background::LatestRead,
    semantic::{
        context::kind_name,
        index::{SemanticIndex, SymbolSearch},
    },
    syntax::outline::DocumentSymbol,
    theme::AppTheme,
    ui::{commands, icons::Icon, text_fit},
};
use egui::{Context, Key};
use std::{
    sync::{atomic::Ordering, Arc},
    time::Duration,
};

#[derive(Clone)]
enum Inventory {
    Workspace(Arc<SemanticIndex>),
    Document(Arc<[DocumentSymbol]>),
}

impl Inventory {
    fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Workspace(a), Self::Workspace(b)) => Arc::ptr_eq(a, b),
            (Self::Document(a), Self::Document(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }

    fn search(&self, query: &str, cancelled: impl Fn() -> bool) -> Option<Vec<SymbolChoice>> {
        match self {
            Self::Workspace(index) => Some(
                SymbolSearch::rank_cancellable(
                    query,
                    &index.search_entries,
                    SymbolSearch::VISIBLE_LIMIT,
                    cancelled,
                )?
                .into_iter()
                .map(|entry| {
                    let mut metadata = kind_name(entry.kind);
                    if entry.definition_count > 1 {
                        metadata.push_str(&format!(" · {} definitions", entry.definition_count));
                    }
                    SymbolChoice {
                        name: entry.name,
                        detail: format!("{}:{}", entry.path, entry.line),
                        metadata,
                        action: ModalAction::JumpToIndexedSymbol(entry.id),
                    }
                })
                .collect(),
            ),
            Self::Document(symbols) => {
                let needle = SymbolSearch::query_key(query);
                let mut stack: Vec<_> = symbols.iter().rev().collect();
                let mut choices = Vec::new();
                let mut visited = 0;
                while let Some(symbol) = stack.pop() {
                    if visited % 128 == 0 && cancelled() {
                        return None;
                    }
                    visited += 1;
                    if SymbolSearch::contains_query(&symbol.name, &needle) {
                        choices.push(SymbolChoice {
                            name: symbol.name.clone(),
                            detail: format!("line {}", symbol.line),
                            metadata: symbol.kind.title().into(),
                            action: ModalAction::JumpToSymbol(symbol.selection_range.start),
                        });
                    }
                    stack.extend(symbol.children.iter().rev());
                }
                (!cancelled()).then_some(choices)
            }
        }
    }
}

struct SymbolChoice {
    name: String,
    detail: String,
    metadata: String,
    action: ModalAction,
}

/// Searches workspace symbols, falling back to the current document outline.
///
/// Hosts publish replacement Arc snapshots when their index or source changes.
/// Clones keep query/selection and start independent search/interaction lifetimes.
pub struct GoToSymbolModal {
    /// Search query.
    pub query: String,
    /// Highlighted result index.
    pub selected_idx: usize,
    owner: uuid::Uuid,
    input: PickerInput,
    requested: Option<(Inventory, Arc<str>)>,
    worker: LatestRead<Option<Vec<SymbolChoice>>>,
    choices: Option<Vec<SymbolChoice>>,
    failed: bool,
    reveal: bool,
    focused: bool,
    generation: u64,
}

impl Default for GoToSymbolModal {
    fn default() -> Self {
        Self {
            query: String::new(),
            selected_idx: 0,
            owner: uuid::Uuid::new_v4(),
            input: PickerInput::default(),
            requested: None,
            worker: LatestRead::default(),
            choices: None,
            failed: false,
            reveal: true,
            focused: false,
            generation: 0,
        }
    }
}

impl Clone for GoToSymbolModal {
    fn clone(&self) -> Self {
        Self {
            query: self.query.clone(),
            selected_idx: self.selected_idx,
            ..Self::default()
        }
    }
}

impl std::fmt::Debug for GoToSymbolModal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GoToSymbolModal")
            .field("query", &self.query)
            .field("selected_idx", &self.selected_idx)
            .finish_non_exhaustive()
    }
}

impl GoToSymbolModal {
    /// Renders the picker using shared immutable semantic or document inputs.
    pub fn show(
        &mut self,
        ctx: &Context,
        theme: &AppTheme,
        index: Option<&Arc<SemanticIndex>>,
        symbols: &Arc<[DocumentSymbol]>,
    ) -> Option<ModalAction> {
        let key = self.input.begin(ctx, &mut self.worker);
        if key == Some(Key::Escape) {
            return Some(ModalAction::Close);
        }

        let inventory = index.map_or_else(
            || Inventory::Document(symbols.clone()),
            |index| Inventory::Workspace(index.clone()),
        );
        let workspace = index.is_some();
        let mut action = None;
        picker::window(
            if workspace {
                "Go to Symbol (Workspace)"
            } else {
                "Go to Symbol"
            },
            self.owner,
        )
        .show(ctx, |ui| {
            picker::query(
                ui,
                &mut self.focused,
                &mut self.query,
                Icon::Code,
                if workspace {
                    "Search symbols in workspace…"
                } else {
                    "Symbol name…"
                },
                theme.accent,
            );
            self.search(ctx, inventory);
            if let Some(key) = key.filter(|_| !self.failed) {
                self.input.navigation.push_back(key);
            }
            action = self.navigate();
            picker::body(ui, |ui| {
                if let Some(choices) = &self.choices {
                    if choices.is_empty() {
                        ui.label(if SymbolSearch::trim_query(&self.query).is_empty() {
                            "No Symbols"
                        } else {
                            "No Matches"
                        });
                        ui.label(
                            if workspace && SymbolSearch::trim_query(&self.query).is_empty() {
                                "The index has no repository symbols."
                            } else if workspace {
                                "Try a shorter name, initials, or another spelling."
                            } else {
                                "No matching declarations in this document."
                            },
                        );
                    } else if let Some(clicked) = self.rows(ui, theme) {
                        action = Some(clicked);
                    }
                } else {
                    ui.label(if self.failed {
                        "Symbol search failed. Change the query to retry."
                    } else {
                        "Searching…"
                    });
                }
            });
            if let Some(button) =
                picker::footer(ui, theme, &self.footer(workspace), "Go", self.selected())
            {
                action = Some(button);
            }
        });
        self.input.finish(ctx, &mut self.worker, action)
    }

    fn search(&mut self, ctx: &Context, inventory: Inventory) {
        let changed = self
            .requested
            .as_ref()
            .is_none_or(|(old, query)| !old.same(&inventory) || query.as_ref() != self.query);
        if changed {
            if self.requested.is_some() {
                self.selected_idx = 0;
            }
            self.generation = self.generation.wrapping_add(1);
            self.choices = None;
            self.failed = false;
            self.input.navigation.clear();
            self.reveal = true;
            let request = (inventory, Arc::<str>::from(self.query.as_str()));
            let (inventory, query) = request.clone();
            self.requested = Some(request);
            let context = ctx.clone();
            self.worker.spawn(move |cancel| {
                let result = inventory.search(&query, || cancel.load(Ordering::Relaxed));
                context.request_repaint();
                result
            });
        }
        match self.worker.poll() {
            Ok(Some(result)) => {
                self.failed = result.is_none();
                self.choices = result;
            }
            Err(_) => {
                self.failed = true;
            }
            Ok(None) => {}
        }
        if self.failed {
            self.input.navigation.clear();
        }
        if self.worker.is_running() {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }

    fn navigate(&mut self) -> Option<ModalAction> {
        let count = self.choices.as_ref()?.len();
        self.selected_idx = self.selected_idx.min(count.saturating_sub(1));
        while let Some(key) = self.input.navigation.pop_front() {
            if key == Key::Enter {
                return self.selected();
            }
            if let Some(index) = commands::list_selection(key, Some(self.selected_idx), count) {
                self.selected_idx = index;
                self.reveal = true;
            }
        }
        None
    }

    fn selected(&self) -> Option<ModalAction> {
        self.choices
            .as_ref()?
            .get(self.selected_idx)
            .map(|choice| choice.action.clone())
    }

    fn footer(&self, workspace: bool) -> String {
        match &self.choices {
            Some(choices) => format!(
                "{} {} · {}",
                choices.len(),
                if choices.len() == 1 {
                    "symbol"
                } else {
                    "symbols"
                },
                if workspace {
                    "Index ready"
                } else {
                    "Document outline"
                }
            ),
            None => if self.failed {
                "Search unavailable"
            } else {
                "Searching…"
            }
            .into(),
        }
    }

    fn rows(&mut self, ui: &mut egui::Ui, theme: &AppTheme) -> Option<ModalAction> {
        let choices = self.choices.as_ref()?;
        let height = PickerRow::height(ui);
        let id = egui::Id::new("symbol_results");
        let scroll = picker::scroll(ui, id, self.selected_idx, &mut self.reveal);
        let mut action = None;
        scroll.show_rows(ui, height, choices.len(), |ui, rows| {
            for index in rows {
                let choice = &choices[index];
                let owner = (self.owner, self.generation, index);
                commands::owned_ui(ui, owner, |ui| {
                    let response = PickerRow {
                        title: &choice.name,
                        detail: &choice.detail,
                        metadata: &choice.metadata,
                    }
                    .show(ui, theme, index == self.selected_idx, height)
                    .on_hover_text(text_fit::bounded(&format!(
                        "{} · {}\n{}",
                        choice.name, choice.metadata, choice.detail
                    )));
                    if commands::row_activated(&response) {
                        action = Some(choice.action.clone());
                    }
                });
            }
        });
        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        semantic::index::{SemanticSymbolID, SymbolSearchEntry},
        syntax::outline::SymbolKind,
        ui::{
            commands::test_key,
            test_support::{painted_frame, text_center, text_shapes},
        },
    };
    use egui::Event;

    fn index(count: usize) -> Arc<SemanticIndex> {
        let index = SemanticIndex {
            search_entries: (0..count)
                .map(|n| SymbolSearchEntry {
                    id: SemanticSymbolID::new(format!("symbol{n:04}"), ""),
                    name: format!("symbol{n:04}"),
                    kind: 17,
                    path: format!("src/file{n:04}.rs"),
                    line: n + 1,
                    definition_count: 3,
                })
                .collect(),
            ..Default::default()
        };
        Arc::new(index)
    }

    fn symbols(count: usize) -> Arc<[DocumentSymbol]> {
        (0..count)
            .map(|n| DocumentSymbol {
                name: format!("f{n:04}"),
                kind: SymbolKind::Function,
                range: n * 8..n * 8 + 8,
                selection_range: n * 8..n * 8 + 5,
                line: n + 1,
                children: vec![],
                detail: None,
            })
            .collect()
    }

    fn frame(
        ctx: &Context,
        modal: &mut GoToSymbolModal,
        index: Option<&Arc<SemanticIndex>>,
        symbols: &Arc<[DocumentSymbol]>,
        events: Vec<Event>,
    ) -> (egui::FullOutput, Option<ModalAction>) {
        let mut action = None;
        let output = painted_frame(ctx, egui::vec2(850.0, 650.0), events, |ui| {
            action = modal.show(ui.ctx(), &AppTheme::dark(), index, symbols);
        });
        (output, action)
    }

    fn settled(
        ctx: &Context,
        modal: &mut GoToSymbolModal,
        index: Option<&Arc<SemanticIndex>>,
        symbols: &Arc<[DocumentSymbol]>,
    ) -> (egui::FullOutput, Option<ModalAction>) {
        crate::ui::test_support::settled_modal(|| {
            let result = frame(ctx, modal, index, symbols, vec![]);
            (
                result.0,
                result.1,
                !modal.worker.is_running() && modal.input.events.is_empty(),
            )
        })
    }

    #[test]
    fn workspace_keyboard_and_pointer_selection_emit_symbol_identity() {
        let ctx = Context::default();
        let index = index(1);
        let empty = symbols(0);
        let mut modal = GoToSymbolModal::default();
        let (output, _) = settled(&ctx, &mut modal, Some(&index), &empty);
        let position = text_center(&output, "symbol0000");
        let expected = ModalAction::JumpToIndexedSymbol(index.search_entries[0].id.clone());
        assert_eq!(
            frame(
                &ctx,
                &mut modal,
                Some(&index),
                &empty,
                vec![test_key(Key::Enter)]
            )
            .1,
            Some(expected.clone())
        );
        assert_eq!(
            frame(
                &ctx,
                &mut modal,
                Some(&index),
                &empty,
                crate::ui::test_support::click(position)
            )
            .1,
            Some(expected)
        );
    }

    #[test]
    fn query_text_and_enter_in_the_same_frame_select_the_new_match() {
        let ctx = Context::default();
        let index = index(240);
        let empty = symbols(0);
        let mut modal = GoToSymbolModal::default();
        settled(&ctx, &mut modal, Some(&index), &empty);
        modal.selected_idx = 199;
        let (_, immediate) = frame(
            &ctx,
            &mut modal,
            Some(&index),
            &empty,
            vec![Event::Text("symbol0007".into()), test_key(Key::Enter)],
        );
        let action = immediate.or_else(|| settled(&ctx, &mut modal, Some(&index), &empty).1);
        assert_eq!(
            action,
            Some(ModalAction::JumpToIndexedSymbol(
                index.search_entries[7].id.clone()
            ))
        );
        assert_eq!(modal.selected_idx, 0);
    }

    #[test]
    fn document_picker_virtualizes_rows_and_reveals_keyboard_selection() {
        let ctx = Context::default();
        let symbols = symbols(1000);
        let mut modal = GoToSymbolModal::default();
        settled(&ctx, &mut modal, None, &symbols);
        modal.selected_idx = 998;
        frame(
            &ctx,
            &mut modal,
            None,
            &symbols,
            vec![test_key(Key::ArrowDown)],
        );
        let (output, _) = settled(&ctx, &mut modal, None, &symbols);
        assert!(text_shapes(&output).any(|text| text.galley.job.text == "f0999"));
        assert!(text_shapes(&output).count() < 100);
        let shape = output.shapes.iter().find(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == "f0999")).unwrap();
        if let egui::Shape::Text(text) = &shape.shape {
            assert!(shape.clip_rect.contains_rect(egui::Rect::from_min_size(
                text.pos,
                egui::vec2(text.galley.size().x, text.galley.size().y + 14.0)
            )));
        }
        assert_eq!(
            frame(&ctx, &mut modal, None, &symbols, vec![test_key(Key::Enter)]).1,
            Some(ModalAction::JumpToSymbol(999 * 8))
        );
    }

    #[test]
    fn workspace_limit_metadata_and_mixed_repeated_keys_preserve_order() {
        let ctx = Context::default();
        let index = index(240);
        let empty = symbols(0);
        let mut modal = GoToSymbolModal::default();
        let (output, _) = settled(&ctx, &mut modal, Some(&index), &empty);
        assert!(
            text_shapes(&output).any(|text| text.galley.job.text == "200 symbols · Index ready")
        );
        assert!(text_shapes(&output).any(|text| text.galley.job.text == "Function · 3 definitions"));
        let events = std::iter::once(Event::Text("symbol".into()))
            .chain((0..128).map(|_| test_key(Key::ArrowDown)))
            .chain([test_key(Key::Enter), Event::Text("discarded".into())])
            .collect();
        frame(&ctx, &mut modal, Some(&index), &empty, events);
        assert_eq!(
            settled(&ctx, &mut modal, Some(&index), &empty).1,
            Some(ModalAction::JumpToIndexedSymbol(
                index.search_entries[128].id.clone()
            ))
        );
        assert_eq!(modal.query, "symbol");
    }

    #[test]
    fn query_index_source_and_cancel_retire_inflight_results() {
        for boundary in ["query", "index", "source", "cancel"] {
            let ctx = Context::default();
            let original = index(20);
            let empty = symbols(0);
            let mut modal = GoToSymbolModal::default();
            settled(&ctx, &mut modal, Some(&original), &empty);
            let (release, wait) = std::sync::mpsc::channel();
            let (started, start) = std::sync::mpsc::channel();
            let (observed, observation) = std::sync::mpsc::channel();
            modal.worker.spawn(move |cancel| {
                started.send(()).unwrap();
                wait.recv().unwrap();
                observed.send(cancel.load(Ordering::Relaxed)).unwrap();
                Some(vec![])
            });
            start.recv_timeout(Duration::from_secs(5)).unwrap();
            let replacement = index(1);
            let document = symbols(1);
            let (active, outline) = match boundary {
                "query" => {
                    modal.query = "symbol0009".into();
                    (Some(&original), &empty)
                }
                "index" => (Some(&replacement), &empty),
                "source" => (None, &document),
                _ => (Some(&original), &empty),
            };
            let events = if boundary == "cancel" {
                vec![test_key(Key::Escape)]
            } else {
                vec![]
            };
            let (_, action) = frame(&ctx, &mut modal, active, outline, events);
            release.send(()).unwrap();
            assert!(observation.recv_timeout(Duration::from_secs(5)).unwrap());
            if boundary == "cancel" {
                assert_eq!(action, Some(ModalAction::Close));
            } else {
                settled(&ctx, &mut modal, active, outline);
                assert!(modal.selected().is_some());
            }
        }
    }

    #[test]
    fn empty_matches_and_long_unicode_names_paths_are_bounded_at_both_scales() {
        for scale in [1.0, 2.0] {
            let ctx = Context::default();
            ctx.set_pixels_per_point(scale);
            let empty = symbols(0);
            let mut modal = GoToSymbolModal::default();
            let (output, _) = settled(&ctx, &mut modal, Some(&index(0)), &empty);
            assert!(text_shapes(&output).any(|text| text.galley.job.text == "No Symbols"));
            let mut index = (*index(1)).clone();
            index.search_entries[0].name = "Long名é".repeat(300);
            index.search_entries[0].path = "directoryé/".repeat(300);
            let index = Arc::new(index);
            let (output, _) = settled(&ctx, &mut modal, Some(&index), &empty);
            assert!(
                text_shapes(&output)
                    .filter(|text| text.galley.job.text.contains('…'))
                    .count()
                    >= 2
            );
            let metadata = text_shapes(&output)
                .find(|text| text.galley.job.text == "Function · 3 definitions")
                .unwrap();
            assert_eq!(
                metadata.galley.job.sections[0].format.color,
                AppTheme::dark().text_primary
            );
            for text in text_shapes(&output).filter(|text| text.galley.job.text.contains('…')) {
                assert!(text.galley.size().x <= 610.0);
            }
            modal.query = "zzzz-no-match".into();
            let (output, _) = settled(&ctx, &mut modal, Some(&index), &empty);
            assert!(text_shapes(&output).any(|text| text.galley.job.text == "No Matches"));
            assert!(modal.selected().is_none());
            modal.query = "\n".into();
            let (output, _) = settled(&ctx, &mut modal, Some(&index), &empty);
            assert!(text_shapes(&output).any(|text| text.galley.job.text == "No Matches"));
        }
    }

    #[test]
    fn pointer_press_cannot_open_an_index_replacement_with_the_same_identity() {
        let ctx = Context::default();
        let original = index(1);
        let empty = symbols(0);
        let mut modal = GoToSymbolModal::default();
        let (output, _) = settled(&ctx, &mut modal, Some(&original), &empty);
        let position = text_center(&output, "symbol0000");
        let pointer = |pressed| Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        frame(
            &ctx,
            &mut modal,
            Some(&original),
            &empty,
            vec![Event::PointerMoved(position), pointer(true)],
        );
        let replacement = index(1);
        let generation = modal.generation;
        settled(&ctx, &mut modal, Some(&replacement), &empty);
        assert!(modal.generation > generation);
        assert_eq!(
            frame(
                &ctx,
                &mut modal,
                Some(&replacement),
                &empty,
                vec![pointer(false)]
            )
            .1,
            None
        );
        assert_eq!(
            frame(
                &ctx,
                &mut modal,
                Some(&replacement),
                &empty,
                crate::ui::test_support::click(position)
            )
            .1,
            Some(ModalAction::JumpToIndexedSymbol(
                replacement.search_entries[0].id.clone()
            ))
        );
    }

    #[test]
    fn document_filter_uses_canonical_character_substrings_and_preserves_offsets() {
        let ctx = Context::default();
        let mut outline = symbols(6).to_vec();
        for (symbol, name) in outline.iter_mut().zip([
            "Cafe\u{301}Factory",
            "CaféWidget",
            "👩‍👩‍👦Builder",
            "1\u{fe0f}\u{20e3}Counter",
            "prefix👍🏽Factory",
            "CAFÉLeaf",
        ]) {
            symbol.name = name.into();
        }
        let outline: Arc<[DocumentSymbol]> = outline.into();
        let mut modal = GoToSymbolModal::default();
        for (query, offsets) in [
            ("café", vec![0, 8, 40]),
            ("cafe\u{301}", vec![0, 8, 40]),
            ("\u{a0}FÉ\t", vec![0, 8, 40]),
            ("cafe", vec![]),
            ("👩", vec![]),
            ("👩‍👩‍👦", vec![16]),
            ("1", vec![]),
            ("1\u{fe0f}\u{20e3}", vec![24]),
            ("👍", vec![]),
            ("👍🏽", vec![32]),
            ("\n", vec![]),
            ("cft", vec![]),
            ("\t\u{a0}", vec![0, 8, 16, 24, 32, 40]),
        ] {
            modal.query = query.into();
            let (output, _) = settled(&ctx, &mut modal, None, &outline);
            let choices = modal.choices.as_ref().unwrap();
            assert_eq!(
                choices
                    .iter()
                    .map(|choice| choice.action.clone())
                    .collect::<Vec<_>>(),
                offsets
                    .into_iter()
                    .map(ModalAction::JumpToSymbol)
                    .collect::<Vec<_>>(),
                "{query:?}"
            );
            for choice in choices {
                let ModalAction::JumpToSymbol(byte) = choice.action else {
                    panic!("wrong destination")
                };
                assert_eq!(
                    choice.name,
                    outline[byte / 8].name,
                    "display name must retain its original bytes"
                );
            }
            if !SymbolSearch::trim_query(query).is_empty() && choices.is_empty() {
                assert!(text_shapes(&output).any(|text| text.galley.job.text == "No Matches"));
            }
        }
        modal.query = "café".into();
        settled(&ctx, &mut modal, None, &outline);
        assert_eq!(
            frame(&ctx, &mut modal, None, &outline, vec![test_key(Key::Enter)]).1,
            Some(ModalAction::JumpToSymbol(0))
        );
    }
}
