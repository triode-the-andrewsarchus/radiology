//! Recorded Swift compiler output flows through pinned generation and actual app frames.
use super::focus::{frame, key};
use super::*;
use crate::pull_request::review::{tests::git, ReviewSnapshot};
use crate::semantic::indexing::tools::ToolDirectory;
use egui::{Event, Key, Modifiers};
use std::os::unix::fs::PermissionsExt;

const READER: &str = "src/main/kotlin/Reader.kt";
const STORE: &str = "src/main/kotlin/Store.kt";
const INPUTS: [&str; 5] = [
    "settings.gradle.kts",
    "build.gradle.kts",
    READER,
    STORE,
    "src/main/kotlin/Noise.kt",
];

#[test]
fn tracked_kotlin_dependency_outside_browser_inventory_is_copied_and_invalidates_cache() {
    let (_owner, mut app) = kotlin_review(true);
    let root = app.repository_root.clone().unwrap();
    let head = app.review.as_ref().unwrap().head_root.clone();
    git(&root, &["rm", "-rf", "."]);
    for path in INPUTS {
        std::fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
        std::fs::copy(head.join(path), root.join(path)).unwrap();
    }
    let dependency = root.join("vendor/dependency/input.txt");
    std::fs::create_dir_all(dependency.parent().unwrap()).unwrap();
    std::fs::write(&dependency, "first\n").unwrap();
    git(&root, &["add", "-f", "."]);
    git(&root, &["commit", "-m", "Tracked Kotlin dependency"]);
    let bin = &app.index_tools.search_path[0];
    let observed = bin.join("copied-dependency");
    std::fs::write(bin.join("gradle"),
        "#!/bin/sh\nset -eu\n/bin/cat vendor/dependency/input.txt >> \"$(/usr/bin/dirname \"$0\")/copied-dependency\"\n").unwrap();
    let runs = bin.join("runs");
    app.open_repository(&root);
    settle(&mut app);
    assert!(app.review.is_none());
    assert!(!app
        .all_files
        .iter()
        .any(|file| file.path.starts_with("vendor/")));
    assert_eq!(std::fs::read_to_string(&observed).unwrap(), "first\n");
    std::fs::write(&dependency, "second version\n").unwrap();
    app.load_repository_index();
    settle(&mut app);
    let report = app.generation_report().unwrap();
    assert!(!report.cached && report.documents == 3);
    assert!(report
        .outcomes
        .iter()
        .all(|outcome| outcome.failure.is_none() && outcome.reused_from.is_none()));
    assert_eq!(
        std::fs::read_to_string(&observed).unwrap(),
        "first\nsecond version\n"
    );
    assert_eq!(std::fs::read_to_string(runs).unwrap().lines().count(), 3);
    assert_eq!(
        std::fs::read_to_string(dependency).unwrap(),
        "second version\n"
    );
    assert!(app.semantic_index.as_ref().unwrap().matches_buffer(
        &SourceBuffer::from_bytes(READER, std::fs::read(head.join(READER)).unwrap(), None).unwrap()
    ));
    for path in INPUTS {
        assert_eq!(
            std::fs::read(root.join(path)).unwrap(),
            std::fs::read(head.join(path)).unwrap()
        );
    }
    assert_eq!(
        git(&root, &["status", "--porcelain"]),
        "M vendor/dependency/input.txt"
    );
}

