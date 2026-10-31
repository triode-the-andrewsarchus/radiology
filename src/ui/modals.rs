//! Modal dialog overlays for fuzzy file navigation, line jumping, symbol outlines, and pull request review.
//!
//! Provides focused modal popups for keyboard-driven navigation workflows across large codebases.

use crate::repository::file_node::RepositoryFile;
use crate::repository::quick_search::QuickFileSearch;
use crate::semantic::index::{SemanticIndex, SemanticSymbolID};
use crate::syntax::outline::DocumentSymbol;
use crate::theme::AppTheme;
use crate::ui::icons::Icon;
use egui::{Context, Key, TextEdit, Window};

/// Action triggered by user input inside any modal dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalAction {
    /// Opens the specified file path relative to repository root.
    OpenFile(String),
    /// Jumps the editor caret to the given 1-based line number.
    JumpToLine(usize),
    /// Jumps the editor caret to the symbol starting at the given byte offset.
    JumpToSymbol(usize),
    /// Resolves the canonical definition for an indexed symbol identity.
    JumpToIndexedSymbol(SemanticSymbolID),
    /// Fetches and opens the GitHub pull request with the given number.
    OpenPullRequest(usize),
    /// Dismisses the modal dialog without performing an action.
    Close,
}

/// Modal dialog for fuzzy file path search and navigation (Ctrl/Cmd+K).
#[derive(Debug, Clone, Default)]
pub struct QuickFilePickerModal {
    /// Current search query string typed by the user.
    pub query: String,
    /// Currently highlighted index in the matched results list.
    pub selected_idx: usize,
}

impl QuickFilePickerModal {
    /// Renders the quick file picker dialog and handles arrow key and enter selections.
    ///
    /// # Arguments
    ///
    /// * `ctx` - EGUI context for global window rendering and keyboard input.
    /// * `theme` - Visual color palette.
    /// * `files` - List of all repository files available for fuzzy matching.
    pub fn show(
        &mut self,
        ctx: &Context,
        theme: &AppTheme,
        files: &[RepositoryFile],
    ) -> Option<ModalAction> {
        let mut action = None;

        Window::new("Go to File")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 100.0))
            .fixed_size(egui::vec2(550.0, 380.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    Icon::Search.ui(ui, 14.0, theme.text_muted);
                    let id = ui.make_persistent_id("file_picker_query");
                    ui.memory_mut(|memory| memory.request_focus(id));
                    let edit = TextEdit::singleline(&mut self.query)
                        .id(id)
                        .hint_text("Type a filename or fuzzy path...")
                        .desired_width(ui.available_width() - 20.0);
                    ui.add(edit);
                });

                ui.separator();

                let results = QuickFileSearch::search(files, &self.query, 12);

                if ctx.input(|i| i.key_pressed(Key::ArrowDown))
                    && self.selected_idx + 1 < results.files.len()
                {
                    self.selected_idx += 1;
                }
                if ctx.input(|i| i.key_pressed(Key::ArrowUp)) && self.selected_idx > 0 {
                    self.selected_idx -= 1;
                }
                if ctx.input(|i| i.key_pressed(Key::Enter)) {
                    if let Some(res) = results.files.get(self.selected_idx) {
                        action = Some(ModalAction::OpenFile(res.path.clone()));
                    }
                }
                if ctx.input(|i| i.key_pressed(Key::Escape)) {
                    action = Some(ModalAction::Close);
                }

                if self.selected_idx >= results.files.len() && !results.files.is_empty() {
                    self.selected_idx = 0;
                }

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (idx, res) in results.files.iter().enumerate() {
                        let is_selected = idx == self.selected_idx;
                        let bg = if is_selected {
                            theme.accent.linear_multiply(0.2)
                        } else {
                            egui::Color32::TRANSPARENT
                        };

                        let item_resp = egui::Frame::none()
                            .fill(bg)
                            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                            .rounding(4.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(&res.path).monospace());
                                });
                            })
                            .response;

                        if item_resp.interact(egui::Sense::click()).clicked() {
                            action = Some(ModalAction::OpenFile(res.path.clone()));
                        }
                    }
                });
            });

        action
    }
}

