//! Bounded source location with Swift-style suffix compaction and full-location menu.

use super::*;
use crate::ui::{commands::MenuItem, context_menu, text_fit};
use egui::{vec2, Align, FontId, Layout, Sense, TextStyle};

pub(super) struct Header<'a> {
    pub buffer: &'a SourceBuffer,
    pub symbols: &'a [DocumentSymbol],
    pub line: usize,
    pub changes: Option<&'a FileLineChanges>,
    pub change: Option<&'a crate::pull_request::change::FileChange>,
    pub can_collapse: bool,
}

pub(super) fn show(ui: &mut Ui, theme: &AppTheme, header: Header<'_>) -> Option<SourceViewAction> {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 28.0), Sense::hover());
    let rect = rect.shrink2(vec2(10.0, 0.0));
    let mut tools = ui.child_ui(rect, Layout::right_to_left(Align::Center));
    tools.set_clip_rect(ui.clip_rect().intersect(rect));
    tools.spacing_mut().item_spacing.x = 6.0;
    let mut action = None;
    if let Some(revision) = header.buffer.revision {
        action = revision_buttons(&mut tools, revision, header.change)
            .map(SourceViewAction::ToggleRevision);
        tools.separator();
    }
    tools.add_enabled_ui(header.can_collapse, |ui| {
        if Icon::Wrap
            .button(ui, 11.0, theme.text_secondary)
            .on_hover_text("Collapse all function bodies")
            .clicked()
        {
            action = Some(SourceViewAction::CollapseFunctions);
        }
    });
    if let Some(diff) = header.changes.filter(|diff| !diff.hunks.is_empty()) {
        let navigation = diff.navigation_at(
            header.line,
            header.buffer.revision.unwrap_or(ReviewRevision::After),
        );
        for (icon, enabled, help, event) in [
            (
                Icon::ArrowDown,
                navigation.next.is_some(),
                "Next change block (Alt+↓)",
                SourceViewAction::NextChangeBlock,
            ),
            (
                Icon::ArrowUp,
                navigation.previous.is_some(),
                "Previous change block (Alt+↑)",
                SourceViewAction::PrevChangeBlock,
            ),
        ] {
            tools.add_enabled_ui(enabled, |ui| {
                if icon
                    .button(ui, 11.0, theme.text_primary)
                    .on_hover_text(help)
                    .clicked()
                {
                    action = Some(event);
                }
            });
        }
    }
    tools.separator();
    let available = tools.available_rect_before_wrap();
    let mut trail = ui.child_ui(available, Layout::left_to_right(Align::Center));
    trail.set_clip_rect(ui.clip_rect().intersect(available));
    trail.spacing_mut().item_spacing.x = 0.0;
    show_trail(&mut trail, theme, &header).or(action)
}

struct Entry<'a> {
    text: &'a str,
    jump: Option<usize>,
    color: Color32,
    width: f32,
}

fn entries<'a>(
    ui: &Ui,
    theme: &AppTheme,
    header: &'a Header<'a>,
    font: &FontId,
) -> (Vec<Entry<'a>>, usize) {
    let mut result = Vec::new();
    let path: Vec<_> = header
        .buffer
        .path
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    let directories = path.len().saturating_sub(1);
    for (index, text) in path.into_iter().enumerate() {
        result.push(Entry {
            text,
            jump: None,
            color: if index < directories {
                theme.text_secondary
            } else {
                theme.text_primary
            },
            width: 0.0,
        });
    }
    for (index, symbol) in header.symbols.iter().enumerate() {
        result.push(Entry {
            text: &symbol.name,
            jump: Some(symbol.selection_range.start),
            color: if index + 1 == header.symbols.len() {
                theme.text_primary
            } else {
                theme.text_secondary
            },
            width: 0.0,
        });
    }
    for entry in &mut result {
        entry.width = text_fit::middle(ui, entry.text, font.clone(), entry.color, f32::INFINITY)
            .size()
            .x;
    }
    (result, directories)
}

