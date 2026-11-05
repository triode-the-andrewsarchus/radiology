//! Real app frames exercise modal/canvas focus rather than a parallel test renderer.
use super::*;
mod files;
mod help;
mod ownership;
mod panes;
mod references;
mod traits;
use egui::{Event, Key, Modifiers};

pub(in crate::ui::app) fn key(key: Key, modifiers: Modifiers) -> Event {
    let mut event = crate::ui::commands::test_key(key);
    if let Event::Key {
        modifiers: actual, ..
    } = &mut event
    {
        *actual = modifiers;
    }
    event
}

pub(in crate::ui::app) fn frame(
    app: &mut RadiologyApp,
    ctx: &Context,
    events: Vec<Event>,
) -> egui::FullOutput {
    frame_with_overlay(app, ctx, events, |_| {})
}

/// Publishes worker results and paints them with the caller's actual scale/input context.
pub(in crate::ui::app) fn settle_frame(app: &mut RadiologyApp, ctx: &Context) {
    settle(app);
    frame(app, ctx, vec![]);
}

fn scaled_reader(text: &str, scale: f32) -> (tempfile::TempDir, RadiologyApp, Context) {
    let (dir, mut app) = reader_with_source(text);
    let ctx = Context::default();
    ctx.set_pixels_per_point(scale);
    frame(&mut app, &ctx, vec![]);
    (dir, app, ctx)
}

#[test]
fn welcome_recent_paths_stay_compact_and_open_their_canonical_repository() {
    use crate::ui::test_support::{click, text_shapes};
    for scale in [1.0, 2.0] {
        for dark in [true, false] {
            let (owner, mut app) = reader();
            let mut parent = owner.path().to_path_buf();
            for _ in 0..12 {
                parent.push("long-café-component-with spaces-and-details");
            }
            let recent: Vec<_> = (0..SessionStore::RECENT_LIMIT)
                .map(|index| {
                    let path = parent.join(format!("line\nbreak-recent-{index}"));
                    std::fs::create_dir_all(&path).unwrap();
                    std::fs::write(path.join("source.rs"), "// exact recent source\r\n").unwrap();
                    path.to_string_lossy().into_owned()
                })
                .collect();
            app.session.recent_repositories = recent.clone();
            let ctx = Context::default();
            ctx.set_pixels_per_point(scale);
            if !dark {
                app.execute_command(&ctx, ToolbarAction::ToggleTheme);
            }
            let mut last_point = egui::Pos2::ZERO;
            for size in [
                egui::vec2(800.0, 500.0),
                egui::vec2(1100.0, 800.0),
                egui::vec2(1400.0, 900.0),
            ] {
                for _ in 0..3 {
                    frame_sized(&mut app, &ctx, vec![], size, |_| {});
                }
                let output = frame_sized(&mut app, &ctx, vec![], size, |_| {});
                for index in 0..SessionStore::RECENT_LIMIT {
                    let end = format!("recent-{index}");
                    let row = text_shapes(&output)
                        .find(|shape| shape.galley.job.text.ends_with(&end))
                        .unwrap();
                    assert_eq!(row.galley.rows.len(), 1, "{size:?}/{scale}/{dark}/{index}");
                    assert!(row.galley.size().x <= 500.0);
                    assert!(!row.galley.job.text.contains(['\r', '\n']));
                    assert!(row.pos.x >= 0.0 && row.pos.x + row.galley.size().x <= size.x);
                    assert!(row.pos.y >= 0.0 && row.pos.y + row.galley.size().y <= size.y);
                    last_point = row.pos + row.galley.size() / 2.0;
                }
            }
            frame_sized(
                &mut app,
                &ctx,
                click(last_point),
                egui::vec2(1400.0, 900.0),
                |_| {},
            );
            settle(&mut app);
            assert_eq!(
                app.repository_root.as_deref(),
                Some(std::path::Path::new(recent.last().unwrap()))
            );
            app.open_file("source.rs");
            settle(&mut app);
            assert_eq!(
                app.open_buffer.as_ref().unwrap().text,
                "// exact recent source\r\n"
            );
        }
    }
}

