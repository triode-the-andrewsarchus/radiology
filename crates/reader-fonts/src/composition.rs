//! Egui display projection; original source bytes stay with the host.

use crate::{EmojiGlyph, EmojiShaper};
use egui::{text::LayoutJob, Color32, Context, FontFamily, FontId, Galley, Painter, TextureHandle};
use std::{
    collections::VecDeque,
    ops::Range,
    sync::{Arc, Mutex},
};
use unicode_segmentation::UnicodeSegmentation;

const FAMILY: &str = "reader-composed-emoji";
// This glyph has the emoji face's common advance. It is laid out transparently,
// unlike a space which would introduce a preferred wrap inside the cluster.
const SLOT: char = '◻';
const SHAPE_LIMIT: usize = 256;
const IMAGE_LIMIT: usize = 64;
const IMAGE_BYTES: usize = 4 * 1024 * 1024;

/// Registers a dedicated family for composed emoji display slots.
///
/// Existing primary and fallback families are unchanged. Call before applying
/// definitions to the context; repeated registration is safe. Used by
/// [`EmojiGalley::layout`] only when both `egui` and `shaping` are enabled.
pub fn register_composition(definitions: &mut egui::FontDefinitions) {
    definitions.font_data.insert(
        FAMILY.into(),
        egui::FontData::from_static(crate::EMOJI_FONT_BYTES),
    );
    definitions
        .families
        .insert(FontFamily::Name(FAMILY.into()), vec![FAMILY.into()]);
}

#[derive(Clone)]
struct Image {
    texture: TextureHandle,
    origin: [i32; 2],
    advance: f32,
}

struct Slot {
    chars: Range<usize>,
    color: Color32,
    image: Image,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct RasterKey {
    glyph: u16,
    offset: [i32; 2],
    advance: [i32; 2],
    height: u32,
}

/// Egui layout retaining original scalar indices while painting compound emoji.
///
/// The inner galley contains display substitutions, never canonical source.
/// Hosts keep original bytes for copying and map source scalar indices through
/// [`Self::caret_index`] and [`Self::visual_range`] for input and highlights.
/// ASCII, single-scalar glyphs and unsupported clusters retain ordinary layout.
/// Context-owned FIFO caches retain at most 256 shape results, 64 textures and
/// four MiB of texture pixels; returned layouts retain their own texture handles.
///
/// # Examples
///
/// ```
/// # #[cfg(all(feature = "egui", feature = "shaping"))] {
/// let ctx = egui::Context::default();
/// let mut fonts = egui::FontDefinitions::default();
/// reader_fonts::register_composition(&mut fonts);
/// ctx.set_fonts(fonts);
/// let _ = ctx.run(egui::RawInput::default(), |ctx| {
///     let job = egui::text::LayoutJob::simple(
///         "1️⃣".into(), egui::FontId::monospace(14.0),
///         egui::Color32::WHITE, f32::INFINITY,
///     );
///     let layout = reader_fonts::EmojiGalley::layout(ctx, job, &[]);
///     assert_eq!(layout.caret_index(1), 3);
///     assert_eq!(layout.visual_range(1..2), 0..3);
/// });
/// # }
/// ```
pub struct EmojiGalley {
    galley: Arc<Galley>,
    slots: Vec<Slot>,
    pixels_per_point: f32,
}

impl std::ops::Deref for EmojiGalley {
    type Target = Galley;
    fn deref(&self) -> &Galley {
        &self.galley
    }
}

impl EmojiGalley {
    /// Lays out host-formatted text, composing supported multi-scalar graphemes.
    ///
    /// `hard_breaks` are sorted UTF-8 byte boundaries in the job's original text,
    /// such as joins across hidden source. Composition never crosses these joins
    /// or a newline. A cluster spanning styles uses its first scalar's font size
    /// and color. Without [`register_composition`], ordinary layout is preserved.
    ///
    /// # Panics
    ///
    /// Panics if a hard break is out of order, out of bounds or splits a scalar.
    #[must_use]
    pub fn layout(ctx: &Context, job: LayoutJob, hard_breaks: &[usize]) -> Arc<Self> {
        let pixels_per_point = ctx.pixels_per_point();
        let mut slots = Vec::new();
        let job = if !job.text.is_ascii()
            && ctx.fonts(|fonts| fonts.families().contains(&FontFamily::Name(FAMILY.into())))
        {
            let cache = ctx.data_mut(|data| {
                data.get_temp_mut_or_default::<Arc<Mutex<Cache>>>(egui::Id::new(FAMILY))
                    .clone()
            });
            let mut cache = cache.lock().unwrap_or_else(|poison| poison.into_inner());
            project(ctx, job, hard_breaks, &mut cache, &mut slots)
        } else {
            job
        };
        let galley = ctx.fonts(|fonts| fonts.layout_job(job));
        Arc::new(Self {
            galley,
            slots,
            pixels_per_point,
        })
    }

