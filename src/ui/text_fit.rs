//! Measured labels and bounded explanations for constrained panes and breadcrumbs.

use egui::{text::LayoutJob, Color32, FontId, Galley, Ui};
use std::{borrow::Cow, sync::Arc};

/// Shows up to three metadata rows with a bounded full explanation on hover.
pub(crate) fn explanation(ui: &mut Ui, text: &str, color: Color32) {
    let text_layout = layout_lines(
        ui,
        text,
        egui::TextStyle::Small.resolve(ui.style()),
        color,
        ui.available_width(),
        3,
        false,
    );
    ui.add(egui::Label::new(text_layout))
        .on_hover_text(bounded(text));
}

pub(crate) fn middle(ui: &Ui, text: &str, font: FontId, color: Color32, width: f32) -> Arc<Galley> {
    fit(ui, text, font, color, width, false)
}

/// Fits one row while preserving the filename/location end of a path.
pub(crate) fn head(ui: &Ui, text: &str, font: FontId, color: Color32, width: f32) -> Arc<Galley> {
    fit(ui, text, font, color, width, true)
}

fn fit(ui: &Ui, text: &str, font: FontId, color: Color32, width: f32, head: bool) -> Arc<Galley> {
    let text = bounded(text);
    let text = text.as_ref();
    let shape = |text: String| {
        let mut job = LayoutJob::simple(text, font.clone(), color, f32::INFINITY);
        job.break_on_newline = false;
        ui.fonts(|fonts| fonts.layout_job(job))
    };
    let full = shape(text.to_owned());
    if full.size().x <= width {
        return full;
    }
    let boundaries: Vec<_> = text
        .char_indices()
        .map(|(at, _)| at)
        .chain(std::iter::once(text.len()))
        .collect();
    let count = boundaries.len() - 1;
    let (mut low, mut high) = (0, count);
    let mut fitted = shape(String::new());
    while low < high {
        let keep = low + (high - low) / 2;
        let candidate = shape(format!(
            "{}…{}",
            &text[..boundaries[if head { 0 } else { keep.div_ceil(2) }]],
            &text[boundaries[count - if head { keep } else { keep / 2 }]..]
        ));
        if candidate.size().x <= width.max(0.0) {
            fitted = candidate;
            low = keep + 1;
        } else {
            high = keep;
        }
    }
    fitted
}

/// Shapes a bounded prefix into a fixed number of tail-elided rows.
pub(crate) fn lines(
    ui: &Ui,
    text: &str,
    font: FontId,
    color: Color32,
    width: f32,
    rows: usize,
) -> Arc<Galley> {
    layout_lines(ui, text, font, color, width, rows, true)
}

fn layout_lines(
    ui: &Ui,
    text: &str,
    font: FontId,
    color: Color32,
    width: f32,
    rows: usize,
    break_anywhere: bool,
) -> Arc<Galley> {
    let mut end = text.len().min(4096);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    let text = if end < text.len() {
        format!("{}…", &text[..end])
    } else {
        text.to_owned()
    };
    let mut job = LayoutJob::simple(text, font, color, width.max(0.0));
    job.wrap.max_rows = rows;
    job.wrap.break_anywhere = break_anywhere;
    ui.fonts(|fonts| fonts.layout_job(job))
}

// Keep ancillary labels/help bounded while preserving both ends of identifiers.
pub(crate) fn bounded(text: &str) -> Cow<'_, str> {
    if text.len() <= 4096 {
        return Cow::Borrowed(text);
    }
    let mut left = 2046;
    while !text.is_char_boundary(left) {
        left -= 1;
    }
    let mut right = text.len() - 2047;
    while !text.is_char_boundary(right) {
        right += 1;
    }
    Cow::Owned(format!("{}…{}", &text[..left], &text[right..]))
}
