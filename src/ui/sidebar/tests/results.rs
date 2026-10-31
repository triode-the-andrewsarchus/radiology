use super::*;
use crate::pull_request::change::FileChangeStatus;
use crate::ui::test_support::click;

struct Results {
    pr: PullRequestSummary,
    changes: Vec<FileChange>,
    search: RepositorySearch,
}

impl Results {
    fn new() -> Self {
        let path = format!(
            "{}/very-long-🦀-café-source-name-{}.rs",
            "nested-directory/".repeat(12),
            "context-".repeat(15)
        );
        let mut pr = crate::pull_request::test_support::summary(17);
        pr.title = "AnUnbrokenPullRequestTitle".repeat(40);
        let change = FileChange {
            path: path.clone(),
            old_path: Some(format!("old/{path}")),
            status: FileChangeStatus::Renamed { similarity: 85 },
            before_blob: String::new(),
            after_blob: String::new(),
            additions: 12,
            deletions: 3,
            is_binary: false,
        };
        let binary = FileChange {
            path: "assets/image.dat".into(),
            old_path: None,
            is_binary: true,
            status: FileChangeStatus::Added,
            ..change.clone()
        };
        let mut search = RepositorySearch::default();
        search.matches = (1897..1900)
            .map(|line| SearchMatch {
                path: path.clone(),
                line,
                column: 7,
                byte_offset: line * 100,
                preview: format!("needle café {}", "🦀unbroken-".repeat(1000)),
            })
            .collect();
        Self {
            pr,
            changes: vec![change, binary],
            search,
        }
    }
    fn first_action(&self, tab: SidebarTab) -> SidebarAction {
        if tab == SidebarTab::Changes {
            SidebarAction::OpenFile(self.changes[0].path.clone())
        } else {
            SidebarAction::OpenSearchMatch(self.search.matches[0].clone())
        }
    }
    fn frame(
        &self,
        ctx: &egui::Context,
        tab: SidebarTab,
        width: f32,
        events: Vec<egui::Event>,
    ) -> (egui::FullOutput, Option<SidebarAction>, egui::Rect) {
        let mut action = None;
        let mut pane = egui::Rect::NOTHING;
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1280.0, 900.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                pane = egui::SidePanel::left("result-pane")
                    .default_width(width)
                    .min_width(180.0)
                    .max_width(500.0)
                    .show(ctx, |ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        let theme = if ctx.style().visuals.dark_mode {
                            AppTheme::dark()
                        } else {
                            AppTheme::light()
                        };
                        action = match tab {
                            SidebarTab::Changes => Sidebar::changes(
                                ui,
                                &theme,
                                Some(&self.pr),
                                &ReviewChangesPresentation::new(
                                    &self.changes,
                                    &SourceFilters::default(),
                                ),
                                None,
                                None,
                            ),
                            SidebarTab::Search => Sidebar::search(
                                ui,
                                &theme,
                                &mut SearchOptions::default(),
                                &self.search,
                            ),
                            SidebarTab::Files => unreachable!(),
                        };
                    })
                    .response
                    .rect;
            },
        );
        (output, action, pane)
    }
}

