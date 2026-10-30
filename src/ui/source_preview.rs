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
    match phase {
        PreviewPhase::None => {}
        PreviewPhase::Loading => {
            ui.colored_label(theme.text_muted, "Loading source…");
        }
        PreviewPhase::Unavailable(error) => {
            ui.colored_label(theme.text_muted, error);
        }
        PreviewPhase::Loaded(preview) => {
            egui::Frame::none()
                .fill(theme.bg_base)
                .inner_margin(4.0)
                .show(ui, |ui| {
                    for line in &preview.lines {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("{:>3}", line.number))
                                    .monospace()
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
                        });
                    }
                    if preview.truncated {
                        ui.colored_label(theme.text_muted, "… More source follows");
                    }
                });
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
