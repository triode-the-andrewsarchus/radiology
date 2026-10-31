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

## Pinned reference kind and category — 2026-10-31

Pinned summaries now show the SCIP kind name and a category marker beside the
fitted symbol name, matching Swift's `InspectorSymbolSummary` information hierarchy.
The five categories follow `SemanticSymbolKind`: type, function, value, module
and other. Their compact glyphs use the existing outline convention and body type
role; kind metadata uses Small/secondary. Automatic declaration headers also show
their existing parsed-kind glyph. No icon package, asset dependency or new type
scale was introduced.

The app resolves metadata through `symbol_info_for_id` using the effective pinned
identity, including its captured document for `local` symbols, independently of
the current source. Metadata reflects the current accepted index. Missing metadata
shows Symbol; unknown codes retain their numeric kind name and neutral marker.
The public `InspectorContent.reference_kind` field documents these host obligations
and accepts `0` for unavailable metadata. The renderer takes no index ownership
and stores no second metadata snapshot. Pinned signature disclosure remains closed.

One shared internal SCIP category classifier serves presentation and the existing
callability check, replacing the latter's separate function-kind table. Descriptor
fallback callability remains intact. Existing `kind_name`, scoped symbol lookup,
measured fitting and source-bound declaration state are reused.

Three new tests cover representative wire kinds/unknowns, all categories and
missing metadata with long names at 216/300/420-point widths in both themes/scales,
and a real app pin to a local variable while another document has the same local
ID with parameter metadata. The app case retains Variable after navigation,
reflects an index refresh to Kind 9001, and releases the pin with Follow Caret.
The fixture explicitly reloads its rewritten source before pinning, preserving
the production contract that ordinary same-file navigation reuses displayed bytes.
The previous pinned-summary branch fails the new metadata rendering test.

Workspace passes 500 app tests (seven ignored), two binary, four font, eleven text
and 34 doctests. Final focused cases, strict Clippy/check/build/fmt/diff and warning-
denied rustdoc pass; rendered `reference_kind` documentation and its link were
inspected. Architecture is NEUTRAL, retaining 89 clones, 24 high-complexity functions
and max CCN 100, with no new hotspot/cycle or package dependency. A repeated app assertion
was replaced by a navigation matrix, preserving both source-document checks.

Native 1×/2× fixtures now include real SCIP SymbolInformation kind 17. Captures
show the Function label/marker, narrow dark/light pinned summaries, caller entry
navigation and Follow Caret returning to the enclosing declaration header. Both
runs retain exact 473-byte Unicode source copy, Back restoration, unchanged fixture
hashes and zero app/compositor exits. Archive:
`~/.local/state/radiology-qa/pinned-metadata-20261031-5a5c7cc`.

Broader modal/global keyboard ordering, pinned-review/mixed-language/error and
lifecycle visuals, reader/large-file/performance/grammar/index/PR/cache acceptance
and final Swift/reuse audit remain open. Full accessibility and actual sibling
adoption remain deferred; standalone builds and keyboard behavior stay required.