#[test]
fn result_labels_keep_pane_widths_and_compact_search_row_geometry() {
    let mut fixture = Results::new();
    fixture.search.running = true;
    let mut failures = Vec::new();
    for dark in [true, false] {
        for scale in [1.0, 2.0] {
            for width in [180.0, 260.0, 420.0] {
                for tab in [SidebarTab::Changes, SidebarTab::Search] {
                    let ctx = egui::Context::default();
                    ctx.set_visuals(if dark {
                        AppTheme::dark().visuals()
                    } else {
                        AppTheme::light().visuals()
                    });
                    ctx.set_pixels_per_point(scale);
                    let (output, _, pane) = fixture.frame(&ctx, tab, width, vec![]);
                    failures.extend(geometry_failures(&output, pane, tab, width));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn geometry_failures(
    output: &egui::FullOutput,
    pane: egui::Rect,
    tab: SidebarTab,
    width: f32,
) -> Vec<String> {
    let mut failures = Vec::new();
    if pane.width() > width.max(196.0) + 1.0 {
        failures.push(format!("{tab:?} expanded {width} to {}", pane.width()));
    }
    if tab == SidebarTab::Changes {
        for shape in &output.shapes {
            if let egui::Shape::Text(text) = &shape.shape {
                if text.galley.job.text.starts_with("#17") && text.galley.rows.len() > 2 {
                    failures.push(format!("Changes title exceeded two rows at {width}"));
                }
            }
        }
    }
    if tab == SidebarTab::Search {
        let mut tops = Vec::new();
        for shape in &output.shapes {
            if let egui::Shape::Text(text) = &shape.shape {
                if text.galley.job.text.contains(":189") {
                    let first_row: String = text.galley.rows[0]
                        .glyphs
                        .iter()
                        .map(|glyph| glyph.chr)
                        .collect();
                    if !first_row.contains(":189") {
                        failures.push(format!("Search path wrapped at {width}"));
                    }
                    tops.push(text.pos.y);
                }
            }
        }
        if tops.len() != 3 || tops.windows(2).any(|pair| pair[1] - pair[0] > 64.0) {
            failures.push(format!("Search rows are not compact: {tops:?}"));
        }
    }
    failures
}

#[test]
fn fitted_results_keep_exact_navigation_and_binary_change_metadata() {
    let fixture = Results::new();
    for tab in [SidebarTab::Changes, SidebarTab::Search] {
        let ctx = egui::Context::default();
        fixture.frame(&ctx, tab, 260.0, vec![]);
        let (output, _, _) = fixture.frame(&ctx, tab, 260.0, vec![]);
        let point = first_caption(&output, tab);
        let (_, action, _) = fixture.frame(&ctx, tab, 260.0, click(point));
        let expected = fixture.first_action(tab);
        assert_eq!(action, Some(expected));
        if tab == SidebarTab::Changes {
            assert!(output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == "binary")));
        } else {
            let mut previews = 0;
            for shape in &output.shapes {
                if let egui::Shape::Text(text) = &shape.shape {
                    if text.galley.job.text.starts_with("needle") {
                        previews += 1;
                        assert!(text.galley.elided, "long previews should signal truncation");
                        assert!(text.galley.rows.len() <= 2);
                        assert!(
                            text.galley.job.text.len() <= 4099,
                            "shaping must stay bounded"
                        );
                    }
                }
            }
            assert_eq!(previews, fixture.search.matches.len());
        }
    }
}

fn first_caption(output: &egui::FullOutput, tab: SidebarTab) -> egui::Pos2 {
    output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text)
                if match tab {
                    SidebarTab::Changes => {
                        text.galley.job.text.starts_with("very-long-")
                            && text.galley.job.text.ends_with(".rs")
                    }
                    _ => text.galley.job.text.ends_with(":1897"),
                } =>
            {
                Some(text.pos + text.galley.size() / 2.0)
            }
            _ => None,
        })
        .expect("visible result caption")
}

#[test]
fn result_rows_retire_pressed_targets_when_results_reorder() {
    for (tab, stops) in [(SidebarTab::Changes, 1), (SidebarTab::Search, 7)] {
        let mut fixture = Results::new();
        let ctx = egui::Context::default();
        fixture.frame(&ctx, tab, 260.0, vec![]);
        let (output, _, _) = fixture.frame(&ctx, tab, 260.0, vec![]);
        let point = first_caption(&output, tab);
        let mut press = click(point);
        let release = press.pop().unwrap();
        fixture.frame(&ctx, tab, 260.0, press);
        fixture.changes.swap(0, 1);
        fixture.search.matches.swap(0, 1);
        let (_, action, _) = fixture.frame(&ctx, tab, 260.0, vec![release]);
        assert!(
            action.is_none(),
            "{tab:?} retargeted an old press to {action:?}"
        );
        let expected = fixture.first_action(tab);
        assert_eq!(
            fixture.frame(&ctx, tab, 260.0, click(point)).1,
            Some(expected.clone())
        );
        ctx.memory_mut(|memory| memory.stop_text_input());
        for _ in 0..stops {
            fixture.frame(
                &ctx,
                tab,
                260.0,
                vec![crate::ui::commands::test_key(egui::Key::Tab)],
            );
        }
        assert_eq!(
            fixture
                .frame(
                    &ctx,
                    tab,
                    260.0,
                    vec![crate::ui::commands::test_key(egui::Key::Space)]
                )
                .1,
            Some(expected),
            "keyboard activation should remain available"
        );
    }
}
