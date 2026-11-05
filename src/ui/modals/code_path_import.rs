//! Multiline trace authoring; Return belongs to the editor, save owns dismissal.
use super::*;
use crate::{background::LatestRead, code_paths::MAX_IMPORT_BYTES};
use std::{io::Read, path::Path};

/// Pasted UTF-8 trace import with an independently cancellable text-file chooser.
#[derive(Default)]
pub struct CodePathImportModal {
    /// Complete authored text retained on parse or storage failure.
    pub text: String,
    focused: bool,
    imported: bool,
    composing: bool,
    error: Option<String>,
    file: LatestRead<Option<Result<String, String>>>,
}

impl CodePathImportModal {
    pub(crate) fn imported(&mut self) {
        self.imported = true;
    }

    /// Renders an import sheet; successful saves are dismissed by the host.
    /// `busy` disables competing edits/submissions while durable storage owns them.
    pub fn show(
        &mut self,
        ctx: &Context,
        theme: &AppTheme,
        busy: bool,
        storage_error: Option<&str>,
    ) -> Option<ModalAction> {
        if self.imported {
            return Some(ModalAction::Close);
        }
        match self.file.poll() {
            Ok(Some(Some(Ok(text)))) => {
                self.text = text;
                self.error = None;
            }
            Ok(Some(Some(Err(error)))) => self.error = Some(error),
            Err(_) => self.error = Some("Text-file chooser stopped unexpectedly".into()),
            _ => {}
        }
        if self.file.is_running() {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
        }
        ctx.input(|input| {
            for event in &input.events {
                match event {
                    egui::Event::CompositionStart | egui::Event::CompositionUpdate(_) => {
                        self.composing = true
                    }
                    egui::Event::CompositionEnd(_) => self.composing = false,
                    _ => {}
                }
            }
        });
        let mut action = None;
        egui::Window::new("Import Code Path").collapsible(false).resizable(true)
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 90.0))
            .default_size(egui::vec2(620.0, 440.0)).min_width(300.0)
            .show(ctx, |ui| {
                ctx.move_to_top(ui.layer_id());
                ui.label("First line: a title. Then one file:function:description per line.");
                let edit_id = ui.make_persistent_id("code_path_text");
                if !self.focused { ui.memory_mut(|memory| memory.request_focus(edit_id)); }
                let enabled = !busy && !self.file.is_running();
                ui.add_enabled_ui(enabled, |ui| {
                    egui::ScrollArea::vertical().id_source("code_path_import_text").max_height((ui.available_height()-120.0).max(120.0)).show(ui, |ui| {
                        let response = ui.add(egui::TextEdit::multiline(&mut self.text).id(edit_id).font(egui::TextStyle::Monospace).desired_rows(12).desired_width(f32::INFINITY).lock_focus(false));
                        self.focused |= response.enabled() && response.has_focus();
                    });
                });
                ui.label(egui::RichText::new("Use repository-relative paths and declaration names. Escape name colons as \\:; descriptions may contain colons. UTF-8, up to 2 MiB.").small().color(theme.text_secondary));
                if let Some(error) = self.error.as_deref().or(storage_error) {
                    crate::ui::text_fit::explanation(ui, error, ui.visuals().error_fg_color);
                }
                if busy { ui.horizontal(|ui| { ui.spinner(); ui.label("Saving code path…"); }); }
                action = self.buttons(ui, enabled, busy);
                if !busy && !self.composing && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, Key::Escape)) {
                    action = Some(ModalAction::Close);
                }
            });
        action
    }
    fn buttons(&mut self, ui: &mut egui::Ui, enabled: bool, busy: bool) -> Option<ModalAction> {
        let mut action = None;
        let edit_id = ui.make_persistent_id("code_path_text");
        ui.horizontal(|ui| {
            let choose = ui.add_enabled(enabled, egui::Button::new("Choose Text File…"));
            if choose.clicked() {
                self.file.spawn(|_| {
                    rfd::FileDialog::new()
                        .add_filter("UTF-8 text", &["txt", "md"])
                        .pick_file()
                        .map(|path| read_text_file(&path))
                });
            }
            let (cancel, import) = ui
                .with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let import = ui.add_enabled(
                        enabled && !self.text.trim().is_empty(),
                        egui::Button::new("Import"),
                    );
                    if import.clicked() {
                        self.error = None;
                        action = Some(ModalAction::ImportCodePath(self.text.clone()));
                    }
                    let cancel = ui.add_enabled(!busy, egui::Button::new("Cancel"));
                    if cancel.clicked() {
                        action = Some(ModalAction::Close);
                    }
                    (cancel, import)
                })
                .inner;
            let stops: Vec<_> = [
                Some(edit_id),
                enabled.then_some(choose.id),
                Some(cancel.id),
                (enabled && !self.text.trim().is_empty()).then_some(import.id),
            ]
            .into_iter()
            .flatten()
            .collect();
            picker::focus_ring(ui, &stops);
        });
        action
    }
}

