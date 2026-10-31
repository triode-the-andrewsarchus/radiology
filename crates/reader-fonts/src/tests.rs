use super::*;
use sha2::{Digest, Sha256};

#[test]
fn bundled_assets_match_the_existing_host_font_provenance() {
    for (bytes, expected) in [
        (
            UI_FONT_BYTES,
            "43c240a110cf888686d13cd17400fd0ca376ce55656150e52abff95f67a9532e",
        ),
        (
            CJK_FONT_BYTES,
            "86f7ba3101617c899dd2ea44c51f0d58e58892709949c8619f5800ada8d9b601",
        ),
        (
            EMOJI_FONT_BYTES,
            "8f685aeeef29fd7ab230e1b9aafda3bbc0e3c35c19553038e95d12e468338f07",
        ),
    ] {
        assert_eq!(format!("{:x}", Sha256::digest(bytes)), expected);
    }
}

#[cfg(feature = "egui")]
#[test]
fn registration_preserves_custom_primary_fonts_and_is_idempotent() {
    use egui::{FontDefinitions, FontFamily};
    let mut definitions = FontDefinitions::default();
    let custom = FontFamily::Name("host-code".into());
    definitions.families.insert(
        custom.clone(),
        definitions.families[&FontFamily::Monospace].clone(),
    );
    let before = definitions.families.clone();
    append_fallbacks(&mut definitions);
    let registered = definitions.families.clone();
    let count = definitions.font_data.len();
    append_fallbacks(&mut definitions);
    assert_eq!(definitions.families, registered);
    assert_eq!(definitions.font_data.len(), count);
    for (family, original) in before {
        assert!(definitions.families[&family].starts_with(&original));
        assert_eq!(definitions.families[&family].len(), original.len() + 2);
    }
    let mut empty = FontDefinitions {
        font_data: Default::default(),
        families: Default::default(),
    };
    append_fallbacks(&mut empty);
    assert!(empty.families.contains_key(&FontFamily::Proportional));
    assert!(empty.families.contains_key(&FontFamily::Monospace));
}

#[cfg(feature = "egui")]
#[test]
fn cjk_and_modern_emoji_draw_without_missing_glyphs_or_latin_geometry_changes() {
    use egui::{Color32, Context, FontDefinitions, FontId};
    for scale in [1.0, 1.5, 2.0] {
        let plain = Context::default();
        let reader = Context::default();
        let mut definitions = FontDefinitions::default();
        append_fallbacks(&mut definitions);
        reader.set_fonts(definitions);
        for context in [&plain, &reader] {
            context.set_pixels_per_point(scale);
            let _ = context.run(Default::default(), |_| {});
        }
        for font in [FontId::proportional(14.0), FontId::monospace(22.0)] {
            let text = "source { value = 42; }";
            let original = plain
                .fonts(|fonts| fonts.layout_no_wrap(text.into(), font.clone(), Color32::WHITE));
            reader.fonts(|fonts| {
                let actual = fonts.layout_no_wrap(text.into(), font.clone(), Color32::WHITE);
                assert_eq!(actual.size(), original.size());
                for (a, b) in actual.rows[0].glyphs.iter().zip(&original.rows[0].glyphs) {
                    assert_eq!((a.pos, a.size), (b.pos, b.size));
                }
                let unicode = "日本語 简体中文 繁體中文 한국어 ひらがな カタカナ 🦀 🫠 🫶 🩷 🫨";
                assert!(fonts.has_glyphs(&font, unicode));
                let missing =
                    fonts.layout_no_wrap("\u{10ffff}".into(), font.clone(), Color32::WHITE);
                let galley = fonts.layout_no_wrap(unicode.into(), font.clone(), Color32::WHITE);
                assert_eq!(galley.rows[0].text(), unicode);
                for glyph in &galley.rows[0].glyphs {
                    if !glyph.chr.is_whitespace() {
                        assert_ne!(
                            glyph.uv_rect, missing.rows[0].glyphs[0].uv_rect,
                            "{} at scale {scale}",
                            glyph.chr
                        );
                    }
                }
            });
        }
    }
}

#[cfg(feature = "egui")]
#[test]
fn optional_ui_face_changes_interface_text_but_preserves_source_and_host_families() {
    use egui::{Color32, Context, FontDefinitions, FontFamily, FontId};
    let mut definitions = FontDefinitions::default();
    let custom = FontFamily::Name("host-preview".into());
    definitions.families.insert(
        custom.clone(),
        definitions.families[&FontFamily::Monospace].clone(),
    );
    append_fallbacks(&mut definitions);
    let original = definitions.families.clone();
    let before = Context::default();
    before.set_fonts(definitions.clone());
    use_ui_font(&mut definitions);
    let registered = definitions.families.clone();
    let count = definitions.font_data.len();
    use_ui_font(&mut definitions);
    append_fallbacks(&mut definitions);
    assert_eq!(definitions.families, registered);
    assert_eq!(definitions.font_data.len(), count);
    assert_eq!(definitions.families[&custom], original[&custom]);
    assert_eq!(
        definitions.families[&FontFamily::Monospace],
        original[&FontFamily::Monospace]
    );
    assert!(definitions.families[&FontFamily::Proportional]
        .ends_with(&original[&FontFamily::Proportional]));
    let after = Context::default();
    after.set_fonts(definitions);
    for scale in [1.0, 1.5, 2.0] {
        for context in [&before, &after] {
            context.set_pixels_per_point(scale);
            let _ = context.run(Default::default(), |_| {});
        }
        for font in [FontId::monospace(14.0), FontId::monospace(28.0)] {
            let source = "fn 日本語() { let emoji = \"🦀 🫠\"; }";
            let layout = |context: &Context| {
                context.fonts(|fonts| {
                    fonts.layout_no_wrap(source.into(), font.clone(), Color32::WHITE)
                })
            };
            let (a, b) = (layout(&before), layout(&after));
            assert_eq!(a.size(), b.size());
            for (a, b) in a.rows[0].glyphs.iter().zip(&b.rows[0].glyphs) {
                assert_eq!((a.chr, a.pos, a.size), (b.chr, b.pos, b.size));
            }
        }
        let label = "Repository · Declaration";
        let width = |context: &Context| {
            context.fonts(|fonts| {
                fonts
                    .layout_no_wrap(label.into(), FontId::proportional(14.0), Color32::WHITE)
                    .size()
                    .x
            })
        };
        assert_ne!(
            width(&before),
            width(&after),
            "UI face must actually change at {scale}×"
        );
        after.fonts(|fonts| assert!(fonts.has_glyphs(&FontId::proportional(12.0), "日本語 🫠")));
    }
}
