//! Bounded caller-path painting inside the reference list's shared scroll area.

use super::reference_list::ReferenceAction;
use crate::{
    semantic::{
        caller_pill_color,
        incoming_calls::{CallerRow, CallerTree},
    },
    theme::AppTheme,
};
use egui::{Align2, FontId, Id, Sense, Ui};

pub(super) const ROW_HEIGHT: f32 = 44.0;
pub(super) const HEADER_HEIGHT: f32 = 24.0;

pub(super) fn control_id(reference: usize, caller: Option<usize>) -> Id {
    Id::new(("reference_caller", reference, caller))
}

// Include source identity so a retained press cannot transfer to a replacement row.
pub(super) fn row_id(reference: usize, caller: usize, row: &CallerRow, digest: Option<&str>) -> Id {
    control_id(reference, Some(caller)).with((&row.location, digest))
}

pub(super) fn more_id(reference: usize, tree: &CallerTree) -> Id {
    control_id(reference, None).with((
        tree.rows.first().map(|row| &row.location),
        &tree.source_identities,
    ))
}

fn row_height(row: &CallerRow) -> f32 {
    ROW_HEIGHT + 20.0 * f32::from(row.note.is_some())
}

pub(super) fn row_offset(tree: &CallerTree, caller: usize) -> f32 {
    tree.rows.iter().take(caller).map(row_height).sum()
}

pub(super) fn height(tree: Option<&CallerTree>, error: Option<&str>) -> f32 {
    HEADER_HEIGHT
        + tree.map_or(0.0, |tree| {
            row_offset(tree, tree.rows.len())
                + 24.0 * f32::from(!tree.notices.is_empty())
                + 24.0 * f32::from(tree.has_more)
        })
        + 24.0 * f32::from(error.is_some())
}

pub(super) fn show(
    ui: &mut Ui,
    theme: &AppTheme,
    reference: usize,
    tree: Option<&CallerTree>,
    loading: bool,
    error: Option<&str>,
) -> Option<ReferenceAction> {
    let (header, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), HEADER_HEIGHT),
        Sense::hover(),
    );
    let title = match tree {
        Some(tree) if loading => format!("Callers · {} rows · Tracing…", tree.rows.len()),
        Some(tree) => format!("Callers · {} rows", tree.rows.len()),
        None if loading => "Callers · Tracing…".into(),
        None => "Callers".into(),
    };
    let title = super::text_fit::middle(
        ui,
        &title,
        FontId::proportional(12.0),
        theme.text_secondary,
        header.width(),
    );
    ui.painter().galley(
        header.left_center() - egui::vec2(0.0, title.size().y / 2.0),
        title,
        theme.text_secondary,
    );
    ui.interact(header, Id::new(("caller_header", reference)), Sense::hover()).on_hover_text("Calls traced backward from this site's containing callable. Dynamic dispatch, unindexed code and callback invocations may be missing. Up/Down moves between caller controls; Return opens a site; Left returns to the reference.");
    let mut action = None;
    if let Some(tree) = tree {
        action = show_rows(ui, theme, reference, tree);
        if !tree.notices.is_empty() {
            ui.add_sized(
                [ui.available_width(), 24.0],
                egui::Label::new(
                    egui::RichText::new("Partial caller tree").color(theme.text_muted),
                )
                .truncate(true),
            )
            .on_hover_text(tree.notices.join("\n"));
        }
        if tree.has_more {
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(ui.available_width(), 24.0), Sense::hover());
            let response = ui
                .interact(rect, more_id(reference, tree), Sense::click())
                .union(response);
            super::commands::retain_navigation_keys(ui.ctx(), more_id(reference, tree));
            ui.painter().text(
                rect.left_center(),
                Align2::LEFT_CENTER,
                "Trace More Callers",
                FontId::proportional(12.0),
                if loading {
                    theme.text_muted
                } else {
                    theme.accent
                },
            );
            if response.has_focus() {
                ui.painter().rect_stroke(
                    rect.shrink(1.0),
                    3.0,
                    egui::Stroke::new(1.0_f32, theme.accent),
                );
            }
            if super::commands::row_activated(&response) && !loading {
                action = Some(ReferenceAction::MoreCallers(reference));
            }
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, "Trace More Callers")
            });
        }
    }
    if let Some(error) = error {
        ui.add_sized(
            [ui.available_width(), 24.0],
            egui::Label::new(egui::RichText::new("Callers unavailable").color(theme.text_muted))
                .truncate(true),
        )
        .on_hover_text(error);
    }
    action
}

fn show_rows(
    ui: &mut Ui,
    theme: &AppTheme,
    reference: usize,
    tree: &CallerTree,
) -> Option<ReferenceAction> {
    let mut action = None;
    let mut top = ui.cursor().top();
    for (caller, row) in tree.rows.iter().enumerate() {
        let bottom = top + row_height(row);
        if bottom >= ui.clip_rect().top() && top <= ui.clip_rect().bottom() {
            ui.add_space((top - ui.cursor().top()).max(0.0));
            if show_row(
                ui,
                theme,
                reference,
                caller,
                row,
                tree.source_identities
                    .get(&row.location.path)
                    .map(String::as_str),
            ) {
                action = Some(ReferenceAction::CommitCaller(reference, caller));
            }
        }
        top = bottom;
    }
    ui.add_space((top - ui.cursor().top()).max(0.0));
    action
}