/// Modal dialog for directly jumping to a specific line number (Ctrl/Cmd+L).
#[derive(Debug, Clone, Default)]
pub struct GoToLineModal {
    /// String input typed into the line number entry field.
    pub line_str: String,
}

impl GoToLineModal {
    /// Renders the jump-to-line dialog.
    ///
    /// # Arguments
    ///
    /// * `ctx` - EGUI context for global window rendering and input.
    /// * `_theme` - Visual color palette.
    /// * `max_lines` - Total number of lines in the active source file buffer.
    pub fn show(
        &mut self,
        ctx: &Context,
        _theme: &AppTheme,
        max_lines: usize,
    ) -> Option<ModalAction> {
        let mut action = None;

        Window::new("Go to Line")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 150.0))
            .fixed_size(egui::vec2(300.0, 120.0))
            .show(ctx, |ui| {
                ui.label(format!("Enter line number (1 to {max_lines}):"));
                let id = ui.make_persistent_id("line_picker_query");
                ui.memory_mut(|memory| memory.request_focus(id));
                let edit = TextEdit::singleline(&mut self.line_str)
                    .id(id)
                    .hint_text("Line number")
                    .desired_width(ui.available_width());
                ui.add(edit);

                if ctx.input(|i| i.key_pressed(Key::Enter)) {
                    if let Ok(line_num) = self.line_str.trim().parse::<usize>() {
                        action = Some(ModalAction::JumpToLine(line_num.clamp(1, max_lines)));
                    }
                }
                if ctx.input(|i| i.key_pressed(Key::Escape)) {
                    action = Some(ModalAction::Close);
                }

                ui.horizontal(|ui| {
                    if ui.button("Jump").clicked() {
                        if let Ok(line_num) = self.line_str.trim().parse::<usize>() {
                            action = Some(ModalAction::JumpToLine(line_num.clamp(1, max_lines)));
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        action = Some(ModalAction::Close);
                    }
                });
            });

        action
    }
}

/// Modal dialog for filtering and jumping to AST outline symbols in the current file (Ctrl/Cmd+T).
#[derive(Debug, Clone, Default)]
pub struct GoToSymbolModal {
    /// Query string for filtering document symbol names.
    pub query: String,
    /// Currently highlighted index in the filtered symbol list.
    pub selected_idx: usize,
}

struct SymbolChoice {
    name: String,
    detail: String,
    icon: &'static str,
    action: ModalAction,
}

