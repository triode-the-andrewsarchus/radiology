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
            bg_base: Color32::from_rgb(18, 20, 24),
            bg_surface: Color32::from_rgb(26, 29, 36),
            bg_sidebar: Color32::from_rgb(22, 24, 30),
            bg_gutter: Color32::from_rgb(20, 22, 28),
            bg_active_line: Color32::from_rgba_premultiplied(255, 255, 255, 12),
            border: Color32::from_rgb(44, 48, 58),
            text_primary: Color32::from_rgb(230, 235, 245),
            text_secondary: Color32::from_rgb(160, 168, 185),
            text_muted: Color32::from_rgb(105, 115, 135),
            accent: Color32::from_rgb(96, 165, 250),
            diff_add: Color32::from_rgba_premultiplied(34, 197, 94, 60),
            diff_del: Color32::from_rgba_premultiplied(239, 68, 68, 60),
            diff_mod: Color32::from_rgba_premultiplied(234, 179, 8, 60),
        }
    }

    /// Returns the default light desktop theme.
    #[must_use]
    pub fn light() -> Self {
        Self {
            is_dark: false,
            bg_base: Color32::from_rgb(248, 250, 252),
            bg_surface: Color32::from_rgb(255, 255, 255),
            bg_sidebar: Color32::from_rgb(241, 245, 249),
            bg_gutter: Color32::from_rgb(245, 247, 250),
            bg_active_line: Color32::from_rgba_premultiplied(0, 0, 0, 10),
            border: Color32::from_rgb(226, 232, 240),
            text_primary: Color32::from_rgb(15, 23, 42),
            text_secondary: Color32::from_rgb(71, 85, 105),
            text_muted: Color32::from_rgb(148, 163, 184),
            accent: Color32::from_rgb(37, 99, 235),
            diff_add: Color32::from_rgba_premultiplied(34, 197, 94, 45),
            diff_del: Color32::from_rgba_premultiplied(239, 68, 68, 45),
            diff_mod: Color32::from_rgba_premultiplied(234, 179, 8, 45),
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
