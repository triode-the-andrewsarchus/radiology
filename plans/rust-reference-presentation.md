# Reference and caller presentation and region bounds

Swift's `ReferenceFileHeader`, `ReferenceCallerPill`, `ReferenceCallerRow` and
`ReferenceRow` define this increment's presentation contract. Caller names use
middle truncation, line numbers retain their own slot, filenames remain visible,
and boundary notes appear separately. File headers prioritize path suffixes.

## Presentation and interaction changes

Caller rows now fit name, filename, line/depth metadata and any boundary note into
separate slots. Notes add 20 points to the existing 44-point row. Height estimates,
visible-slice painting and keyboard reveal share those variable extents. A layout
walk paints only intersecting caller rows; it introduces no new cache or model.
Long labels use the shared UTF-8-safe 4 KiB fitter. Reference file headers use its
new head-elision mode, reusing the measured search rather than clipping a bare
path suffix without an omission mark. Full models/destinations remain intact.

Reference row IDs include exact path, canonical occurrence bytes and source digest.
Chevron IDs derive from their owning row; the existing icon renderer now accepts
explicit IDs while preserving ordinary icon-button keyboard allocation. Caller
IDs include their exact source location/revision and accepted digest; Trace More
also binds its tree's source identities. Changing a target retires retained presses.
The current-rectangle activation check moved from Sidebar to the shared command
module and serves both result panes and reference/caller controls. Menu ownership
uses the same reference row ID. List actions still return focus to the list rail;
caller controls and global F12 remain supported.

A native 2× resize exposed a separate defect: the reference region could consume
the middle region, whose Definition content then painted over Bookmarks. Bookmarks
now reserves its actual extent first; the reference maximum leaves 120 points for
Definition/Outline. Each of the three regions renders through a clipped child UI
whose content cannot enlarge its parent layout. This also bounds restored oversized
panel state and content in very small regions, without replacing the shared lists.

## Verification (2026-10-31)

Four new regressions cover caller slots/notes/line/depth at 216/300/420-point widths,
both themes/scales, reference and caller press retirement on target/source changes,
moved reference/disclosure rectangles, and Tab/Space activation of explicit IDs.
The 1,000-caller visible-slice test now mixes noted/unnoted row heights and reveals
the final caller/depth. The existing three-region test now also restores oversized
panel extents, retaining a usable middle region in every theme/scale/width case.
Common painted-text geometry inspection and fixture setup are reused.

Saved negatives using previous production function bodies fail for three distinct
reasons: a replacement reference accepts an old press; a long caller loses its
name suffix; oversized regions leave no required space between Outline and
Bookmarks. The normal icon-button regression caught during implementation was
repaired; declaration Tab/Space tests pass again.

Workspace passes 493 app tests (seven ignored), two binary, four font and eleven
text tests, plus 34 doctests. Final focused tests, strict Clippy, workspace/all-target
check, build, fmt and diff checks pass. Architecture retains 89 clone groups and
24 high-complexity functions, with no added hotspot/cycle/max-CCN increase; average
CCN drops 3.37 → 3.36. Its raw diff-aware report flags five existing test clusters.
The complete whitespace-normalized clone multiset has zero additions/removals;
those findings do not represent new duplicate mechanisms.

Private native 1×/1280×900 and 2×/1024×768 runs load a two-document UTF-8 SCIP
fixture, pin the exact target, expand its reference and callers, enter caller focus,
then Down/Return navigates to the entry call at line 5. Explicit source selection
copies all 473 bytes, including the long Unicode identifier. Both fresh clipboard
offers match SHA-256
`0e93fc179c83050e1f5b372c8a6715de8af7f2b2bacc441b89498a0c7c87e01b`.
Back restores source line 2. Narrowed dark/light panes retain caller metadata and
separate Definition/Bookmarks after enlarging References. Long caller notes remain
scrollable; clipping at a region boundary is intentional. Final fixture hashes
are unchanged and app/compositor exits are zero. The pre-repair 2× overlap capture
and previous layout body are retained. An excluded harness trial passed a floating
coordinate to an integer-only pointer driver; its forced cleanup is not accepted
product evidence. Final drivers use rounded coordinates and fresh private roots.

Archive: `~/.local/state/radiology-qa/reference-presentation-20261031-f29df40`.
It includes final native actions/captures/index/source/hash fixtures, pre-repair
layout evidence, three negatives, validation logs, complete architecture reports,
clone audit, source patch and binary/commit provenance. QA reuses sibling compositor
helpers only; the app remains independently buildable with no product dependency.

## Remaining scope

Check reference-to-caller focus handoff with Right/Down/Return in one raw-input
batch; the accepted native runs separate the initial Right registration. Pinned
summary kind/icon metadata still differs from Swift's `InspectorSymbolSummary`.
Broader pinned-review/mixed-language/error/Trace More visual and lifecycle cases,
reader/large-file/performance/grammar/index/PR/cache acceptance and the final Swift
requirement/reuse audit remain open. Full assistive accessibility and actual
sibling adoption remain deferred; keyboard behavior stays required.
