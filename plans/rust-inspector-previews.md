# Inspector preview and navigation contract

## Swift behavior and implementation

Swift's DeclarationRegionView separates selection, disclosure and commit. A
single click or Up/Down selects; Right/Left or the chevron shows/hides source;
Return or double-click commits. Escape clears selection. Selection alone does
not expand content or move the source reader. Rust now follows these rules.
Multiple reference source disclosures can remain open independently. Focused
reference keys are consumed before the reader's global Return-to-definition
command; modal input remains exclusive. Reader and inspector key dispatch share
one ordered event-consumption path, so fast mixed keys keep their original order.
Inspector focus remains eligible for global reader commands such as F12. The definition region also owns its
Right/Left disclosure and Return commit while focused.

`source::preview` owns toolkit-independent bounded excerpts and replaceable jobs
using LatestRead. Requests capture immutable source/index inputs, reuse displayed
bytes for the same file and SourceRead for cross-file checkout/pinned PR reads.
Full range conversion validates the declared encoding and generated source digest;
Before cannot consume After semantics. Old requests cannot publish after a new
source/index/target or repository/review transition. Same-token caret movement
and identical inputs do not restart work or collapse details. Equal-byte reloads
retain the same captured inputs; changed sources/indexes reset disclosure.

Definition excerpts end at the exact parsed declaration, including nested ones
and internal blank lines. Unparsed destinations stop at a blank line. Excerpts
have a 12-line cap and explicit continuation; reference excerpts cover up to two
lines either side. Lines are bounded to 2048 bytes, cropped near the indexed
occurrence on UTF-8 scalar boundaries and marked when cropped. Original line
numbers, CRLF handling and exact occurrence emphasis are preserved. Presentation
never becomes a navigation target; explicit commits reuse indexed navigation.

ReferenceContext owns selection and independently cancellable expanded-row
previews. Closing a row drops its job; replacing reference results clears row
state. Explicit-reference selection and loaded excerpts survive unchanged file
navigation and commits. A per-path/revision source identity ledger includes bytes
read by preview workers, so changed destinations invalidate excerpts on their
first reader visit. Pending requests rebind to newly displayed bytes before
publishing; loaded excerpts remain reusable. The reference viewport reuses
reader-text's RowHeights to virtualize
variable-height rows and reveal keyboard selection. It introduces no runtime
sibling dependency. Source-preview rendering is shared by both inspector regions.

## Checks and acceptance

Six preview tests cover nested declaration ends, internal blank lines, truncation,
unparsed fallback, UTF-16 emoji/long-line cropping, stale/Before/cancel rejection,
request replacement, pinned Git bytes and same-file buffer precedence. Application
tests verify source/history preservation, same-token reuse, on-definition fetch
suppression, keyboard selection/disclosure/commit and modal exclusion. Pointer
replay verifies single-click selection and double-click commit.

Current gates: 235 application tests, one external Clang test ignored, 5 reader-text
tests and 30 doctests pass; fmt, Clippy with warnings denied, build and rustdoc pass.
The architecture
baseline is before-inspector-previews at 39cb614: no new product hotspot/cycle;
main inspector show CCN drops 17 to 9. The comparison still flags short test
setup/pointer sequences and a six-line source read branch; no duplicate product
query algorithm has been introduced. Sharing reference fixtures keeps clone groups
at 86 and reduces duplication from 0.72% to 0.70%.

Native private-Wayland evidence is in `/tmp/radiology-inspector-previews-native/`.
`build-evidence.json` records the final tested product commit, binary SHA-256,
process identity, automated gates and successful application/compositor exits.
The final app and both compositors exit zero; helper pointer clients are
intentionally terminated on replacement/cleanup.

- `final-definition-preview.png` shows the real save declaration, stopping before
  its adjacent sibling, after Left/Right disclosure keys without source movement.
- `final-explicit-references.png` shows Find References working while a non-text
  inspector region has focus, with the independent definition excerpt still open.
- `final-reference-preview.png` shows single-click selection followed by Right
  disclosure, with real b.rs source and unchanged a.rs caret/definition context.
- `final-return-commit.png` reaches b.rs's exact call at line 5. Selection and
  the loaded reference excerpt remain intact through the commit.
