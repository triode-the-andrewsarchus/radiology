# Source emoji composition

## Confirmed gap

Native source captures in the [Unicode matching audit](rust-symbol-unicode.md)
show a keycap as a digit plus a separate enclosing glyph. The bundled Noto Emoji
face contains U+20E3 and proper GSUB compositions; font coverage is not the cause.
Egui 0.27 lays out scalar glyphs individually. The sibling renderer likewise uses
per-character font fallback and disables substitution loading. Existing renderer
code therefore does not provide the required composition to reuse directly.

## Shaping foundation — 2026-11-02

The existing `reader-fonts` crate now has an optional CPU-only `shaping` feature.
`EmojiShaper` retains the parsed bundled face, one Common/LTR OpenType plan and a
reusable buffer. It uses [Rustybuzz's planned shaping](https://docs.rs/rustybuzz/0.20.1/rustybuzz/fn.shape_with_plan.html),
removing default-ignorable placeholder output glyphs while retaining original
source byte/scalar counts. Recognized clusters yield glyph IDs, font-unit metrics
and offsets; unsupported or oversized clusters return None for host fallback.

`rasterize` returns bounded 8-bit coverage, baseline-relative origin, scaled
advances and ascent. It reuses ab_glyph, already in the lockfile. No font bytes are
changed, no new native font library is required and no GUI/context state is owned
by the shaper. Hosts control sizing, caching, color, drawing and source mapping.
Default asset-only hosts still have no normal dependencies; egui registration and
optional Fira Sans selection keep their existing policies.

New optional lockfile packages: rustybuzz 0.20.1, core_maths 0.1.1, libm 0.2.16,
unicode-bidi-mirroring 0.4.0, unicode-ccc 0.4.0, unicode-properties 0.1.5 and
unicode-script 0.5.9. Existing package versions stay unchanged. Normal Radiology
builds still enable only the existing egui feature, so this foundation does not
change the app's source rendering.

## Foundation verification

An independent native HarfBuzz 15.0.0 oracle over the exact font asset supplies
24 glyph IDs/advances/offsets: digits and keycaps with/without variation selectors,
families, skin tones, flags, hearts and ZWJ sequences. Rust output matches all of
them; original UTF-8/scalar counts and buffer reuse are checked. Initial native
output retained zero-advance placeholder glyphs; the explicit remove-ignorables
flag is now shared by the reference and Rust path. Raster tests check nonempty
coverage, area scaling, baseline geometry, invalid sizes/glyphs and recovery.
Tiny antialiased glyphs need not contain an opaque pixel; coverage mass rather than
an opaque-pixel assumption now verifies their scaling.

Feature-enabled workspace: 601 app tests passed, seven ignored; two binary,
eight font, 13 text and 38 doctests passed. CPU-only shaping: five tests and one
doctest; asset-only: one test, zero doctests. Strict feature-enabled Clippy/check/
rustdoc, normal app build, formatting and diff checks pass. Generated rustdoc
contains the API and limits. Two isolated Cargo hosts build/run offline with 16
jobs: asset mode has only reader-fonts in its dependency tree; shaping mode has no
Radiology/egui dependency. The shaping executable links no native HarfBuzz.

The standalone example renders keycap/family/tone/flag coverage at 28/56 pixel
heights. Composed images are inspected, including a digit inside its keycap.
These are CPU raster artifacts, not a claim of native app rendering acceptance.
The checked-in generator is used explicitly for QA and never during Cargo builds.

## Required integration and acceptance

Enable shaping only as the source renderer consumes it. Retain raw source and the
existing SourceFragment byte/character contract. Validate a layout projection that
reserves one composed glyph's advance while retaining all original scalar cursor
indices; the bundled emoji space has the font's common advance and may provide a
blank measured slot. This design still needs a prototype before committing to it.
Do not inject private-use aliases into source or silently normalize source bytes.

Use real shaped coverage with host-owned, bounded caches and device-scale keys.
Cache shaping by original cluster text; keep per-occurrence source counts separate
from raster caching. Raster keys must account for glyph ID, offsets and pixel size.
Visual match/selection washes may need whole-glyph bounds even when a canonical
source range starts inside the cluster; copying must retain the exact source range.
Account for baseline/line-height geometry, wraps, styles, fragment boundaries and
fold-joined ranges. Preserve ASCII/CJK geometry and unsupported-cluster fallback.
Verify click/drag, grapheme caret/selection, copy, find/occurrence washes, Critical
Lens, zoom/wrap/folds and source focus at native 1×/2×. Keyboard and pointer behavior
remain required; full accessibility is deferred. Actual sibling adoption is also
deferred; its fontdue renderer can reuse the IDs/metrics or coverage over the
unchanged shared asset when requested.

Source composition and broader application parity remain open. No integration or
native source-shaping improvement is claimed for this foundation.

Architecture against `before-reader-font-shaping`: NEUTRAL, +359 NLOC/11 functions/
three files including QA/example code, health +0.11, no new hotspot/cycle; all 109
clone groups unchanged by normalized audit. The normal copied app also passes its
standalone resource verifier. Evidence:
`~/.local/state/radiology-qa/reader-shaping-20261102-9ad5278`.
