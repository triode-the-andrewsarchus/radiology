//! Atomic source publication and navigation transactions.
use super::*;
use crate::source::loader::{PreparedSource, SourceLoadRequest, SourceRead, SourceTarget};

#[derive(Clone, Copy, Default)]
pub(super) struct NavigationIntent {
    pub record_history: bool,
    // Structural moves keep the reading trail; semantic moves record the exact caret.
    pub reading_departure: bool,
    pub reveal: bool,
    pub history_step: Option<bool>,
    pub force_read: bool,
    // Commit an explicit review-side choice only after navigation succeeds.
    pub remember_revision: bool,
    pub restore_position: bool,
    pub flash_destination: bool,
}

#[derive(Clone)]
pub(super) struct PendingNavigation {
    pub(super) request: SourceLoadRequest,
    pub(super) intent: NavigationIntent,
    pub(super) trait_owner: Option<super::traits::TraitNavigationOwner>,
}

impl RadiologyApp {
    /// Publishes actual caret movement even when another source action wins the frame.
    pub(super) fn finish_source_interaction(&mut self, previous: (usize, usize)) {
        let moved = self.source_viewport.take_caret_movement()
            || previous != (self.caret_line, self.caret_col);
        if moved {
            self.cancel_code_path_navigation();
        }
        if let Some(buffer) = &self.open_buffer {
            let caret = buffer.byte_offset(self.caret_line, self.caret_col);
            self.reading_positions.observe(
                buffer,
                self.source_viewport.top_byte(buffer),
                moved.then_some(caret),
            );
            if moved && self.history_tracks_position {
                self.history.update_current(buffer.location_at_byte(caret));
            }
        }
        if moved {
            self.refresh_symbol_context();
        }
    }

    pub(super) fn checkout_refresh_destination(&self) -> Option<(SourceLocation, SourceTarget)> {
        if let Some(pending) = &self.pending_navigation {
            if matches!(pending.request.source, SourceRead::Checkout(_)) {
                return Some((
                    pending.request.location.clone(),
                    pending.request.target.clone(),
                ));
            }
        }
        self.open_buffer.as_ref().map(|buffer| {
            let location =
                buffer.location_at_byte(buffer.byte_offset(self.caret_line, self.caret_col));
            let target = SourceTarget::Byte(location.byte_offset);
            (location, target)
        })
    }

    /// Changes source presentation and schedules call projection only when needed.
    pub fn set_source_mode(&mut self, mode: SourceMode) {
        if self.mode != mode {
            self.mode = mode;
            self.refresh_critical_projection();
        }
    }

    pub(super) fn refresh_critical_projection(&mut self) {
        self.refresh_critical_annotations();
        if let Some(buffer) = self
            .open_buffer
            .as_ref()
            .filter(|_| self.mode == SourceMode::Critical && self.call_sites.supported)
        {
            let index = (buffer.revision != Some(ReviewRevision::Before))
                .then(|| self.semantic_index.clone())
                .flatten();
            self.critical_lens.start(
                buffer.clone(),
                self.call_sites.clone(),
                index,
                self.critical_marks.overrides().clone(),
            );
        } else {
            self.critical_lens.stop();
        }
    }

