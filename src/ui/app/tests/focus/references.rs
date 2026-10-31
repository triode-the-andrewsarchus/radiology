use super::*;
use crate::ui::reference_list::{ReferenceAction, ReferenceViewport};

pub(super) fn caller_reader() -> (tempfile::TempDir, RadiologyApp, Context) {
    let (dir, mut app) = reader_with_reference_index();
    let buffer = SourceBuffer::from_str(
        "b.rs",
        "fn beta() {\n    alpha();\n}\nfn entry() {\n    beta();\n}\n",
    );
    std::fs::write(dir.path().join("repo/b.rs"), &buffer.text).unwrap();
    let mut documents: Vec<_> = reference_index().documents.values().cloned().collect();
    let document = documents
        .iter_mut()
        .find(|doc| doc.relative_path == "b.rs")
        .unwrap();
    for (name, definition, byte) in [
        ("entry", true, buffer.text.find("entry").unwrap()),
        ("beta", false, buffer.text.rfind("beta").unwrap()),
    ] {
        document
            .occurrences
            .push(crate::semantic::test_support::occurrence(
                &buffer,
                byte..byte + name.len(),
                &format!("scip test fixture 1 {name}()."),
                definition,
            ));
    }
    app.semantic_index = Some(Arc::new(SemanticIndex::from_scip(ScipIndex {
        documents,
        ..Default::default()
    })));
    app.find_references_at_caret();
    settle(&mut app);
    app.handle_reference_action(ReferenceAction::SetExpanded(1, true));
    settle(&mut app);
    let tree = app.reference_context.disclosures[&1].callers().unwrap();
    assert_eq!(
        tree.rows
            .iter()
            .map(|row| row.name.as_str())
            .collect::<Vec<_>>(),
        ["beta", "entry"]
    );
    let ctx = Context::default();
    frame(&mut app, &ctx, vec![]);
    ctx.memory_mut(|memory| memory.request_focus(ReferenceViewport::focus_id()));
    frame(&mut app, &ctx, vec![]);
    (dir, app, ctx)
}

fn reference_keys(app: &mut RadiologyApp, ctx: &Context, keys: &[Key], prepared: bool) {
    let events = keys.iter().map(|&k| key(k, Modifiers::NONE)).collect();
    if prepared {
        let _ = ctx.run(
            egui::RawInput {
                events,
                ..Default::default()
            },
            |ctx| app.handle_shortcuts(ctx),
        );
    } else {
        frame(app, ctx, events);
    }
}

#[test]
fn coalesced_reference_to_caller_keys_commit_the_caller_and_preserve_departure_history() {
    for prepared in [false, true] {
        let (_dir, mut app, ctx) = caller_reader();
        reference_keys(
            &mut app,
            &ctx,
            &[Key::ArrowRight, Key::ArrowDown, Key::Enter],
            prepared,
        );
        settle(&mut app);
        assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
        assert_eq!(
            (app.caret_line, app.caret_col),
            (5, 4),
            "prepared={prepared}"
        );
        app.execute_command(&ctx, ToolbarAction::NavigateBack);
        settle(&mut app);
        assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
        assert_eq!((app.caret_line, app.caret_col), (1, 4));
    }
}

#[test]
fn caller_exit_routes_the_remaining_batch_to_reference_selection() {
    for exit in [Key::Escape, Key::ArrowLeft] {
        for prepared in [false, true] {
            let (_dir, mut app, ctx) = caller_reader();
            reference_keys(&mut app, &ctx, &[Key::ArrowRight], prepared);
            frame(&mut app, &ctx, vec![]);
            reference_keys(
                &mut app,
                &ctx,
                &[exit, Key::ArrowDown, Key::Enter],
                prepared,
            );
            settle(&mut app);
            assert_eq!(
                (app.caret_line, app.caret_col),
                (2, 4),
                "exit={exit:?}, prepared={prepared}"
            );
        }
    }
}

