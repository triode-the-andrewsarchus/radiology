//! Exact request retirement and cache ownership through the real app fixture.
use super::context_model::settle_metadata;
use super::*;
use crate::source::preview::{PreviewKind, PreviewPhase, PreviewTarget};
use std::{sync::mpsc, time::Duration};

// The caller retains the second checkout through the following retirement cases.
pub(super) fn assert_owner_changes(
    app: &mut RadiologyApp,
    ctx: &Context,
    root: &Path,
) -> tempfile::TempDir {
    app.open_file("src/shapes.rs");
    focus::settle_frame(app, ctx);
    let rows = load_area_rows(app, ctx);
    let target = app.reference_context.target().cloned().unwrap();
    let source = app.open_buffer.clone().unwrap();
    let old_index = app.semantic_index.clone().unwrap();
    assert!(
        old_index.source_digests.is_empty(),
        "Swift's legacy fixture has no captured source provenance"
    );
    let revision = Arc::new((*old_index).clone());
    app.semantic_index = Some(revision.clone());
    app.refresh_symbol_context();
    assert!(app.reference_context.is_query_running());
    assert!(app.under_caret_job.is_running());
    focus::settle_frame(app, ctx);
    assert_eq!(app.reference_context.target(), Some(&target));
    assert!(!Arc::ptr_eq(&rows, &app.reference_context.results));
    assert!(Arc::ptr_eq(&source, app.open_buffer.as_ref().unwrap()));

    app.caret_line = 5;
    app.caret_col = 6;
    app.refresh_symbol_context();
    focus::settle_frame(app, ctx);
    let before = app.open_buffer.clone().unwrap();
    let original = before.text.clone();
    let before_rows = app.reference_context.results.clone();
    let PreviewPhase::Loaded(preview) = app.definition_preview.phase() else {
        panic!("missing original Circle preview")
    };
    assert_eq!(preview.lines[0].text, "pub struct Circle {");
    let edited = original.replace("pub struct Circle {", "pub struct Circle { // edited");
    std::fs::write(root.join("src/shapes.rs"), &edited).unwrap();
    app.execute_command(ctx, ToolbarAction::ReloadSource);
    focus::settle_frame(app, ctx);
    // Reload retains an absolute byte; Swift's update supplies a newly resolved
    // caret after inserting text on an earlier line. Use that same input here.
    app.caret_line = 5;
    app.caret_col = 6;
    app.refresh_symbol_context();
    settle_metadata(app, ctx);
    assert!(Arc::ptr_eq(app.semantic_index.as_ref().unwrap(), &revision));
    let fresh = app.open_buffer.clone().unwrap();
    assert_ne!(fresh.content_digest(), before.content_digest());
    let PreviewPhase::Loaded(preview) = app.definition_preview.phase() else {
        panic!("missing refreshed Circle preview")
    };
    assert_eq!(preview.lines[0].text, "pub struct Circle { // edited");
    assert_eq!(preview.content_digest, fresh.content_digest());
    assert!(!Arc::ptr_eq(&before_rows, &app.reference_context.results));
    focus::settle_frame(app, ctx);
    let definition = app
        .reference_context
        .results
        .hits
        .iter()
        .find(|hit| hit.is_definition)
        .unwrap();
    assert_eq!(definition.source_line.text, "pub struct Circle { // edited");
    let fresh_rows = app.reference_context.results.clone();
    app.caret_col = 7;
    app.refresh_symbol_context();
    assert!(!app.reference_context.is_query_running());
    assert!(Arc::ptr_eq(&fresh_rows, &app.reference_context.results));
    assert_eq!(
        std::fs::read_to_string(root.join("src/shapes.rs")).unwrap(),
        edited
    );

    // Return original bytes so the next scale and the identical-root test share
    // the unmodified fixture, while exercising actual accepted-source recovery.
    std::fs::write(root.join("src/shapes.rs"), &original).unwrap();
    app.execute_command(ctx, ToolbarAction::ReloadSource);
    focus::settle_frame(app, ctx);
    let rows = load_area_rows(app, ctx);
    let digest = app
        .open_buffer
        .as_ref()
        .unwrap()
        .content_digest()
        .to_owned();
    let other = crate::semantic::test_support::scip_rust_fixture();
    app.open_repository(other.path());
    focus::settle_frame(app, ctx);
    // Reuse the exact producer index object, not just an equal re-import.
    app.semantic_index = Some(revision.clone());
    app.open_file("src/shapes.rs");
    focus::settle_frame(app, ctx);
    load_area_rows(app, ctx);
    assert!(Arc::ptr_eq(app.semantic_index.as_ref().unwrap(), &revision));
    assert_eq!(app.open_buffer.as_ref().unwrap().content_digest(), digest);
    assert_eq!(app.reference_context.target(), Some(&target));
    assert!(!Arc::ptr_eq(&rows, &app.reference_context.results));
    other
}

