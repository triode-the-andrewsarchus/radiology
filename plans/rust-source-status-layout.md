# Compact source status metadata — 2026-11-05

## Behavior

Native Before captures at the smaller1024×768 logical window showed “Read Only”
splitting inside its own label onto a second row. Swift's ReaderDetail uses its
metadata type style for the footer. Rust mixed body-size labels with an explicitly
11.5-point Line button and default8-point spacing around every field/separator.

The existing footer now uses the host's Small text style consistently, including
the Line button, and4-point metadata spacing. The native app defines Small at12
points; no new type size, scaling system, control or abstraction is introduced.
All metadata and diagnostic details remain visible. Existing content-sized wrapping
continues at genuinely narrow widths and for diagnostic rows; it is not replaced
with clipping or hidden warnings. Pin controls retain their separate row/style.
The change is scoped to the metadata child UI and preserves the clickable Line
control and source font preferences.

The requested SwiftUI/macOS/typography skills were searched for but remain absent
from the installed skill catalog/files. The checked-in Swift footer and existing
Rust metadata type scale provide the design reference for this bounded Linux fix.

## Verification

Temporary execution is in `~/.cache/radiology-qa-source-status`; durable evidence:
`~/.local/state/radiology-qa/source-status-20261105-27cc3ab`.

The existing footer interaction regression now covers checkout, Before, After and
diagnostics,1×/2×, widths800/1024/1100/1400 and return to800. It uses the actual app
UI face and12-point Small style. It checks every metadata label, mesh bounds,
nonoverlap, stable height on return, unchanged bytes/caret and pointer opening of
Go to Line. Routine metadata at widths≥1024 must occupy a single row and each label
must itself have one shaped row. The old code fails specifically because Before's
Read Only label has two rows. Earlier harness attempts that checked only label
origins missed that internal wrap; their logs remain as diagnostics.

Native1×/2× captures use real Git, controlled PR metadata and raw keyboard/clipboard
on private Wayland. After and Before navigation/Back still pass, and dark/light
Before captures visibly retain the complete footer on one row at the previously
failing width. Five exact Copy checks per scale preserve original bytes. Attached
file/HEAD/ref/status and executable hashes remain unchanged; app/compositors exit
zero. These native runs cover normal metadata, while the app regression covers
failure diagnostics and narrower/resized panes.

Full locked workspace774 app tests/11 opt-in ignored, two binary,13 font,22 text
and60 doctests pass. The final geometry-helper refactor passes the focused matrix;
strict all-target workspace Clippy, locked build, embedded resource smoke and
format/diff checks pass.

Architecture baseline `before-source-status-layout`: +56 NLOC/one test assertion
helper, no new production function/dependency/API. The test matrix previously
mixed all geometry and interaction assertions; the geometry now has a cohesive
helper(CCN8) and interaction matrix(CCN11, previously12). Existing36 hotspots,
average/max3.39/132, one cycle and eight structural findings remain unchanged.
The full clone Counter remains125 instances/1335 duplicate lines with no additions
or removals. Raw diff-aware warnings refer to an unchanged old focus-test clone
and rank noise in untouched Swift Python.

## Remaining goal

The observed footer issue is closed. Continue the remaining Swift behavior
reconciliation and SDK-backed Android success acceptance, followed by the final
clean standalone/resource/library-host audit. Actual sibling adoption and full
assistive accessibility remain deferred by the user.