#[test]
fn explicit_kotlin_checkout_reuse_keeps_partial_utf16_and_survives_source_cache_removal() {
    use crate::semantic::indexing::service::{GenerationRequest, IndexGenerator, IndexReuseSource};
    use std::time::{Duration, Instant};
    let (owner, app) = kotlin_review(false);
    let attached = app.repository_root.as_ref().unwrap();
    let source_cache = app
        .session
        .repository_cache(attached)
        .unwrap()
        .join("indexes");
    let root = owner.path().join("independent checkout");
    let head = &app.review.as_ref().unwrap().head_root;
    for path in ["build.gradle.kts", "settings.gradle.kts", READER, STORE] {
        let destination = root.join(path);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::copy(head.join(path), destination).unwrap();
    }
    let runs = app.index_tools.search_path[0].join("runs");
    let mut request = GenerationRequest {
        attached_root: root.clone(),
        root,
        reuse: true,
        reuse_sources: vec![IndexReuseSource {
            root: attached.clone(),
            cache: source_cache.clone(),
            label: "original Kotlin".into(),
        }],
        settings: app.index_personality.settings.clone(),
        review: None,
        cache: owner.path().join("independent cache"),
        tools: app.index_tools.clone(),
        timeout: Duration::from_secs(10),
        limits: Default::default(),
    };
    let mut generator = IndexGenerator::default();
    for cached in [false, true] {
        if cached {
            std::fs::remove_dir_all(&source_cache).unwrap();
            request.reuse_sources.clear();
            request.tools = ToolDirectory {
                managed: None,
                home: None,
                search_path: vec![],
            };
            generator.start_cached(request.clone());
        } else {
            generator.start(request.clone());
        }
        let deadline = Instant::now() + Duration::from_secs(10);
        let generated = loop {
            if let Some(generated) = generator.poll() {
                break generated;
            }
            assert!(Instant::now() < deadline, "{:?}", generator.error());
            std::thread::sleep(Duration::from_millis(5));
        };
        assert_eq!(generated.report.cached, cached);
        assert_eq!(generated.report.documents, 2);
        let outcome = &generated.report.outcomes[0];
        assert!(outcome.failure.is_none() && outcome.partial_build);
        assert_eq!(outcome.reused_from.as_deref(), Some("original Kotlin"));
        let index = generated.index.unwrap();
        assert!(!index.documents.contains_key("src/main/kotlin/Noise.kt"));
        assert_eq!(
            index.column_encoding(READER),
            Some(crate::source::ColumnEncoding::Utf16)
        );
        let buffer = app.open_buffer.as_ref().unwrap();
        let call = index
            .occurrence_at_byte(buffer, buffer.text.find("read(2)").unwrap() + 1)
            .unwrap();
        let (path, range) = index.find_definition(&call.symbol).unwrap();
        assert_eq!((path, range.start_line), (STORE, 3));
        assert_eq!(std::fs::read_to_string(&runs).unwrap().lines().count(), 1);
    }
}

