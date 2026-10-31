# Unwrapped bounded-fragment geometry

## Confirmed defect — 2026-10-31

With Wrap Lines off, the viewport still assigns a vertical RowHeights entry to
every 4 KiB projected fragment. A long canonical line therefore appears on several
visual rows and Down advances to another fragment of that same source line.
This contradicts the Swift unwrapped reader behavior. The existing short-line
horizontal scrolling tests do not establish this case.

A real source-viewport regression fails on 9fc02dc: the first and last fragments
of one unwrapped line have y offsets 0 and 60.890625. Its next assertion requires
Down from the line start to reach line 2. Native 1280×900 evidence shows the same
continuations and a Line 1 footer after Down. Original CRLF fixture bytes remain
unchanged; app, Weston and Niri exit zero.

The original failing regression is retained as ui-regression.rs in the before
archive. It is now restored as a passing application test. The initial backend
increment at 47766a5 did not repair the UI; the viewport integration below does
repair the stacked-fragment and Down behavior. Remaining interaction acceptance
is listed explicitly below.

## Reusable measurement model

reader-text::UnwrappedRows groups fragments by first_in_row, retains estimated and
measured widths, computes row-relative offsets and tracks the first unmeasured
fragment of each row. Hosts can prioritize the exact prefix needed for a caret or
hit target and defer unresolved geometry while keeping layout work bounded.
Out-of-order measurements close gaps without touching unrelated rows. Coordinates
use f64 accumulation; renderers choose their own coordinate conversion.

The model owns no text, font or UI object. It neither changes canonical source
bytes nor shapes an unbounded line. All measurement values come from the host.
Hosts rebuild it when source projection or font/scale geometry changes. This is
available to the standalone reader and future little-screen-clip adoption through
the existing framework-free reader-text package, without adding dependencies.

Tests cover grouping, zero widths, exact boundary lookup, out-of-order readiness,
idempotent measurements, unrelated-row isolation, repeated updates against linear
geometry, Unicode bounded projections and a fold joining a hidden newline. Its
runnable rustdoc demonstrates prefix readiness. Workspace gates pass 453 app
tests (seven ignored), two binary checker tests, four font tests, eleven text
tests and 34 doctests. Strict Clippy, fmt/diff and warning-denied reader-text
rustdoc pass; the generated UnwrappedRows page is inspected. An initial Clippy
single-range fixture lint is corrected with a borrowed range slice. Architecture
comparison is neutral with no new clone group, hotspot or dependency cycle.

## Viewport integration — 2026-10-31

The viewport now uses UnwrappedRows for bounded-fragment placement, measured
content width and caret translation. All fragments in a projected line share the
same y offset; only the last owns the vertical RowHeights weight. Grouping follows
first_in_line, including folds that join surviving canonical lines. Wrapped
layout retains its existing visual rows and does not allocate the horizontal
model. Gutter, fold and test-label controls remain owned by the first fragment.

FragmentFrame carries one bounded galley and shared paint/hit geometry. Horizontal
clipping skips offscreen continuation layouts; adjacent fragments have disjoint
hit intervals so later fragments cannot overwrite a pointer selection. Hover,
find/pin/selection painting and caret use the translated origin. Current-line and
destination bands share the whole projected row while text remains bounded.
Repeated unchanged width/height records avoid scanning their suffix/Fenwick tree.

Background measurement retains the existing 64-fragment / 2 ms frame budget.
A priority frontier measures the prefix needed by navigation or reveal before
unrelated document rows. Up/Down and PageUp/PageDown move between projected rows,
using shaped horizontal positions rather than scalar-column guesses. The existing
source event classifier feeds a queue: movement awaiting geometry holds later
selection/copy events in order. Source/projection/font changes, focus loss, manual
pointer/scroll input and new host navigation retire stale queued input. Pending
reveal survives measurement and recalculates after viewport/font/scale changes;
manual input supersedes it. Left crosses a continuation boundary by one scalar
and stays at byte zero at the document start.

The original failing y/Down regression now passes. Additional tests cover:

- Deferred End/Shift-Down/Copy on a line exceeding two frame budgets, preserving
  original Unicode/CRLF bytes and ordered selection.
- Deferred-input retirement on new host navigation, document, zoom and focus.
- Horizontal Unicode hits and exact Copy at 1×/1.5×/2×, continuation-boundary Left,
  wrap/zoom transitions and logical PageUp/PageDown.
- The existing folded Unicode/hidden-newline hit/arrow/copy case in both modes.

Workspace gates pass 457 app tests (seven ignored), two binary checker tests,
four font tests, eleven text tests and 34 doctests. Strict Clippy, build, fmt and
diff checks pass. Architecture comparison against before-unwrapped-viewport is
neutral: no new clone, hotspot, dependency cycle or maximum-CCN increase. An early
implementation increased render/prepare complexity and duplicated font setup;
shared FragmentFrame geometry, reveal handling and test setup remove those flags.

## Native evidence and remaining acceptance

Before evidence is retained under
`~/.local/state/radiology-qa/unwrapped-before-20261031-9fc02dc`.
Current evidence is under
`~/.local/state/radiology-qa/unwrapped-viewport-20261031-47766a5`, with SHA256SUMS,
fixture/binary/source hashes, logs and process exits. Final 1×/2× checks exercise
Down to line 2, End reveal and exact current-line/long-line/Copy All CRLF payloads.
The earlier 2× trial additionally records light/dark rendering, wrap toggling and
an exact `_END` suffix copied after a far-right pointer hit. Modal-opening batches
that had not yet accepted the typed line are excluded from navigation acceptance;
the final runs separate modal entry and submission. Reused screenshot paths in
the trial can return old size metadata before the new image is written; final
runs use unique capture paths. The PNGs themselves are retained and inspected.
Fixtures remain unchanged and app/compositors exit zero.

Required follow-up, before calling the whole source interaction audit complete:

1. Retained pointer gestures are now implemented and verified in
   [ordered source gestures](rust-source-gesture-order.md). The original external
   regression is restored as a passing viewport test. Primary/Shift selection,
   drag/Copy, Ctrl-click and delayed menu keys use exact shaped geometry and
   ownership retirement. See that document for current native evidence and the
   remaining long-name/focus findings.
2. Repair toolbar menu traversal/focus restoration. In the 2× trial, bare source
   keys stop responding after toolbar use, even after a pointer caret change.
   Go To Line restores keyboard ownership; light-end-via-modal.png confirms End
   thereafter. This is distinct from the repaired reader context menus.
3. Complete broader large-file/long-name/wrap/scroll/scaling acceptance and exact
   cross-file Back restoration on long unwrapped lines. Existing short/ordinary
   navigation tests do not prove every long-line case.

Full Swift requirement acceptance, broader grammar/index/PR/cache corpus and the
final standalone/reuse boundary audit remain active. Full accessibility and actual
sibling integration remain deferred.
