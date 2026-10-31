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
complex text/emoji composition. No screenshot or source-selection/scaling acceptance
is inferred from the layout tests; integrated native verification remains open.
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
for these required license files (90 to 91 groups). Native glyph/selection/font
acceptance and broader palette/type-role/sibling integration remain open.
