use super::*;

fn luminance(color: Color32) -> f32 {
    let linear = |value: u8| {
        let value = value as f32 / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(color.r()) + 0.7152 * linear(color.g()) + 0.0722 * linear(color.b())
}

fn contrast(a: Color32, b: Color32) -> f32 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[test]
fn interface_text_remains_readable_on_surfaces_selection_and_widget_states() {
    for theme in [AppTheme::light(), AppTheme::dark()] {
        let visuals = theme.visuals();
        let surfaces = [
            theme.bg_base,
            theme.bg_surface,
            theme.bg_sidebar,
            theme.bg_gutter,
            theme.bg_active_line,
            visuals.selection.bg_fill,
        ];
        for surface in surfaces {
            assert_eq!(surface.a(), 255);
            for foreground in [theme.text_primary, theme.text_secondary, theme.text_muted] {
                assert!(
                    contrast(foreground, surface) >= 4.5,
                    "dark={}: {foreground:?} on {surface:?} = {}",
                    theme.is_dark,
                    contrast(foreground, surface)
                );
            }
        }
        for widget in [
            visuals.widgets.noninteractive,
            visuals.widgets.inactive,
            visuals.widgets.hovered,
            visuals.widgets.active,
            visuals.widgets.open,
        ] {
            for background in [widget.bg_fill, widget.weak_bg_fill] {
                assert!(contrast(widget.fg_stroke.color, background) >= 4.5);
            }
        }
        for background in [theme.bg_base, theme.bg_sidebar, theme.bg_surface] {
            for foreground in [
                visuals.hyperlink_color,
                visuals.warn_fg_color,
                visuals.error_fg_color,
            ] {
                assert!(
                    contrast(foreground, background) >= 4.5,
                    "{foreground:?} on {background:?}: {}",
                    contrast(foreground, background)
                );
            }
            assert!(contrast(theme.border, background) >= 3.0);
            assert!(contrast(visuals.text_cursor.color, background) >= 3.0);
        }
        assert!(contrast(visuals.selection.stroke.color, visuals.selection.bg_fill) >= 4.5);
    }
}

#[test]
fn host_colors_drive_visuals_without_mutating_context_or_typography() {
    let context = egui::Context::default();
    let before = context.style();
    let mut theme = AppTheme::light();
    theme.bg_sidebar = Color32::from_rgb(240, 235, 230);
    theme.bg_surface = Color32::from_rgb(255, 250, 245);
    theme.text_primary = Color32::from_rgb(25, 20, 15);
    theme.accent = Color32::from_rgb(85, 40, 110);
    let visuals = theme.visuals();
    assert_eq!(
        context.style().visuals.panel_fill,
        before.visuals.panel_fill
    );
    assert_eq!(visuals.panel_fill, theme.bg_sidebar);
    assert_eq!(visuals.window_fill, theme.bg_sidebar);
    assert_eq!(visuals.widgets.inactive.bg_fill, theme.bg_surface);
    assert_eq!(visuals.widgets.inactive.fg_stroke.color, theme.text_primary);
    assert_eq!(visuals.hyperlink_color, theme.accent);
    assert_ne!(
        visuals.selection.bg_fill,
        AppTheme::light().visuals().selection.bg_fill
    );
    context.set_visuals(visuals);
    assert_eq!(context.style().text_styles, before.text_styles);
    assert_eq!(context.style().spacing, before.spacing);
    assert_eq!(context.style().visuals.panel_fill, theme.bg_sidebar);
}