#[test]
fn search_query_edits_retire_rows_and_explicit_submit_wins_over_debounce() {
    use crate::ui::test_support::{click, text_center, text_shapes};
    for scale in [1.0, 2.0] {
        let (_owner, mut app) = reader_with_source("// needle café\r\n");
        app.jump_to_location("a.rs", 1, 3);
        settle(&mut app);
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        app.execute_command(&ctx, ToolbarAction::OpenSearch);
        let output = frame(&mut app, &ctx, vec![]);
        assert!(text_shapes(&output).any(|shape| shape.galley.job.text == "Search the Source"));
        app.search_options.query = "needle".into();
        app.start_repository_search();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while app.repository_search.poll() {
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(app.repository_search.matches.len(), 1);
        let output = frame(&mut app, &ctx, vec![]);
        let old_row = text_center(&output, "a.rs:1");
        let mut events = vec![
            key(Key::A, Modifiers::CTRL | Modifiers::COMMAND),
            Event::Text("missing".into()),
        ];
        events.extend(click(old_row));
        let output = frame(&mut app, &ctx, events);
        assert_eq!(app.search_options.query, "missing");
        assert!(app.repository_search.matches.is_empty());
        assert!(app.repository_search.is_searching());
        // The app stages the later pointer boundary into the next real frame.
        frame(&mut app, &ctx, vec![]);
        assert!(!app.is_source_loading());
        assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
        assert_eq!((app.caret_line, app.caret_col), (1, 3));
        assert_eq!(app.history.current().unwrap().byte_offset, 3);
        assert!(!text_shapes(&output).any(|shape| shape.galley.job.text == "a.rs:1"));
        app.execute_command(&ctx, ToolbarAction::OpenSearch);
        frame(&mut app, &ctx, vec![]);
        frame(
            &mut app,
            &ctx,
            vec![
                key(Key::A, Modifiers::CTRL | Modifiers::COMMAND),
                Event::Text("needle".into()),
                key(Key::Enter, Modifiers::NONE),
            ],
        );
        assert_eq!(app.search_options.query, "needle");
        assert!(
            app.repository_search.running,
            "explicit submission starts immediately"
        );
        app.repository_search.stop();
        app.search_options.include_glob = "a.rs".into();
        app.execute_command(&ctx, ToolbarAction::OpenSearch);
        frame(&mut app, &ctx, vec![]);
        let output = frame(
            &mut app,
            &ctx,
            vec![
                key(Key::A, Modifiers::CTRL | Modifiers::COMMAND),
                Event::Text(" ".into()),
            ],
        );
        assert_eq!(app.search_options.query, " ");
        assert!(app.repository_search.is_searching());
        assert!(!text_shapes(&output).any(|shape| shape.galley.job.text == "Search the Source"));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while app.repository_search.poll() {
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(
            app.repository_search
                .matches
                .iter()
                .map(|hit| hit.byte_offset)
                .collect::<Vec<_>>(),
            vec![2, 9]
        );
    }
}

#[test]
fn composed_file_query_preserves_the_sidebar_search_owner() {
    let (owner, mut app) = reader();
    let root = owner.path().join("checkout");
    repository(&root);
    std::fs::write(root.join("日本.rs"), "// 日本\n").unwrap();
    app.open_repository(&root);
    settle(&mut app);
    let ctx = Context::default();
    app.search_options.query = "keep".into();
    app.execute_command(&ctx, ToolbarAction::OpenSearch);
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    assert!(ctx.memory(|memory| memory.has_focus(Sidebar::search_query_id())));
    app.execute_command(&ctx, ToolbarAction::OpenQuickFilePicker);
    frame(
        &mut app,
        &ctx,
        vec![
            Event::CompositionStart,
            Event::CompositionUpdate("にほん".into()),
            Event::CompositionEnd("日本".into()),
            key(Key::Enter, Modifiers::NONE),
            Event::CompositionEnd("suffix".into()),
        ],
    );
    assert_eq!(app.search_options.query, "keep");
    finish_file_picker(&mut app, &ctx);
    settle(&mut app);
    assert_eq!(app.selected_path.as_deref(), Some("日本.rs"));
    assert_eq!(app.open_buffer.as_ref().unwrap().text, "// 日本\n");
    assert_eq!(app.search_options.query, "keep");
}

#[test]
fn native_ime_area_tracks_the_editor_caret_through_composition_and_resize() {
    for scale in [1.0, 2.0] {
        for command in [
            ToolbarAction::ImportCodePath,
            ToolbarAction::OpenQuickFilePicker,
        ] {
            let (_owner, mut app) = isolated_source("a.rs", "// retained source\n");
            let ctx = Context::default();
            ctx.set_pixels_per_point(scale);
            app.execute_command(&ctx, command);
            for (size, events) in [
                (egui::vec2(1100.0, 800.0), vec![]),
                (
                    egui::vec2(1100.0, 800.0),
                    vec![
                        Event::CompositionStart,
                        Event::CompositionUpdate("にほん".into()),
                    ],
                ),
                (
                    egui::vec2(800.0, 600.0),
                    vec![Event::CompositionUpdate("日本".into())],
                ),
                (
                    egui::vec2(1100.0, 800.0),
                    vec![Event::CompositionEnd("日本".into())],
                ),
            ] {
                // Settle the initial modal measurement and resized geometry.
                frame_sized(&mut app, &ctx, events, size, |_| {});
                let output = frame_sized(&mut app, &ctx, vec![], size, |_| {});
                let ime = output
                    .platform_output
                    .ime
                    .expect("focused editor enables native IME");
                assert_eq!(
                    ime.rect, ime.cursor_rect,
                    "candidate anchor at {scale}×/{size:?}"
                );
                assert!(ime.rect.width() <= 4.0);
                assert!(ime.rect.height() > 0.0);
                assert!(egui::Rect::from_min_size(egui::Pos2::ZERO, size).contains_rect(ime.rect));
                assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
                assert_eq!(
                    app.open_buffer.as_ref().unwrap().text,
                    "// retained source\n"
                );
            }
            frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
            let output = frame(&mut app, &ctx, vec![]);
            assert!(matches!(app.modal, ActiveModal::None));
            assert!(output.platform_output.ime.is_none());
        }
    }
}

fn assert_footer_labels(
    output: &egui::FullOutput,
    bounds: egui::Rect,
    labels: &[&str],
    single_row: bool,
) {
    use crate::ui::test_support::text_shapes;
    let mut rectangles = Vec::new();
    let mut row_positions = Vec::new();
    for &label in labels {
        let text = text_shapes(output)
            .find(|text| text.galley.job.text == label && text.pos.y >= bounds.top())
            .unwrap_or_else(|| {
                panic!(
                    "missing footer {label}: {:?}",
                    text_shapes(output)
                        .map(|text| &text.galley.job.text)
                        .collect::<Vec<_>>()
                )
            });
        if single_row {
            assert_eq!(
                text.galley.rows.len(),
                1,
                "{label} wrapped internally in {bounds:?}"
            );
        }
        row_positions.push(text.pos.y);
        for row in text
            .galley
            .rows
            .iter()
            .filter(|row| !row.visuals.mesh.vertices.is_empty())
        {
            let rect = row.visuals.mesh_bounds.translate(text.pos.to_vec2());
            assert!(
                bounds.contains_rect(rect),
                "{label}: {rect:?} outside footer {bounds:?}"
            );
            for previous in &rectangles {
                assert!(
                    !rect.intersects(*previous),
                    "footer overlap {label}: {rect:?} with {previous:?}"
                );
            }
            rectangles.push(rect);
        }
    }
    if single_row {
        let min = row_positions.iter().copied().fold(f32::INFINITY, f32::min);
        let max = row_positions
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            max - min <= 4.0,
            "routine footer metadata wrapped in {bounds:?}: {row_positions:?}"
        );
    }
}

#[test]
fn narrow_source_footer_keeps_metadata_disjoint_and_line_control_usable() {
    use crate::ui::test_support::{click, text_center};
    for scale in [1.0, 2.0] {
        for (diagnostics, revision) in [
            (false, None),
            (true, None),
            (false, Some(ReviewRevision::Before)),
            (false, Some(ReviewRevision::After)),
        ] {
            let (_owner, mut app) = isolated_source("a.kt", "// one\n// café two\n// three\n");
            let byte = app.open_buffer.as_ref().unwrap().byte_offset(2, 4);
            app.jump_to_location("a.kt", 2, byte);
            settle(&mut app);
            let ctx = Context::default();
            ctx.set_pixels_per_point(scale);
            let mut fonts = egui::FontDefinitions::default();
            reader_fonts::use_ui_font(&mut fonts);
            ctx.set_fonts(fonts);
            ctx.style_mut(|style| {
                style
                    .text_styles
                    .insert(egui::TextStyle::Small, egui::FontId::proportional(12.0));
            });
            if diagnostics {
                app.execute_command(&ctx, ToolbarAction::ToggleTheme);
                settle(&mut app);
                app.syntax_error = Some("fixture structure failure".into());
                app.fold_error = Some("fixture fold failure".into());
                app.syntactic_occurrences.error = Some("fixture occurrence failure".into());
                Arc::make_mut(&mut app.native_highlights).error =
                    Some("fixture highlight failure".into());
            }
            Arc::make_mut(app.open_buffer.as_mut().unwrap()).revision = revision;
            let mut first_height = None;
            for width in [800.0, 1024.0, 1100.0, 1400.0, 800.0] {
                let size = egui::vec2(width, 800.0);
                for _ in 0..4 {
                    frame_sized(&mut app, &ctx, vec![], size, |_| {});
                }
                let output = frame_sized(&mut app, &ctx, vec![], size, |_| {});
                let canvas = ctx
                    .read_response(crate::ui::source_view::SourceViewport::focus_id())
                    .unwrap()
                    .rect;
                let footer_bounds = egui::Rect::from_min_max(
                    egui::pos2(
                        canvas.left() - crate::ui::source_view::SourceView::GUTTER_WIDTH,
                        canvas.bottom(),
                    ),
                    egui::pos2(canvas.right(), size.y),
                );
                let mut labels = vec![
                    "Kotlin",
                    "4 lines",
                    "Line 2",
                    if revision == Some(ReviewRevision::Before) {
                        "Not Indexed (Before)"
                    } else {
                        "No Index"
                    },
                    "UTF-8",
                    "Read Only",
                ];
                if diagnostics {
                    labels.extend([
                        "Structure unavailable",
                        "Folding unavailable",
                        "Occurrence highlighting unavailable",
                        "Basic syntax colors",
                    ]);
                }
                if let Some(revision) = revision {
                    labels.push(revision.title());
                }
                assert_footer_labels(
                    &output,
                    footer_bounds,
                    &labels,
                    !diagnostics && width >= 1024.0,
                );
                if width == 800.0 {
                    let initial = first_height.get_or_insert(canvas.height());
                    assert!(
                        (canvas.height() - *initial).abs() <= 1.0,
                        "footer failed to settle after resizing"
                    );
                }
                let original = app.open_buffer.clone().unwrap();
                frame_sized(
                    &mut app,
                    &ctx,
                    click(text_center(&output, "Line 2")),
                    size,
                    |_| {},
                );
                frame_sized(&mut app, &ctx, vec![], size, |_| {});
                assert!(matches!(app.modal, ActiveModal::GoToLine(_)));
                frame_sized(
                    &mut app,
                    &ctx,
                    vec![key(Key::Escape, Modifiers::NONE)],
                    size,
                    |_| {},
                );
                assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &original));
                assert_eq!((app.caret_line, app.caret_col), (2, 4));
            }
        }
    }
}

