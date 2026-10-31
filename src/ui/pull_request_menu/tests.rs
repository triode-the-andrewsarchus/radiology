use super::*;
use crate::ui::test_support::{menu_frame, painted_frame, text_center};

use crate::pull_request::test_support::summary;

fn render(
    menu: &PullRequestMenu<'_>,
    ctx: &egui::Context,
    events: Vec<egui::Event>,
) -> (egui::FullOutput, Option<ReaderCommand>) {
    let mut action = None;
    let output = menu_frame(ctx, egui::vec2(700.0, 600.0), events, |ui| {
        action = menu.show(ui)
    });
    (output, action)
}

fn open(menu: &PullRequestMenu<'_>, ctx: &egui::Context) -> egui::FullOutput {
    let (output, _) = render(menu, ctx, vec![]);
    let title = menu
        .pending
        .map(|number| format!("#{number}…"))
        .or_else(|| menu.active.map(|pr| format!("#{}", pr.number)))
        .unwrap_or_else(|| "Pull Request".into());
    render(
        menu,
        ctx,
        crate::ui::test_support::click(text_center(&output, &title)),
    );
    render(menu, ctx, vec![]).0
}

fn text(output: &egui::FullOutput) -> Vec<String> {
    output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect()
}

fn click(menu: &PullRequestMenu<'_>, text: &str) -> Option<ReaderCommand> {
    let ctx = egui::Context::default();
    let output = open(menu, &ctx);
    let point = output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(label) if label.galley.job.text.trim_start_matches("✓ ") == text => {
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
            crate::ui::context_menu::close(&ctx);
            let text = text(&open(&menu, &ctx));
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
        crate::ui::context_menu::close(&ctx);
        let text = text(&open(&empty, &ctx));
        assert!(text.iter().any(|text| text == "No recent pull requests."));
    }
}

#[test]
fn keyboard_selection_reveals_recent_rows_including_the_last_scrolled_choice() {
    let active = summary(7);
    for (active, rows, tabs, expected) in [
        (Some(&active), vec![summary(3), summary(9)], 2, 3),
        // Disabled current choices and scrollbars are not keyboard stops.
        (None, (1..=25).map(summary).collect(), 25, 25),
    ] {
        let ctx = egui::Context::default();
        let menu = PullRequestMenu {
            active,
            pending: None,
            recent: &rows,
            phase: &PullRequestListPhase::Loaded,
            enabled: true,
        };
        open(&menu, &ctx);
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
        let output = settled(&menu, &ctx, 16);
        let (_, action) = render(
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
        assert!(
            visible_choice(&output, expected),
            "focused choice should be visible after scroll settles"
        );
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
    assert!(output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text.trim_start_matches("✓ ") == "Local Checkout")));
    assert!(!output.shapes.iter().any(
        |shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == help)
    ));
}

fn settled(menu: &PullRequestMenu<'_>, ctx: &egui::Context, frames: usize) -> egui::FullOutput {
    for _ in 0..frames {
        render(menu, ctx, vec![]);
    }
    render(menu, ctx, vec![]).0
}

fn visible_choice(output: &egui::FullOutput, number: usize) -> bool {
    output.shapes.iter().any(|shape| {
        matches!(&shape.shape, egui::Shape::Text(text)
        if text.galley.job.text.trim_start_matches("✓ ").starts_with(&format!("#{number}  "))
            && shape.clip_rect.contains(text.pos + text.galley.size() * 0.5))
    })
}

#[test]
fn manual_menu_scrolling_is_not_pulled_back_to_the_keyboard_selection() {
    let ctx = egui::Context::default();
    let rows: Vec<_> = (1..=25).map(summary).collect();
    let menu = PullRequestMenu {
        active: None,
        pending: None,
        recent: &rows,
        phase: &PullRequestListPhase::Loaded,
        enabled: true,
    };
    open(&menu, &ctx);
    render(
        &menu,
        &ctx,
        vec![egui::Event::PointerMoved(egui::pos2(100.0, 180.0))],
    );
    // Wait through the complete smoothed wheel tail: 16 frames would hide the
    // old repeated-reveal bug by checking before manual scroll input expires.
    for (event, frames, visible) in [
        (crate::ui::commands::test_key(egui::Key::Home), 16, true),
        (egui::Event::Scroll(egui::vec2(0.0, -500.0)), 120, false),
        (crate::ui::commands::test_key(egui::Key::Home), 16, true),
    ] {
        render(&menu, &ctx, vec![event]);
        assert_eq!(
            visible_choice(&settled(&menu, &ctx, frames), 1),
            visible,
            "row visibility after {frames} settled frames"
        );
    }
}
