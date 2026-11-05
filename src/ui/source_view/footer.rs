//! Content-sized source metadata, outside the selectable source canvas.

use super::*;

pub(super) fn show(
    ui: &mut Ui,
    theme: &AppTheme,
    content: &SourceViewContent<'_>,
) -> Option<SourceViewAction> {
    let mut action = None;
    egui::TopBottomPanel::bottom(ui.id().with("source_footer"))
        .frame(egui::Frame::none().inner_margin(egui::Margin::symmetric(8.0, 2.0)))
        .show_inside(ui, |ui| {
            action = show_pin_groups(ui, theme, content.pin_store);
            ui.horizontal_wrapped(|ui| {
                ui.style_mut().override_text_style = Some(egui::TextStyle::Small);
                ui.spacing_mut().item_spacing.x = 4.0;
                let language = crate::syntax::language::LanguageRegistry::for_path(&content.buffer.path);
                ui.colored_label(theme.text_secondary, language.display_name);
                ui.colored_label(theme.text_muted, "·");
                ui.colored_label(theme.text_secondary, format!("{} lines", content.buffer.line_count()));
                let structure_status = content.structure_error
                    .map(|error| ("Structure unavailable", error))
                    .or_else(|| content.partial_structure.then_some((
                        "Partial structure",
                        "This file does not fully parse, so highlighting, the outline and other structural results may be incomplete. Source remains available.",
                    )));
                let errors = [
                    ("Mark settings unavailable", content.mark_error),
                    ("Folding unavailable", content.fold_error),
                    ("Occurrence highlighting unavailable", content.occurrence_error),
                    ("Basic syntax colors", content.highlight_error),
                ].into_iter().filter_map(|(label, error)| error.map(|error| (label, error)));
                for (label, error) in structure_status.into_iter().chain(errors) {
                    ui.colored_label(theme.text_secondary, label).on_hover_text(error);
                }
                let test_status = content.test_code.status();
                for status in content.critical_status.into_iter().chain(test_status.as_deref()) {
                    ui.colored_label(theme.text_muted, "·");
                    ui.colored_label(theme.text_muted, status);
                }
                ui.colored_label(theme.text_muted, "·");
                if content.has_semantic_index {
                    ui.colored_label(Color32::from_rgb(34, 197, 94), "Semantic data");
                } else {
                    ui.colored_label(theme.text_muted, if content.buffer.revision == Some(ReviewRevision::Before) {
                        "Not Indexed (Before)"
                    } else {
                        "No Index"
                    });
                }
                ui.colored_label(theme.text_muted, "·");
                let line_btn = ui.add(egui::Button::new(
                    RichText::new(format!("Line {}", *content.caret_line)).small().color(theme.text_secondary)
                ).frame(false));
                if line_btn.on_hover_text("Go to Line (Ctrl+L)").clicked() {
                    action = Some(SourceViewAction::OpenGoToLine);
                }
                if let Some(revision) = content.buffer.revision {
                    ui.colored_label(theme.text_muted, "·");
                    ui.colored_label(theme.accent, revision.title());
                }
                ui.colored_label(theme.text_muted, "·");
                ui.colored_label(theme.text_muted, "UTF-8");
                ui.colored_label(theme.text_muted, "·");
                ui.colored_label(theme.text_muted, "Read Only");
            });
        });
    action
}
