//! Query-scoped file search with immutable inputs and retired worker publication.

use super::{
    picker::{self, PickerInput, PickerRow},
    ModalAction,
};
use crate::{
    background::LatestRead,
    repository::{
        file_node::RepositoryFile,
        quick_search::{QuickFileSearch, QuickFileSearchResponse},
    },
    theme::AppTheme,
    ui::{commands, icons::Icon, text_fit},
};
use egui::{Context, Key};
use std::{
    sync::{atomic::Ordering, Arc},
    time::Duration,
};

/// Modal dialog for fuzzy file navigation (Ctrl/Cmd+K).
///
/// Replace the shared inventory when repository contents change. Clones keep the
/// query and selection but start independent searches and interaction lifetimes.
pub struct QuickFilePickerModal {
    /// Current search query.
    pub query: String,
    /// Highlighted index within the bounded results.
    pub selected_idx: usize,
    owner: uuid::Uuid,
    requested: Option<(Arc<[RepositoryFile]>, Arc<str>)>,
    worker: LatestRead<Option<QuickFileSearchResponse>>,
    results: Option<QuickFileSearchResponse>,
    failed: bool,
    input: PickerInput,
    reveal: bool,
    focused: bool,
}

impl Default for QuickFilePickerModal {
    fn default() -> Self {
        Self {
            query: String::new(),
            selected_idx: 0,
            owner: uuid::Uuid::new_v4(),
            requested: None,
            worker: LatestRead::default(),
            results: None,
            failed: false,
            input: PickerInput::default(),
            reveal: true,
            focused: false,
        }
    }
}

impl Clone for QuickFilePickerModal {
    fn clone(&self) -> Self {
        Self {
            query: self.query.clone(),
            selected_idx: self.selected_idx,
            ..Self::default()
        }
    }
}

impl std::fmt::Debug for QuickFilePickerModal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuickFilePickerModal")
            .field("query", &self.query)
            .field("selected_idx", &self.selected_idx)
            .finish_non_exhaustive()
    }
}

impl QuickFilePickerModal {
    /// Renders the picker over a shared immutable repository inventory.
    /// Ranking runs on the existing bounded read pool, never on the UI thread.
    /// Hosts capture raw input before the frame through [`super::ModalInput`].
    pub fn show(
        &mut self,
        ctx: &Context,
        theme: &AppTheme,
        files: &Arc<[RepositoryFile]>,
    ) -> Option<ModalAction> {
        let key = self.input.begin(ctx, &mut self.worker);
        if key == Some(Key::Escape) {
            return Some(ModalAction::Close);
        }

        let mut action = None;
        picker::window("Go to File", self.owner).show(ctx, |ui| {
            let query = picker::query(
                ui,
                &mut self.focused,
                &mut self.query,
                Icon::Search,
                "Type a filename or fuzzy path…",
                theme.text_muted,
            );
            self.search(ctx, files);
            self.input.route(ctx, key, &query, !self.failed);
            action = self.apply_navigation();
            picker::body(ui, |ui| {
                if let Some(result) = &self.results {
                    if result.files.is_empty() {
                        ui.label(if self.query.trim().is_empty() {
                            "No Files"
                        } else {
                            "No Matches"
                        });
                        ui.label(if self.query.trim().is_empty() {
                            "This repository contains no files."
                        } else {
                            "Try fewer characters or a different order."
                        });
                    } else {
                        if let Some(clicked) = self.rows(ui, theme) {
                            action = Some(clicked);
                        }
                    }
                } else {
                    ui.label(if self.failed {
                        "File search failed. Change the query to retry."
                    } else {
                        "Searching…"
                    });
                }
            });
            if let Some(button) = picker::footer(
                ui,
                theme,
                &self.footer(),
                "Open",
                self.selected_file().map(ModalAction::OpenFile),
                query.id,
            ) {
                action = Some(button);
            }
        });
        self.input.finish(ctx, &mut self.worker, action)
    }

