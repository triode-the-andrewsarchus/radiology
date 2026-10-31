//! Compact reader errors with selectable, complete diagnostics on demand.

use crate::theme::AppTheme;
use egui::{Context, Label, RichText, ScrollArea, TopBottomPanel, Window};

/// Presentation state for the reader's current recoverable error.
#[derive(Default)]
pub struct ErrorNotice {
    details_open: bool,
}

impl ErrorNotice {
    /// Shows a bounded notice and returns whether the user dismissed the error.
    /// Passing `None` also retires the details window.
    pub fn show(&mut self, ctx: &Context, theme: &AppTheme, message: Option<&str>) -> bool {
        let Some(message) = message else {
            self.details_open = false;
            return false;
        };
        let mut dismissed = false;
        TopBottomPanel::top("reader_notice").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Reserve the actions before sizing the summary, including narrow windows.
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    dismissed = ui.small_button("Dismiss").clicked();
                    if ui.small_button("Details…").clicked() {
                        self.details_open = true;
                    }
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.add(
                                Label::new(
                                    RichText::new(summary(message)).color(theme.text_primary),
                                )
                                .truncate(true),
                            );
                        },
                    );
                });
            });
        });
        if dismissed {
            self.details_open = false;
        }
        if self.details_open {
            self.show_details(ctx, message);
        }
        dismissed
    }

    fn show_details(&mut self, ctx: &Context, message: &str) {
        Window::new("Error Details")
            .open(&mut self.details_open)
            .default_width(640.0)
            .default_height(320.0)
            .max_width((ctx.screen_rect().width() - 32.0).max(160.0))
            .max_height((ctx.screen_rect().height() - 64.0).max(100.0))
            .show(ctx, |ui| {
                if ui.button("Copy Details").clicked() {
                    ui.output_mut(|output| output.copied_text = message.into());
                }
                ui.separator();
                ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add(Label::new(message).selectable(true));
                    });
            });
    }
}

fn summary(message: &str) -> String {
    let line = message
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("An operation failed.")
        .trim();
    let mut characters = line.chars();
    let mut summary: String = characters
        .by_ref()
        .take(240)
        .map(|ch| if ch.is_control() { ' ' } else { ch })
        .collect();
    if characters.next().is_some()
        || message
            .lines()
            .filter(|line| !line.trim().is_empty())
            .take(2)
            .count()
            > 1
    {
        summary.push('…');
    }
    summary
}

#[cfg(test)]
mod tests;
