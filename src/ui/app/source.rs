//! Atomic source publication and navigation transactions.
use super::*;
use crate::source::loader::{PreparedSource, SourceLoadRequest, SourceRead, SourceTarget};

#[derive(Clone, Copy, Default)]
pub(super) struct NavigationIntent {
    pub record_history: bool,
    pub reveal: bool,
    pub history_step: Option<bool>,
    pub force_read: bool,
}

#[derive(Clone)]
pub(super) struct PendingNavigation {
    request: SourceLoadRequest,
    intent: NavigationIntent,
}

impl RadiologyApp {
    /// Changes source presentation and schedules call projection only when needed.
    pub fn set_source_mode(&mut self, mode: SourceMode) {
        if self.mode != mode {
            self.mode = mode;
            self.refresh_critical_projection();
        }
    }

    pub(super) fn refresh_critical_projection(&mut self) {
        self.refresh_critical_annotations();
        if self.mode == SourceMode::Critical {
            if let Some(buffer) = &self.open_buffer {
                let index = (buffer.revision != Some(ReviewRevision::Before))
                    .then(|| self.semantic_index.clone())
                    .flatten();
                self.critical_lens.start(
                    buffer.clone(),
                    self.call_sites.clone(),
                    index,
                    self.critical_marks.overrides().clone(),
                );
            }
        } else {
            self.critical_lens.stop();
        }
    }

    fn refresh_critical_annotations(&mut self) {
        self.critical_annotations.stop();
        if self.mode != SourceMode::Critical {
            return;
        }
        let (Some(buffer), Some(index), Some(root)) = (
            &self.open_buffer,
            &self.semantic_index,
            &self.repository_root,
        ) else {
            return;
        };
        if buffer.revision == Some(ReviewRevision::Before) {
            return;
        }
        let source = self.review.as_ref().map_or_else(
            || SourceRead::Checkout(root.clone()),
            |review| SourceRead::Review(review.clone()),
        );
        self.critical_annotations.start(
            crate::critical::worker::GraphInputs {
                source,
                buffer: buffer.clone(),
                calls: self.call_sites.clone(),
                index: index.clone(),
                overrides: self.critical_marks.overrides().clone(),
            },
            self.fold_ranges.clone(),
        );
    }

    /// Returns callable counts only for the current exact source/revision.
    #[must_use]
    pub fn critical_annotations(&self) -> Option<&crate::critical::annotations::SourceAnnotations> {
        self.open_buffer
            .as_ref()
            .and_then(|buffer| self.critical_annotations.current(buffer))
    }

