//! Modal dialog overlays for fuzzy file navigation, line jumping, symbol outlines, and pull request review.
//!
//! Provides focused modal popups for keyboard-driven navigation workflows across large codebases.
//! Hosts use [`ModalInput`] to capture input before egui begins its focus pass.

use crate::semantic::index::SemanticSymbolID;
use crate::theme::AppTheme;
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
    /// Queues a durable Code Path import, retaining the sheet until save succeeds.
    ImportCodePath(String),
    /// Dismisses the modal dialog without performing an action.
    Close,
}

mod input;
pub use input::ModalInput;

mod file_picker;
pub use file_picker::QuickFilePickerModal;

/// Modal dialog for directly jumping to a specific line number (Ctrl/Cmd+L).
#[derive(Debug, Clone, Default)]
pub struct GoToLineModal {
    /// String input typed into the line number entry field.
    pub line_str: String,
    focus_requested: bool,
}

impl GoToLineModal {
    /// Creates a line prompt prefilled with the current one-based source line.
    #[must_use]
    pub fn new(line: usize) -> Self {
        Self {
            line_str: line.max(1).to_string(),
            ..Default::default()
        }
    }

    /// Renders the jump-to-line dialog.
    ///
    /// # Arguments
    ///
    /// * `ctx` - EGUI context for global window rendering and input.
    /// * `theme` - Visual color palette.
    /// * `max_lines` - Total number of lines in the active source file buffer.
    pub fn show(
        &mut self,
        ctx: &Context,
        theme: &AppTheme,
        max_lines: usize,
    ) -> Option<ModalAction> {
        let mut action = None;

        Window::new("Go to Line")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 150.0))
            .fixed_size(egui::vec2(300.0, 120.0))
            .show(ctx, |ui| {
                ctx.move_to_top(ui.layer_id());
                ui.label("Line number");
                let id = ui.make_persistent_id("line_picker_query");
                if !self.focus_requested {
                    ui.memory_mut(|memory| memory.request_focus(id));
                    let mut state = egui::TextEdit::load_state(ctx, id).unwrap_or_default();
                    state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::two(
                            egui::text::CCursor::new(0),
                            egui::text::CCursor::new(self.line_str.chars().count()),
                        )));
                    state.store(ctx, id);
                    self.focus_requested = ui.is_visible();
                }
                let edit = TextEdit::singleline(&mut self.line_str)
                    .id(id)
                    .hint_text("Line number")
                    .desired_width(ui.available_width());
                let response = ui.add(edit);
                let number = self.line_str.trim().parse::<i64>().ok().map(|line| {
                    usize::try_from(line.max(1))
                        .unwrap_or(usize::MAX)
                        .min(max_lines.max(1))
                });
                let submit = (response.has_focus() || response.lost_focus())
                    && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, Key::Enter));
                if submit {
                    action = number.map(ModalAction::JumpToLine);
                    if action.is_none() {
                        response.request_focus();
                    }
                }
                let detail = if number.is_some() || self.line_str.is_empty() {
                    format!("1–{}", max_lines.max(1))
                } else {
                    "Enter a line number.".into()
                };
                action = picker::footer(
                    ui,
                    theme,
                    &detail,
                    "Go",
                    number.map(ModalAction::JumpToLine),
                    response.id,
                )
                .or(action.take());
                if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, Key::Escape)) {
                    action = Some(ModalAction::Close);
                }
            });

        action
    }
}

mod picker;
mod symbol_picker;
pub use symbol_picker::GoToSymbolModal;

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
                ctx.move_to_top(ui.layer_id());
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
                action = picker::footer(
                    ui,
                    _theme,
                    "",
                    "Open",
                    number.map(ModalAction::OpenPullRequest),
                    resp.id,
                )
                .or(action.take());
            });

        action
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

mod code_path_import;
pub use code_path_import::CodePathImportModal;
