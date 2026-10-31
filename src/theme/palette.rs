//! Semantic reader colors and their EGUI widget mapping.

use crate::source::selection::OccurrenceColor;
use egui::Color32;

/// Theme color definitions for panels, typography, gutters, and diff indicators.
///
/// # Examples
///
/// ```rust
/// use radiology::theme::AppTheme;
///
/// let dark = AppTheme::dark();
/// assert!(dark.is_dark);
///
/// let light = AppTheme::light();
/// assert!(!light.is_dark);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct AppTheme {
    /// Whether this theme represents a dark appearance.
    pub is_dark: bool,
    /// Base background color.
    pub bg_base: Color32,
    /// Raised control surface color.
    pub bg_surface: Color32,
    /// Sidebar panel background color.
    pub bg_sidebar: Color32,
    /// Source line gutter background color.
    pub bg_gutter: Color32,
    /// Subtle background tint for the active caret line.
    pub bg_active_line: Color32,
    /// Divider and border line color.
    pub border: Color32,
    /// Primary high-contrast text color.
    pub text_primary: Color32,
    /// Secondary supporting text color.
    pub text_secondary: Color32,
    /// Muted text color for comments and line numbers.
    pub text_muted: Color32,
    /// Accent color used for focus, links, and badges.
    pub accent: Color32,
    /// Background highlight for added diff lines.
    pub diff_add: Color32,
    /// Background highlight for deleted diff lines.
    pub diff_del: Color32,
    /// Background highlight for modified diff lines.
    pub diff_mod: Color32,
}

impl AppTheme {
    /// Returns widget visuals derived from the reader's semantic colors.
    ///
    /// Hosts opt in with [`egui::Context::set_visuals`]; this does not change
    /// typography, spacing, zoom or context state. Selection uses an opaque,
    /// restrained accent tint so custom-painted rows retain readable text.
    ///
    /// # Examples
    ///
    /// ```
    /// use radiology::theme::AppTheme;
    ///
    /// let theme = AppTheme::light();
    /// let context = egui::Context::default();
    /// context.set_visuals(theme.visuals());
    /// assert_eq!(context.style().visuals.panel_fill, theme.bg_sidebar);
    /// ```
    #[must_use]
    pub fn visuals(&self) -> egui::Visuals {
        let mut visuals = if self.is_dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.panel_fill = self.bg_sidebar;
        visuals.window_fill = self.bg_sidebar;
        visuals.window_stroke = egui::Stroke::new(1.0_f32, self.border);
        visuals.extreme_bg_color = self.bg_base;
        visuals.code_bg_color = self.bg_base;
        visuals.faint_bg_color = self.bg_sidebar;
        visuals.hyperlink_color = self.accent;
        visuals.text_cursor.color = self.accent;
        visuals.selection.bg_fill = accent_tint(self.bg_sidebar, self.accent, 0.12);
        visuals.selection.stroke.color = self.accent;
        visuals.warn_fg_color = if self.is_dark {
            Color32::from_rgb(245, 198, 93)
        } else {
            Color32::from_rgb(122, 73, 0)
        };
        visuals.error_fg_color = if self.is_dark {
            Color32::from_rgb(255, 128, 120)
        } else {
            Color32::from_rgb(173, 36, 32)
        };
        for widget in [
            &mut visuals.widgets.noninteractive,
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
            &mut visuals.widgets.open,
        ] {
            widget.bg_fill = self.bg_surface;
            widget.weak_bg_fill = self.bg_surface;
            widget.bg_stroke.color = self.border;
            widget.fg_stroke.color = self.text_primary;
        }
        visuals.widgets.noninteractive.bg_fill = self.bg_sidebar;
        visuals.widgets.noninteractive.weak_bg_fill = self.bg_base;
        for widget in [
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
            &mut visuals.widgets.open,
        ] {
            widget.bg_fill = visuals.selection.bg_fill;
            widget.weak_bg_fill = visuals.selection.bg_fill;
            widget.bg_stroke.color = self.accent;
        }
        visuals
    }

    /// Returns the neutral wash for the caret's enclosing callable.
    #[must_use]
    pub fn function_shade_color(&self) -> egui::Color32 {
        self.text_primary
            .gamma_multiply(if self.is_dark { 0.045 } else { 0.03 })
    }

    /// Returns the default dark desktop theme.
    #[must_use]
    pub fn dark() -> Self {
        Self {
            is_dark: true,
            bg_base: Color32::from_gray(31),
            bg_surface: Color32::from_gray(56),
            bg_sidebar: Color32::from_gray(44),
            bg_gutter: Color32::from_gray(31),
            // Five-percent sRGB accent mix, precomposited to avoid gamma-amplified washes.
            bg_active_line: Color32::from_rgb(33, 39, 42),
            border: Color32::from_gray(140),
            text_primary: Color32::from_gray(242),
            text_secondary: Color32::from_gray(213),
            text_muted: Color32::from_gray(181),
            accent: Color32::from_rgb(66, 185, 243),
            diff_add: Color32::from_rgba_unmultiplied(48, 209, 88, 35),
            diff_del: Color32::from_rgba_unmultiplied(255, 69, 58, 35),
            diff_mod: Color32::from_rgba_unmultiplied(255, 159, 10, 35),
        }
    }

    /// Returns the default light desktop theme.
    #[must_use]
    pub fn light() -> Self {
        Self {
            is_dark: false,
            bg_base: Color32::from_gray(238),
            bg_surface: Color32::WHITE,
            bg_sidebar: Color32::from_gray(227),
            bg_gutter: Color32::from_gray(238),
            bg_active_line: Color32::from_rgb(228, 231, 233),
            border: Color32::from_gray(112),
            text_primary: Color32::from_gray(38),
            text_secondary: Color32::from_gray(72),
            text_muted: Color32::from_gray(89),
            accent: Color32::from_rgb(33, 92, 132),
            diff_add: Color32::from_rgba_unmultiplied(52, 199, 89, 26),
            diff_del: Color32::from_rgba_unmultiplied(255, 59, 48, 26),
            diff_mod: Color32::from_rgba_unmultiplied(255, 149, 0, 26),
        }
    }

    /// Returns the base palette color for the given [`OccurrenceColor`].
    #[must_use]
    pub fn occurrence_color(&self, color: OccurrenceColor) -> Color32 {
        match color {
            OccurrenceColor::Teal => Color32::from_rgb(20, 184, 166),
            OccurrenceColor::Amber => Color32::from_rgb(245, 158, 11),
            OccurrenceColor::Violet => Color32::from_rgb(139, 92, 246),
            OccurrenceColor::Rose => Color32::from_rgb(244, 63, 94),
        }
    }
}

// Mix in sRGB before painting: a translucent wash would vary with the receiving
// surface and egui's blending. All built-in chrome surfaces are opaque.
fn accent_tint(base: Color32, accent: Color32, strength: f32) -> Color32 {
    let [r, g, b, _] = std::array::from_fn(|i| {
        egui::lerp(base[i] as f32..=accent[i] as f32, strength).round() as u8
    });
    Color32::from_rgb(r, g, b)
}

#[cfg(test)]
mod tests;
