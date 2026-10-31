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
    let name = super::text_fit::middle(
        ui,
        &context.name,
        egui::TextStyle::Body.resolve(ui.style()),
        theme.text_primary,
        (ui.available_width() - 2.0 * ui.spacing().button_padding.x).max(0.0),
    );
    if ui.add(egui::Button::new(name).frame(false)).clicked() {
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
        let label = super::text_fit::middle(
            ui,
            &format!("{file}:{} · Here", context.line),
            egui::TextStyle::Small.resolve(ui.style()),
            theme.text_muted,
            ui.available_width(),
        );
        ui.add(Label::new(label)).on_hover_text(format!(
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
        let label = super::text_fit::middle(
            ui,
            &format!("{file}:{line}{more}"),
            egui::TextStyle::Small.resolve(ui.style()),
            theme.accent,
            ui.available_width(),
        );
        if ui
            .add(egui::Link::new(label))
            .on_hover_text(format!(
                "Go to {}:{line}. Back returns to the caret.",
                super::text_fit::bounded(&destination.path)
            ))
            .clicked()
        {
            return Some(InspectorAction::JumpToIndexedLocation {
                path: destination.path.clone(),
                range: destination.range.clone(),
            });
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
        let text = super::text_fit::middle(
            ui,
            text,
            egui::TextStyle::Small.resolve(ui.style()),
            theme.text_muted,
            ui.available_width(),
        );
        ui.add(Label::new(text)).on_hover_text(if context.id.is_local() { "This symbol is scoped to this document; the index records no usable local definition." } else { "No usable repository definition is indexed. This may be external code or missing index coverage; package metadata does not establish a source destination." });
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
    let facts = super::text_fit::lines(
        ui,
        &facts,
        egui::TextStyle::Small.resolve(ui.style()),
        theme.text_muted,
        ui.available_width(),
        2,
    );
    ui.add(Label::new(facts)).on_hover_text("Counts describe indexed sites. The References region separately reports validated source results and incomplete coverage.");
    if let Some(package) = &context.external_package {
        let label = super::text_fit::middle(
            ui,
            &format!("Package · {package}"),
            egui::TextStyle::Small.resolve(ui.style()),
            theme.text_muted,
            ui.available_width(),
        );
        ui.add(Label::new(label))
            .on_hover_text(super::text_fit::bounded(package).as_ref());
    }
    if !context.signature.is_empty() {
        let signature = super::text_fit::lines(
            ui,
            &context.signature,
            egui::FontId::monospace(12.0),
            theme.text_primary,
            ui.available_width(),
            8,
        );
        ui.add(Label::new(signature).selectable(true));
    }
    if let Some(documentation) = &context.documentation {
        let text = super::text_fit::lines(
            ui,
            documentation,
            egui::TextStyle::Small.resolve(ui.style()),
            theme.text_secondary,
            ui.available_width(),
            4,
        );
        ui.add(Label::new(text).selectable(true));
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

    fn indexed_context() -> UnderCaretContext {
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
        UnderCaretContext::prepare(
            &buffer,
            &index,
            buffer.text.rfind("target").unwrap(),
            &AtomicBool::new(false),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn compact_states_and_expanded_index_facts_are_distinct() {
        let mut context = indexed_context();
        let render = |context: &UnderCaretContext, expanded| {
            crate::ui::test_support::painted_text(
                &egui::Context::default(),
                egui::vec2(400.0, 500.0),
                |ui| {
                    show(
                        ui,
                        &AppTheme::dark(),
                        context,
                        &PreviewPhase::None,
                        expanded,
                    );
                },
            )
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
    #[test]
    fn long_indexed_details_keep_pane_width_and_full_navigation_target() {
        let mut context = indexed_context();
        context.name = format!("λ{}_TAIL", "🦀long-identifier".repeat(1000));
        context.enclosing_name = Some(context.name.clone());
        context.external_package = Some(context.name.clone());
        context.documentation = Some("Documentation words ".repeat(15));
        context.signature =
            "fn signature(".to_owned() + &"argument: VeryLongType, ".repeat(25) + ")";
        let destination = context.destination.as_mut().unwrap();
        destination.path = format!("nested/{}.rs", "long-🦀-source-".repeat(60));
        let expected = InspectorAction::JumpToIndexedLocation {
            path: destination.path.clone(),
            range: destination.range.clone(),
        };
        for dark in [true, false] {
            for scale in [1.0, 2.0] {
                for width in [216.0, 300.0, 420.0] {
                    let (ctx, theme) = crate::ui::test_support::themed_context(dark, scale);
                    let mut action = None;
                    let mut frame = |events| {
                        crate::ui::test_support::painted_frame(
                            &ctx,
                            egui::vec2(width, 800.0),
                            events,
                            |ui| {
                                ui.style_mut().interaction.selectable_labels = false;
                                action = show(ui, &theme, &context, &PreviewPhase::None, true);
                            },
                        )
                    };
                    frame(vec![]); // Apply egui's pending scale before measuring geometry.
                    let output = frame(vec![]);
                    let mut location = None;
                    for shape in &output.shapes {
                        if let egui::Shape::Text(text) = &shape.shape {
                            assert!(
                                text.pos.x + text.galley.size().x <= width + 0.1,
                                "overflow at {:?} size {:?} rows {}: {:?}",
                                text.pos,
                                text.galley.size(),
                                text.galley.rows.len(),
                                text.galley.job.text
                            );
                            assert!(text.galley.job.text.len() <= 4099);
                            if text.galley.job.text.starts_with("fn signature(") {
                                assert!(text.galley.rows.len() <= 8);
                            }
                            if text.galley.job.text.starts_with("Documentation") {
                                assert!(text.galley.rows.len() <= 4);
                            }
                            if text.galley.job.text.ends_with(".rs:1") {
                                location = Some(text.pos + text.galley.size() / 2.0);
                            }
                        }
                    }
                    frame(crate::ui::test_support::click(
                        location.expect("fitted destination keeps line suffix"),
                    ));
                    assert_eq!(action, Some(expected.clone()));
                }
            }
        }
    }
}
