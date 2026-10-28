//! Local-find controls; search execution and storage remain GUI-independent.

use crate::source::{find::SourceFind, SourceBuffer};
use egui::{Key, Modifiers, Ui};
use std::{ops::Range, sync::Arc};

/// View state for the source find strip and its active exact-byte match.
#[derive(Default)]
pub struct SourceFindPanel {
    /// Current query, completion, and complete result store.
    pub search: SourceFind,
    /// Active match emphasized in the source viewport.
    pub active_range: Option<Range<usize>>,
    visible: bool,
    request_focus: bool,
    restart: bool,
    pending_step: Option<bool>,
    document: Option<(String, Option<crate::source::ReviewRevision>, u64)>,
}

impl SourceFindPanel {
    /// Opens the strip and focuses its query without changing the current document.
    pub fn open(&mut self) {
        self.visible = true;
        self.request_focus = true;
        self.restart = true;
    }

    /// Requests next/previous match navigation through the shared command route.
    pub fn request_step(&mut self, forward: bool) {
        self.visible = true;
        self.pending_step = Some(forward);
    }

    /// Renders local find and returns an exact match to reveal in the source.
    pub fn show(&mut self, ui: &mut Ui, source: &Arc<SourceBuffer>) -> Option<Range<usize>> {
        self.sync_document(source);
        let completed = self.search.poll();
        let mut step = self.pending_step.take().or(completed.then_some(true));
        if self.visible {
            let previous = self.search.options.clone();
            self.controls(ui, &mut step);
            if previous != self.search.options {
                self.active_range = None;
                self.search.start(source.clone());
                step = None;
            }
            if let Some(error) = &self.search.error {
                ui.colored_label(ui.visuals().error_fg_color, error);
            }
            ui.separator();
        }
        if !ui.is_enabled() {
            self.pending_step = step;
            return None;
        }
        if self.search.busy {
            self.pending_step = step;
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(25));
        }
        let range = step
            .filter(|_| self.visible)
            .and_then(|forward| self.search.step(forward));
        if let Some(range) = &range {
            self.active_range = Some(range.clone());
            ui.ctx().request_repaint();
        }
        range
    }

    fn controls(&mut self, ui: &mut Ui, step: &mut Option<bool>) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Find");
            let query = ui.add(
                egui::TextEdit::singleline(&mut self.search.options.query)
                    .id_source("source_find_query")
                    .hint_text("Find in file")
                    .desired_width(220.0),
            );
            if self.request_focus && ui.is_enabled() {
                query.request_focus();
                self.request_focus = false;
            }
            if ui.is_enabled() && (query.has_focus() || query.lost_focus()) {
                if ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Enter)) {
                    *step = Some(true);
                    query.request_focus();
                }
                if ui.input_mut(|input| input.consume_key(Modifiers::SHIFT, Key::Enter)) {
                    *step = Some(false);
                    query.request_focus();
                }
                if ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape)) {
                    self.visible = false;
                    query.surrender_focus();
                }
            }
            ui.checkbox(&mut self.search.options.case_sensitive, "Aa")
                .on_hover_text("Match case");
            ui.checkbox(&mut self.search.options.regex, ".*")
                .on_hover_text("Regular expression");
            let ready = self.search.count() > 0 && !self.search.busy;
            if ui
                .add_enabled(ready, egui::Button::new("Previous"))
                .clicked()
            {
                *step = Some(false);
            }
            if ui.add_enabled(ready, egui::Button::new("Next")).clicked() {
                *step = Some(true);
            }
            if self.search.busy {
                ui.spinner();
            } else {
                ui.label(format!(
                    "{} / {}",
                    self.search.active_number(),
                    self.search.count()
                ));
            }
            if ui.small_button("Close").clicked() {
                self.visible = false;
                query.surrender_focus();
            }
        });
        if !self.visible {
            self.search.stop();
        }
    }

    fn sync_document(&mut self, source: &Arc<SourceBuffer>) {
        let key = (
            source.path.clone(),
            source.revision,
            source.content_identity(),
        );
        if self.document.as_ref() != Some(&key) || self.restart {
            self.search.stop();
            if self.visible {
                self.search.start(source.clone());
            }
            self.restart = false;
            self.active_range = None;
            self.document = Some(key);
        }
    }

    /// Reads only the matches overlapping the displayed original-source range.
    pub fn matches(&mut self, fragment: Range<usize>) -> Vec<(Range<usize>, bool)> {
        if !self.visible {
            return Vec::new();
        }
        self.search
            .in_fragment(fragment)
            .into_iter()
            .map(|(_, range)| {
                let active = self.active_range.as_ref() == Some(&range);
                (range, active)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn frame(panel: &mut SourceFindPanel, source: &Arc<SourceBuffer>) -> Option<Range<usize>> {
        let ctx = egui::Context::default();
        let mut found = None;
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                found = panel.show(ui, source);
            });
        });
        found
    }

    fn finish(panel: &mut SourceFindPanel, source: &Arc<SourceBuffer>) -> Option<Range<usize>> {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let found = frame(panel, source);
            if !panel.search.busy {
                return found;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn find_navigation_restarts_for_changed_content_at_the_same_path() {
        let mut panel = SourceFindPanel::default();
        panel.search.options.query = "é".into();
        let first = Arc::new(SourceBuffer::from_str("same.rs", "😀 é é\r\n"));
        panel.open();
        assert_eq!(finish(&mut panel, &first), Some(5..7));
        assert_eq!(panel.search.count(), 2);
        panel.request_step(false);
        assert_eq!(frame(&mut panel, &first), Some(8..10));
        let revised = Arc::new(SourceBuffer::from_str("same.rs", "é"));
        assert_eq!(finish(&mut panel, &revised), Some(0..2));
        assert_eq!(panel.search.count(), 1);
        panel.visible = false;
        let third = Arc::new(SourceBuffer::from_str("same.rs", "no matches"));
        assert_eq!(frame(&mut panel, &third), None);
        assert!(!panel.search.busy);
        panel.open();
        assert_eq!(finish(&mut panel, &third), None);
        assert_eq!(panel.search.count(), 0);
    }
    #[test]
    fn repeated_enter_stays_in_find_instead_of_following_source_definition() {
        let ctx = egui::Context::default();
        let source = Arc::new(SourceBuffer::from_str("find.rs", "hit hit"));
        let mut panel = SourceFindPanel::default();
        panel.search.options.query = "hit".into();
        panel.open();
        let mut render = |events| {
            let mut range = None;
            let _ = ctx.run(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ctx| {
                    assert!(crate::ui::commands::take_shortcuts(ctx, false).is_empty());
                    egui::CentralPanel::default().show(ctx, |ui| {
                        range = panel.show(ui, &source);
                    });
                },
            );
            range
        };
        let deadline = Instant::now() + Duration::from_secs(5);
        while render(Vec::new()).is_none() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        let focused = ctx.memory(|memory| memory.focused());
        assert!(focused.is_some());
        for expected in [4..7, 0..3] {
            assert_eq!(
                render(vec![egui::Event::Key {
                    key: Key::Enter,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                }]),
                Some(expected)
            );
            assert_eq!(ctx.memory(|memory| memory.focused()), focused);
        }
    }
}
