//! Optional CPU shaping and monochrome rasterization of the bundled emoji face.

use ab_glyph::{point, Font, FontRef, GlyphId, PxScale, ScaleFont};
use rustybuzz::{script, BufferFlags, Direction, Face, ShapePlan, UnicodeBuffer};
use unicode_segmentation::UnicodeSegmentation;

/// One glyph shaped from an original Unicode character cluster.
///
/// Positions use font units, with positive Y pointing upward. Source counts
/// describe the original input; they are never display substitutions or offsets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmojiGlyph {
    /// Glyph index in [`crate::EMOJI_FONT_BYTES`].
    pub glyph_id: u16,
    /// Original cluster length in UTF-8 bytes.
    pub source_bytes: usize,
    /// Original cluster length in Unicode scalars.
    pub source_scalars: usize,
    /// Horizontal and vertical advances in font units.
    pub advance: [i32; 2],
    /// Horizontal and vertical offsets in font units.
    pub offset: [i32; 2],
    /// Font units per em, for hosts using another rasterizer.
    pub units_per_em: u16,
}

/// Monochrome glyph coverage positioned relative to a text baseline.
#[derive(Clone, Debug, PartialEq)]
pub struct EmojiRaster {
    /// Pixel width of the coverage image.
    pub width: usize,
    /// Pixel height of the coverage image.
    pub height: usize,
    /// Top-left pixel relative to the baseline, with positive Y downward.
    pub origin: [i32; 2],
    /// Horizontal and vertical pixel advances, with positive Y downward.
    pub advance: [f32; 2],
    /// Pixel distance from the font's top to its baseline.
    pub ascent: f32,
    /// Row-major coverage, from 0 (transparent) to 255 (opaque).
    pub coverage: Vec<u8>,
}

/// Reusable shaping plan and buffer for the bundled monochrome emoji font.
///
/// This feature has no GUI, font discovery, native HarfBuzz or application
/// dependency. Hosts own sizing, caching, drawing and source-coordinate mapping.
/// Unsupported, multi-character or oversized inputs return `None` so a host can
/// keep its existing text fallback. Hosts also own paragraph and bidi layout.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "shaping")] {
/// let mut shaper = reader_fonts::EmojiShaper::new();
/// let glyph = shaper.shape_cluster("1\u{fe0f}\u{20e3}").unwrap();
/// assert_eq!(glyph.source_scalars, 3);
/// let image = shaper.rasterize(&glyph, 28.0).unwrap();
/// assert_eq!(image.coverage.len(), image.width * image.height);
/// # }
/// ```
pub struct EmojiShaper {
    face: Face<'static>,
    plan: ShapePlan,
    raster_font: FontRef<'static>,
    buffer: Option<UnicodeBuffer>,
}

impl Default for EmojiShaper {
    fn default() -> Self {
        Self::new()
    }
}

impl EmojiShaper {
    /// Creates a shaper over the unchanged bundled font bytes.
    ///
    /// # Panics
    ///
    /// Panics if the bundled font is invalid. Asset provenance and shaping tests
    /// verify this build-time invariant; no user font is parsed by this API.
    #[must_use]
    pub fn new() -> Self {
        let face =
            Face::from_slice(crate::EMOJI_FONT_BYTES, 0).expect("bundled emoji shaping face");
        let plan = ShapePlan::new(
            &face,
            Direction::LeftToRight,
            Some(script::COMMON),
            None,
            &[],
        );
        Self {
            face,
            plan,
            raster_font: FontRef::try_from_slice(crate::EMOJI_FONT_BYTES)
                .expect("bundled emoji raster face"),
            buffer: Some(UnicodeBuffer::new()),
        }
    }

    /// Shapes one grapheme into one supported font glyph with positive advance.
    ///
    /// Returns `None` for empty input, more than 256 UTF-8 bytes, multiple
    /// graphemes, missing glyphs, zero advance or a sequence the font cannot
    /// compose into a single glyph. Raw input bytes and scalar counts are retained.
    pub fn shape_cluster(&mut self, text: &str) -> Option<EmojiGlyph> {
        if text.is_empty() || text.len() > 256 || text.graphemes(true).count() != 1 {
            return None;
        }
        let mut buffer = self.buffer.take().unwrap_or_default();
        buffer.push_str(text);
        buffer.set_direction(Direction::LeftToRight);
        buffer.set_script(script::COMMON);
        buffer.set_flags(BufferFlags::REMOVE_DEFAULT_IGNORABLES);
        let shaped = rustybuzz::shape_with_plan(&self.face, &self.plan, buffer);
        let result = match (shaped.glyph_infos(), shaped.glyph_positions()) {
            ([info], [position])
                if info.glyph_id != 0 && info.cluster == 0 && position.x_advance > 0 =>
            {
                u16::try_from(info.glyph_id)
                    .ok()
                    .map(|glyph_id| EmojiGlyph {
                        glyph_id,
                        source_bytes: text.len(),
                        source_scalars: text.chars().count(),
                        advance: [position.x_advance, position.y_advance],
                        offset: [position.x_offset, position.y_offset],
                        units_per_em: self.face.units_per_em() as u16,
                    })
            }
            _ => None,
        };
        self.buffer = Some(shaped.clear());
        result
    }