impl GoToSymbolModal {
    /// Searches indexed workspace symbols or the current document outline.
    /// Both modes preserve their typed destinations and share input/rendering.
    pub fn show(
        &mut self,
        ctx: &Context,
        theme: &AppTheme,
        semantic_index: Option<&SemanticIndex>,
        symbols: &[DocumentSymbol],
    ) -> Option<ModalAction> {
        let workspace = semantic_index.is_some();
        let mut action = None;
        Window::new(if workspace {
            "Go to Symbol (Workspace)"
        } else {
            "Go to Symbol"
        })
        .id(egui::Id::new("reader_symbol_picker"))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 100.0))
        .fixed_size(egui::vec2(540.0, 380.0))
        .show(ctx, |ui| {
            let changed = ui
                .horizontal(|ui| {
                    Icon::Code.ui(ui, 14.0, theme.accent);
                    let id = ui.make_persistent_id("symbol_picker_query");
                    ui.memory_mut(|memory| memory.request_focus(id));
                    let response = ui.add(
                        TextEdit::singleline(&mut self.query)
                            .id(id)
                            .hint_text(if workspace {
                                "Search symbols in workspace..."
                            } else {
                                "Symbol name..."
                            })
                            .desired_width(ui.available_width() - 20.0),
                    );
                    response.changed()
                })
                .inner;
            // Resolve after text input, including text and Enter in one frame.
            let choices = symbol_choices(&self.query, semantic_index, symbols);
            if changed {
                self.selected_idx = 0;
            }
            action = self.keyboard_selection(ctx, &choices);
            ui.separator();
            // Horizontal layout reserves at least the standard interaction height.
            let height = ui
                .text_style_height(&egui::TextStyle::Body)
                .max(ui.spacing().interact_size.y)
                + 8.0;
            let stride = height + ui.spacing().item_spacing.y;
            let id = ui.make_persistent_id("symbol_choices");
            let mut scroll = egui::ScrollArea::vertical().id_source("symbol_choices");
            let moved = ctx.input(|input| {
                input.key_pressed(Key::ArrowDown) || input.key_pressed(Key::ArrowUp)
            });
            if changed {
                scroll = scroll.vertical_scroll_offset(0.0);
            } else if moved {
                let offset = egui::scroll_area::State::load(ctx, id)
                    .unwrap_or_default()
                    .offset
                    .y;
                let top = self.selected_idx as f32 * stride;
                let bottom = top + height;
                let target = offset.min(top).max(bottom - ui.available_height()).max(0.0);
                scroll = scroll.vertical_scroll_offset(target);
            }
            scroll.show_rows(ui, height, choices.len(), |ui, rows| {
                for idx in rows {
                    let choice = &choices[idx];
                    let bg = if idx == self.selected_idx {
                        theme.accent.linear_multiply(0.2)
                    } else {
                        egui::Color32::TRANSPARENT
                    };
                    let response = egui::Frame::none()
                        .fill(bg)
                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                        .rounding(4.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.colored_label(theme.accent, choice.icon);
                                ui.add(
                                    egui::Label::new(egui::RichText::new(&choice.name).strong())
                                        .truncate(true),
                                );
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(&choice.detail).color(theme.text_muted),
                                    )
                                    .truncate(true),
                                )
                                .on_hover_text(&choice.detail);
                            })
                        })
                        .response;
                    if response.interact(egui::Sense::click()).clicked() {
                        action = Some(choice.action.clone());
                    }
                }
            });
        });
        action
    }

    fn keyboard_selection(
        &mut self,
        ctx: &Context,
        choices: &[SymbolChoice],
    ) -> Option<ModalAction> {
        self.selected_idx = self.selected_idx.min(choices.len().saturating_sub(1));
        if ctx.input(|input| input.key_pressed(Key::ArrowDown))
            && self.selected_idx + 1 < choices.len()
        {
            self.selected_idx += 1;
        }
        if ctx.input(|input| input.key_pressed(Key::ArrowUp)) {
            self.selected_idx = self.selected_idx.saturating_sub(1);
        }
        if ctx.input(|input| input.key_pressed(Key::Escape)) {
            return Some(ModalAction::Close);
        }
        if ctx.input(|input| input.key_pressed(Key::Enter)) {
            return choices
                .get(self.selected_idx)
                .map(|choice| choice.action.clone());
        }
        None
    }
}

fn symbol_choices(
    query: &str,
    index: Option<&SemanticIndex>,
    symbols: &[DocumentSymbol],
) -> Vec<SymbolChoice> {
    if let Some(index) = index {
        return index
            .search_symbols(query, 100)
            .into_iter()
            .map(|entry| SymbolChoice {
                name: entry.name,
                detail: format!("{}:{}", entry.path, entry.line),
                icon: "sym",
                action: ModalAction::JumpToIndexedSymbol(entry.id),
            })
            .collect();
    }
    let query = query.trim().to_lowercase();
    let mut stack: Vec<_> = symbols.iter().rev().collect();
    let mut choices = Vec::new();
    while let Some(symbol) = stack.pop() {
        if query.is_empty() || symbol.name.to_lowercase().contains(&query) {
            choices.push(SymbolChoice {
                name: symbol.name.clone(),
                detail: format!("line {}", symbol.line),
                icon: symbol.kind.icon(),
                action: ModalAction::JumpToSymbol(symbol.selection_range.start),
            });
        }
        stack.extend(symbol.children.iter().rev());
    }
    choices
}

/// Modal dialog for entering a pull request number to review from GitHub.
#[derive(Debug, Clone, Default)]
pub struct PullRequestModal {
    /// String input typed into the pull request number field.
    pub pr_str: String,
    focus_requested: bool,
}

