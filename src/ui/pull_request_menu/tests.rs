use super::*;
use crate::ui::test_support::{painted_frame, painted_text};

use crate::pull_request::test_support::summary;

fn render(
    menu: &PullRequestMenu<'_>,
    ctx: &egui::Context,
    events: Vec<egui::Event>,
) -> (egui::FullOutput, Option<ReaderCommand>) {
    let mut action = None;
    let output = painted_frame(ctx, egui::vec2(700.0, 600.0), events, |ui| {
        action = menu.contents(ui)
    });
    (output, action)
}

fn click(menu: &PullRequestMenu<'_>, text: &str) -> Option<ReaderCommand> {
    let ctx = egui::Context::default();
    render(menu, &ctx, vec![]);
    let (output, _) = render(menu, &ctx, vec![]);
    let point = output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(label) if label.galley.job.text == text => {
                Some(label.pos + label.galley.size() * 0.5)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing menu label {text}"));
    let mut action = None;
    for pressed in [true, false] {
        action = render(
            menu,
            &ctx,
            vec![
                egui::Event::PointerMoved(point),
                egui::Event::PointerButton {
                    pos: point,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        )
        .1;
    }
    action
}

#[test]
fn menu_routes_recent_fallback_checkout_number_and_independent_refresh_commands() {
    let active = summary(7);
    let mut rows = [summary(3), summary(9)];
    rows[0].state = "MERGED".into();
    rows[1].state = "CLOSED".into();
    let menu = PullRequestMenu {
        active: Some(&active),
        pending: None,
        recent: &rows,
        phase: &PullRequestListPhase::Loaded,
        enabled: true,
    };
    for (label, command) in [
        ("#3  Review 3 · Merged", ReaderCommand::OpenPullRequest(3)),
        ("#9  Review 9 · Closed", ReaderCommand::OpenPullRequest(9)),
        ("Local Checkout", ReaderCommand::ExitPRReview),
        ("Open Pull Request… (Ctrl+P)", ReaderCommand::OpenPRDialog),
        ("Refresh PR (Ctrl+R)", ReaderCommand::RefreshPR),
        ("Refresh List", ReaderCommand::RefreshPRList),
    ] {
        assert_eq!(click(&menu, label), Some(command));
    }
    assert_eq!(
        click(&menu, "#7  Review 7 · Open"),
        None,
        "the active review appears even outside the recent list, without reopening on selection"
    );
    let loading = PullRequestMenu {
        phase: &PullRequestListPhase::Loading,
        ..menu
    };
    assert_eq!(
        click(&loading, "Cancel List Load"),
        Some(ReaderCommand::CancelPRList)
    );
    assert_eq!(click(&loading, "Refresh List"), None);
    assert_eq!(
        click(&loading, "#3  Review 3 · Merged"),
        Some(ReaderCommand::OpenPullRequest(3))
    );
    let disabled = PullRequestMenu {
        enabled: false,
        ..loading
    };
    assert_eq!(click(&disabled, "#3  Review 3 · Merged"), None);
    assert_eq!(click(&disabled, "Open Pull Request… (Ctrl+P)"), None);
}

#[test]
fn menu_reports_empty_loading_failure_and_retention_with_wrapped_titles_and_bounded_errors() {
    let mut active = summary(7);
    active.title = "A long Unicode 🦀 title with several source and review details ".repeat(6);
    let rows = [summary(3)];
    for dark in [true, false] {
        let ctx = egui::Context::default();
        ctx.set_visuals(if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });
        for phase in [
            PullRequestListPhase::Loading,
            PullRequestListPhase::Failed("Authentication required. ".repeat(100)),
            PullRequestListPhase::Cancelled,
        ] {
            let menu = PullRequestMenu {
                active: Some(&active),
                pending: None,
                recent: &rows,
                phase: &phase,
                enabled: true,
            };
            let text = painted_text(&ctx, egui::vec2(400.0, 600.0), |ui| {
                menu.contents(ui);
            });
            assert!(text.iter().any(|text| text == "Showing the previous list."));
            assert!(text
                .iter()
                .any(|text| text.contains("#7") && text.contains("🦀")));
            assert!(text.iter().all(|text| text.chars().count() <= 401));
        }
        let empty = PullRequestMenu {
            active: None,
            pending: None,
            recent: &[],
            phase: &PullRequestListPhase::Loaded,
            enabled: true,
        };
        let text = painted_text(&ctx, egui::vec2(400.0, 600.0), |ui| {
            empty.contents(ui);
        });
        assert!(text.iter().any(|text| text == "No recent pull requests."));
    }
}

#[test]
fn keyboard_selection_reveals_recent_rows_including_the_last_scrolled_choice() {
    let active = summary(7);
    for (active, rows, tabs, expected) in [
        (Some(&active), vec![summary(3), summary(9)], 3, 3),
        // A long list includes the scrollbar as an additional keyboard stop.
        (None, (1..=25).map(summary).collect(), 27, 25),
    ] {
        let ctx = egui::Context::default();
        let menu = PullRequestMenu {
            active,
            pending: None,
            recent: &rows,
            phase: &PullRequestListPhase::Loaded,
            enabled: true,
        };
        render(&menu, &ctx, vec![]);
        for _ in 0..tabs {
            render(
                &menu,
                &ctx,
                vec![egui::Event::Key {
                    key: egui::Key::Tab,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                }],
            );
        }
        for _ in 0..16 {
            render(&menu, &ctx, vec![]);
        }
        let (output, action) = render(
            &menu,
            &ctx,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
        );
        assert_eq!(action, Some(ReaderCommand::OpenPullRequest(expected)));
        assert!(output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text.starts_with(&format!("#{expected}  ")) && shape.clip_rect.contains(text.pos + text.galley.size() * 0.5))), "focused choice should be visible after scroll settles");
    }
}

#[test]
fn toolbar_tooltip_remains_available_when_closed_and_does_not_cover_the_open_popup() {
    let ctx = egui::Context::default();
    ctx.style_mut(|style| style.interaction.tooltip_delay = 0.0);
    let menu = PullRequestMenu {
        active: None,
        pending: None,
        recent: &[],
        phase: &PullRequestListPhase::Loaded,
        enabled: true,
    };
    let paint = |events| {
        painted_frame(&ctx, egui::vec2(700.0, 600.0), events, |ui| {
            menu.show(ui);
        })
    };
    let output = paint(vec![]);
    let point = output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == "Pull Request" => {
                Some(text.pos + text.galley.size() * 0.5)
            }
            _ => None,
        })
        .unwrap();
    paint(vec![egui::Event::PointerMoved(point)]);
    for _ in 0..3 {
        paint(vec![]);
    }
    let help = "Review a recent pull request or open a number (Ctrl+P).";
    let output = paint(vec![]);
    assert!(output.shapes.iter().any(
        |shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == help)
    ));
    for pressed in [true, false] {
        paint(vec![egui::Event::PointerButton {
            pos: point,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        }]);
    }
    let output = paint(vec![]);
    assert!(output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == "Local Checkout")));
    assert!(!output.shapes.iter().any(
        |shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == help)
    ));
}
