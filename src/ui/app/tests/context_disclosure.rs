//! Independent disclosure state and explicit reference focus on Swift's fixture.
use super::*;
use crate::ui::{
    context_inspector::InspectorAction,
    reference_list::ReferenceAction,
    test_support::{click, text_shapes},
};

pub(super) fn assert_disclosure_and_focus(app: &mut RadiologyApp, ctx: &Context, root: &Path) {
    app.open_file("src/shapes.rs");
    focus::settle_frame(app, ctx);
    app.caret_line = 8;
    app.caret_col = 22;
    app.refresh_symbol_context();
    focus::settle_frame(app, ctx);
    assert_eq!(app.under_caret.as_ref().unwrap().name, "radius");
    assert!(!app.definition_expanded && app.reference_context.disclosures.is_empty());
    let signature = "pub fn area(&self) -> f64";
    assert!(!has_signature(&focus::frame(app, ctx, vec![]), signature));
    toggle_signature(app, ctx, "area");
    assert!(has_signature(&focus::frame(app, ctx, vec![]), signature));
    let call = app
        .reference_context
        .results
        .hits
        .iter()
        .position(|hit| hit.path == "src/lib.rs")
        .unwrap();
    app.handle_inspector_action(ctx, InspectorAction::ExpandDefinition(true));
    app.handle_reference_action(ReferenceAction::SetExpanded(call, true));
    app.handle_reference_action(ReferenceAction::SetExpanded(usize::MAX, true));
    focus::settle_frame(app, ctx);
    assert_eq!(
        app.reference_context
            .disclosures
            .keys()
            .copied()
            .collect::<Vec<_>>(),
        [call]
    );
    app.caret_col = 25;
    app.refresh_symbol_context();
    focus::settle_frame(app, ctx);
    assert!(app.definition_expanded && app.reference_context.disclosures.contains_key(&call));
    assert!(has_signature(&focus::frame(app, ctx, vec![]), signature));
    app.caret_col = 18;
    app.refresh_symbol_context();
    focus::settle_frame(app, ctx);
    assert!(!app.definition_expanded && app.reference_context.disclosures.contains_key(&call));
    assert!(has_signature(&focus::frame(app, ctx, vec![]), signature));
    app.handle_inspector_action(ctx, InspectorAction::ExpandDefinition(true));
    app.caret_line = 6;
    app.caret_col = 40;
    app.refresh_symbol_context();
    focus::settle_frame(app, ctx);
    assert!(!app.definition_expanded && app.reference_context.disclosures.is_empty());
    let new_signature = "pub fn new(radius: f64) -> Self";
    assert!(!has_signature(
        &focus::frame(app, ctx, vec![]),
        new_signature
    ));
    for expanded in [true, false, true] {
        app.handle_reference_action(ReferenceAction::SetExpanded(0, expanded));
        assert_eq!(app.reference_context.disclosures.contains_key(&0), expanded);
    }
    toggle_signature(app, ctx, "new");
    assert!(has_signature(
        &focus::frame(app, ctx, vec![]),
        new_signature
    ));
    app.reset_repository_state();
    assert!(!app.definition_expanded && app.reference_context.disclosures.is_empty());
    app.open_repository(root);
    focus::settle_frame(app, ctx);
    app.open_file("src/shapes.rs");
    focus::settle_frame(app, ctx);
    app.caret_line = 8;
    app.caret_col = 22;
    app.refresh_symbol_context();
    focus::settle_frame(app, ctx);
    assert!(!has_signature(&focus::frame(app, ctx, vec![]), signature));
    toggle_signature(app, ctx, "area");
    assert!(has_signature(&focus::frame(app, ctx, vec![]), signature));
    let index = app.semantic_index.clone().unwrap();
    app.semantic_index = Some(Arc::new((*index).clone()));
    app.refresh_symbol_context();
    focus::settle_frame(app, ctx);
    assert!(
        !has_signature(&focus::frame(app, ctx, vec![]), signature),
        "index replacement must retire the old declaration disclosure"
    );

    app.open_file("src/lib.rs");
    focus::settle_frame(app, ctx);
    app.caret_line = 5;
    app.caret_col = 28;
    app.refresh_symbol_context();
    focus::settle_frame(app, ctx);
    app.execute_command(ctx, ToolbarAction::FindReferences);
    focus::settle_frame(app, ctx);
    let explicit = app.reference_context.explicit().cloned().unwrap();
    assert_eq!(explicit.name, "Circle");
    let rows = app.reference_context.results.clone();
    assert!(!rows.hits.is_empty());
    app.open_file("src/shapes.rs");
    focus::settle_frame(app, ctx);
    app.caret_line = 8;
    app.caret_col = 12;
    app.refresh_symbol_context();
    focus::settle_frame(app, ctx);
    assert_eq!(
        app.enclosing_source_context().unwrap().last().unwrap().name,
        "area"
    );
    assert_eq!(app.reference_context.target(), Some(&explicit));
    assert!(Arc::ptr_eq(&rows, &app.reference_context.results));
    let source = app.open_buffer.clone().unwrap();
    let history = format!("{:?}", app.history);
    app.handle_inspector_action(ctx, InspectorAction::FollowCaret);
    focus::settle_frame(app, ctx);
    assert!(app.reference_context.explicit().is_none());
    assert_eq!(app.reference_context.target().unwrap().name, "area");
    assert_eq!(
        (
            app.reference_context.results.reference_count,
            app.reference_context.results.definition_count
        ),
        (1, 1)
    );
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &source));
    assert_eq!(format!("{:?}", app.history), history);
}

fn has_signature(output: &egui::FullOutput, signature: &str) -> bool {
    text_shapes(output).any(|text| text.galley.job.text == signature)
}

fn toggle_signature(app: &mut RadiologyApp, ctx: &Context, name: &str) {
    let output = focus::frame(app, ctx, vec![]);
    let (clip, title) = output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.pos.x > 780.0 && text.galley.job.text == name => {
                Some((shape.clip_rect, text))
            }
            _ => None,
        })
        .unwrap();
    // The first control in the actual summary row is the 16-point disclosure.
    let position = egui::pos2(clip.left() + 8.0, title.pos.y + title.galley.size().y * 0.5);
    focus::frame(app, ctx, click(position));
}
