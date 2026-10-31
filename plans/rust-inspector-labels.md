# Inspector labels and declaration signature disclosure

The Swift reference is `DeclarationSummaryView`, `InspectorSymbolSummary`,
`SignatureText`, `DefinitionDetailsView`, `OutlineRow` and `BookmarkRow`.
`ContextModel.setDeclaration` closes signature disclosure on declaration changes;
`DeclarationContext.maxSignatureLines` is eight. This increment restores that
missing disclosure and applies compact presentation to the Rust inspector.

## Behavior and ownership

The declaration starts collapsed. Its keyboard-focusable triangle opens a
selectable signature, limited to eight rendered rows (fewer when space is tight).
Both the fitted name and filename/line link return the original declaration byte.
The signature's shared reader context menu offers Copy Signature, including Ctrl+C
while the menu owns input; it copies the entire original signature, beyond the
4 KiB presentation bound. Existing reference/definition/bookmark dispatch remains.

`DeclarationSummary` is ephemeral view state passed to `ContextInspector::show`.
It reuses `SourceIdentity` (path, review revision, cached SHA-256 digest), paired
with the declaration selection range. It retains disclosure only for that exact
owner. Different declarations, paths, revisions or bytes close it; returning to
an earlier declaration stays collapsed. Pinned references and missing context
clear it. The standalone app resets it at repository replacement, including roots
with identical relative paths and bytes. Reusing hosts must perform that same
reset, as documented on the public type. No new persistence or product dependency.

Shared measured middle fitting now bounds shaping/help to a UTF-8-safe 4 KiB
head/tail sample. Display bounds leave symbol names, bookmarks, signatures and
indexed destinations intact. Declaration/pinned/under-caret names, location and
package captions, bookmark scope/labels/locations and outline names reuse it.
Deep outline indentation is capped at 35% of the row so labels remain legible;
hierarchy, disclosure, changed markers, line counts and exact targets remain.
Definition facts use two rows, signatures eight and prose four, matching Swift's
signature/prose limits. Quiet location/metadata use the existing Small type role;
code uses the established 12-point monospace role. Source previews remain shared.

## Verification (2026-10-31)

Three added regressions cover actual Tab/Space disclosure, exact full-signature
copy through the owned menu, exact declaration and indexed-location actions,
owner retirement, huge Unicode names and signature display bounds. The existing
three-region test now exercises long declaration/bookmark labels and 20-level
outlines at 220/300/420-point pane widths, both themes and 1×/2×. Indexed details
also exercise the minimum 216-point pane width. Shortcut captions and all three
regions remain visible. Scale tests settle egui's pending zoom before measuring
unconstrained central-panel geometry.

Replacing the indexed renderer with its previous production implementation makes
the new long-details regression fail its shaping bound. The saved negative is
`indexed-negative.log`. Workspace passes 489 app tests (seven ignored), two binary,
four font and eleven text tests, plus 34 doctests. Final focused tests, strict
Clippy, workspace/all-target check, fmt, build and diff checks pass. Warning-denied
rustdoc passes; rendered `DeclarationSummary` ownership docs and `show` signature
were inspected. Architecture reports IMPROVED: clone groups 92 → 89, no new
hotspot/cycle or max-CCN increase; average CCN 3.36 → 3.37. Test setup reuses the
shared painter and paired palette/scale setup; no duplicate production mechanism.

Private native Wayland runs at 1×/1280×900 and 2×/1024×768 use a real nested Git
fixture with a 486-byte valid Unicode function identifier. Both show the collapsed
and expanded declaration, fitted outline/location/bookmark labels, narrowed/restored
panes, light/dark themes and collapse after peer-file navigation/Back. Both copy
the exact 500-byte signature (SHA-256
`f19edfcd715f1bd52c43eee218aa8a503c0e77645b899b19696dc3b3a89799d4`).
The 1× light capture also verifies expansion in the 236-point narrowed pane.
The 2× light narrow capture verifies collapsed geometry; the later attempted
narrow disclosure click hit the resize edge, so that capture is not evidence of
expanded-light behavior. The default-width 2× dark expansion is accepted. Native
fixtures have no semantic index; indexed detail actions are headless coverage.
Fresh clipboard offers match, fixtures remain byte-identical, and final app and
compositor exits are zero. Earlier exploratory click/End frames are excluded.

Evidence archive: `~/.local/state/radiology-qa/inspector-labels-20261031-29a178d`.
It contains final drivers/actions/captures, fixtures/hashes, validation logs,
architecture baseline/comparison/measurement, source patch and provenance.

Broader reference-list/pinned-review/mixed-language visual acceptance, cross-owner
keyboard/enabled-state checks, reader/large-file/performance, grammar/index/PR/cache
cases and final requirement-by-requirement Swift/reuse audit remain open. Full
assistive accessibility and actual sibling integration stay deferred. The app is
standalone; QA-only sibling compositor helpers introduce no product dependency.