    fn search(&mut self, ctx: &Context, files: &Arc<[RepositoryFile]>) {
        let changed = self.requested.as_ref().is_none_or(|(inventory, query)| {
            !Arc::ptr_eq(inventory, files) || query.as_ref() != self.query
        });
        if changed {
            if self.requested.is_some() {
                self.selected_idx = 0;
            }
            self.input.navigation.clear();
            self.results = None;
            self.failed = false;
            self.reveal = true;
            let inventory = files.clone();
            let query: Arc<str> = self.query.as_str().into();
            self.requested = Some((inventory.clone(), query.clone()));
            let context = ctx.clone();
            self.worker.spawn(move |cancel| {
                let result = QuickFileSearch::search_cancellable(
                    &inventory,
                    &query,
                    QuickFileSearch::VISIBLE_LIMIT,
                    || cancel.load(Ordering::Relaxed),
                );
                context.request_repaint();
                result
            });
        }
        match self.worker.poll() {
            Ok(Some(Some(result))) => {
                self.results = Some(result);
            }
            Ok(Some(None)) | Err(_) => {
                self.failed = true;
                self.input.navigation.clear();
            }
            Ok(None) => {}
        }
        if self.worker.is_running() {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }

    fn apply_navigation(&mut self) -> Option<ModalAction> {
        let count = self.results.as_ref()?.files.len();
        self.selected_idx = self.selected_idx.min(count.saturating_sub(1));
        while let Some(key) = self.input.navigation.pop_front() {
            if key == Key::Enter {
                return self.selected_file().map(ModalAction::OpenFile);
            }
            if let Some(index) = commands::list_selection(key, Some(self.selected_idx), count) {
                self.selected_idx = index;
                self.reveal = true;
            }
        }
        None
    }

    fn selected_file(&self) -> Option<String> {
        self.results
            .as_ref()?
            .files
            .get(self.selected_idx)
            .map(|file| file.path.clone())
    }

    fn footer(&self) -> String {
        let Some(result) = &self.results else {
            return if self.failed {
                "Search unavailable"
            } else {
                "Searching…"
            }
            .into();
        };
        if result.is_limited() {
            format!(
                "Showing {} of {} matches · Type more to refine",
                result.files.len(),
                result.total_count
            )
        } else {
            format!(
                "{} {}",
                result.total_count,
                if result.total_count == 1 {
                    "match"
                } else {
                    "matches"
                }
            )
        }
    }

    fn rows(&mut self, ui: &mut egui::Ui, theme: &AppTheme) -> Option<ModalAction> {
        let result = self.results.as_ref()?;
        let height = PickerRow::height(ui);
        let id = egui::Id::new("file_results");
        let scroll = picker::scroll(ui, id, self.selected_idx, &mut self.reveal);
        let mut action = None;
        scroll.show_rows(ui, height, result.files.len(), |ui, rows| {
            for index in rows {
                let file = &result.files[index];
                let owner = (
                    self.owner,
                    &self.query,
                    Arc::as_ptr(&self.requested.as_ref().unwrap().0) as *const () as usize,
                    &file.path,
                );
                let owner = ui.make_persistent_id(owner);
                commands::owned_ui(ui, owner, |ui| {
                    let response = file_row(ui, theme, file, index == self.selected_idx, height);
                    if commands::row_activated(&response) {
                        action = Some(ModalAction::OpenFile(file.path.clone()));
                    }
                });
            }
        });
        action
    }
}

fn file_row(
    ui: &mut egui::Ui,
    theme: &AppTheme,
    file: &RepositoryFile,
    selected: bool,
    height: f32,
) -> egui::Response {
    let (directory, name) = file
        .path
        .rsplit_once('/')
        .unwrap_or((&file.path, &file.path));
    PickerRow {
        title: name,
        detail: directory,
        metadata: "",
    }
    .show(ui, theme, selected, height)
    .on_hover_text(text_fit::bounded(&file.path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::{
        commands::test_key,
        test_support::{text_center, text_shapes},
    };
    use egui::Event;

    fn files(count: usize) -> Arc<[RepositoryFile]> {
        (0..count)
            .map(|n| RepositoryFile::new(format!("src/file{n:04}.rs")))
            .collect()
    }

    fn frame(
        ctx: &Context,
        picker: &mut QuickFilePickerModal,
        files: &Arc<[RepositoryFile]>,
        events: Vec<Event>,
    ) -> (egui::FullOutput, Option<ModalAction>) {
        crate::ui::test_support::hosted_modal_frame(
            ctx,
            egui::Id::new(picker.owner).with("input"),
            egui::vec2(850.0, 650.0),
            events,
            |ctx| picker.show(ctx, &AppTheme::dark(), files),
        )
    }

    fn settled(
        ctx: &Context,
        picker: &mut QuickFilePickerModal,
        files: &Arc<[RepositoryFile]>,
    ) -> (egui::FullOutput, Option<ModalAction>) {
        crate::ui::test_support::settled_modal(|| {
            let result = frame(ctx, picker, files, vec![]);
            (
                result.0,
                result.1,
                !picker.worker.is_running() && picker.input.events.is_empty(),
            )
        })
    }

    #[test]
    fn narrow_query_and_return_opens_first_fresh_match_from_old_selection() {
        let ctx = Context::default();
        let files = files(240);
        let mut picker = QuickFilePickerModal::default();
        settled(&ctx, &mut picker, &files);
        picker.selected_idx = 99;
        let (_, immediate) = frame(
            &ctx,
            &mut picker,
            &files,
            vec![Event::Text("file0007.rs".into()), test_key(Key::Enter)],
        );
        let action = immediate.or_else(|| settled(&ctx, &mut picker, &files).1);
        assert_eq!(
            action,
            Some(ModalAction::OpenFile("src/file0007.rs".into()))
        );
        assert_eq!(picker.selected_idx, 0);
    }

    #[test]
    fn repeated_keys_reveal_beyond_twelve_and_enter_discards_its_text_suffix() {
        let ctx = Context::default();
        let files = files(240);
        let mut picker = QuickFilePickerModal::default();
        let (output, _) = settled(&ctx, &mut picker, &files);
        assert!(
            text_shapes(&output)
                .any(|text| text.galley.job.text
                    == "Showing 100 of 240 matches · Type more to refine")
        );
        assert!(
            text_shapes(&output)
                .filter(|text| text.galley.job.text.starts_with("file"))
                .count()
                < 20,
            "list must virtualize"
        );
        frame(
            &ctx,
            &mut picker,
            &files,
            (0..28).map(|_| test_key(Key::ArrowDown)).collect(),
        );
        let (output, _) = settled(&ctx, &mut picker, &files);
        assert_eq!(picker.selected_idx, 28);
        let text = text_shapes(&output)
            .find(|text| text.galley.job.text == "file0028.rs")
            .expect("selection must be painted");
        let clip = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(painted) if painted.galley.job.text == "file0028.rs" => {
                    Some(shape.clip_rect)
                }
                _ => None,
            })
            .unwrap();
        let row = egui::Rect::from_min_size(
            text.pos,
            egui::vec2(text.galley.size().x, text.galley.size().y + 14.0),
        );
        assert!(
            clip.contains_rect(row),
            "selected filename and directory must be visible: {row:?} in {clip:?}"
        );
        let (_, action) = frame(
            &ctx,
            &mut picker,
            &files,
            vec![test_key(Key::Enter), Event::Text("unrelated".into())],
        );
        assert_eq!(
            action,
            Some(ModalAction::OpenFile("src/file0028.rs".into()))
        );
        assert_eq!(picker.query, "");
        assert!(picker.input.events.is_empty());
    }

    #[test]
    fn mixed_text_navigation_and_return_preserve_order() {
        let ctx = Context::default();
        let files = files(240);
        let mut picker = QuickFilePickerModal::default();
        settled(&ctx, &mut picker, &files);
        frame(
            &ctx,
            &mut picker,
            &files,
            vec![
                Event::Text("file00".into()),
                test_key(Key::ArrowDown),
                test_key(Key::ArrowDown),
                test_key(Key::Enter),
                Event::Text("suffix".into()),
            ],
        );
        let (_, action) = settled(&ctx, &mut picker, &files);
        assert_eq!(
            action,
            Some(ModalAction::OpenFile("src/file0002.rs".into()))
        );
        assert_eq!(picker.query, "file00");
    }

    #[test]
    fn query_inventory_and_cancel_retire_inflight_publication() {
        for boundary in ["query", "inventory", "cancel"] {
            let ctx = Context::default();
            let original = files(240);
            let mut picker = QuickFilePickerModal::default();
            settled(&ctx, &mut picker, &original);
            let (release, wait) = std::sync::mpsc::channel();
            let (observed, observation) = std::sync::mpsc::channel();
            let (started, start) = std::sync::mpsc::channel();
            picker.worker.spawn(move |cancel| {
                started.send(()).unwrap();
                wait.recv().unwrap();
                observed.send(cancel.load(Ordering::Relaxed)).unwrap();
                Some(QuickFileSearch::search(&files(1), "", 100))
            });
            start.recv_timeout(Duration::from_secs(5)).unwrap();
            let mut accepted = original.clone();
            match boundary {
                "query" => {
                    picker.query = "file0009".into();
                }
                "inventory" => {
                    accepted = vec![RepositoryFile::new("other/accepted.rs")].into();
                }
                _ => {}
            }
            let events = if boundary == "cancel" {
                vec![test_key(Key::Escape)]
            } else {
                vec![]
            };
            let (_, action) = frame(&ctx, &mut picker, &accepted, events);
            release.send(()).unwrap();
            assert!(observation.recv_timeout(Duration::from_secs(5)).unwrap());
            if boundary == "cancel" {
                assert_eq!(action, Some(ModalAction::Close));
                assert!(!picker.worker.is_running());
            } else {
                settled(&ctx, &mut picker, &accepted);
                let expected = if boundary == "query" {
                    "src/file0009.rs"
                } else {
                    "other/accepted.rs"
                };
                assert_eq!(picker.selected_file().as_deref(), Some(expected));
            }
        }
    }

    #[test]
    fn failed_worker_disables_open_and_accepts_query_retry_after_return() {
        let ctx = Context::default();
        let files = files(20);
        let mut picker = QuickFilePickerModal::default();
        settled(&ctx, &mut picker, &files);
        picker.results = None;
        picker.worker.spawn(|_| None);
        let (output, _) = settled(&ctx, &mut picker, &files);
        assert!(text_shapes(&output).any(|text| text.galley.job.text == "Search unavailable"));
        assert!(picker.selected_file().is_none());
        frame(
            &ctx,
            &mut picker,
            &files,
            vec![test_key(Key::Enter), Event::Text("file0003".into())],
        );
        settled(&ctx, &mut picker, &files);
        assert_eq!(picker.selected_file().as_deref(), Some("src/file0003.rs"));
    }

    #[test]
    fn empty_no_matches_and_bounded_filename_directory_rows_at_both_scales() {
        for scale in [1.0, 2.0] {
            let ctx = Context::default();
            ctx.set_pixels_per_point(scale);
            let mut picker = QuickFilePickerModal::default();
            let (output, _) = settled(&ctx, &mut picker, &files(0));
            assert!(text_shapes(&output).any(|text| text.galley.job.text == "No Files"));
            assert_eq!(picker.selected_file(), None);
            let long_path = format!(
                "src/{}/{}.rs",
                "directoryé".repeat(90),
                "名verylong".repeat(90)
            );
            let inventory: Arc<[RepositoryFile]> = vec![RepositoryFile::new(&long_path)].into();
            let (output, _) = settled(&ctx, &mut picker, &inventory);
            for text in text_shapes(&output).filter(|text| text.galley.job.text.contains('…')) {
                assert!(text.galley.size().x <= 610.0);
            }
            assert!(
                text_shapes(&output)
                    .filter(|text| text.galley.job.text.contains('…'))
                    .count()
                    >= 2
            );
            assert_eq!(picker.selected_file().as_deref(), Some(long_path.as_str()));
            picker.query = "not-present".into();
            let (output, _) = settled(&ctx, &mut picker, &inventory);
            assert!(text_shapes(&output).any(|text| text.galley.job.text == "No Matches"));
            assert_eq!(picker.footer(), "0 matches");
            assert_eq!(picker.selected_file(), None);
        }
    }

    #[test]
    fn pointer_release_cannot_retarget_replaced_results_but_fresh_click_opens() {
        let ctx = Context::default();
        let files = files(20);
        let mut picker = QuickFilePickerModal::default();
        let (output, _) = settled(&ctx, &mut picker, &files);
        let position = text_center(&output, "file0000.rs");
        let pointer = |pressed| Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        frame(
            &ctx,
            &mut picker,
            &files,
            vec![Event::PointerMoved(position), pointer(true)],
        );
        picker.query = "file0001".into();
        settled(&ctx, &mut picker, &files);
        assert_eq!(
            frame(&ctx, &mut picker, &files, vec![pointer(false)]).1,
            None
        );
        let (_, action) = frame(
            &ctx,
            &mut picker,
            &files,
            crate::ui::test_support::click(position),
        );
        assert_eq!(
            action,
            Some(ModalAction::OpenFile("src/file0001.rs".into()))
        );
    }
    #[test]
    fn selection_clamps_at_result_edges_and_clears_when_inventory_becomes_empty() {
        for scale in [1.0, 2.0] {
            let ctx = Context::default();
            ctx.set_pixels_per_point(scale);
            let files = files(3);
            let mut picker = QuickFilePickerModal::default();
            settled(&ctx, &mut picker, &files);
            assert_eq!(picker.selected_file().as_deref(), Some("src/file0000.rs"));
            frame(
                &ctx,
                &mut picker,
                &files,
                (0..10).map(|_| test_key(Key::ArrowDown)).collect(),
            );
            settled(&ctx, &mut picker, &files);
            assert_eq!(picker.selected_file().as_deref(), Some("src/file0002.rs"));
            frame(
                &ctx,
                &mut picker,
                &files,
                (0..10).map(|_| test_key(Key::ArrowUp)).collect(),
            );
            settled(&ctx, &mut picker, &files);
            assert_eq!(picker.selected_file().as_deref(), Some("src/file0000.rs"));
            let empty = Arc::from([]);
            settled(&ctx, &mut picker, &empty);
            assert!(picker.selected_file().is_none());
            assert_eq!(picker.selected_idx, 0);
            assert!(frame(&ctx, &mut picker, &empty, vec![test_key(Key::Enter)])
                .1
                .is_none());
        }
    }
}