#[test]
fn long_toolbar_identity_keeps_controls_visible_and_repository_copy_canonical() {
    use crate::ui::test_support::{click, text_center, text_shapes};
    let (_owner, root, review) = crate::pull_request::review::tests::fixture();
    for scale in [1.0, 2.0] {
        for width in [800.0, 1100.0, 1400.0] {
            for (dark, reviewing) in [(false, false), (true, false), (false, true), (true, true)] {
                let (_settings, mut app) = reader();
                app.open_repository(&root);
                settle(&mut app);
                if reviewing {
                    app.accept_review(review.clone());
                    settle(&mut app);
                }
                let ctx = Context::default();
                ctx.set_pixels_per_point(scale);
                if app.theme.is_dark != dark {
                    app.execute_command(&ctx, ToolbarAction::ToggleTheme);
                }
                app.repository_name = Some(format!("repository-{}-tail", "長いcafé\n".repeat(600)));
                app.branch_name = Some(format!("feature/{}-done", "branch\n".repeat(800)));
                let size = egui::vec2(width, 800.0);
                frame_sized(&mut app, &ctx, vec![], size, |_| {});
                let output = frame_sized(&mut app, &ctx, vec![], size, |_| {});
                for label in [
                    "Sidebar",
                    "Normal",
                    "Critical",
                    "View",
                    "Search",
                    "Inspector",
                ] {
                    let text = text_shapes(&output)
                        .find(|text| text.galley.job.text == label)
                        .unwrap_or_else(|| panic!("missing {label} at {width}/{scale}/{dark}"));
                    let rect = egui::Rect::from_min_size(text.pos, text.galley.size());
                    assert!(
                        rect.min.x >= 0.0 && rect.max.x <= width && rect.max.y < 40.0,
                        "{label} outside toolbar: {rect:?} at {width}/{scale}/{dark}"
                    );
                }
                let review_title = if reviewing {
                    format!("#{}", review.summary.number)
                } else {
                    "Pull Request".into()
                };
                let review_point = text_center(&output, &review_title);
                assert!(review_point.x < width && review_point.y < 40.0);
                if reviewing {
                    assert!(text_center(&output, "Exit PR").x < width);
                }
                let title = text_shapes(&output)
                    .find(|text| text.galley.job.text.starts_with("repository-"))
                    .unwrap();
                assert!(title.galley.size().x <= 140.0);
                assert_eq!(title.galley.rows.len(), 1);
                assert!(title.galley.job.text.ends_with("-tail"));
                let branch = text_shapes(&output)
                    .find(|text| text.galley.job.text.starts_with("feat"))
                    .unwrap();
                assert!(branch.galley.size().x <= 80.0);
                assert_eq!(branch.galley.rows.len(), 1);
                assert!(branch.galley.job.text.ends_with("-done"));
                let position = title.pos + title.galley.size() / 2.0;
                frame_sized(&mut app, &ctx, click(position), size, |_| {});
                let output = frame_sized(&mut app, &ctx, vec![], size, |_| {});
                let active_title = text_shapes(&output)
                    .find(|text| text.galley.job.text.starts_with("repository-"))
                    .unwrap();
                assert_eq!(
                    active_title.galley.job.sections[0].format.color,
                    ctx.style().visuals.selection.stroke.color
                );
                let output = frame_sized(
                    &mut app,
                    &ctx,
                    click(text_center(&output, "Copy Repository Path")),
                    size,
                    |_| {},
                );
                assert_eq!(
                    output.platform_output.copied_text,
                    app.repository_root.as_ref().unwrap().display().to_string()
                );
            }
        }
    }
}

#[test]
fn repository_menu_refresh_is_contextual_and_pending_review_cannot_restart() {
    use crate::ui::test_support::{click, text_center, text_shapes};
    for scale in [1.0, 2.0] {
        let (_owner, root, review) = crate::pull_request::review::tests::fixture();
        let (_settings, mut app) = reader();
        app.open_repository(&root);
        settle(&mut app);
        app.jump_to_location("a file.rs", 1, 4);
        settle(&mut app);
        let before = app.open_buffer.clone().unwrap();
        std::fs::write(root.join("new.rs"), "fn new_file() {}\n").unwrap();
        let (ctx, output) = focused_source_frame(&mut app, scale);
        let title = app.repository_name.clone().unwrap();
        frame(&mut app, &ctx, click(text_center(&output, &title)));
        let output = frame(&mut app, &ctx, vec![]);
        frame(
            &mut app,
            &ctx,
            click(text_center(&output, "Reload Repository (Ctrl+R)")),
        );
        assert!(app.is_repository_opening());
        settle(&mut app);
        assert!(app.all_files.iter().any(|file| file.path == "new.rs"));
        assert_eq!(app.open_buffer.as_ref().unwrap().text, before.text);
        assert_eq!((app.caret_line, app.caret_col), (1, 4));
        app.accept_review(review);
        settle(&mut app);
        let accepted = app.review.clone().unwrap();
        let (release, gate) = std::sync::mpsc::channel();
        let (finished, outcome) = std::sync::mpsc::channel();
        app.review_job.spawn(move |cancel| {
            let _ = gate.recv_timeout(std::time::Duration::from_secs(5));
            let _ = finished.send(cancel.load(std::sync::atomic::Ordering::Relaxed));
            Err("controlled refresh failure".into())
        });
        app.pending_pr = Some(accepted.summary.number);
        let output = frame(&mut app, &ctx, vec![]);
        frame(&mut app, &ctx, click(text_center(&output, &title)));
        let output = frame(&mut app, &ctx, vec![]);
        assert!(
            !text_shapes(&output).any(|text| text.galley.job.text == "Reload Repository (Ctrl+R)")
        );
        frame(
            &mut app,
            &ctx,
            click(text_center(&output, "Refresh PR (Ctrl+R)")),
        );
        frame(
            &mut app,
            &ctx,
            vec![
                key(Key::Escape, Modifiers::NONE),
                key(Key::R, Modifiers::CTRL),
            ],
        );
        assert!(app.review_job.is_running());
        assert_eq!(app.pending_pr, Some(accepted.summary.number));
        release.send(()).unwrap();
        assert!(!outcome
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap());
        settle(&mut app);
        assert!(Arc::ptr_eq(app.review.as_ref().unwrap(), &accepted));
        assert!(app
            .error
            .as_ref()
            .unwrap()
            .contains("controlled refresh failure"));
    }
}

#[test]
fn modal_background_clicks_scroll_and_closing_input_preserve_the_reader() {
    let text: String = (1..=300)
        .map(|line| format!("// line {line} café\r\n"))
        .collect();
    for scale in [1.0, 2.0] {
        for command in [
            ToolbarAction::OpenGoToLine,
            ToolbarAction::OpenQuickFilePicker,
            ToolbarAction::OpenGoToSymbol,
            ToolbarAction::OpenPRDialog,
        ] {
            let (_dir, mut app, ctx) = scaled_reader(&text, scale);
            let buffer = app.open_buffer.clone().unwrap();
            let selection = buffer.line_byte_range(2);
            app.navigate(buffer.location_at_byte(selection.end), true);
            app.source_viewport.select(selection.clone());
            app.source_viewport.request_focus(&ctx);
            let before = frame(&mut app, &ctx, vec![]);
            let sidebar = crate::ui::test_support::text_center(&before, "Sidebar");
            app.execute_command(&ctx, command);
            let mut output = frame(&mut app, &ctx, crate::ui::test_support::click(sidebar));
            assert!(app.show_sidebar, "opening sheet leaked pointer input");
            for _ in 0..3 {
                output = frame(&mut app, &ctx, vec![]);
            }
            let sidebar = crate::ui::test_support::text_center(&output, "Sidebar");
            let focus = ctx.memory(|memory| memory.focused());
            let top = app.source_viewport.top_byte(&buffer);
            frame(&mut app, &ctx, crate::ui::test_support::click(sidebar));
            assert!(
                app.show_sidebar,
                "background Sidebar click escaped the sheet"
            );
            assert_eq!(ctx.memory(|memory| memory.focused()), focus);
            frame(
                &mut app,
                &ctx,
                vec![
                    Event::PointerMoved(egui::pos2(350.0, 650.0)),
                    Event::Scroll(egui::vec2(0.0, -240.0)),
                ],
            );
            frame(&mut app, &ctx, vec![]);
            assert_eq!(app.source_viewport.top_byte(&buffer), top);
            let mut close = crate::ui::test_support::click(sidebar);
            close.insert(0, key(Key::Escape, Modifiers::NONE));
            frame(&mut app, &ctx, close);
            assert!(matches!(app.modal, ActiveModal::None));
            assert!(app.show_sidebar, "closing sheet leaked its pointer input");
            assert_eq!(
                buffer.byte_offset(app.caret_line, app.caret_col),
                selection.end
            );
            assert!(app.source_viewport.has_focus(&ctx));
            frame(&mut app, &ctx, crate::ui::test_support::click(sidebar));
            assert!(!app.show_sidebar, "fresh input must work after dismissal");
            app.source_viewport.request_focus(&ctx);
            assert_eq!(
                frame(&mut app, &ctx, vec![Event::Copy])
                    .platform_output
                    .copied_text,
                text[selection]
            );
        }
    }
}

