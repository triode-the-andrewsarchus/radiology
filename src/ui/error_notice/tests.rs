use super::*;
use crate::ui::test_support::{click, text_center as text_position};
use egui::{Event, Pos2, Rect, Vec2};

fn frame(
    notice: &mut ErrorNotice,
    ctx: &Context,
    message: Option<&str>,
    size: Vec2,
    events: Vec<Event>,
    theme: &AppTheme,
) -> (egui::FullOutput, bool) {
    let mut dismissed = false;
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            events,
            ..Default::default()
        },
        |ctx| dismissed = notice.show(ctx, theme, message),
    );
    (output, dismissed)
}

#[test]
fn notices_stay_one_row_for_long_unicode_and_multiline_diagnostics_at_narrow_sizes() {
    let message = format!(
        "Source failed: {}\n{}",
        "café 🦀 ".repeat(200),
        "compiler details\n".repeat(2000)
    );
    assert!(summary(&message).chars().count() <= 241);
    assert!(!summary(&message).contains('\n'));
    assert_eq!(summary("\n  \n"), "An operation failed.");
    for theme in [AppTheme::dark(), AppTheme::light()] {
        for (width, scale) in [(220.0, 1.0), (400.0, 1.5), (1000.0, 2.0)] {
            let ctx = Context::default();
            ctx.set_pixels_per_point(scale);
            ctx.set_visuals(if theme.is_dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            let mut notice = ErrorNotice::default();
            let (output, dismissed) = frame(
                &mut notice,
                &ctx,
                Some(&message),
                egui::vec2(width, 600.0),
                vec![],
                &theme,
            );
            assert!(!dismissed);
            assert!(
                ctx.used_rect().height() <= 40.0,
                "notice expanded: {:?}",
                ctx.used_rect()
            );
            let summary_x = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Text(text)
                        if text.galley.job.text.starts_with("Source failed:") =>
                    {
                        Some(text.pos.x)
                    }
                    _ => None,
                })
                .expect("summary must be painted");
            assert!(
                summary_x <= 12.0,
                "summary must align with reader notices: {summary_x}"
            );
            text_position(&output, "Dismiss");
            text_position(&output, "Details…");
        }
    }
}

#[test]
fn details_preserve_and_copy_the_complete_message_and_dismissal_closes_them() {
    let ctx = Context::default();
    let mut notice = ErrorNotice::default();
    let theme = AppTheme::dark();
    let size = egui::vec2(800.0, 600.0);
    let message = format!("Read failed\n{}FINAL café 🦀", "diagnostics\n".repeat(100));
    let (output, _) = frame(&mut notice, &ctx, Some(&message), size, vec![], &theme);
    let details = text_position(&output, "Details…");
    frame(
        &mut notice,
        &ctx,
        Some(&message),
        size,
        click(details),
        &theme,
    );
    assert!(notice.details_open);
    let (output, _) = frame(&mut notice, &ctx, Some(&message), size, vec![], &theme);
    let copy = text_position(&output, "Copy Details");
    let (output, _) = frame(&mut notice, &ctx, Some(&message), size, click(copy), &theme);
    assert_eq!(output.platform_output.copied_text, message);
    let dismiss = text_position(&output, "Dismiss");
    let (_, dismissed) = frame(
        &mut notice,
        &ctx,
        Some(&message),
        size,
        click(dismiss),
        &theme,
    );
    assert!(dismissed);
    assert!(!notice.details_open);
    notice.details_open = true;
    frame(&mut notice, &ctx, None, size, vec![], &theme);
    assert!(!notice.details_open);
}
