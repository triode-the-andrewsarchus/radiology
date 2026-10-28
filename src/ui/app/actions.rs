//! Reader commands independent of where an action originated.

use super::*;

impl RadiologyApp {
    pub(super) fn execute_command(&mut self, ctx: &Context, command: ToolbarAction) {
        use ToolbarAction::*;
        match command {
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
            OpenBookmark(index) => self.open_bookmark(index),
            ToggleBookmark => {
                if let Some(buffer) = &self.open_buffer {
                    self.bookmarks
                        .toggle(buffer, buffer.byte_offset(self.caret_line, self.caret_col));
                    self.save_bookmarks();
                }
            }
            CollapseFunctions => self
                .fold_store
                .collapse_all_functions(&self.outline_symbols),
            FoldToDefinitions => {
                self.fold_store
                    .folded_lines
                    .extend(self.fold_ranges.iter().map(|fold| fold.start_line));
            }
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
            ToggleMode(mode) => self.mode = mode,
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
            ToggleTheme => self.toggle_theme(ctx),
        }
    }

    fn filters_changed(&mut self) {
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
        if let Some(buffer) = self.open_buffer.clone() {
            self.rehighlight(&buffer);
        }
    }

    fn navigate_history(&mut self, forward: bool) {
        self.record_departure();
        let previous = self.history.clone();
        let location = if forward {
            self.history.forward()
        } else {
            self.history.back()
        };
        if let Some(location) = location {
            self.navigate(location.clone(), false);
            if self.open_buffer.as_ref().is_none_or(|buffer| {
                buffer.path != location.path || buffer.revision != location.revision
            }) {
                self.history = previous;
            }
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
        let Some((path, range)) = index.find_definition_for_id(&id) else {
            return;
        };
        let Some(root) = &self.repository_root else {
            return;
        };
        let target = self.read_source(root, path, buffer.revision);
        match target {
            Ok(target) => {
                if let Some(byte) = index.byte_range(&target, range).map(|range| range.start) {
                    self.navigate(target.location_at_byte(byte), true);
                }
            }
            Err(error) => self.error = Some(format!("Definition: {error}")),
        }
    }

    fn change_revision(&mut self, revision: ReviewRevision) {
        let Some(buffer) = &self.open_buffer else {
            return;
        };
        if self.active_pr.is_none() {
            return;
        }
        let requested = SourceLocation::with_revision(&buffer.path, 0, Some(revision));
        let result = self.resolve_location(requested).and_then(|location| {
            let root = self
                .repository_root
                .as_deref()
                .ok_or("No repository is open")?;
            let target = self.read_source(root, &location.path, location.revision)?;
            let line = self
                .line_changes
                .as_ref()
                .map_or(self.caret_line, |changes| {
                    changes.map_line(
                        self.caret_line,
                        buffer.revision.unwrap_or(ReviewRevision::After),
                    )
                });
            Ok(target.location_at_byte(target.byte_offset(line, self.caret_col)))
        });
        match result {
            Ok(location) => self.navigate(location, true),
            Err(error) => self.error = Some(error),
        }
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