#[test]
fn modal_surface_blocks_existing_utility_windows_but_keeps_its_own_cancel_clickable() {
    let (_dir, mut app, ctx) = scaled_reader("// first\r\n// second\r\n", 1.0);
    let changes = std::cell::Cell::new(0);
    let run = |app: &mut RadiologyApp, events| {
        frame_with_overlay(app, &ctx, events, |ctx| {
            egui::Window::new("Background utility")
                .fixed_pos(egui::pos2(15.0, 350.0))
                .constrain(false)
                .resizable(false)
                .show(ctx, |ui| {
                    if ui.button("Change background").clicked() {
                        changes.set(changes.get() + 1);
                    }
                });
        })
    };
    run(&mut app, vec![]);
    let output = run(&mut app, vec![]);
    let background = crate::ui::test_support::text_center(&output, "Change background");
    assert!(
        background.x < 200.0,
        "utility control must be outside every dialog"
    );
    run(&mut app, crate::ui::test_support::click(background));
    assert_eq!(changes.get(), 1, "utility click before the dialog");
    let mut expected = 1;
    for command in [
        ToolbarAction::OpenGoToLine,
        ToolbarAction::OpenQuickFilePicker,
        ToolbarAction::OpenGoToSymbol,
        ToolbarAction::OpenPRDialog,
        ToolbarAction::OpenGoToLine,
    ] {
        app.execute_command(&ctx, command);
        let mut output = run(&mut app, vec![]);
        for _ in 0..3 {
            output = run(&mut app, vec![]);
        }
        let cancel = crate::ui::test_support::text_center(&output, "Cancel");
        let focus = ctx.memory(|memory| memory.focused());
        run(&mut app, crate::ui::test_support::click(background));
        assert_eq!(changes.get(), expected, "utility click behind the dialog");
        assert_eq!(ctx.memory(|memory| memory.focused()), focus);
        assert!(!matches!(app.modal, ActiveModal::None));
        run(&mut app, crate::ui::test_support::click(cancel));
        assert!(matches!(app.modal, ActiveModal::None));
        run(&mut app, crate::ui::test_support::click(background));
        expected += 1;
        assert_eq!(
            changes.get(),
            expected,
            "fresh utility click after dismissal"
        );
    }
    assert_eq!(
        changes.get(),
        6,
        "only the before/after utility clicks should run"
    );
}

#[test]
fn modal_tabs_cycle_only_enabled_controls_and_batch_cancel_returns_to_source() {
    for command in [
        ToolbarAction::OpenGoToLine,
        ToolbarAction::OpenQuickFilePicker,
        ToolbarAction::OpenGoToSymbol,
        ToolbarAction::OpenPRDialog,
    ] {
        let text = "// first\r\n// café 1️⃣\r\n// last";
        let (_dir, mut app, ctx) = scaled_reader(text, 1.0);
        let buffer = app.open_buffer.clone().unwrap();
        let range = buffer.line_byte_range(2);
        app.navigate(buffer.location_at_byte(range.end), true);
        app.source_viewport.select(range.clone());
        app.source_viewport.request_focus(&ctx);
        app.execute_command(&ctx, command);
        for _ in 0..6 {
            frame(&mut app, &ctx, vec![]);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(!matches!(app.modal, ActiveModal::None));
        if let ActiveModal::GoToLine(modal) = &app.modal {
            assert_eq!(modal.line_str, "2");
        }
        let query = ctx.memory(|memory| memory.focused()).unwrap();
        let layer = ctx.read_response(query).unwrap().layer_id;
        let mut cycle = Vec::new();
        loop {
            frame(&mut app, &ctx, vec![key(Key::Tab, Modifiers::NONE)]);
            let focus = ctx.memory(|memory| memory.focused()).unwrap();
            assert_eq!(ctx.read_response(focus).unwrap().layer_id, layer);
            if focus == query {
                break;
            }
            assert!(!cycle.contains(&focus));
            cycle.push(focus);
            assert!(cycle.len() <= 2);
        }
        assert!(!cycle.is_empty());
        frame(
            &mut app,
            &ctx,
            vec![
                key(Key::Tab, Modifiers::SHIFT),
                key(Key::Enter, Modifiers::NONE),
                key(Key::ArrowRight, Modifiers::NONE),
            ],
        );
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![]);
        }
        assert!(matches!(app.modal, ActiveModal::None));
        assert_eq!(buffer.byte_offset(app.caret_line, app.caret_col), range.end);
        assert!(app.source_viewport.has_focus(&ctx));
        assert_eq!(
            frame(&mut app, &ctx, vec![Event::Copy])
                .platform_output
                .copied_text,
            text[range]
        );
    }
}

#[test]
fn line_dialog_validates_signed_input_clamps_and_keeps_invalid_edits_focused() {
    for (value, expected) in [
        ("", None),
        ("bad", None),
        ("9223372036854775808", None),
        ("-3", Some(1)),
        ("0", Some(1)),
        ("99", Some(3)),
        (" +2 ", Some(2)),
    ] {
        let (_dir, mut app) = reader_with_source("// first\r\n// second\r\n// last");
        let ctx = Context::default();
        frame(&mut app, &ctx, vec![]);
        app.jump_to_location("a.rs", 2, 13);
        app.source_viewport.request_focus(&ctx);
        frame(&mut app, &ctx, vec![key(Key::L, Modifiers::CTRL)]);
        frame(&mut app, &ctx, vec![]);
        let history = app.history.current().cloned();
        frame(
            &mut app,
            &ctx,
            vec![
                key(Key::A, Modifiers::CTRL),
                key(Key::Backspace, Modifiers::NONE),
                Event::Paste(value.into()),
                key(Key::Enter, Modifiers::NONE),
            ],
        );
        if let Some(line) = expected {
            assert!(matches!(app.modal, ActiveModal::None), "{value}");
            assert_eq!((app.caret_line, app.caret_col), (line, 0), "{value}");
        } else {
            assert!(matches!(app.modal, ActiveModal::GoToLine(_)), "{value}");
            assert_eq!(app.history.current(), history.as_ref());
            let output = frame(&mut app, &ctx, vec![]);
            assert!(ctx.wants_keyboard_input());
            let field = ctx
                .read_response(ctx.memory(|memory| memory.focused()).unwrap())
                .unwrap()
                .rect
                .center();
            let go = crate::ui::test_support::text_center(&output, "Go");
            frame(&mut app, &ctx, crate::ui::test_support::click(go));
            assert!(matches!(app.modal, ActiveModal::GoToLine(_)));
            frame(&mut app, &ctx, crate::ui::test_support::click(field));
            frame(
                &mut app,
                &ctx,
                vec![
                    key(Key::A, Modifiers::CTRL),
                    Event::Text("3".into()),
                    key(Key::Enter, Modifiers::NONE),
                ],
            );
            assert!(matches!(app.modal, ActiveModal::None));
            assert_eq!((app.caret_line, app.caret_col), (3, 0));
        }
    }
}

#[test]
fn line_dialog_opening_batch_submits_the_edited_number_and_retires_later_keys() {
    for scale in [1.0, 1.5, 2.0] {
        for paste in [false, true] {
            let text = "// first\r\n// café\r\n// last";
            let (_dir, mut app, ctx) = scaled_reader(text, scale);
            let buffer = app.open_buffer.clone().unwrap();
            app.navigate(buffer.location_at_byte(3), true);
            app.source_viewport.request_focus(&ctx);
            frame(
                &mut app,
                &ctx,
                vec![
                    key(Key::L, Modifiers::CTRL),
                    key(Key::A, Modifiers::CTRL),
                    if paste {
                        Event::Paste("3".into())
                    } else {
                        Event::Text("3".into())
                    },
                    key(Key::Enter, Modifiers::NONE),
                    key(Key::ArrowRight, Modifiers::NONE),
                    Event::Copy,
                ],
            );
            for _ in 0..3 {
                frame(&mut app, &ctx, vec![]);
            }
            assert!(
                matches!(app.modal, ActiveModal::None),
                "scale={scale}, paste={paste}"
            );
            assert_eq!((app.caret_line, app.caret_col), (3, 0));
            assert!(Arc::ptr_eq(&buffer, app.open_buffer.as_ref().unwrap()));
            assert!(app.source_viewport.has_focus(&ctx));
            app.execute_command(&ctx, ToolbarAction::NavigateBack);
            assert_eq!((app.caret_line, app.caret_col), (1, 3));
        }
    }
}

