use super::focus::{frame, key, settle_frame};
use super::*;
use crate::critical::{
    overrides::{MarkEdit, MarkTarget},
    Criticality,
};
use crate::semantic::test_support::{document, occurrence};

#[test]
fn noncall_marks_mode_toggles_and_caret_frames_preserve_the_reader_at_both_scales() {
    const PI: &str = "rust-analyzer cargo core . f64/consts/PI.";
    let owner = tempfile::tempdir().unwrap();
    let root = owner.path();
    let text = "// café 😀\nfn demo() { let amount = core::f64::consts::PI; missing(42); }\n";
    std::fs::write(root.join("a.rs"), text).unwrap();
    std::fs::write(root.join("extra.rs"), "fn extra() { absent(42); }").unwrap();
    for scale in [1.0, 2.0] {
        let (_settings, mut app, ctx) = checkout_context(root, "a.rs", scale);
        let source = app.open_buffer.clone().unwrap();
        let byte = text.find("PI").unwrap();
        // Install a decoded UTF-16 index through the existing app field boundary.
        app.semantic_index = Some(Arc::new(SemanticIndex::from_scip(ScipIndex {
            documents: vec![document(
                &source,
                vec![occurrence(&source, byte..byte + 2, PI, false)],
            )],
            ..Default::default()
        })));
        app.navigate(source.location_at_byte(byte), false);
        settle_frame(&mut app, &ctx);
        app.pin_at_byte(byte);
        assert!(app.critical_projection().is_none());
        let history = format!("{:?}", app.history);
        let folds = format!("{:?}", app.fold_store);
        let pins = app.pin_store.groups.clone();
        let caret = (app.caret_line, app.caret_col);
        app.set_source_mode(SourceMode::Critical);
        settle_frame(&mut app, &ctx);
        for (target, decision) in [
            (MarkTarget::Symbol(PI.into()), Some(Criticality::Critical)),
            (MarkTarget::Symbol(PI.into()), None),
            (
                MarkTarget::Package("rust-analyzer cargo core".into()),
                Some(Criticality::Critical),
            ),
        ] {
            let before = app.critical_lens.cache_counts().0;
            assert!(app.apply_critical_mark(MarkEdit { target, decision }));
            settle_frame(&mut app, &ctx);
            assert!(app.critical_lens.cache_counts().0 > before);
            let projection = app.critical_projection().unwrap();
            assert!(projection.matches(&source) && projection.is_receded(byte));
            assert!(!projection.is_receded(text.find("missing").unwrap()));
            assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &source));
            assert_eq!(format!("{:?}", app.history), history);
            assert_eq!(format!("{:?}", app.fold_store), folds);
            assert_eq!(app.pin_store.groups, pins);
            assert_eq!((app.caret_line, app.caret_col), caret);
        }
        let menu = crate::critical::menu::MarkContext {
            index: app.semantic_index.as_ref().unwrap(),
            overrides: app.critical_overrides(),
        }
        .at_byte(&source, byte)
        .unwrap();
        assert_eq!(menu.package_name.as_deref(), Some("core"));
        assert_eq!(
            menu.package_key.as_deref(),
            Some("rust-analyzer cargo core")
        );
        assert_eq!(menu.built_in.criticality, Criticality::Ordinary);
        let counts = app.critical_lens.cache_counts();
        let projection = app.critical_projection().unwrap().clone();
        app.source_viewport.request_focus(&ctx);
        frame(&mut app, &ctx, vec![]);
        for _ in 0..5 {
            frame(
                &mut app,
                &ctx,
                vec![key(egui::Key::ArrowRight, egui::Modifiers::NONE)],
            );
        }
        assert_ne!((app.caret_line, app.caret_col), caret);
        assert_eq!(app.critical_lens.cache_counts(), counts);
        assert_eq!(app.critical_projection(), Some(&projection));
        assert!(!app.history.can_go_back() && !app.history.can_go_forward());
        assert_eq!(
            app.history.current().unwrap().byte_offset,
            source.byte_offset(app.caret_line, app.caret_col)
        );
        let moved_history = format!("{:?}", app.history);
        app.set_source_mode(SourceMode::Normal);
        assert!(app.critical_projection().is_none());
        app.set_source_mode(SourceMode::Critical);
        settle_frame(&mut app, &ctx);
        assert_eq!(app.critical_lens.cache_counts(), (counts.0, counts.1 + 1));
        assert_eq!(format!("{:?}", app.history), moved_history);
        app.flush_critical_marks(std::time::Duration::from_secs(5))
            .unwrap();
        assert!(app.critical_mark_error().is_none());
        app.open_file("extra.rs");
        settle_frame(&mut app, &ctx);
        let extra = app.open_buffer.as_ref().unwrap();
        let name = extra.text.find("absent").unwrap();
        assert!(app.critical_projection().unwrap().matches(extra));
        assert!(!app.critical_projection().unwrap().is_receded(name));
        assert!(app
            .semantic_index
            .as_ref()
            .unwrap()
            .occurrence_at_byte(extra, name)
            .is_none());
        app.open_repository(root);
        settle_frame(&mut app, &ctx);
        assert!(app.critical_projection().is_none());
        assert_eq!(
            app.critical_overrides()
                .packages
                .get("rust-analyzer cargo core"),
            Some(&Criticality::Critical)
        );
        assert_eq!(std::fs::read_to_string(root.join("a.rs")).unwrap(), text);
    }
}

