//! Canonical sparse numbers, fold meters and measured count badges.

use super::*;

fn paint_line_number(
    painter: &egui::Painter,
    row: Rect,
    line: usize,
    content: &CanvasContent<'_>,
) -> Option<Rect> {
    if !line.is_multiple_of(5) && line != *content.caret_line {
        return None;
    }
    let theme = content.theme;
    Some(painter.text(
        pos2(row.left() - 6.0, row.top() + 2.0),
        egui::Align2::RIGHT_TOP,
        line.to_string(),
        FontId::monospace(12.0),
        if line == *content.caret_line {
            theme.accent
        } else {
            theme.text_muted
        },
    ))
}

pub(super) fn paint_gutter(
    ui: &mut Ui,
    painter: &egui::Painter,
    surface: Rect,
    row: Rect,
    part: &SourceFragment,
    content: &CanvasContent<'_>,
) -> Option<SourceViewAction> {
    let theme = content.theme;
    let number = paint_line_number(painter, row, part.line, content);
    if content
        .changes
        .iter()
        .any(|range| range.contains(&part.line))
    {
        painter.rect_filled(
            Rect::from_min_size(pos2(row.left() - 3.0, row.top()), vec2(3.0, row.height())),
            0.0,
            if content.buffer.revision == Some(ReviewRevision::Before) {
                Color32::from_rgb(255, 59, 48)
            } else {
                Color32::from_rgb(52, 199, 89)
            },
        );
    }
    if let Some(region) = content.test_code.header(part.line) {
        let rect = Rect::from_min_size(
            pos2(surface.left() + 4.0, row.top() + 3.0),
            vec2(28.0, 16.0),
        );
        let hidden = content.test_code.is_hidden(region.byte_range.start);
        let icon = if hidden {
            Icon::ChevronRight
        } else {
            Icon::ChevronDown
        };
        icon.paint(
            painter,
            Rect::from_min_size(rect.min, vec2(10.0, 12.0)),
            theme.text_muted,
        );
        let count = region.line_range.end.saturating_sub(region.header_line + 1);
        let response = ui
            .interact(
                rect,
                Id::new(("test_gutter", region.byte_range.start)),
                Sense::click(),
            )
            .on_hover_text(format!("Test code — {count} lines hidden; click to toggle"));
        return response
            .clicked()
            .then_some(SourceViewAction::ToggleTestRegion(region.byte_range.start));
    }
    paint_structural_fold(ui, painter, surface, row, part, content, number)
}

fn paint_structural_fold(
    ui: &mut Ui,
    painter: &egui::Painter,
    surface: Rect,
    row: Rect,
    part: &SourceFragment,
    content: &CanvasContent<'_>,
    number: Option<Rect>,
) -> Option<SourceViewAction> {
    let theme = content.theme;
    let fold = content
        .folds
        .iter()
        .find(|fold| fold.header_line(content.buffer) == part.line)?;
    let rect = Rect::from_min_size(
        pos2(surface.left() + 4.0, row.top() + 3.0),
        vec2(28.0, 16.0),
    );
    let collapsed = content.fold_store.is_folded(&fold.byte_range);
    let icon = if collapsed {
        Icon::ChevronRight
    } else {
        Icon::ChevronDown
    };
    icon.paint(
        painter,
        Rect::from_min_size(rect.min, vec2(10.0, 12.0)),
        theme.text_muted,
    );
    let hidden = fold.hidden_line_count(content.buffer);
    let annotation = (content.mode == SourceMode::Critical)
        .then_some(content.annotations)
        .flatten()
        .and_then(|annotations| annotations.at_range(content.buffer, &fold.byte_range));
    if collapsed {
        if fold.kind == crate::syntax::folds::FoldKind::Callable {
            paint_function_meter(painter, surface, row, hidden, theme);
        }
        paint_fold_count(painter, surface, row, hidden, annotation, theme, number);
    }
    let tooltip = if collapsed {
        annotation.map_or_else(
            || format!("Collapsed block — {hidden} lines hidden; click to expand"),
            |annotation| {
                format!(
                    "Collapsed function — {hidden} lines hidden · {}",
                    annotation.detail
                )
            },
        )
    } else {
        "Click to collapse this block".into()
    };
    let hit = Rect::from_min_max(rect.min, pos2(surface.left() + 58.0, rect.bottom()));
    let response = ui.interact(
        hit,
        Id::new(("source_fold", fold.byte_range.start, fold.byte_range.end)),
        Sense::click(),
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, &tooltip));
    response
        .on_hover_text(tooltip)
        .clicked()
        .then_some(SourceViewAction::ToggleFold(fold.byte_range.clone()))
}

fn paint_function_meter(
    painter: &egui::Painter,
    surface: Rect,
    row: Rect,
    hidden: usize,
    theme: &AppTheme,
) {
    let width = 6.0 + 10.0 * ((hidden.min(256) + 1) as f32).log2() / 257.0_f32.log2();
    painter.rect_filled(
        Rect::from_min_size(
            pos2(surface.left() + (20.0 - width) / 2.0, row.top() + 15.0),
            vec2(width, 3.0),
        ),
        1.5,
        theme.text_secondary,
    );
}

fn paint_fold_count(
    painter: &egui::Painter,
    surface: Rect,
    row: Rect,
    hidden: usize,
    annotation: Option<&crate::critical::annotations::FoldAnnotation>,
    theme: &AppTheme,
    number: Option<Rect>,
) {
    let (label, color) = annotation.map_or_else(
        || {
            (
                if hidden < 10_000 {
                    hidden.to_string()
                } else {
                    "9999+".into()
                },
                theme.text_secondary,
            )
        },
        |annotation| {
            let emphasized = annotation.count.completeness
                == crate::critical::CountCompleteness::Partial
                || annotation.count.count > 0;
            (
                annotation.count.label(),
                if emphasized {
                    theme.accent
                } else {
                    theme.text_muted
                },
            )
        },
    );
    let galley = painter.layout_no_wrap(label, FontId::proportional(10.0), color);
    let width = (galley.size().x + 6.0).clamp(14.0, 32.0);
    let right = number.map_or(row.left() - 6.0, |number| number.left() - 3.0);
    let rect = Rect::from_min_size(
        pos2(surface.left() + 22.0, row.top() + 1.0),
        vec2(width, 17.0),
    );
    if rect.right() > right || galley.size().x > width {
        return;
    }
    let radius = if annotation.is_some() {
        3.0
    } else {
        rect.height() / 2.0
    };
    if annotation.is_none() {
        painter.rect_filled(rect, radius, theme.text_secondary.gamma_multiply(0.10));
    }
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(
            1.0_f32,
            if annotation.is_some() {
                color
            } else {
                theme.border
            },
        ),
    );
    painter.galley(rect.center() - galley.size() / 2.0, galley, color);
}