#[test]
fn accepted_navigation_collapses_endpoint_selections_before_later_source_input() {
    let text = "// café 1️⃣\r\n// second 👍🏽\r\n// last";
    let first_end = text.find('\r').unwrap();
    for scale in [1.0, 1.5, 2.0] {
        let (_dir, mut app, ctx) = scaled_reader(text, scale);
        let buffer = app.open_buffer.clone().unwrap();
        for wrap in [false, true] {
            app.session.wrap_lines = wrap;
            frame(&mut app, &ctx, vec![]);
            for (anchor, focus) in [(0, first_end), (first_end, 0)] {
                for (destination, extended) in [
                    (0, "/"),
                    (first_end, "\r\n"),
                    (first_end + 2, "/"),
                    (3, "c"),
                ] {
                    app.caret_line = buffer.line_at_byte(focus);
                    app.caret_col = buffer.column_at_byte(focus);
                    app.source_viewport.select(anchor..focus);
                    app.navigate(buffer.location_at_byte(destination), true);
                    assert!(Arc::ptr_eq(&buffer, app.open_buffer.as_ref().unwrap()));
                    app.source_viewport.request_focus(&ctx);
                    let output = frame(&mut app, &ctx, vec![Event::Copy]);
                    assert!(
                        output.platform_output.copied_text.is_empty(),
                        "stale selection at {destination}, {anchor}..{focus}, scale={scale}"
                    );
                    assert_eq!(
                        buffer.byte_offset(app.caret_line, app.caret_col),
                        destination
                    );
                    let output = frame(
                        &mut app,
                        &ctx,
                        vec![key(Key::ArrowRight, Modifiers::SHIFT), Event::Copy],
                    );
                    assert_eq!(output.platform_output.copied_text, extended);
                    app.navigate(buffer.location_at_byte(destination), true);
                    let output = frame(
                        &mut app,
                        &ctx,
                        vec![key(Key::A, Modifiers::CTRL), Event::Copy],
                    );
                    assert_eq!(output.platform_output.copied_text, text);
                    assert_eq!(
                        frame(&mut app, &ctx, vec![Event::Copy])
                            .platform_output
                            .copied_text,
                        text
                    );
                }
            }
        }
    }
}

#[test]
fn canceled_and_failed_navigation_keep_the_displayed_selection_and_history() {
    let (_dir, mut app) = reader_with_source("// café 1️⃣\r\n// second\r\n");
    let ctx = Context::default();
    frame(&mut app, &ctx, vec![]);
    let buffer = app.open_buffer.clone().unwrap();
    let selected = buffer.line_byte_range(1);
    app.source_viewport.select(selected.clone());
    app.caret_line = buffer.line_at_byte(selected.end);
    app.caret_col = buffer.column_at_byte(selected.end);
    app.source_viewport.request_focus(&ctx);
    let departure = (app.caret_line, app.caret_col);
    let history = app.history.current().cloned();
    app.execute_command(&ctx, ToolbarAction::ReloadSource);
    assert!(app.pending_navigation.is_some());
    app.cancel_source_load();
    assert_eq!(
        frame(&mut app, &ctx, vec![Event::Copy])
            .platform_output
            .copied_text,
        buffer.text[selected.clone()]
    );
    app.open_file("missing.rs");
    settle(&mut app);
    assert!(app.error.is_some());
    let output = frame(&mut app, &ctx, vec![Event::Copy]);
    assert_eq!(output.platform_output.copied_text, buffer.text[selected]);
    assert!(Arc::ptr_eq(&buffer, app.open_buffer.as_ref().unwrap()));
    assert_eq!((app.caret_line, app.caret_col), departure);
    assert_eq!(app.history.current(), history.as_ref());
}

#[test]
fn history_destinations_collapse_selection_but_find_and_preview_keep_their_ranges() {
    let (_dir, mut app) = reader_with_source("// café 1️⃣\r\n// second café\r\n");
    let ctx = Context::default();
    frame(&mut app, &ctx, vec![]);
    let buffer = app.open_buffer.clone().unwrap();
    let start = buffer.text.find("café").unwrap();
    let end = buffer.line_byte_range(1).end;
    app.navigate(buffer.location_at_byte(start), true);
    app.navigate(buffer.location_at_byte(end), true);
    for (command, expected) in [
        (ToolbarAction::NavigateBack, start),
        (ToolbarAction::NavigateForward, end),
    ] {
        app.source_viewport.select(start..end);
        app.execute_command(&ctx, command);
        app.source_viewport.request_focus(&ctx);
        assert_eq!(
            frame(&mut app, &ctx, vec![Event::Copy])
                .platform_output
                .copied_text,
            ""
        );
        assert_eq!(buffer.byte_offset(app.caret_line, app.caret_col), expected);
    }
    app.source_find.search.options.query = "café".into();
    app.source_find.open();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while app.source_find.active_range.is_none() {
        frame(&mut app, &ctx, vec![]);
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    let range = app.source_find.active_range.clone().unwrap();
    app.source_viewport.request_focus(&ctx);
    frame(&mut app, &ctx, vec![]);
    assert!(app.source_viewport.has_focus(&ctx));
    assert_eq!(
        frame(&mut app, &ctx, vec![Event::Copy])
            .platform_output
            .copied_text,
        buffer.text[range.clone()]
    );
    app.source_viewport
        .preview_byte(&buffer, buffer.line_byte_range(2).start);
    app.queue_rehighlight();
    settle(&mut app);
    for _ in 0..3 {
        let output = frame(&mut app, &ctx, vec![Event::Copy]);
        assert_eq!(
            output.platform_output.copied_text,
            buffer.text[range.clone()]
        );
    }
}

fn frame_with_overlay(
    app: &mut RadiologyApp,
    ctx: &Context,
    events: Vec<Event>,
    overlay: impl FnMut(&Context),
) -> egui::FullOutput {
    frame_sized(app, ctx, events, egui::vec2(1100.0, 800.0), overlay)
}

fn frame_sized(
    app: &mut RadiologyApp,
    ctx: &Context,
    events: Vec<Event>,
    size: egui::Vec2,
    mut overlay: impl FnMut(&Context),
) -> egui::FullOutput {
    use eframe::App;
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
        events,
        ..Default::default()
    };
    app.raw_input_hook(ctx, &mut input);
    ctx.run(input, |ctx| {
        app.render(ctx);
        overlay(ctx);
    })
}