    /// Rasterizes a shaped glyph at a host-selected physical pixel height.
    ///
    /// Height spans the font ascender to descender, as in `ab_glyph::PxScale`.
    /// Returns `None` for a nonfinite height, a height outside `(0, 512]`, an
    /// invalid glyph, an absent outline or coverage exceeding 1,048,576 pixels.
    /// Pixel output is independent of theme and GUI textures.
    #[must_use]
    pub fn rasterize(&self, glyph: &EmojiGlyph, height: f32) -> Option<EmojiRaster> {
        if !height.is_finite()
            || height <= 0.0
            || height > 512.0
            || glyph.glyph_id == 0
            || glyph.glyph_id >= self.face.number_of_glyphs()
        {
            return None;
        }
        let scale = height / self.raster_font.height_unscaled();
        let position = point(
            glyph.offset[0] as f32 * scale,
            -glyph.offset[1] as f32 * scale,
        );
        let outline = self.raster_font.outline_glyph(
            GlyphId(glyph.glyph_id).with_scale_and_position(PxScale::from(height), position),
        )?;
        let bounds = outline.px_bounds();
        let width = bounds.width() as usize;
        let pixel_height = bounds.height() as usize;
        let count = width
            .checked_mul(pixel_height)
            .filter(|count| *count <= 1_048_576)?;
        let mut coverage = vec![0; count];
        outline.draw(|x, y, alpha| {
            coverage[y as usize * width + x as usize] = (alpha * 255.0).round() as u8;
        });
        Some(EmojiRaster {
            width,
            height: pixel_height,
            origin: [bounds.min.x as i32, bounds.min.y as i32],
            advance: [
                glyph.advance[0] as f32 * scale,
                -glyph.advance[1] as f32 * scale,
            ],
            ascent: self.raster_font.as_scaled(height).ascent(),
            coverage,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shaped_clusters_match_independent_harfbuzz_glyphs_and_original_byte_counts() {
        let mut shaper = EmojiShaper::new();
        let mut count = 0;
        for line in include_str!("../tests/data/emoji-shaping.tsv")
            .lines()
            .filter(|line| !line.starts_with('#'))
        {
            let fields: Vec<_> = line.split_whitespace().collect();
            let text: String = fields[0]
                .split(',')
                .map(|value| char::from_u32(u32::from_str_radix(value, 16).unwrap()).unwrap())
                .collect();
            let values: Vec<i32> = fields[1..]
                .iter()
                .map(|value| value.parse().unwrap())
                .collect();
            let glyph = shaper
                .shape_cluster(&text)
                .unwrap_or_else(|| panic!("unsupported {text:?}"));
            assert_eq!(
                (i32::from(glyph.glyph_id), glyph.advance, glyph.offset),
                (values[0], [values[1], values[2]], [values[3], values[4]]),
                "{text:?}"
            );
            assert_eq!(
                (glyph.source_bytes, glyph.source_scalars, glyph.units_per_em),
                (text.len(), text.chars().count(), 2048)
            );
            assert_eq!(
                shaper.shape_cluster(&text),
                Some(glyph),
                "buffer reuse must be independent of earlier clusters"
            );
            count += 1;
        }
        assert_eq!(count, 24);
    }

    #[test]
    fn unsupported_or_oversized_input_retains_a_usable_shaper() {
        let mut shaper = EmojiShaper::default();
        let expected = shaper.shape_cluster("1\u{fe0f}\u{20e3}").unwrap();
        let oversized = format!("a{}", "\u{301}".repeat(256));
        for text in [
            "",
            "two",
            "👩👩",
            "e\u{301}",
            "漢",
            "\r\n",
            "\u{200d}",
            oversized.as_str(),
        ] {
            assert_eq!(shaper.shape_cluster(text), None, "{text:?}");
            assert_eq!(shaper.shape_cluster("1\u{fe0f}\u{20e3}"), Some(expected));
        }
    }

    #[test]
    fn composed_rasters_have_coverage_baseline_geometry_and_scale_without_a_gui() {
        let mut shaper = EmojiShaper::new();
        for text in ["1\u{fe0f}\u{20e3}", "👩‍👩‍👦", "👍🏽", "🇺🇸"] {
            let glyph = shaper.shape_cluster(text).unwrap();
            let a = shaper.rasterize(&glyph, 14.0).unwrap();
            let b = shaper.rasterize(&glyph, 28.0).unwrap();
            assert!(a.width > 0 && a.height > 0 && a.origin[1] < 0 && a.ascent > 0.0);
            assert_eq!(a.coverage.len(), a.width * a.height);
            assert!(a.coverage.iter().any(|alpha| *alpha != 0));
            let mass = |image: &EmojiRaster| {
                image
                    .coverage
                    .iter()
                    .map(|alpha| u64::from(*alpha))
                    .sum::<u64>()
            };
            assert!(
                mass(&b) > mass(&a) * 3,
                "coverage should scale with glyph area for {text:?}"
            );
            assert!(b.width > a.width && b.height > a.height);
            assert_eq!(b.advance[0], a.advance[0] * 2.0);
            assert_eq!(b.ascent, a.ascent * 2.0);
        }
    }

    #[test]
    fn raster_limits_reject_invalid_sizes_and_glyphs_without_large_allocations() {
        let mut shaper = EmojiShaper::new();
        let glyph = shaper.shape_cluster("1\u{fe0f}\u{20e3}").unwrap();
        for height in [0.0, -1.0, f32::NAN, f32::INFINITY, 513.0] {
            assert!(shaper.rasterize(&glyph, height).is_none());
        }
        assert!(shaper
            .rasterize(
                &EmojiGlyph {
                    glyph_id: 0,
                    ..glyph
                },
                14.0
            )
            .is_none());
        assert!(shaper
            .rasterize(
                &EmojiGlyph {
                    glyph_id: u16::MAX,
                    ..glyph
                },
                14.0
            )
            .is_none());
        assert!(shaper.rasterize(&glyph, 14.0).is_some());
    }
}
