//! Vector SVG icon system for Radiology.
//!
//! Provides resolution-independent vector icons embedded directly into the binary
//! that adapt to the active Light/Dark theme via `egui_extras` SVG loader and `egui::Image::tint`.

use egui::{self, Color32, Rect, Response, Ui, Vec2};

/// Enumeration of vector SVG icons used across Radiology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    /// Dark/Light appearance theme indicator.
    Contrast,
    /// Sun icon for light mode.
    Sun,
    /// Moon icon for dark mode.
    Moon,
    /// Arrow pointing up.
    ArrowUp,
    /// Arrow pointing down.
    ArrowDown,
    /// Arrow pointing left (back navigation).
    ArrowLeft,
    /// Arrow pointing right (forward navigation).
    ArrowRight,
    /// Chevron pointing down (expanded fold/tree).
    ChevronDown,
    /// Chevron pointing right (collapsed fold/tree, breadcrumb separator).
    ChevronRight,
    /// Folder directory icon.
    Folder,
    /// Code source file icon.
    Code,
    /// Text file icon.
    Text,
    /// Search magnifying glass icon.
    Search,
    /// Git diff / changes icon.
    Diff,
    /// Bookmark / pin icon.
    Pin,
    /// Checkmark icon.
    Check,
    /// Close / remove cross icon.
    Close,
    /// Line wrap toggle icon.
    Wrap,
}

impl Icon {
    /// Returns the canonical asset filename (without `.svg` extension).
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Contrast => "contrast",
            Self::Sun => "sun",
            Self::Moon => "moon",
            Self::ArrowUp => "arrow_up",
            Self::ArrowDown => "arrow_down",
            Self::ArrowLeft => "arrow_left",
            Self::ArrowRight => "arrow_right",
            Self::ChevronDown => "chevron_down",
            Self::ChevronRight => "chevron_right",
            Self::Folder => "folder",
            Self::Code => "code",
            Self::Text => "text",
            Self::Search => "search",
            Self::Diff => "diff",
            Self::Pin => "pin",
            Self::Check => "check",
            Self::Close => "x",
            Self::Wrap => "wrap",
        }
    }

    /// Returns the static bytes URI identifying this SVG resource in egui.
    #[inline]
    #[must_use]
    pub const fn uri(&self) -> &'static str {
        match self {
            Self::Contrast => "bytes://icons/contrast.svg",
            Self::Sun => "bytes://icons/sun.svg",
            Self::Moon => "bytes://icons/moon.svg",
            Self::ArrowUp => "bytes://icons/arrow_up.svg",
            Self::ArrowDown => "bytes://icons/arrow_down.svg",
            Self::ArrowLeft => "bytes://icons/arrow_left.svg",
            Self::ArrowRight => "bytes://icons/arrow_right.svg",
            Self::ChevronDown => "bytes://icons/chevron_down.svg",
            Self::ChevronRight => "bytes://icons/chevron_right.svg",
            Self::Folder => "bytes://icons/folder.svg",
            Self::Code => "bytes://icons/code.svg",
            Self::Text => "bytes://icons/text.svg",
            Self::Search => "bytes://icons/search.svg",
            Self::Diff => "bytes://icons/diff.svg",
            Self::Pin => "bytes://icons/pin.svg",
            Self::Check => "bytes://icons/check.svg",
            Self::Close => "bytes://icons/x.svg",
            Self::Wrap => "bytes://icons/wrap.svg",
        }
    }

    /// Returns the raw embedded SVG file bytes.
    #[inline]
    #[must_use]
    pub const fn svg_bytes(&self) -> &'static [u8] {
        match self {
            Self::Contrast => include_bytes!("../../assets/icons/contrast.svg"),
            Self::Sun => include_bytes!("../../assets/icons/sun.svg"),
            Self::Moon => include_bytes!("../../assets/icons/moon.svg"),
            Self::ArrowUp => include_bytes!("../../assets/icons/arrow_up.svg"),
            Self::ArrowDown => include_bytes!("../../assets/icons/arrow_down.svg"),
            Self::ArrowLeft => include_bytes!("../../assets/icons/arrow_left.svg"),
            Self::ArrowRight => include_bytes!("../../assets/icons/arrow_right.svg"),
            Self::ChevronDown => include_bytes!("../../assets/icons/chevron_down.svg"),
            Self::ChevronRight => include_bytes!("../../assets/icons/chevron_right.svg"),
            Self::Folder => include_bytes!("../../assets/icons/folder.svg"),
            Self::Code => include_bytes!("../../assets/icons/code.svg"),
            Self::Text => include_bytes!("../../assets/icons/text.svg"),
            Self::Search => include_bytes!("../../assets/icons/search.svg"),
            Self::Diff => include_bytes!("../../assets/icons/diff.svg"),
            Self::Pin => include_bytes!("../../assets/icons/pin.svg"),
            Self::Check => include_bytes!("../../assets/icons/check.svg"),
            Self::Close => include_bytes!("../../assets/icons/x.svg"),
            Self::Wrap => include_bytes!("../../assets/icons/wrap.svg"),
        }
    }

    /// Creates an `egui::Image` with the exact square dimension.
    pub fn image(&self, size: f32) -> egui::Image<'static> {
        egui::Image::from_bytes(self.uri(), self.svg_bytes()).fit_to_exact_size(Vec2::splat(size))
    }

    /// Creates an `egui::Image` with exact square dimension and color tint.
    pub fn image_tinted(&self, size: f32, tint: Color32) -> egui::Image<'static> {
        self.image(size).tint(tint)
    }

    /// Directly renders the tinted icon onto `ui`.
    pub fn ui(&self, ui: &mut Ui, size: f32, tint: Color32) -> Response {
        ui.add(self.image_tinted(size, tint))
    }

    /// Renders the icon as a clickable button with hover feedback.
    pub fn button(&self, ui: &mut Ui, size: f32, tint: Color32) -> Response {
        let (rect, response) =
            ui.allocate_exact_size(Vec2::splat(size + 6.0), egui::Sense::click());
        let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(size));
        let icon_color = if response.hovered() {
            tint
        } else {
            tint.linear_multiply(0.85)
        };
        if response.hovered() {
            ui.painter()
                .rect_filled(rect, 3.0, ui.visuals().widgets.hovered.bg_fill);
        }
        self.paint(ui.painter(), icon_rect, icon_color);
        response
    }

    /// Paints an SVG into an existing painter canvas without creating a widget.
    pub fn paint(&self, painter: &egui::Painter, rect: egui::Rect, color: Color32) {
        let pixels_per_point = painter.ctx().pixels_per_point();
        let raster_size = rect.size() * pixels_per_point;
        if let Ok(egui::load::TexturePoll::Ready { texture }) = self
            .image(rect.width() * pixels_per_point)
            .load_for_size(painter.ctx(), raster_size)
        {
            painter.image(
                texture.id,
                rect,
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                color,
            );
        } else {
            painter.ctx().request_repaint();
        }
    }
}