// Async file search publishes through real app frames before file loading begins.
fn finish_file_picker(app: &mut RadiologyApp, ctx: &Context) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while matches!(app.modal, ActiveModal::QuickFilePicker(_)) {
        frame(app, ctx, vec![]);
        assert!(
            std::time::Instant::now() < deadline,
            "file picker did not commit"
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

fn source_reader() -> (tempfile::TempDir, RadiologyApp, Context) {
    let (dir, mut app) = opened_repository();
    app.open_file("a.rs");
    settle(&mut app);
    let ctx = Context::default();
    frame(&mut app, &ctx, vec![]);
    (dir, app, ctx)
}

fn welcome_frame(scale: f32) -> (tempfile::TempDir, RadiologyApp, Context, egui::FullOutput) {
    let (owner, mut app) = reader();
    let ctx = Context::default();
    ctx.set_pixels_per_point(scale);
    frame(&mut app, &ctx, vec![]);
    let output = frame(&mut app, &ctx, vec![]);
    (owner, app, ctx, output)
}

fn assert_source_line_copy(
    app: &mut RadiologyApp,
    ctx: &Context,
    caret: (usize, usize),
    expected: &str,
) {
    assert!(app.source_viewport.has_focus(ctx));
    assert_eq!((app.caret_line, app.caret_col), caret);
    let output = frame(app, ctx, vec![key(Key::End, Modifiers::SHIFT), Event::Copy]);
    assert_eq!(output.platform_output.copied_text, expected);
}

fn open_toolbar_menu(app: &mut RadiologyApp, ctx: &Context, label: &str) {
    let output = frame(app, ctx, vec![]);
    app.source_viewport.request_focus(ctx);
    frame(
        app,
        ctx,
        crate::ui::test_support::click(crate::ui::test_support::text_center(&output, label)),
    );
}

#[test]
fn committed_line_modal_returns_source_keys_without_a_click_and_owns_closing_batch() {
    let (_dir, mut app, ctx) = source_reader();
    app.source_viewport.request_focus(&ctx);
    frame(&mut app, &ctx, vec![key(Key::L, Modifiers::CTRL)]);
    frame(&mut app, &ctx, vec![Event::Text("2".into())]);
    frame(
        &mut app,
        &ctx,
        vec![
            key(Key::Enter, Modifiers::NONE),
            key(Key::ArrowDown, Modifiers::NONE),
        ],
    );
    assert!(matches!(app.modal, ActiveModal::None));
    assert_source_line_copy(&mut app, &ctx, (2, 0), "    let café = 1;");
}

#[test]
fn cancelled_modal_restores_the_live_find_query_and_source_cancellation_restores_canvas() {
    let (_dir, mut app, ctx) = source_reader();
    frame(&mut app, &ctx, vec![key(Key::F, Modifiers::CTRL)]);
    frame(&mut app, &ctx, vec![]);
    let query_focus = ctx.memory(|memory| memory.focused()).unwrap();
    assert!(!app.source_viewport.has_focus(&ctx));
    frame(&mut app, &ctx, vec![key(Key::L, Modifiers::CTRL)]);
    frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(query_focus));
    frame(&mut app, &ctx, vec![Event::Text("café".into())]);
    assert_eq!(app.source_find.search.options.query, "café");
    assert_eq!((app.caret_line, app.caret_col), (1, 0));

    app.source_viewport.request_focus(&ctx);
    frame(&mut app, &ctx, vec![key(Key::T, Modifiers::CTRL)]);
    frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    assert!(app.source_viewport.has_focus(&ctx));
    frame(&mut app, &ctx, vec![key(Key::End, Modifiers::NONE)]);
    assert!(app.caret_col > 0);
}

#[test]
fn breadcrumb_commit_returns_source_keys_after_pointer_focus() {
    let (_dir, mut app, ctx) = source_reader();
    app.fold_store.unfold_all();
    app.jump_to_location("a.rs", 2, 0);
    frame(&mut app, &ctx, vec![]);
    let output = frame(&mut app, &ctx, vec![]);
    let source = ctx
        .read_response(crate::ui::source_view::SourceViewport::focus_id())
        .expect("source viewport")
        .rect;
    let symbol = output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text)
                if text.galley.job.text == "alpha"
                    && text.pos.y < source.top()
                    && text.pos.x < source.right() =>
            {
                Some(text.pos + text.galley.size() / 2.0)
            }
            _ => None,
        })
        .expect("source breadcrumb");
    frame(&mut app, &ctx, crate::ui::test_support::click(symbol));
    assert_eq!(app.caret_line, 1);
    assert!(app.source_viewport.has_focus(&ctx));
    let output = frame(
        &mut app,
        &ctx,
        vec![
            key(Key::Home, Modifiers::NONE),
            key(Key::End, Modifiers::SHIFT),
            Event::Copy,
        ],
    );
    assert_eq!(output.platform_output.copied_text, "fn alpha() {");
}

#[test]
fn first_file_modal_keeps_canvas_focus_while_loading_without_stealing_later_find_focus() {
    let (_dir, mut app) = opened_repository();
    let ctx = Context::default();
    frame(&mut app, &ctx, vec![key(Key::K, Modifiers::CTRL)]);
    frame(&mut app, &ctx, vec![Event::Text("a.rs".into())]);
    frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
    finish_file_picker(&mut app, &ctx);
    assert!(app.source_viewport.has_focus(&ctx));
    settle(&mut app);
    frame(&mut app, &ctx, vec![]);
    assert!(app.source_viewport.has_focus(&ctx));
    let output = frame(
        &mut app,
        &ctx,
        vec![key(Key::End, Modifiers::SHIFT), Event::Copy],
    );
    assert_eq!(output.platform_output.copied_text, "fn alpha() {");
    frame(&mut app, &ctx, vec![key(Key::F, Modifiers::CTRL)]);
    let query_focus = ctx.memory(|memory| memory.focused()).unwrap();
    app.open_file("b.rs");
    settle(&mut app);
    frame(&mut app, &ctx, vec![]);
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(query_focus));
    frame(&mut app, &ctx, vec![Event::Text("beta".into())]);
    assert_eq!(app.source_find.search.options.query, "beta");
}

#[test]
fn repository_reset_retires_modal_input_and_its_return_owner() {
    use eframe::App;
    let (_dir, mut app, ctx) = source_reader();
    app.source_viewport.request_focus(&ctx);
    frame(&mut app, &ctx, vec![key(Key::L, Modifiers::CTRL)]);
    let mut input = egui::RawInput {
        events: vec![key(Key::Enter, Modifiers::NONE)],
        ..Default::default()
    };
    app.raw_input_hook(&ctx, &mut input);
    assert!(input.events.is_empty());
    app.reset_repository_state();
    assert!(app.modal_focus.is_none());
    app.modal = ActiveModal::GoToLine(GoToLineModal::new(1));
    frame(&mut app, &ctx, vec![]);
    assert!(matches!(app.modal, ActiveModal::GoToLine(_)));
    assert!(!app.source_viewport.has_focus(&ctx));
}

#[test]
fn view_menu_keyboard_activation_restores_source_selection_without_an_extra_click() {
    let (_dir, mut app, ctx) = source_reader();
    app.fold_store.unfold_all();
    app.jump_to_location("a.rs", 2, 0);
    let original_wrap = app.session.wrap_lines;
    open_toolbar_menu(&mut app, &ctx, "View");
    frame(
        &mut app,
        &ctx,
        vec![
            key(Key::Home, Modifiers::NONE),
            key(Key::ArrowDown, Modifiers::NONE),
            key(Key::Enter, Modifiers::NONE),
        ],
    );
    assert_eq!(app.session.wrap_lines, !original_wrap);
    assert_source_line_copy(&mut app, &ctx, (2, 0), "    let café = 1;");
}

#[test]
fn toolbar_left_reaches_pr_dialog_and_escape_returns_to_source() {
    let (_dir, mut app, ctx) = source_reader();
    open_toolbar_menu(&mut app, &ctx, "View");
    frame(
        &mut app,
        &ctx,
        vec![
            key(Key::ArrowLeft, Modifiers::NONE),
            key(Key::Home, Modifiers::NONE),
            key(Key::Enter, Modifiers::NONE),
        ],
    );
    frame(&mut app, &ctx, vec![]);
    assert!(matches!(app.modal, ActiveModal::PullRequest(_)));
    frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    frame(&mut app, &ctx, vec![]); // Replay input held during the window's sizing pass.
    assert!(matches!(app.modal, ActiveModal::None));
    assert!(app.source_viewport.has_focus(&ctx));
    assert!(app.pending_pr.is_none());
}

#[test]
fn focused_toolbar_menu_keeps_global_definition_shortcuts_available() {
    let (_dir, mut app) = reader_at_indexed_call();
    let ctx = Context::default();
    open_toolbar_menu(&mut app, &ctx, "View");
    assert!(crate::ui::context_menu::is_open(&ctx));
    frame(
        &mut app,
        &ctx,
        vec![key(Key::J, Modifiers::CTRL | Modifiers::ALT)],
    );
    settle(&mut app);
    frame(&mut app, &ctx, vec![]);
    assert!(!crate::ui::context_menu::is_open(&ctx));
    assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
    assert_eq!((app.caret_line, app.caret_col), (1, 3));
    assert!(app.source_viewport.has_focus(&ctx));
}

