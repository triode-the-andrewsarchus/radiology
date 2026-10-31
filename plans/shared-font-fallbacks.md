# Portable reader font fallbacks

## Scope and reuse

`crates/reader-fonts` provides the same licensed CJK and monochrome emoji faces
already bundled by little-screen-clip. The font bytes and notices match the host
exactly; hashes are checked in a test and recorded in the package README. This
vendoring keeps Radiology independently buildable with no sibling path dependency.
The sibling checkout is read-only and still uses its original renderer/workbench
setup. This is a migration boundary, not a claim of completed cross-host adoption.

Default features export static asset bytes with no normal dependencies. Optional
`egui` registration appends those faces to existing families, including custom
host families; proportional/monospace families are created when missing. Primary
face order and type sizes remain host-owned, and registration is idempotent.
Radiology installs these definitions once in its existing window constructor.
Headless `from_session` still has no graphics side effects; embedding hosts choose
font/context setup explicitly. A screen renderer can use the asset-only package
without adopting egui, image/media or GPU dependencies.

This reuses the sibling's fallback policy without importing its unrelated style
helpers or large rendering crate. Its Fira Sans primary face, palette and type
role/spacing integration remain part of the broader UI audit. Current source zoom
and compact metadata hierarchy are preserved. The requested design/typography
skills remain unavailable locally; the checked-in design system and actual host
font setup were reviewed before and after this decision. No new type scale is
introduced by fallback registration.

## Correctness and limits

The egui test exercises both proportional and monospace layout at 1×, 1.5× and 2×.
Representative Japanese/Chinese/Korean text, kana and modern emoji all report
coverage and use glyph texture regions distinct from the missing-glyph fallback.
Latin galley sizes, glyph positions and glyph sizes exactly match plain defaults.
Another test checks custom-family preservation, repeated calls and empty initial
definitions. Asset-only mode independently passes its provenance test and has no
egui entry in its normal Cargo dependency tree.

Bundling increases executable assets by about 16.6 MiB. It does not cover every
script/ideograph, replace the host's primary font choice, add color emoji or provide
complex text/emoji composition. Layout tests alone do not establish native acceptance; the separate native run
below checks integrated rendering and exact source copying.
The existing measured source viewport continues to own geometry and byte mapping.

## Validation — 2026-10-30

Commands/logs:

- `cargo test -p reader-fonts --features egui` — three tests and one doctest.
- `cargo test -p reader-fonts --no-default-features` — one test, no GUI dependency.
- `cargo tree -p reader-fonts --no-default-features --edges normal` — the package only.
- Workspace tests, strict Clippy, formatting, build and warning-denied crate rustdoc.
- Logs: `/tmp/radiology-fonts-{tests,assets-only,workspace,clippy,build,doc}.log`.

Full workspace gates pass 432 app tests (seven ignored), three font tests, eight
shared text tests and 31 doctests. Warning-denied rustdoc renders the registration
contract/example correctly.

Architecture baseline `before-shared-font-fallbacks`: no new code hotspot/cycle or
code clone. The reported new 89-line clone is the OFL license text in the two
original third-party notices. Both complete notices are retained rather than
rewritten to improve metrics. Duplication rises by 0.08 percentage points solely
for these required license files (90 to 91 groups). Native glyph/selection acceptance is recorded below. Broader palette/type-role
and sibling integration remain open.


## Native acceptance — 2026-10-30

The standalone debug app ran in private Weston/Niri sessions at 1280×900 logical
size with output scales 1×, 1.5× and 2× (1280×900, 1920×1350 and 2560×1800 physical
captures). The fixture includes a Japanese/emoji filename and declaration, mixed
Japanese/Chinese/Korean/kana comments, and crab/melting-face/heart-hands/pink-heart/
shaking-face emoji. The source, filename, breadcrumb and outline render their
fallback glyphs; source glyphs remain aligned with selection and local find.
Monochrome emoji are expected. The sibling capture helper is a QA-only tool; the
app has no sibling runtime or build dependency.

Evidence is stored in `/tmp/radiology-fonts-native-{1.0,1.5,2.0}`. Each accepted
directory has `acceptance.json`, `SHA256SUMS`, clipboard bytes, captures and
`exit-statuses.json`. App, Weston and Niri exit zero at every scale. Fixture hashes
are identical before/after all accepted runs. The Japanese file SHA-256 is
`87b0afd62920a13a5e58503c005621474271a6d60f015c83934951df2c27b3c3`;
the Latin file is `960b7ddc8ead20ef925e0e6ceedba3fcc8a872cf3e7c6b354620455014dd487a`.

- 1×: CJK comment and emoji line select/copy exact original bytes, including
  indentation; local find highlights the melting-face glyph at 1/1 in dark and
  light appearance. See `cjk-comment-revealed.png`, `cjk-line-selected.png`,
  `emoji-local-find.png` and `unicode-reader-light-find.png`.
- 1.5×: source/filename/outline fallback rendering and exact emoji-line copy pass.
  See `unicode-source-scaled.png` and `emoji-selection-scaled.png`. Light appearance
  and CJK clipboard copying were not separately exercised at this scale.
- 2×: rapid Home/Shift-End/Copy preserves exact CJK and emoji line bytes after
  [the ordered-input fix](rust-source-input-order.md) in `daf295f`. Find reports
  1/1 with the matching glyph highlighted in both appearances. See
  `cjk-selection-scaled.png`, `emoji-selection-scaled.png`,
  `unicode-find-scaled.png` and `unicode-light-find-scaled.png`.

The first capture/copy experiment was invalid because Niri screenshots replace
clipboard text with PNG data. Accepted runs read clipboard bytes before capture.
Subsequent sequential 2× checks exposed a genuine coalesced-input bug, reproduced
by a regression test and fixed separately. Archived directories ending in
`-failed` or `-before-key-fix` are not accepted evidence. At 2×, the app closes
normally through Niri; interrupting the idle QA stdin loop triggers its cleanup
block and reaps owned children.

These checks establish representative fallback rendering, source byte fidelity
and the exercised selection/find paths. They do not complete every script,
wrapping/scrolling scenario, accessibility, host primary-font/type-role/palette
alignment, sibling adoption or the final Swift requirement audit. No additional
production change or automated rerun is part of this evidence-only increment.
