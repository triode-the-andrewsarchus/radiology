//! Reader commands independent of where an action originated.

use super::*;

impl RadiologyApp {
    pub(super) fn execute_command(&mut self, ctx: &Context, command: ToolbarAction) {
        use ToolbarAction::*;
        match command {
            ReloadSource => {
                if let Some(buffer) = &self.open_buffer {
                    let location = buffer
                        .location_at_byte(buffer.byte_offset(self.caret_line, self.caret_col));
                    let target = crate::source::loader::SourceTarget::Byte(location.byte_offset);
                    self.request_navigation(
                        location,
                        target,
                        source::NavigationIntent {
                            force_read: true,
                            reveal: true,
                            ..Default::default()
                        },
                    );
                }
            }
            GenerateIndex => self.generate_index(),
            CancelIndexGeneration => self.index_generator.stop(),
            CheckIndexTools => self.check_index_tools(),
            CancelIndexToolCheck => self.tool_probe_job.stop(),
            ReloadIndexImport => self.load_repository_index(),
            CancelIndexImport => {
                self.index_importer.stop();
                if self.index_cache_status()
                    == &crate::semantic::indexing::service::CacheLoadStatus::Loading
                {
                    self.index_generator.stop();
                }
            }
            ToggleSidebar => self.show_sidebar = !self.show_sidebar,
            ToggleInspector => self.show_inspector = !self.show_inspector,
            OpenRecentRepo(path) => self.open_repository(Path::new(&path)),
            OpenRepositoryDialog => {
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    self.open_repository(&folder);
                }
            }
            CopyRepoPath => {
                if let Some(root) = &self.repository_root {
                    ctx.output_mut(|output| output.copied_text = root.display().to_string());
                }
            }
            NavigateBack => self.navigate_history(false),
            NavigateForward => self.navigate_history(true),
            NextChangeBlock => self.navigate_change(true),
            PrevChangeBlock => self.navigate_change(false),
            GoToDefinition => self.go_to_definition(),
            FindReferences => self.find_references_at_caret(),
            OpenBookmark(index) => self.open_bookmark(index),
            ToggleBookmark => self.toggle_current_bookmark(),
            CollapseFunctions => self.fold_store.collapse_all_functions(&self.fold_ranges),
            FoldToDefinitions => self.fold_store.fold_to_definitions(&self.fold_ranges),
            OpenQuickFilePicker => {
                self.modal = ActiveModal::QuickFilePicker(QuickFilePickerModal::default())
            }
            OpenGoToLine => self.modal = ActiveModal::GoToLine(GoToLineModal::default()),
            OpenGoToSymbol => self.modal = ActiveModal::GoToSymbol(GoToSymbolModal::default()),
            OpenPRDialog => self.show_pull_request_prompt(),
            OpenPullRequest(number) => self.open_pull_request(number),
            RefreshPRList => self.recent_pull_requests.refresh(),
            CancelPRList => self.recent_pull_requests.cancel(),
            FindInFile if self.open_buffer.is_some() => self.source_find.open(),
            NextFindMatch if self.open_buffer.is_some() => self.source_find.request_step(true),
            PreviousFindMatch if self.open_buffer.is_some() => self.source_find.request_step(false),
            FindInFile | NextFindMatch | PreviousFindMatch => {}
            OpenSearch => {
                self.sidebar_tab = SidebarTab::Search;
                self.show_sidebar = true;
            }
            ToggleMode(mode) => self.set_source_mode(mode),
            ToggleRevision(revision) => self.change_revision(revision),
            ToggleCurrentRevision => {
                if let Some(revision) = self.active_revision {
                    self.change_revision(revision.other());
                }
            }
            ExitPRReview => {
                self.save_bookmarks();
                if let Some(root) = self.repository_root.clone() {
                    self.open_repository(&root);
                }
            }
            RefreshPR => self.refresh(),
            ToggleSourceFilesOnly => {
                self.session.source_filters.source_files_only =
                    !self.session.source_filters.source_files_only;
                self.filters_changed();
            }
            ToggleHideTests => {
                self.session.source_filters.hides_tests = !self.session.source_filters.hides_tests;
                self.filters_changed();
            }
            ShowAllFiles => {
                self.session.source_filters = Default::default();
                self.filters_changed();
            }
            ToggleWrapLines => {
                self.session.wrap_lines = !self.session.wrap_lines;
                self.session.save();
            }
            IncreaseSourceFont | DecreaseSourceFont | ResetSourceFont => {
                self.session.source_font_size = match command {
                    IncreaseSourceFont => (self.session.source_font_size + 1.0).min(32.0),
                    DecreaseSourceFont => (self.session.source_font_size - 1.0).max(8.0),
                    _ => 14.0,
                };
                self.session.save();
            }
            ToggleTheme => self.toggle_theme(ctx),
        }
    }

    fn resolve_occurrences(
        &self,
        byte: usize,
    ) -> Option<crate::source::selection::OccurrenceSelection> {
        use crate::source::selection::OccurrenceSelection;
        let buffer = self.open_buffer.as_ref()?;
        let syntactic = self.syntactic_occurrences.selection(buffer, byte);
        if let Some(index) = self
            .semantic_index
            .as_ref()
            .filter(|_| buffer.revision != Some(ReviewRevision::Before))
        {
            if let Some(occurrence) = index.occurrence_at_byte(buffer, byte) {
                if let Some(document) = index.documents.get(&buffer.path) {
                    let ranges: Vec<_> = document
                        .occurrences
                        .iter()
                        .filter(|candidate| candidate.symbol == occurrence.symbol)
                        .filter_map(|candidate| {
                            candidate
                                .range
                                .as_ref()
                                .and_then(|range| index.byte_range(buffer, range))
                        })
                        .collect();
                    if syntactic.as_ref().is_none_or(|selection| {
                        selection.ranges.iter().all(|range| ranges.contains(range))
                    }) {
                        let active = occurrence
                            .range
                            .as_ref()
                            .and_then(|range| index.byte_range(buffer, range))?;
                        return Some(OccurrenceSelection {
                            symbol_name: occurrence.symbol.clone(),
                            label: buffer.text.get(active.clone())?.to_owned(),
                            active,
                            ranges,
                        });
                    }
                }
            }
        }
        syntactic.map(|selection| {
            let word = &buffer.text[selection.active.clone()];
            OccurrenceSelection {
                symbol_name: format!(
                    "syntactic:{}:{}:{word}",
                    selection.scope.start, selection.scope.end
                ),
                label: word.to_owned(),
                active: selection.active,
                ranges: selection.ranges,
            }
        })
    }

    pub(super) fn select_occurrences_at_byte(&mut self, byte: usize) {
        self.pin_store.ephemeral = self.resolve_occurrences(byte);
    }

    pub(super) fn pin_at_byte(&mut self, byte: usize) {
        if let Some(group) = self.resolve_occurrences(byte) {
            self.pin_store
                .toggle_labeled_pin(&group.symbol_name, &group.label, group.ranges, byte);
        }
        self.pin_store.ephemeral = None;
    }

    pub(super) fn filters_changed(&mut self) {
        if let Some(buffer) = &self.open_buffer {
            self.test_code
                .set_enabled(buffer, self.session.source_filters.hides_tests);
        }
        self.refresh_file_tree();
        self.session.save();
    }

    fn toggle_theme(&mut self, ctx: &Context) {
        let dark = !self.theme.is_dark;
        self.theme = if dark {
            AppTheme::dark()
        } else {
            AppTheme::light()
        };
        self.session.dark_mode = dark;
        self.session.save();
        ctx.set_visuals(if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });
        self.queue_rehighlight();
    }

    fn navigate_history(&mut self, forward: bool) {
        let mut history = self.history.clone();
        let location = if forward {
            history.forward()
        } else {
            history.back()
        };
        if let Some(location) = location {
            let byte = location.byte_offset;
            self.request_navigation(
                location,
                crate::source::loader::SourceTarget::Byte(byte),
                source::NavigationIntent {
                    reveal: true,
                    history_step: Some(forward),
                    ..Default::default()
                },
            );
        }
    }

    pub(super) fn change_navigation(&self) -> crate::pull_request::diff::ChangeNavigation {
        self.open_buffer
            .as_ref()
            .zip(self.line_changes.as_ref())
            .map_or_else(Default::default, |(buffer, changes)| {
                changes.navigation_at(
                    self.caret_line,
                    buffer.revision.unwrap_or(ReviewRevision::After),
                )
            })
    }

    fn navigate_change(&mut self, next: bool) {
        let targets = self.change_navigation();
        let line = if next { targets.next } else { targets.previous };
        if let (Some(line), Some(buffer)) = (line, &self.open_buffer) {
            self.navigate_target(
                buffer.location_at_byte(buffer.byte_offset(line, 0)),
                None,
                source::NavigationIntent {
                    record_history: true,
                    reveal: true,
                    flash_destination: true,
                    ..Default::default()
                },
            );
        }
    }

    fn go_to_definition(&mut self) {
        self.refresh_under_caret_context();
        if let Some(context) = &self.under_caret {
            if let Some(destination) = context.destination.clone() {
                self.navigate_to_indexed_range(&destination.path, &destination.range);
            }
            return;
        }
        let (Some(index), Some(buffer)) = (&self.semantic_index, &self.open_buffer) else {
            return;
        };
        if buffer.revision == Some(ReviewRevision::Before) {
            return;
        }
        let byte = buffer.byte_offset(self.caret_line, self.caret_col);
        let Some(occurrence) = index.occurrence_at_byte(buffer, byte) else {
            return;
        };
        let id = SemanticSymbolID::new(&occurrence.symbol, &buffer.path);
        self.navigate_to_indexed_symbol(&id);
    }

    pub(super) fn navigate_to_indexed_symbol(&mut self, id: &SemanticSymbolID) {
        let Some((path, range)) = self
            .semantic_index
            .as_ref()
            .and_then(|index| index.find_definition_for_id(id))
            .map(|(path, range)| (path.to_owned(), range.clone()))
        else {
            return;
        };
        self.navigate_to_indexed_range(&path, &range);
    }

    pub(super) fn navigate_to_indexed_range(
        &mut self,
        path: &str,
        range: &crate::semantic::ScipRange,
    ) {
        let revision = self
            .open_buffer
            .as_ref()
            .and_then(|buffer| buffer.revision)
            .or(self.active_revision);
        if revision == Some(ReviewRevision::Before) {
            return;
        }
        let Some(index) = &self.semantic_index else {
            return;
        };
        let (Some(line), Ok(column), Some(encoding)) = (
            usize::try_from(range.start_line)
                .ok()
                .and_then(|line| line.checked_add(1)),
            usize::try_from(range.start_col),
            index.column_encoding(path),
        ) else {
            return;
        };
        let content_digest = index.source_digests.get(path).cloned();
        if !index.source_digests.is_empty() && content_digest.is_none() {
            self.error = Some("Index has no source identity for this destination".into());
            return;
        }
        let location = SourceLocation::with_revision(path, 0, revision);
        self.request_navigation(
            location,
            crate::source::loader::SourceTarget::Encoded {
                line,
                column,
                encoding,
                content_digest,
            },
            source::NavigationIntent {
                record_history: true,
                flash_destination: true,
                reveal: true,
                ..Default::default()
            },
        );
    }

    fn change_revision(&mut self, revision: ReviewRevision) {
        let Some(buffer) = &self.open_buffer else {
            return;
        };
        if self.active_pr.is_none() {
            return;
        }
        let location = SourceLocation::with_revision(&buffer.path, 0, Some(revision));
        if self
            .review
            .as_ref()
            .is_none_or(|review| review.resolve(location.clone()).is_err())
        {
            return;
        }
        let line = self
            .line_changes
            .as_ref()
            .map_or(self.caret_line, |changes| {
                changes.map_line(
                    self.caret_line,
                    buffer.revision.unwrap_or(ReviewRevision::After),
                )
            });
        let target = crate::source::loader::SourceTarget::LineColumn {
            line,
            column: self.caret_col,
        };
        self.request_navigation(
            location,
            target,
            source::NavigationIntent {
                record_history: true,
                reveal: true,
                ..Default::default()
            },
        );
    }

    fn refresh(&mut self) {
        if let Some(pr) = &self.active_pr {
            self.open_pull_request(pr.number);
        } else if let Some(root) = self.repository_root.clone() {
            self.request_repository(repository::RepositoryRequest {
                path: root,
                destination: repository::RepositoryDestination::RestoreCurrent,
            });
        }
    }
}
