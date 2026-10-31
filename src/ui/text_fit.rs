//! Measured single-row labels shared by constrained panes and breadcrumbs.

use egui::{text::LayoutJob, Color32, FontId, Galley, Ui};
use std::sync::Arc;

pub(crate) fn middle(ui: &Ui, text: &str, font: FontId, color: Color32, width: f32) -> Arc<Galley> {
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
            &text[..boundaries[keep.div_ceil(2)]],
            &text[boundaries[count - keep / 2]..]
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
    job.wrap.break_anywhere = true;
    ui.fonts(|fonts| fonts.layout_job(job))
}