    /// Returns a cursor index outside the interior of a composed cluster.
    ///
    /// Interior boundaries snap to the cluster end; the start remains unchanged.
    /// This maps a hit on the right half of the display slot past the whole glyph.
    #[must_use]
    pub fn caret_index(&self, index: usize) -> usize {
        self.slots
            .iter()
            .find(|slot| slot.chars.start < index && index < slot.chars.end)
            .map_or(index, |slot| slot.chars.end)
    }

    /// Expands a nonempty scalar highlight to include intersecting composed glyphs.
    ///
    /// Empty ranges remain empty. The host's canonical selection is not changed.
    #[must_use]
    pub fn visual_range(&self, mut range: Range<usize>) -> Range<usize> {
        if !range.is_empty() {
            for slot in &self.slots {
                if range.start < slot.chars.end && range.end > slot.chars.start {
                    range.start = range.start.min(slot.chars.start);
                    range.end = range.end.max(slot.chars.end);
                }
            }
        }
        range
    }

    /// Paints ordinary text and composed coverage at the galley's baseline.
    pub fn paint(&self, painter: &Painter, origin: egui::Pos2, fallback: Color32) {
        painter.galley(origin, self.galley.clone(), fallback);
        for slot in &self.slots {
            let cursor = self.galley.from_ccursor(egui::text::CCursor {
                index: slot.chars.start,
                prefer_next_row: true,
            });
            let Some(glyph) = self
                .rows
                .get(cursor.rcursor.row)
                .and_then(|row| row.glyphs.get(cursor.rcursor.column))
            else {
                continue;
            };
            let image = &slot.image;
            let offset =
                egui::vec2(image.origin[0] as f32, image.origin[1] as f32) / self.pixels_per_point;
            let rect = egui::Rect::from_min_size(
                origin + glyph.pos.to_vec2() + offset,
                image.texture.size_vec2() / self.pixels_per_point,
            );
            painter.image(
                image.texture.id(),
                rect,
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                slot.color,
            );
        }
    }
}

#[derive(Default)]
struct Cache {
    shaper: EmojiShaper,
    shapes: VecDeque<(String, Option<EmojiGlyph>)>,
    images: VecDeque<(RasterKey, Image)>,
    bytes: usize,
}

impl Cache {
    fn shape(&mut self, text: &str) -> Option<EmojiGlyph> {
        if text.len() > 256 {
            return None;
        }
        if let Some((_, glyph)) = self.shapes.iter().find(|(key, _)| key == text) {
            return *glyph;
        }
        // A surviving suffix after a hidden gap must not acquire a base from
        // before that gap (e.g. variation selector + enclosing keycap).
        let first = text.chars().next()?;
        let glyph = self
            .shaper
            .shape_cluster(&text[..first.len_utf8()])
            .and_then(|_| self.shaper.shape_cluster(text));
        if self.shapes.len() == SHAPE_LIMIT {
            self.shapes.pop_front();
        }
        self.shapes.push_back((text.into(), glyph));
        glyph
    }

