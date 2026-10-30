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
            ReloadIndexImport if self.review.is_none() => self.load_repository_index(),
            ReloadIndexImport => {}
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
            OpenPRDialog => self.modal = ActiveModal::PullRequest(PullRequestModal::default()),
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

    pub(super) fn pin_at_byte(&mut self, byte: usize) {
        let Some(buffer) = &self.open_buffer else {
            return;
        };
        if let Some(index) = self
            .semantic_index
            .as_ref()
            .filter(|_| buffer.revision != Some(ReviewRevision::Before))
        {
            if let Some(occurrence) = index.occurrence_at_byte(buffer, byte) {
                if let Some(document) = index.documents.get(&buffer.path) {
                    let ranges = document
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
                    let label = occurrence
                        .range
                        .as_ref()
                        .and_then(|range| index.byte_range(buffer, range))
                        .and_then(|range| buffer.text.get(range))
                        .unwrap_or(&occurrence.symbol);
                    self.pin_store
                        .toggle_labeled_pin(&occurrence.symbol, label, ranges, byte);
                    return;
                }
            }
        }
        let Some(character) = buffer.text.get(byte..).and_then(|text| text.chars().next()) else {
            return;
        };
        let identifier = |character: char| character.is_alphanumeric() || character == '_';
        if !identifier(character) {
            return;
        }
        let start = buffer.text[..byte]
            .char_indices()
            .rev()
            .take_while(|(_, character)| identifier(*character))
            .last()
            .map_or(byte, |(index, _)| index);
        let end = byte
            + buffer.text[byte..]
                .chars()
                .take_while(|character| identifier(*character))
                .map(char::len_utf8)
                .sum::<usize>();
        let word = &buffer.text[start..end];
        let mut enclosing = Vec::new();
        for symbol in &self.outline_symbols {
            symbol.breadcrumbs(byte, &mut enclosing);
        }
        let scope = enclosing
            .iter()
            .rev()
            .find(|symbol| {
                matches!(
                    symbol.kind,
                    crate::syntax::outline::SymbolKind::Function
                        | crate::syntax::outline::SymbolKind::Method
                )
            })
            .map_or(0..buffer.byte_len(), |symbol| symbol.range.clone());
        let Ok(pattern) = regex::Regex::new(&format!(r"\b{}\b", regex::escape(word))) else {
            return;
        };
        let ranges = pattern
            .find_iter(&buffer.text[scope.clone()])
            .map(|matched| scope.start + matched.start()..scope.start + matched.end())
            .collect();
        self.pin_store
            .toggle_labeled_pin(&format!("{word}@{}", scope.start), word, ranges, byte);
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

    fn navigate_change(&mut self, next: bool) {
        let Some(changes) = &self.line_changes else {
            return;
        };
        let revision = self.active_revision.unwrap_or(ReviewRevision::After);
        let line = if next {
            changes.next_change_line(self.caret_line, revision)
        } else {
            changes.previous_change_line(self.caret_line, revision)
        };
        if let (Some(line), Some(buffer)) = (line, &self.open_buffer) {
            self.navigate(buffer.location_at_byte(buffer.byte_offset(line, 0)), true);
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
            let location = self.open_buffer.as_ref().map(|buffer| {
                buffer.location_at_byte(buffer.byte_offset(self.caret_line, self.caret_col))
            });
            self.open_repository(&root);
            if let Some(location) = location {
                self.navigate(location, false);
            }
        }
    }
}
