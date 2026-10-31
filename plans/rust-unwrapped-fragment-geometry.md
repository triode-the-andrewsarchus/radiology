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

The failing regression is retained as ui-regression.rs in the durable evidence
folder below. It has not been committed as a failing or ignored application test.
The UI defect remains open; the backend increment in this document is preparation
for its repair.

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

## Required viewport integration

1. Keep source fragments bounded. Use one vertical row for their whole projected
   line when wrapping is disabled; keep wrapped-row behavior separate.
2. Apply the measured horizontal prefix to painting, caret reveal, hover and hit
   testing. Only the first fragment owns the gutter/fold/test controls. Shape only
   relevant bounded fragments and schedule remaining measurements with a budget.
3. Route Up/Down between projected logical rows in unwrapped mode, preserving the
   shaped horizontal position. Preserve event/copy order if geometry needs more
   frames; do not substitute a scalar-column guess for a shaped position.
4. Retain a pending destination until its prefix is measured, respect manual
   scrolling, and invalidate measurements/queued input on document or geometry
   changes. Keep byte selections, folds, find and history in canonical coordinates.
5. Restore the recorded regression and verify long Unicode/CRLF lines across
   fragment boundaries, horizontal scrolling, wrap/zoom transitions, hit testing,
   copy and exact cross-file Back restoration. Run native 1×/2× checks, including
   the broader large-file/long-name acceptance cases.

Evidence, regression source, fixture/binary hashes and captures are retained under
`~/.local/state/radiology-qa/unwrapped-before-20261031-9fc02dc` with SHA256SUMS.
This increment does not claim the application uses UnwrappedRows yet. Full parity,
the remaining keyboard/reader/index/PR/cache audit and the final Swift checklist
remain active. Accessibility and actual sibling integration stay deferred.
