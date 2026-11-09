//! Source-bound enclosing-declaration summary and signature disclosure.
use super::{AppTheme, InspectorAction};
use crate::{
    source::{buffer::SourceIdentity, SourceBuffer},
    syntax::outline::DocumentSymbol,
};
use egui::Ui;
use std::ops::Range;

/// Retains signature disclosure for the current exact declaration.
///
/// Pass the same value to [`super::ContextInspector::show`] across frames.
/// Reset it to its default when changing repositories, including repositories
/// containing identical relative paths and source bytes.
#[derive(Default)]
pub struct DeclarationSummary {
    owner: Option<(SourceIdentity, Range<usize>)>,
    expanded: bool,
}

impl DeclarationSummary {
    pub(super) fn show(
        &mut self,
        ui: &mut Ui,
        theme: &AppTheme,
        declaration: Option<&DocumentSymbol>,
        buffer: Option<&SourceBuffer>,
        pinned: Option<(&str, i32)>,
        semantic_kind: i32,
    ) -> Option<InspectorAction> {
        let current = declaration.zip(buffer).filter(|_| pinned.is_none());
        let owner = current
            .map(|(symbol, buffer)| (SourceIdentity::new(buffer), symbol.selection_range.clone()));
        if self.owner != owner {
            self.owner = owner;
            self.expanded = false;
            ui.scroll_to_cursor(Some(egui::Align::Min));
        }
        if let Some((name, kind)) = pinned {
            ui.horizontal(|ui| {
                ui.colored_label(theme.text_muted, kind_glyph(kind));
                let text = crate::ui::text_fit::middle(
                    ui,
                    name,
                    egui::TextStyle::Body.resolve(ui.style()),
                    theme.text_primary,
                    ui.available_width(),
                );
                ui.add(egui::Label::new(text))
                    .on_hover_text(crate::ui::text_fit::bounded(name).as_ref());
            });
            ui.label(
                egui::RichText::new(crate::semantic::context::kind_name(kind))
                    .small()
                    .color(theme.text_secondary),
            );
            return None;
        }
        let Some((symbol, buffer)) = current else {
            ui.colored_label(theme.text_muted, "No enclosing declaration");
            return None;
        };
        let mut action = None;
        ui.horizontal(|ui| {
            let icon = if self.expanded {
                crate::ui::icons::Icon::ChevronDown
            } else {
                crate::ui::icons::Icon::ChevronRight
            };
            if icon
                .button(ui, 10.0, theme.text_secondary)
                .on_hover_text(if self.expanded {
                    "Hide signature"
                } else {
                    "Show signature"
                })
                .clicked()
            {
                self.expanded = !self.expanded;
            }
            ui.colored_label(theme.text_muted, symbol.kind.icon());
            let name = crate::ui::text_fit::middle(
                ui,
                &symbol.name,
                egui::TextStyle::Body.resolve(ui.style()),
                theme.accent,
                ui.available_width(),
            );
            if ui
                .add(egui::Link::new(name))
                .on_hover_text(crate::ui::text_fit::bounded(&symbol.name).as_ref())
                .clicked()
            {
                action = Some(InspectorAction::JumpToSymbol(symbol.selection_range.start));
            }
        });
        let line = buffer.line_at_byte(symbol.selection_range.start);
        let location = format!(
            "{}:{line}",
            buffer.path.rsplit('/').next().unwrap_or(&buffer.path)
        );
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(if semantic_kind == 0 {
                    symbol.kind.title().to_owned()
                } else {
                    crate::semantic::context::kind_name(semantic_kind)
                })
                .small()
                .color(theme.text_secondary),
            );
            let location = crate::ui::text_fit::middle(
                ui,
                &location,
                egui::TextStyle::Small.resolve(ui.style()),
                theme.accent,
                ui.available_width(),
            );
            if ui
                .add(egui::Link::new(location))
                .on_hover_text(format!(
                    "Go to {}:{line}. Back returns to the caret.",
                    crate::ui::text_fit::bounded(&buffer.path)
                ))
                .clicked()
            {
                action = Some(InspectorAction::JumpToSymbol(symbol.selection_range.start));
            }
        });
        if self.expanded {
            let signature = symbol
                .detail
                .as_deref()
                .unwrap_or_else(|| buffer.line_text(symbol.line))
                .trim();
            let display = display_signature(signature, symbol, buffer);
            let font = egui::FontId::monospace(12.0);
            let available_rows = ((ui.available_height() - 64.0)
                / ui.fonts(|fonts| fonts.row_height(&font)))
            .floor()
            .clamp(1.0, 8.0) as usize;
            let text = crate::ui::text_fit::lines(
                ui,
                &display,
                font,
                theme.text_primary,
                ui.available_width(),
                available_rows,
            );
            let response = ui.add(egui::Label::new(text).selectable(true));
            use crate::ui::context_menu::Item;
            if crate::ui::context_menu::show(&response, &self.owner, || {
                vec![Item::action("Copy Signature", ()).shortcut(
                    egui::Key::C,
                    egui::Modifiers::CTRL,
                    "Ctrl+C",
                )]
            })
            .is_some()
            {
                ui.ctx()
                    .output_mut(|output| output.copied_text = signature.to_owned());
            }
        }
        action
    }
}

// Keep source/detail ownership and exact-copy text separate from bounded display.
fn display_signature(signature: &str, symbol: &DocumentSymbol, buffer: &SourceBuffer) -> String {
    let bounded = crate::semantic::context::bounded_signature(signature);
    let line_start = buffer.line_starts[buffer.line_at_byte(symbol.range.start) - 1];
    let indent = buffer
        .text
        .get(line_start..symbol.range.start)
        .filter(|indent| {
            indent.len() <= bounded.len() && indent.bytes().all(|byte| matches!(byte, b' ' | b'\t'))
        })
        .unwrap_or("");
    bounded
        .lines()
        .enumerate()
        .map(|(offset, line)| {
            if offset == 0 {
                line
            } else {
                line.strip_prefix(indent).unwrap_or(line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_owned()
}

fn kind_glyph(kind: i32) -> &'static str {
    use crate::semantic::symbol_kind::SymbolCategory;
    match SymbolCategory::from_kind(kind) {
        SymbolCategory::Type => "T",
        SymbolCategory::Function => "ƒ",
        SymbolCategory::Value => "v",
        SymbolCategory::Module => "M",
        SymbolCategory::Other => "·",
    }
}

#[cfg(test)]
mod tests;