#[test]
fn source_commands_preserve_coalesced_bookmark_move_and_copy_order() {
    for scale in [1.0, 1.5, 2.0] {
        for focused in [false, true] {
            let (_dir, mut app) = reader_with_source("// é🦀漢字 SAME_LINE\r\n");
            app.jump_to_location("a.rs", 1, 3);
            settle(&mut app);
            let ctx = Context::default();
            ctx.set_pixels_per_point(scale);
            frame(&mut app, &ctx, vec![]);
            if focused {
                app.source_viewport.request_focus(&ctx);
            } else if let Some(id) = ctx.memory(|memory| memory.focused()) {
                ctx.memory_mut(|memory| memory.surrender_focus(id));
            }
            frame(
                &mut app,
                &ctx,
                vec![
                    key(Key::D, Modifiers::CTRL),
                    key(Key::ArrowRight, Modifiers::NONE),
                    key(Key::D, Modifiers::CTRL),
                    key(Key::End, Modifiers::SHIFT),
                    Event::Copy,
                ],
            );
            assert_eq!(app.bookmarks.bookmarks.len(), 1);
            frame(&mut app, &ctx, vec![]);
            assert_eq!(
                app.bookmarks
                    .bookmarks
                    .iter()
                    .map(|entry| entry.byte_offset)
                    .collect::<Vec<_>>(),
                vec![3, 5],
                "scale={scale}, focused={focused}"
            );
            let output = frame(&mut app, &ctx, vec![]);
            assert_eq!(output.platform_output.copied_text, "🦀漢字 SAME_LINE");
            assert!(
                !app.current_bookmarked(),
                "selection ends beyond both saved columns"
            );
        }
    }
}

fn source_at_indexed_gap() -> (tempfile::TempDir, RadiologyApp, Context) {
    let (dir, mut app) = reader_at_indexed_call();
    let ctx = Context::default();
    app.caret_line = 2;
    app.caret_col = 3;
    frame(&mut app, &ctx, vec![]);
    app.source_viewport.request_focus(&ctx);
    (dir, app, ctx)
}

#[test]
fn source_commands_resolve_references_after_preceding_movement() {
    let (_dir, mut app, ctx) = source_at_indexed_gap();
    frame(
        &mut app,
        &ctx,
        vec![
            key(Key::ArrowRight, Modifiers::NONE),
            key(Key::R, Modifiers::CTRL | Modifiers::ALT),
        ],
    );
    assert_eq!((app.caret_line, app.caret_col), (2, 4));
    assert_eq!(app.reference_context.target().unwrap().name, "alpha");
    assert_eq!(app.selected_path.as_deref(), Some("b.rs"));
}

#[test]
fn source_command_navigation_retires_the_remaining_source_batch() {
    let (_dir, mut app, ctx) = source_at_indexed_gap();
    frame(
        &mut app,
        &ctx,
        vec![
            key(Key::ArrowRight, Modifiers::NONE),
            key(Key::J, Modifiers::CTRL | Modifiers::ALT),
            key(Key::D, Modifiers::CTRL),
        ],
    );
    settle(&mut app);
    frame(&mut app, &ctx, vec![]);
    assert_eq!(app.selected_path.as_deref(), Some("a.rs"));
    assert_eq!((app.caret_line, app.caret_col), (1, 3));
    assert!(
        app.bookmarks.bookmarks.is_empty(),
        "navigation must retire later source commands"
    );
}

#[test]
fn escape_cancels_a_bookmark_drag_without_navigation_or_reordering() {
    use crate::ui::test_support::{click, text_center};
    let (_dir, mut app, ctx) = source_reader();
    let buffer = app.open_buffer.clone().unwrap();
    app.bookmarks.bookmarks = (0..30)
        .map(|index| {
            crate::bookmarks::bookmark::SourceBookmark::new(&buffer, 0, format!("drag-{index:02}"))
        })
        .collect();
    let original = app.bookmarks.bookmarks.clone();
    let history = format!("{:?}", app.history);
    frame(&mut app, &ctx, vec![]);
    let output = frame(&mut app, &ctx, vec![]);
    let from = text_center(&output, "drag-00");
    let to = text_center(&output, "drag-01");
    let mut press = click(from);
    press.pop();
    frame(&mut app, &ctx, press);
    frame(&mut app, &ctx, vec![Event::PointerMoved(to)]);
    assert!(crate::ui::bookmark_list::BookmarkList::has_focus(&ctx));
    assert!(egui::DragAndDrop::has_any_payload(&ctx));
    frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    assert!(!egui::DragAndDrop::has_any_payload(&ctx));
    assert!(app.bookmark_list.selected.is_none());
    frame(&mut app, &ctx, vec![click(to).pop().unwrap()]);
    assert_eq!(app.bookmarks.bookmarks, original);
    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
    assert_eq!(format!("{:?}", app.history), history);
}

#[test]
fn bookmark_empty_states_follow_repository_source_and_last_entry_transitions() {
    use crate::ui::test_support::{text_center, text_shapes};
    for scale in [1.0, 1.5, 2.0] {
        let (dir, mut app, ctx, output) = welcome_frame(scale);
        text_center(&output, "No Repository");
        assert!(!text_shapes(&output).any(|text| text.galley.job.text == "Checkout"));
        assert!(!app.command_enabled(&ToolbarAction::ToggleBookmark));
        frame(&mut app, &ctx, vec![key(Key::D, Modifiers::CTRL)]);
        assert!(app.bookmarks.bookmarks.is_empty());
        open_empty_repository(&mut app, &dir.path().join("empty"));
        assert!(app.open_buffer.is_none());
        let output = frame(&mut app, &ctx, vec![]);
        text_center(&output, "Checkout");
        text_center(&output, "No Bookmarks");
        assert!(!app.command_enabled(&ToolbarAction::ToggleBookmark));
        let populated = dir.path().join("populated");
        repository(&populated);
        app.open_repository(&populated);
        settle(&mut app);
        let output = frame(&mut app, &ctx, vec![]);
        text_center(&output, "No Bookmarks");
        assert!(app.command_enabled(&ToolbarAction::ToggleBookmark));
        frame(&mut app, &ctx, vec![key(Key::D, Modifiers::CTRL)]);
        let output = frame(&mut app, &ctx, vec![]);
        assert_eq!(app.bookmarks.bookmarks.len(), 1);
        assert!(!text_shapes(&output).any(|text| text.galley.job.text == "No Bookmarks"));
        frame(&mut app, &ctx, vec![key(Key::D, Modifiers::CTRL)]);
        let output = frame(&mut app, &ctx, vec![]);
        text_center(&output, "No Bookmarks");
        assert!(app.bookmarks.bookmarks.is_empty());
    }
}

fn focused_source_frame(app: &mut RadiologyApp, scale: f32) -> (Context, egui::FullOutput) {
    let ctx = Context::default();
    ctx.set_pixels_per_point(scale);
    frame(app, &ctx, vec![]);
    app.source_viewport.request_focus(&ctx);
    let output = frame(app, &ctx, vec![]);
    (ctx, output)
}

fn source_text_point(
    ctx: &Context,
    output: &egui::FullOutput,
    needle: &str,
    offset: usize,
) -> egui::Pos2 {
    let rect = ctx
        .read_response(crate::ui::source_view::SourceViewport::focus_id())
        .unwrap()
        .rect;
    let text = crate::ui::test_support::text_shapes(output)
        .find(|text| rect.contains(text.pos) && text.galley.job.text.contains(needle))
        .unwrap();
    let byte = text.galley.job.text.find(needle).unwrap() + offset;
    let cursor = text.galley.from_ccursor(egui::text::CCursor::new(
        text.galley.job.text[..byte].chars().count(),
    ));
    text.pos + text.galley.pos_from_cursor(&cursor).center().to_vec2()
}

#[test]
fn source_pointer_and_keyboard_batches_preserve_bookmark_and_copy_order() {
    for scale in [1.0, 1.5, 2.0] {
        for case in ["bookmark-after", "bookmark-before", "copy", "drag-copy"] {
            let (_dir, mut app) = reader_with_source("// é🦀漢字 FIRST\r\n// café SECOND\r\n");
            app.jump_to_location("a.rs", 1, 3);
            settle(&mut app);
            let (ctx, output) = focused_source_frame(&mut app, scale);
            let start = source_text_point(&ctx, &output, "café", 0);
            let end = source_text_point(&ctx, &output, "SECOND", "SECOND".len());
            let target = app.open_buffer.as_ref().unwrap().text.find("café").unwrap();
            let mut events = crate::ui::test_support::click(start);
            match case {
                "bookmark-after" => events.push(key(Key::D, Modifiers::CTRL)),
                "bookmark-before" => events.insert(0, key(Key::D, Modifiers::CTRL)),
                "copy" => events.extend([key(Key::End, Modifiers::SHIFT), Event::Copy]),
                "drag-copy" => {
                    events.pop();
                    events.extend([
                        Event::PointerMoved(end),
                        Event::PointerButton {
                            pos: end,
                            button: egui::PointerButton::Primary,
                            pressed: false,
                            modifiers: Modifiers::NONE,
                        },
                        Event::Copy,
                    ]);
                }
                _ => unreachable!(),
            }
            let mut copied = frame(&mut app, &ctx, events).platform_output.copied_text;
            assert!(
                app.deferred_input.is_empty(),
                "source-only input acquired extra latency"
            );
            for _ in 0..3 {
                let output = frame(&mut app, &ctx, vec![]);
                if !output.platform_output.copied_text.is_empty() {
                    copied = output.platform_output.copied_text;
                }
            }
            match case {
                "bookmark-after" | "bookmark-before" => {
                    assert_eq!(app.bookmarks.bookmarks.len(), 1);
                    assert_eq!(
                        app.bookmarks.bookmarks[0].byte_offset,
                        if case == "bookmark-after" { target } else { 3 },
                        "{scale}x {case}"
                    );
                    assert_eq!((app.caret_line, app.caret_col), (2, 3));
                }
                _ => assert_eq!(copied, "café SECOND", "{scale}x {case}"),
            }
        }
    }
}

