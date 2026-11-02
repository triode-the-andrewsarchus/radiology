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
unicode-script 0.5.9. Existing package versions stay unchanged. At the foundation commit, Radiology
builds enabled only the existing egui feature. The source integration below now
enables both features; the foundation alone did not change app rendering.

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

### Grapheme-preserving fragment boundaries

Both `reader-text` fragment builders now share a bounded prefix calculation.
Graphemes that fit within the full fragment limit move intact to the next fragment
when the remaining room is insufficient. Oversized clusters retain scalar-safe
splitting; lookahead is bounded by twice the fragment budget plus four bytes,
including for pathological combining sequences. ASCII remains a fast path.
Hidden gaps are hard boundaries: unrelated surviving spans cannot synthesize a
cluster. Canonical byte envelopes, scalar cursor indices, line ownership and copy
bytes retain their existing contracts. Unicode segmentation reuses the already
locked 1.13.3 dependency; no dependency versions change.

The boundary matrix covers accents, keycaps, family ZWJ, tone modifiers and flags,
six limits through the app's 4,096 bytes, five prefix positions, CRLF, folded
suffixes with partial remaining room, scalar-coordinate round trips and 100,000
combining marks. The app harness checks wrapped/unwrapped fragmentation at the
actual 4,096-byte boundary and exact Select All/Copy. These checks establish the
fragment contract; they do not establish composed glyph painting or native UI
acceptance.

Workspace verification: 602 app tests passed (seven ignored), two binary, eight
font, 15 text and 38 doctests. Strict Clippy/check/rustdoc, normal app build,
formatting/diff and copied standalone resource verification pass. Architecture
against `before-grapheme-fragments`: NEUTRAL, +118 NLOC/three functions, no new
hotspot/cycle/clone group, 109 groups retained, health -0.01. Evidence:
`~/.local/state/radiology-qa/grapheme-fragments-20261102-7e239ac`.

## Source renderer integration — 2026-11-02

Radiology now enables shaping and explicitly registers a dedicated composition
family during app initialization. The optional `reader-fonts::EmojiGalley` bridge
has no Radiology dependency. Asset-only and CPU-only consumers retain their
existing feature boundaries. The shared reader-text fragment supplies adjacent
whole-grapheme cursor movement, retaining the existing ownership of boundaries
following hidden gaps.

The bridge projects supported multi-scalar graphemes after the complete styled
layout job is assembled. One transparent square in the unchanged emoji face
reserves the exact shaped advance; zero-width joiners retain the remaining scalar
indices. A square avoids the preferred break introduced by a space; the font has
no NBSP mapping. Neither these display substitutions nor normalization enter the
source buffer. Unregistered contexts, ASCII, single scalars and unsupported or
oversized clusters keep ordinary layout. A suffix beginning with an unsupported
base cannot compose across a hard source join. Styles spanning a cluster use the
first scalar's size/color, including syntax and Critical Lens receding.

Egui uses em sizes while ab_glyph rasterization uses ascender-to-descender height;
the dedicated face's zero-gap row height supplies the correct physical scale,
rounded as egui rounds it. Painting uses its actual glyph baseline and coverage,
tinted with the original style. Layout width admits at least one composed glyph
so wrapping cannot separate its zero-width scalar indices. Ordinary source font
registration and ASCII/CJK caret geometry remain unchanged.

A context-owned cache retains at most 256 shape results, 64 images and four MiB
of image pixels, with FIFO eviction. Raster keys include glyph, offsets, advances
and physical height; theme and original source counts are not raster identities.
Each returned layout retains the handles it needs; the cache owns no Context,
worker, daemon or native HarfBuzz dependency. Hosts own the original text and
canonical selection. Hit testing and vertical/page movement snap composed
interiors to the glyph end. Find/selection/occurrence rectangles expand only their
visual scalar range; copying remains the exact canonical range, even if it starts
inside a compound. Left/right movement uses complete graphemes per visible span.

### Verification

- Five font-bridge matrices cover scalar indices, pointer halves, highlights,
  wrapping through narrow widths, first-style ownership, hard joins, unchanged
  ordinary geometry, cache reuse/scaling/eviction and bounded memory. Shared text
  tests cover compound movement and hidden-boundary ownership. The source harness
  checks pointer/keyboard selection, partial canonical copy, whole-glyph highlight
  widths and separately textured glyphs at 1×/1.5×/2× with wrap on/off.
- Workspace: 603 app tests passed, seven ignored; two binary, 13 font, 16 text and
  38 then-existing doctests passed. The added public bridge example passes as the
  39th doctest. Strict Clippy/check/rustdoc, build/fmt/diff pass; rendered API docs
  were inspected. CPU-only: five tests/one doc; asset-only: one test. A separate
  offline host imports only reader-fonts/egui and exercises the public bridge,
  with 16 build jobs. A copied standalone app verifies all resources.
- Native v2 old-control at 1× reproduces separate glyphs and scalar-only selection,
  with ten exact copies. Final v4 1×/2× each pass 18 fresh clipboard checks: whole
  keycap/family/tone/flag/accent, complete lines, left/right pointer halves, drag,
  expanded-source CRLF copy, the 4,096-byte edge, zoom/wrap and find/Escape focus.
  Captures include dark/light, colored string glyphs, folding, zoom/wrap, find and
  source-mode transitions; inspected at both scales. Original fixture/binary
  hashes remain unchanged and all final process exits are zero.
- Early native v1 ended on stdin EOF and is not accepted. Native v2 after-runs
  exposed the pre-existing unsupported Ctrl+Home command, which the script had
  incorrectly assumed for line-start movement. V3 uses the supported Home and
  passes; v4 adds pointer/drag/fold/edge coverage. These failed attempts are kept
  separately from accepted evidence. The [document-edge increment](rust-keyboard-audit.md#source-document-edge-navigation--2026-11-02)
  subsequently fixes Ctrl+Home/End; other modified source bindings remain in the
  broader keyboard audit.

Architecture against `before-source-emoji-paint`: NEUTRAL, +565 NLOC/20 functions/
one module including tests, no new hotspots/cycles/clone groups, all 109 groups
retained, health -0.01. No new dependency versions or lockfile changes.
Archive: `~/.local/state/radiology-qa/source-composition-20261102-1444bf2`.

## Remaining scope

The confirmed source compound-emoji defect is integrated and accepted. General
Unicode shaping/bidi, compound emoji in other UI labels/previews, source
word/paragraph navigation and the broader Swift parity/corpus review remain open. The shaper is
intentionally bounded to supported single-glyph clusters; unsupported text keeps
the existing scalar fallback. Full accessibility and actual sibling adoption
remain deferred. The existing broad requirement ledger stays authoritative.

## Foundation architecture evidence

Architecture against `before-reader-font-shaping`: NEUTRAL, +359 NLOC/11 functions/
three files including QA/example code, health +0.11, no new hotspot/cycle; all 109
clone groups unchanged by normalized audit. The normal copied app also passes its
standalone resource verifier. Evidence:
`~/.local/state/radiology-qa/reader-shaping-20261102-9ad5278`.
