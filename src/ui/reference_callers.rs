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

pub(super) fn height(tree: Option<&CallerTree>, error: Option<&str>) -> f32 {
    HEADER_HEIGHT
        + tree.map_or(0.0, |tree| {
            ROW_HEIGHT * tree.rows.len() as f32
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
    ui.painter().text(
        header.left_center(),
        Align2::LEFT_CENTER,
        title,
        FontId::proportional(12.0),
        theme.text_secondary,
    );
    ui.interact(header, Id::new(("caller_header", reference)), Sense::hover()).on_hover_text("Calls traced backward from this site's containing callable. Dynamic dispatch, unindexed code and callback invocations may be missing. Up/Down moves between caller controls; Return opens a site; Left returns to the reference.");
    let mut action = None;
    if let Some(tree) = tree {
        let origin = ui.cursor().top();
        let first = ((ui.clip_rect().top() - origin).max(0.0) / ROW_HEIGHT).floor() as usize;
        let last = (((ui.clip_rect().bottom() - origin).max(0.0) / ROW_HEIGHT).ceil() as usize)
            .min(tree.rows.len());
        let first = first.min(tree.rows.len());
        ui.add_space(first as f32 * ROW_HEIGHT);
        for (caller, row) in tree.rows.iter().enumerate().take(last).skip(first) {
            if show_row(ui, theme, reference, caller, row) {
                action = Some(ReferenceAction::CommitCaller(reference, caller));
            }
        }
        ui.add_space((tree.rows.len() - last.max(first)) as f32 * ROW_HEIGHT);
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
                .interact(rect, control_id(reference, None), Sense::click())
                .union(response);
            super::commands::retain_navigation_keys(ui.ctx(), control_id(reference, None));
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
            if response.clicked() && !loading {
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

fn show_row(
    ui: &mut Ui,
    theme: &AppTheme,
    reference: usize,
    caller: usize,
    row: &CallerRow,
) -> bool {
    let (outer, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), ROW_HEIGHT), Sense::hover());
    let rect = egui::Rect::from_min_max(
        outer.min + egui::vec2(row.depth.min(4) as f32 * 10.0, 1.0),
        outer.max - egui::vec2(0.0, 1.0),
    );
    let response = ui.interact(rect, control_id(reference, Some(caller)), Sense::click());
    super::commands::retain_navigation_keys(ui.ctx(), control_id(reference, Some(caller)));
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
    let title = format!("{} · {}:{}", row.name, file, row.line);
    painter.text(
        rect.min + egui::vec2(5.0, 4.0),
        Align2::LEFT_TOP,
        title,
        FontId::monospace(12.0),
        theme.text_primary,
    );
    let detail = row.note.as_deref().unwrap_or(&row.location.path);
    painter.text(
        rect.min + egui::vec2(5.0, 23.0),
        Align2::LEFT_TOP,
        detail,
        FontId::proportional(11.0),
        theme.text_secondary,
    );
    if row.depth > 4 {
        painter.text(
            rect.right_top() + egui::vec2(-4.0, 4.0),
            Align2::RIGHT_TOP,
            format!("L{}", row.depth),
            FontId::proportional(11.0),
            theme.text_muted,
        );
    }
    let label = format!(
        "Caller level {} · {} · {}:{}{}",
        row.depth,
        row.name,
        row.location.path,
        row.line,
        row.note
            .as_ref()
            .map_or(String::new(), |note| format!(" · {note}"))
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, &label));
    let clicked = response.clicked();
    response.on_hover_text(label);
    clicked
}

#[cfg(test)]
mod tests {
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
        let tree = tree(1000);
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
            last = Some(render(ROW_HEIGHT * 1000.0 - 180.0));
        }
        let last = last.unwrap();
        assert!(last.shapes.len() < 100);
        assert!(last.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == "caller999 · source.rs:1")));
        assert!(last.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == "L999")));
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
                egui::Shape::Text(text) if text.galley.job.text == "caller1 · source.rs:1" => {
                    Some(text.pos + egui::vec2(20.0, 5.0))
                }
                _ => None,
            })
            .unwrap();
        render(vec![
            egui::Event::PointerMoved(position),
            egui::Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            },
        ]);
        assert_eq!(action, Some(ReferenceAction::CommitCaller(7, 1)));
    }
}