    fn refresh_critical_annotations(&mut self) {
        self.critical_annotations.stop();
        self.critical_graph_inputs = None;
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
        let inputs = Arc::new(crate::critical::worker::GraphInputs {
            source,
            buffer: buffer.clone(),
            calls: self.call_sites.clone(),
            index: index.clone(),
            overrides: self.critical_marks.overrides().clone(),
        });
        self.critical_annotations
            .start((*inputs).clone(), self.fold_ranges.clone());
        self.critical_graph_inputs = Some(inputs);
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
    ///
    /// Files without supported call syntax have no projection.
    #[must_use]
    pub fn critical_projection(&self) -> Option<&crate::critical::SourceProjection> {
        self.open_buffer
            .as_ref()
            .and_then(|buffer| self.critical_lens.projection(buffer))
    }

    fn report_source_error(&mut self, target: Option<&SourceTarget>, message: String) {
        if matches!(target, Some(SourceTarget::ByteRange { .. })) {
            self.trait_context.navigation_error = Some(message);
            self.show_inspector = true;
        } else if let Some(SourceTarget::Bookmark(bookmark)) = target {
            if self.bookmarks.index_of(bookmark.id).is_some() {
                self.bookmark_list.navigation_failed(bookmark.id, message);
                self.show_inspector = true;
            }
        } else {
            self.error = Some(message);
        }
    }

    pub(super) fn request_navigation(
        &mut self,
        location: SourceLocation,
        target: SourceTarget,
        intent: NavigationIntent,
    ) {
        self.cancel_code_path_navigation();
        let location = match self.resolve_location(location) {
            Ok(location) => location,
            Err(error) => {
                self.report_source_error(Some(&target), error);
                return;
            }
        };
        self.source_viewport.cancel_deferred_input();
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
                    Err(error) => self.report_source_error(Some(&target), error),
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
            native_highlights: None,
        };
        self.source_loader.start(
            request.clone(),
            self.highlighter.clone(),
            self.theme.is_dark,
        );
        self.pending_navigation = Some(PendingNavigation {
            request,
            intent,
            trait_owner: None,
        });
        self.error = None;
    }

    /// Derives current caret context only while the accepted reader owns it.
    /// Preparing a destination hides transient context without discarding the
    /// accepted source, folds or history needed for cancellation and recovery.
    pub(super) fn enclosing_source_context(&self) -> Option<Vec<crate::syntax::DocumentSymbol>> {
        if !self.source_location_ready()
            || self.is_repository_opening()
            || self.is_code_path_loading()
        {
            return None;
        }
        let buffer = self.open_buffer.as_ref()?;
        let byte = buffer.byte_offset(self.caret_line, self.caret_col);
        let mut enclosing = Vec::new();
        for symbol in self.outline_symbols.iter() {
            symbol.breadcrumbs(byte, &mut enclosing);
        }
        Some(enclosing)
    }

    /// Cancels pending source work while preserving the current document/history.
    pub fn cancel_source_load(&mut self) {
        self.cancel_code_path_navigation();
        self.source_loader.stop();
        self.pending_navigation = None;
    }

    /// Returns whether source loading or appearance work is still pending.
    #[must_use]
    pub fn is_source_loading(&self) -> bool {
        self.source_loader.is_running() || self.is_code_path_loading()
    }

    /// Publishes ready source work and schedules repaint while a request is pending.
    ///
    /// Embedded hosts can call this without constructing an eframe window.
    pub fn poll_source(&mut self, ctx: &Context) {
        self.poll_code_path_storage(ctx);
        self.poll_under_caret_context(ctx);
        if self.definition_preview.poll() {
            ctx.request_repaint();
        }
        if self.definition_preview.is_running() {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
        }
        if self.reference_context.poll() {
            ctx.request_repaint();
        }
        if self.is_reference_loading() {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
        }
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
        self.retire_trait_navigation();
        if let Some(result) = self.source_loader.poll() {
            match result {
                Ok(prepared) if prepared.dark_mode != self.theme.is_dark => {
                    self.queue_rehighlight()
                }
                Ok(prepared) => self.publish_source(prepared),
                Err(error) => {
                    let pending = self.pending_navigation.take();
                    let path = pending.as_ref().map_or("Source appearance", |pending| {
                        pending.request.location.path.as_str()
                    });
                    self.report_source_error(
                        pending.as_ref().map(|pending| &pending.request.target),
                        format!("{path}: {error}"),
                    );
                }
            }
            ctx.request_repaint();
        }
        self.poll_code_path_navigation(ctx);
        if self.source_loader.is_running() {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
        }
    }

