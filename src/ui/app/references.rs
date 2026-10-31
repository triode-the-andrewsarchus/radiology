//! Binding reference focus to exact parsed declarations and immutable indexes.

use super::*;

impl RadiologyApp {
    /// Updates enclosing-declaration references without rebuilding on each caret move.
    pub fn refresh_symbol_context(&mut self) {
        self.sync_outline();
        self.refresh_under_caret_context();
        let automatic = self.semantic_inputs.as_ref().and_then(|inputs| {
            let byte = inputs.buffer.byte_offset(self.caret_line, self.caret_col);
            let mut enclosing = Vec::new();
            for symbol in &self.outline_symbols {
                symbol.breadcrumbs(byte, &mut enclosing);
            }
            ReferenceTarget::declaration(&inputs.index, &inputs.buffer, enclosing.last()?)
        });
        self.reference_context
            .update(self.semantic_inputs.clone(), automatic);
    }

    pub(super) fn find_references_at_caret(&mut self) {
        if let Some(buffer) = &self.open_buffer {
            self.find_references_at_byte(buffer.byte_offset(self.caret_line, self.caret_col));
        }
    }

    pub(super) fn find_references_at_byte(&mut self, byte: usize) {
        let target = self
            .open_buffer
            .as_ref()
            .zip(self.semantic_index.as_ref())
            .and_then(|(buffer, index)| ReferenceTarget::at_byte(index, buffer, byte));
        let Some(target) = target else {
            self.error = Some("No exact indexed symbol at this source position.".into());
            return;
        };
        self.refresh_symbol_context();
        self.reference_context.show(target);
        self.show_inspector = true;
        self.error = None;
    }

    pub(super) fn follow_reference_caret(&mut self) {
        self.refresh_symbol_context();
        self.reference_context.follow_caret();
    }

    /// Returns whether the current reference context awaits background publication.
    #[must_use]
    pub fn is_reference_loading(&self) -> bool {
        self.reference_context.is_running()
            || self.definition_preview.is_running()
            || self.under_caret_job.is_running()
    }
}

impl RadiologyApp {
    pub(super) fn reset_previews(&mut self) {
        self.semantic_inputs = None;
        self.definition_preview = Default::default();
        self.definition_identity = None;
        self.under_caret = None;
        self.under_caret_job.stop();
        self.under_caret_error = None;
        self.definition_expanded = false;
        self.reference_viewport = Default::default();
        self.outline_list = Default::default();
    }

    fn refresh_semantic_inputs(&mut self) -> bool {
        use crate::source::preview::SourceQueryInputs;
        let configuration = self
            .open_buffer
            .as_ref()
            .zip(self.semantic_index.as_ref())
            .filter(|(buffer, _)| buffer.revision != Some(ReviewRevision::Before));
        if let Some((buffer, index)) = configuration {
            let same = self.semantic_inputs.as_ref().is_some_and(|inputs| {
                inputs.buffer.position_key() == buffer.position_key()
                    && inputs.buffer.content_digest() == buffer.content_digest()
                    && Arc::ptr_eq(&inputs.index, index)
            });
            if same {
                return false;
            }
            let source = self
                .review
                .as_ref()
                .map(|review| crate::source::loader::SourceRead::Review(review.clone()))
                .or_else(|| {
                    self.repository_root
                        .as_ref()
                        .map(|root| crate::source::loader::SourceRead::Checkout(root.clone()))
                })
                .unwrap_or_else(|| crate::source::loader::SourceRead::Buffer(buffer.clone()));
            self.semantic_inputs = Some(Arc::new(SourceQueryInputs {
                source,
                buffer: buffer.clone(),
                index: index.clone(),
            }));
        } else if self.semantic_inputs.take().is_none() {
            return false;
        }
        true
    }

    pub(super) fn refresh_under_caret_context(&mut self) {
        let changed_inputs = self.refresh_semantic_inputs();
        let byte = self.open_buffer.as_ref().map_or(0, |buffer| {
            buffer.byte_offset(self.caret_line, self.caret_col)
        });
        let identity = self.semantic_inputs.as_ref().and_then(|inputs| {
            let occurrence = inputs.index.occurrence_at_byte(&inputs.buffer, byte)?;
            if occurrence.symbol.is_empty()
                || occurrence.symbol_roles & crate::semantic::scip_proto::SCIP_ROLE_IMPORT != 0
            {
                return None;
            }
            let range = occurrence
                .range
                .as_ref()
                .filter(|range| range.start_line == range.end_line)?;
            let bytes = inputs
                .index
                .byte_range(&inputs.buffer, range)
                .filter(|bytes| !bytes.is_empty())?;
            Some((
                SemanticSymbolID::new(&occurrence.symbol, &inputs.buffer.path),
                bytes,
            ))
        });
        if !changed_inputs && self.definition_identity == identity {
            return;
        }
        self.definition_expanded = false;
        self.definition_identity = identity;
        self.under_caret = None;
        self.under_caret_error = None;
        self.under_caret_job.stop();
        self.definition_preview
            .request(self.semantic_inputs.clone(), None);
        if let Some(inputs) = self
            .semantic_inputs
            .as_ref()
            .filter(|_| self.definition_identity.is_some())
        {
            let inputs = inputs.clone();
            self.under_caret_job.spawn(move |cancel| {
                crate::semantic::context::UnderCaretContext::prepare(
                    &inputs.buffer,
                    &inputs.index,
                    byte,
                    cancel,
                )
            });
        }
    }

