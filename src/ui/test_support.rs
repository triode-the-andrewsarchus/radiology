//! Common deterministic panel rendering for UI acceptance tests.

pub(crate) fn painted_text(
    ctx: &egui::Context,
    size: egui::Vec2,
    mut paint: impl FnMut(&mut egui::Ui),
) -> Vec<String> {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                paint(ui);
            });
        },
    )
    .shapes
    .into_iter()
    .filter_map(|shape| match shape.shape {
        egui::Shape::Text(text) => Some(text.galley.job.text.clone()),
        _ => None,
    })
    .collect()
}