    pub(super) fn publish_source(&mut self, prepared: PreparedSource) {
        self.retire_trait_navigation();
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
        let bookmark_slot = if let SourceTarget::Bookmark(original) = &pending.request.target {
            let Some(slot) = self.bookmarks.index_of(original.id) else {
                return;
            };
            Some(slot)
        } else {
            None
        };
        self.commit_departure(pending.intent);
        self.selected_path = Some(prepared.buffer.path.clone());
        self.active_revision = prepared.buffer.revision;
        self.rust_trait_impls = syntax.rust_trait_impls;
        self.call_sites = Arc::new(syntax.calls);
        self.syntactic_occurrences = syntax.occurrences;
        self.native_highlights = syntax.native_highlights;
        self.outline_symbols = syntax.symbols.into();
        self.syntax_error = syntax.parse_error;
        self.fold_ranges = syntax.folds;
        self.fold_error = syntax.fold_error;
        self.fold_store
            .activate(&prepared.buffer, &self.fold_ranges);
        self.fold_store
            .apply_prepared_defaults(prepared.initial_folds);
        self.pin_store.activate(&prepared.buffer);
        self.test_code.load(
            &prepared.buffer,
            syntax.tests,
            self.session.source_filters.hides_tests,
        );
        self.highlighted_lines = prepared.highlights;
        self.line_changes = prepared.changes;
        self.open_buffer = Some(prepared.buffer);
        self.refresh_file_tree();
        self.refresh_critical_projection();
        self.error = None;
        let destination_byte = prepared.destination.byte_offset;
        self.apply_destination(pending.intent, prepared.destination);
        if let (Some(slot), Some(buffer)) = (bookmark_slot, &self.open_buffer) {
            let entry = &mut self.bookmarks.bookmarks[slot];
            let updated = crate::bookmarks::bookmark::SourceBookmark {
                id: entry.id,
                ..crate::bookmarks::bookmark::SourceBookmark::new(
                    buffer,
                    destination_byte,
                    entry.label.clone(),
                )
            };
            if *entry != updated {
                *entry = updated;
                self.save_bookmarks();
            }
        }
    }

    fn commit_departure(&mut self, intent: NavigationIntent) {
        if intent.record_history || intent.history_step.is_some() {
            self.record_departure(intent);
        }
        if let Some(forward) = intent.history_step {
            if forward {
                self.history.forward();
            } else {
                self.history.back();
            }
        }
    }

    fn apply_destination(&mut self, intent: NavigationIntent, mut location: SourceLocation) {
        let Some(buffer) = &self.open_buffer else {
            return;
        };
        if intent.remember_revision {
            if let (Some(review), Some(revision)) = (&self.review, location.revision) {
                if let Some(change) = review.change(&location.path) {
                    self.review_revision_choices
                        .insert(change.path.clone(), revision);
                }
            }
        }
        let (byte, restored) =
            self.reading_positions
                .activate(buffer, location.byte_offset, intent.restore_position);
        location.byte_offset = byte;
        self.files_reveal_pending = true;
        if intent.reveal || restored {
            self.test_code.reveal_byte(buffer, byte);
        }
        self.source_viewport.accept_navigation();
        if intent.flash_destination {
            self.source_viewport.flash_destination(buffer, byte);
        }
        self.caret_line = buffer.line_at_byte(byte);
        self.caret_col = buffer.column_at_byte(byte);
        self.scroll_to_line = (!restored).then_some(self.caret_line);
        if restored {
            self.source_viewport.restore_top(buffer, byte);
        }
        self.fold_store.reveal_byte(byte, buffer, &self.fold_ranges);
        if intent.record_history || intent.history_step.is_some() {
            self.history_tracks_position = intent.restore_position;
            if let Some(fold) = crate::syntax::folds::enclosing_function_fold(
                byte,
                &self.outline_symbols,
                &self.fold_ranges,
            ) {
                self.fold_store.folded_ranges.remove(&fold.byte_range);
            }
        }
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
                    native_highlights: Some(self.native_highlights.clone()),
                },
                self.highlighter.clone(),
                self.theme.is_dark,
            );
        }
    }

    pub(super) fn show_source_loading(&mut self, ctx: &Context) {
        if !self.is_source_loading() {
            return;
        }
        TopBottomPanel::top("source_loading").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(self.pending_navigation.as_ref().map_or_else(
                    || {
                        if self.is_code_path_loading() {
                            "Resolving code path step…".into()
                        } else {
                            "Updating source colors…".into()
                        }
                    },
                    |pending| format!("Opening {}…", pending.request.location.path),
                ));
                if ui.small_button("Cancel").clicked() {
                    self.cancel_source_load();
                }
            });
        });
    }
}