pub(super) fn assert_retirements(app: &mut RadiologyApp, ctx: &Context, root: &Path, other: &Path) {
    // Swift resets already published facts as well as pending work.
    request_circle(app, ctx);
    focus::settle_frame(app, ctx);
    assert_eq!(app.under_caret.as_ref().unwrap().name, "Circle");
    assert_eq!(
        app.enclosing_source_context().unwrap().last().unwrap().name,
        "total_area"
    );
    assert!(matches!(
        app.definition_preview.phase(),
        PreviewPhase::Loaded(_)
    ));
    assert!(!app.reference_context.results.hits.is_empty());
    app.reset_repository_state();
    assert_reset_context(app);
    for transition in ["caret", "reset", "root", "index"] {
        app.open_repository(root);
        focus::settle_frame(app, ctx);
        request_circle(app, ctx);
        let inputs = app.semantic_inputs.clone().unwrap();
        let old = crate::semantic::context::UnderCaretContext::prepare(
            &inputs.buffer,
            &inputs.index,
            inputs.buffer.byte_offset(5, 28),
            &std::sync::atomic::AtomicBool::new(false),
        )
        .unwrap()
        .unwrap();
        assert_eq!(old.name, "Circle");
        let destination = old.destination.as_ref().unwrap();
        app.definition_preview.request(
            Some(inputs.clone()),
            Some(PreviewTarget {
                path: destination.path.clone(),
                range: destination.range.clone(),
                kind: PreviewKind::Definition,
            }),
        );
        assert!(app.definition_preview.is_running());
        assert!(app.reference_context.is_query_running());
        // Transfer the real publication endpoint out of its worker. This holds
        // completion deterministically without occupying a thread or sleeping.
        let (forward, endpoint) = mpsc::channel();
        app.under_caret_job
            .spawn_with_sender(move |_, sender| forward.send(sender).unwrap());
        let late = endpoint.recv_timeout(Duration::from_secs(5)).unwrap();
        match transition {
            "reset" => app.reset_repository_state(),
            "index" => {
                let mut replacement = (*inputs.index).clone();
                replacement.symbols.get_mut(&old.id).unwrap().display_name = "Fresh Circle".into();
                app.semantic_index = Some(Arc::new(replacement));
                app.refresh_symbol_context();
                assert!(app.under_caret_job.is_running());
                assert!(app.reference_context.is_query_running());
            }
            _ => {
                if transition == "root" {
                    app.open_repository(other);
                    focus::settle_frame(app, ctx);
                }
                app.open_file("src/shapes.rs");
                focus::settle_frame(app, ctx);
                app.caret_line = 8;
                app.caret_col = 9;
                app.refresh_symbol_context();
            }
        }
        focus::settle_frame(app, ctx);
        if transition == "index" {
            assert_eq!(app.under_caret.as_ref().unwrap().name, "Fresh Circle");
        } else {
            assert!(app.under_caret.is_none());
            assert_eq!(app.definition_preview.phase(), &PreviewPhase::None);
            if transition == "reset" {
                assert_reset_context(app);
            } else {
                assert_eq!(app.reference_context.target().unwrap().name, "area");
            }
        }
        let accepted = app.under_caret.clone();
        let preview = app.definition_preview.phase().clone();
        let rows = app.reference_context.results.clone();
        assert!(
            late.send(Ok(Some(old))).is_err(),
            "{transition} retained an old result receiver"
        );
        app.poll_under_caret_context(ctx);
        app.definition_preview.poll();
        app.reference_context.poll();
        assert_eq!(app.under_caret, accepted);
        assert_eq!(app.definition_preview.phase(), &preview);
        assert!(Arc::ptr_eq(&rows, &app.reference_context.results));
        if transition == "reset" {
            assert_reset_context(app);
        }
    }
}

fn load_area_rows(
    app: &mut RadiologyApp,
    ctx: &Context,
) -> Arc<crate::semantic::reference_results::ReferenceResults> {
    app.caret_line = 8;
    app.caret_col = 12;
    app.refresh_symbol_context();
    focus::settle_frame(app, ctx);
    assert_eq!(app.reference_context.target().unwrap().name, "area");
    let rows = app.reference_context.results.clone();
    assert_eq!((rows.reference_count, rows.definition_count), (1, 1));
    rows
}

fn assert_reset_context(app: &RadiologyApp) {
    assert!(app.under_caret.is_none());
    assert!(app.enclosing_source_context().is_none());
    assert_eq!(app.definition_preview.phase(), &PreviewPhase::None);
    assert!(!app.reference_context.available());
    assert!(app.reference_context.results.hits.is_empty());
    assert!(app.reference_context.explicit().is_none());
}

fn request_circle(app: &mut RadiologyApp, ctx: &Context) {
    app.open_file("src/lib.rs");
    focus::settle_frame(app, ctx);
    app.caret_line = 5;
    app.caret_col = 28;
    app.refresh_symbol_context();
}