fn show_row(
    ui: &mut Ui,
    theme: &AppTheme,
    reference: usize,
    caller: usize,
    row: &CallerRow,
    digest: Option<&str>,
) -> bool {
    let (outer, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), row_height(row)),
        Sense::hover(),
    );
    let rect = egui::Rect::from_min_max(
        outer.min + egui::vec2(row.depth.min(4) as f32 * 10.0, 1.0),
        outer.max - egui::vec2(0.0, 1.0),
    );
    let id = row_id(reference, caller, row, digest);
    let response = ui.interact(rect, id, Sense::click());
    super::commands::retain_navigation_keys(ui.ctx(), id);
    let color = caller_pill_color(&row.name, &row.location.path);
    ui.painter().rect_filled(
        rect,
        3.0,
        color.linear_multiply(if response.hovered() || response.has_focus() {
            0.22
        } else {
            0.10
        }),
    );
    if response.has_focus() {
        ui.painter()
            .rect_stroke(rect, 3.0, egui::Stroke::new(1.0_f32, theme.accent));
    }
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    let file = row
        .location
        .path
        .rsplit('/')
        .next()
        .unwrap_or(&row.location.path);
    let metadata = if row.depth > 4 {
        format!("{} · L{}", row.line, row.depth)
    } else {
        row.line.to_string()
    };
    let metadata = super::text_fit::middle(
        ui,
        &metadata,
        FontId::proportional(11.0),
        theme.text_muted,
        rect.width() * 0.4,
    );
    let name = super::text_fit::middle(
        ui,
        &row.name,
        FontId::monospace(12.0),
        theme.text_primary,
        (rect.width() - metadata.size().x - 16.0).max(0.0),
    );
    painter.galley(rect.min + egui::vec2(5.0, 4.0), name, theme.text_primary);
    painter.galley(
        egui::pos2(rect.right() - metadata.size().x - 5.0, rect.top() + 4.0),
        metadata,
        theme.text_muted,
    );
    let file = super::text_fit::middle(
        ui,
        file,
        FontId::proportional(11.0),
        theme.text_secondary,
        rect.width() - 10.0,
    );
    painter.galley(rect.min + egui::vec2(5.0, 23.0), file, theme.text_secondary);
    if let Some(note) = &row.note {
        let text = super::text_fit::middle(
            ui,
            note,
            FontId::proportional(11.0),
            theme.text_muted,
            rect.width() - 10.0,
        );
        painter.galley(rect.min + egui::vec2(5.0, 43.0), text, theme.text_muted);
    }
    let label = format!(
        "Caller level {} · {} · {}:{}{}",
        row.depth,
        super::text_fit::bounded(&row.name),
        super::text_fit::bounded(&row.location.path),
        row.line,
        row.note.as_ref().map_or(String::new(), |note| format!(
            " · {}",
            super::text_fit::bounded(note)
        ))
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, &label));
    let clicked = super::commands::row_activated(&response);
    response.on_hover_text(label);
    clicked
}

#[cfg(test)]
mod tests {
    mod presentation;
    use super::*;
    use crate::{semantic::incoming_calls::CallerRow, source::SourceLocation};

    fn tree(count: usize) -> CallerTree {
        CallerTree {
            rows: (0..count)
                .map(|index| CallerRow {
                    depth: index,
                    name: format!("caller{index}"),
                    location: SourceLocation::new("source.rs", 0),
                    line: 1,
                    column: 0,
                    symbol: None,
                    note: None,
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn deep_trees_paint_only_the_visible_slice_and_reveal_the_end() {
        let ctx = egui::Context::default();
        let mut tree = tree(1000);
        for (index, row) in tree.rows.iter_mut().enumerate() {
            if index % 2 == 0 {
                row.note = Some("A boundary note".into());
            }
        }
        let render = |offset| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(300.0, 220.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        egui::ScrollArea::vertical()
                            .vertical_scroll_offset(offset)
                            .show(ui, |ui| {
                                ui.spacing_mut().item_spacing.y = 0.0;
                                show(ui, &AppTheme::dark(), 0, Some(&tree), false, None);
                            });
                    });
                },
            )
        };
        let initial = render(0.0);
        assert!(
            initial.shapes.len() < 100,
            "offscreen caller rows were painted"
        );
        let mut last = None;
        for _ in 0..3 {
            last = Some(render(row_offset(&tree, 1000) - 180.0));
        }
        let last = last.unwrap();
        assert!(last.shapes.len() < 100);
        assert!(last.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == "caller999")));
        assert!(last.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == "1 · L999")));
    }

    #[test]
    fn coalesced_caller_click_commits_the_exact_tree_row() {
        let ctx = egui::Context::default();
        let tree = tree(2);
        let mut action = None;
        let mut render = |events| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(300.0, 220.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        action = show(ui, &AppTheme::dark(), 7, Some(&tree), false, None);
                    });
                },
            )
        };
        let initial = render(vec![]);
        let position = initial
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.job.text == "caller1" => {
                    Some(text.pos + egui::vec2(20.0, 5.0))
                }
                _ => None,
            })
            .unwrap();
        render(crate::ui::test_support::click(position));
        assert_eq!(action, Some(ReferenceAction::CommitCaller(7, 1)));
    }
}
