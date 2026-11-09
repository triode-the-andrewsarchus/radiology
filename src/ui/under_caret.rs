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
    repeats_heading: bool,
) -> Option<InspectorAction> {
    let destination = context.destination.as_ref().map(|target| {
        (
            &target.path,
            [
                target.range.start_line,
                target.range.start_col,
                target.range.end_line,
                target.range.end_col,
            ],
        )
    });
    let owner = (
        &context.id,
        &context.location,
        &context.content_digest,
        destination,
    );
    if !repeats_heading {
        let name = super::text_fit::middle(
            ui,
            &context.name,
            egui::TextStyle::Body.resolve(ui.style()),
            theme.text_primary,
            (ui.available_width() - 2.0 * ui.spacing().button_padding.x).max(0.0),
        );
        let name_owner = ui.make_persistent_id(("definition_name", owner));
        let response = super::commands::owned_ui(ui, name_owner, |ui| {
            ui.add(egui::Button::new(name).frame(false))
        });
        if super::commands::row_activated(&response) {
            ui.memory_mut(|memory| memory.request_focus(ContextInspector::definition_focus_id()));
        }
    }
    // A disappearing name must not lend its pressed identity to the location or details.
    let content_owner = ui.make_persistent_id(("definition_content", owner));
    super::commands::owned_ui(ui, content_owner, |ui| {
        let mut action = show_location(ui, theme, context);
        let response = ui.small_button(if expanded {
            "Hide details"
        } else {
            "Show details"
        });
        if super::commands::row_activated(&response) {
            ui.memory_mut(|memory| memory.request_focus(ContextInspector::definition_focus_id()));
            action = Some(InspectorAction::ExpandDefinition(!expanded));
        }
        if expanded {
            show_details(ui, theme, context, preview);
        }
        action
    })
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
        let response = ui.add(egui::Link::new(label)).on_hover_text(format!(
            "Go to {}:{line}. Back returns to the caret.",
            super::text_fit::bounded(&destination.path)
        ));
        if super::commands::row_activated(&response) {
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
                        Label::new(
                            super::source_preview::snippet_jobs(std::slice::from_ref(line), theme)
                                .next()
                                .expect("one local definition line"),
                        )
                        .truncate(true)
                        .selectable(true),
                    );
                });
            }
        }
    } else if context.definition_count == 0
        && !context.is_on_definition
        && matches!(preview, PreviewPhase::None)
    {
        super::text_fit::explanation(
            ui,
            "No definition indexed in this repository.",
            theme.text_secondary,
        );
    } else {
        super::source_preview::show(ui, theme, preview);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::test_support::indexed_under_caret_context;

    #[test]
    fn definition_controls_retire_presses_when_headings_or_destinations_change() {
        use crate::ui::test_support::{click, painted_frame, text_center};
        let mut context = indexed_under_caret_context();
        let ctx = egui::Context::default();
        let frame = |context: &UnderCaretContext, repeated, events| {
            let mut action = None;
            let output = painted_frame(&ctx, egui::vec2(300.0, 300.0), events, |ui| {
                action = show(
                    ui,
                    &AppTheme::dark(),
                    context,
                    &PreviewPhase::None,
                    false,
                    repeated,
                );
            });
            (output, action)
        };
        frame(&context, false, vec![]);
        let output = frame(&context, false, vec![]).0;
        let position = text_center(&output, &context.name);
        let mut press = click(position);
        press.pop();
        assert_eq!(frame(&context, false, press).1, None);
        assert_eq!(
            frame(&context, true, vec![click(position).pop().unwrap()]).1,
            None
        );
        let output = frame(&context, false, vec![]).0;
        let position = text_center(&output, "source.rs:1");
        let mut press = click(position);
        press.pop();
        assert_eq!(frame(&context, false, press).1, None);
        context.destination.as_mut().unwrap().path = "moved.rs".into();
        let (output, action) = frame(&context, false, vec![click(position).pop().unwrap()]);
        assert_eq!(
            action, None,
            "a refreshed destination must retire the old press"
        );
        assert_eq!(
            frame(&context, false, click(text_center(&output, "moved.rs:1"))).1,
            Some(InspectorAction::JumpToIndexedLocation {
                path: "moved.rs".into(),
                range: context.destination.as_ref().unwrap().range.clone()
            })
        );
    }

    #[test]
    fn compact_states_and_expanded_index_facts_are_distinct() {
        let mut context = indexed_under_caret_context();
        let render = |context: &UnderCaretContext, expanded, repeated| {
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
                        repeated,
                    );
                },
            )
            .join("\n")
        };
        let compact = render(&context, false, false);
        assert!(compact.contains("source.rs:1"));
        assert!(!compact.contains("Useful documentation"));
        let expanded = render(&context, true, false);
        assert!(expanded.contains("Function · 1 indexed reference"));
        assert!(expanded.contains("fn target()") && expanded.contains("Useful documentation"));
        let repeated = render(&context, true, true);
        assert!(!repeated.lines().any(|line| line == context.name));
        assert!(repeated.contains("fn target()") && repeated.contains("Useful documentation"));
        context.is_on_definition = true;
        context.line = 1;
        context.destination = None;
        assert!(render(&context, false, false).contains("source.rs:1 · Here"));
        context.is_on_definition = false;
        context.definition_count = 0;
        context.external_package = Some("dependency 2.3".into());
        assert!(render(&context, false, false).contains("dependency 2.3"));
        assert!(render(&context, true, false).contains("No definition indexed in this repository"));
        context.external_package = None;
        assert!(render(&context, true, false).contains("No definition indexed in this repository"));
        context.is_on_definition = true;
        assert!(!render(&context, true, false).contains("No definition indexed"));
        context.is_on_definition = false;
        context.id = crate::semantic::index::SemanticSymbolID::new("local 0", "source.rs");
        context.local_definition_line = Some(crate::source::preview::PreviewLine {
            number: 1,
            text: "\t\tlet café\t= 1;".into(),
            emphasis: None,
            clipped: false,
        });
        let local = render(&context, true, false);
        assert!(!local.contains("No definition indexed"));
        assert!(local.lines().any(|line| line == "let café    = 1;"));
        assert_eq!(
            context.local_definition_line.as_ref().unwrap().text,
            "\t\tlet café\t= 1;"
        );
    }
    #[test]
    fn long_indexed_details_keep_pane_width_and_full_navigation_target() {
        let mut context = indexed_under_caret_context();
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
                                action =
                                    show(ui, &theme, &context, &PreviewPhase::None, true, false);
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