#[test]
fn collapse_then_right_reopens_without_focusing_a_retired_caller() {
    let (_dir, mut app, ctx) = caller_reader();
    reference_keys(&mut app, &ctx, &[Key::ArrowRight], false);
    reference_keys(
        &mut app,
        &ctx,
        &[Key::ArrowLeft, Key::ArrowRight, Key::ArrowRight],
        false,
    );
    assert!(app.reference_context.disclosures.contains_key(&1));
    settle(&mut app);
    assert!(ctx.memory(|memory| memory.has_focus(ReferenceViewport::focus_id())));
}

#[test]
fn reference_keys_do_not_take_the_live_source_find_query() {
    let (_dir, mut app, ctx) = caller_reader();
    reference_keys(&mut app, &ctx, &[Key::ArrowRight], false);
    frame(&mut app, &ctx, vec![key(Key::F, Modifiers::CTRL)]);
    frame(&mut app, &ctx, vec![Event::Text("alpha".into())]);
    assert!(ctx.wants_keyboard_input());
    frame(
        &mut app,
        &ctx,
        vec![
            key(Key::ArrowDown, Modifiers::NONE),
            key(Key::Enter, Modifiers::NONE),
            key(Key::Escape, Modifiers::NONE),
        ],
    );
    settle(&mut app);
    assert_eq!(app.reference_context.selected, Some(1));
    assert!(app.reference_context.disclosures.contains_key(&1));
    assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
}

#[test]
fn pinned_metadata_uses_the_captured_local_document_and_current_index() {
    let (dir, mut app) = opened_repository();
    let buffers = [
        SourceBuffer::from_str("a.rs", "fn main() {\n    let local_value = 1;\n}\n"),
        SourceBuffer::from_str("b.rs", "fn main(argument: u8) {}\n"),
    ];
    let mut documents = Vec::new();
    for (buffer, name, kind) in [
        (&buffers[0], "local_value", 61),
        (&buffers[1], "argument", 37),
    ] {
        std::fs::write(dir.path().join(&buffer.path), &buffer.text).unwrap();
        let byte = buffer.text.find(name).unwrap();
        documents.push(ScipDocument {
            relative_path: buffer.path.clone(),
            occurrences: vec![crate::semantic::test_support::occurrence(
                buffer,
                byte..byte + name.len(),
                "local 0",
                true,
            )],
            symbols: vec![crate::semantic::ScipSymbolInformation {
                symbol: "local 0".into(),
                display_name: name.into(),
                kind,
                ..Default::default()
            }],
            ..Default::default()
        });
    }
    app.open_file("a.rs");
    settle(&mut app);
    app.execute_command(&Context::default(), ToolbarAction::ReloadSource);
    settle(&mut app);
    app.semantic_index = Some(Arc::new(SemanticIndex::from_scip(ScipIndex {
        documents: documents.clone(),
        ..Default::default()
    })));
    app.jump_to_location("a.rs", 2, buffers[0].text.find("local_value").unwrap());
    settle(&mut app);
    app.find_references_at_caret();
    settle(&mut app);
    let ctx = Context::default();
    for (path, line, byte) in [
        ("a.rs", 2, buffers[0].text.find("local_value").unwrap()),
        ("b.rs", 1, buffers[1].text.find("argument").unwrap()),
    ] {
        app.jump_to_location(path, line, byte);
        settle(&mut app);
        let output = frame(&mut app, &ctx, vec![]);
        assert!(crate::ui::test_support::text_shapes(&output)
            .any(|text| text.galley.job.text == "Variable" && text.pos.y < 120.0));
    }
    assert_eq!(
        app.reference_context
            .target()
            .unwrap()
            .id
            .document
            .as_deref(),
        Some("a.rs")
    );
    documents[0].symbols[0].kind = 9001;
    app.semantic_index = Some(Arc::new(SemanticIndex::from_scip(ScipIndex {
        documents,
        ..Default::default()
    })));
    app.refresh_symbol_context();
    settle(&mut app);
    let output = frame(&mut app, &ctx, vec![]);
    assert!(crate::ui::test_support::text_shapes(&output)
        .any(|text| text.galley.job.text == "Kind 9001" && text.pos.y < 120.0));
    app.follow_reference_caret();
    settle(&mut app);
    let output = frame(&mut app, &ctx, vec![]);
    assert!(!crate::ui::test_support::text_shapes(&output)
        .any(|text| text.galley.job.text == "Kind 9001"));
}
