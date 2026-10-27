use crate::repository::file_node::RepositoryFile;
use crate::repository::quick_search::QuickFileSearch;
use crate::syntax::outline::DocumentSymbol;
use crate::theme::AppTheme;
use egui::{Context, Key, TextEdit, Window};

pub enum ModalAction {
    OpenFile(String),
    JumpToLine(usize),
    JumpToSymbol(usize), // byte offset
    OpenPullRequest(usize),
    Close,
}

#[derive(Debug, Clone, Default)]
pub struct QuickFilePickerModal {
    pub query: String,
    pub selected_idx: usize,
}

impl QuickFilePickerModal {
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
                    ui.label("🔍");
                    let edit = TextEdit::singleline(&mut self.query)
                        .hint_text("Type a filename or fuzzy path...")
                        .desired_width(ui.available_width() - 20.0);
                    let resp = ui.add(edit);
                    resp.request_focus();
                });

                ui.separator();

                let results = QuickFileSearch::search(files, &self.query, 12);

                if ctx.input(|i| i.key_pressed(Key::ArrowDown))
                    && self.selected_idx + 1 < results.len()
                {
                    self.selected_idx += 1;
                }
                if ctx.input(|i| i.key_pressed(Key::ArrowUp)) && self.selected_idx > 0 {
                    self.selected_idx -= 1;
                }
                if ctx.input(|i| i.key_pressed(Key::Enter)) {
                    if let Some(res) = results.get(self.selected_idx) {
                        action = Some(ModalAction::OpenFile(res.file.path.clone()));
                    }
                }
                if ctx.input(|i| i.key_pressed(Key::Escape)) {
                    action = Some(ModalAction::Close);
                }

                if self.selected_idx >= results.len() && !results.is_empty() {
                    self.selected_idx = 0;
                }

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (idx, res) in results.iter().enumerate() {
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
                                    ui.label(egui::RichText::new(&res.file.path).monospace());
                                });
                            })
                            .response;

                        if item_resp.interact(egui::Sense::click()).clicked() {
                            action = Some(ModalAction::OpenFile(res.file.path.clone()));
                        }
                    }
                });
            });

        action
    }
}

#[derive(Debug, Clone, Default)]
pub struct GoToLineModal {
    pub line_str: String,
}

impl GoToLineModal {
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
                let edit = TextEdit::singleline(&mut self.line_str)
                    .hint_text("Line number")
                    .desired_width(ui.available_width());
                let resp = ui.add(edit);
                resp.request_focus();

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

#[derive(Debug, Clone, Default)]
pub struct GoToSymbolModal {
    pub query: String,
    pub selected_idx: usize,
}

impl GoToSymbolModal {
    pub fn show(
        &mut self,
        ctx: &Context,
        theme: &AppTheme,
        symbols: &[DocumentSymbol],
    ) -> Option<ModalAction> {
        let mut action = None;

        let mut flattened = Vec::new();
        fn flatten_syms(syms: &[DocumentSymbol], out: &mut Vec<DocumentSymbol>) {
            for s in syms {
                out.push(s.clone());
                flatten_syms(&s.children, out);
            }
        }
        flatten_syms(symbols, &mut flattened);

        let query_lower = self.query.trim().to_lowercase();
        let matches: Vec<&DocumentSymbol> = flattened
            .iter()
            .filter(|s| query_lower.is_empty() || s.name.to_lowercase().contains(&query_lower))
            .collect();

        Window::new("Go to Symbol")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 100.0))
            .fixed_size(egui::vec2(500.0, 360.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("🔣");
                    let edit = TextEdit::singleline(&mut self.query)
                        .hint_text("Symbol name...")
                        .desired_width(ui.available_width() - 20.0);
                    let resp = ui.add(edit);
                    resp.request_focus();
                });

                ui.separator();

                if ctx.input(|i| i.key_pressed(Key::ArrowDown))
                    && self.selected_idx + 1 < matches.len()
                {
                    self.selected_idx += 1;
                }
                if ctx.input(|i| i.key_pressed(Key::ArrowUp)) && self.selected_idx > 0 {
                    self.selected_idx -= 1;
                }
                if ctx.input(|i| i.key_pressed(Key::Enter)) {
                    if let Some(sym) = matches.get(self.selected_idx) {
                        action = Some(ModalAction::JumpToSymbol(sym.selection_range.start));
                    }
                }
                if ctx.input(|i| i.key_pressed(Key::Escape)) {
                    action = Some(ModalAction::Close);
                }

                if self.selected_idx >= matches.len() && !matches.is_empty() {
                    self.selected_idx = 0;
                }

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (idx, sym) in matches.iter().enumerate() {
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
                                    ui.colored_label(theme.accent, sym.kind.icon());
                                    ui.label(egui::RichText::new(&sym.name).strong());
                                    ui.colored_label(
                                        theme.text_muted,
                                        format!("line {}", sym.line),
                                    );
                                });
                            })
                            .response;

                        if item_resp.interact(egui::Sense::click()).clicked() {
                            action = Some(ModalAction::JumpToSymbol(sym.selection_range.start));
                        }
                    }
                });
            });

        action
    }
}

#[derive(Debug, Clone, Default)]
pub struct PullRequestModal {
    pub pr_str: String,
}

impl PullRequestModal {
    pub fn show(&mut self, ctx: &Context, _theme: &AppTheme) -> Option<ModalAction> {
        let mut action = None;

        Window::new("Open Pull Request")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 150.0))
            .fixed_size(egui::vec2(320.0, 140.0))
            .show(ctx, |ui| {
                ui.label("Enter GitHub Pull Request #:");
                let edit = TextEdit::singleline(&mut self.pr_str)
                    .hint_text("PR number (e.g. 42)")
                    .desired_width(ui.available_width());
                let resp = ui.add(edit);
                resp.request_focus();

                if ctx.input(|i| i.key_pressed(Key::Enter)) {
                    if let Ok(num) = self.pr_str.trim().parse::<usize>() {
                        action = Some(ModalAction::OpenPullRequest(num));
                    }
                }
                if ctx.input(|i| i.key_pressed(Key::Escape)) {
                    action = Some(ModalAction::Close);
                }

                ui.horizontal(|ui| {
                    if ui.button("Review PR").clicked() {
                        if let Ok(num) = self.pr_str.trim().parse::<usize>() {
                            action = Some(ModalAction::OpenPullRequest(num));
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