    pub(super) fn poll_under_caret_context(&mut self, ctx: &Context) {
        let result = match self.under_caret_job.poll() {
            Ok(None) => return,
            Ok(Some(result)) => result,
            Err(_) => Err("Symbol details stopped unexpectedly.".into()),
        };
        match result {
            Ok(context) => {
                use crate::source::preview::{PreviewKind, PreviewTarget};
                let target = context
                    .as_ref()
                    .filter(|context| !context.id.is_local())
                    .and_then(|context| context.destination.as_ref())
                    .map(|destination| PreviewTarget {
                        path: destination.path.clone(),
                        range: destination.range.clone(),
                        kind: PreviewKind::Definition,
                    });
                self.definition_preview
                    .request(self.semantic_inputs.clone(), target);
                self.under_caret = context;
            }
            Err(error) => self.under_caret_error = Some(error),
        }
        ctx.request_repaint();
    }
}

impl RadiologyApp {
    pub(super) fn handle_reference_keys(&mut self, ctx: &Context, events: &mut Vec<egui::Event>) {
        if crate::ui::commands::context_menu_is_open(ctx) {
            return;
        }
        // Apply each action before resolving the next key's owner and selection.
        crate::ui::commands::take_event_actions(events, |key, modifiers| {
            let action = self
                .reference_viewport
                .action_for_key(ctx, key, modifiers)?;
            self.handle_reference_action(action);
            Some(())
        });
    }

    pub(super) fn handle_reference_action(
        &mut self,
        action: crate::ui::reference_list::ReferenceAction,
    ) {
        use crate::ui::reference_list::ReferenceAction;
        match action {
            ReferenceAction::More => self.reference_context.more(),
            ReferenceAction::MoreCallers(row) => self.reference_context.more_callers(row),
            ReferenceAction::CommitCaller(reference, caller) => {
                self.commit_reference_caller(reference, caller)
            }
            ReferenceAction::Select(row) => self.reference_context.select(row),
            ReferenceAction::SetExpanded(row, expanded) => {
                self.reference_context.expand(row, expanded)
            }
            ReferenceAction::MoveSelection(forward) => {
                self.reference_context.move_selection(forward);
                self.reference_viewport.reveal_selection();
            }
            ReferenceAction::SetSelectionExpanded(expanded) => {
                if let Some(row) = self.reference_context.selected {
                    let callers_ready = self
                        .reference_context
                        .disclosures
                        .get(&row)
                        .and_then(|disclosure| disclosure.callers())
                        .is_some();
                    if !expanded || !callers_ready || !self.reference_viewport.focus_callers(row) {
                        self.reference_context.expand(row, expanded);
                    }
                }
            }
            ReferenceAction::ClearSelection => self.reference_context.selected = None,
            ReferenceAction::Commit(row) => {
                if let Some(row) = row {
                    self.reference_context.select(row);
                }
                if let Some(node) = row
                    .or(self.reference_context.selected)
                    .and_then(|row| self.reference_context.results.hits.get(row))
                    .cloned()
                {
                    self.navigate_to_indexed_range(&node.path, &node.range);
                }
            }
        }
    }
}

impl RadiologyApp {
    fn commit_reference_caller(&mut self, reference: usize, caller: usize) {
        let Some(tree) = self
            .reference_context
            .disclosures
            .get(&reference)
            .and_then(|disclosure| disclosure.callers())
        else {
            return;
        };
        let Some(row) = tree.rows.get(caller) else {
            return;
        };
        let Some(digest) = tree.source_identities.get(&row.location.path) else {
            return;
        };
        let (location, line, column, digest) =
            (row.location.clone(), row.line, row.column, digest.clone());
        self.request_navigation(
            location,
            crate::source::loader::SourceTarget::Encoded {
                line,
                column,
                encoding: crate::source::ColumnEncoding::Utf32,
                content_digest: Some(digest),
            },
            source::NavigationIntent {
                record_history: true,
                flash_destination: true,
                reveal: true,
                ..Default::default()
            },
        );
    }
}
