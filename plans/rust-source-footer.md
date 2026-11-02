# Measured source footer — 2026-11-02

## Behavior and architecture

Native toolbar QA exposed source-footer overlap at the 800-point window minimum
with both side panes open: Rust/line count/No Index collided and the line control
was hidden. Swift ReaderDetail presents the corresponding language/count/test/
structure/semantic/location/revision/encoding/read-only information.

A private footer module now renders that existing metadata in a wrapping bottom
panel. The panel owns its content height; the viewport fills the remaining area
instead of reserving an unrelated fixed 28 points. Metadata, diagnostic hover
messages, revision and semantic distinctions and Go to Line remain available.
Existing typography, status policy, source coordinates, modal actions and panel
mechanisms are reused. Footer action publication survives breadcrumb rendering.
No new state model, dependency, worker, host API or sibling changes.

The viewport scrollbar/autoscroll regression now derives the outside pointer from
its actual canvas edge, replacing the old hard-coded y=345. This preserves the
behavior being tested after the viewport starts filling its available area.

The four picker failures recorded below describe the footer checkpoint. They are
now resolved through [shared modal input](rust-picker-preframe-focus.md), restoring
a full unfiltered green workspace suite.

## Evidence and limits

- RED: the corrected actual-app regression cannot find Line 2 with the old footer
  at 800 points. Initial fixture assumptions about gutter bounds and navigation
  byte offsets were corrected before interpreting the result.
- The green matrix covers 1×/2×, normal/degraded metadata and dark/light, repeatedly
  resizing 800→1100→1400→800 in one Context. It checks visible glyph-row bounds,
  pairwise non-overlap, metadata/control presence, stable viewport height on return,
  pointer Go to Line, cancellation and exact source/caret retention. Diagnostics
  include structure, folding, occurrence and highlight failures. Theme recoloring
  is settled before injecting diagnostics, avoiding a changing loading-banner height.
  Wrapped egui labels have leading/empty rows; assertions use drawable row meshes.
- Final native v5 old/v6 rebuilt-new private Weston/Niri runs at 800×700,
  1×/2× all pass.
  Old captures show the collision and absent line control. New runs verify all
  footer metadata, click Line 1, submit 250, freshly copy the exact canonical Unicode/
  CRLF line, show Line 250 and retain readable metadata after switching to light.
  Fixture/Git/binary hashes are unchanged; final app/compositor exits are zero.
  Captures were inspected. Initial v1–v4 attempts retain OCR I/L ambiguity, float
  pointer-coordinate rejection and an oversized light OCR crop; these are harness
  failures, not evidence of a footer product defect. Final crops/OCR substitutions
  are recorded in the scripts, with original captures preserved.
- Full testing exposed four **inherited** picker failures, reproduced unchanged on
  b5b8385 in a clean detached worktree. See [focus investigation](rust-picker-preframe-focus.md).
  The suite is not fully green: the final run explicitly excludes those four and
  passes 626 app tests, seven ignored/four filtered; two binary, 13 font and 19 text
  tests also pass. All 40 doctests pass with TMPDIR under home cache. Initial fully
  concurrent testing also hit one source-query timeout; the bounded four-thread
  unaffected suite passes. Strict Clippy/check/build/rustdoc/fmt/diff and copied
  standalone resource verification pass. The speculative picker change is absent.

## Architecture review

Baseline before-source-footer-layout at b5b8385: 87,125 NLOC, 3,668 functions,
average CCN 3.34, max 132 and 109 clone groups. Final overall NEUTRAL: +118 NLOC/
two functions including the regression; one private UI file, max/average unchanged,
109 clone groups, no new production hotspot/cycle, health -0.01. SourceView::show
CCN 9→2 (164→88 NLOC), while the measured footer owns CCN 8/63 NLOC. Separation is
functional: the footer owns area allocation and the canvas fills the remainder.
The tool flags the new matrix test as high complexity; its status/scale/resize and
geometry checks are necessary test cases, with rectangle containment simplifying
bounds validation. No new production abstraction is added to move a metric.

Evidence archive: ~/.local/state/radiology-qa/source-footer-20261102-b5b8385.
The baseline worktree ~/.cache/radiology-footer-baseline remains available for the
picker investigation. Picker ownership, broader UI/input/corpus and final Swift
acceptance remain open. Accessibility and sibling adoption stay deferred.
