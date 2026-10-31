//! Common deterministic panel rendering for UI acceptance tests.

pub(crate) fn painted_text(
    ctx: &egui::Context,
    size: egui::Vec2,
    paint: impl FnMut(&mut egui::Ui),
) -> Vec<String> {
    painted_frame(ctx, size, vec![], paint)
        .shapes
        .into_iter()
        .filter_map(|shape| match shape.shape {
            egui::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect()
}

pub(crate) fn painted_frame(
    ctx: &egui::Context,
    size: egui::Vec2,
    events: Vec<egui::Event>,
    mut paint: impl FnMut(&mut egui::Ui),
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
            events,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                paint(ui);
            });
        },
    )
}

/// Finds a rendered label's center for pointer-driven acceptance checks.
pub(crate) fn text_center(output: &egui::FullOutput, label: &str) -> egui::Pos2 {
    output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == label => {
                Some(text.pos + text.galley.size() / 2.0)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing painted label {label}"))
}

/// Sends one complete pointer click to an observed control position.
pub(crate) fn click(position: egui::Pos2) -> Vec<egui::Event> {
    vec![
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
    ]
}

/// Sends one complete context-menu click to an observed control position.
pub(crate) fn secondary_click(position: egui::Pos2) -> Vec<egui::Event> {
    click(position)
        .into_iter()
        .map(|event| match event {
            egui::Event::PointerButton {
                pos,
                pressed,
                modifiers,
                ..
            } => egui::Event::PointerButton {
                pos,
                pressed,
                modifiers,
                button: egui::PointerButton::Secondary,
            },
            other => other,
        })
        .collect()
}

/// Runs a panel through the native reader-menu input boundary before egui focus.
pub(crate) fn menu_frame(
    ctx: &egui::Context,
    size: egui::Vec2,
    events: Vec<egui::Event>,
    paint: impl FnMut(&mut egui::Ui),
) -> egui::FullOutput {
    let mut input = egui::RawInput {
        events,
        ..Default::default()
    };
    super::context_menu::take_input(ctx, &mut input);
    painted_frame(ctx, size, input.events, paint)
}
