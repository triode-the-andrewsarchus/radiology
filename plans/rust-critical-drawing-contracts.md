# Critical projection drawing contracts

Reference: all three `SourceProjectionDrawingTests` groups, compared with Rust
`9122073` plus this checkpoint. Runtime already satisfies the drawing contracts;
this change strengthens actual viewport-frame coverage without product changes.

## Platform mechanism

Swift applies temporary TextKit attributes at draw time. Rust builds projected
layout sections from immutable syntax-color spans, paints selection/find/pin
layers independently and copies canonical source bytes. There is no incoming
AppKit attribute dictionary or TextKit effective-range callback to port literally.
The equivalent observable contracts are exact bounded color changes, independent
decoration/selection state, unchanged source and copy, immediate clearing and
source-owned projection application on every rendered frame.

## Assertion mapping

| Swift group | Rust evidence |
| --- | --- |
| `recededRangesDimTheStorageColorAndKeepIncomingAttributes` | Expanded `critical_projection_splits_colored_spans_without_changing_source_geometry_or_bytes` runs at both scales in dark/light themes. Three syntax-color spans surround a call after an emoji. Actual source galleys preserve original position/size and the prominent call's color; receded sections retain their own syntax color with the established 0.38 dark/0.30 light opacity. Selection and pinned occurrence bands remain painted, and their model state, original highlighted spans and source bytes stay identical. Actual Copy returns the selected canonical text. A replacement projection strictly inside one color run changes only those bytes, restores previously receded prefix bytes, and leaves each boundary's outside byte unchanged. None clears; Normal ignores the projection. The constructor's existing canonical byte normalization replaces Swift's UTF-16 sorting, covered independently by the model/projection audit. |
| `rangesClearOnSourceReplacementAndSurviveAFoldAndRedraw` | New `critical_projection_survives_fold_redraw_and_copy_keeps_hidden_original_bytes` applies a clone-only projection in a CRLF function, selects all, then folds/unfolds twice at each scale. Folded frames paint no clone text, preserve projection/selection/caret and copy the complete original source including hidden bytes and CRLF. Unfolded frames again paint dimmed syntax. The expanded color test changes same-path source bytes and proves stale projection is ignored, then restores identical source and reapplies the exact same projection. Existing app/source transaction guards own accepted-source clearing; these tests exercise the renderer's independent exact-source check. |
| `drawingConsultsTheHookForEveryPass` | Three successive actual egui frames each paint the projected colors and both selection/pin layers while Copy succeeds. Subsequent frames replace the ranges, clear, switch mode, reject changed content and reapply the original matching projection. Each result is checked from that frame's painted TextShape galley, rather than a hook-call counter or a cached model range. This proves observable redraw behavior; Rust does not add a TextKit-style instrumentation hook. |

Rust retains its existing draw-time opacity and independent feedback/pin idioms;
this does not redesign source type, weight, layout or theme. The source tests use
real egui input/paint output, not a new compositor screenshot or Swift/AppKit run.
The [model/projection audit](rust-critical-model-projection-contracts.md) separately
proves exact UTF-8 range boundaries, Unicode semantic encodings, unindexed syntactic
call projection and source/revision identity. Existing hover/navigation checks
retain their own input/feedback evidence.

## Verification and architecture

Archive: `~/.local/state/radiology-qa/critical-drawing-20261109-9122073/`.
All 65 viewport tests and ten Critical Lens app/popup checks pass. The final
two focused projection tests pass after explicit single-range collection syntax
satisfies strict all-target workspace Clippy. Formatting and diff checks pass. No fresh full-workspace build/docs/test
run is claimed for this test-only change; the latest full run in `091cd94` passed
854 app tests, 11 existing ignored, and all 65 doctests.

Baseline `before-critical-drawing-20261109` to `verified-critical-drawing-20261109`:
+119 measured NLOC/one test function, unchanged measured files, average/max CCN
3.41/132, 36 hotspots and cycles. Full normalized clone multisets retain all 123
instances with zero additions/removals. The comparator reports the old touched
viewport assertion clone. Existing Reader, text-shape, band, selection, pin,
fold and copy mechanisms are reused; no production abstraction or dependency
was added.

With the policy/storage, frontier, model/projection, summary/hover and worker
checkpoints, all nine Critical Lens/hover suites (29 annotated Swift groups) now
have explicit mappings. This does not establish overall app parity: Code Paths,
semantic/context/navigation, source/syntax, compiler/indexing and inspector suites
retain remaining reconciliation, followed by final whole-app/native and standalone
acceptance maintenance. Known parallel timing limits and Android remain open.
Full accessibility and actual sibling adoption remain deferred.