- `final-back-restored.png` returns to a.rs's exact save occurrence after the
  emoji while retaining the explicit reference state.
- `multiple-reference-previews.png` and `reference-left-escape.png` verify
  independent simultaneous disclosures and keyboard collapse/selection clearing.

Fixture indexes are unverified imports; generated-source identity behavior and
pinned PR bytes are covered by the backend/application tests. The 1000-row test
verifies bounded painting and keyboard reveal; broad corpus, scaling and full
Swift runtime comparison remain part of the final acceptance queue.

The configured skill locations still lack swiftui-pro, macos-design and
typography-designer. Swift source, existing reader typography and the shared
source primitives guide the implementation.

## Still required for full inspector parity

Source-backed rows, grouping, definitions and coverage are now integrated;
see [reference results](rust-reference-results.md). Parsed/validated caller trees
are integrated with retained disclosures and native keyboard/navigation acceptance;
see [incoming callers](rust-incoming-callers.md). Background under-caret metadata,
local/package explanations and implementation links are integrated in
[metadata acceptance](rust-inspector-metadata.md). Outline/bookmark keyboard parity, bounded revisit
caches, broader corpus/runtime checks and final Swift comparison remain open.
This package does not complete the full Rust/Swift objective.

## Measured preview extents and bounded statuses — 2026-11-01

A new component regression confirms that caller reveal geometry and preview
rendering disagreed: Loading reserved 40 pt but consumed 14 pt. Loaded row heights
also depended on label/font geometry, while long failure text could consume
arbitrary width/height. Virtualized references measured painted rows afterward,
but caller offsets continued using the independently predicted preview extent.

The shared preview renderer now allocates exactly `height(phase)` and clips a
child to that rectangle. Loaded snippets use 18 pt source slots and the existing
12 pt monospace face for code and line numbers, including empty lines. The source
background fills the allocated excerpt; original text, emphasis and selectable
labels remain unchanged. Loading includes a spinner. Unavailable previews show a
compact Source unavailable heading and a fitted reason with its complete error
on hover. Caller controls therefore start at the same offset before and after
painting, independently of diagnostic wrapping. No public API, state, worker,
cache, dependency or source-coordinate transformation was added.

The regression fails on the old renderer and now passes every phase at 1×/1.5×/2×,
light/dark and 216/300 pt widths, including long multiline diagnostics, Unicode,
empty source rows, five-digit line numbers and truncation. Workspace passes
554 app tests (seven ignored), two binary, four font, 13 text and 36 doctests with
eight test threads. Strict Clippy/check/build/rustdoc/fmt/diff pass. Architecture
is NEUTRAL: +113 NLOC including the test, one new test function, renderer CCN 6;
no new hotspot/cycle/max-CCN increase and all 91 clone groups unchanged after full
normalized multiset audit. Health changes -0.01.

Four final native ready flows pass at 1×/2×. Failure runs remove only a private
reference destination after rows load, observe bounded source/caller failure,
restore the bytes and collapse/reopen into successful previews; accepted source
copy remains exact. Navigation runs expand the observed reference row, enter/
move/commit callers with Right/Down/Return in one invocation, copy the exact
473-byte destination line and restore the original call site with Back. Narrow
light/dark captures remain; final 2× failure/caller captures were visually
inspected. Each final run verifies binary/fixture hashes and zero child exits.

Earlier controllers used stale coordinates, left a status-covering hover tooltip,
or sent keys before pointer selection/focus completed; their trials are excluded.
Two earliest controllers required explicit cleanup of six verified private child
PIDs; disappearance was verified. Final controllers capture selected state before
keys and close gracefully. Only keyboard handoff is batched; same-frame pointer/
keyboard ownership remains a broader audit item, not accepted by these runs.
Swift-style snippet dedenting is also still absent and remains a concrete source
presentation follow-up. Requested design skills remain unavailable; this repair
reuses existing font roles and pane fitting.

Durable archive: `~/.local/state/radiology-qa/preview-status-20261101-780f87c`.
Only `preview-{failure,layout}-ready-*` flows are final native acceptance. Broader
mixed-language/review/input/large-file/semantic and final Swift/standalone/reuse
acceptance remain open; full accessibility and actual sibling adoption deferred.