fn read_text_file(path: &Path) -> Result<String, String> {
    let path = path.canonicalize().map_err(|error| error.to_string())?;
    let file =
        crate::repository::identity::open_regular_file(&path).map_err(|error| error.to_string())?;
    if file.metadata().map_err(|error| error.to_string())?.len() > MAX_IMPORT_BYTES as u64 {
        return Err("Code paths must be no larger than 2 MiB".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_IMPORT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > MAX_IMPORT_BYTES {
        return Err("Code paths must be no larger than 2 MiB".into());
    }
    String::from_utf8(bytes).map_err(|_| "Code path files must contain UTF-8 text".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn import_editor_waits_for_active_enabled_focus_before_accepting_text() {
        let ctx = Context::default();
        let mut owner = ModalInput::default();
        let mut dialog = CodePathImportModal::default();
        for (active, busy) in [
            (false, false),
            (false, false),
            (true, true),
            (true, true),
            (true, false),
            (true, false),
        ] {
            super::super::focus_tests::frame(&ctx, &mut owner, active, vec![], |ctx| {
                dialog.show(ctx, &AppTheme::dark(), busy, None)
            });
            assert_eq!(dialog.focused, active && !busy);
        }
        super::super::focus_tests::frame(
            &ctx,
            &mut owner,
            true,
            vec![egui::Event::Text("Boot café\na.rs:main:Enter".into())],
            |ctx| dialog.show(ctx, &AppTheme::dark(), false, None),
        );
        assert_eq!(dialog.text, "Boot café\na.rs:main:Enter");
        // The file button is the next tab stop; initial focus must not steal it.
        super::super::focus_tests::frame(
            &ctx,
            &mut owner,
            true,
            vec![crate::ui::commands::test_key(Key::Tab)],
            |ctx| dialog.show(ctx, &AppTheme::dark(), false, None),
        );
        super::super::focus_tests::frame(
            &ctx,
            &mut owner,
            true,
            vec![egui::Event::Text("ignored".into())],
            |ctx| dialog.show(ctx, &AppTheme::dark(), false, None),
        );
        assert_eq!(dialog.text, "Boot café\na.rs:main:Enter");
    }

    #[test]
    fn file_import_is_regular_utf8_and_bounded_by_original_bytes() {
        let owner = tempfile::tempdir().unwrap();
        let path = owner.path().join("trace.txt");
        std::fs::write(&path, "Boot café 🦀\r\na.c:boot:Enter\r\n").unwrap();
        assert!(read_text_file(&path).unwrap().contains("café 🦀\r\n"));
        std::fs::write(&path, [0xff]).unwrap();
        assert!(read_text_file(&path).unwrap_err().contains("UTF-8"));
        std::fs::write(&path, vec![b'a'; MAX_IMPORT_BYTES]).unwrap();
        assert_eq!(read_text_file(&path).unwrap().len(), MAX_IMPORT_BYTES);
        std::fs::write(&path, vec![b'a'; MAX_IMPORT_BYTES + 1]).unwrap();
        assert!(read_text_file(&path).unwrap_err().contains("2 MiB"));
        assert!(read_text_file(owner.path()).is_err());
    }
}