    fn image(&mut self, ctx: &Context, glyph: &EmojiGlyph, height: f32) -> Option<Image> {
        let key = RasterKey {
            glyph: glyph.glyph_id,
            offset: glyph.offset,
            advance: glyph.advance,
            height: height.to_bits(),
        };
        if let Some((_, image)) = self.images.iter().find(|(cached, _)| *cached == key) {
            return Some(image.clone());
        }
        let raster = self.shaper.rasterize(glyph, height)?;
        let bytes = raster.coverage.len() * 4;
        if bytes > IMAGE_BYTES {
            return None;
        }
        while self.images.len() >= IMAGE_LIMIT || self.bytes + bytes > IMAGE_BYTES {
            let (_, old) = self.images.pop_front()?;
            self.bytes -= old.texture.size()[0] * old.texture.size()[1] * 4;
        }
        let pixels = raster
            .coverage
            .into_iter()
            .map(Color32::from_white_alpha)
            .collect();
        let texture = ctx.load_texture(
            FAMILY,
            egui::ColorImage {
                size: [raster.width, raster.height],
                pixels,
            },
            egui::TextureOptions::LINEAR,
        );
        let image = Image {
            texture,
            origin: raster.origin,
            advance: raster.advance[0],
        };
        self.bytes += bytes;
        self.images.push_back((key, image.clone()));
        Some(image)
    }
}

fn append_original(out: &mut LayoutJob, original: &LayoutJob, range: Range<usize>) {
    let first = original
        .sections
        .partition_point(|section| section.byte_range.end <= range.start);
    for section in &original.sections[first..] {
        if section.byte_range.start >= range.end {
            break;
        }
        let start = section.byte_range.start.max(range.start);
        let end = section.byte_range.end.min(range.end);
        if start < end {
            out.append(
                &original.text[start..end],
                if start == section.byte_range.start {
                    section.leading_space
                } else {
                    0.0
                },
                section.format.clone(),
            );
        }
    }
}

fn project(
    ctx: &Context,
    job: LayoutJob,
    breaks: &[usize],
    cache: &mut Cache,
    slots: &mut Vec<Slot>,
) -> LayoutJob {
    let mut out = job.clone();
    out.text.clear();
    out.sections.clear();
    let mut segment_start = 0;
    let mut copied = 0;
    let mut chars = 0;
    for end in breaks
        .iter()
        .copied()
        .chain(std::iter::once(job.text.len()))
    {
        assert!(
            end >= segment_start && job.text.is_char_boundary(end),
            "invalid composition boundary"
        );
        for (offset, text) in job.text[segment_start..end].grapheme_indices(true) {
            let start = segment_start + offset;
            let count = text.chars().count();
            let first = chars;
            chars += count;
            if count < 2 || text.contains(['\n', '\r']) {
                continue;
            }
            let section = job
                .sections
                .partition_point(|section| section.byte_range.end <= start);
            let Some(format) = job.sections.get(section).map(|section| &section.format) else {
                continue;
            };
            let Some(glyph) = cache.shape(text) else {
                continue;
            };
            let font = FontId::new(format.font_id.size, FontFamily::Name(FAMILY.into()));
            // Egui sizes fonts in ems; rasterize takes ascender-to-descender
            // height. This dedicated face has zero line gap, so row height is
            // that exact metric before Egui's physical-pixel rounding.
            let height =
                (ctx.fonts(|fonts| fonts.row_height(&font)) * ctx.pixels_per_point()).round();
            let Some(image) = cache.image(ctx, &glyph, height) else {
                continue;
            };
            let advance = ctx.fonts(|fonts| fonts.glyph_width(&font, SLOT));
            if glyph.advance[1] != 0
                || (advance - image.advance / ctx.pixels_per_point()).abs() > 0.01
            {
                continue;
            }
            append_original(&mut out, &job, copied..start);
            let mut slot_format = format.clone();
            slot_format.font_id = font;
            slot_format.color = Color32::TRANSPARENT;
            let display = format!("{SLOT}{}", "\u{200d}".repeat(count - 1));
            out.append(&display, 0.0, slot_format);
            // Keep even a single wide cluster on one row with all its indices.
            out.wrap.max_width = out.wrap.max_width.max(advance);
            slots.push(Slot {
                chars: first..chars,
                color: format.color,
                image,
            });
            copied = start + text.len();
        }
        segment_start = end;
    }
    append_original(&mut out, &job, copied..job.text.len());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(scale: f32, registered: bool) -> Context {
        let ctx = Context::default();
        let mut definitions = egui::FontDefinitions::default();
        crate::append_fallbacks(&mut definitions);
        if registered {
            register_composition(&mut definitions);
        }
        ctx.set_fonts(definitions);
        ctx.set_pixels_per_point(scale);
        ctx.begin_frame(egui::RawInput::default());
        ctx
    }

    fn job(text: &str, width: f32) -> LayoutJob {
        LayoutJob::simple(text.into(), FontId::monospace(14.0), Color32::WHITE, width)
    }

    #[test]
    fn composed_slots_retain_scalar_indices_and_whole_glyph_hits_at_each_scale() {
        for scale in [1.0, 1.5, 2.0] {
            let ctx = context(scale, true);
            let text = "a1️⃣👨‍👩‍👧‍👦👍🏽🇺🇸z";
            let galley = EmojiGalley::layout(&ctx, job(text, f32::INFINITY), &[]);
            assert_eq!(galley.slots.len(), 4);
            assert_eq!(galley.job.text.chars().count(), text.chars().count());
            for slot in &galley.slots {
                let left = galley.pos_from_ccursor(egui::text::CCursor::new(slot.chars.start));
                let right = galley.pos_from_ccursor(egui::text::CCursor::new(slot.chars.end));
                let width = right.left() - left.left();
                assert!(
                    (width - slot.image.advance.round() / scale).abs() < 0.01,
                    "scale={scale} {:?}: width={width}, advance={}",
                    slot.chars,
                    slot.image.advance / scale
                );
                for (fraction, expected) in [(0.2, slot.chars.start), (0.8, slot.chars.end)] {
                    let hit = galley.cursor_from_pos(egui::vec2(
                        left.left() + width * fraction,
                        left.center().y,
                    ));
                    assert_eq!(galley.caret_index(hit.ccursor.index), expected);
                }
                assert_eq!(
                    galley.visual_range(slot.chars.start + 1..slot.chars.end),
                    slot.chars
                );
                assert_eq!(
                    galley.visual_range(slot.chars.start..slot.chars.start),
                    slot.chars.start..slot.chars.start
                );
            }
            let _ = ctx.end_frame();
        }
    }

    #[test]
    fn wrapping_keeps_all_indices_with_the_composed_glyph_even_in_a_narrow_view() {
        let ctx = context(1.0, true);
        for width in [1.0, 16.0, 30.0, 90.0] {
            let text = "xx 1️⃣ab 👨‍👩‍👧‍👦 cd 👍🏽 🇺🇸";
            let galley = EmojiGalley::layout(&ctx, job(text, width), &[]);
            assert_eq!(galley.slots.len(), 4);
            let mut start = 0;
            for row in &galley.rows {
                let end = start + row.char_count_including_newline();
                for slot in &galley.slots {
                    assert!(
                        !(slot.chars.start < end && end < slot.chars.end),
                        "wrap inside {:?} at {width}: {start}..{end}",
                        slot.chars
                    );
                }
                start = end;
            }
            assert_eq!(start, text.chars().count());
        }
        let _ = ctx.end_frame();
    }

    #[test]
    fn original_styles_and_hard_source_joins_have_distinct_composition_rules() {
        let ctx = context(2.0, true);
        let mut input = job("1", f32::INFINITY);
        input.sections[0].format.color = Color32::RED;
        input.append(
            "\u{fe0f}\u{20e3}z",
            0.0,
            egui::TextFormat::simple(FontId::monospace(14.0), Color32::BLUE),
        );
        let joined = EmojiGalley::layout(&ctx, input.clone(), &[]);
        assert_eq!(joined.slots.len(), 1);
        assert_eq!(joined.slots[0].color, Color32::RED);
        assert_eq!(
            joined.job.sections.last().unwrap().format.color,
            Color32::BLUE
        );
        let broken = EmojiGalley::layout(&ctx, input.clone(), &[1]);
        assert!(broken.slots.is_empty());
        assert_eq!(broken.job.text, input.text);
        let _ = ctx.end_frame();
    }

    #[test]
    fn plain_geometry_and_unregistered_hosts_keep_existing_layout() {
        for registered in [false, true] {
            let ctx = context(1.0, registered);
            for text in [
                "ASCII 123 #*",
                "café e\u{301} 漢字",
                "🦀🫠",
                if registered { "plain" } else { "1️⃣" },
            ] {
                let input = job(text, 90.0);
                let original = ctx.fonts(|fonts| fonts.layout_job(input.clone()));
                let result = EmojiGalley::layout(&ctx, input, &[]);
                assert!(result.slots.is_empty());
                assert_eq!(result.job.text, text);
                assert_eq!(original.size(), result.size());
                for index in 0..=text.chars().count() {
                    let cursor = egui::text::CCursor::new(index);
                    assert_eq!(
                        original.pos_from_ccursor(cursor),
                        result.pos_from_ccursor(cursor)
                    );
                }
            }
            let _ = ctx.end_frame();
        }
    }

    #[test]
    fn raster_cache_reuses_shapes_but_separates_scale_and_bounds_retained_memory() {
        let ctx = context(1.0, true);
        let mut cache = Cache::default();
        let glyph = cache.shape("1️⃣").unwrap();
        let first = cache.image(&ctx, &glyph, 14.0).unwrap();
        assert_eq!(
            first.texture.id(),
            cache.image(&ctx, &glyph, 14.0).unwrap().texture.id()
        );
        assert_ne!(
            first.texture.id(),
            cache.image(&ctx, &glyph, 28.0).unwrap().texture.id()
        );
        for height in 1..=512 {
            cache.image(&ctx, &glyph, height as f32).unwrap();
            assert!(cache.images.len() <= IMAGE_LIMIT && cache.bytes <= IMAGE_BYTES);
        }
        for index in 0..SHAPE_LIMIT * 2 {
            cache.shape(&format!("unsupported{index}"));
        }
        assert_eq!(cache.shapes.len(), SHAPE_LIMIT);
        assert_eq!(cache.shape("1️⃣"), Some(glyph));
        let _ = ctx.end_frame();
    }
}
