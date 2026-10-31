use super::*;
use crate::ui::test_support::{click, painted_frame, text_center, themed_context};

#[test]
fn caller_names_files_notes_and_depth_keep_their_slots_in_narrow_panes() {
    let mut tree = tree(2);
    tree.rows[0].name = format!("λ{}_TAIL", "caller-🦀".repeat(1000));
    tree.rows[0].location.path = format!("nested/{}_FILE.rs", "long-filename-".repeat(50));
    tree.rows[0].line = 1234;
    tree.rows[0].depth = 17;
    tree.rows[0].note = Some(format!("Boundary {}_NOTE", "details ".repeat(30)));
    for dark in [true, false] {
        for scale in [1.0, 2.0] {
            for width in [216.0, 300.0, 420.0] {
                let (ctx, theme) = themed_context(dark, scale);
                let render = || {
                    painted_frame(&ctx, egui::vec2(width, 300.0), vec![], |ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        show(ui, &theme, 7, Some(&tree), false, None);
                    })
                };
                render();
                let output = render();
                let labels: Vec<_> = crate::ui::test_support::text_shapes(&output).collect();
                for suffix in ["_TAIL", "_FILE.rs", "_NOTE"] {
                    assert!(
                        labels
                            .iter()
                            .any(|text| text.galley.job.text.ends_with(suffix)),
                        "lost {suffix} at {width}"
                    );
                }
                assert!(labels
                    .iter()
                    .any(|text| text.galley.job.text == "1234 · L17"));
                for text in labels {
                    assert!(text.galley.job.text.len() <= 4096);
                    assert_eq!(text.galley.rows.len(), 1);
                    assert!(
                        text.pos.x >= 8.0 && text.pos.x + text.galley.size().x <= width - 8.0 + 0.1,
                        "text overflow: {:?}",
                        text.galley.job.text
                    );
                }
                let first_name = output
                    .shapes
                    .iter()
                    .find_map(|shape| match &shape.shape {
                        egui::Shape::Text(text) if text.galley.job.text.ends_with("_TAIL") => {
                            Some(text.pos.y + text.galley.size().y / 2.0)
                        }
                        _ => None,
                    })
                    .unwrap();
                assert_eq!(text_center(&output, "caller1").y - first_name, 64.0);
            }
        }
    }
}

#[test]
fn caller_press_retires_when_its_location_or_source_changes() {
    for source_change in [false, true] {
        let ctx = egui::Context::default();
        let mut original = tree(1);
        original
            .source_identities
            .insert("source.rs".into(), "old digest".into());
        let mut replacement = original.clone();
        if source_change {
            replacement
                .source_identities
                .insert("source.rs".into(), "new digest".into());
        } else {
            replacement.rows[0].location.byte_offset = 123;
        }
        let mut action = None;
        let mut render = |events, changed| {
            painted_frame(&ctx, egui::vec2(300.0, 240.0), events, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                action = show(
                    ui,
                    &AppTheme::dark(),
                    7,
                    Some(if changed { &replacement } else { &original }),
                    false,
                    None,
                );
            })
        };
        let initial = render(vec![], false);
        let position = text_center(&initial, "caller0");
        let mut events = click(position);
        let release = events.split_off(2);
        render(events, false);
        render(release, true);
        assert_eq!(
            action, None,
            "retained caller press changed source={source_change}"
        );
        let _ = painted_frame(&ctx, egui::vec2(300.0, 240.0), click(position), |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            action = show(ui, &AppTheme::dark(), 7, Some(&replacement), false, None);
        });
        assert_eq!(action, Some(ReferenceAction::CommitCaller(7, 0)));
    }
}
