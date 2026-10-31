//! Common deterministic panel rendering for UI acceptance tests.

pub(crate) fn painted_text(
    ctx: &egui::Context,
    size: egui::Vec2,
    paint: impl FnMut(&mut egui::Ui),
) -> Vec<String> {
    let output = painted_frame(ctx, size, vec![], paint);
    text_shapes(&output)
        .map(|text| text.galley.job.text.clone())
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
    text_shapes(output)
        .find(|text| text.galley.job.text == label)
        .map(|text| text.pos + text.galley.size() / 2.0)
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

/// Applies a paired palette and UI scale for pane acceptance fixtures.
pub(crate) fn themed_context(dark: bool, scale: f32) -> (egui::Context, crate::theme::AppTheme) {
    let ctx = egui::Context::default();
    let theme = if dark {
        crate::theme::AppTheme::dark()
    } else {
        crate::theme::AppTheme::light()
    };
    ctx.set_visuals(theme.visuals());
    ctx.set_pixels_per_point(scale);
    (ctx, theme)
}

/// Iterates painted text geometry without repeating shape filtering in assertions.
pub(crate) fn text_shapes(
    output: &egui::FullOutput,
) -> impl Iterator<Item = &egui::epaint::TextShape> {
    output.shapes.iter().filter_map(|shape| match &shape.shape {
        egui::Shape::Text(text) => Some(text),
        _ => None,
    })
}
