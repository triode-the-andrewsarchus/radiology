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
helpers or large rendering crate. The explicit Fira Sans opt-in is now integrated, as recorded below; palette and
broader type-role/spacing integration remain part of the UI audit. Current source zoom
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


## Optional host-matching interface face — 2026-10-30

The same dependency-free asset package now exposes Fira Sans regular, copied with
its full original OFL notice from `clip-workbench/assets/fonts`. Asset hash:
`43c240a110cf888686d13cd17400fd0ca376ce55656150e52abff95f67a9532e`.
It adds 487,472 bytes (about 476 KiB) when embedded. `use_ui_font` is a separate,
explicit egui opt-in: it selects Fira Sans for proportional text while retaining
the original faces as fallbacks. Monospace/custom families and font sizes remain
host-owned. `append_fallbacks` alone still preserves primary fonts. Both operations
are idempotent and compose in either order. There is no added Rust dependency,
sibling checkout requirement or change to the sibling application.

Standalone Radiology opts in at window creation. Its compact reader scale retains
14 pt body/button, 20 pt heading and independently adjustable 14 pt default source;
Small metadata rises from egui's 10 pt default to 12 pt. Color and existing emphasis
distinguish supporting text. The sibling's 16 pt body/36 pt controls are workbench
metrics, not an instruction to enlarge every dense reader row. Code remains
monospace, with the existing measured geometry, source zoom and bytes. Headless
`from_session` does not configure or mutate a host context.

The requested typography/macOS/SwiftUI skills are still absent from configured
skill/plugin roots and the historical path in `design-system.md`. Before/after
review uses that checked-in specification, the original parity review's explicit
host-size guidance and the sibling's real font setup. This increment does not
complete broader type-role/spacing or palette alignment.

Validation:

- Four font tests and two doctests pass. New coverage proves UI layout actually
  changes, source glyph positions/sizes are identical at 14/28 pt and 1×/1.5×/2×,
  custom families survive, both registrations compose and repeated calls are safe.
  All three asset hashes are checked. Asset-only mode passes independently with
  no normal dependency, including no egui.
- `cargo test --workspace -- --test-threads=8`: 433 app tests (seven ignored), four
  font tests, eight shared-text tests and 32 doctests. Strict Clippy, build,
  formatting and warning-denied crate rustdoc pass. The rendered `use_ui_font`
  contract/example was visually inspected in Chromium.
- The initial default-thread suite timed out in the source-loader test
  `checkout_hunks_use_literal_paths_and_cover_untracked_source_on_first_open`.
  Its unchanged five-second helper deadline ran under heavy external build/test
  load. It passed on its own (1.93 s) and in the eight-thread workspace rerun.
  No timeout or production source-loading behavior was changed.
- Logs: `/tmp/radiology-ui-font-{tests,assets-only,workspace,workspace-recheck,clippy,build,doc,hunks-recheck}.log`.
  Rendered API capture: `/tmp/radiology-ui-font-rustdoc.png`.
- Architecture baseline `before-reader-ui-typography`: no new code clone, hotspot
  or cycle. The new clone group is the original OFL text shared with the Noto
  notices (91→92 groups, +0.08 percentage points); complete notices are retained.

Native evidence is in `/tmp/radiology-ui-font-native-{1.0,2.0}`, each with
`acceptance.json`, `SHA256SUMS`, captures, exact `emoji-copy.txt` and exit statuses.
At 1×, the 1280×900 workspace shows the UI face, source, inspector and local find
in both appearances. At 2×, a 1024×768 logical workspace (2048×1536 capture) keeps
toolbar/inspector/footer controls visible; find controls wrap to a second row,
including count and Close, without overlap. Both runs copy the exact original
emoji line and find the melting-face glyph at 1/1 in dark/light. Source fixture
bytes remain unchanged; app and both compositors close normally with exit zero.
The sibling capture helper is QA-only. These are representative native checks,
not broad accessibility, all-width or every-project acceptance.
