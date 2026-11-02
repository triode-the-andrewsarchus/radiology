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

/// Drives asynchronous picker publication through actual frames with a deadline.
pub(crate) fn settled_modal(
    mut frame: impl FnMut() -> (egui::FullOutput, Option<super::modals::ModalAction>, bool),
) -> (egui::FullOutput, Option<super::modals::ModalAction>) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let mut frames = 0;
    loop {
        frames += 1;
        let (output, action, idle) = frame();
        if action.is_some() || (frames >= 2 && idle) {
            return (output, action);
        }
        assert!(std::time::Instant::now() < deadline, "modal did not settle");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
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

/// Builds the shared, source-validated definition/reference inspector fixture.
pub(crate) fn indexed_under_caret_context() -> crate::semantic::context::UnderCaretContext {
    use crate::{
        semantic::{
            context::UnderCaretContext, ScipDocument, ScipIndex, ScipSymbolInformation,
            SemanticIndex,
        },
        source::SourceBuffer,
    };
    use std::sync::atomic::AtomicBool;
    let buffer = SourceBuffer::from_str("source.rs", "fn target() {}\nfn main() { target(); }\n");
    let symbol = "test cargo app 1 target().";
    let occurrences = [
        buffer.text.find("target").unwrap(),
        buffer.text.rfind("target").unwrap(),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, byte)| {
        crate::semantic::test_support::occurrence(&buffer, byte..byte + 6, symbol, index == 0)
    })
    .collect();
    let index = SemanticIndex::from_scip(ScipIndex {
        documents: vec![ScipDocument {
            relative_path: buffer.path.clone(),
            position_encoding: 2,
            occurrences,
            symbols: vec![ScipSymbolInformation {
                symbol: symbol.into(),
                kind: 17,
                signature: "fn target()".into(),
                documentation: vec!["Useful documentation.".into()],
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    });
    UnderCaretContext::prepare(
        &buffer,
        &index,
        buffer.text.rfind("target").unwrap(),
        &AtomicBool::new(false),
    )
    .unwrap()
    .unwrap()
}

pub(crate) fn reference_hit(index: usize) -> crate::semantic::reference_results::ReferenceHit {
    crate::semantic::reference_results::ReferenceHit {
        path: "a.rs".into(),
        range: crate::semantic::ScipRange::from_packed(&[index as i32, 0, 1]).unwrap(),
        byte_range: index..index + 1,
        source_line: crate::source::preview::PreviewLine {
            number: index + 1,
            text: format!("caller{index}"),
            emphasis: Some(0..1),
            clipped: false,
        },
        is_definition: false,
    }
}
pub(crate) fn reference_results(
    hits: Vec<crate::semantic::reference_results::ReferenceHit>,
) -> crate::semantic::reference_results::ReferenceResults {
    crate::semantic::reference_results::ReferenceResults {
        groups: vec![crate::semantic::reference_results::ReferenceGroup {
            path: "a.rs".into(),
            hits: 0..hits.len(),
            content_digest: String::new(),
        }],
        reference_count: hits.len(),
        hits,
        ..Default::default()
    }
}

/// Verifies that a named label is fully painted inside its actual clip rectangle.
pub(crate) fn assert_visible_text(output: &egui::FullOutput, label: &str) {
    let (clip, text) = output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == label => {
                Some((shape.clip_rect, text))
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing painted label {label}"));
    let rect = egui::Rect::from_min_size(text.pos, text.galley.size());
    assert!(
        clip.expand(0.5).contains_rect(rect),
        "clipped {label}: {rect:?} by {clip:?}"
    );
}