#[test]
fn indexed_commands_after_a_source_click_resolve_the_clicked_symbol_and_history() {
    for scale in [1.0, 2.0] {
        for command in [ToolbarAction::FindReferences, ToolbarAction::GoToDefinition] {
            let (_dir, mut app) = reader_with_reference_index();
            app.jump_to_location("b.rs", 1, 4);
            settle(&mut app);
            let buffer = app.open_buffer.as_ref().unwrap();
            app.fold_store.reveal_byte(
                buffer.text.find("alpha").unwrap(),
                buffer,
                &app.fold_ranges,
            );
            let (ctx, output) = focused_source_frame(&mut app, scale);
            let point = source_text_point(&ctx, &output, "alpha", 0);
            let mut events = crate::ui::test_support::click(point);
            events.push(match command {
                ToolbarAction::FindReferences => key(
                    Key::R,
                    Modifiers {
                        ctrl: true,
                        alt: true,
                        ..Default::default()
                    },
                ),
                _ => key(Key::F12, Modifiers::NONE),
            });
            frame(&mut app, &ctx, events);
            for _ in 0..3 {
                frame(&mut app, &ctx, vec![]);
            }
            settle(&mut app);
            if command == ToolbarAction::FindReferences {
                assert_eq!(app.reference_context.explicit().unwrap().name, "alpha");
                assert_eq!(
                    (app.selected_path.as_deref(), app.caret_line, app.caret_col),
                    (Some("b.rs"), 2, 4)
                );
            } else {
                assert_eq!(
                    (app.selected_path.as_deref(), app.caret_line, app.caret_col),
                    (Some("a.rs"), 1, 3)
                );
                app.execute_command(&ctx, ToolbarAction::NavigateBack);
                settle(&mut app);
                assert_eq!(
                    (app.selected_path.as_deref(), app.caret_line, app.caret_col),
                    (Some("b.rs"), 2, 4)
                );
            }
        }
    }
}

#[test]
fn current_function_toggle_owns_only_source_presses_and_committed_jumps_expand_bodies() {
    let text = "fn outer() {\n fn inner() {\n  work();\n }\n}\nfn sibling() {\n other();\n}\n";
    for scale in [1.0, 2.0] {
        let (_dir, mut app, ctx) = scaled_reader(text, scale);
        let buffer = app.open_buffer.clone().unwrap();
        let inner = text.find("fn inner").unwrap();
        let body = crate::syntax::folds::enclosing_function_fold(
            inner,
            &app.outline_symbols,
            &app.fold_ranges,
        )
        .unwrap()
        .byte_range
        .clone();
        app.caret_line = buffer.line_at_byte(inner);
        app.caret_col = buffer.column_at_byte(inner);
        app.source_viewport.request_focus(&ctx);
        frame(&mut app, &ctx, vec![]);
        let history = format!("{:?}", app.history);
        frame(
            &mut app,
            &ctx,
            vec![key(Key::X, Modifiers::NONE), Event::Text("x".into())],
        );
        assert!(!app.fold_store.is_folded(&body));
        assert_eq!(buffer.byte_offset(app.caret_line, app.caret_col), inner);
        assert_eq!(format!("{:?}", app.history), history);
        let outer = crate::syntax::folds::enclosing_function_fold(
            0,
            &app.outline_symbols,
            &app.fold_ranges,
        )
        .unwrap()
        .byte_range
        .clone();
        app.fold_store.toggle(outer.clone());
        frame(&mut app, &ctx, vec![key(Key::X, Modifiers::NONE)]);
        assert!(!app.fold_store.is_folded(&outer));
        assert!(!app.fold_store.is_folded(&body));
        let mut repeat = key(Key::X, Modifiers::NONE);
        if let Event::Key { repeat: value, .. } = &mut repeat {
            *value = true;
        }
        frame(&mut app, &ctx, vec![repeat]);
        assert!(!app.fold_store.is_folded(&body));
        frame(&mut app, &ctx, vec![key(Key::X, Modifiers::SHIFT)]);
        assert!(!app.fold_store.is_folded(&body));
        frame(
            &mut app,
            &ctx,
            vec![Event::CompositionStart, key(Key::X, Modifiers::NONE)],
        );
        frame(&mut app, &ctx, vec![key(Key::X, Modifiers::NONE)]);
        assert!(!app.fold_store.is_folded(&body));
        frame(
            &mut app,
            &ctx,
            vec![
                Event::CompositionEnd("x".into()),
                key(Key::X, Modifiers::NONE),
            ],
        );
        assert!(app.fold_store.is_folded(&body));
        frame(&mut app, &ctx, vec![]);
        assert!(app.fold_store.is_folded(&body));
        app.queue_rehighlight();
        settle(&mut app);
        assert!(app.fold_store.is_folded(&body));
        app.navigate(buffer.location_at_byte(inner), true);
        settle(&mut app);
        assert!(!app.fold_store.is_folded(&body));
        assert_eq!(buffer.byte_offset(app.caret_line, app.caret_col), inner);
        let sibling = text.find("fn sibling").unwrap();
        let other = crate::syntax::folds::enclosing_function_fold(
            sibling,
            &app.outline_symbols,
            &app.fold_ranges,
        )
        .unwrap();
        assert!(app.fold_store.is_folded(&other.byte_range));
        app.execute_command(&ctx, ToolbarAction::OpenSearch);
        frame(&mut app, &ctx, vec![]);
        let folds = app.fold_store.folded_ranges.clone();
        frame(
            &mut app,
            &ctx,
            vec![key(Key::X, Modifiers::NONE), Event::Text("x".into())],
        );
        assert_eq!(app.search_options.query, "x");
        assert_eq!(app.fold_store.folded_ranges, folds);
    }
}

#[test]
fn unicode_file_queries_commit_original_paths_and_preserve_reader_bytes() {
    for scale in [1.0, 2.0] {
        let (owner, mut app) = opened_repository();
        let root = owner.path().join("repo");
        let cases = [
            ("strasse", "src/Straße.rs"),
            ("οσ", "src/ος.rs"),
            ("office", "src/oﬃce.rs"),
            ("test", "src/Teſt.rs"),
            ("cafe", "src/CAFE\u{301}.rs"),
        ];
        for (_, path) in cases {
            std::fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
            std::fs::write(root.join(path), format!("// exact {path} café 🦀\n")).unwrap();
        }
        app.open_repository(&root);
        settle(&mut app);
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        for (query, path) in cases {
            app.open_file("a.rs");
            settle_frame(&mut app, &ctx);
            frame(&mut app, &ctx, vec![key(Key::K, Modifiers::CTRL)]);
            frame(
                &mut app,
                &ctx,
                vec![Event::Text(query.into()), key(Key::Enter, Modifiers::NONE)],
            );
            finish_file_picker(&mut app, &ctx);
            settle_frame(&mut app, &ctx);
            assert_eq!(app.selected_path.as_deref(), Some(path));
            assert!(app.source_viewport.has_focus(&ctx));
            let output = frame(
                &mut app,
                &ctx,
                vec![key(Key::A, Modifiers::CTRL), Event::Copy],
            );
            assert_eq!(
                output.platform_output.copied_text,
                format!("// exact {path} café 🦀\n")
            );
            assert_eq!(
                std::fs::read_to_string(root.join(path)).unwrap(),
                app.open_buffer.as_ref().unwrap().text,
            );
        }
    }
}
