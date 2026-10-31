//! Portable CJK and monochrome emoji font assets.
//!
//! The assets match little-screen-clip's bundled fallback faces byte for byte.
//! Asset-only hosts need no GUI dependency. Enable `egui` to append them to a
//! host's existing font definitions without changing its primary fonts or sizes.
//! Font software uses SIL OFL 1.1; both notices ship in `assets/`.

/// The bundled Noto Sans CJK JP regular face (SIL OFL 1.1).
pub const CJK_FONT_BYTES: &[u8] = include_bytes!("../assets/NotoSansCJK-JP-Regular.ttf");

/// The bundled Noto Emoji regular monochrome face (SIL OFL 1.1).
pub const EMOJI_FONT_BYTES: &[u8] = include_bytes!("../assets/NotoEmoji-Regular.ttf");

/// Appends portable fallback faces to every existing font family.
///
/// Adds proportional and monospace families when absent. Existing primary fonts
/// retain their order, including custom host families; repeated calls are safe.
/// The host decides when to apply definitions to its context and owns typography.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "egui")] {
/// let mut definitions = egui::FontDefinitions::default();
/// reader_fonts::append_fallbacks(&mut definitions);
/// let context = egui::Context::default();
/// context.set_fonts(definitions);
/// # }
/// ```
#[cfg(feature = "egui")]
pub fn append_fallbacks(definitions: &mut egui::FontDefinitions) {
    use egui::{FontData, FontFamily};
    const FACES: [(&str, &[u8]); 2] = [
        ("reader-fallback-cjk", CJK_FONT_BYTES),
        ("reader-fallback-emoji", EMOJI_FONT_BYTES),
    ];
    for (name, bytes) in FACES {
        definitions
            .font_data
            .entry(name.into())
            .or_insert_with(|| FontData::from_static(bytes));
    }
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        definitions.families.entry(family).or_default();
    }
    for fonts in definitions.families.values_mut() {
        for (name, _) in FACES {
            if !fonts.iter().any(|font| font == name) {
                fonts.push(name.into());
            }
        }
    }
}

#[cfg(test)]
mod tests;
