# Inspector preview and navigation contract

## Swift behavior and implementation

Swift's DeclarationRegionView separates selection, disclosure and commit. A
single click or Up/Down selects; Right/Left or the chevron shows/hides source;
Return or double-click commits. Escape clears selection. Selection alone does
not expand content or move the source reader. Rust now follows these rules.
Multiple reference source disclosures can remain open independently. Focused
reference keys are consumed before the reader's global Return-to-definition
command; modal input remains exclusive. The definition region also owns its
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
publishing; loaded excerpts remain reusable. The reference viewport reuses reader-text's RowHeights to virtualize
variable-height rows and reveal keyboard selection. It introduces no runtime
sibling dependency. Source-preview rendering is shared by both inspector regions.

## Checks and acceptance

Six preview tests cover nested declaration ends, internal blank lines, truncation,
unparsed fallback, UTF-16 emoji/long-line cropping, stale/Before/cancel rejection,
request replacement, pinned Git bytes and same-file buffer precedence. Application
tests verify source/history preservation, same-token reuse, on-definition fetch
suppression, keyboard selection/disclosure/commit and modal exclusion. Pointer
replay verifies single-click selection and double-click commit.

Current gates: 233 application tests, one external Clang test ignored, 5 reader-text
tests and 30 doctests pass; fmt, Clippy with warnings denied, build and rustdoc pass.
Native focus, scrolling and visual acceptance are under final verification. The architecture
baseline is before-inspector-previews at 39cb614: no new product hotspot/cycle;
main inspector show CCN drops 17 to 9. The comparison still flags short test
setup/pointer sequences and a six-line source read branch; no duplicate product
query algorithm has been introduced. Sharing reference fixtures keeps clone groups
at 86 and reduces duplication from 0.72% to 0.70%.

The configured skill locations still lack swiftui-pro, macos-design and
typography-designer. Swift source, existing reader typography and the shared
source primitives guide the implementation.

## Still required for full inspector parity

Reference rows still need always-visible source-line text, grouping, definitions
and coverage accounting; currently their existing identity/path capsules remain.
Parsed/validated incoming call-tree disclosure, richer under-caret metadata and
external/local explanations, outline/bookmark keyboard parity, bounded revisit
caches, broader corpus/runtime checks and final Swift comparison remain open.
This package does not complete the full Rust/Swift objective.
