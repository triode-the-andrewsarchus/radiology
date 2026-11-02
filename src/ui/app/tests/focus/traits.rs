//! Real app lifecycle, pane, input and source-loader acceptance for trait rows.
use super::*;
use crate::ui::{
    test_support::{click, text_center, text_shapes},
    trait_implementations::{TraitAction, TraitImplementationList},
};

fn trait_reader() -> (tempfile::TempDir, RadiologyApp, Context) {
    let (owner, root) = crate::semantic::trait_impls::tests::imported_fixture();
    let mut app = RadiologyApp::from_session(SessionStore::load_from(
        owner.path().join("settings/session.json"),
    ));
    app.open_repository(&root);
    settle(&mut app);
    app.open_file("src/lib.rs");
    settle(&mut app);
    caret_text(&mut app, "fn render");
    let ctx = Context::default();
    frame(&mut app, &ctx, vec![]);
    (owner, app, ctx)
}

fn caret_text(app: &mut RadiologyApp, needle: &str) {
    let buffer = app.open_buffer.as_ref().unwrap();
    let byte = buffer.text.find(needle).unwrap();
    (app.caret_line, app.caret_col) = (buffer.line_at_byte(byte), buffer.column_at_byte(byte));
    app.refresh_symbol_context();
    settle(app);
}

fn labels(output: &egui::FullOutput) -> Vec<String> {
    text_shapes(output)
        .map(|shape| shape.galley.job.text.clone())
        .collect()
}

#[test]
fn trait_rows_primary_click_keys_and_back_preserve_exact_source_departure() {
    for scale in [1.0, 2.0] {
        let (_owner, mut app, ctx) = trait_reader();
        ctx.set_pixels_per_point(scale);
        frame(&mut app, &ctx, vec![]);
        let output = frame(&mut app, &ctx, vec![]);
        assert_eq!(app.trait_context.rows().len(), 3);
        assert!(
            labels(&output).contains(&"Trait Implementations".into()),
            "scale {scale}: {:?}",
            labels(&output)
        );
        assert!(!labels(&output).contains(&"Definition".into()));
        let departure = app.open_buffer.as_ref().unwrap().location_at_byte(
            app.open_buffer
                .as_ref()
                .unwrap()
                .byte_offset(app.caret_line, app.caret_col),
        );
        let point = text_center(&output, "Alpha");
        frame(&mut app, &ctx, click(point));
        settle(&mut app);
        assert_eq!(app.selected_path.as_deref(), Some("src/implementations.rs"));
        assert_eq!(app.trait_context.selected, Some(0));
        assert_eq!(
            app.open_buffer
                .as_ref()
                .unwrap()
                .byte_offset(app.caret_line, app.caret_col),
            app.trait_context.rows()[0].byte_range.start
        );
        assert!(TraitImplementationList::has_focus(&ctx));
        let before = (app.caret_line, app.caret_col);
        frame(&mut app, &ctx, vec![key(Key::ArrowDown, Modifiers::NONE)]);
        assert_eq!(app.trait_context.selected, Some(1));
        assert_eq!((app.caret_line, app.caret_col), before);
        frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
        settle(&mut app);
        assert_eq!(app.trait_context.selected, Some(1));
        assert_eq!(
            app.open_buffer
                .as_ref()
                .unwrap()
                .byte_offset(app.caret_line, app.caret_col),
            app.trait_context.rows()[1].byte_range.start
        );
        app.execute_command(&ctx, ToolbarAction::NavigateBack);
        settle(&mut app);
        app.execute_command(&ctx, ToolbarAction::NavigateBack);
        settle(&mut app);
        assert_eq!(app.selected_path.as_deref(), Some("src/lib.rs"));
        assert_eq!(
            app.open_buffer
                .as_ref()
                .unwrap()
                .byte_offset(app.caret_line, app.caret_col),
            departure.byte_offset
        );
        let output = frame(&mut app, &ctx, vec![]);
        let point = text_center(&output, "Alpha");
        frame(
            &mut app,
            &ctx,
            crate::ui::test_support::secondary_click(point),
        );
        let menu = frame(&mut app, &ctx, vec![]);
        assert!(labels(&menu).contains(&"Go to Trait Implementation".into()));
        assert_eq!(app.selected_path.as_deref(), Some("src/lib.rs"));
        frame(&mut app, &ctx, vec![key(Key::Home, Modifiers::NONE)]);
        frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
        settle(&mut app);
        assert_eq!(app.selected_path.as_deref(), Some("src/implementations.rs"));
        assert_eq!(app.trait_context.selected, Some(0));
        app.execute_command(&ctx, ToolbarAction::NavigateBack);
        settle(&mut app);
        frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
        assert!(app.trait_context.selected.is_none());
    }
}