impl PullRequestModal {
    /// Creates a number prompt, optionally prefilled from the active or recent PR.
    #[must_use]
    pub fn new(number: Option<usize>) -> Self {
        Self {
            pr_str: number.map(|number| number.to_string()).unwrap_or_default(),
            focus_requested: false,
        }
    }

    /// Renders the pull request number prompt dialog.
    ///
    /// # Arguments
    ///
    /// * `ctx` - EGUI context for global window rendering and input.
    /// * `_theme` - Visual color palette.
    pub fn show(&mut self, ctx: &Context, _theme: &AppTheme) -> Option<ModalAction> {
        let mut action = None;

        Window::new("Open Pull Request")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 150.0))
            .fixed_size(egui::vec2(320.0, 140.0))
            .show(ctx, |ui| {
                ui.label("Enter GitHub Pull Request #:");
                let id = ui.make_persistent_id("pull_request_query");
                if !self.focus_requested {
                    ui.memory_mut(|memory| memory.request_focus(id));
                }
                let edit = TextEdit::singleline(&mut self.pr_str)
                    .id(id)
                    .hint_text("PR number (e.g. 42)")
                    .desired_width(ui.available_width());
                let resp = ui.add(edit);
                if !self.focus_requested {
                    resp.request_focus();
                    self.focus_requested = true;
                }
                let number = self
                    .pr_str
                    .trim()
                    .parse::<usize>()
                    .ok()
                    .filter(|number| *number > 0);
                let submit = (resp.has_focus() || resp.lost_focus())
                    && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, Key::Enter));
                if submit {
                    action = number.map(ModalAction::OpenPullRequest);
                }
                if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, Key::Escape)) {
                    action = Some(ModalAction::Close);
                }
                if !self.pr_str.is_empty() && number.is_none() {
                    ui.label("Enter a positive pull request number.");
                }
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(number.is_some(), egui::Button::new("Open"))
                        .clicked()
                    {
                        action = number.map(ModalAction::OpenPullRequest);
                    }
                    if ui.button("Cancel").clicked() {
                        action = Some(ModalAction::Close);
                    }
                });
            });

        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::index::SymbolSearchEntry;

    fn frame(events: Vec<egui::Event>) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(700.0, 600.0),
            )),
            events,
            ..Default::default()
        }
    }
    fn key(key: Key, pressed: bool) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Default::default(),
        }
    }

    #[test]
    fn workspace_keyboard_and_pointer_selection_emit_symbol_identity() {
        let context = Context::default();
        let id = SemanticSymbolID::new("scip test fixture 1 target().", "");
        let mut index = SemanticIndex::default();
        index.search_entries.push(SymbolSearchEntry {
            id: id.clone(),
            name: "target".into(),
            kind: 0,
            path: "b.ts".into(),
            line: 3,
            definition_count: 1,
        });
        let mut modal = GoToSymbolModal::default();
        let mut action = None;
        let _ = context.run(frame(vec![]), |context| {
            action = modal.show(context, &AppTheme::dark(), Some(&index), &[]);
        });
        let output = context.run(frame(vec![]), |context| {
            action = modal.show(context, &AppTheme::dark(), Some(&index), &[]);
        });
        assert!(action.is_none());
        let position = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.job.text == "target" => {
                    Some(text.pos + text.galley.size() * 0.5)
                }
                _ => None,
            })
            .unwrap();
        let _ = context.run(
            frame(vec![egui::Event::Key {
                key: Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }]),
            |context| {
                action = modal.show(context, &AppTheme::dark(), Some(&index), &[]);
            },
        );
        assert_eq!(action, Some(ModalAction::JumpToIndexedSymbol(id.clone())));
        for pressed in [true, false] {
            let _ = context.run(
                frame(vec![
                    egui::Event::PointerMoved(position),
                    egui::Event::PointerButton {
                        pos: position,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ]),
                |context| {
                    action = modal.show(context, &AppTheme::dark(), Some(&index), &[]);
                },
            );
        }
        assert_eq!(action, Some(ModalAction::JumpToIndexedSymbol(id)));
    }
    #[test]
    fn query_text_and_enter_in_the_same_frame_select_the_new_match() {
        let context = Context::default();
        let mut index = SemanticIndex::default();
        for name in ["alpha", "target"] {
            index.search_entries.push(SymbolSearchEntry {
                id: SemanticSymbolID::new(name, ""),
                name: name.into(),
                kind: 0,
                path: "a.rs".into(),
                line: 1,
                definition_count: 1,
            });
        }
        let mut modal = GoToSymbolModal::default();
        for _ in 0..2 {
            let _ = context.run(frame(vec![]), |context| {
                modal.show(context, &AppTheme::dark(), Some(&index), &[]);
            });
        }
        let mut action = None;
        let _ = context.run(
            frame(vec![
                egui::Event::Text("target".into()),
                key(Key::Enter, true),
            ]),
            |context| {
                action = modal.show(context, &AppTheme::dark(), Some(&index), &[]);
            },
        );
        assert_eq!(
            action,
            Some(ModalAction::JumpToIndexedSymbol(SemanticSymbolID::new(
                "target", ""
            )))
        );
    }

    #[test]
    fn document_picker_virtualizes_rows_and_reveals_keyboard_selection() {
        use crate::syntax::outline::SymbolKind;
        let symbols: Vec<_> = (0..1000)
            .map(|n| DocumentSymbol {
                name: format!("f{n:04}"),
                kind: SymbolKind::Function,
                range: n * 8..n * 8 + 8,
                selection_range: n * 8..n * 8 + 5,
                line: n + 1,
                children: vec![],
                detail: None,
            })
            .collect();
        let context = Context::default();
        let mut modal = GoToSymbolModal::default();
        for _ in 0..2 {
            let _ = context.run(frame(vec![]), |context| {
                modal.show(context, &AppTheme::dark(), None, &symbols);
            });
        }
        modal.selected_idx = 998;
        let output = context.run(frame(vec![key(Key::ArrowDown, true)]), |context| {
            modal.show(context, &AppTheme::dark(), None, &symbols);
        });
        let painted: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
                _ => None,
            })
            .collect();
        assert!(
            painted.contains(&"f0999"),
            "keyboard target not painted: {painted:?}"
        );
        assert!(painted.len() < 100, "offscreen symbol rows were rendered");
        let mut action = None;
        let _ = context.run(frame(vec![key(Key::Enter, true)]), |context| {
            action = modal.show(context, &AppTheme::dark(), None, &symbols);
        });
        assert_eq!(action, Some(ModalAction::JumpToSymbol(999 * 8)));
    }
}

