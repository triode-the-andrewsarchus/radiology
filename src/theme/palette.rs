//! Color palettes and typography settings for EGUI desktop rendering.

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
    /// Surface card background color.
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
    /// Returns the default dark desktop theme.
    #[must_use]
    pub fn dark() -> Self {
        Self {
            is_dark: true,
            bg_base: Color32::from_rgb(28, 28, 30),
            bg_surface: Color32::from_rgb(28, 28, 30),
            bg_sidebar: Color32::from_rgb(34, 34, 36),
            bg_gutter: Color32::from_rgb(28, 28, 30),
            bg_active_line: Color32::from_rgba_premultiplied(10, 132, 255, 18),
            border: Color32::from_rgb(56, 56, 58),
            text_primary: Color32::from_rgb(242, 242, 247),
            text_secondary: Color32::from_rgb(174, 174, 178),
            text_muted: Color32::from_rgb(110, 110, 115),
            accent: Color32::from_rgb(10, 132, 255),
            diff_add: Color32::from_rgba_premultiplied(48, 209, 88, 35),
            diff_del: Color32::from_rgba_premultiplied(255, 69, 58, 35),
            diff_mod: Color32::from_rgba_premultiplied(255, 159, 10, 35),
        }
    }

    /// Returns the default light desktop theme.
    #[must_use]
    pub fn light() -> Self {
        Self {
            is_dark: false,
            bg_base: Color32::from_rgb(255, 255, 255),
            bg_surface: Color32::from_rgb(255, 255, 255),
            bg_sidebar: Color32::from_rgb(246, 246, 248),
            bg_gutter: Color32::from_rgb(255, 255, 255),
            bg_active_line: Color32::from_rgba_premultiplied(0, 122, 255, 14),
            border: Color32::from_rgb(229, 229, 234),
            text_primary: Color32::from_rgb(28, 28, 30),
            text_secondary: Color32::from_rgb(99, 99, 102),
            text_muted: Color32::from_rgb(142, 142, 147),
            accent: Color32::from_rgb(0, 122, 255),
            diff_add: Color32::from_rgba_premultiplied(52, 199, 89, 26),
            diff_del: Color32::from_rgba_premultiplied(255, 59, 48, 26),
            diff_mod: Color32::from_rgba_premultiplied(255, 149, 0, 26),
        }
    }

    /// Returns a translucent background highlight color for the given [`OccurrenceColor`].
    #[must_use]
    pub fn occurrence_tint(&self, color: OccurrenceColor) -> Color32 {
        match color {
            OccurrenceColor::Teal => Color32::from_rgba_premultiplied(20, 184, 166, 80),
            OccurrenceColor::Amber => Color32::from_rgba_premultiplied(245, 158, 11, 80),
            OccurrenceColor::Violet => Color32::from_rgba_premultiplied(139, 92, 246, 80),
            OccurrenceColor::Rose => Color32::from_rgba_premultiplied(244, 63, 94, 80),
        }
    }
}