#[test]
fn trait_context_retains_only_same_trait_owner_and_empty_negative_contexts_remain_visible() {
    let (_owner, mut app, ctx) = trait_reader();
    app.trait_context.select(Some(2));
    caret_text(&mut app, "{ 1 }");
    assert_eq!(app.trait_context.selected, Some(2));
    let mut new_index = app.semantic_index.as_ref().unwrap().as_ref().clone();
    new_index.source_digests.clear();
    app.semantic_index = Some(Arc::new(new_index));
    app.refresh_symbol_context();
    assert!(app.trait_context.selected.is_none());
    caret_text(&mut app, "Empty {}");
    assert!(app
        .trait_context
        .target()
        .unwrap()
        .symbol
        .ends_with("left/Empty#"));
    assert!(app.trait_context.rows().is_empty());
    let output = frame(&mut app, &ctx, vec![]);
    assert!(labels(&output).contains(&"No indexed implementations".into()));
    caret_text(&mut app, "Marker {}");
    let output = frame(&mut app, &ctx, vec![]);
    assert_eq!(app.trait_context.rows().len(), 1);
    assert!(labels(&output).contains(&"negative".into()));
    let mut before = app.open_buffer.as_ref().unwrap().as_ref().clone();
    before.revision = Some(ReviewRevision::Before);
    app.open_buffer = Some(Arc::new(before));
    app.refresh_symbol_context();
    assert!(app.trait_context.target().is_none());
    assert!(app.trait_context.rows().is_empty());
}

#[test]
fn trait_activation_rejects_changed_bytes_and_retires_index_and_row_changes_without_history() {
    let (_owner, mut app, ctx) = trait_reader();
    let root = app.repository_root.clone().unwrap();
    let path = root.join("src/implementations.rs");
    let original = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, original.replace("Alpha", "Gamma")).unwrap();
    let source = app.open_buffer.clone().unwrap();
    let history = format!("{:?}", app.history);
    app.handle_trait_action(TraitAction::Commit(0));
    settle(&mut app);
    assert!(Arc::ptr_eq(&source, app.open_buffer.as_ref().unwrap()));
    assert_eq!(format!("{:?}", app.history), history);
    assert!(app
        .trait_context
        .navigation_error
        .as_ref()
        .unwrap()
        .contains("source bytes changed"));
    std::fs::write(&path, &original).unwrap();
    app.handle_trait_action(TraitAction::Commit(0));
    app.handle_trait_action(TraitAction::Select(Some(1)));
    assert!(app.pending_navigation.is_none());
    settle(&mut app);
    assert_eq!(format!("{:?}", app.history), history);
    app.handle_trait_action(TraitAction::Commit(1));
    app.semantic_index = Some(Arc::new(
        app.semantic_index.as_ref().unwrap().as_ref().clone(),
    ));
    app.refresh_symbol_context();
    assert!(app.pending_navigation.is_none());
    settle(&mut app);
    assert!(Arc::ptr_eq(&source, app.open_buffer.as_ref().unwrap()));
    assert_eq!(format!("{:?}", app.history), history);
    app.handle_trait_action(TraitAction::Commit(0));
    settle(&mut app);
    assert!(app.trait_context.navigation_error.is_none());
    assert_eq!(app.selected_path.as_deref(), Some("src/implementations.rs"));
    let displayed = app.open_buffer.clone().unwrap();
    let current_history = format!("{:?}", app.history);
    std::fs::write(&path, original.replace("Alpha", "Gamma")).unwrap();
    app.handle_trait_action(TraitAction::Commit(1));
    settle(&mut app);
    assert!(Arc::ptr_eq(&displayed, app.open_buffer.as_ref().unwrap()));
    assert_eq!(format!("{:?}", app.history), current_history);
    assert!(app.trait_context.navigation_error.is_some());
    frame(&mut app, &ctx, vec![]);
}