fn kotlin_review(noise: bool) -> (tempfile::TempDir, RadiologyApp) {
    let (owner, root, old) = crate::pull_request::review::tests::fixture();
    let fixtures =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("Tests/RadiologyTests/Fixtures/scip-kotlin");
    git(&root, &["restore", "."]);
    git(&root, &["checkout", "review"]);
    git(&root, &["rm", "-rf", "."]);
    for path in INPUTS {
        if !noise && path.ends_with("Noise.kt") {
            continue;
        }
        let text = std::fs::read_to_string(fixtures.join(path)).unwrap();
        std::fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
        std::fs::write(
            root.join(path),
            if path == READER {
                text.replace("read(2)", "read(1)")
            } else if path == STORE {
                text.replace("item:$id", "previous:$id")
            } else {
                text
            },
        )
        .unwrap();
    }
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-m", "Kotlin base"]);
    let mut summary = old.summary.clone();
    summary.base_ref_oid = Some(git(&root, &["rev-parse", "HEAD"]));
    std::fs::copy(fixtures.join(READER), root.join(READER)).unwrap();
    std::fs::copy(fixtures.join(STORE), root.join(STORE)).unwrap();
    git(&root, &["commit", "-am", "Kotlin argument change"]);
    summary.head_ref_oid = Some(git(&root, &["rev-parse", "HEAD"]));
    git(&root, &["checkout", "main"]);
    std::fs::create_dir_all(root.join(READER).parent().unwrap()).unwrap();
    std::fs::write(root.join(READER), "// checkout only\n").unwrap();
    let review = ReviewSnapshot::prepare(&root, summary, &owner.path().join("cache")).unwrap();
    let tools = owner.path().join("tools");
    let bin = tools.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::create_dir(tools.join("lib")).unwrap();
    std::fs::copy(fixtures.join("index.scip"), bin.join("fixture.scip")).unwrap();
    std::fs::write(
        tools.join("lib/scip-java-1.jar"),
        include_bytes!("../../../../Tests/RadiologyTests/Fixtures/kotlin-sdk-plugins.zip"),
    )
    .unwrap();
    for (name, script) in [
        ("scip-java", "#!/bin/sh\nset -eu\nif [ \"$1\" = --version ]; then echo 'scip-java recorded-compiler-fixture'; exit 0; fi\n[ \"$1\" = aggregate ]\n[ \"$2\" = --output ]\n/bin/cat \"$(/usr/bin/dirname \"$0\")/fixture.scip\" > \"$3\"\necho aggregate >> \"$(/usr/bin/dirname \"$0\")/runs\"\n".to_owned()),
        ("gradle", format!("#!/bin/sh\nexit {}\n", i32::from(!noise))),
    ] {
        std::fs::write(bin.join(name), script).unwrap();
        std::fs::set_permissions(bin.join(name), std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let mut app = RadiologyApp::from_session(SessionStore::load_from(
        owner.path().join("settings/session.json"),
    ));
    app.index_tools = ToolDirectory {
        managed: Some(tools),
        home: None,
        search_path: vec![bin, "/usr/bin".into(), "/bin".into()],
    };
    app.open_repository(&root);
    settle(&mut app);
    app.pending_open_file = Some(READER.into());
    app.accept_review(review);
    settle(&mut app);
    assert!(
        app.semantic_index.is_some(),
        "{:?}",
        app.generation_report()
    );
    (owner, app)
}

#[test]
fn recorded_kotlin_overloads_keep_utf16_pr_navigation_references_and_cached_revision_scope() {
    use crate::semantic::availability::SemanticAvailability;
    use crate::ui::reference_list::ReferenceAction;
    for scale in [1.0, 2.0] {
        for noise in [false, true] {
            let (owner, mut app) = kotlin_review(noise);
            let root = app.repository_root.clone().unwrap();
            let status = git(&root, &["status", "--porcelain"]);
            let head = git(&root, &["rev-parse", "HEAD"]);
            let ctx = Context::default();
            ctx.set_pixels_per_point(scale);
            if !noise {
                app.execute_command(&ctx, ToolbarAction::ToggleTheme);
            }
            let tools = app.index_tools.clone();
            let runs = tools.search_path[0].join("runs");
            let report = app.generation_report().unwrap();
            assert_eq!(report.documents, if noise { 3 } else { 2 });
            assert_eq!(report.outcomes[0].partial_build, !noise);
            assert_eq!(
                app.semantic_index
                    .as_ref()
                    .unwrap()
                    .documents
                    .contains_key("src/main/kotlin/Noise.kt"),
                noise
            );
            app.execute_command(&ctx, ToolbarAction::ToggleRevision(ReviewRevision::Before));
            settle(&mut app);
            app.open_file(STORE);
            settle(&mut app);
            assert_eq!(app.active_revision, Some(ReviewRevision::After));
            app.open_file(READER);
            settle(&mut app);
            assert_eq!(
                app.active_revision,
                Some(ReviewRevision::Before),
                "reopening a changed file must retain its chosen side"
            );
            app.execute_command(&ctx, ToolbarAction::ToggleRevision(ReviewRevision::After));
            settle(&mut app);
            let buffer = app.open_buffer.clone().unwrap();
            assert!(buffer.text.contains("café 🚀"));
            assert!(app
                .line_changes
                .as_ref()
                .is_some_and(|changes| changes.total_additions() == 1));
            assert_eq!(
                app.semantic_index.as_ref().unwrap().column_encoding(READER),
                Some(crate::source::ColumnEncoding::Utf16)
            );
            let index = app.semantic_index.as_ref().unwrap();
            assert!(index.documents[READER]
                .occurrences
                .iter()
                .filter_map(|occurrence| occurrence.range.as_ref())
                .all(|range| index.byte_range(&buffer, range).is_some()));
            let mut identities = Vec::new();
            for (needle, definition_line, definition_name) in [
                ("read(2)", 4, "read(id"),
                ("read(\"name\")", 5, "read(name"),
            ] {
                let byte = buffer.text.find(needle).unwrap();
                app.open_file(STORE);
                settle(&mut app);
                app.execute_command(&ctx, ToolbarAction::ToggleRevision(ReviewRevision::Before));
                settle(&mut app);
                assert_eq!(app.active_revision, Some(ReviewRevision::Before));
                app.history.clear();
                app.navigate(
                    SourceLocation::with_revision(READER, byte, Some(ReviewRevision::After)),
                    true,
                );
                settle(&mut app);
                app.source_viewport.request_focus(&ctx);
                frame(&mut app, &ctx, vec![]);
                let previous_back = app.history.clone().back();
                frame(&mut app, &ctx, vec![key(Key::ArrowRight, Modifiers::NONE)]);
                settle(&mut app);
                let departure = buffer.location_at_byte(byte + 1);
                assert_eq!(
                    buffer.byte_offset(app.caret_line, app.caret_col),
                    departure.byte_offset
                );
                assert_eq!(app.history.clone().back(), previous_back);
                let under = app.under_caret.as_ref().unwrap();
                assert_eq!(under.name, "read");
                assert_eq!(under.occurrence, byte..byte + 4);
                assert_eq!(under.destination.as_ref().unwrap().path, STORE);
                identities.push(under.id.clone());
                frame(
                    &mut app,
                    &ctx,
                    vec![key(
                        Key::R,
                        Modifiers {
                            ctrl: true,
                            alt: true,
                            ..Default::default()
                        },
                    )],
                );
                settle(&mut app);
                assert_eq!(app.reference_context.explicit().unwrap().name, "read");
                let results = &app.reference_context.results;
                assert_eq!(
                    (
                        results.reference_count,
                        results.definition_count,
                        results.unmapped
                    ),
                    (1, 1, 0)
                );
                assert!(results.unreadable.is_empty());
                assert!(results
                    .hits
                    .iter()
                    .all(|hit| !hit.path.ends_with("Noise.kt")));
                let definition = results
                    .hits
                    .iter()
                    .position(|hit| hit.is_definition)
                    .unwrap();
                // Match Swift's stored-Before preference without changing the displayed After reader.
                app.review_revision_choices
                    .insert(READER.into(), ReviewRevision::Before);
                frame(&mut app, &ctx, vec![key(Key::F12, Modifiers::NONE)]);
                settle(&mut app);
                assert_eq!(app.selected_path.as_deref(), Some(STORE));
                assert_eq!(app.caret_line, definition_line);
                let destination = app.open_buffer.as_ref().unwrap();
                assert_eq!(
                    destination.byte_offset(app.caret_line, app.caret_col),
                    destination.text.find(definition_name).unwrap()
                );
                assert_eq!(app.active_revision, Some(ReviewRevision::After));
                assert_eq!(
                    app.review_revision_choices.get(STORE),
                    Some(&ReviewRevision::After)
                );
                app.execute_command(&ctx, ToolbarAction::NavigateBack);
                settle(&mut app);
                assert_eq!(app.selected_path.as_deref(), Some(READER));
                assert_eq!(
                    buffer.byte_offset(app.caret_line, app.caret_col),
                    departure.byte_offset
                );
                assert_eq!(
                    app.review_revision_choices.get(READER),
                    Some(&ReviewRevision::Before)
                );
                app.review_revision_choices
                    .insert(STORE.into(), ReviewRevision::Before);
                app.handle_reference_action(ReferenceAction::Commit(Some(definition)));
                settle(&mut app);
                assert_eq!(
                    (
                        app.selected_path.as_deref(),
                        app.caret_line,
                        app.active_revision
                    ),
                    (Some(STORE), definition_line, Some(ReviewRevision::After))
                );
                assert_eq!(
                    app.review_revision_choices.get(STORE),
                    Some(&ReviewRevision::After)
                );
                app.execute_command(&ctx, ToolbarAction::NavigateBack);
                settle(&mut app);
                assert_eq!(
                    buffer.byte_offset(app.caret_line, app.caret_col),
                    departure.byte_offset
                );
            }
            assert_ne!(identities[0], identities[1]);
            let integer = buffer.text.find("read(2)").unwrap() + 1;
            app.navigate(
                SourceLocation::with_revision(READER, integer, Some(ReviewRevision::After)),
                true,
            );
            settle(&mut app);
            app.follow_reference_caret();
            settle(&mut app);
            assert_eq!(app.under_caret.as_ref().unwrap().id, identities[0]);
            assert_eq!(app.reference_context.target().unwrap().name, "changed");
            app.execute_command(&ctx, ToolbarAction::ToggleRevision(ReviewRevision::Before));
            settle(&mut app);
            assert!(app.open_buffer.as_ref().unwrap().text.contains("read(1)"));
            let before = app.open_buffer.as_ref().unwrap();
            app.navigate(
                before.location_at_byte(before.text.find("read(1)").unwrap() + 1),
                true,
            );
            settle(&mut app);
            assert!(app.under_caret.is_none() && app.semantic_inputs.is_none());
            assert!(app
                .outline_symbols
                .iter()
                .any(|symbol| symbol.name == "changed"));
            assert!(!app.reference_context.available());
            let before_frame = frame(&mut app, &ctx, vec![]);
            assert!(
                crate::ui::test_support::text_shapes(&before_frame).any(|shape| shape
                    .galley
                    .job
                    .text
                    .contains("Before not indexed"))
            );
            assert!(!app.command_enabled(&ToolbarAction::GoToDefinition));
            assert!(!app.command_enabled(&ToolbarAction::FindReferences));
            assert_eq!(
                SemanticAvailability::from_services(
                    app.open_buffer.as_deref(),
                    app.semantic_index.as_deref(),
                    &app.index_generator,
                    &app.index_importer
                ),
                SemanticAvailability::RevisionNotIndexed
            );
            app.execute_command(&ctx, ToolbarAction::ToggleRevision(ReviewRevision::After));
            settle(&mut app);
            app.navigate(buffer.location_at_byte(integer), true);
            settle(&mut app);
            assert_eq!(app.under_caret.as_ref().unwrap().id, identities[0]);
            app.index_tools = ToolDirectory {
                managed: None,
                home: None,
                search_path: vec![],
            };
            app.load_repository_index();
            settle(&mut app);
            assert!(app.generation_report().unwrap().cached);
            assert_eq!(
                app.generation_report().unwrap().outcomes[0].partial_build,
                !noise
            );
            assert_eq!(std::fs::read_to_string(&runs).unwrap().lines().count(), 1);
            assert_eq!(
                SemanticAvailability::from_services(
                    app.open_buffer.as_deref(),
                    app.semantic_index.as_deref(),
                    &app.index_generator,
                    &app.index_importer
                ),
                SemanticAvailability::Available
            );
            app.index_tools = tools;
            app.generate_index(true);
            settle(&mut app);
            assert!(app.generation_report().unwrap().outcomes[0]
                .reused_from
                .is_some());
            assert_eq!(std::fs::read_to_string(runs).unwrap().lines().count(), 1);
            let caret = (app.caret_line, app.caret_col);
            let history = app.history.current().cloned();
            let snapshot = app.review.as_ref().unwrap();
            let refreshed = ReviewSnapshot::prepare(
                &root,
                snapshot.summary.clone(),
                &owner.path().join("cache"),
            )
            .unwrap();
            app.accept_review(refreshed);
            settle(&mut app);
            assert_eq!((app.caret_line, app.caret_col), caret);
            assert_eq!(app.history.current(), history.as_ref());
            assert_eq!(app.under_caret.as_ref().unwrap().id, identities[0]);
            assert_eq!(app.active_revision, Some(ReviewRevision::After));
            app.source_viewport.request_focus(&ctx);
            frame(
                &mut app,
                &ctx,
                vec![key(Key::A, Modifiers::CTRL), Event::Copy],
            );
            assert_eq!(
                frame(&mut app, &ctx, vec![Event::Copy])
                    .platform_output
                    .copied_text,
                buffer.text
            );
            assert_eq!(git(&root, &["status", "--porcelain"]), status);
            assert_eq!(git(&root, &["rev-parse", "HEAD"]), head);
            assert_eq!(
                std::fs::read_to_string(root.join(READER)).unwrap(),
                "// checkout only\n"
            );
        }
    }
}
