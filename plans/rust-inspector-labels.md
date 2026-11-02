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


## Contextual declaration absence — 2026-11-01

Swift's DeclarationRegionView renders No File/No Declaration with an honest reason
unless an enclosing declaration or explicit reference target exists. Rust now
follows this rule instead of always showing Declaration, References · 0 and an
index-availability message. No File explains how to open source. Parsed source
outside a declaration explains where to place the caret; unsupported/failed
structure reports that structure is absent, or Reading structure while preparing.
Accepted parsed source keeps its caret guidance while another source prepares.

The existing accepted buffer, language registry, parser error and preparation
state supply the reason without new persistent state or background work. Shared
bounded explanations retain Body/Small roles. Explicit reference pins keep their
summary/results and Follow Caret regardless of the enclosing declaration. Releasing
the pin outside a declaration returns to the empty state. Losing a declaration
resets the existing signature disclosure, matching Swift ContextModel retirement.
No new production function, dependency, cache, worker or public API.

Existing real app outline cases now check declaration absence for Rust/Kotlin
without declarations, Markdown examples and hidden-test projections; normal
parser-failure publication and repository reset check No Declaration/No File.
The local-symbol metadata test retains an explicit pin after leaving its function,
then verifies Follow Caret removes both the pin and reference region. New narrow
180/300-point dark/light 1×/1.5×/2× fixtures cover no source, unsupported/failed
structure, loading and accepted parsed source with three-row bounds. A keyboard
disclosure test expands a signature, leaves its declaration and returns collapsed.
An existing app case fails before the repair; removing disclosure retirement
independently fails the new regression. Restored inspector tests pass all eight.
Shared borrowed InspectorContent setup replaces repeated fixture construction.

Workspace passes 547 app tests (seven ignored), two binary, four font, 13 reader-text
and 36 doctests. Strict Clippy/check/build/rustdoc/fmt/diff pass. Architecture adds
215 NLOC including tests and three test functions/fixtures. Declaration rendering
is CCN 8; no new hotspot, cycle or maximum complexity. Full normalized clone audit
retains all 91 baseline groups with no addition/removal; the raw touched-file flag
is unchanged reference-test content. Aggregate health changes -0.02.

Eight native private-window cases at 1×/2× cover no repository/source, Markdown
without a structural parser, parsed Rust outside/inside/outside a function, and
a controlled imported index. A pinned alpha target remains visible after Ctrl+End
places the caret beyond beta's parsed range; Follow Caret restores No Declaration
with caret guidance and no reference region. All loaded cases copy their complete
source exactly. Captures, final binary/fixture hashes and zero child exits pass.
Initial QA reused the startup initial.png filename, allowing its asynchronous
replacement to race OCR and produce stale No File observations. Those trials are
retained and excluded. Final capture names are unique and the controller polls
for required/forbidden region text before accepting each transition.

Evidence: `~/.local/state/radiology-qa/declaration-empty-20261101-924bbe3`.
Broader inspector/input/pinned-review/language corpora, reader/index/PR/cache and
final Swift visual/feature/standalone/reuse audit remain open. Full accessibility
and actual sibling integration remain deferred.


## Shared symbol headings and stable definition controls — 2026-11-01

Swift UnderCaretDetailView omits its name when the enclosing-declaration or pinned
reference summary already identifies that exact symbol. Rust now follows this
rule. Different global IDs and document-scoped local IDs retain their own names,
even when display names or raw local strings match. Unknown summary identity
retains the lower name. Location, details, source preview and keyboard navigation
remain available when the duplicate caption is omitted.

InspectorContent adds the borrowed optional `summary_identity` presentation field.
Callers pass the indexed identity displayed above, or None when unknown. App
rendering now derives name, identity and kind from one existing ReferenceTarget
lookup. This changes the inspector input API, without new storage, query work,
application state, cache, worker or dependency. The existing indexed-context
fixture is shared through test_support; one owning inspector fixture replaces
repeated default input setup. Existing typography roles are unchanged.

A conditional-widget regression caught a real issue before commit: pressing the
name, hiding it and releasing over the newly positioned location could navigate.
The widgets had inherited the same automatic egui ID. Definition name and body
now use separate role/source owners, including symbol, captured location/digest
and exact destination path/SCIP coordinates. A shared `commands::owned_ui` root
seeds automatic as well as persistent IDs; egui 0.27 push_id alone does not do so.
It inherits layout, style, clip, enabled and visible state for reader content.
The existing live-rectangle activation guard also rejects moved controls. A
changed destination retires its old press, while a fresh click still navigates.
No extra gesture cache or separate command path is introduced.

The real app local-symbol test initially paints the pinned name twice; it now
paints it once and retains a different document's parameter name. A narrow
dark/light 1×/1.5×/2× matrix covers matching/different/unknown global and local
identities, exact link actions, details focus and Right/Left/Return commands.
Expanded signatures/documentation remain present without the name. Comparing only
raw symbol text fails the different-document regression. The conditional-heading
press regression fails before widget ownership; its destination replacement and
fresh-click checks now pass. Final focused under-caret tests pass all three.

Workspace passes 549 app tests (seven ignored), two binary, four font, 13 reader-text
and 36 doctests. Strict Clippy/check/build/rustdoc/fmt/diff pass. Architecture adds
179 NLOC including tests and three functions: the shared widget helper and two
regressions. The helper is CCN 1; under-caret show remains CCN 6 and parent rendering
remains CCN 15. No new hotspot/cycle/max-CCN or clone group. All 91 baseline clone
groups remain unchanged after normalized multiset audit; health changes -0.01.

Final native 1×/2× indexed flows show a distinct lower symbol, suppress its name
after pinning, expand/collapse details, navigate with Return, restore Back, release
with Follow Caret and suppress the name for an automatic enclosing declaration.
Navigation retains inspector focus; QA uses Ctrl+Alt+I to return to the reader
before copying the exact destination name `alpha`. Final Back copies complete
b.rs bytes. Final binary/fixture hashes, inspected captures and zero exits pass.
Only `definition-heading-owned` runs are final-build acceptance. Earlier OCR
failures, an incorrect source-focus assumption and pre-guard binaries are retained
and excluded. Final QA uses observed pane bounds, normalized control labels,
distinct source breadcrumbs and exact clipboard payloads.

Evidence: `~/.local/state/radiology-qa/definition-heading-20261101-380dae8`.
Broader inspector/pinned-review/interaction corpora, reader/language/index/PR/cache
and final Swift feature/visual/standalone/reuse acceptance remain open. Full
accessibility and actual sibling integration remain deferred.