#[test]
fn summary_modes_preserve_loaded_references_and_caret_moves_keep_count_inputs() {
    use crate::ui::reference_list::ReferenceAction;
    let owner = crate::semantic::test_support::scip_rust_fixture();
    for scale in [1.0, 2.0] {
        let (_settings, mut app, ctx) = checkout_context(owner.path(), "src/lib.rs", scale);
        let source = app.open_buffer.clone().unwrap();
        let area = source.text.find("area()").unwrap();
        app.navigate(source.location_at_byte(area), false);
        settle_frame(&mut app, &ctx);
        app.find_references_at_caret();
        settle_frame(&mut app, &ctx);
        let row = app
            .reference_context
            .results
            .hits
            .iter()
            .position(|hit| hit.is_definition)
            .unwrap();
        app.reference_context.select(row);
        app.reference_context.expand(row, true);
        settle_frame(&mut app, &ctx);
        app.definition_expanded = true;
        let references = app.reference_context.results.clone();
        let disclosure = app.reference_context.disclosures[&row].phase().clone();
        let under_caret = app.under_caret.clone();
        let definition = app.definition_identity.clone();
        let history = format!("{:?}", app.history);
        let caret = (app.caret_line, app.caret_col);
        for mode in [
            SourceMode::Critical,
            SourceMode::Normal,
            SourceMode::Critical,
        ] {
            app.set_source_mode(mode);
            settle_frame(&mut app, &ctx);
            assert_eq!(app.reference_context.results, references);
            assert_eq!(app.reference_context.selected, Some(row));
            assert_eq!(app.reference_context.disclosures[&row].phase(), &disclosure);
            assert!(app.definition_expanded);
            assert_eq!(app.under_caret, under_caret);
            assert_eq!(app.definition_identity, definition);
            assert_eq!(format!("{:?}", app.history), history);
            assert_eq!((app.caret_line, app.caret_col), caret);
            assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &source));
            if mode == SourceMode::Normal {
                assert!(app.critical_annotations().is_none());
            } else {
                assert_eq!(
                    app.critical_annotations()
                        .unwrap()
                        .at_line(&source, 5)
                        .unwrap()
                        .count
                        .label(),
                    "1"
                );
            }
        }
        for (decision, expected) in [(Some(Criticality::Ordinary), "0"), (None, "1")] {
            let symbol = app.under_caret.as_ref().unwrap().id.symbol.clone();
            assert!(app.apply_critical_mark(MarkEdit {
                target: MarkTarget::Symbol(symbol),
                decision
            }));
            assert!(app.critical_annotations().is_none());
            settle_frame(&mut app, &ctx);
            assert_eq!(
                app.critical_annotations()
                    .unwrap()
                    .at_line(&source, 5)
                    .unwrap()
                    .count
                    .label(),
                expected
            );
        }
        let inputs = app.critical_graph_inputs.clone().unwrap();
        let counts = app.critical_lens.cache_counts();
        app.source_viewport.request_focus(&ctx);
        frame(&mut app, &ctx, vec![]);
        for _ in 0..2 {
            frame(
                &mut app,
                &ctx,
                vec![key(egui::Key::ArrowRight, egui::Modifiers::NONE)],
            );
        }
        let departure = app
            .open_buffer
            .as_ref()
            .unwrap()
            .location_at_byte(source.byte_offset(app.caret_line, app.caret_col));
        assert!(Arc::ptr_eq(
            app.critical_graph_inputs.as_ref().unwrap(),
            &inputs
        ));
        assert_eq!(app.critical_lens.cache_counts(), counts);
        assert_eq!(app.reference_context.results, references);
        assert_eq!(app.reference_context.selected, Some(row));
        app.handle_reference_action(ReferenceAction::Commit(Some(row)));
        settle_frame(&mut app, &ctx);
        assert_eq!(app.selected_path.as_deref(), Some("src/shapes.rs"));
        app.execute_command(&ctx, ToolbarAction::NavigateBack);
        settle_frame(&mut app, &ctx);
        assert_eq!(
            app.open_buffer
                .as_ref()
                .unwrap()
                .location_at_byte(source.byte_offset(app.caret_line, app.caret_col)),
            departure
        );
        app.open_file("Cargo.toml");
        settle_frame(&mut app, &ctx);
        assert!(app.critical_annotations().is_none());
        app.open_repository(owner.path());
        settle_frame(&mut app, &ctx);
        assert!(app.critical_annotations().is_none() && app.critical_graph_inputs.is_none());
    }
}