#[cfg(test)]
mod pull_request_tests {
    use super::*;

    fn key(key: Key) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }
    }
    fn show(
        modal: &mut PullRequestModal,
        ctx: &Context,
        events: Vec<egui::Event>,
    ) -> Option<ModalAction> {
        let mut action = None;
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(700.0, 600.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| action = modal.show(ctx, &AppTheme::dark()),
        );
        action
    }

    #[test]
    fn number_prompt_rejects_zero_invalid_and_overflow_and_submits_only_a_focused_valid_field() {
        for value in [
            "0",
            "-1",
            "invalid",
            "9999999999999999999999999999999999",
            "",
        ] {
            let ctx = Context::default();
            let mut modal = PullRequestModal {
                pr_str: value.into(),
                ..Default::default()
            };
            show(&mut modal, &ctx, vec![]);
            assert_eq!(
                show(&mut modal, &ctx, vec![key(Key::Enter)]),
                None,
                "{value}"
            );
        }
        let ctx = Context::default();
        let mut modal = PullRequestModal::new(Some(7));
        show(&mut modal, &ctx, vec![]);
        assert_eq!(
            show(&mut modal, &ctx, vec![key(Key::Enter)]),
            Some(ModalAction::OpenPullRequest(7))
        );
        let ctx = Context::default();
        let mut modal = PullRequestModal::new(Some(7));
        show(&mut modal, &ctx, vec![]);
        show(&mut modal, &ctx, vec![key(Key::Tab)]);
        assert_eq!(show(&mut modal, &ctx, vec![key(Key::Tab)]), None);
        assert_eq!(
            show(&mut modal, &ctx, vec![key(Key::Enter)]),
            Some(ModalAction::Close),
            "Cancel keeps keyboard focus instead of being stolen by the input"
        );
    }
}