#[test]
fn trait_pane_preferences_keep_selection_and_retire_hidden_keyboard_owner() {
    use crate::session::inspector::InspectorPane;
    let (_owner, mut app, ctx) = trait_reader();
    app.trait_context.select(Some(1));
    for mask in 0..16 {
        super::panes::set_mask(&mut app, &ctx, mask);
        frame(&mut app, &ctx, vec![]);
        let output = frame(&mut app, &ctx, vec![]);
        let visible = labels(&output);
        let panes = app.session.inspector_panes;
        assert_eq!(
            visible.contains(&"Trait Implementations".into()),
            panes.implementations,
            "mask {mask}: {visible:?}"
        );
        assert_eq!(
            visible.contains(&"Definition".into()) || visible.contains(&"Outline".into()),
            panes.definition && !panes.implementations
        );
        assert_eq!(app.trait_context.selected, Some(1));
    }
    app.execute_command(&ctx, ToolbarAction::ShowAllInspectorPanes);
    ctx.memory_mut(|memory| memory.request_focus(TraitImplementationList::focus_id()));
    app.execute_command(
        &ctx,
        ToolbarAction::ToggleInspectorPane(InspectorPane::Implementations),
    );
    assert!(!TraitImplementationList::has_focus(&ctx));
    frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
    assert_eq!(app.trait_context.selected, Some(1));
    assert!(app.pending_navigation.is_none());
}

#[test]
fn trait_activation_reads_pinned_after_objects_and_before_clears_the_pane() {
    let before = "trait Trait {}\nimpl Trait for Before {}\n";
    let after = "trait Trait {}\nimpl Trait for After {}\n";
    let (_owner, root, review) =
        crate::pull_request::review::tests::fixture_with_sources(before, after);
    let (_settings, mut app) =
        super::super::review_refresh::open_review(&root, review, ReviewRevision::After);
    let buffer = app.open_buffer.as_ref().unwrap();
    let start = after.rfind("Trait").unwrap();
    let mut index = SemanticIndex::from_scip(ScipIndex {
        documents: vec![ScipDocument {
            relative_path: "a file.rs".into(),
            position_encoding: 2,
            occurrences: vec![crate::semantic::test_support::occurrence(
                buffer,
                start..start + 5,
                "test Trait#",
                false,
            )],
            ..Default::default()
        }],
        ..Default::default()
    });
    index
        .source_digests
        .insert(buffer.path.clone(), buffer.content_digest().into());
    index.trait_implementations = Arc::new(
        crate::semantic::trait_impls::TraitImplementationIndex::build(
            &index,
            &crate::source::loader::SourceRead::Review(app.review.clone().unwrap()),
            Some(&["a file.rs".into()]),
            None,
            &std::collections::HashSet::new(),
            &std::sync::atomic::AtomicBool::new(false),
        )
        .unwrap(),
    );
    app.semantic_index = Some(Arc::new(index));
    caret_text(&mut app, "impl");
    assert_eq!(app.trait_context.rows()[0].target_type, "After");
    std::fs::write(root.join("a file.rs"), "changed checkout bytes").unwrap();
    std::fs::write(
        app.review.as_ref().unwrap().head_root.join("a file.rs"),
        "changed materialization bytes",
    )
    .unwrap();
    app.handle_trait_action(TraitAction::Commit(0));
    settle(&mut app);
    assert_eq!(app.open_buffer.as_ref().unwrap().text, after);
    assert_eq!(app.active_revision, Some(ReviewRevision::After));
    let ctx = Context::default();
    app.execute_command(&ctx, ToolbarAction::ToggleRevision(ReviewRevision::Before));
    settle(&mut app);
    assert_eq!(app.open_buffer.as_ref().unwrap().text, before);
    assert!(app.trait_context.target().is_none());
}

#[cfg(unix)]
#[test]
fn trait_activation_cannot_follow_an_external_checkout_symlink_even_with_identical_bytes() {
    let (owner, mut app, _ctx) = trait_reader();
    let source = app.open_buffer.clone().unwrap();
    let history = format!("{:?}", app.history);
    let inside = app
        .repository_root
        .as_ref()
        .unwrap()
        .join("src/implementations.rs");
    let outside = owner.path().join("outside.rs");
    std::fs::copy(&inside, &outside).unwrap();
    std::fs::remove_file(&inside).unwrap();
    std::os::unix::fs::symlink(&outside, &inside).unwrap();
    app.handle_trait_action(TraitAction::Commit(0));
    settle(&mut app);
    assert!(app
        .trait_context
        .navigation_error
        .as_ref()
        .unwrap()
        .contains("leaves the repository"));
    assert!(Arc::ptr_eq(&source, app.open_buffer.as_ref().unwrap()));
    assert_eq!(format!("{:?}", app.history), history);
}