/// Tries complete declarations before dropping their outer context; keeps nearby directories.
fn compact(entries: &[Entry<'_>], directories: usize, width: f32) -> Vec<usize> {
    let symbols = entries.len().saturating_sub(directories + 1);
    for keep in (1..=symbols.max(1)).rev() {
        for omitted in 0..=directories {
            let indices: Vec<_> = (omitted..=directories)
                .chain(entries.len() - keep.min(symbols)..entries.len())
                .collect();
            let omitted_any = indices.len() < entries.len();
            let needed = indices
                .iter()
                .map(|&index| entries[index].width)
                .sum::<f32>()
                + (indices.len().saturating_sub(1) as f32 * 18.0)
                + if omitted_any { 38.0 } else { 0.0 };
            if needed <= width {
                return indices;
            }
        }
    }
    entries.len().checked_sub(1).into_iter().collect()
}

fn show_trail(ui: &mut Ui, theme: &AppTheme, header: &Header<'_>) -> Option<SourceViewAction> {
    let font = TextStyle::Body.resolve(ui.style());
    let (entries, directories) = entries(ui, theme, header, &font);
    if entries.is_empty() {
        return None;
    }
    let visible = compact(&entries, directories, ui.available_width());
    let mut action = None;
    if visible.len() < entries.len()
        || entries
            .last()
            .is_some_and(|entry| entry.width > ui.available_width())
    {
        action = overflow(ui, header);
        separator(ui, theme);
    }
    for (position, &index) in visible.iter().enumerate() {
        if position > 0 {
            separator(ui, theme);
        }
        let entry = &entries[index];
        let galley = text_fit::middle(
            ui,
            entry.text,
            font.clone(),
            entry.color,
            ui.available_width(),
        );
        let (rect, response) = ui.allocate_exact_size(
            vec2(galley.size().x, 24.0),
            if entry.jump.is_some() {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        ui.painter().galley(
            pos2(rect.left(), rect.center().y - galley.size().y / 2.0),
            galley,
            entry.color,
        );
        let response = response.on_hover_text(format!(
            "{}{}",
            header.buffer.path,
            entry
                .jump
                .map_or_else(String::new, |_| format!(" / {}", entry.text))
        ));
        if response.clicked() {
            action = entry.jump.map(SourceViewAction::JumpToSymbol);
        }
    }
    action
}

fn separator(ui: &mut Ui, theme: &AppTheme) {
    let (rect, _) = ui.allocate_exact_size(vec2(18.0, 24.0), Sense::hover());
    Icon::ChevronRight.paint(
        ui.painter(),
        Rect::from_center_size(rect.center(), vec2(10.0, 10.0)),
        theme.text_muted,
    );
}

fn overflow(ui: &mut Ui, header: &Header<'_>) -> Option<SourceViewAction> {
    #[derive(Clone)]
    enum Action {
        CopyPath,
        Jump(usize),
    }
    let (rect, response) = ui.allocate_exact_size(vec2(20.0, 24.0), Sense::click());
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "…",
        TextStyle::Body.resolve(ui.style()),
        ui.visuals().text_color(),
    );
    let response = response.on_hover_text(&header.buffer.path);
    let opening = (response.clicked()
        || (response.contains_pointer() && ui.input(|input| input.pointer.primary_pressed())))
    .then_some(rect.left_bottom());
    let scope = (
        header.buffer.position_key(),
        header.buffer.content_identity(),
        header
            .symbols
            .iter()
            .map(|symbol| (&symbol.name, symbol.selection_range.start))
            .collect::<Vec<_>>(),
    );
    match context_menu::show_at(&response, scope, opening, || {
        let mut items = vec![
            MenuItem::action(&header.buffer.path, Action::CopyPath).hint("Copy relative path")
        ];
        if !header.symbols.is_empty() {
            items.push(MenuItem::separator());
        }
        items.extend(header.symbols.iter().map(|symbol| {
            MenuItem::action(&symbol.name, Action::Jump(symbol.selection_range.start))
        }));
        items
    }) {
        Some(Action::CopyPath) => {
            ui.output_mut(|output| output.copied_text = header.buffer.path.clone());
            None
        }
        Some(Action::Jump(byte)) => Some(SourceViewAction::JumpToSymbol(byte)),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::test_support::{click, painted_frame, text_center};

    fn symbols() -> Vec<DocumentSymbol> {
        ["OuterType", "innerMethod", "target"]
            .into_iter()
            .enumerate()
            .map(|(index, name)| DocumentSymbol {
                name: name.into(),
                kind: crate::syntax::outline::SymbolKind::Function,
                range: 0..14,
                selection_range: index * 4..index * 4 + 3,
                line: 1,
                children: vec![],
                detail: None,
            })
            .collect()
    }

    #[test]
    fn compact_header_keeps_priority_labels_and_fixed_review_controls_within_its_pane() {
        for scale in [1.0, 2.0] {
            for theme in [AppTheme::dark(), AppTheme::light()] {
                for width in [360.0, 440.0, 560.0, 740.0, 1100.0] {
                    let ctx = egui::Context::default();
                    ctx.set_pixels_per_point(scale);
                    let mut buffer = SourceBuffer::from_str(
                        "one-very-long-directory/two-very-long-directory/src/file.rs",
                        "fn target() {}\n",
                    );
                    buffer.revision = Some(ReviewRevision::After);
                    let symbols = symbols();
                    let changes = crate::pull_request::diff::UnifiedDiffParser::parse(
                        "@@ -2 +2 @@\n-old\n+new\n@@ -8 +8 @@\n-old\n+new\n",
                    );
                    let mut output = Default::default();
                    for _ in 0..2 {
                        output = painted_frame(&ctx, vec2(width, 100.0), vec![], |ui| {
                            let available = ui.available_rect_before_wrap();
                            show(
                                ui,
                                &theme,
                                Header {
                                    buffer: &buffer,
                                    symbols: &symbols,
                                    line: 4,
                                    changes: Some(&changes),
                                    change: None,
                                    can_collapse: true,
                                },
                            );
                            assert!(ui.min_rect().right() <= available.right() + 0.5);
                            assert!(
                                ui.next_widget_position().y
                                    <= available.top() + 28.5 + ui.spacing().item_spacing.y
                            );
                        });
                    }
                    let labels: Vec<_> = output
                        .shapes
                        .iter()
                        .filter_map(|shape| match &shape.shape {
                            egui::Shape::Text(text) => {
                                assert_eq!(text.galley.rows.len(), 1);
                                assert!(
                                    text.pos.x + text.galley.size().x
                                        <= shape.clip_rect.right() + 1.0,
                                    "{} at {width}/{scale}",
                                    text.galley.job.text
                                );
                                Some(text.galley.job.text.as_str())
                            }
                            _ => None,
                        })
                        .collect();
                    for retained in ["Before", "After", "file.rs", "target"] {
                        assert!(
                            labels.contains(&retained),
                            "missing {retained} at {width}/{scale}: {labels:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn compacted_location_menu_copies_full_path_and_jumps_to_every_enclosing_declaration() {
        let ctx = egui::Context::default();
        let buffer = SourceBuffer::from_str(
            format!("nested/{}/very-long-🦀-filename.rs", "directory".repeat(12)),
            "fn target() {}\n",
        );
        let symbols = symbols();
        let frame = |events| {
            let mut action = None;
            let output = painted_frame(&ctx, vec2(360.0, 300.0), events, |ui| {
                action = show(
                    ui,
                    &AppTheme::dark(),
                    Header {
                        buffer: &buffer,
                        symbols: &symbols,
                        line: 1,
                        changes: None,
                        change: None,
                        can_collapse: true,
                    },
                );
            });
            (output, action)
        };
        let (output, _) = frame(vec![]);
        let target = text_center(&output, "target");
        assert_eq!(
            frame(click(target)).1,
            Some(SourceViewAction::JumpToSymbol(8))
        );
        for (keys, expected) in [
            (vec![egui::Key::Home, egui::Key::Enter], None),
            (
                vec![egui::Key::Home, egui::Key::ArrowDown, egui::Key::Enter],
                Some(0),
            ),
            (
                vec![
                    egui::Key::Home,
                    egui::Key::ArrowDown,
                    egui::Key::ArrowDown,
                    egui::Key::Enter,
                ],
                Some(4),
            ),
            (vec![egui::Key::End, egui::Key::Enter], Some(8)),
        ] {
            let (output, _) = frame(vec![]);
            frame(click(text_center(&output, "…")));
            let events = keys
                .into_iter()
                .map(crate::ui::commands::test_key)
                .collect();
            let (output, action) = frame(events);
            assert_eq!(action, expected.map(SourceViewAction::JumpToSymbol));
            if expected.is_none() {
                assert_eq!(output.platform_output.copied_text, buffer.path);
            }
        }
    }
}
