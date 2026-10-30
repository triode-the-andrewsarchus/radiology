//! Compact definition states and bounded index facts for the inspector.

use super::context_inspector::{ContextInspector, InspectorAction};
use crate::{semantic::context::UnderCaretContext, source::preview::PreviewPhase, theme::AppTheme};
use egui::{Label, RichText, Ui};

pub(super) fn show(
    ui: &mut Ui,
    theme: &AppTheme,
    context: &UnderCaretContext,
    preview: &PreviewPhase,
    expanded: bool,
) -> Option<InspectorAction> {
    let mut action = None;
    if ui
        .add(egui::Button::new(RichText::new(&context.name).strong()).frame(false))
        .clicked()
    {
        ui.memory_mut(|memory| memory.request_focus(ContextInspector::definition_focus_id()));
    }
    if let Some(next) = show_location(ui, theme, context) {
        action = Some(next);
    }
    if ui
        .small_button(if expanded {
            "Hide details"
        } else {
            "Show details"
        })
        .clicked()
    {
        ui.memory_mut(|memory| memory.request_focus(ContextInspector::definition_focus_id()));
        action = Some(InspectorAction::ExpandDefinition(!expanded));
    }
    if expanded {
        show_details(ui, theme, context, preview);
    }
    action
}

fn show_location(
    ui: &mut Ui,
    theme: &AppTheme,
    context: &UnderCaretContext,
) -> Option<InspectorAction> {
    if context.is_on_definition {
        let file = context
            .location
            .path
            .rsplit('/')
            .next()
            .unwrap_or(&context.location.path);
        ui.colored_label(theme.text_muted, format!("{file}:{} · Here", context.line))
            .on_hover_text(format!(
                "Defined here · {} indexed definitions",
                context.definition_count
            ));
    } else if let Some(destination) = &context.destination {
        let line = i64::from(destination.range.start_line) + 1;
        let file = destination
            .path
            .rsplit('/')
            .next()
            .unwrap_or(&destination.path);
        let more = if context.definition_count > 1 {
            format!(" +{}", context.definition_count - 1)
        } else {
            String::new()
        };
        if ui.link(format!("{file}:{line}{more}")).on_hover_text(format!("Go to {}:{line}. Destination bytes are validated before navigation; Back returns to the caret.", destination.path)).clicked() {
            return Some(InspectorAction::JumpToIndexedLocation { path: destination.path.clone(), range: destination.range.clone() });
        }
    } else {
        let text = if context.id.is_local() {
            "Local definition not indexed"
        } else if context.definition_count > 0 {
            "Definition unavailable"
        } else {
            context
                .external_package
                .as_deref()
                .unwrap_or("Definition not indexed")
        };
        ui.colored_label(theme.text_muted, text).on_hover_text(if context.id.is_local() { "This symbol is scoped to this document; the index records no usable local definition." } else { "No usable repository definition is indexed. This may be external code or missing index coverage; package metadata does not establish a source destination." });
    }
    None
}

fn show_details(
    ui: &mut Ui,
    theme: &AppTheme,
    context: &UnderCaretContext,
    preview: &PreviewPhase,
) {
    let mut facts = if context.id.is_local() {
        format!(
            "Local {} · {} occurrence{} in this file",
            context.kind.to_lowercase(),
            context.file_occurrence_count,
            if context.file_occurrence_count == 1 {
                ""
            } else {
                "s"
            }
        )
    } else {
        format!(
            "{} · {} indexed reference{} · {} definition{}",
            context.kind,
            context.reference_count,
            if context.reference_count == 1 {
                ""
            } else {
                "s"
            },
            context.definition_count,
            if context.definition_count == 1 {
                ""
            } else {
                "s"
            }
        )
    };
    if let Some(enclosing) = &context.enclosing_name {
        facts.push_str(&format!(" · in {enclosing}"));
    }
    ui.colored_label(theme.text_muted, facts).on_hover_text("Counts describe indexed sites. The References region separately reports validated source results and incomplete coverage.");
    if let Some(package) = &context.external_package {
        ui.colored_label(theme.text_muted, format!("Package · {package}"));
    }
    if !context.signature.is_empty() {
        ui.add(Label::new(RichText::new(&context.signature).monospace()).selectable(true));
    }
    if let Some(documentation) = &context.documentation {
        ui.add(
            Label::new(RichText::new(documentation).color(theme.text_secondary)).selectable(true),
        );
    }
    if context.id.is_local() {
        if !context.is_on_definition {
            if let Some(line) = &context.local_definition_line {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("{:>3}", line.number))
                            .monospace()
                            .color(theme.text_muted),
                    );
                    ui.add(
                        Label::new(super::source_preview::line_job(line, theme))
                            .truncate(true)
                            .selectable(true),
                    );
                });
            }
        }
    } else {
        super::source_preview::show(ui, theme, preview);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        semantic::{
            context::UnderCaretContext, ScipDocument, ScipIndex, ScipSymbolInformation,
            SemanticIndex,
        },
        source::SourceBuffer,
    };
    use std::sync::atomic::AtomicBool;

    #[test]
    fn compact_states_and_expanded_index_facts_are_distinct() {
        let buffer =
            SourceBuffer::from_str("source.rs", "fn target() {}\nfn main() { target(); }\n");
        let symbol = "test cargo app 1 target().";
        let occurrences = [
            buffer.text.find("target").unwrap(),
            buffer.text.rfind("target").unwrap(),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, byte)| {
            crate::semantic::test_support::occurrence(&buffer, byte..byte + 6, symbol, index == 0)
        })
        .collect();
        let index = SemanticIndex::from_scip(ScipIndex {
            documents: vec![ScipDocument {
                relative_path: buffer.path.clone(),
                position_encoding: 2,
                occurrences,
                symbols: vec![ScipSymbolInformation {
                    symbol: symbol.into(),
                    kind: 17,
                    signature: "fn target()".into(),
                    documentation: vec!["Useful documentation.".into()],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        });
        let mut context = UnderCaretContext::prepare(
            &buffer,
            &index,
            buffer.text.rfind("target").unwrap(),
            &AtomicBool::new(false),
        )
        .unwrap()
        .unwrap();
        let render = |context: &UnderCaretContext, expanded| {
            let ctx = egui::Context::default();
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 500.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        show(
                            ui,
                            &AppTheme::dark(),
                            context,
                            &PreviewPhase::None,
                            expanded,
                        );
                    });
                },
            );
            output
                .shapes
                .iter()
                .filter_map(|shape| match &shape.shape {
                    egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        let compact = render(&context, false);
        assert!(compact.contains("source.rs:1"));
        assert!(!compact.contains("Useful documentation"));
        let expanded = render(&context, true);
        assert!(expanded.contains("Function · 1 indexed reference"));
        assert!(expanded.contains("fn target()") && expanded.contains("Useful documentation"));
        context.is_on_definition = true;
        context.line = 1;
        context.destination = None;
        assert!(render(&context, false).contains("source.rs:1 · Here"));
        context.is_on_definition = false;
        context.definition_count = 0;
        context.external_package = Some("dependency 2.3".into());
        assert!(render(&context, false).contains("dependency 2.3"));
    }
}
