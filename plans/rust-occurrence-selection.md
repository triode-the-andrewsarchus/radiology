# Transient occurrence selection and pin styling

Swift selects a drawing-only occurrence group on an ordinary source click with
no text selection. The clicked range stays active even when the caret later moves.
Clicking a miss clears the group; double-click pin/unpin clears it. Document
changes do not restore transient state. Rust now follows these event rules.

## Ownership and reuse

`OccurrenceSelection` carries the resolved identity, original label, clicked
range and canonical occurrence ranges. The existing pin resolver now returns this
value for either transient selection or pinning, preserving the parsed callable/
exclusion and complete-versus-partial SCIP policy. It creates no semantic target,
reference focus, bookmark or history entry. `OccurrencePinStore.ephemeral` owns
only this transient presentation; exact-document memory still stores pins alone.

The source canvas emits SelectOccurrences on a plain completed click, not on every
caret event. egui 0.27 counts double-clicks by time alone: native acceptance caught
a gutter click followed by an accidental source pin. A local source click sequence
now requires consecutive source clicks within six points and 300 ms, ignores
triple-click toggles, and resets across other controls, selection/drag and source
changes. It reuses egui's tolerances while adding source ownership/position. Drag selection and modified clicks do not select occurrence groups.
The existing pointer/caret flow continues to update inspector context and history
independently. Pin/unpin clears the transient group after resolution, so the first
click of a double-click cannot leave a stray wash. A later ordinary click may
highlight a pinned group too, as in Swift. Failed source reads preserve the
current display and decoration; successful document changes clear the transient.

## Rendering

Selection/find/occurrence rendering shares one shaped source-range rectangle
visitor, with no heap buffer for rectangles and no hidden-body layout. Canonical
byte spans still pass through the fold/test projection. Pins paint before the
transient group; text and caret paint last. Nonactive occurrences use 16% accent;
the clicked occurrence uses 28% accent with a 55% accent, one-point outline.
Occurrence rectangles expand 1.5 points horizontally and use 2.5-point corners.

Locked pins now carry the heavier 1.4-point outline in their palette color, with
Swift's dark/light fill opacity (22%/15%) and stroke opacity (88%/72%). Palette
base colors are opaque; opacity is applied once at the paint boundary. Pin legend
fills follow the same source-surface opacity. Group colors and lifetime rules do
not change. Folded-away ranges contribute no rectangles or strokes.

## Verification

Full gates pass 312 app tests (one external-Clang fixture ignored), eight shared
reader-text tests and 30 doctests; fmt, strict Clippy, build and rustdoc pass.
Tests cover click versus drag dispatch, exact active/group ranges, shared partial/
complete SCIP resolution, independent caret refresh, misses, pin/unpin, successful
and failed file changes, no transient restoration, both theme opacities/outlines,
and hidden-range rejection without mutating source selection. Spatially distinct rapid
clicks select separate groups, a nearby second click pins, triple-clicks do not
toggle again, and drag selection retires the prior click sequence.

Architecture against `before-transient-occurrences`: +282 NLOC, eight functions,
no new files, unchanged complexity/clone groups, no new product hotspot or cycle.
An initial render/paint hotspot was removed by isolating occurrence click policy
and the ordered occurrence-decoration pass. The tool's REGRESSION label refers
to six baseline test fragments in touched app/source-view files; those fragments
are already present in the baseline, and clone groups remain 91.

Broader scaling/accessibility/native corpus acceptance, PR refresh mapping,
source/graph/context caches, background/index/PR lifecycle and sibling palette/
chrome integration remain open. Full Rust/Swift parity remains unproven.


Native evidence is `/tmp/radiology-transient-native/`: final-prefixed single-click
dark/light, arrow-preserves-group, click-miss-clears, pinned-dark/light and
unpinned-light captures, plus `build-evidence.json` hashing each capture. The
partial SCIP fixture highlights both code spellings without adding a pin on the
first click. ArrowDown moves the caret from line 2 to line 3 while the first
occurrence retains its active outline. A string click clears the transient group.
Two source clicks pin outlined teal ranges; unpinning from the cfg-inactive
occurrence leaves neither a legend nor a transient wash. A later single click
creates the light-theme transient group again. All captures with `before-fix-`
names use the earlier binary and document the global-double-click defect only.

Final native binary SHA-256 is `d09088fbf1b3f4c1ba6c0e5e7db1cb6cc4efe223aae896d2f530052d62304cfc`,
based on `026e837` plus this implementation. Final app PID 45229 and private
Weston/Niri exited zero, as did the initial app. Persistent pointer helpers were
intentionally terminated. The acceptance driver imports the sibling compositor
helper only for testing; Radiology remains independently buildable and runnable.
