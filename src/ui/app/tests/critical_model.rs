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
