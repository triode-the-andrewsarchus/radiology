//! Variable-height, source-trace step rows with independent keyboard selection.
use crate::{code_paths::CodePath, theme::AppTheme};
use egui::{Id, Key, Ui};
use uuid::Uuid;

#[derive(Default)]
pub(crate) struct CodePathList {
    trace: Option<Uuid>,
    selected: Option<usize>,
    reveal: bool,
    rows: reader_text::RowHeights,
}

impl CodePathList {
    pub(crate) const KEYS: &[Key] = &[
        Key::ArrowUp,
        Key::ArrowDown,
        Key::Home,
        Key::End,
        Key::Enter,
        Key::Escape,
    ];
    pub(crate) fn focus_id() -> Id {
        Id::new("code_path_steps_focus")
    }
    pub(crate) fn has_focus(ctx: &egui::Context) -> bool {
        ctx.memory(|memory| memory.has_focus(Self::focus_id()))
    }

    pub(crate) fn key(&mut self, key: Key, trace: &CodePath) -> Option<Uuid> {
        if let Some(row) = super::commands::list_selection(key, self.selected, trace.steps.len()) {
            self.selected = Some(row);
            self.reveal = true;
        }
        match key {
            Key::Enter => self
                .selected
                .and_then(|row| trace.steps.get(row))
                .map(|step| step.id),
            Key::Escape => {
                self.selected = None;
                None
            }
            _ => None,
        }
    }

    pub(crate) fn show(
        &mut self,
        ui: &mut Ui,
        theme: &AppTheme,
        trace: &CodePath,
        opened: Option<Uuid>,
    ) -> Option<Uuid> {
        if self.trace != Some(trace.id) {
            *self = Self {
                trace: Some(trace.id),
                ..Default::default()
            };
        }
        let region = ui.available_rect_before_wrap();
        ui.interact(
            region,
            Self::focus_id(),
            egui::Sense::focusable_noninteractive(),
        );
        super::commands::retain_navigation_keys(ui.ctx(), Self::focus_id());
        let width = (region.width() - ui.spacing().scroll.allocated_width()).max(1.0);
        let estimate = 3.0 * ui.text_style_height(&egui::TextStyle::Body) + 20.0;
        self.rows.reset(
            trace.steps.len(),
            width,
            estimate,
            ui.ctx().pixels_per_point(),
        );
        let mut scroll = egui::ScrollArea::vertical()
            .max_height(region.height())
            .id_source(("code_path_steps", trace.id))
            .auto_shrink([false, false]);
        if std::mem::take(&mut self.reveal) {
            if let Some(row) = self.selected {
                scroll = scroll.vertical_scroll_offset(self.rows.offset(row));
            }
        }
        let mut action = None;
        scroll.show_viewport(ui, |ui, viewport| {
            let start = self.rows.row_at(viewport.min.y);
            let end = (self.rows.row_at(viewport.max.y) + 2).min(trace.steps.len());
            let origin = ui.min_rect().min;
            for row in start..end {
                let step = &trace.steps[row];
                let x = origin.x + 28.0;
                let text_width = (width - 34.0).max(1.0);
                let name = super::text_fit::lines(
                    ui,
                    &step.function,
                    egui::TextStyle::Monospace.resolve(ui.style()),
                    theme.text_primary,
                    text_width,
                    1,
                );
                // Descriptions remain complete and wrap to the pane width.
                let description = ui.fonts(|fonts| {
                    fonts.layout(
                        step.description.clone(),
                        egui::TextStyle::Body.resolve(ui.style()),
                        theme.text_primary,
                        text_width,
                    )
                });
                let path = super::text_fit::middle(
                    ui,
                    &step.path,
                    egui::TextStyle::Small.resolve(ui.style()),
                    theme.text_secondary,
                    text_width,
                );
                let height = name.size().y + description.size().y + path.size().y + 18.0;
                self.rows.set_height(row, height);
                let rect = egui::Rect::from_min_size(
                    origin + egui::vec2(0.0, self.rows.offset(row)),
                    egui::vec2(width, height),
                );
                let response = ui.interact(rect, ui.id().with(step.id), egui::Sense::click());
                if self.selected == Some(row)
                    || (self.selected.is_none() && opened == Some(step.id))
                {
                    ui.painter()
                        .rect_filled(rect.shrink(1.0), 3.0, ui.visuals().selection.bg_fill);
                } else if response.hovered() {
                    ui.painter().rect_filled(
                        rect.shrink(1.0),
                        3.0,
                        ui.visuals().widgets.hovered.bg_fill,
                    );
                }
                let y = rect.top() + 5.0;
                ui.painter().text(
                    egui::pos2(origin.x + 4.0, y),
                    egui::Align2::LEFT_TOP,
                    (row + 1).to_string(),
                    egui::TextStyle::Small.resolve(ui.style()),
                    theme.text_secondary,
                );
                ui.painter()
                    .galley(egui::pos2(x, y), name.clone(), theme.text_primary);
                ui.painter().galley(
                    egui::pos2(x, y + name.size().y + 3.0),
                    description.clone(),
                    theme.text_primary,
                );
                ui.painter().galley(
                    egui::pos2(x, y + name.size().y + description.size().y + 6.0),
                    path,
                    theme.text_secondary,
                );
                if response.clicked() {
                    self.selected = Some(row);
                    action = Some(step.id);
                    ui.memory_mut(|memory| memory.request_focus(Self::focus_id()));
                }
                super::commands::retain_navigation_keys(ui.ctx(), response.id);
                if let Some(step) = super::context_menu::show(&response, trace.id, || {
                    vec![super::context_menu::Item::action("Open Source", step.id)]
                }) {
                    action = Some(step);
                }
                response.on_hover_text(format!(
                    "{}\n{}\n{}",
                    step.function, step.description, step.path
                ));
            }
            ui.allocate_space(egui::vec2(width, self.rows.total_height()));
        });
        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn long_descriptions_wrap_and_large_traces_only_paint_visible_steps() {
        let mut text = "Many steps\n".to_string();
        for n in 0..10_000 {
            text.push_str(&format!(
                "a.rs:boot_{n}:Step {n} has a complete description that wraps in a narrow pane.\n"
            ));
        }
        let trace = CodePath::parse(&text).unwrap();
        for scale in [1.0, 2.0] {
            let ctx = egui::Context::default();
            ctx.set_pixels_per_point(scale);
            let mut list = CodePathList::default();
            let frame = |list: &mut CodePathList| {
                ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(180.0, 500.0),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            list.show(ui, &AppTheme::dark(), &trace, None);
                        });
                    },
                )
            };
            let _ = frame(&mut list); // Apply the pending pixels-per-point viewport change.
            let output = frame(&mut list);
            let labels: Vec<_> = crate::ui::test_support::text_shapes(&output).collect();
            assert!(
                labels.len() < 100,
                "painted {} labels at scale {scale}",
                labels.len()
            );
            assert!(labels.iter().any(|shape| shape
                .galley
                .job
                .text
                .starts_with("Step 0 has a complete")
                && shape.galley.rows.len() > 1));
            list.key(Key::End, &trace);
            let output = frame(&mut list);
            assert!(
                crate::ui::test_support::text_shapes(&output)
                    .any(|shape| shape.galley.job.text == "boot_9999")
            );
        }
    }
}