    /// Returns source-local preparation or syntax coverage for the active lens.
    #[must_use]
    pub fn critical_status(&self) -> Option<&'static str> {
        if self.mode != SourceMode::Critical {
            return None;
        }
        Some(if self.critical_lens.is_running() {
            "Preparing Critical Lens…"
        } else if self.critical_lens.error().is_some() {
            "Critical Lens unavailable"
        } else if self.critical_annotations.is_running() {
            "Preparing critical counts…"
        } else if self.critical_annotations.error().is_some() {
            "Critical counts unavailable"
        } else if !self.call_sites.supported {
            "Call syntax unavailable"
        } else {
            "Call-focused"
        })
    }

    /// Returns whether call-focused source preparation is awaiting publication.
    #[must_use]
    pub fn is_critical_loading(&self) -> bool {
        self.critical_lens.is_running() || self.critical_annotations.is_running()
    }

    /// Returns the current exact-source Critical Lens presentation, when ready.
    #[must_use]
    pub fn critical_projection(&self) -> Option<&crate::critical::SourceProjection> {
        self.open_buffer
            .as_ref()
            .and_then(|buffer| self.critical_lens.projection(buffer))
    }

    pub(super) fn request_navigation(
        &mut self,
        location: SourceLocation,
        target: SourceTarget,
        intent: NavigationIntent,
    ) {
        let location = match self.resolve_location(location) {
            Ok(location) => location,
            Err(error) => {
                self.error = Some(error);
                return;
            }
        };
        if !intent.force_read {
            if let Some(buffer) = self.open_buffer.as_ref().filter(|buffer| {
                buffer.path == location.path
                    && buffer.revision == location.revision
                    && target.accepts_source_identity(buffer)
            }) {
                let result = target.resolve(buffer);
                if self.pending_navigation.is_some() {
                    self.cancel_source_load();
                }
                match result {
                    Ok(location) => {
                        self.commit_departure(intent);
                        self.apply_destination(intent, location);
                    }
                    Err(error) => self.error = Some(error),
                }
                return;
            }
        }
        let Some(root) = &self.repository_root else {
            return;
        };
        let source = self.review.as_ref().map_or_else(
            || SourceRead::Checkout(root.clone()),
            |review| SourceRead::Review(review.clone()),
        );
        let request = SourceLoadRequest {
            source,
            location,
            target,
        };
        self.source_loader.start(
            request.clone(),
            self.highlighter.clone(),
            self.theme.is_dark,
        );
        self.pending_navigation = Some(PendingNavigation { request, intent });
        self.error = None;
    }

    /// Cancels pending source work while preserving the current document/history.
    pub fn cancel_source_load(&mut self) {
        self.source_loader.stop();
        self.pending_navigation = None;
    }

    /// Returns whether source loading or appearance work is still pending.
    #[must_use]
    pub fn is_source_loading(&self) -> bool {
        self.source_loader.is_running()
    }

    /// Publishes ready source work and schedules repaint while a request is pending.
    ///
    /// Embedded hosts can call this without constructing an eframe window.
    pub fn poll_source(&mut self, ctx: &Context) {
        if self.critical_marks.poll() {
            self.refresh_critical_projection();
        }
        if self.critical_marks.is_busy() {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
        }
        self.critical_annotations.poll();
        if let Some(error) = self.critical_lens.poll() {
            self.error = Some(error);
        }
        if self.is_critical_loading() {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
        }
        if let Some(result) = self.source_loader.poll() {
            match result {
                Ok(prepared) if prepared.dark_mode != self.theme.is_dark => {
                    self.queue_rehighlight()
                }
                Ok(prepared) => self.publish_source(prepared),
                Err(error) => {
                    let path = self
                        .pending_navigation
                        .take()
                        .map(|pending| pending.request.location.path)
                        .unwrap_or_else(|| "Source appearance".into());
                    self.error = Some(format!("{path}: {error}"));
                }
            }
            ctx.request_repaint();
        }
        if self.source_loader.is_running() {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
        }
    }

    fn publish_source(&mut self, prepared: PreparedSource) {
        let Some(pending) = self.pending_navigation.take() else {
            if self
                .open_buffer
                .as_ref()
                .is_some_and(|buffer| Arc::ptr_eq(buffer, &prepared.buffer))
            {
                self.highlighted_lines = prepared.highlights;
            }
            return;
        };
        let Some(syntax) = prepared.syntax else {
            return;
        };
        self.commit_departure(pending.intent);
        self.selected_path = Some(prepared.buffer.path.clone());
        self.active_revision = prepared.buffer.revision;
        self.call_sites = Arc::new(syntax.calls);
        self.outline_symbols = syntax.symbols;
        self.fold_ranges = syntax.folds;
        self.fold_store.unfold_all();
        self.pin_store.activate(&prepared.buffer);
        self.test_code.load(
            &prepared.buffer,
            syntax.tests,
            self.session.source_filters.hides_tests,
        );
        self.highlighted_lines = prepared.highlights;
        self.line_changes = prepared.changes;
        self.open_buffer = Some(prepared.buffer);
        self.refresh_critical_projection();
        self.error = None;
        self.apply_destination(pending.intent, prepared.destination);
    }

    fn commit_departure(&mut self, intent: NavigationIntent) {
        if intent.record_history || intent.history_step.is_some() {
            self.record_departure();
        }
        if let Some(forward) = intent.history_step {
            if forward {
                self.history.forward();
            } else {
                self.history.back();
            }
        }
    }

    fn apply_destination(&mut self, intent: NavigationIntent, location: SourceLocation) {
        let Some(buffer) = &self.open_buffer else {
            return;
        };
        let byte = location.byte_offset;
        if intent.reveal {
            self.test_code.reveal_byte(buffer, byte);
        }
        self.caret_line = buffer.line_at_byte(byte);
        self.caret_col = buffer.column_at_byte(byte);
        self.scroll_to_line = Some(self.caret_line);
        self.fold_store
            .reveal_line(self.caret_line, &self.fold_ranges);
        if intent.record_history {
            self.history.visit(location);
        }
        self.refresh_symbol_context();
    }

    pub(super) fn queue_rehighlight(&mut self) {
        self.highlighted_lines.clear();
        if let Some(pending) = &self.pending_navigation {
            self.source_loader.start(
                pending.request.clone(),
                self.highlighter.clone(),
                self.theme.is_dark,
            );
        } else if let Some(buffer) = &self.open_buffer {
            self.source_loader.start(
                SourceLoadRequest {
                    source: SourceRead::Buffer(buffer.clone()),
                    location: buffer.location_at_byte(0),
                    target: SourceTarget::Byte(0),
                },
                self.highlighter.clone(),
                self.theme.is_dark,
            );
        }
    }

    pub(super) fn show_source_loading(&mut self, ctx: &Context) {
        if !self.source_loader.is_running() {
            return;
        }
        TopBottomPanel::top("source_loading").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(self.pending_navigation.as_ref().map_or_else(
                    || "Updating source colors…".into(),
                    |pending| format!("Opening {}…", pending.request.location.path),
                ));
                if ui.small_button("Cancel").clicked() {
                    self.cancel_source_load();
                }
            });
        });
    }
}
