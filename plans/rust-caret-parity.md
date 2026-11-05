# Insertion caret reconciliation — 2026-11-05

Reviewing the three Swift `SourceCaretTests` found that Rust painted its caret
under a range selection, in an unfocused reader and in an empty read-only document.
The new visibility regression fails against the previous implementation. A further
geometry regression exposed interior composed-character bytes painting at the
trailing edge rather than the cluster's start.

`SourceViewport::paint_caret` now checks the existing focus owner, native-window
focus, host keyboard permission, selection and menu ownership before painting.
Opening a menu suppresses the caret in that first frame too. The source byte,
selection, current-line wash and function wash remain independent of this paint
decision. Empty documents have no insertion caret; a nonempty document's final
empty line still does.

Caret/reveal geometry uses `ProjectedFragment::adjacent_grapheme` to find the
containing visible grapheme. Interior bytes paint at its start while exact EOF
stays after the whole cluster. Each projected span keeps its existing hidden-gap
boundary. This reuses bounded fragment segmentation; no whole-file scan, cache,
dependency or public API was added. Pointer hits still use the existing trailing
edge mapping; canonical navigation, selection and Copy retain their original bytes.

## Swift assertions and Rust evidence

All three new tests live in `src/ui/source_view/viewport.rs` and inspect actual
egui line-segment paint output at 1× and 2×, rather than repeating the predicate.

| Swift contract | Rust regression |
| --- | --- |
| Typeset caret across tabs, emoji, line ends and EOF | `insertion_caret_geometry_follows_tabs_graphemes_line_ends_and_eof` checks ordered advances, tab width, row origins, final newline and unterminated emoji/family/combining clusters. Appending a glyph preserves EOF placement within half a physical pixel, allowing separate-galley advance rounding. Interior scalar positions paint at the cluster start without altering the canonical caret. |
| Caret hidden in folded text | `insertion_caret_skips_folded_bytes_and_empty_documents` checks a hidden body, surviving opener/closer, following declaration and empty document, preserving source. |
| Empty selection and focused/key reader only | `insertion_caret_requires_empty_selection_reader_ownership_and_active_window` checks unfocused reader, range and collapsed selections, inactive window, host sheet, another editor, menu opening/closing and focus return. |

Swift TextKit accepts UTF-16 positions; Rust's boundary remains canonical UTF-8
with scalar columns. Existing source-coordinate and compound-glyph tests cover
conversion/hit/Copy, and continue passing alongside the new paint assertions.

Native private-compositor runs at both scales capture a focused caret, range
selection, collapsed selection, Go to Line, Find and refocus. Selection Copy and
full-source Copy match exact expected bytes. Attached files, Git HEAD/refs/status
and executable hashes stay unchanged; all child processes exit zero. This is
Linux egui acceptance; AppKit is not executed on this host.

## Checks and architecture

The locked workspace passes 782 app tests (11 opt-in ignored), two binary tests,
13 font tests, 22 text tests and 60 doctests. Strict Clippy/rustdoc, formatting,
build/resource smoke and the separate full-reader consumer pass. That consumer
uses the current repository crate and checks focus, viewport/caret separation,
Unicode Copy and stale-source rejection at both scales. The prior isolated export
and source-unavailable resource acceptance remains documented separately.

Against `before-reader-caret-focus`, reported NLOC increases by 186. Duplication
remains 125 instances/1335 lines; full clone-fragment/file-pair counters have zero
additions or removals. The diff-aware duplicate warning names two unchanged old
word-navigation tests whose line numbers shifted. No new hotspot, dependency cycle
or structural issue appears; maximum CCN stays 132. Caret painting is a cohesive
helper (CCN10), and shared geometry remains small. Analyzer function inventories
omit some Rust tests around character literals, so their reported net function
count is not a complete source inventory; the diff adds one production helper,
three tests and one shared paint-output assertion helper.
Average measured CCN increases from 3.39 to 3.40 for the required visibility and
geometry branches. The other raw warning swaps equally scored chokepoints in
untouched Python scripts; it does not identify a new dependency or source change.

Evidence is retained at
`~/.local/state/radiology-qa/caret-focus-20261105-83782a7/`; working data lives in
`~/.cache/radiology-qa-caret-focus/`. The broader Swift suite reconciliation and
SDK-backed Android gate remain open. Full accessibility and sibling app adoption
remain deferred. No font scale, typography, platform window behavior or host API
changed in this pass.
