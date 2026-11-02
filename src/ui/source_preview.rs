//! Shared bounded snippet presentation for definition and reference disclosures.

use crate::{
    source::preview::{PreviewLine, PreviewPhase},
    theme::AppTheme,
};
use egui::{text::LayoutJob, FontId, Label, RichText, TextFormat, Ui};

pub(super) fn height(phase: &PreviewPhase) -> f32 {
    match phase {
        PreviewPhase::None => 0.0,
        PreviewPhase::Loaded(preview) => {
            8.0 + 18.0 * (preview.lines.len() + usize::from(preview.truncated)) as f32
        }
        _ => 40.0,
    }
}

pub(super) fn show(ui: &mut Ui, theme: &AppTheme, phase: &PreviewPhase) {
    let extent = height(phase);
    if extent == 0.0 {
        return;
    }
    // Caller reveal and virtual row estimates use this same extent. Content must
    // not grow it when diagnostics wrap or font/line-number geometry changes.
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), extent),
        egui::Sense::hover(),
    );
    let mut child = ui.child_ui(rect, egui::Layout::top_down(egui::Align::Min));
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    child.spacing_mut().item_spacing.y = 0.0;
    match phase {
        PreviewPhase::None => {}
        PreviewPhase::Loading => {
            child.horizontal(|ui| {
                ui.spinner();
                ui.colored_label(theme.text_muted, "Loading source…");
            });
        }
        PreviewPhase::Unavailable(error) => {
            child.label(RichText::new("Source unavailable").color(theme.text_muted));
            let reason = super::text_fit::middle(
                &child,
                error,
                egui::TextStyle::Small.resolve(child.style()),
                theme.text_secondary,
                child.available_width(),
            );
            child.add(Label::new(reason)).on_hover_text(error);
        }
        PreviewPhase::Loaded(preview) => {
            ui.painter().rect_filled(rect, 0.0, theme.bg_base);
            let mut snippet =
                child.child_ui(rect.shrink(4.0), egui::Layout::top_down(egui::Align::Min));
            for line in &preview.lines {
                snippet.allocate_ui_with_layout(
                    egui::vec2(snippet.available_width(), 18.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.label(
                            RichText::new(format!("{:>3}", line.number))
                                .font(FontId::monospace(12.0))
                                .color(theme.text_muted),
                        );
                        let response = ui.add(
                            Label::new(line_job(line, theme))
                                .truncate(true)
                                .selectable(true),
                        );
                        if line.clipped {
                            response.on_hover_text(
                                "This long line is cropped around the indexed occurrence.",
                            );
                        }
                    },
                );
            }
            if preview.truncated {
                snippet.allocate_ui_with_layout(
                    egui::vec2(snippet.available_width(), 18.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.colored_label(theme.text_muted, "… More source follows");
                    },
                );
            }
        }
    }
}

pub(super) fn line_job(line: &PreviewLine, theme: &AppTheme) -> LayoutJob {
    let normal = TextFormat {
        font_id: FontId::monospace(12.0),
        color: theme.text_primary,
        ..Default::default()
    };
    let mut job = LayoutJob::default();
    if let Some(range) = &line.emphasis {
        job.append(&line.text[..range.start], 0.0, normal.clone());
        job.append(
            &line.text[range.clone()],
            0.0,
            TextFormat {
                color: theme.accent,
                ..normal.clone()
            },
        );
        job.append(&line.text[range.end..], 0.0, normal);
    } else {
        job.append(&line.text, 0.0, normal);
    }
    job
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::preview::SourcePreview;
    use crate::ui::test_support::{painted_frame, themed_context};

    #[test]
    fn preview_extent_matches_caller_reveal_geometry_for_every_phase() {
        let loaded = PreviewPhase::Loaded(SourcePreview {
            path: "source.rs".into(),
            revision: None,
            content_digest: "exact source".into(),
            lines: vec![
                PreviewLine {
                    number: 12345,
                    text: "    fn café() {".into(),
                    emphasis: Some(7..12),
                    clipped: false,
                },
                PreviewLine {
                    number: 12346,
                    text: String::new(),
                    emphasis: None,
                    clipped: false,
                },
                PreviewLine {
                    number: 12347,
                    text: "        漢字🦀();".into(),
                    emphasis: None,
                    clipped: true,
                },
                PreviewLine {
                    number: 12348,
                    text: "    }".into(),
                    emphasis: None,
                    clipped: false,
                },
            ],
            truncated: true,
        });
        for dark in [true, false] {
            for scale in [1.0, 1.5, 2.0] {
                for width in [216.0, 300.0] {
                    let (ctx, theme) = themed_context(dark, scale);
                    let render = |phase: &PreviewPhase| {
                        let mut extent = 0.0;
                        let output = painted_frame(&ctx, egui::vec2(width, 300.0), vec![], |ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            let start = ui.cursor().top();
                            show(ui, &theme, phase);
                            extent = ui.cursor().top() - start;
                            ui.label("Callers");
                        });
                        (extent, output)
                    };
                    render(&PreviewPhase::None);
                    for phase in [
                        PreviewPhase::None,
                        PreviewPhase::Loading,
                        PreviewPhase::Unavailable(
                            "Source unavailable at a deeply nested path.\n".repeat(200),
                        ),
                        loaded.clone(),
                    ] {
                        let (extent, output) = render(&phase);
                        assert!(
                            (extent - height(&phase)).abs() <= 0.5,
                            "{scale}x {width} {phase:?}: rendered {extent}, predicted {}",
                            height(&phase)
                        );
                        for shape in &output.shapes {
                            if let egui::Shape::Text(text) = &shape.shape {
                                if text.galley.job.text != "Callers" {
                                    assert!(shape.clip_rect.right() <= width - 8.0 + 0.5);
                                }
                                assert!(text.galley.job.text.len() <= 4096);
                            }
                        }
                    }
                }
            }
        }
    }
}
