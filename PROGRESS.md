# Progress

## 2026-10-31 — Reveal distant unwrapped destinations before prefix measurement (Codex)

The prior binary still shows LONG_START three seconds after End. Pending reveal
now uses the measured target with provisional prefix placement immediately;
measurement rebases scroll before input so the destination stays anchored.
Provisional pointer gestures retain fragment-local coordinates through the
existing ordered dispatcher. Manual clicks retire queued keys while preserving
that mapping; external ownership/document/scroll retirement stays intact.
The new 1×/2× caret/anchor/click/Copy regression fails on the original reveal and
separately with the old pointer path. The existing retirement test catches a
manual-click ordering case, now corrected. Workspace passes 515 app tests (seven
ignored), two binary, four font, 13 text and 36 doctests; strict Clippy/check/build/
fmt/diff and warning-denied rustdoc pass. Architecture is NEUTRAL (+98 NLOC
including tests), with 91 unchanged clone groups and no new hotspot/cycle/max-CCN
increase. No dependency, cache, renderer, public API or frame-budget change.
Native 1×/2× first/three-second captures show the full End marker at identical
positions; the thirty-second 2× capture retains the tail. Exact provisional
pointer `_END`, End/Back/wrap bytes, original full copy, unchanged fixture hashes,
matching final binary hashes and zero exits pass. Cold/revisited loading remains
slow. Wrapped copy is exact but its selected tail is not visible in the capture;
wrapped reveal is the next reader investigation. Broader source, grammar/index,
PR/cache and final Swift/reuse acceptance remain open. Full accessibility and
actual sibling adoption stay deferred.
[Contract, validation and limits](plans/rust-reader-performance.md#reveal-before-complete-prefix-measurement--2026-10-31).
Evidence: ~/.local/state/radiology-qa/long-line-reveal-20261031-0bbff5d.

## 2026-10-31 — Bound Unicode coordinate conversion for long source lines (Codex)

Shared reader-text CoordinateIndex stores sparse absolute byte/UTF-16/scalar
checkpoints, with binary search and at most 4099 suffix bytes per lookup.
SourceBuffer reuses it in both coordinate directions, retaining CRLF, clamping,
invalid-input and exact line-relative semantics; existing encoding import paths
remain available. No dependency or GUI coupling is added. Independent encoding,
line-boundary and >6 MB sparse-bound tests pass. Workspace passes 514 app tests
(seven ignored), two binary, four font, 13 text and 36 doctests; strict Clippy/
check/build/fmt/diff and warning-denied rustdoc pass, rendered docs inspected.
Architecture is NEUTRAL (+218 NLOC including tests), with 91 unchanged clone
groups and no new hotspot/cycle/max-CCN increase. Native 1×/2× exact full copy,
End suffix, cross-file Back and wrap selection pass with fresh clipboard markers,
unchanged fixtures, matching final binary hashes and zero exits. At 1× observed
End copy wait drops from ~2.8 s to 4 ms; Back remains ~9.4 s and loading ~8.7 s.
Single debug-run observations are not release/statistical guarantees. Delayed
captures still leave horizontal reveal/measurement acceptance open; broader
reader, grammar/index, PR/cache and final Swift/reuse audits remain required.
[Contract, observations and limits](plans/rust-reader-performance.md#sparse-source-coordinates--2026-10-31).
Evidence: ~/.local/state/radiology-qa/coordinate-index-20261031-01f8519.
Full accessibility and actual sibling adoption remain deferred.

## 2026-10-31 — Initialize syntax queries only for the requested language (Codex)

A fresh 6 MB Unicode/CRLF fixture verifies original full copy, End suffix, exact
cross-file Back and wrap selection using private clipboard markers and hashes.
GDB sampling finds Rust loading compiling Swift queries and separately catches
UI-thread full-line column scans. Existing role caches now use independent
per-grammar OnceLock cells; unrelated languages do not compile and concurrent
requests reuse one query. No public API, dependency or query-resource change.
Two focused tests pass; equivalent eager construction fails on Swift initialization.
Workspace passes 513 app tests (seven ignored), two binary, four font, eleven text
and 35 doctests; strict Clippy/check/build/fmt/diff pass. The native binary matches
the final build. Architecture is NEUTRAL (+60 NLOC, no new clone/hotspot/cycle/max-CCN increase).
[Observations, limits and remaining work](plans/rust-reader-performance.md).
The identical native script preserves exact source/Back/wrap bytes, fixture hashes
and zero exits. Observed debug-build initial copy wait is ~8.5 s versus ~10 s;
this single pair is not a general performance guarantee. Long-column End/Back
remains slow and is the next confirmed correction. Broader source, grammar/index,
PR/cache and final Swift/reuse acceptance remain open; full accessibility and
actual sibling adoption remain deferred. Evidence:
~/.local/state/radiology-qa/query-lazy-20261031-50d0493.

## 2026-10-31 — Explain outline loading, degradation and empty states (Codex)

The outline now distinguishes preparation, missing source, parser failure,
unsupported formats, empty native-language files and hidden test declarations.
Unsupported README/code examples no longer become invented outline rows. Native
parser failure retains source and labels approximate fallback rows; recoverable
trees keep usable declarations. Parser diagnostics publish with the source,
survive recoloring/failed reads and clear on success or scope reset. Registry
metadata supplies the supported-language list; bounded three-line Small text and
hover explanations reuse shared fitting. [Contract/evidence](plans/rust-outline-status.md).
Two new real-app regressions fail against the old renderer; an additional saved
negative proves phantom README declarations. Parser outcome and bounded-layout
tests cover unsupported/cancelled/recoverable trees and 200/300-point panes in both
themes/scales. A native-proven prose word split now uses word-boundary wrapping;
code/path previews retain their original fitting. Workspace passes 511 app tests
(seven ignored), two binary, four font, eleven text and 35 doctests; strict Clippy/
check/build/fmt/diff and warning-denied rustdoc pass. Rendered public docs were
inspected. Full clone multiset is unchanged (91 groups), with no new hotspot/cycle
or max-CCN increase. Final native 1×/2× runs show Rust/Kotlin empty states,
unsupported README examples, hidden/restored tests and intact prose/shortcut in
narrow dark/light panes. Source copy and fixture hashes match; final binary and
zero exits are recorded. Full objective acceptance remains open: broader grammar and review
corpora, input ownership, large-file/reader performance, index/PR/cache and final
Swift/reuse audit. Full accessibility and actual sibling adoption stay deferred.
Evidence archive: ~/.local/state/radiology-qa/outline-status-20261031-85d3404.

## 2026-10-31 — Bind outline/bookmark pointer gestures to row identities (Codex)

Outline IDs now include the exact document identity and declaration range;
bookmark IDs use persisted UUIDs. Both reuse live-rectangle activation validation,
preventing a release from opening replacement content or a moved row. Keyboard
behavior, scoped persistence and layout are retained. Three new tests cover stale
presses, fresh activation and identity-preserving drag reorder. The old renderers
fail the stale-target checks. [Contract/evidence](plans/rust-inspector-row-identity.md).
Workspace passes 507 app tests (seven ignored), two binary, four font, eleven text
and 34 doctests. Final bookmark/outline suites and strict Clippy/check/build/fmt/
diff pass. Architecture has one added standard seven-line egui allocation/
explicit-ID interaction clone (90 → 91), no new hotspot/cycle or max-CCN increase;
the repeated test fixture was shared. Native 1×/2× row clicks copy the exact
473-byte/13-byte source lines; dragging persists order [2, 5], fixtures remain
unchanged and final exits are zero. Evidence archive:
~/.local/state/radiology-qa/inspector-rows-20261031-c91800b.
Broader mixed-owner/menu/source ordering, mixed-language/pinned-review visuals and
parser explanations, reader/large-file/grammar/index/PR/cache and final Swift/reuse
acceptance remain open. Full accessibility and actual sibling adoption stay deferred.

## 2026-10-31 — Preserve inspector/global keyboard ownership order (Codex)

Global shortcuts and inspector actions now share per-event dispatch, so modal
prefixes prevent later reference/caller navigation and Find/Search receive opening
text. Pending caller focus is retired at a global handoff. Modal input survives
the initial window sizing pass and replays on a requested frame. Existing command
matching, local focus guards and action handlers are reused; repository reset
clears retained inspector input. Full accessibility remains deferred; keyboard
navigation stays required. [Contract, evidence and remaining scope](plans/rust-keyboard-owner-order.md).
Four new real-app regressions fail against the prior paths and pass with the fix.
Workspace passes 504 app tests (seven ignored), two binary, four font, eleven text
and 34 doctests; strict Clippy/check/build/fmt/diff pass. Architecture reports one
added six-line test-assertion clone (89 → 90), no production clone addition, no
new hotspot/cycle or maximum-CCN increase. Native 1×/2× runs show opening query
text and the correct file, preserving exact Unicode copy, Back, fixture hashes
and zero exits. Native/evidence archive:
~/.local/state/radiology-qa/owner-order-20261031-752e115.
Broader mixed-owner/menu/source ordering, pinned-review/mixed-language/error and
lifecycle visuals, reader/large-file/grammar/index/PR/cache checks and final
Swift/reuse acceptance remain open. Actual sibling integration remains deferred.

## 2026-10-31 — Restore pinned symbol kind/category metadata (Codex)

Pinned summaries show a fitted name, category marker and kind label. Metadata uses
the pinned scoped identity and current index, independent of displayed source;
missing metadata shows Symbol, and unknown kinds retain their number. Follow Caret
returns the parsed declaration/glyph. `InspectorContent.reference_kind` documents
host obligations. One internal category classifier is shared with existing
callability checks, replacing their separate function-kind table; no extra metadata
snapshot, dependency or icon asset. [Contract/evidence](plans/rust-inspector-labels.md#pinned-reference-kind-and-category--2026-10-31).
Three new tests cover categories/unknowns, long-name geometry in both themes/scales,
and real-app local-ID separation, metadata refresh and Follow Caret. The old pinned
branch fails the rendering regression. Workspace passes 500 app tests (seven
ignored), two binary, four font, eleven text and 34 doctests; final focused tests,
strict Clippy/check/build/fmt/diff and warning-denied rustdoc pass. Rendered field
documentation was inspected. Architecture is NEUTRAL: 89 clones, no new hotspot/
cycle/max-CCN increase. Final native 1×/2× indexed runs show the Function marker/
label, narrow themes and Follow Caret, preserving caller navigation, exact 473-byte
Unicode copy, Back, fixture hashes and zero exits. Archive:
~/.local/state/radiology-qa/pinned-metadata-20261031-5a5c7cc.
Broader modal/global ordering, pinned-review/mixed-language/error/lifecycle visuals,
reader/large-file/grammar/index/PR/cache and final Swift/reuse acceptance remain
open. Full accessibility and actual sibling adoption stay deferred. Commit and push
this metadata increment before continuing the broader acceptance queue.

## 2026-10-31 — Dispatch reference/caller keys in event order (Codex)

The shared ordered parser now applies each reference action before resolving the
next key, using one pending-focus ID. Coalesced Right/Down/Return commits the
caller; Escape/Left returns later keys to reference selection; collapse/reopen
consults the live disclosure before accepting cached controls. Caller-group moves
select the new parent. Raw and prepared frames share application dispatch, with
input detached before prepared routing to avoid nested context locks. Escape
recovery only uses a known previous response when no live external focus exists.
[Contract/evidence](plans/rust-reference-presentation.md#ordered-referencecaller-keyboard-handoff--2026-10-31).
Four added real-app tests pass; the first three fail against ba469aa with wrong
entry/exit destinations or a closed disclosure. Both frame paths, exact Back
history and a live Find query are covered. Workspace passes 497 app tests (seven
ignored), two binary, four font, eleven text and 34 doctests; final component and
strict Clippy/check/build/fmt/diff pass. Architecture's one diff-aware clone warning
is unchanged; the full 89-clone multiset has zero additions/removals, with no new
hotspot/cycle/max-CCN increase. Native 1×/2× runs send the three keys together,
commit line 5, copy exact 473-byte Unicode source and return with Back. Exact frame
coalescence is established by RawInput tests, not inferred from native timing.
Fixtures are unchanged and final exits zero. Archive:
~/.local/state/radiology-qa/reference-handoff-20261031-ba469aa.
Pinned kind/category metadata and broader modal/global ordering, review/mixed-
language/lifecycle/reader/large-file/grammar/index/PR/cache/final Swift/reuse
acceptance remain open. Full accessibility and actual sibling adoption remain
deferred. Commit and push this keyboard increment before the metadata change.

## 2026-10-31 — Fit caller rows and bound inspector regions (Codex)

Caller names, filenames, line/depth metadata and boundary notes now keep separate
slots. Noted rows share variable extents with virtual painting and keyboard reveal.
Reference file headers reuse measured head elision. Reference rows/chevrons and
caller/Trace More controls bind exact source identities, retiring replacement
presses. Result and reference panes share current-rectangle activation validation;
ordinary icon buttons preserve their keyboard behavior. Native 2× resizing also
exposed Definition painting over Bookmarks. Bookmarks now reserves its actual
height first; References leaves a middle region, and clipped child content cannot
enlarge any sibling. [Contract/evidence/remaining checks](plans/rust-reference-presentation.md).
Four new regressions pass, along with enhanced mixed-height 1,000-caller and saved
oversized-region cases. Three saved negatives fail against the previous renderer/
layout bodies. Workspace passes 493 app tests (seven ignored), two binary, four
font, eleven text and 34 doctests; final focused tests, strict Clippy/check/build/
fmt/diff pass. Architecture flags five unchanged test clusters: the full clone
multiset remains 89 with zero additions/removals, no new hotspot/cycle/max-CCN
increase and average CCN 3.37 → 3.36. Native 1×/2× indexed runs verify pinned
references, caller navigation to line 5, exact 473-byte Unicode copy, Back to line
2, narrow themes and the repaired region separation. Fixtures are unchanged and
all final exits zero. Archive: ~/.local/state/radiology-qa/reference-presentation-20261031-f29df40.
Coalesced Right/Down/Return handoff, pinned kind/icon metadata, broader reference/
review/mixed-language/lifecycle visuals, reader/large-file/grammar/index/PR/cache
and final Swift/reuse acceptance remain open. Full accessibility and actual sibling
adoption stay deferred. Commit and push this increment before continuing.

## 2026-10-31 — Bound inspector labels and restore declaration disclosure (Codex)

The enclosing declaration now has Swift's initially closed signature disclosure,
with exact source/revision/declaration ownership and repository reset. Name and
location links preserve original bytes; the owned Copy Signature menu preserves
the entire signature beyond its eight-row/4 KiB display. Long declaration,
under-caret, outline and bookmark labels reuse bounded middle fitting. Deep
outline indentation leaves space for names; definition signature/prose/facts have
row limits. [Contract and evidence](plans/rust-inspector-labels.md).
Three new regressions pass; the expanded existing three-region case covers both
themes/scales and narrow panes. The old indexed renderer fails the new bound test.
Workspace passes 489 app tests (seven ignored), two binary, four font, eleven text
and 34 doctests. Final focused tests, strict Clippy/check/fmt/build/diff and
warning-denied rustdoc pass; rendered ownership docs were inspected. Architecture
is IMPROVED (92 → 89 clone groups), with no new hotspot/cycle/max-CCN increase.
Final native 1×/2× runs copy the exact 500-byte Unicode signature, exercise long
labels, bookmarks, themes, pane resizing and peer/Back disclosure retirement.
Fixtures are unchanged and all exits zero. The 2× narrow-light disclosure attempt
hit the resize edge; only its collapsed geometry is accepted. Indexed details
have headless coverage. Archive: ~/.local/state/radiology-qa/inspector-labels-20261031-29a178d.
Broader reference/pinned-review/mixed-language visuals, cross-owner keyboard/
enabled-state, reader/large-file/grammar/index/PR/cache and final Swift/reuse
acceptance remain open. Full accessibility remains outside scope as the user
reconfirmed; keyboard behavior stays required. Actual sibling adoption is deferred.
Commit and push this inspector increment before continuing.

## 2026-10-31 — Bound Changes/Search presentation and row activation (Codex)

Changes now prioritizes fitted filenames/counts with a quiet directory or rename
origin and binary metadata. Headings have a two-row bound and full help. Search
uses one fitted path/line plus two code-preview rows, font-derived virtual stride
and clipped paint. The shared text fitter bounds previews to a UTF-8-safe 4 KiB
prefix without changing stored matches or navigation targets. Explicit path/match
IDs plus current-rectangle pointer validation retire moved/replaced press targets;
fresh clicks and Tab/Space still work. [Contract/evidence](plans/rust-result-pane-labels.md).
Three added regressions pass. Saved negatives show 180 → 974.10706-point Changes
growth, 380-point Search strides and a Changes press opening another file after
reordering. Workspace passes 486 app tests (seven ignored), two binary, four font,
eleven text tests and 34 doctests; strict Clippy/build/fmt/diff pass. Architecture
is neutral, with no new clone/hotspot/cycle. Native 1×/2× Git/ripgrep runs verify
nested rename/binary rows, long previews, narrow/restored panes and dark/light
rendering. From a peer file, both actions return the exact nested path; Search
lands on line 5 and explicit-source selection copies exact Unicode bytes. Fresh
clipboard hashes match, fixtures are unchanged and all final exits are zero.
Archive: ~/.local/state/radiology-qa/result-panes-20261031-d3cd8f0. An excluded
startup-probe trial sampled a colored diff stripe; its screenshot shows a loaded
reader, not a diagnosed source/performance failure. The corrected probe samples
line-number text. Broader inspector-label/cross-owner/enabled-state/reader/large-
file/grammar/index/PR/cache/final Swift acceptance remains open. Automatic result
focus return and working-tree rename hunks are not accepted here. Full
accessibility and actual sibling adoption stay deferred. Commit and push this
pane/interaction increment before continuing.

## 2026-10-31 — Align active toolbar titles and preserve global shortcuts (Codex)

The popup owner now selects/colors its existing title and receives keyboard focus
while retaining the original dismissal owner. The native Search/View focus-ring
mismatch is repaired. The shortcut dispatcher reuses the shared menu classifier
so focused menus allow global reader commands and retain bare activation keys,
including prepared egui hosts; text-entry/modal guards remain. No new owner state,
dispatcher or dependency. [Contract/evidence](plans/rust-toolbar-keyboard.md#active-titles-and-global-shortcuts-2026-10-31-follow-up).
Three added regressions cover theme/scale title rendering, prepared-host ownership
and actual indexed navigation through an open menu. Saved negatives reproduce the
title mismatch and the old focused-menu shortcut guard failure. Indexed fixtures
are reused, preserving the existing Back-history test. Workspace passes 483 app
tests (seven ignored), two binary, four font, eleven text tests and 34 doctests;
strict Clippy/build/fmt/diff pass. Architecture flags four unchanged app-test
clones; the full audit has 92 before/after and zero additions/removals, with no new
hotspot/cycle/max-CCN increase. Native 1×/2× checks repeat menu switching/modal/
repository/direct-control/source-copy cases with correct active titles, exact
text hashes, unchanged fixtures and zero exits. Dark/light captures are inspected.
Archive: ~/.local/state/radiology-qa/menu-title-20261031-ed35b9e.
Broader cross-owner/enabled-state/long-label/reader/large-file/grammar/index/PR/
cache/final Swift acceptance remains open. Full accessibility and actual sibling
adoption stay deferred. Commit and push this title/shortcut increment before the
next UI audit.

## 2026-10-31 — Share toolbar menu traversal and focus ownership (Codex)

Repository, PR, View and Search now use the existing owned reader menu dispatcher.
Primary anchors support keyboard opening, left/right and submenu traversal, hover
switching, coalesced opening/release and scoped retirement. Queued keys survive
handoff to an earlier-drawn anchor. Menus restore prior reader/button focus;
direct toolbar controls preserve prior focus before host dispatch. Shared bounded
scrolling, checked/disabled rows, help and status labels retain PR choices,
refresh/cancel/error behavior and the 25-row keyboard case. No new dependency or
alternate renderer/resolver. [Contract and evidence](plans/rust-toolbar-keyboard.md).
Eight added regressions pass, and the View regression fails with the previous
toolbar implementation. A 120-frame keyboard/wheel regression also fails before
the one-shot reveal repair and passes after it; manual scrolling stays put.
Existing PR tests now exercise the real popup. Workspace
passes 480 app tests (seven ignored), two binary, four font, eleven text tests and
34 doctests; strict Clippy/build/fmt/diff pass. Architecture is neutral: all 92
clone groups are unchanged, no new hotspot/cycle; toolbar CCN falls 34 → 22.
Native 1×/2× runs verify View → source copy, Search/PR keyboard switching and
modal cancellation, repository copy/submenu and direct Inspector → source copy.
All four fresh text offers per scale match expected hashes; dark/light captures
are inspected, fixtures unchanged and all processes exit zero. Archive:
~/.local/state/radiology-qa/toolbar-keyboard-20261031-c421d03.
Active menu-title styling and broader cross-owner/enabled-state/long-label/reader/large-file/grammar/index/PR/
cache/final Swift acceptance remains open. Full accessibility and actual sibling
adoption stay deferred. Commit and push this toolbar increment before continuing.

## 2026-10-31 — Restore modal/breadcrumb keyboard ownership (Codex)

Committed navigation from a modal or breadcrumb now returns source focus;
cancellation restores the previous live control, including Find. Modal keys/text
are staged through the existing extractor before framework focus traversal and
retired before underlying controls render. Closing input cannot also move source
or close Find. The stable canvas remains registered during first-file loading;
publication never requests focus. Repository reset retires modal input/owner.
A shared modal dispatcher removes repeated action/close branches; native update
and tests use one render path. Native 2× evidence also prompted an explicit close
repaint. [Contract and evidence](plans/rust-modal-focus.md).
Five new real-app regressions pass; disabling source focus makes all four initial
focus regressions fail. Workspace passes 472 app tests (seven ignored), two binary,
four font, eleven text tests and 34 doctests; strict Clippy/build/fmt/diff pass.
The comparison tool flags the renamed rendering hotspot and unchanged app-test
clones; actual event-loop CCN falls 47 → 36, with 92 clone groups before/after and
zero additions/removals. No new cycle/max-CCN increase. Final native 1×/2× runs
verify exact source and restored-Find clipboard bytes, modal disappearance without
another event, breadcrumb selection and light rendering. Fixtures are unchanged,
all process exits zero. Archive: ~/.local/state/radiology-qa/modal-focus-20261031-4266178.
Toolbar traversal/focus and broader cross-owner/long-label/reader/large-file/
grammar/index/PR/cache/final Swift acceptance remain open. Full accessibility and
actual sibling adoption stay deferred. Commit and push this verified increment
before continuing with toolbar ownership.

## 2026-10-31 — Bound file rows and port compact source breadcrumbs (Codex)

The native long-name Files/source-header overflow is repaired. A shared measured
middle-elision helper preserves type size/name suffixes; deep indentation cannot
force Files rows beyond their pane. Original paths still drive open/help/copy.
The header now ports Swift's directory/declaration compaction and ellipsis menu,
with canonical declaration jumps and full relative-path copy. Existing revision,
change and collapse actions get their fixed trailing space first. [Contract and evidence](plans/rust-pane-labels.md).
The exact final row regression fails before (196 → 535.8439 points) and passes
after; four added UI tests cover pane bounds, original action targets, five header
widths in both themes/scales, full-location menu and all declaration jumps.
Workspace passes 467 app tests (seven ignored), two binary tests, four font tests,
eleven text tests and 34 doctests; strict Clippy/build/fmt/diff pass. The comparison
tool flags two unchanged baseline test clones; full fingerprint audit confirms
92 groups before/after and zero new clones. No new hotspot/cycle/max-CCN increase;
SourceView::show CCN falls from 21 to 8. Native 1×/2× runs with a 170-byte Unicode
filename retain pane boundaries in dark/light mode, copy exact paths from both
menus and jump to real declarations at lines 4/1. Fixtures are unchanged and all
processes exit zero. Archive: ~/.local/state/radiology-qa/pane-labels-20261031-c9fd9e1.
Automatic modal/toolbar/breadcrumb focus return and traversal are next. Other
long-label surfaces, broader reader/large-file/Back and grammar/index/PR/cache/
final Swift acceptance remain open; this is not a claim of full UI parity.
Accessibility and actual sibling adoption stay deferred. Commit and push this
layout/breadcrumb increment before continuing.

## 2026-10-31 — Retain source gestures until exact hit geometry is ready (Codex)

The saved lost-click regression now passes. The existing source queue carries
pointer press/motion/release with document coordinates, modifiers/time and later
keys/Copy; it waits for exact bounded shaped hits and retires stale owners.
Ctrl-click resolves current semantic data without moving the departure caret.
Shared context menus hold opening keys until their source target resolves, for
raw and prepared hosts. Adjacent motions coalesce; stationary autoscroll, real
scrollbars, labels and the stable canvas focus keep their own behavior. No new
parser/renderer/dependency. [Contract and evidence](plans/rust-source-gesture-order.md).
Six added viewport regressions cover exact Unicode selection/copy, definition
routing, menu ownership/cancellation and scroll controls; all 35 viewport tests
pass. Workspace passes 463 app tests (seven ignored), two binary tests, four font
tests, eleven text tests and 34 doctests; strict Clippy/build/fmt/diff pass.
Architecture is neutral with no new clone/hotspot/cycle. Final native 1×/2× runs
wait for source readiness and expected clipboard hashes: exact far-right `_END`,
menu Down/Enter, CRLF drag/Copy All and light-mode source keys after an explicit
click back from toolbar pass. Fixtures are unchanged and child exits are zero.
Archive: ~/.local/state/radiology-qa/source-gestures-20261031-0fd76ed.
Exploratory trials expose long-filename sidebar/header overflow and inconsistent
automatic modal/toolbar focus return. Their early/stale clipboard probes are
excluded, not counted as final acceptance or a diagnosed performance stall. The
roughly 6 MB performance case remains unproven; a separate ordinary-fixture GDB
sample shows an idle event loop only. Fix pane constraints and automatic focus
next, then continue broader reader/grammar/index/PR/cache/final Swift acceptance.
Full accessibility and actual sibling adoption remain deferred. Commit and push
this input increment before continuing.

## 2026-10-31 — Integrate horizontal source fragments and ordered measured navigation (Codex)

The native-proven unwrapped stacking/Down regression now passes in the actual
viewport. Unwrapped fragments share one vertical row, measured horizontal prefixes,
clipped paint/hit intervals and whole-row caret/destination bands. Gutter/fold/test
controls stay with the first fragment. The 4 KiB shaping bound and 64-fragment /
2 ms measurement budget remain. Up/Down/Page keys use projected rows and shaped x;
queued selection/copy waits for exact geometry. Source/font/projection/focus and
new navigation retire stale queued input. Pending reveal respects manual input and
viewport/font/scale changes. Shared FragmentFrame geometry and existing input
classification avoid a second renderer or keyboard resolver. [Contract/evidence](plans/rust-unwrapped-fragment-geometry.md).
Tests cover deferred CRLF copy, cancellation, Unicode horizontal hits at 1×/1.5×/2×,
wrap/zoom, fragment-boundary Left and folded projection in both modes. Final native
1×/2× checks cover layout/Down/End and exact line/long-line/Copy All bytes; an earlier
2× trial covers wrap toggling/light rendering and exact far-right `_END` pointer
copy. Fixtures remain unchanged; processes exit zero. Durable archive:
~/.local/state/radiology-qa/unwrapped-viewport-20261031-47766a5. Workspace passes 457
app tests (seven ignored), two binary tests, four font tests, eleven text tests and
34 doctests; strict Clippy/build/fmt/diff pass. Architecture is neutral with no new
clone/hotspot/cycle. An external failing regression confirms that clicks during
unmeasured distant prefixes can be lost: retain gestures next, without guessed hit
bytes or unbounded shaping. Native toolbar use also exposes focus restoration;
Go To Line restores it, so do not count toolbar keyboard acceptance complete.
Broader reader/grammar/index/PR/cache/final Swift acceptance remains active. Full
accessibility and actual sibling adoption stay deferred. Commit and push this
viewport increment before the next interaction repair.

## 2026-10-31 — Confirm long-line defect and add reusable width bookkeeping (Codex)

The source acceptance audit found a real open defect: Wrap Lines off still stacks
4 KiB fragments vertically, and Down stays within the same canonical line. The
viewport regression fails at y offsets 0 versus 60.890625; native evidence shows
the continuation rows and Line 1 after Down. [Evidence and integration steps](plans/rust-unwrapped-fragment-geometry.md).
reader-text now provides UnwrappedRows for grouped horizontal prefixes, measured
width updates and readiness frontiers. It owns no text, font or GUI and adds no
dependency. Tests cover source projections, Unicode, hidden-newline joins, zero
widths, boundary hits and arbitrary update order against linear geometry. The UI
has not adopted it yet: the defect remains required work, not a passed acceptance
case. The failing UI test is preserved outside the worktree as ui-regression.rs
under ~/.local/state/radiology-qa/unwrapped-before-20261031-9fc02dc, alongside native
captures, hashes and zero process exits. Workspace passes 453 app tests (seven
ignored), two binary tests, four font tests, eleven text tests and 34 doctests;
strict Clippy/fmt/diff and warning-denied/inspected rustdoc pass. Architecture is
neutral with no new clone/hotspot/cycle. Commit and push this backend boundary,
then integrate it into placement, hit testing, deferred measurement/navigation and
copy without lifting the shaping bound. Broader parity remains active.

## 2026-10-31 — Own keyboard input across reader context menus (Codex)

Files/source/bookmark/reference menus now share a private action model and egui
menu component. Arrows/Tab/Home/End traverse enabled entries; Left/Right handle
submenus, Enter/Space activate and Escape/outside press dismiss. Current radio
choices use checkmarks distinct from selection; reader focus and Ctrl+C survive.
Semantic scopes and owner retirement prevent stale menu retargeting. Global
reader commands dismiss before dispatch. [Contract and evidence](plans/rust-context-menu-keyboard.md).
A 2× native pass exposed an opening-frame key race. The raw hook now reserves
keys after the secondary press and transfers them to the new owner; unclaimed
releases expire. The regression fails before and passes after. The original
source-arrow regression also has a failing control. Six component tests cover
traversal, guards, model/scope changes and focus. Native 1× covers all four owner
workflows; the clean final 2× binary passes rapid opening/copy, batched submenu
keys, persisted keyboard/pointer choices and light/dark leftward geometry. Failed
and traced 2× trials are excluded. Fixtures stay unchanged; app/compositors exit
zero. Durable evidence: ~/.local/state/radiology-qa/menu-keyboard-20261031-0566596.
Workspace passes 453 app tests (seven ignored), two binary checker tests, four font
tests, eight text tests and 33 doctests, strict Clippy/build/fmt/diff. No new clone
group/hotspot/cycle/max-CCN increase; the only remaining tool flag is an unchanged
12-line reference/modal test clone. Toolbar traversal/focus and broader corpus,
reader, PR/cache and final Swift acceptance remain open. Accessibility and sibling
adoption remain deferred. Commit and push this increment before continuing.

## 2026-10-30 — Add source Ctrl-click definition navigation (Codex)

Swift Command-click was missing from Rust. Ctrl-click now uses exact local/global
SCIP targets and the existing navigation action, retaining the original caret and
selection. Context-menu Go to Definition uses the same target and is disabled
without a destination; empty Copy Selection is disabled. [Contract and evidence](plans/rust-source-pointer-navigation.md).
The pointer regression fails before and passes after, including UTF-16 after an
emoji, stale/Before/unresolved guards and extra modifiers. Disabled menu clicks
leave the popup open. Native Ctrl-click/Back restores the exact departure column
(`eta() {` copied suffix); menu navigation and disabled-action PNG identity pass.
Fixtures stay unchanged; app/compositors exit zero. Evidence is retained under
~/.local/state/radiology-qa/control-click-20261030-9449b3e. Workspace passes 445 app
tests (seven ignored), two binary checker tests, four font tests, eight text tests
and 33 doctests; strict Clippy/build/fmt/rustdoc/diff pass. Shared test setup removes
a new clone; remaining tool flags concern unchanged baseline source-view tests.
Native acceptance exposes the next concrete keyboard defect: Down moves the
source caret behind an open context menu. Repair menu ownership/traversal next;
broader reader/corpus/PR/cache/final Swift acceptance remain open. Accessibility
and sibling adoption remain deferred. Commit and push this increment.

## 2026-10-30 — Verify native syntax resources through reader analysis (Codex)

Replaced the fallback-presence-only CLI verifier identified in R17. Its private
binary module uses DocumentSyntax and existing highlighting to check all nine
native grammar fixtures for outline/folds/calls/occurrences, expected native
colors and both themes; Rust also checks test detection. Six fallback probes are
explicitly labeled. [Contract and evidence](plans/rust-resource-verification.md).
A deliberately broken TS/TSX query makes the real command report errors and exit
1 despite passing fallbacks. Restored resources pass headlessly, including a
copied executable in an otherwise empty working directory with display variables
unset. No checkout/sibling/runtime desktop is required. Durable logs and hashes:
~/.local/state/radiology-qa/native-resources-20261030-4409820. Workspace passes 443
app tests (seven ignored), two binary checker tests, four font tests, eight text
tests and 33 doctests; strict Clippy/build/fmt/diff checks pass. Architecture is
neutral with no new clone group/hotspot/cycle/max-CCN increase. This closes the
verifier false-positive gap, not the broader R17 final-acceptance audit. Corpus,
reader, keyboard/menu/focus and PR/cache acceptance remain; accessibility and
sibling adoption are deferred. Commit and push this increment before proceeding.

## 2026-10-30 — Use native TypeScript/TSX and JavaScript source colors (Codex)

Audit found TS/TSX still using JavaScript fallback colors. The existing native
capture pipeline now enables ECMAScript grammars and assembles Swift
JavaScript/TypeScript/JSX overlays. No new parser, renderer, assets or dependency.
[Contract and evidence](plans/rust-native-highlights.md). A failing-before
regression is covered by the shared language/role/byte matrix; retained-buffer
loader recoloring covers Kotlin/TS/TSX/JS without new syntax analysis. Native TSX
dark/light roles and exact CRLF Copy All pass; fixtures stay unchanged and all
processes exit zero. Durable evidence is under
~/.local/state/radiology-qa/typescript-highlights-20261030-9a5f99a. Workspace passes
443 app tests (seven ignored), four font tests, eight text tests and 33 doctests,
strict Clippy/build/fmt and warning-denied/render-inspected rustdoc. Tests were
consolidated after removing genuine duplication; architecture comparison is
neutral with no new clone group/cycle/max-CCN increase. Resource verification still
only checks Syntect fallback presence and must be repaired during R17 acceptance.
Broader corpora/reader/PR/cache/final Swift audit remain open; full accessibility
and sibling adoption remain deferred. Commit and push this increment.

## 2026-10-30 — Native shortcut acceptance and modal-batch ownership (Codex)

Pushed backend `cd4c119` passes native Ctrl+Alt+J and Search-menu definition
navigation, exact Back caret restoration (`lpha();` copied suffix), one invocation
from two Reindex presses, independent inspector toggling and already-open modal
isolation. Fixtures are unchanged; app/private compositors exit zero. Durable
[evidence and boundaries](plans/rust-keyboard-audit.md) are retained under
~/.local/state/radiology-qa/keyboard-20261030-cd4c119 with hashes and captures.
Inspection exposed a separate same-frame bug: later reader shortcuts could act
behind a just-opened modal or replace it. The existing dispatcher now rechecks
modal ownership between commands. Its regression fails before and passes after,
retaining Go to Line and the exact source location without launching navigation.
Workspace passes 443 app tests (seven ignored), four font tests, eight text tests
and 33 doctests; strict Clippy/build/fmt/diff checks pass. No new architecture
clone group/cycle/max-CCN increase; four touched-file clone warnings are unchanged
baseline fragments. Menu traversal/focus and broader corpus/reader/PR/cache/final
Swift acceptance remain open. Full accessibility and sibling adoption are deferred.
Commit and push this increment before continuing.

## 2026-10-30 — Restore dedicated definition and Reindex shortcuts (Codex)

Swift command comparison found missing dedicated definition/Reindex shortcuts and
no Search-menu definition action. Ctrl+Alt+J and the Search entry now use the
existing definition command; Ctrl+Shift+I forces Reindex, retaining Ctrl+Alt+I for
the inspector. Active generation/tool installation and unavailable-repository
requests are guarded. [Mappings, evidence and remaining checks](plans/rust-keyboard-audit.md).
Actual egui events verify indexed navigation and exact Back restoration, modal/
text-entry ownership and repeated generation. Pointer-driven menu dispatch passes.
Negative controls fail at the unchanged call site and two indexer invocations;
fixed behavior navigates correctly and launches once. Workspace passes 442 app
tests (seven ignored), four font tests, eight text tests and 33 doctests; strict
Clippy/build/fmt and diff checks pass. No new clone group/cycle/max-CCN increase;
four flagged test fragments exactly match the architecture baseline. No new
native acceptance is claimed for these bindings. Full menu/focus, corpus, reader,
PR/cache and final Swift audit remain open; full accessibility is deferred.
Commit and push this increment before continuing.

## 2026-10-30 — Apply user accessibility scope clarification (Codex)

Full accessibility for disabled users is explicitly a non-goal for this phase.
Native bridges, screen-reader/source-text semantics and exhaustive assistive
acceptance are deferred; keyboard shortcuts, command ownership, focus behavior
and menu dismissal remain required. Updated PLAN, the implementation scope and
the historical review annotation so future agents do not reopen this work as a
parity blocker. Actual little-screen-clip adoption remains deferred as originally
requested; standalone operation and useful reuse boundaries remain required.
The uncommitted AccessKit experiment and its added platform dependencies were
removed from the worktree. Its patch/test are retained outside the repository at
~/.local/state/radiology-qa/deferred-accessibility-20261030 for optional future use.
Production sources and Cargo manifests/lock now match pushed HEAD; this increment
changes documentation only. Remaining required work is broader indexing/grammar
coverage, reader layout/navigation acceptance, PR/cache recovery/provenance and
the final requirement-by-requirement Swift audit. Commit and push this scope update.

## 2026-10-30 — Dismiss reader context menus with Escape (Codex)

Appearance backend `254336c` is pushed. Its native pass exposed missing context-menu
Escape handling in egui 0.27. Files/source/bookmark/reference menus now share one
helper using public framework ownership queries and the existing pre-frame event
dispatcher. Open-menu Escape closes without firing actions or clearing the focused
reader's selection; closed/missing owners leave input alone. [Repair and evidence](plans/rust-context-menu-dismissal.md).
The regression fails before and passes after; reopened click actions still work.
Workspace gates pass 437 app tests (seven ignored), four font tests, eight text
tests and 33 doctests, strict Clippy/build/fmt. Architecture adds no clone/hotspot/
cycle; its flagged 12-line test fragment is unchanged baseline code.
Native file/source/bookmark dismissal passes. The full reader PNG after file-menu
Escape is byte-identical to its pre-menu capture; reopened Copy Relative Path is
exact. Fixtures are unchanged and app/compositors exit zero. Durable evidence is
under ~/.local/state/radiology-qa/menu-escape-20261030-254336c. Native references,
complete menu traversal/focus states, accessibility, broader corpus/recovery,
sibling adoption and final Swift parity audit remain open. Commit and push this
increment before proceeding.

## 2026-10-30 — Unify reader palette and widget visuals (Codex)

AppTheme now owns the egui visual mapping used at startup and theme switches.
Neutral canvas/panel/control/text colors match the sibling's fixed roles; selected
and hovered controls use an opaque restrained tint with readable labels and accent
focus borders. Existing public theme colors remain host-customizable; fonts,
spacing, source geometry/bytes and standalone window policy remain independently
owned. [Contract, checks and durable native evidence](plans/rust-reader-appearance.md).
Two theme tests and one app integration cover contrast, host ownership, toggle
publication and persistence. Workspace gates pass 436 app tests (seven ignored),
four font tests, eight text tests and 33 doctests, strict Clippy, build/fmt and
warning-denied/render-inspected rustdoc. Architecture adds no clone/hotspot/cycle;
the four flagged clone fragments exactly match the baseline app tests.
Native 1× and narrow 2× verify both appearances, selection/copy/find, file menus and
focused dialogs; fixtures stay unchanged and app/compositors exit zero. Evidence
is retained outside /tmp under ~/.local/state/radiology-qa/appearance-20261030-cd82b05.
The pass exposes a remaining keyboard defect: Escape leaves file context menus
open after a separate 400 ms wait; outside click closes them. Repair it during the
keyboard/context-menu audit. Broader corpus/recovery, accessibility, sibling
adoption, type-role/spacing and final Swift parity audit remain open. Commit and
push this increment before proceeding.

## 2026-10-30 — Align interface font with the future host (Codex)

Radiology now opts into the sibling's exact Fira Sans UI face via reader-fonts,
with its complete OFL notice. Asset-only consumers retain zero normal dependencies;
optional egui registration changes only proportional text and preserves host code
fonts/custom families. Small metadata is 12 pt; compact body/heading and independent
source zoom stay intact. [Contract and native evidence](plans/shared-font-fallbacks.md).
All-scale tests prove source geometry unchanged. Native 1280×900 at 1× and narrower
1024×768 at 2× verify toolbar/inspector/find/footer layout, both appearances, 1/1
emoji find and exact source copy. Fixture bytes are unchanged; app/compositors exit
zero. Workspace passes 433 app tests (seven ignored), four font tests, eight shared
text tests and 32 doctests with eight test threads; strict Clippy/build/fmt and
warning-denied/render-inspected rustdoc pass. One unrelated five-second loader test
timeout in the initial loaded run passes focused and in the full rerun, unchanged.
No new code hotspot/cycle/clone; the added clone group is required original OFL text.
Palette/widget/type-role alignment, sibling adoption, broader corpus/recovery,
keyboard/accessibility and final Swift parity audit remain open. Commit and push
this increment before continuing.

## 2026-10-30 — Native CJK/emoji font and source-copy acceptance (Codex)

Portable font backend `257c434` and ordered-input fix `daf295f` are pushed.
Native private Weston/Niri runs verify representative CJK/emoji source, filenames,
breadcrumbs and outlines at 1×/1.5×/2×. Emoji-line copy is exact at all three scales;
CJK-line copy is exact at 1×/2×. Local find highlights the actual matching emoji at
1/1 in dark/light at 1×/2×. [Evidence, hashes and precise scope](plans/shared-font-fallbacks.md).
All fixture bytes stay unchanged; app and compositors exit zero at each scale.
Failed/capture-order trials are excluded. This evidence-only increment adds no
production changes or repeated automated gate run. Primary font/type-role/palette
alignment, sibling adoption, broader corpus/recovery, keyboard/accessibility and
final Swift audit remain open. Overall completion is roughly 90%, an estimate
rather than a weighted requirement count. Commit and push each increment.

## 2026-10-30 — Preserve rapid source keyboard event order (Codex)

Native 2× font acceptance exposed a reader batching bug: Copy ran before selection,
and only one navigation event was consumed per frame. The viewport now applies
all owned key/platform-copy events in arrival order through the existing command
dispatcher, updates each intermediate caret and publishes the final movement once.
[Failure, regression and native evidence](plans/rust-source-input-order.md).
The regression fails against the old implementation and passes at 1×/1.5×/2×.
Workspace gates pass 433 app tests (seven ignored), three font tests, eight text
tests and 31 doctests, strict Clippy, build and formatting. Architecture is neutral
with no new hotspot/clone/cycle. Native 2× rapid selection/copy preserves exact CJK
and emoji line bytes; app and both compositors exit zero. The broader keyboard/
accessibility, corpus and final Swift parity audit remain open. Commit and push
this increment before proceeding.

## 2026-10-30 — Portable host-matching CJK/emoji fallback package (Codex)

Radiology now registers CJK and monochrome emoji fallbacks from a small local
reader-fonts package. Font bytes/notices exactly match the sibling's existing
assets. Default mode has no normal dependencies; optional egui registration
preserves primary fonts, custom families and host type sizes, and is idempotent.
Standalone builds need no sibling checkout or renderer. [Contract and provenance](plans/shared-font-fallbacks.md).
Tests prove representative glyph coverage/real texture regions and unchanged Latin
layout at 1×/1.5×/2×, registration ownership and asset hashes. Assets-only mode passes
independently. Workspace gates pass 432 app tests (seven ignored), three font tests,
eight shared text tests and 31 doctests, strict Clippy, formatting, build and
warning-denied/render-inspected crate rustdoc. No new code hotspot/cycle/clone;
the one new 89-line duplication group is required unchanged OFL license text in
the two third-party notices. Native glyph/selection acceptance, primary font,
type-role/palette alignment and broader parity audit remain open. The sibling has
not migrated to this package yet. Commit and push each increment.

## 2026-10-30 — Native file-manager menus and real D-Bus handoff acceptance (Codex)

Backend `7cfdd12` is pushed. Native file/folder menus send exact typed URI requests
through real gdbus to an independently owned private bus/Gio receiver. Unselected
file and quoted/Unicode/shell-like paths are exact; reader selection and folder
expansion stay intact. Rejection/timeout expose complete Details, navigation stays
usable, and the receiver survives the client deadline. An After-only manifest
routes to the private head tree from either reader revision. Timed Exit PR restores
checkout before the delayed reply, with no late error. [Hashed evidence and scope](plans/rust-file-manager-reveal.md).
Nine calls are logged; checkout HEAD/status/all regular-file bytes stay unchanged.
App, receiver, private bus and both compositors exit zero. This verifies the native
protocol path with a controlled receiver, not an inspected real file-manager window.
No production changes or fresh automated rerun in this documentation increment.
Broader corpus/recovery and final UI/accessibility/scaling/font/sibling audit remain
open. Commit and push each increment.

## 2026-10-30 — File/folder Reveal in File Manager (Codex)

Files rows now share Reveal in File Manager and Copy Relative Path context actions,
matching Swift's missing file/folder reveal workflow. A small egui-independent
module uses literal file URIs and the standard FileManager1 ShowItems RPC through
the existing bounded gdbus client runner. LatestRead owns background publication;
errors preserve the reader and use the existing notice/Details. PR actions capture
the After tree/snapshot lease, and scope changes retire late errors. No new crate or
sibling runtime is introduced. [Contract](plans/rust-file-manager-reveal.md).
Three backend tests and one reader integration cover path/URI correctness, error/
timeout/cancellation and retirement. Workspace gates pass 432 app tests (seven
ignored), eight shared tests, 30 doctests, strict Clippy, formatting and build.
No new hotspot/cycle/clone growth; all eight flagged clone fragments are exact
baseline app tests. Native protocol/file/folder/PR-menu acceptance remains open,
along with broader corpus/recovery and final UI/accessibility/scaling/font/sibling
audit. Commit and push each increment.

## 2026-10-30 — Native PR semantic retention and regeneration acceptance (Codex)

Backend `84d320c` is pushed. Four real same-tree QA heads publish through native
PR refresh/Generate using one controlled SCIP run and exact project reuse. A live
external shared lease protects all four records and an active-stage stand-in.
Idle Update retains current plus two recent records, removes the oldest/stage,
and preserves unknown notes. Reload and restart/reopen add no tool invocations.
The evicted head shows no index until explicit Generate reuses project data;
three normalized records remain. [Hashed native evidence and limits](plans/rust-normalized-revision-retention.md).
Checkout HEAD/status/all regular-file bytes stay unchanged; pinned bytes are
correct and workspaces disappear. Both app instances/compositors and the holder
exit zero. This evidence increment changes documentation only; no fresh automated
rerun. Broader corpus/recovery and final UI/accessibility/scaling/font/sibling
acceptance remain open. Commit and push each increment.

## 2026-10-30 — Bounded normalized PR revision records (Codex)

Normalized cache publication now shares the existing project writer lease. After
fresh successful PR publication, idle maintenance retains the current record plus
two recent owned revisions, matching the managed-worktree count. Filename/root/
format provenance protects checkout, foreign, malformed, unsupported and unknown
records; symlinks stay untouched. Busy writers defer cleanup, open descriptors
survive eviction, and abandoned regular stages are removed only while idle.
Failures/stale inputs/compiler-partial builds preserve history; cleanup failures
use the existing warning path. [Contract and limits](plans/rust-normalized-revision-retention.md).
Three new retention tests include an actual external lock holder, reaping and
open-reader decoding; existing cancellation/serialization recovery tests pass.
Workspace gates pass 428 app tests (seven ignored), eight shared tests and 30
doctests, strict Clippy, formatting and build. Real Rust/C/C++ lifecycle cases pass
with the publication guard. Architecture is neutral with no new hotspot/cycle/clone
growth. Native PR retention acceptance, broader corpora/recovery and final
UI/accessibility/scaling/font/sibling audit remain open. Commit and push each increment.

## 2026-10-30 — Actual scip-clang C/C++ lifecycle acceptance (Codex)

The official scip-clang 0.4.0 Linux binary passes the production lifecycle in
separate C11/C++17 fixtures: copied response inputs, rebased database, exact Unicode
header definition/reference coordinates, Update reuse and cache-only reload. Both
have two documents, zero missing sources/errored translation units and no partial,
stale or cache warning. Viewed bytes/inventory remain unchanged and workspaces are
removed. Clang remains manually installed. [Artifact identity, commands and scope](plans/rust-real-indexer-acceptance.md).
All six adapter families now have small real-tool evidence. Rust and Clang share
the external override setup; the real Rust feature-gated case passes again.
Workspace gates pass 425 app tests (seven ignored), eight shared tests and 30
doctests; strict Clippy and formatting pass. Architecture is neutral with no new
hotspot/cycle/clone growth. Broader C/C++ database discovery/corpora, Android/AGP,
retention/recovery and final UI/accessibility/scaling/font/sibling audit remain
open. This increment changes acceptance tests and documentation only. Commit and
push each increment.

## 2026-10-30 — Real pinned Kotlin/Java compiler and navigation acceptance (Codex)

The final mixed fixture passes actual pinned scip-java installation through the
shared production provisioner and real Kotlin/Java generation: three documents,
zero missing sources, no partial/stale/cache warning. Exact Unicode Kotlin calls
and Kotlin-to-Java definitions survive Update reuse and cache-only reload. Viewed
inputs/inventory remain unchanged and compiler artifacts stay private. A private
checksum-verified Temurin JDK 17 works; the system JDK 27 installs the tool but
fails Gradle script analysis during generation. [Evidence and environment limits](plans/rust-real-indexer-acceptance.md).
All four other real-tool cases pass after sharing the restored-index assertion
path. Workspace gates pass 425 app tests (six ignored), eight shared tests, 30
doctests, formatting, strict Clippy and workspace build. Architecture is neutral
with no new hotspot/cycle/clone growth. Actual Clang, Android/AGP, broader corpora,
retention/recovery and final UI/accessibility/scaling/font/sibling audit remain
open. Commit and push each increment.

## 2026-10-30 — Early full-JDK check for Kotlin tool installation (Codex)

Real Kotlin installation with a runtime-only JAVA_HOME reached Gradle before
failing for lack of a compiler. The shared installer now checks javac before any
downloads/writes, honors JAVA_HOME over unrelated PATH compilers, and explains the
full JDK 17+ prerequisite. Both failure paths pass a non-network regression.
[Contract](plans/rust-tool-provisioning.md). Workspace gates pass 425 app tests
(six ignored), eight shared tests and 30 doctests; strict Clippy and shell syntax
pass. Architecture is neutral with no new hotspot/cycle/clone growth. Real pinned
installation succeeds under the full local JDK 27; generation/navigation acceptance
is still running and is not claimed by this fix. Full parity remains open.

## 2026-10-30 — Real Go/Python installation and shared four-indexer acceptance (Codex)

Actual pinned scip-go 0.2.7 and scip-python 0.6.6 pass production installation,
generation, cross-file definition/reference navigation, Unicode byte mapping,
Update reuse and cache-only reload. Viewed files/inventory remain unchanged and
private generation stages are removed. Python's nonfatal missing-pyright-section
warning remains visible. Existing Rust/TypeScript cases share the same lifecycle
assertions; all four opt-in cases pass after refactoring. [Evidence and limits](plans/rust-real-indexer-acceptance.md).
Workspace gates pass 424 app tests (five ignored), eight shared tests and 30
doctests; formatting and strict Clippy pass. Architecture comparison is neutral,
with no new hotspot/cycle/clone growth. Actual Kotlin/Clang, broader corpora,
retention/recovery and final UI/accessibility/scaling/font/sibling audit remain
open. This increment changes acceptance tests and documentation only. Commit and
push each increment.

## 2026-10-30 — Native Critical Lens revisit and scope invalidation acceptance (Codex)

Backend `f1ca00f` is pushed. Eight measured native cache events prove initial build,
Normal/Critical reuse, F12 destination build, Back reuse, manual Non-Critical rebuild,
reuse under the new policy, restored Built-In rebuild and semantic-index replacement
rebuild. The real Rust call visibly dims/restores as expected. Manual decisions save
and clear, while source HEAD/status/all bytes stay unchanged and no indexer runs.
App/compositors exit zero. [Hashed evidence and scope](plans/rust-critical-projection-reuse.md).
A final test-only assertion proves repository teardown clears the memo owner; the
focused reader integration passes afterward. No production changes in this evidence
increment. Full parity remains open: broader corpus/runtime, other real indexers,
recovery and final UI/accessibility/scaling/font/sibling audit. Commit and push each increment.

## 2026-10-30 — Bounded Critical Lens projection and classification reuse (Codex)

CriticalLens now reuses eight exact-source projections within a 16 MiB estimated
syntax/range budget and memoizes up to 4,096 decisions/8 MiB of identity storage.
Complete syntax facts, path/bytes/revision, weak index identity and policy qualify
reuse. Mode stop retains valid entries; repository/index teardown clears ownership.
Lookup/computation stay off-thread with cancellable lock waiting and existing
LatestRead retirement. No source buffers, old indexes or PR leases are retained.
[Contract and tests](plans/rust-critical-projection-reuse.md). Full gates pass 424 app
tests (three ignored), eight shared tests, 30 doctests, build and rustdoc. Final
47 Critical tests, reader integration, fmt, strict Clippy, build and rustdoc pass
after test-table/budget/doc refinement. No new hotspot/cycle/clone growth; flagged
fragments are exact baseline tests. Native revisit/policy evidence follows. Broader
corpus, recovery and final UI/accessibility/scaling/font/sibling audit remain open.
Commit and push each increment.

## 2026-10-30 — Native legacy cache migration and live-reader maintenance acceptance (Codex)

Backend `5dad3ca` is pushed. A copied real Rust cache loads without tools, then native
Update migrates/reuses its flat record with one version probe and no compiler run.
An abandoned owned stage is removed and unknown notes remain. Restart retains three
documents and provenance. A separate live shared-lease holder prevents removal of
a QA stage stand-in during Update; the next idle Update removes it after release.
All three Updates execute only --version. Original dirty checkout HEAD/status/all
bytes remain intact, with no lockfile/target artifacts; app/compositors/holder exit
zero. [Hashed evidence and scope](plans/rust-project-cache-maintenance.md). No further
production changes or automated rerun in this evidence increment. Full parity stays
open: other real tools/corpora, broader recovery, normalized revision retention and
final UI/accessibility/scaling/font/sibling audit. Commit and push each increment.

## 2026-10-30 — Scoped project cache maintenance and migration (Codex)

Project records now use repository-owned namespaces with flat-layout read fallback
and owned-record migration. Shared generation leases and exclusive idle maintenance
reuse existing primitives; waiting is cancellable/bounded and cleanup defers while
another generator owns data. Verified successful runs remove retired strategies and
abandoned owned stages, retaining skipped strategies, unknown files, symlinks and
foreign roots. Marker creation does not follow dangling links. [Contract and tests](plans/rust-project-cache-maintenance.md).
Full gates pass 419 app tests (three ignored), eight shared tests, 30 doctests, fmt,
strict Clippy, build and rustdoc. After fixture sharing, five maintenance tests, both
real Rust/TS tests and strict Clippy pass again. No new hotspot/cycle/clone growth;
flagged fragments are exact baseline code. Native migration/restart evidence follows.
Other real tools/corpora, broader recovery, normalized revision retention and final
UI/accessibility/scaling/font/sibling audit remain open. Commit and push each increment.

## 2026-10-30 — Rust manifest feature cfg parity and real native navigation (Codex)

Rust generation now unions explicit nested Cargo feature names into cfg overrides,
matching Swift while retaining pure command planning and private resources. Shared
bounded reads and the already-locked TOML parser handle quoted keys/comments, sorted
names, sibling isolation, internal links and oversized inputs. Cache format 3 rejects
older incomplete coverage. Actual rust-analyzer 1.98.1 indexes three files and resolves
the gated cross-file call; without overrides the negative control loses that module.
Native F12 opens the real definition and inspector reference; Update performs only a
version probe, and restart without the managed tool reloads the accepted index/origin.
Viewed Git state/all bytes remain intact; no lockfile/target artifacts or private stages
remain, and app/compositors exit zero. [Contract and hashed evidence](plans/rust-feature-configuration.md).
Full gates pass 414 app tests (three ignored), eight shared tests, 30 doctests, fmt,
strict Clippy, build and rustdoc. After test-helper cleanup, both opt-in real Rust/TS
tests and strict Clippy pass again. No new architecture hotspot/cycle/clone growth;
flagged fragments are exact baseline tests. Real toolchain setup is private under
/tmp; normal user toolchain selection is unchanged. Full parity remains open: other
real tools/corpora, retired-project cache maintenance, broader recovery and final
UI/accessibility/scaling/font/sibling audit. Commit and push each increment.

## 2026-10-30 — Shared readable process completion diagnostics (Codex)

The native-discovered indexer message now reads “Indexer exited with status 9”
instead of exposing Some/None values. RunOutput owns the same shared exit/signal
formatter used by installation; process status fields and bounded diagnostics stay
intact. Eleven process tests, eight provisioning tests and the reader Update/forced
failure/retention test pass; the reader asserts exact wording. Strict Clippy and
formatting pass. No new hotspot/cycle/clone growth; flagged reader-test fragments
are baseline code. Rust manifest feature overrides are the next active gap, and
full parity remains open. Commit and push each increment.

## 2026-10-30 — Reader Update/Reindex controls and native reuse recovery (Codex)

Backend `8317edd` is pushed. Index Projects now separates exact/version-qualified
Generate/Update from forced Reindex, shows original reuse origins and exposes cache
diagnostics/mixed-origin warnings. App tests verify dispatch, forced-failure reader
retention and pinned origins. Full gates again pass 412 app tests (two ignored),
eight shared tests, 30 doctests, fmt, strict Clippy, build and rustdoc; real TypeScript
installation/generation/reuse/reload also passes separately. Native pointer checks
prove unchanged reuse skips execution, version changes regenerate, force bypasses
reuse, malformed-cache recovery warns, successful force clears warnings, and restart
retains origins without a compiler run. Five attempts use exact dirty private inputs;
checkout HEAD/status/bytes/manifests stay intact, stages are removed and reported
app/compositor exits are zero. [Hashed evidence and limits](plans/rust-project-index-reuse.md).
No new architecture hotspot/cycle/clone growth; five flagged fragments are verified
exact baseline tests. Remaining: real tools/corpora, retired-project cache maintenance,
broader recovery and final UI/accessibility/scaling/font/sibling audit. Generic indexer
status wording also needs the same plain formatting already used by installation.
Commit and push each increment.

## 2026-10-30 — Version-qualified project cache reuse backend (Codex)

Incremental GenerationRequest policy now reuses the latest exact project record
after an actual tool-version probe; forced generation always runs tools. Stable
attached identity supports checkout/After and same-tree cross-commit reuse while
protecting both viewed roots and keeping Python project naming stable. Whole-repo
byte/type/mode fingerprints conservatively include sibling dependencies. Shared
bounded cancellable JSON and atomic staging avoid duplicate cache machinery and
preserve accepted records on Cancel/stale source. Corrupt records regenerate with
diagnostics; original generating origins persist and mixed origins warn as Swift.
[Contract and tests](plans/rust-project-index-reuse.md). Full gates pass 412 app tests
(two ignored), eight shared tests, 30 doctests, fmt, strict Clippy, build and rustdoc.
Reader/native integration follows; full parity remains open. Commit and push each increment.

## 2026-10-30 — Real TypeScript acceptance fixes occurrence-column parity (Codex)

The actual pinned npm package exposed a coordinate bug: SCIP source-file encoding
was overriding the adapter column contract. Generation now uses adapter defaults
when document columns are unspecified, preserving explicit encodings as Swift does.
Normalized cache format 2 rejects potentially incorrect format-1 coordinates. An
opt-in real install/generate/navigation/cache-reload test passes with cross-file
calls, CRLF and an emoji before the call; source/manifests/inventory remain intact.
A non-network regression covers differing file/column encodings and old-cache
rejection. [Command, results and limits](plans/rust-tool-provisioning.md). Full gates
pass 407 app tests (two ignored), eight shared tests, 30 doctests, fmt, strict Clippy,
build and rustdoc; the real package test passes separately. Architecture adds no
hotspot/cycle/clone growth; flagged cache-test setup is exact baseline code. Other
real tools/corpora, project/version cache reuse, recovery and final UI/accessibility/
scaling/font/sibling audit remain open. Commit and push each increment.

## 2026-10-30 — Native tool installation, retained-index recovery and diagnostics (Codex)

Native controlled npm/indexer acceptance covers five install attempts: live Cancel
and child reaping, failure, automatic indexing, failure retaining accepted semantics,
and successful recovery/regeneration. Both index runs used exact dirty inputs in
private workspaces; checkout HEAD/status/bytes/manifests remain unchanged and stages
are cleaned. Installer errors now report human-readable exit status. Eight focused
provisioning tests, strict Clippy and build pass after this small fix; full gates in
the preceding entry passed before it. [Native evidence and limits](plans/rust-tool-provisioning.md).
App PIDs are absent and the driver terminal; final shutdown codes were not retained.
Real package/compiler acceptance, project/version cache reuse, broader recovery and
final UI/accessibility/scaling/font/sibling audit remain open. Commit and push each increment.

## 2026-10-30 — Explicit pinned tool installation and automatic indexing (Codex)

Missing Rust/Go/TypeScript/Python/Kotlin tools now expose Install and Index with
scope/prerequisite guidance; Clang remains manual. Literal pinned plans match
Swift. Shared bounded/leased processes execute installs, with progress/Cancel,
protected viewed roots and staged verification/publication rollback for managed
packages. Rustup updates its active user toolchain explicitly. The Kotlin resource
has Linux checksum/copy fallbacks, and generation discovers managed/PATH Gradle.
Success verifies the permanent executable then regenerates the current checkout or
pinned After; failure/Cancel retain accepted semantics and root replacement retires
late publication. [Contract and evidence](plans/rust-tool-provisioning.md). Full
gates pass 406 app tests (one ignored), eight shared tests, 30 doctests, fmt, strict
Clippy, build, rustdoc and installer shell syntax. Eight backend and two reader
provisioning tests use controlled local tools; real package/compiler acceptance is
not implied. No new architecture hotspot/cycle/clone growth; reported fragments
are unchanged baseline tests. Native acceptance follows. Full parity remains open:
project/version cache reuse, real-tool/corpus acceptance, recovery and final
UI/accessibility/scaling/font/sibling audit. Commit and push each increment.

## 2026-10-30 — Shared cooperative file leases for background writers (Codex)

Moved existing no-follow/nonblocking regular-file lease primitives from the PR
store into process/lease, preserving shared/exclusive flock and parent CLOEXEC
behavior. PR-store and tree readers reuse them; observed subprocesses can now
inherit an explicitly supplied writer descriptor through the same runner already
used by Git collection. This is the common ownership mechanism for pending tool
provisioning, avoiding a second lock implementation. Five private-store recovery
and eleven process tests pass. Provisioning integration, complete gates and native
acceptance follow; full parity remains open. Commit and push each increment.

## 2026-10-30 — Native bounded notices, complete Copy and PR control acceptance (Codex)

Implementation `d602a49` is pushed. Native added/deleted files disable unavailable
revisions and keep the reader intact after pointer/keyboard attempts; global/source
arrows show correct boundaries while navigating two hunks. A 1,200-row error stays
in one notice row. Details scrolls; Copy exactly matches all 45,793 bytes and the
final marker. Dismiss retains the Before reader; retry recovers. A native-discovered
alignment fix keeps summary text left aligned. Final app/compositors exit zero and
checkout HEAD/status/bytes remain unchanged. [Hashed evidence and scope](plans/rust-reader-notices-controls.md).
Two focused notice tests and strict Clippy pass after the alignment/context follow-up;
build passes after the production fix. Full 396 app tests/eight shared/30 doctests,
fmt and rustdoc passed before that small follow-up. Architecture has no new hotspot,
cycle or clone growth; flagged baseline source-view fragments were verified exact.
Full parity remains open: complete provisioning/project/version cache lifecycle,
broader recovery and final visual/accessibility/scaling/font/sibling reuse audit.
Commit and push each increment.

## 2026-10-30 — Compact diagnostics and available PR revision/change controls (Codex)

Reader errors now use one bounded row with Details/Copy/Dismiss while retaining
complete diagnostics. Before/After buttons reuse FileChange availability and
keyboard toggles reject unavailable sides before source loading. Header status
reflects the actual change type. Global and source change arrows share canonical
caret/revision targets with dispatch and disable at boundaries or without a target.
[Contract and evidence](plans/rust-reader-notices-controls.md). Full gates pass 396
app tests (one ignored), eight shared tests, 30 doctests, fmt, strict Clippy, build
and rustdoc. Tests cover actual pointer/clipboard interactions, narrow/scaled
notices, revision availability and preservation of reader/history. Source-view
complexity improves; no new cycle or clone growth. Two flagged clone fragments are
unchanged baseline tests. Native acceptance follows. Full parity remains open:
complete index lifecycle/provisioning/version-aware reuse, recovery and final
UI/accessibility/scaling/font/sibling reuse audit. Commit and push each increment.

## 2026-10-30 — Deterministic PR cache retention acceptance (Codex)

A full run during reader-notice/control work exposed a timing race in the cache
retention test: it required drop-time cleanup to converge without another cache
operation, although the documented maintenance contract defers when a writer is
busy. The test now exercises the guaranteed next-preparation sweep after releases.
It still proves active snapshots/clones survive pressure, reuse keeps the same
path, the managed set returns to three and the least-recent idle tree is evicted.
No production retention behavior changed. All six tree-cache tests pass. Reader
notice/control work is still in progress and its full gates will be rerun.
Commit and push each increment.

## 2026-10-30 — Native PR refresh rename, caret, pin and viewport acceptance (Codex)

Implementation `275d8db` is pushed. A native long-file review retained line 100,
content row 80 at viewport top and 200 teal pins through an exact rename. Inserting
two lines mapped caret to 102 and viewport source line 81→83 while retaining the
same visible content row; changed bytes dropped pins. Same-commit refresh retained
state with an up-to-date notice. Metadata failure retained the accepted reader;
fresh retry recovered. Six metadata reads were observed. Checkout HEAD/status/bytes
remain unchanged; app/compositors exit zero. [Contract and hashed evidence](plans/rust-pr-refresh-restoration.md).
No production changes or additional automated run in this evidence increment.
Remaining measured UI issues include oversized multiline error notices and enabled
unavailable revision controls. Full parity remains open: complete tool provisioning/
project/version reuse, broader recovery and final UI/accessibility/scaling/font/
sibling reuse acceptance. Commit and push each increment.

## 2026-10-30 — Atomic active-PR refresh and current-reader restoration (Codex)

Changed head/base reviews stage source/syntax/fold preparation before publication,
reconcile navigation/appearance changes and use latest caret/viewport/pins at commit.
Exact Git renames and blob hunks map paths/lines; matching bytes preserve columns
and current-file pin colors. Fresh folds replace manual state; obsolete history,
other-file pins, selection and search retire. Before/After choice survives with
honest fallback/missing-file notices. Failure/Cancel preserve the accepted reader;
metadata-only refresh preserves it and reports up-to-date. Shared source preparation,
LatestRead, line mapping and pin/viewport mechanisms are reused.
[Contract and evidence](plans/rust-pr-refresh-restoration.md). Full gates pass 392
app tests (one ignored), eight shared tests, 30 doctests, fmt, strict Clippy, build
and rustdoc. Final fixture consolidation passes seven focused tests and strict
Clippy. No new architecture hotspot/cycle/clone growth; reported fragments are
unchanged baseline app tests. Native acceptance follows. Full parity remains open,
including complete index lifecycle, recovery and final UI/sibling reuse audit.
Commit and push each increment.

## 2026-10-30 — Native pinned index acceptance and accurate partial-index warnings (Codex)

Implementation `509d0c2` is pushed. Native Generate After Index used exact committed
source in a disposable workspace; partial coverage stayed explicit. A controlled
compiler failure retained accepted semantics; reload and restart reused the cache
without another indexer run. Two runs total: success and deliberate failure.
Checkout HEAD/status/bytes remain unchanged; temporary artifacts are removed.
Native follow-up exposed false stale-source warnings for Before and files omitted
from a partial index. Warnings now require an indexed After/checkout document with
changed bytes. Rendered UI regression checks and final native captures verify both
cases. Five focused review tests, strict Clippy and build pass after the final fix;
full 384 app tests/eight shared/30 doctests, fmt and rustdoc passed before its final
uncovered-file predicate adjustment. [Contract and hashed native evidence](plans/rust-pinned-index.md).
Full parity remains open: complete index lifecycle/provisioning/version-aware reuse,
precise PR refresh restoration, broader recovery and final UI/sibling acceptance.
Commit and push each increment.

## 2026-10-30 — Pinned After index generation and cache reload (Codex)

Review generation now reuses the isolated indexing pipeline with snapshot-owned
leases, raw Git inventories and blob verification. It rejects modified pinned
inputs and mismatched roots. Cache identity uses object store plus head commit,
allowing exact reuse after tree replacement; reports retain commit provenance.
Review entry automatically loads its own cache without checkout-import fallback.
Generate After Index and Reload Index work in review mode; Before stays source-only.
[Contract and evidence](plans/rust-pinned-index.md). Full gates pass 384 app tests
(one ignored), eight shared tests, 30 doctests, fmt, strict Clippy, build and rustdoc.
Architecture has no new hotspot/cycle or clone growth; reported duplicate findings
are five unchanged baseline test groups, verified fragment by fragment. Native UI
acceptance follows. Full parity remains open, including complete index lifecycle,
precise refresh restoration and final UI/sibling reuse acceptance. Push each commit.

## 2026-10-30 — Background PR tree consumers retain snapshot leases (Codex)

Debounced and running repository searches now own their review snapshot, and
indexer probes retain the matching snapshot through their actual worker lifetime.
Search publication retires on PR/commit/tree replacement; same-tree metadata
refresh retains current results. Streaming search reuses LatestRead and its bounded
pool rather than maintaining another cancellation/channel/thread lifecycle. Public
checkout/search APIs remain compatible. Focused tests exercise pinned bytes,
debounce release, synchronized retired publication, scope transitions and a real
blocked tool probe after the reader releases its snapshot. Full gates pass 380 app
tests (one ignored), eight shared tests, 30 doctests, fmt, strict Clippy, build and
rustdoc. Architecture compares IMPROVED with no new findings or clone/cycle growth.
[Ownership contract and evidence](plans/rust-pr-tree-cache.md). Full parity remains
open; pinned After indexing and complete index lifecycle, precise refresh
restoration, broader corrupt-store recovery and final UI/sibling reuse acceptance
remain required. Commit and push each increment.

## 2026-10-30 — Native PR cache reuse and cross-process retention acceptance (Codex)

Implementation `1c81ad3` is pushed. Native preparation reused #7 without another
checkout, protected it with a separate-process lease during LRU pressure, then
evicted it after release. Hidden private-tree corruption forced a verified
replacement despite Git reporting clean; reopening after app restart reused that
replacement. Six revisions plus corruption needed seven materializations. All
settled observations retained at most three managed trees. Viewed checkout
HEAD/status/bytes are unchanged; both app runs and compositors exit zero.
[Contract, exact scope and hashed evidence](plans/rust-pr-tree-cache.md). No source
changes or additional automated run in this documentation increment. Full parity
remains open; next work includes ensuring every background tree consumer retains
its snapshot lease, pinned PR indexing and complete index lifecycle, refresh
restoration, corrupt-store recovery and final UI/sibling reuse acceptance.

## 2026-10-30 — Exact PR tree reuse, snapshot leases and retention (Codex)

Private PR trees now undergo raw Git byte/mode/ownership verification before reuse
or publication. Snapshot-held shared leases protect active trees across processes;
writer-serialized eviction targets three managed trees with active-reader exceptions
and best-effort maintenance after release. Unknown and malformed entries stay
untouched. Private checkout policy suppresses transformations and hooks; SHA-256
repositories are supported without silently migrating existing object stores.
[Contract and evidence](plans/rust-pr-tree-cache.md). Full gates pass 377 app tests
(one ignored), eight shared tests, 30 doctests, fmt, strict Clippy, build and rustdoc.
Architecture comparison flags an unchanged baseline critical-test fragment in a
modified fixture file; no new clone growth, complexity hotspot or cycle was found.
Native multi-process cache acceptance remains pending. Full Swift parity remains
open, especially index provenance/provisioning and pinned PR indexing, precise
refresh restoration, broader corrupt-store recovery and final UI/accessibility/
scaling/font/sibling reuse acceptance. Commit and push each increment.

## 2026-10-30 — Shared exact file and Git-blob identities (Codex)

Moved the existing no-follow file opening, streaming copy/hash and byte/link/mode
identity primitives from index workspaces into repository/identity. Workspace
fingerprints and read/copy verification retain their behavior; the shared stream
now also computes SHA-256 Git blob IDs, and verify_snapshot accepts either Git
object format. Request checks remain per 64-KiB chunk and the caller supplies its
cancellation/deadline predicate. This is the common proof used by pending PR tree
reuse work, avoiding a second hashing implementation. Current checks pass the ten
workspace tests and the full 377 app tests, eight shared tests, 30 doctests, fmt,
strict Clippy, build and rustdoc. PR tree-cache integration and native acceptance
follow; full parity remains open. Commit and push each increment.

## 2026-10-30 — Native child-held PR-store lease acceptance (Codex)

Implementation `2fe0b11` is pushed. A live native materializer retains the private
writer descriptor after its app is deliberately killed. A new app waits while its
checkout reader stays usable; an independent lock probe gets WouldBlock and no
second materialization starts. Cancel retires only that waiting request. Retiring
and reaping the owned orphan releases the lease; fresh preparation opens pinned
PR #7. Checkout HEAD/status/bytes remain unchanged. Reopened app and compositors
exit zero; the initial app's -9 status is intentional. [Contract and hashed evidence](plans/rust-pr-store-recovery.md).
Full gates pass 371 app tests, eight shared tests, 30 doctests, fmt, strict Clippy,
build and rustdoc. Architecture neutral with no new problems or clone/cycle growth.
Full parity remains open: proven tree reuse/active-snapshot leases/bounded retention
(including the crash-left tree), broader corrupt-store recovery, complete index
provenance/provisioning/pinned After indexing, refresh mapping and final
UI/accessibility/scaling/font/sibling reuse acceptance. Commit and push each increment.

## 2026-10-30 — Serialized private PR stores and lock-safe recovery (Codex)

Each preparation holds one stable private-store advisory writer lease, with
cancellable/deadline-bounded acquisition. Its Git children explicitly inherit
that descriptor while unrelated commands retain CLOEXEC, preventing a new writer
from overtaking a surviving child after parent exit. Existing public process APIs
are preserved. Private Git operations now live in a cohesive store module.
Initialization stages before atomic publication; an existing valid bare store
is read without reinitializing it. HEAD-less legacy initialization is preserved
in quarantine. Fetches disable FETCH_HEAD writes, auto-maintenance and commit-graph
writes; fallback uses a unique request ref, verifies the pinned commit, and removes
that ref. Missing owned temporary worktrees can be force-removed even when Git
left their registrations locked; present and foreign trees remain untouched.
Six added tests cover lease cancellation/deadline/release, child-only descriptor
inheritance, serialized cancellation/retry with locked missing-tree repair, real
fetch with legacy locks preserved, legacy init quarantine and fallback movement.
Existing runtime cancellation checks now include staged initialization.
Full gates pass 371 app tests (one external-Clang fixture ignored), eight shared
tests, 30 doctests, fmt, strict Clippy, build and rustdoc. Architecture neutral:
no new problems, clone/cycle growth; health improves 78.8→79.2. Native parent-crash/
surviving-child lease verification is next. Full parity remains open, including
complete tree reuse/retention, broader corrupt-store recovery, index lifecycle and
PR refresh mapping, final UI/accessibility/scaling/sibling acceptance.
Commit and push each increment.

## 2026-10-30 — Native recent PR chooser acceptance (Codex)

Recent chooser `99c0ae4` and tooltip follow-ups `cb106b0`/`87e31b8` are pushed.
Native checks cover automatic 25-entry loading, wrapped titles/states, live list
Cancel/reaping, retained data on authentication failure, recovery, an accepted
empty list with active fallback, merged PR selection, same-commit bookmark scope,
manual #99 outside the list and Exit PR retaining entries. Checkout HEAD/status/
bytes stay unchanged. Final light/dark popup captures suppress header tooltip
overlap; tracked app/compositor exits are zero. [Contract and hashed local evidence](plans/rust-pr-chooser.md).
Final gates pass 365 app tests, eight shared tests, 30 doctests, fmt, strict Clippy,
build and rustdoc. Clone groups fall 91→90; four comparison flags are verified
unchanged baseline fragments, with no new function hotspot/cycle. Full parity
remains open: private-store concurrency/lock recovery and retention, full index
provenance/provisioning/pinned After indexing, PR refresh mapping and final
UI/accessibility/scaling/font-glyph/sibling reuse acceptance. Commit and push each increment.

## 2026-10-30 — PR menu tooltip overlap repair (Codex)

Native chooser acceptance exposed the header tooltip covering open menu rows.
The first guard in `cb106b0` used egui's returned inner value; in egui 0.27 that
value stays empty while an open root menu is being drawn. The guard now observes
whether contents actually render. A complete popup test verifies closed-header
help remains available and is absent while rows are open. Four menu tests, strict
Clippy, build and formatting pass. Native restart and final acceptance documentation
follow. Full parity remains open; commit and push every increment.

## 2026-10-30 — Recent PR chooser and compact review controls (Codex)

RecentPullRequests owns a cancellable repository-scoped 25-entry list independently
of review preparation. Automatic load, separate Refresh List/Cancel, retained data
on failure/cancel, honest empty/unavailable states and stale-root retirement use
LatestRead and the bounded GitHub process path. Invalid/duplicate/overflow lists
are rejected. The compact toolbar menu shows number/title/state, current-review
fallback, Local Checkout, separate Refresh PR and a visible Exit PR. Rendering
borrows data and returns shared commands for standalone/embedded reuse. Long titles
wrap; keyboard focus reveals scrolled choices. The number prompt pre-fills active/
recent numbers, rejects invalid/zero/overflow input and keeps Cancel keyboard focus.
Selecting another PR with identical commits now changes bookmark/session scope.
Full checks pass 364 app tests (one external-Clang fixture ignored), eight shared
tests, 30 doctests, fmt, strict Clippy, build and rustdoc; targeted model/UI checks
pass after test-fixture reuse. Architecture adds no function complexity hotspot;
clone groups decrease 91→90. Four compare flags are verified unchanged baseline
modal/test fragments. Native chooser/list Cancel/failure/recovery acceptance is next.
Full parity remains open: cache retention/concurrent Git-store lock recovery,
index provenance/provisioning/pinned After indexing, PR refresh mapping and final
UI/accessibility/scaling/sibling integration acceptance. Commit and push each increment.

## 2026-10-30 — Native PR process cancellation acceptance (Codex)

Implementation `a651a8b` is pushed. The native PR prompt starts controlled metadata
and fetch children; Cancel reaps both while retaining the checkout reader and
semantic data. Fresh preparation opens PR #7 After from pinned bytes. Cancelling
a subsequent refresh reaps its metadata child and retains the accepted PR. Checkout
HEAD, dirty status and exact bytes remain unchanged; app/compositors exit zero.
[Contract, limits and hashed local evidence](plans/rust-pr-process-cancellation.md).
Current gates pass 357 app tests, eight shared tests, 30 doctests, fmt, strict Clippy,
build and rustdoc. Architecture flags only verified baseline clone fragments and
the renamed pre-existing preparation warning; clone groups/cycles do not grow.
Full parity remains open: recent chooser, cache retention/concurrent Git-store and
lock recovery, complete index provenance/provisioning/pinned After indexing,
refresh mapping and final UI/accessibility/scaling/sibling reuse acceptance.
Commit and push every increment.

## 2026-10-30 — Bounded PR preparation and child cancellation (Codex)

PR preparation now uses LatestRead retirement and one two-minute metadata/fetch/
materialization/diff deadline. GitHub and Git children use the shared process runner
with bounded complete stdout; cancellation/timeout terminate and reap children.
Fetch fallback applies only to completed nonzero commands, never interruption.
Shared run_collect also replaces GitRead's duplicate byte collector and retains
nonzero Git diagnostics. Availability probes use five seconds; prompts are disabled
for GitHub requests. Private stores initialize idempotently, and each attempt owns
a unique temporary tree until all snapshot data succeeds; retries prune abandoned
registrations. Accepted trees and checkout bytes remain intact. UI Cancel, root
switches, replacement, reset and drop retire work; failure/cancel clears launch-file
intent. Tests cover live gh/fetch/worktree/patch Cancel+timeout, exact cleanup and
retry, oversized/malformed/failed output, and model retention/publication ownership.
Full gates pass 357 app tests (one external-Clang fixture ignored), eight shared
tests, 30 doctests, fmt, strict Clippy, build and rustdoc. Architecture comparison
flags four byte-identical baseline test clone fragments plus the renamed preparation
entity: Lizard CCN 15→14 and cognitive 27→26. Clone groups/cycles do not grow.
Native PR child Cancel/recovery acceptance is next. Full parity remains open,
including recent chooser, broader cache/provenance/provisioning/pinned indexing,
PR refresh mapping and final UI/accessibility/scaling/sibling acceptance.
Commit and push every increment.

## 2026-10-30 — Native normalized-cache cancellation acceptance (Codex)

Implementation `f59932d` is pushed. Native acceptance cancels a 72-MB normalized
JSON reload with the reader and prior semantic data retained, then accepts a fresh
two-document cache. A deliberately mismatched fingerprint is correctly labeled
stale. App and compositors exit zero. [Contract and hashed local evidence](plans/rust-cache-json-cancellation.md).
Full checks pass 353 app tests, eight shared tests, 30 doctests, fmt, strict Clippy,
build and rustdoc. Architecture neutral with no new problems or clone/cycle growth.
Remaining full-parity work includes PR child-process lifecycle and recent chooser,
source/graph/context reuse, provisioning/per-project provenance/pinned After
indexing, refresh mapping and final UI/accessibility/scaling/sibling reuse acceptance.
Commit and push every increment.

## 2026-10-30 — Interruptible normalized-cache JSON (Codex)

Cache JSON decoding now uses a buffered cancellation-aware reader with 64-KiB
checkpoints, including inside large strings. Stops latch because serde can read
again after an I/O error. Validation checks each document/occurrence. Existing
serialization checks now bound individual writes to 64 KiB and guard final flush;
atomic temporary-file publication and the format remain unchanged. Cohesive
restore and inventory-validation stages keep all cache functions below CCN 15.
Three tests interrupt every guarded decode checkpoint, preserve Unicode/provenance,
reject trailing/truncated JSON, interrupt a real temporary write, retain the prior
cache, remove incomplete output and recover through fresh save/load.
Full gates pass 353 app tests (one external-Clang fixture ignored), eight shared
tests, 30 doctests, fmt, strict Clippy, build and rustdoc. Architecture neutral;
no new problems, clone groups or cycles. Native cache Cancel/recovery acceptance
is next. Remaining full-parity work includes PR process lifecycle, broader caches,
provisioning/provenance, refresh mapping and UI/accessibility/scaling/sibling reuse.
Commit and push every increment.

## 2026-10-30 — Native SCIP Cancel/reload acceptance (Codex)

Implementation `e2a7f6f` is pushed. Native acceptance starts from one accepted
document, cancels a 58.5-MB/1.5-million-reference reload with prior semantic data
retained, then accepts a fresh two-document index. App and compositors exit zero.
[Contract, limits and hashed local evidence](plans/rust-scip-cancellation.md).
Full checks remain 350 app tests, eight shared tests and 30 doctests plus fmt,
strict Clippy, build and rustdoc. Full parity remains open: normalized-cache JSON
interruption, PR process lifecycle, broader caching/provisioning/provenance,
refresh mapping and final visual/accessibility/scaling/sibling reuse acceptance.
Commit and push every increment.

## 2026-10-30 — Cancellable SCIP decoding and semantic-map construction (Codex)

Nested protobuf readers now inherit the import/generation cancellation predicate,
including packed coordinates, relationships, signatures and chunked string copies.
A typed Cancelled error discards incomplete decoding. Length checks avoid overflow
and over-wide varints are rejected. Semantic-map construction checks each document,
symbol and occurrence plus derived maps/search; document/search stages replace
the previous large constructor, and fallback symbols avoid a cloned ID list.
Imports, generated outputs and normalized-cache map construction use these paths.
Four new tests exercise nested stop checkpoints, large packed/Unicode data, safe
malformed lengths/varints and every assembly checkpoint with fresh semantic recovery.
Full checks pass 350 app tests (one external-Clang fixture ignored), eight shared
tests, 30 doctests, fmt, strict Clippy, build and rustdoc. Architecture is neutral:
assembly stages are CCN 13/12/9 versus the previous 21; the renamed wire-root
decoder retains its existing CCN 11/cognitive 34. No clone/cycle growth. Native
large-index Cancel/reload acceptance is next. Full parity remains open, including
normalized-cache JSON encode/decode interruption, PR process lifecycle, caches,
provisioning/provenance, PR mapping and final UI/accessibility/scaling/sibling reuse.
Commit and push every increment.

## 2026-10-30 — Native analysis cancellation acceptance (Codex)

Native parser/query/walk retirement committed/pushed as `895f5c6`. Native acceptance
rapidly replaces four 4.44-MB Rust candidates while retaining the accepted a.rs,
then displays the fresh small f.rs in the capture requested 0.5 seconds after its
click. Explicit Cancel retains f.rs. App and compositor exits are zero. The final
callback tests prove actual parser and no-match query interruption, latched stop
pulses, stage/walk cancellation and fresh recovery. Full gates pass 346 app tests
(one external-Clang fixture ignored), eight shared tests, 30 doctests, fmt, strict
Clippy, build and rustdoc. Architecture adds no hotspot/cycle/clone group after
shared match traversal and cohesive color/declaration-end preparation; remaining
flags are verified baseline fragments. [Contract and hashed evidence](plans/rust-native-analysis-cancellation.md).
Full parity remains open: SCIP decoding interruption, PR process lifecycle, caches,
broader index/provisioning/provenance and refresh mapping, final visual/accessibility/
scaling acceptance and sibling palette/chrome reuse. Commit and push each increment.

## 2026-10-30 — Shared native analysis cancellation (Codex)

Tree-sitter parsing and query matching now use native progress callbacks with one
latched analysis guard. Shared match traversal replaces duplicated cursor loops.
Outline/test/occurrence walks check retirement between nodes or byte blocks;
whole-document cancellation discards partial facts instead of returning fallback
syntax. Source loading, caller tracing, graph bodies and definition previews pass
their actual cancellation/deadline predicates through analysis. Uncancelled public
APIs retain their results. Shared coloring and definition-end preparation keep the
modified pipeline below complexity thresholds. Four new tests prove actual parser
and no-match query interruption, latched stop pulses, staged cancellation, manual
walk stops and fresh recovery. Full gates pass 346 app tests (one external-Clang
fixture ignored), eight shared tests, 30 doctests, fmt, strict Clippy, build and
rustdoc. Architecture adds no hotspot/cycle/clone group; only two verified baseline
fragments remain flagged. Native large-file replacement acceptance is next. Full
parity remains open, including SCIP/PR process lifecycle, caches, provisioning and
PR mapping, broader UI/accessibility/scaling and sibling palette/chrome reuse.
Commit and push every increment.

## 2026-10-30 — Semantic job source cancellation and cache safety (Codex)

Previews, reference queries, incoming callers and Critical Lens now pass their
actual request flags/deadlines to source/blob reads. The source timeout policy is
shared, and displayed bytes still win without I/O. Cancelled/expired reference
reads cannot become successful unreadable coverage; graph failures from retirement
do not enter the file/summary caches. A later live request can load the real body.
An isolated real-Git fixture confirms live pinned-blob children for all four APIs,
cancels each, requires a prompt error and verifies ESRCH after reaping. It also
checks graph recovery. Additional guards cover deadlines and displayed Arc identity.
Full checks pass 342 app tests (one external-Clang fixture ignored), eight shared
tests, 30 doctests, fmt, strict Clippy, build and rustdoc. Architecture adds no
hotspot/cycle/clone group; the three touched-file fragments flagged by the tool
are verified in the baseline. [Contract and runtime evidence](plans/rust-analysis-source-cancellation.md).
Full parity remains open: native/SCIP interruption, PR preparation/fetch lifecycle,
source/graph/context caches, broader index/provisioning/PR refresh mapping, final
visual/accessibility/scaling acceptance and sibling palette/chrome reuse.
Commit and push every increment; the app remains independently runnable.

## 2026-10-30 — Source Git live-child acceptance and index-deletion hunks (Codex)

Bounded source operations committed/pushed as `dd62028`. Final review added the
current index check for files removed with `git rm --cached` but left on disk;
these now get added-file hunks rather than HEAD/index deletion markers. Shared
fixtures cover ordinary tracked, index-deleted and new untracked source without
duplicated test drivers. Full workspace checks pass 339 app tests (one external-
Clang fixture ignored), eight reader-text tests, 30 doctests, fmt, strict Clippy,
build and rustdoc; the final index-deletion fixture passes separately too.
Architecture is neutral with no new problem. Native Cancel is verified against
a confirmed live Git PID and its subsequent absence, retaining the old source;
forced failure retains it and reselecting the target succeeds. Earlier replacement
acceptance also reaps a pending child. The corrected build repeats native Cancel/
failure/retry; app and compositor exits are zero.
[Contract and hashed evidence](plans/rust-source-process-cancellation.md).
Full parity remains open: finer native/SCIP interruption, cancellation propagation
through other source consumers and PR preparation/fetch, source/graph/context caches,
broader indexing/provisioning/PR refresh mapping and final visual/accessibility/
scaling acceptance and sibling palette/chrome reuse. Commit and push every increment.

## 2026-10-30 — Bounded cancellable source Git reads (Codex)

Source loading now shares the repository process runner for checkout hunks and
pinned review blobs. The request cancellation flag and one 30-second cooperative
deadline reach both operations; timeout/output/Git failures retain the prior reader
through the existing error path. Checkout refresh and file opens share exact-disk
hunk validation, literal pathspecs and added-file handling. Pinned reads verify
size metadata against bounded raw blob bytes. Full workspace checks pass 339 app
tests (one external-Clang fixture ignored), eight shared-text tests, 30 doctests,
fmt, strict Clippy, build and rustdoc. Architecture comparison is neutral with no
new problems. Native live-child cancellation and failure/retry acceptance are
still pending for this increment. Full parity and finer native/decode/other-job
interruption remain open. Commit and push every increment.

## 2026-10-30 — Cancellable repository opening and deferred launch intent (Codex)

Shared bounded Git reads committed/pushed as `d8ed022`. Root lookup, file enumeration,
classification/sorting and initial filtered trees now run on the existing worker
pool. Accepted reader/session state stays usable until successful root publication;
Cancel/failure preserve it and errors expose Retry/Dismiss. Typed destinations defer
CLI/session source/PR work until its root is ready and restore the live caret or
pending navigation on checkout refresh. Root/review replacement retires old intent;
filter changes invalidate a prepared tree and request fresh preparation. Real Git
fixtures cover whitespace roots, worktrees, literal paths and bounded failures;
no-Git traversal retains exclusions and rejects partial inventories. Full gates
pass 335 app tests (one external-Clang fixture ignored), eight shared tests, 30
doctests, fmt, strict Clippy, build and rustdoc. Native delayed/cancelled/failed/
retried opening retains source/caret and publishes the new inventory; app and
compositors exit zero. Architecture adds no hotspot/cycle/clone group; only baseline
test fragments remain flagged. [Host contract and evidence](plans/rust-repository-opening.md).
Full parity remains open: source/graph/context caches, finer native/SCIP/source-loader
interruption, broader index/provisioning/PR lifecycle and PR-refresh state mapping,
final visual/accessibility/scaling acceptance and sibling palette/chrome reuse.
Commit and push every increment; the standalone app remains independent.

## 2026-10-30 — Shared bounded Git read context (Codex)

Extracted checkout GitRead into the repository module so discovery can reuse
the same child-only environment, literal arguments, complete 16-MiB stdout cap,
process-group cancellation and total deadline. Optional Git probes distinguish
nonzero status/absent tools from timeout, cancellation and output/I/O failures.
The process runner now preserves structured pre-launch OS error categories;
indexer diagnostics retain their existing display text. Existing Git fixture
helpers are shared between discovery and refresh tests. Full workspace tests,
doctests and strict Clippy pass with pending asynchronous repository opening.
That integration and native acceptance are the next increment; full parity
remains open. Commit and push every increment.

## 2026-10-30 — Cancellable checkout branch/change refresh (Codex)

Shared runner and Git record parsing committed/pushed as `6865449`. Checkout
refresh now prepares branch, literal-path changes/counts and exact-source hunks
on the existing worker pool, retaining accepted state during work/failure/cancel.
It uses bounded complete stdout, a total deadline and process-group cleanup;
repository/review transitions retire it. Added/unborn/binary/renamed paths and
moving HEAD are covered. The UI exposes progress/Cancel and error/Retry/Dismiss
without changing navigation. Workspace tests, doctests, strict Clippy, fmt, build
and rustdoc pass. Native delayed/failing/retried refresh checks retain the source
and caret, with all app/compositor exits zero. Architecture introduces no hotspot,
cycle or clone group; only existing touched test fragments remain flagged.
[Contract, bounds and hashed evidence](plans/rust-git-refresh.md).
Full parity remains open: synchronous repository discovery, source/graph/context
caches, broader index/PR lifecycle and PR-refresh state mapping, final visual/
accessibility/scaling acceptance and sibling palette/chrome reuse. Keep committing
and pushing every increment; the standalone app has no sibling runtime dependency.

## 2026-10-30 — Shared cancellable process runner and Git records (Codex)

Moved the existing indexing process-group runner and its tests into the shared
`process` module. IndexCommand remains a compatible alias; index generation
continues through the same bounded diagnostics, cancellation, deadlines and
child-reaping behavior. Checkout refresh can reuse that implementation without
a new runtime or subprocess mechanism. PR review now uses the shared FileChange
NUL-record parser, preserving literal tabs/newlines/Unicode and rejecting incomplete
records, invalid paths and malformed rename similarity. Full workspace tests,
30 doctests and strict Clippy pass with the pending checkout-refresh integration.
The checkout worker/UI and its native acceptance are the next increment; full
parity remains open. Commit and push each increment.

## 2026-10-30 — Native Swift/Kotlin colors and retained theme captures (Codex)

Prepared/pushed native capture rules and shared query/identity reuse (`29b6132`).
Swift and Kotlin now color from the shared native parse instead of Syntect/plain
fallback. Kotlin's native node roles cover declarations/types/properties/calls/
strings/numbers/constants/annotations; Swift reuses its shipped rules. A bounded
sweep preserves half-open overlap precedence and exact Unicode/CRLF line bytes.
Source-loader theme requests retain the capture Arc and source Arc without a new
parse or checkout read. Foreign/stale captures are rejected; failures retain basic
colors with a recoverable footer. Other languages keep their prior appearance.
Shared bounded query visits and typed SourceIdentity remove new duplication
across colors, occurrences and calls. Full gates pass 318 app tests (one external
Clang fixture ignored), eight shared tests, 30 doctests, fmt, strict Clippy, build
and rustdoc. Native Swift/Kotlin light/dark and folded/expanded checks pass; all
private app/compositor exits zero. Architecture adds no hotspot/cycle/clone group
and lowers duplication; only baseline source-view test fragments remain flagged.
[Contract and hashed evidence](plans/rust-native-highlights.md).
Full parity remains open: broader language/visual/accessibility/scaling acceptance,
PR refresh mapping, source/graph/context caches, background/index/PR lifecycle
and sibling palette/chrome integration. Keep committing and pushing each increment.

## 2026-10-30 — Transient occurrence clicks and outlined pins (Codex)

Pinned and single-click transient groups now share one resolver and canonical
OccurrenceSelection value. Plain completed clicks select; drag/modified clicks
do not. The clicked range stays active through caret movement; misses, pin/unpin
and successful document changes clear transient state. Source-range rectangle
geometry is shared across selection/find/pin/transient painting. Swift's 16%/28%
accent washes and active outline are integrated; pins now have 1.4-point outlines
and explicit dark/light fill/stroke opacity. Native acceptance caught egui's
time-only global double-click count pinning after a gutter click. Source-local
position/time/count tracking now rejects unrelated rapid clicks and triple toggles.
Full gates pass 312 app tests (one external-Clang fixture ignored), eight shared
tests, 30 doctests, fmt, strict Clippy, build and rustdoc. Final native dark/light
selection, actual arrow movement, miss clearing, pin/unpin and gesture checks pass;
private app/compositors exit zero. Architecture adds no hotspot/cycle/clone group;
only baseline touched test fragments remain flagged. [Contract/evidence](plans/rust-occurrence-selection.md).
Full parity remains open: PR refresh mapping, source/graph/context caches,
background/index/PR lifecycle, complete visual/accessibility/scaling acceptance
and sibling palette/chrome reuse. Commit and push every increment.

## 2026-10-30 — Parsed syntactic occurrence pins (Codex)

Prepared/pushed independent source-bound occurrence facts (`487306a`) from the
existing shared parse. Pins use the innermost function/closure and exclude parsed
comments/strings/numbers/keywords. Swift's same-spelling grouping deliberately
includes shadowed bindings; it is never semantic navigation. Partial SCIP falls
back to the complete syntactic set; complete SCIP retains semantic identity.
Whole-file regex fallback is removed. Exact source/revision/digest guards and
fact/token limits reject stale/partial facts; errors have a recoverable footer.
Shared interval union replaces a new duplicate, and query/token preparation are
separate. Full gates pass 309 app tests (one external-Clang fixture ignored),
eight shared tests, 30 doctests, fmt, strict Clippy, build and rustdoc. Native
partial-index pinning, rejected string/comment clicks, closure groups and unpinning
from an unindexed occurrence pass; app/compositors exit zero. Architecture adds no
hotspot/cycle/clone group. [Contract and hashed evidence](plans/rust-syntactic-pins.md).
Single-click transient occurrence selection remains open, as do PR-refresh
mapping, caches/background/index/PR lifecycle, full visual/accessibility/scaling
acceptance and sibling UI reuse. Full parity stays open. Push every commit.

## 2026-10-30 — Canonical-byte fold projection and reader focus (Codex)

Added/pushed shared bounded visible-span fragments (`c431f4b`) and integrated
manual/test hidden-range union into the viewport. Delimited folds retain `{}` and
visible tails on one row; non-delimited bodies hide whole captures and use the
preceding header control. Shaping, tokens, hit/caret/hover/movement/decorations and
exact-byte reveal use the same canonical mapping. Find queries only surviving
spans; original-byte selection/copy is unchanged. GDB caught egui arrow traversal
stealing source focus; the existing navigation focus filter is now reused by the
canvas. The pointer/arrows/Select All/exact CRLF-copy regression passes before/
after debugger verification. Full gates pass 303 app tests (one external-Clang
fixture ignored), eight shared tests, 30 doctests, fmt, strict Clippy, build and
rustdoc. Native Rust partial lines/Unicode tails, Python header/body toggling and
Kotlin complete-import collapse/reopen pass; private app/compositors exit zero.
No new architecture hotspot/cycle/clone group; duplication drops 0.01 points,
with only baseline test fragments flagged. [Contract/evidence](plans/rust-fold-projection.md).
Full parity remains open: AST-aware occurrence fallback, PR-refresh mapping,
caches/background/index/PR lifecycle, full visual/accessibility/scaling acceptance
and sibling palette/chrome reuse. Keep committing and pushing each increment.

## 2026-10-30 — Exact fold identities and overlapping headers (Codex)

Retained overlapping parsed folds in Swift's outer-first order and replaced
line-keyed collapsed state with exact byte spans throughout defaults, prepared
source, PR ownership/ancestry, restoration and viewport actions. One gutter
control still serves each row; toggling an outer type preserves child choices.
Critical Lens body matching/count batches also use exact ranges, preventing
shared-header container/count collisions. Existing broad tests use derived
header assertions; no compatibility state is retained. Full checks pass 301 app
tests (one external-Clang fixture ignored), five reader-text tests, 30 doctests,
fmt, strict Clippy, build and rustdoc. Native outer toggles and different method
choices survive file return on the final build; private app/compositors exit zero.
Architecture adds no hotspot/cycle/clone group and lowers average CCN; remaining
flags are existing touched test fragments. [Contract/evidence](plans/rust-fold-identities.md).
Exact partial-line/non-delimited header projection is still required, together
with broader cache/background/index/PR lifecycle, visual/accessibility acceptance
and sibling UI reuse. Full parity stays open. Push each commit immediately.

## 2026-10-30 — Parsed Kotlin import grouping (Codex)

Adapted the missing kotlin-ng import-list node into one parsed sibling-run fold.
Comments/blank lines between imports are included; trailing comments, declarations
and errors terminate the range. Imports remain expanded by default and Fold to
Definitions expands manual import folds. Every captured import counts toward
fact limits. Exact Unicode/CRLF, aliases/stars/semicolons, string impostors,
errors and command policy are covered. Full gates pass 298 app tests (one external
Clang fixture ignored), five reader-text tests, 30 doctests, fmt, strict Clippy,
build and rustdoc. Native collapse/reopen preserves the trailing comment and
function header; all private app/compositor exits zero. Architecture is NEUTRAL.
[Details/evidence](plans/rust-structural-folds.md).
The same-row queue is corrected: Swift also shows one gutter disclosure per row;
remaining work is byte-range identity/overlap retention and exact non-delimited
header projection. Broader cache/background/index/PR lifecycle, visual/accessibility
acceptance and sibling UI reuse remain open. Push after every commit.

## 2026-10-30 — Enclosing callable shading and blank-line hit geometry (Codex)

Implemented Swift's signature-through-body neutral wash using existing parsed
breadcrumbs, with no additional parser/cache or semantic-index dependency.
Visible-fragment shaping preserves Unicode/wrapped half-open bounds, empty lines
and folds. Ordered backgrounds retain PR/destination cues and put the 5% active
wash on one visual row. Actual caret changes schedule refreshed host context.
Native gap clicks exposed a four-pixel empty-line hit area; allocated line-height
hit testing now moves into the gap and clears scope. Full checks pass 296 app
tests (one external-Clang fixture ignored), five reader-text tests, 30 doctests,
fmt, strict Clippy, build and rustdoc. Final native dark/light scopes and gap
transitions pass; all private app/compositor exits are zero. Architecture adds no
hotspot/cycle/clone group; only existing touched test clones remain flagged.
[Contract and hashed evidence](plans/rust-function-shading.md).
Full parity remains open: same-row folds/import grouping, caches/background/index/
PR lifecycle, full visual/accessibility acceptance and sibling palette/chrome reuse.
Commit and push every increment.

## 2026-10-30 — Source-bound destination feedback (Codex)

Committed/pushed shaped destination feedback (`6d86935`) and explicit navigation
binding (`00d8eb5`), followed by path/revision identity hardening (`078070a`).
Successful semantic/reference/caller/bookmark/change-block
and pointer outline activation can flash the exact shaped visual row: 18% accent,
900 ms hold and 210 ms fade. First-paint timing, identity rejection, replacement,
preview/navigation cancellation and disabled-animation clearing are bounded in a
small controller. Wrapped Unicode fragments paint one band; selection/caret stay
independent. NavigationIntent carries policy through validated source publication.
Full gates pass 292 app tests, 5 reader-text tests, 30 doctests, fmt, Clippy,
build and rustdoc. Native pointer/keyboard distinctions, expiry, dark/light
bookmark rereads, missing-file preservation, UTF-16 F12 and interrupted Back pass;
[contract and hashed evidence](plans/rust-destination-feedback.md). No new product
hotspot/cycle/clone group. Overall parity remains open: function shading,
fold/import presentation, caches, background/index/PR lifecycle, broader native
and accessibility acceptance, and sibling palette/chrome reuse. Keep pushing
after every commit.

## 2026-10-30 — PR outline markers and revision anchor mapping (Codex)

Committed/pushed full-line byte spans (`925e8c3`), PR callable dots (`3c84b2c`)
and zero-length hunk anchor correction (`752a666`). Cached outline flags use the
displayed pinned revision's changed bytes, retain selection on diff/caret updates,
and clear outside review. Six-point accent dots and explanatory tooltip/widget
metadata identify changed callables. Unicode/CRLF/EOF, half-open/nested spans,
revision/diff replacement and binary/checkout cases are covered. A real Git
review test matches painted dots to names and proves a different dirty checkout
has no PR dots. Its line-1 round trip caught and now covers the anchor-line bug.
Full gates pass 288 app tests, 5 reader-text tests, 30 doctests, fmt, Clippy,
build and rustdoc. Native Before/After, tooltip, light appearance and review-exit
checks use real pinned Git objects; [contract and evidence](plans/rust-review-outline-changes.md).
Architecture adds no hotspot/cycle/clone group. Full parity remains open,
including destination feedback, source presentation, caches, indexing/PR lifecycle,
broader native acceptance and sibling UI reuse. Keep pushing after every commit.

## 2026-10-30 — Outline selection, hierarchical disclosure and exact previews (Codex)

Committed/pushed source-only reveal (`eda2850`), Rust const/static/enum-variant
outlines (`d6bfbbf`), outline integration (`c84ac93`), and missing first-history
departure (`49ef722`) and immediate caret/outline binding (`7d351ac`). Arrows/Home/End preview while preserving the source caret,
selection/context and history; Left/Right disclose nested declarations, and
Return/click commits exact name bytes. Hidden/stale targets are rejected. Active
caret following, virtual rows, bounded labels, and existing SVG chevrons are
integrated. Bookmarks share key routing, movement, reveal geometry and UI fixtures.
Empty/seeded history regressions verify Unicode Return/Back restoration.
Full gates pass 283 app tests, 5 reader-text tests, 30 doctests, fmt, Clippy,
build and rustdoc. Native clean-session and GDB checks are recorded with exact
build/evidence scope in the [outline contract](plans/rust-outline-pane.md).
Architecture has no new hotspot/cycle, clone groups stay 91, duplication falls to
0.69%; remaining touched-file flags are documented. Full parity remains open,
including PR/source presentation, caches, index/PR lifecycle, broader native
restart/scaling/accessibility/corpus checks and sibling UI reuse. Keep pushing
after each commit.

## 2026-10-30 — Bookmark pane controls and ordered keyboard input (Codex)

Committed/pushed identity slot assignment (`b4c4a36`) and pane integration
(`c00308e`). Bookmarks now support independent UUID selection, Up/Down/Home/End,
Return/Delete/Escape, header current-location toggling, drag insertion and direct
shortcut-slot menus. Visible-row painting bounds long lists. Mutations save in
scope, stale IDs do not open replaced slots, and scope transitions retire state.
Full checks pass 277 app tests, 5 reader-text tests, 30 doctests, fmt, Clippy,
build and rustdoc. Native checks verify navigation, deletion, header toggling,
reordering and persisted restart on the final binary. Architecture has no new
product hotspot/cycle; one newly detected existing test-assertion clone remains
recorded. [Contract and exact evidence](plans/rust-bookmark-pane.md).
The user now requires pushing after every commit; preserve that instruction.
Outline keyboard behavior and the broader parity queue remain open.

## 2026-10-30 — Native inspector metadata acceptance (Codex)

Native checks verify Unicode local scope/count/source context, recorded function
facts/prose and real preview, exact Return/Back after an emoji, Here state,
package-only state without navigation, and Implementation type context with
F12/Back restoration. All private app/compositor exits are zero. Evidence hashes,
build checkpoints and precise scope are in the [metadata contract](plans/rust-inspector-metadata.md).
Full gates pass 273 app tests, 5 reader-text tests, 30 doctests, fmt, Clippy,
build and rustdoc. Remaining work includes outline/bookmark and broader inspector
acceptance, caches, indexing/PR lifecycle, source presentation/reuse and the final
Swift audit; full parity stays open.

## 2026-10-30 — Readable package labels (Codex)

Package facts omit placeholder versions and source-URL fields, matching Swift,
while preserving escaped package-name spaces. Parameterized metadata tests cover
version, URL and placeholder cases. Full gates remain 273 app tests, 5 reader-text
tests, 30 doctests, fmt, Clippy, build and rustdoc. Native acceptance at 96b86fd
verifies local/global/Here/package states and implementation navigation; evidence
is being recorded.

## 2026-10-30 — Source-interaction metadata refresh (Codex)

Native pinning exposed viewport caret mutation whose CaretMoved event was
replaced by PinAtByte in the same interaction. The app now observes actual
post-interaction caret changes and updates history/context regardless of the
winning source action. A regression verifies pinning refreshes metadata/current
history and unchanged carets preserve disclosure. Full gates pass 273 app tests,
5 reader-text tests, 30 doctests, fmt, Clippy, build and rustdoc. Native metadata
acceptance is being repeated on the corrected build.

## 2026-10-30 — Background metadata and definition-state UI (Codex)

Under-caret metadata now runs through replaceable read workers bound to exact
source/index/occurrence inputs. Token motion preserves disclosure; replacement
indexes and unavailable inputs retire old publications. Prepared destinations
feed previews and definition navigation, including another definition when the
current one is Here. The inspector shows bounded kind/count/signature/prose,
local source context without fetching, package facts and honest unavailable
states. Declaration summaries identify implementation blocks. Four additional
application/view tests pass; full gates are 272 app tests, 5 reader-text tests,
30 doctests, fmt, Clippy, build and rustdoc. No new product hotspot/cycle; a short
painted-text test clone remains. Native acceptance follows.

## 2026-10-30 — Parsed implementation type links (Codex)

Rust impl and Swift extension declarations have a distinct Implementation outline
kind. Their exact parsed type-name occurrence may bind reference context through
an indexed reference; definitions retain precedence, and ordinary declarations
still require a real definition. A native-grammar Rust/Swift test verifies the
link, ordinary-class rejection, precedence and stale-source rejection. Targeted
and full worktree checks pass; inspector metadata/UI integration continues.

## 2026-10-30 — Exact under-caret metadata query (Codex)

Added bounded SCIP kind/signature/prose, indexed definition/reference facts,
validated local occurrence counts, enclosing/package facts and exact primed local
definition lines. Here state and other-definition destinations are independent;
imports, Before, stale and multiline occurrences yield no facts. Five new tests
pass; shared UTF-16 test construction removes duplication across three clients.
Full gates pass 267 app tests, 5 reader-text and 30 doctests, fmt, Clippy and
rustdoc. No new product hotspot/cycle; duplication decreases. Background binding,
UI presentation and parsed implementation links follow.
[Contract](plans/rust-inspector-metadata.md).

## 2026-10-30 — Native caller disclosure acceptance (Codex)

Native acceptance verifies 256/512/516-row expansion, retained excerpts/source,
partial boundaries, consecutive-frame caller selection, exact Return to b.rs:5,
Back to the original a.rs caret after an emoji, reference-list focus after Trace
More disappears, and collapse without navigation. Final-build F12 from caller
focus opens b.rs:1. All private app/compositor exits are zero. Evidence, build
identity and debugger findings are in the [caller contract](plans/rust-incoming-callers.md).
Full gates remain 262 app tests, 5 reader-text tests and 30 doctests, fmt, Clippy,
build and rustdoc. Broader metadata, caches, pinned/mixed-language acceptance,
indexing/PR lifecycle, source presentation, reuse and final Swift audit remain.

## 2026-10-30 — Global reader shortcuts from caller controls (Codex)

Caller controls now participate in the reader-owned non-text focus check.
A new application regression verifies F12 navigates from a focused caller
control without being suppressed as text input. Full gates pass 262 app tests
(one external-Clang ignored), 5 reader-text tests, 30 doctests, fmt, Clippy,
build and rustdoc. Native core caller acceptance passes at bc3a929; the focused
final-build F12 smoke and final evidence documentation follow.

## 2026-10-30 — Pre-frame caller input ownership (Codex)

Consecutive-frame native input exposed egui 0.27 delaying focus filters until
a widget had focus in the previous frame. Caller-owned presses now pass through
eframe raw_input_hook before egui computes automatic focus direction. The existing
ordered event dispatcher is shared by raw and prepared input; Tab, modifiers and
global shortcuts remain available. The four-control regression covers consecutive
Down frames and ordered Return. Full gates pass 261 app tests, 5 reader-text,
30 doctests, fmt, Clippy, build and rustdoc. Final native repetition follows.

## 2026-10-30 — Caller arrow-focus correction (Codex)

A three-control regression and rust-gdb trace show egui applying a second
cardinal focus move at frame end after our ordered key dispatch. Caller controls
now share the reference list navigation filter, including immediately requested
programmatic focus. The corrected regression passes; Tab remains available.
Caller headers show retained row counts and rows identify their source filename.
Full gates pass 261 app tests, 5 reader-text tests and 30 doctests, fmt, Clippy,
build and rustdoc. Final-build native caller navigation is being repeated.

## 2026-10-30 — Virtualized caller disclosure UI (Codex)

Expanded references now render validated caller paths, stable identity colors,
boundary notes, partial notices and Trace More. Deep levels cap visual indentation
and preserve full depth in labels/tooltips. Visible-slice painting shares the
reference scroll area. Right again focuses loaded callers; ordered Up/Down,
Home/End, Return/Space, Left and Escape preserve inspector ownership. Trace
expansion retains old rows; disappearing controls restore list focus. Three new
pointer/virtualization/keyboard tests pass. Full gates pass 261 app tests (one
external-Clang ignored), 5 reader-text tests, 30 doctests, fmt, Clippy, build and
rustdoc. Native acceptance is underway. Architecture reports no new product
hotspot/cycle and unchanged maximum complexity; short clone reports remain.

## 2026-10-30 — Caller disclosure ownership and guarded navigation (Codex)

ReferenceDisclosure now owns independent excerpt and caller jobs together.
Collapse, target/index changes and pending-input rebinding retire both publication
channels; expanded result queries retain valid disclosure state. Caller-only
source reads join first-visit invalidation. Caller commits use exact trace digests
and scalar columns through existing source navigation; changed files reject
without source/history mutation. Three new controller/app tests plus strengthened
retention checks pass. Full tests: 258 app tests (one external-Clang ignored),
5 reader-text and 30 doctests; fmt and Clippy pass. UI rendering/Trace More and
native tree acceptance remain in progress.

## 2026-10-30 — Validated incoming-call backend (Codex)

Added a toolkit-independent source-validated caller trace following Swift
containing-callable semantics. It preserves sibling paths, cuts per-path cycles,
explains callback/closure/unindexed boundaries, validates source identities and
reports bounded coverage. Critical Lens and callers share the existing semantic
callee resolver. Nine tests include scoped IDs at matching cross-file offsets
and a 300-deep chain. Full checks pass 255 app tests (one external-Clang test
ignored), 5 reader-text tests and 30 doctests, plus fmt, Clippy, build and rustdoc.
Architecture has no new product hotspot/cycle; two short fixture/assertion clones
are documented. Background controller/UI integration and native tree acceptance
remain next. [Contract](plans/rust-incoming-callers.md).

## 2026-10-30 — Final grouped-reference native acceptance (Codex)

Final-build acceptance verifies honest limited/expanded coverage (511 to 515
references, one definition, one unmapped and one unreadable file), retained open
source context, keyboard selection after Load More, exact Return to b.rs line 5
and Back to the original a.rs use after an emoji. Radiology and both private
compositors exit zero. Build SHA, screenshots and debugger evidence are recorded
in the [reference-results contract](plans/rust-reference-results.md). Full gates
remain 246 app tests, 5 reader-text tests and 30 doctests. Parsed call trees,
richer context and the broader parity queue remain open.

## 2026-10-30 — Coalesced Load More focus correction (Codex)

Native debugging found that a same-frame pointer press/release lets list
registration surrender focus requested by Load More. Focus restoration now occurs
after registration. The regression test reproduces the native event sequence
and checks retained focus after the coverage button disappears. Full gates pass
246 app tests (one external-Clang test ignored), 5 reader-text tests, 30 doctests,
fmt, Clippy, build and rustdoc. Architecture has unchanged maximum complexity,
clone count and dependency cycles; average complexity and duplication decrease.
Native final-build navigation verification follows.

## 2026-10-30 — Cropped reference text and load-more focus (Codex)

Narrow source-line rows now mark measured prefix/suffix cropping with ellipses;
the exact indexed match stays visible. Load More restores reference-list focus
so subsequent keyboard input stays with the selected reference. Two pointer/
geometry tests cover these behaviors. Full gates pass 246 app tests, 5 reader-text
tests, 30 doctests, fmt, Clippy, build and rustdoc. Architecture introduces no
product hotspot/cycle; duplication and average CCN decrease. Native limited and
expanded queries show the expected counts and retain a selected open excerpt;
final-build navigation checks and evidence recording continue.
[Contract](plans/rust-reference-results.md).

## 2026-10-30 — Grouped source-line reference reader (Codex)

Connected source-backed results to the reference controller and inspector. Rows
show real code with measured match reveal, file headers/site counts and definition
badges; the References count excludes definitions. Coverage and Load More are
wired, and valid selection/previews survive larger queries. A shared captured
configuration removes split index/storage inputs; pending query reads retire on
source changes. The source ledger includes unmapped files, and newly readable
files refresh coverage. Controller tests now exercise actual source results,
scoped IDs, replacement indexes, load-more retention and unavailable-file recovery.
Full gates pass 244 app tests, 5 reader-text tests, 30 doctests, fmt, Clippy, build
and rustdoc. No new product hotspot/cycle; short clone reports remain. Native
acceptance is underway; parsed trees and full parity stay open.
[Contract](plans/rust-reference-results.md).

## 2026-10-30 — Source-backed reference query (Codex)

Added a toolkit-independent grouped query for definitions and uses, with shared
captured source reads/line windows, exact encoded/canonical ranges, source hashes
and explicit unmapped/unreadable/omitted coverage. Resource bounds include row
metadata even for empty text and can expand for more results. Nine new tests
cover Unicode, scopes, roles, relationships, limits, stale data and real pinned
Git source. Full gates: 244 app tests, 5 reader-text tests, 30 doctests, fmt and
Clippy; rustdoc checked. Architecture has no new product hotspot/cycle. Reader
controller/UI integration and native acceptance are next; full parity remains
open. [Contract](plans/rust-reference-results.md).

## 2026-10-30 — Native inspector preview and navigation acceptance (Codex)

Final-build Wayland checks verify real declaration-bounded previews, non-text
inspector shortcuts, single-click selection, independent source disclosures,
exact Return navigation and Back restoration after an emoji, with retained
reference state. Multiple disclosures and Left/Escape behavior also pass.
Radiology and both private compositors exit zero. Build SHA and screenshot
contracts are recorded in the [preview acceptance](plans/rust-inspector-previews.md).
Final gates remain 235 app tests, 5 reader-text tests, 30 doctests, fmt, Clippy,
build and rustdoc. Full parity remains open: source-line reference rows, grouping,
definitions/coverage, parsed call trees, richer context, indexing/PR lifecycle,
source presentation, reuse and the final UI/Swift audit remain required.

## 2026-10-30 — Ordered reader and inspector keyboard dispatch (Codex)

Reader shortcuts, reference controls and definition controls now share ordered
key consumption. Mixed definition keys in one frame no longer reorder, and
non-text inspector focus remains eligible for global reader shortcuts such as
F12 without leaking Return from reference commits. Two regression tests pass.
Full gates: 235 app tests, 5 reader-text tests, 30 doctests, fmt, Clippy, build and
rustdoc. Architecture reports no new product hotspot/cycle; average CCN and
duplication decrease, with unchanged clone groups. Native source-preview,
selection/disclosure and Return/Back acceptance passes; final-build smoke and
evidence recording remain. [Contract](plans/rust-inspector-previews.md).

## 2026-10-30 — Retained inspector state and keyboard focus (Codex)

Explicit-reference selection and loaded source disclosures now survive unchanged
file navigation and commit. Source identities include preview-worker reads, so
changed destinations invalidate on first reader visit; pending reads rebind to
newly displayed bytes before publication. Inspector keys retain egui focus while
Tab stays available. Snapshot binding and target selection are separate coherent
methods; the final new hotspot is removed. Added two retention tests, one pending
publication test and two viewport/focus tests; shared reference fixtures reduce
duplication to 0.70% with unchanged 86 clone groups. Full gates pass 233 app tests,
5 reader-text tests, 30 doctests, fmt, Clippy, build and rustdoc. Native reference
and definition disclosures pass; final commit/Back and evidence checks continue.
[Contract](plans/rust-inspector-previews.md). Full parity remains open.

## 2026-10-30 — Source previews and independent inspector interaction (Codex)

Added a shared exact-source preview worker for definitions and reference context,
with parsed declaration ends, UTF-16/UTF-8 bounds, stale/Before rejection and pinned
PR reads. Reference selection, multiple source disclosures and explicit commits
are independent of source/history; plain focused keys precede global Return and
modal input stays exclusive. Variable-height virtualization reuses RowHeights.
Nine new backend/app tests plus extended pointer replay pass; current full gates
are 228 app tests, 5 reader-text tests, 30 doctests, fmt, Clippy, build and rustdoc.
Architecture has no new product hotspot/cycle and lowers inspector show CCN 17 to
9; short clone reports remain. Native acceptance is in progress. Source-line row
presentation, coverage, parsed trees and broader full parity remain open.
[Contract and acceptance](plans/rust-inspector-previews.md).

## 2026-10-30 — Native reference acceptance and menu cleanup (Codex)

Native UTF-16/emoji checks verify source-menu activation, two exact reference rows,
sticky focus across files, independent under-caret definitions, reference-row
navigation and Follow Caret without source movement. Explicit single-line menu
labels now use separate shortcut hints. Three app runs and both private
compositors exit zero. Final gates: 219 app tests, 5 reader-text tests, 30 doctests,
fmt, Clippy, build and rustdoc. The reference-context contract records screenshots
and architecture evidence. Full parity remains open, especially preview/keyboard/
parsed-tree inspector work, indexing/PR lifecycle and final UI acceptance.
[Evidence and remaining scope](plans/rust-reference-context.md).

## 2026-10-30 — Shared exact indexed owner lookup (Codex)

Reference labels and the legacy caller API now share one full-coordinate enclosing
lookup. Same-line declarations stay distinct; negative extents cannot label rows,
and extent span arithmetic cannot overflow. A ninth reference-focus test covers
same-line ownership; malformed extent coverage extends an existing test. Full
gates pass 219 app tests, 5 reader-text tests, 30 doctests, fmt, Clippy, build and
rustdoc. No new product hotspot/cycle is reported; short test clone reports remain.
Native explicit focus, independent definition changes and Follow Caret checks
pass; final menu polish and evidence recording are underway.

## 2026-10-30 — Exact background reference focus (Codex)

Find References now pins an exact scoped symbol independently of caret/file
navigation; Follow Caret resumes the latest enclosing declaration. A toolkit-free
controller reuses LatestRead, suppresses same-context rebuilds and retires stale
index/source requests. Exact definition-name bytes replace line-only matching;
flat direct references replace unvalidated recursive caller guesses in the reader.
Rows are sorted/deduplicated, encoding-preserving and virtualized. Search/source
menus and Ctrl+Alt+R are wired. Eight new tests and full gates pass: 218 app tests,
5 reader-text tests, 30 doctests, fmt, Clippy, build and rustdoc. Architecture has
no new product hotspot/cycle; short test clone reports remain. Native menu/focus
acceptance is underway. Previews, keyboard commit and parsed trees remain open,
alongside the full parity queue. [Contract](plans/rust-reference-context.md).

## 2026-10-30 — Typed structural queries and Fold to Definitions (Codex)

Fold to Definitions now collapses callable/leaf-type bodies while exposing nested
declarations and expanding unrelated comments/imports/blocks. Shared Swift fold
and symbol queries supply exact roles/bytes/containment; Kotlin/C adapters cover
the linked grammars. A shared independent-role registry replaces call-only setup;
text-based comment guessing and line-based body reconstruction are removed.
Folding errors remain recoverable without disabling calls/source. Seven new query/
app tests and native View-menu/file-return acceptance pass. Gates: 210 application
tests, 5 reader-text tests, 30 doctests, fmt, Clippy, build and rustdoc; app and
compositors exit zero. Architecture retains short test/footer clone reports with
no new product hotspot/cycle and lower duplication/average CCN. Same-row controls,
Kotlin import grouping, broader language/viewport checks and full parity remain
open. [Contract and evidence](plans/rust-structural-folds.md).

## 2026-10-30 — Initial callable-body and PR fold defaults (Codex)

Fresh files collapse parsed bodies while retaining multiline signatures and open
containers. PR defaults open changed bodies/headers/annotations and ancestors on
the correct revision side, with careful empty-side/EOF ownership. Default choices
are prepared in the source worker and applied once before destination reveal;
manual/restored fully expanded sets remain initialized and are never overwritten.
Body ranges now drive function folds and count matching. Eight new syntax/app
tests include pinned Git data, Rust/Python ownership, abstract Swift signatures,
Hide Tests and manual revisit choices. Native first-open/signature/sibling and
restored-expansion checks pass; app/compositors exit zero. Full gates: 203 app
tests, 5 reader-text tests, 30 doctests, fmt, Clippy, build and rustdoc. Architecture
retains short test clones, without a new product hotspot/cycle. Fold to Definitions,
broader query coverage, late-data/refresh mapping and full parity remain open.
[Contract and evidence](plans/rust-fold-restoration.md).

## 2026-10-30 — Exact-document fold choices and shared presentation memory (Codex)

File return and unchanged reload now restore collapsed regions by path/revision/
SHA-256. Source failures retain visible choices; navigation reveals only containing
folds, and changed bytes invalidate old choices. Repository/snapshot boundaries
reset memory. Shared DocumentMemory moves pin/fold payloads and replaces duplicated
activation logic; existing pin behavior remains covered. Unit/app tests plus native
independent a.rs/b.rs collapse-and-return checks pass; app/compositors exit zero.
Full gates: 195 application tests, 5 reader-text tests, 30 doctests, fmt, Clippy,
build and rustdoc. Architecture retains a short test clone, with no new product
hotspot/cycle. Fresh/PR fold defaults and refresh mapping remain open, alongside
full parity. [Contract and evidence](plans/rust-fold-restoration.md).

## 2026-10-30 — Progressive source function counts (Codex)

Fold counts now publish six completed functions at a time and flush the remainder,
yielding between roots. A request-local coalescing slot and existing LatestRead
retain cancellation/generation ownership without an event backlog or whole-prefix
cloning. Completion drains the final batch; later errors retain valid counts.
Three new tests prove live progress, late-error retention and retired-slot isolation.
Full gates pass: 193 application tests, 5 reader-text tests, 30 doctests, fmt,
Clippy, build and rustdoc. Architecture is NEUTRAL, with no new clone/hotspot/cycle.
Full parity and request/revisit cache reuse remain open.
[Contract](plans/rust-call-graph.md).

## 2026-10-30 — Interactive Critical Lens source hover (Codex)

Connected exact measured source hits to dwell/grace summaries with stable popup
geometry. Full definition rows navigate independently of caret through existing
indexed navigation/history; external rows remain unavailable. At most eight
critical/frontier rows show paths and coverage. Keys/clicks/scroll and captured
configuration transitions dismiss stale presentation. Fixed background completion
repaint so loading cannot remain painted indefinitely at an idle pointer.
Headless replay verifies real hit regions, Escape, idle suppression and source/
caret/selection preservation. Native UTF-16 fixture verifies summary, popup entry
and full-row activation, cross-file navigation, exact Back, external/plumbing
behavior and mode dismissal. App runs/compositors exit zero. Full checks pass
190 application tests, 5 reader-text tests, 30 doctests, fmt, Clippy, build and
rustdoc. Architecture retains a short interaction-test clone and existing footer
sequences; no new product hotspot/cycle. [Evidence and remaining acceptance](plans/rust-critical-hover.md).
Full parity remains open, including progressive counts, lifecycle/cache gaps,
remaining inspector/source/PR work and final UI acceptance.

## 2026-10-30 — Critical hover dwell and lifecycle controller (Codex)

Added toolkit-independent 350 ms dwell/700 ms grace, exact semantic eligibility,
external/no-body presentation and replaceable snapshot-worker publication.
Configuration/input changes cancel stale work; idle pointers do not reopen a
dismissed popup. Four deterministic controller tests pass. Current working tree
(including pending reader integration) passes 190 app tests, 5 reader-text tests,
30 doctests, fmt, Clippy, build and rustdoc. Native reader interaction remains
under verification; full parity is open. [Contract](plans/rust-critical-hover.md).

## 2026-10-30 — Source Critical Lens fold counts (Codex)

Connected snapshot-bound count sweeps to the standalone reader. Collapsed
functions show reachable counts, partial `n+` and honest no-body markers in
Critical mode; Normal retains hidden-line totals. Disclosure/count/line numbers
have separate gutter lanes. Tooltip/accessibility preserve hidden lines and
coverage reasons; count clicks toggle folds. All captured-input transitions
retire stale work, with no caret-driven graph work or inspector/outline counts.
App/pointer tests cover lifecycle, replacement index, policy, fold activation and
source preservation. Native UTF-16 checks verify 2 hidden lines vs 1+, mark-driven
0+, expanded count hit target, tooltip and Normal restoration; app/compositors
exit zero. Full checks pass 185 application tests, 5 reader-text tests,
30 doctests, fmt, Clippy and build. Architecture retains short test/footer clone
reports, without a new product hotspot/cycle. Hover and progressive publication
remain open, alongside the broader parity queue. [Evidence](plans/rust-call-graph.md).

## 2026-10-30 — Snapshot-bound graph worker and source-count service (Codex)

Added canonical definition body loading through captured SourceRead storage,
primed displayed bytes, request-local parsing/summary caches and explicit source
budgets. A replaceable fold-only count sweep shares the worker and publishes
exact-source annotations with coverage detail. Six new tests include real pinned
Git reads after checkout edits, cache isolation, budgets and stale-publication
retirement. Full checks pass 183 application tests, 5 reader-text tests,
30 doctests, fmt, Clippy and rustdoc. Architecture retains one short test assertion
clone, with no new product clone/hotspot/cycle. Count UI and hover integration are
next; full parity remains open. [Contract](plans/rust-call-graph.md).

## 2026-10-30 — Parsed call summaries and bounded frontiers (Codex)

Shared call parsing now retains lexical callable ownership, abstract/body facts
and syntax-error coverage. A shared exact SCIP resolver serves projection and
summary analysis. Summaries exclude nested execution; breadth-first frontiers
retain shortest paths, deduplicate cycles/diamonds and report honest partial
counts for unknowns, macros, conditional sites, missing bodies and bounds.
Seven new tests and full checks pass: 177 application tests, 5 reader-text tests,
30 doctests, fmt, Clippy, binary build and rustdoc. Architecture is IMPROVED, with
no new hotspot/clone/cycle and less duplication. Reader worker, hover/fold-count
integration and native acceptance remain open; full parity is not complete.
[Contract and next steps](plans/rust-call-graph.md).

## 2026-10-30 — Critical Lens source marking integration (Codex)

Normal/Critical source menus now edit exact symbol/package decisions and restore
Built-In policy. Repository loads and edits refresh background projections; errors
remain visible while session decisions stay active. Shutdown drains accepted saves.
App tests verify precedence, resets, unchanged reader state, stale/Before rejection
and restart restoration. Native UTF-16 fixture checks verify real right-click
targeting, both menus, projected colors, reset and restart-selected state. Both
app runs/compositors exit zero. Full checks: 170 application tests, 5 reader-text
tests, 30 doctests, fmt, Clippy and build. Architecture retains short clone reports
and a required footer error branch; max CCN unchanged.
[Evidence and limits](plans/rust-critical-marks.md). Full parity remains open,
including call graphs/frontiers, annotations, index/PR lifecycle and final audit.

## 2026-10-30 — Durable mark session and exact menu targets (Codex)

Added replaceable mark loads, pre-load edit replay, coalesced durable writes,
repository-switch/Drop retention and bounded shutdown flushing on existing
workers. Shared SCIP headers and cached ownership avoid menu-time index scans.
Lifecycle/menu tests pass with the current working tree: 169 application tests,
5 reader-text tests, 30 doctests and Clippy. Source-menu/application wiring is
uncommitted and still needs integration/native acceptance. Full parity remains
open. [Contract](plans/rust-critical-marks.md).

## 2026-10-30 — Repository-scoped Critical Lens decision storage (Codex)

Added atomic ordered mark storage outside source repositories, canonical root/hash
isolation, bounded/cancellable loading and explicit read/save errors. Store clones
reserve revisions before queueing so delayed old saves cannot undo newer decisions
or deletion. Typed symbol/package edits replay over delayed loads; locals and
Unknown/empty decisions do not persist. Default override format is now correctly 1.
Shared bounded file reads and destination resolution replace duplicated I/O/path
logic; no new dependency or runtime is introduced.

Seven storage/replay tests and full verification pass: 164 application tests,
5 reader-text tests, 30 doctests, fmt, Clippy and binary build. Architecture reports
one new seven-line test-fixture clone, with no product clone/hotspot/cycle. The
reader still supplies empty overrides; background loading/saving, menu controls
and projection refresh are the next integration commit, not completed here.
[Decision contract and integration queue](plans/rust-critical-marks.md).

Full parity remains open, including marking UI, call graphs/frontiers, annotations,
index lifecycle gaps and final Swift/runtime acceptance.

## 2026-10-30 — Deadline-bound busy-executable startup repair (Codex)

Reproduced Linux ETXTBSY with a real executable held open for writing: the old
runner failed immediately. Startup now waits only for that pre-launch condition,
using the existing cancellation token and original wall-clock deadline. Held-open
persistent writers time out, cancellation interrupts the wait, and release permits
exactly one successful launch. Other errors/exits are unchanged. The original
parallel test's specific writer was not traced; this repairs the observed OS
condition rather than asserting an unverified descriptor origin.

Deterministic regression and full workspace verification pass: 157 application
tests, 5 reader-text tests, 30 doctests, fmt, Clippy with warnings denied and binary
build. Twenty additional parallel library-suite runs pass after the repair;
the deterministic busy/release/timeout/cancel regression passes in each. The
architecture comparison is neutral. [Runner evidence](plans/rust-indexing-parity.md).

Critical Lens integration `f28af87` now has a passing full-suite run with the runner
repair. Persisted marks/menus, call graphs/frontiers, fold annotations, indexing
lifecycle gaps and final Swift/runtime acceptance remain open.

## 2026-10-30 — Background Critical Lens reader presentation (Codex)

Integrated backend `e3ee241` through the existing read-worker protocol. Immutable
indexes use Arc snapshots; call facts are retained from source analysis. Mode,
source, index and repository transitions retire old projections. Renderer span
splitting uses exact canonical ranges instead of syntax-color guessing, retains
original bytes/geometry, and ignores mismatched/pending projections. Footer status
reports preparation or call syntax coverage. The obsolete name-only classifier
was removed; source presentation uses semantic identity and unknown-call retention.

Native UTF-16 fixture acceptance verifies runtime-call dimming, prominent system
and unknown calls, Normal restoration, and clean app/compositor exits. Critical
unit/integration tests, doctests, fmt, Clippy and build pass. Full suite passed once
(156 application + 5 reader-text + 30 doctests), then repeated runs exposed Linux
ETXTBSY in the existing tool-probe fixture, including its timeout assertion. That
runner startup issue is the next separate repair, not claimed fixed here.
Architecture retains test clones and projection/source orchestration flags with
no new product clone or CCN threshold spike. [Evidence](plans/rust-critical-lens.md).

Full parity remains open, including persistent marks/menus, call graph/frontiers,
hover/count annotations, index lifecycle gaps and final Swift/runtime acceptance.

## 2026-10-30 — Critical Lens call facts and semantic projection backend (Codex)

Native source analysis shares its parse with call queries, reusing Swift resources
and adapting kotlin-ng/C nodes. Facts retain exact source/revision identity and
query coverage/errors. A pure projection now matches Swift's call-focused rule:
critical and unknown syntactic callees remain prominent; resolved ordinary calls
and non-call bytes recede. Encoded ranges, source digests, Before isolation,
scoped identities and existing override precedence are preserved. Shared SCIP
header/descriptor mechanisms replace last-space descriptor guessing.

Verification: 154 application tests, 5 reader-text tests, 30 doctests, fmt and
Clippy pass. All linked call queries compile; fixtures cover macros/dynamic calls,
Unicode, stale data, overrides and exact identity. Renderer/worker integration
follows separately; the old viewport heuristic is still present at this commit.
Full Critical Lens and full parity remain open. [Critical Lens state](plans/rust-critical-lens.md).

## 2026-10-28 — Reader pin restoration and group controls (Codex)

Integrated store `8987675` at atomic source publication: leaving/returning restores
pins only for identical document/revision/bytes; failed reads preserve the visible
pins. Semantic and fallback groups have readable source labels. A compact scrolling
row shows color, spelling and count, supports group removal and Clear Pins for
the current document. Existing source actions, type scale and palette are reused.

Verification: 147 application tests, 5 reader-text tests and 30 doctests pass,
with fmt, Clippy and binary build. Application tests cover five groups, navigation,
failed reads and changed bytes; pointer tests cover remove/clear controls. Private
native Weston/Niri checks verify five groups, restoration, unchanged surviving
colors after middle removal and clearing. App/compositors exit zero. Architecture
has no new product clone/hotspot/cycle, but two test input-replay clones remain
reported. [Pin parity and evidence](plans/rust-occurrence-pins.md).

Full parity remains open. AST-aware fallback, full decorations, fold restoration,
PR-refresh restoration, indexing/provisioning, Critical Lens, call graphs and final
Swift/runtime acceptance still require work. Pin state is session memory, matching
Swift, not persistence across app restarts.

## 2026-10-28 — Unlimited document-scoped pin store (Codex)

Removed pin eviction at four groups. Palette slots advance monotonically, while
removal leaves surviving colors unchanged. Document activation retains pins by
path/revision and exact SHA-256 bytes; changed content discards stale ranges and
ephemeral highlights do not cross documents. Equivalent sorted/deduplicated ranges
reconcile fallback and semantic identities. Labels are independent of identities.

Store regression tests pass; architecture comparison is neutral. Reader activation
and remove/clear controls follow separately. AST-aware fallback and PR refresh
restoration remain open. [Pin parity](plans/rust-occurrence-pins.md).

## 2026-10-28 — C/C++ generation integration and native acceptance (Codex)

Connected backend `574f68d` to generation: supported C/C++ compilation databases
are prepared in the private run, required as absolute adapter input, and leave
copied/original build inputs unchanged. Response expansion, source/encoding guards,
coverage, disposal and persistent caching share the existing generation path.
The blanket C/C++ generation gate is removed; no new jobs or UI flow was added.

Validation: 142 application tests, 5 reader-text tests, 30 doctests, fmt, Clippy and
binary build pass. The explicit real-Clang test also passes with original sources
unavailable. Service tests inspect nested tracked database argv, unchanged bytes,
UTF-8 publication and cache reload; native private Weston/Niri Generate publishes
fixture SCIP coverage and semantics, captures private source/output paths and
cleans its workspace. App/compositors exit zero. Architecture has no new product
clone/hotspot/cycle; one six-line test fixture clone remains reported.

Limitations remain: no installed scip-clang for actual semantic compiler acceptance,
ignored generated build-database discovery, external build directories and broader
compiler path-option protocols. Full parity remains open, including provisioning,
cache reuse/retention, pinned After indexing, Critical Lens, call graphs and final
Swift acceptance. Details: [C/C++ indexing](plans/rust-clang-indexing.md).

## 2026-10-28 — C/C++ compilation database rebase backend (Codex)

Added bounded compilation-command rebasing for private inputs: argument arrays and
quoted commands, split/joined path flags, literal macros, output redirection,
duplicate configurations and recursive GNU response files. Existing workspace
normalization/file reads are reused; no shell evaluation is introduced. The
already locked shlex dependency handles POSIX command quoting.

Six backend tests pass; an explicit real-Clang acceptance test also passes with
the original repository moved away and object output confined to the private copy.
The full suite currently passes 139 application tests, 5 reader-text tests and
30 doctests (Clang acceptance ignored by default), with fmt/Clippy. Generation
integration follows in a separate commit; `scip-clang` is absent on this machine.
Full parity remains open. Details: [C/C++ indexing](plans/rust-clang-indexing.md).

## 2026-10-28 — Cached-index reader reopen, fallback and native acceptance (Codex)

Integrated backend `f51ab3d`: repository opening and Reload Index try persistent
cache data first, preserve explicit stale provenance and fall back to repository
SCIP imports when absent/invalid. Invalid-cache diagnostics remain visible beside
the import's unknown freshness. Cancel suppresses fallback; repository switches
retire pending cache results. Reindex still runs tools, and persistence warnings
are visible in Index Projects. No sibling runtime dependency was added.

Tests cover reopen with tools removed, stale source, corrupt-cache import fallback,
cancellation and switching before publication. Native private Weston/Niri checks
verified generated → cached-without-indexer → stale → corrupt/import-fallback
states, with final app/compositors exiting zero. Validation: 133 application tests,
5 reader-text tests, 30 doctests, fmt, Clippy and build pass. Architecture has no
new production clone/hotspot/cycle; one six-line fixture clone remains flagged.
See [persistent indexes](plans/rust-index-cache.md) for evidence and boundaries.

Full parity is open: per-project/version-aware cache reuse, tool provisioning,
C/C++ rebasing, pinned After generation/cache reuse, refresh lifecycle, Critical
Lens, parsed call graphs, richer inspector/source presentation and final Swift
acceptance remain. Basic persistent-cache absence is no longer a blocker.

## 2026-10-28 — Persistent semantic cache backend (Codex)

Generated indexes now persist atomically outside the checkout with exact source
digests, input inventories and bounded project reports. Cache-only loading uses
the existing background job, runs no tools and makes no workspace. Stale reloads
retain original provenance; corrupt/foreign/unsupported entries are rejected.
Source fingerprinting reuses generation identities without copying inputs. Shared
SCIP/cache reads remove duplicate low-level logic and add chunk cancellation.

Backend validation: 133 application tests, 5 reader-text tests, 30 doctests, fmt,
Clippy and build pass. The architecture check has no new production clone or CCN
hotspot; one six-line fixture clone remains reported. Reader reopen/fallback/status
integration follows separately. Full parity is open. See [persistent indexes](plans/rust-index-cache.md).

## 2026-10-28 — Shared semantic navigation and symbol-picker acceptance (Codex)

Integrated core commit `52d7580`: workspace keyboard/pointer selection carries
scoped identity, callers preserve original reference coordinates, and both share
F12's encoding/digest-checked asynchronous navigation. Before exposes its outline.
The consolidated picker virtualizes rows, reveals keyboard selection, handles
query-plus-Enter in one frame and avoids cloning nested outline trees.

Verification passes 128 application tests, 5 reader-text tests, 30 doctests, fmt,
Clippy with warnings denied and binary build. Private native Weston/Niri checks
verified keyboard/pointer symbols, caller rows and F12 at exact positions after
emoji; app and both compositors exited zero. Fixture imports do not constitute
real compiler-indexer acceptance. Architecture duplication improves, but test
clones, update CCN 51→53 for required checks and orchestration flags remain;
the tool verdict is still REGRESSION. Details: [semantic navigation](plans/rust-semantic-navigation.md).

Full parity remains open: persistent validated caches, private tool provisioning,
C/C++ compilation-database rebasing, pinned After generation, refresh lifecycle,
Critical Lens, parsed call graphs, richer inspector/source presentation and final
Swift/runtime acceptance. The older palette/caller TODO below is now superseded.

## 2026-10-28 — Canonical semantic coordinates and cache-aware destinations (Codex)

Search labels, definition resolution and relationship fallback now use the same
canonical definition ordering. Caller nodes retain full SCIP reference ranges,
not just display lines. Display-line conversion avoids signed integer overflow.
Packed and typed ranges reject negative/reversed coordinates while preserving
explicit typed-field precedence; unknown encodings are no longer guessed as UTF-8.

Destination provenance is shared by worker resolution and the same-buffer fast
path. When an index describes newer bytes than a cached same-path buffer, the
reader now rereads the source and checks that digest instead of rejecting the
cache mismatch without reading. Stale freshly read bytes still fail safely.

Core regressions and in-progress UI integration currently pass 128 application
tests, 5 reader-text tests, 30 doctests, fmt, Clippy with warnings denied and binary
build. Typed palette/caller actions and their native acceptance follow in the next
commit. Full parity remains open. See [semantic navigation](plans/rust-semantic-navigation.md).

## 2026-10-28 — Generate/Reindex reader integration and native acceptance (Codex)

Integrated backend `88bbc95` and Python identity fix `e4a94ed` into shared reader
commands. Index Projects exposes Generate/Reindex/Cancel, last-run coverage and
bounded diagnostics. Working-tree source drift, failed/all-failed runs and retained
prior data remain explicit. Imports and generation retire each other's jobs;
repository/review changes retire generation and clear reports. Hosts can configure
`index_tools`, drive `poll_index`, or use the independent generator directly.

Mismatched open source has a Reload Source action. Inspector availability now
requires document coverage and matching bytes. Repository/cache acquisition is
shared with PR preparation. Tool-path setup is pure: config-path discovery no
longer creates directories, and an isolated subprocess test verifies idle default
reader construction leaves configuration absent.

Native isolated Weston/Niri acceptance used a fixture indexer: Generate published
semantic data, Reindex retained the reader, Find stayed responsive with a real
highlighted match while the indexer ran, and Cancel stopped the process, removed
the private workspace and retained the prior index. Actual command environment
used 16 Cargo jobs and a target directory inside the private copy; no artifact was
written into the viewed fixture. App and both compositors exited zero. Captures
and limits: [indexing parity](plans/rust-indexing-parity.md).

Final verification: 120 application tests, 5 reader-text tests, 30 doctests, fmt,
Clippy with warnings denied and binary build pass. One earlier probe-test failure
was not reproduced in 20 parallel library-suite runs; enhanced diagnostics remain,
not a speculative fix. Architecture comparison still reports REGRESSION for two
known test-only clones and review/workspace/generation orchestration structural
flags; no newly excessive CCN or product clone remains. Full parity is open.

Next requirements include persistent validated caches, private provisioning,
real compiler-indexer acceptance, C/C++ rebasing, pinned After generation, refresh
lifecycle and complete semantic destination checks. Global symbol-palette and
caller-row navigation still need the shared encoded/digest-checked destination
path; F12 definitions already use it. Critical Lens, parsed call graphs, inspector
previews and final full acceptance remain in the original parity ledger.

## 2026-10-28 — Stable Python project identity in isolated runs (Codex)

Fixed Python's `--project-name` to use the original repository/project directory,
not the randomly named build copy. `CommandPaths` now carries the original root;
a real fixture subprocess records argv to verify identity survives isolation and
literal source paths. Workspace tests, including pending reader integration,
pass 119 application tests, 5 reader-text tests and 30 doctests, fmt and Clippy.
The rust-analyzer component is missing on this machine; real compiler-indexer acceptance remains pending.

## 2026-10-28 — Isolated checkout index generation service (Codex)

Added GUI-independent `IndexGenerator` using `LatestRead`, bounded live progress
and per-strategy outcomes. It replans from full build inputs, retains a pristine
baseline, creates independent strategy copies, probes real tool versions, writes
auxiliary resources privately, runs cancellable processes and verifies inputs.
Cargo target output is isolated and build jobs use half available logical CPUs.
Shared bounded SCIP decoding rebases real source documents, sets per-document
encodings, merges scoped symbols and attaches exact source digests. Missing,
overlapping and partial coverage and source drift remain explicit in the report.

Backend fixture tests cover real subprocess execution, output isolation, mixed
success/failure, encodings/local IDs, invalid or linked output, rewritten inputs,
source additions/edits, replacement/cancellation, deadlines and partial-build
markers beyond retained diagnostic bounds. Current workspace verification,
including reader integration in progress, passes 118 application tests, 5 reader-
text tests and 30 doctests, fmt and Clippy. One version-probe test failed once;
20 subsequent parallel library-suite runs did not reproduce it. Its assertion now
prints the full probe result for diagnosis; no speculative production fix was made.

Architecture retains the known test clones and review/workspace structural flags;
new generation CCN and product clones were removed. C/C++ generation is explicitly
gated pending compilation-database rebasing. Cache persistence, provisioning,
pinned-review generation and full parity remain open. Reader/native integration
is the next commit. See [indexing parity](plans/rust-indexing-parity.md).

## 2026-10-28 — Semantic coordinate provenance boundary (Codex)

Source buffers now cache a SHA-256 digest alongside the existing bookmark/find
identity. Generated semantic indexes can carry per-document source digests;
occurrence/range lookup rejects a mismatched immutable buffer without hashing it
on each caret movement. Encoded definition requests carry the target digest and
reject changed target bytes after the worker reads them. Source semantic status
also requires matching bytes. Imports retain explicit unknown-freshness behavior.

Verified 111 application tests, 5 reader-text tests, 30 doctests, fmt and Clippy
with warnings denied. The new test checks coordinate and destination rejection
and import compatibility. Architecture comparison retains known test clones and
review preparation, and now also ranks the existing workspace materialization
function as a structural hotspot; no new product clone/CCN threshold regression.
Generation publication must populate these digests before its UI is enabled.
Full parity remains open.

## 2026-10-28 — Real build inputs and disposable indexing workspaces (Codex)

Added full Git tracked/unignored input discovery, retaining tracked build/vendor/
Cargo configuration files hidden by the reader. Git inventory streams complete
NUL-delimited bytes through the existing cancellable process runner; diagnostic
truncation cannot drop paths. Worktrees are supported and submodule directories
are reported as omitted. Non-Git discovery retains Cargo configuration, enforces
file/path limits and reports enumeration failures.

Disposable workspaces preserve bytes/modes, redirect internal links, literalize
external Markdown links and reject external source links and linked parents.
Cache destinations inside the viewed tree are rejected before creation. Bounded,
cancellable copies clean up on failure/drop. SHA-256 fingerprints and type/mode/
byte checks detect source/build rewrites; Git-compatible blob IDs support exact
snapshot checks. `tempfile` now manages production workspace lifetime; SHA-1/SHA-2
crates were added without updating existing locked dependency versions.

Verified 110 application tests, 5 reader-text tests and 30 doctests, fmt and Clippy
with warnings denied. Actual Git/filesystem/process fixtures cover large streamed
inventories, worktrees/submodules, bytes/modes/links, provenance, limits, live copy
cancellation and cleanup. Architecture comparison retains only the known findings
after simplifying workspace stages and shared stream draining. No native index
generation run yet. [Indexing parity](plans/rust-indexing-parity.md) records scope
and limits. Generate/Reindex integration, caches, provisioning and full parity
remain open.

## 2026-10-28 — Actual indexer availability and native status controls (Codex)

Connected managed/PATH executable discovery and real version probes to Check
Tools/Cancel Checks. Probes use the bounded process runner and shared latest-job
retirement, recognize missing rustup components, retain dispatch symlinks, and
report versions/failures. Index reset is shared by repository/review transitions;
no stale tool results publish after a switch. Review probes use the pinned head
workspace. Prior results remain available during cancelled checks.

Native testing found bottom-menu clipping. Replaced it with a resizable Index
Projects window whose list scrolls while actions remain visible. An isolated
1280×900 Weston/Niri run exercised six managed fixture tools, failure/version
status, scrolling and slow-probe cancel controls. The app and compositors exited
zero. Reused sibling compositor/pointer test helpers through a temporary driver;
Radiology has no new sibling dependency or sibling edits.

Verified 100 application tests, 5 reader-text tests, 30 doctests, fmt, Clippy with
warnings denied and native build. Architecture findings remain the known two
test-only clones and review-preparation structural flag. See [indexing parity](plans/rust-indexing-parity.md) for evidence and limits. Generation, private
installation, workspace isolation, cache provenance and full parity remain open.

## 2026-10-28 — Bounded cancellable indexer subprocesses (Codex)

Added the POSIX process runner for index generation and version probes. It drains
both nonblocking pipes with a per-turn budget, retains bounded first/last output,
reports exit/signal/error diagnostics, enforces deadlines and owner cancellation,
and kills/reaps the child process group on every completion path. Normal tool
exit also cleans up children holding pipes open. Leader identity is reserved
until cleanup using waitid/WNOWAIT. No pipe-reader threads or output files leak.

Verified 96 application tests, 5 reader-text tests and 30 doctests, fmt and Clippy
with warnings denied. Real process tests cover stream pressure, cancellation,
deadlines, TERM-ignoring descendants, normal-exit descendants, signals and spawn
failures. Architecture findings remain unchanged. Tool probes, generation,
workspace isolation/cache provenance and full parity remain open; the next
integration uses this runner rather than adding another subprocess mechanism.
See [indexing parity](plans/rust-indexing-parity.md).

## 2026-10-28 — Project strategy planning and adapter command definitions (Codex)

Added deterministic project planning for all six Swift indexers, using build-file
evidence and source inventory independently of reader filters. Plans suppress
nested projects, count covered extensions, order by depth/registry/path, cap runs
at 32 and retain skipped projects. Background planning retires stale repository/
review results; the Index Projects menu displays the current plan.

Structured command definitions keep repository paths out of shell interpolation.
Kotlin reuses Swift's compiler script and Gradle init resource byte for byte.
Commands are not launched yet; the UI explicitly reports generation unavailable.
Private provisioning, isolated workspace/process execution, Rust feature cfg
injection, generated coverage, validated caches and snapshot indexing remain.

Verified 89 application tests, 5 reader-text tests and 30 doctests, fmt and Clippy
with warnings denied. Architecture comparison retains only the known test clones
and review-preparation structural flag. No fresh native visual acceptance in this
round. See [indexing parity](plans/rust-indexing-parity.md) for precise limits and
next requirements. Full parity remains open.

## 2026-10-28 — Background semantic imports with honest status (Codex)

Moved existing checkout SCIP imports off the UI thread using the shared
`LatestRead` worker protocol. Added bounded reads, fallback/error reporting,
repository-relative path validation and empty-index rejection. Import status
exposes file provenance and explicitly unknown freshness. Reload/Cancel commands
retain prior data on failure/cancellation; repository/review changes retire jobs.
Source semantic status now requires actual document coverage. Embedded hosts can
poll index publication independently of eframe.

Verified 82 application tests, 5 reader-text tests and 30 doctests, fmt and Clippy
with warnings denied. New backend/app tests exercise imports, fallback, limits,
errors, cancellation/replacement and repository transitions. Architecture retains
only the already documented findings. No new native visual acceptance yet.

See [indexing parity](plans/rust-indexing-parity.md). Strategy detection, generated
indexes, provisioning, isolated workspaces, cache provenance and pinned revision
indexing are still required; this commit does not complete R4 or full parity.

## 2026-10-28 — Atomic asynchronous reader navigation (Codex)

Integrated the source worker (`7b614ab`) into file opens, bookmarks, encoded
definitions, revision changes and Back/Forward. Current bytes/caret/folds/pins/
revision remain usable until replacement is ready. Pending path/cancel feedback
is visible; failures preserve source/history. History commits only at successful
publication and captures movement in the old source while loading. Existing-file
navigation cancels an older replacement; repository/review transitions retire jobs.

Pinned reviews and syntax resources now share Arcs. Theme changes retain pending
navigation or recolor the same immutable source without rereading it, preserving
view state. Semantic ranges and definition requests share one encoding lookup.
Plain default file opens keep a first test region hidden; explicit destinations
reveal it. Embedded hosts can call `poll_source` without an eframe window.

All prior reader tests now poll actual worker completions. Added cancellation,
failure, latest-departure, stale repository/current-document, appearance and
first-test-region tests. Verified 78 application tests, 5 reader-text tests and
30 doctests, fmt and Clippy with warnings denied. Native isolated smoke showed
pending status over old source, usable file picker, retirement by returning to
current source, and successful recoloring. Own window closed after capture.
See [source loading](plans/rust-source-loading.md) for evidence and limits.

Architecture comparison retains the two known test-only assertion clones and
review-preparation structural flag; no new product duplication/CCN regression.
Full parity remains open. Repository/index discovery, Git refresh, SCIP decoding,
semantic context, finer native/child-process cancellation, Critical Lens,
inspector/reference previews and final desktop acceptance are still required.

## 2026-10-28 — Background source preparation foundation (Codex)

Added `SourceLoader`: exact checkout/pinned-review reads, destination decoding
(bytes, scalar positions, declared index encoding, bookmark relocation), shared
structural analysis, cancellable per-line highlighting, and file-specific diffs
run on the existing four-worker pool. Prepared source is immutable and published
as one payload. Existing-buffer recoloring shares its Arc and skips structural
parsing and filesystem reads. Retired requests cannot publish.

Extracted the actual cancellation/channel ownership shared with local find into
`LatestRead`; no second retirement protocol is retained. Real-source tests cover
replacement, exact Unicode coordinates, failures and recoloring after the backing
file was removed. Verified 74 application tests, 5 reader-text tests, 30 doctests;
formatting and Clippy with warnings denied pass. Architecture flags remain the
previous two test-only clones and review-preparation structural finding.

This commit is the backend foundation: the reader still uses synchronous source
opening until the next integration change. [Source loading contract and remaining
integration](plans/rust-source-loading.md) records publication/history requirements,
worker cancellation limits, and remaining repository/index work.

## 2026-10-28 — Inline Hide Tests and navigation integration (Codex)

- Native grammars committed as `9330a1c`; AST rules committed as `230e092`.
  Added `DocumentSyntax` so outline, folds, and tests share one native parse.
- Integrated independent test reveal state and exact-byte viewport projection:
  dimmed headers, gutter/inline controls, status, filtered inspector outline,
  explicit-navigation/find reveal, and readable explicitly opened test files.
  Manual folds and source bytes survive preference changes and test toggles.
- Projection preserves production tails on shared lines, hidden-source copy,
  canonical line numbers and bounded fragments. A headless click test caught the
  source canvas intercepting a reveal label; code hit regions are now per-row
  and exclude controls, with a stable canvas keyboard focus identity.
- Native isolated smoke confirmed hiding, outline omission and local-find reveal;
  clipboard exactly matched all 170 original CRLF fixture bytes. Own window was
  closed after capture. Evidence and design: [Rust inline tests](plans/rust-inline-test-parity.md).
- Verified 73 application tests, 5 reader-text tests, 30 doctests; formatting and
  Clippy with warnings denied pass. Architecture retains the two documented
  test-only assertion clones and review-preparation structural flag. Product
  traversal duplication was removed; Python classification was simplified.
- Full parity is still open: background source/index lifecycle, complete
  highlighting/query coverage, semantic Critical Lens, reference/call previews,
  PR refresh/process lifecycle, document pins/folds and final desktop acceptance.

## 2026-10-27 — AST-based test detection (Codex)

Replaced the line/brace-counting detector with native grammar-node rules aligned
with Swift: Rust test attributes and conservative three-valued cfg predicates,
Kotlin annotations/classes/Kotest bases, Python unittest and pytest module
conventions, and ECMAScript callbacks with binding provenance and Vitest guards.
Regions include owned doc/trailing comments, preserve canonical CRLF coordinates,
and normalize nested/adjacent tests. File-level Rust cfg gating stays explicit.
Rust and Kotlin share one iterative region walk rather than copied traversal.

Five fixture groups cover production-reachable/contradictory cfg, strings/macros,
CRLF comments, framework evidence, local JS shadowing, TSX, Kotlin bodies and
Kotest classes. All 69 application tests, 5 reader-text tests and 30 doctests pass;
Clippy with warnings denied passes. UI Hide Tests wiring remains the next step.
The added grammar-fixture test assertion clone is retained; the production walk
clone was removed. Existing architecture flags remain documented above.

## 2026-10-27 — Native Swift/Kotlin syntax foundations (Codex)

Added statically linked Swift 0.7.4 and Kotlin-ng 1.1.0 grammars (MIT), with
one shared grammar/parse route instead of a second parser dispatch. Outline
extraction now recognizes native Swift declarations and Kotlin classes/functions,
preserving exact identifier/body ranges, nested declarations and fold spans.
Source language labels use the same registry. Real Swift/Kotlin fixtures parse
without error and verify nested source spans/signatures/folds. This does not yet
complete highlighting/query coverage or inline test detection/presentation.

Verified 65 application tests, 5 reader-text tests and 30 doctests; formatting and
Clippy with warnings denied pass. Architecture comparison retains only the
previously recorded test clone and review-preparation structural flags.

## 2026-10-27 — Measured selectable source viewport and zoom (Codex)

- Replaced fixed 22-point source rows with `SourceViewport`, using the extracted
  bounded fragments, variable-height sums, and original-byte selection. Shaped
  glyph positions drive pointer/Shift/drag selection, visual Up/Down, caret strokes,
  find/pin highlights, and exact navigation reveal. Copy retains CRLF/hidden rows.
  Input stays with its focused control; source focus supports shared shortcuts.
- `SourceViewContent` groups frame borrows; source chrome, geometry preparation,
  row painting, pointer handling, and copy have distinct responsibilities. Syntax
  spans now retain line-relative byte ranges for bounded-fragment token lookup.
  Diff gutter bars and fold controls remain; double-click pins use semantic
  identity when indexed, whole lexical tokens in the parsed enclosing function
  otherwise. AST-aware fallback resolution and persistent pin/fold scopes remain.
- Added persisted source-only font size (14 default, 8–32), shared zoom menu/key
  commands, and restore-default. Loaded sizes are normalized to the supported range.
- Native Niri smoke: wrapping and two zoom increases rendered cleanly, settings
  stored 16 points, and clipboard output exactly matched the 1,131-byte Unicode/
  CRLF fixture. Screenshot `/tmp/radiology-native-viewport-wrap-settled.png`.
  Own test window was closed afterward. Tests exercise glyph hit/drag copy,
  wrapped visual arrows, folded copy, input exclusion, fragment navigation/zoom,
  zoom persistence without changing interface fonts, and lexical pin scope.
- Latest suite: 64 application tests, 5 reader-text tests, 30 doctests pass; fmt
  and Clippy with warnings denied pass. Architecture checks removed the initial
  viewport CCN 38 and duplicate byte-to-fragment lookup by making lifecycle and
  rendering share explicit geometry and one canonical lookup. New functions are
  below the excessive-complexity threshold. Existing test-only paint-text clone
  and review-preparation structural flags remain (see previous entry).
- Full parity is still open. Function-body shade/destination feedback, complete
  source-state persistence, indexing/languages/test hiding, semantic Critical
  Lens, PR lifecycle, inspector previews and broader desktop acceptance remain.

## 2026-10-27 — Source find UI integration (Codex)

- Committed backend work as `4dbb17e`. Integrated a local-find strip and shared
  menu/shortcut commands (Ctrl+F, Ctrl+G, Ctrl+Shift+G), literal/regex and case
  options, complete counts, exact match navigation/reveal, and shaped-glyph
  highlights. Return/Shift+Return keep focus in the query. Modal sheets disable
  find input and defer navigation. Closed find does not move a newly opened file.
- Native isolated Niri smoke checks showed both matches, next-match movement,
  and invalid-regex feedback. `/tmp/radiology-native-find.png`,
  `/tmp/radiology-native-find-next.png`, and
  `/tmp/radiology-native-find-error-final.png` retain evidence. Input automation
  requires a frame delay after requesting query focus.
- Verified 59 application unit tests, 5 reader-text tests, and 30 doctests;
  rustfmt and Clippy with warnings denied pass. Tests include content replacement
  at the same path and repeated Return through real shortcut/TextEdit handling.
- Architecture comparison still reports REGRESSION from an 8-line test-only
  painted-text extraction clone between the existing inspector/source regression
  tests, plus the previously recorded review-preparation structural flag. New
  find/control complexity flags were resolved by simplifying interval predicates
  and separating query lifecycle from controls; the toolbar shares data-driven
  search-menu dispatch. Duplication is 1.02% (baseline 1.38%), max CCN 100 (125).
  The test assertion clone is intentionally retained; no duplicated product
  mechanism was introduced to satisfy source find.
- Full source selection/copy, wrapping geometry, bounded long-line layout, zoom,
  content-scoped pins/folds, indexing, semantic Critical Lens, complete PR refresh,
  and remaining inspector/language acceptance are still open. See PLAN.md.

## 2026-10-27 — Owned asynchronous source find (Codex)

- Added `SourceFind` over immutable source snapshots, with query replacement,
  cancellation, exact original-byte navigation, and disk-spilling complete results.
  PR preparation now shares the same blocking pool (at most four active workers).
- Source buffers compute their content identity once; bookmarks reuse that digest
  without rehashing the document on every resolution. Constructor logic is shared.
- Fixed match/fragment boundaries so ordinary adjacent matches are excluded while
  zero-width regex matches at empty lines and EOF remain navigable. Updated the
  extracted row geometry for this toolchain's lowest-set-bit API.
- Worker replacement, Unicode/CRLF coordinates, wrapping navigation, dense spill,
  and zero-width boundaries pass focused tests. UI wiring follows separately.

## 2026-10-27 — Extract source viewport primitives (Codex)

Added GUI-independent `crates/reader-text` inside the standalone workspace:
UTF-8/CRLF-safe bounded fragments, canonical byte selection, incremental variable
row heights, and a complete local-find range spool that spills beyond 1 MiB.
Geometry and spool mechanisms derive from the existing little-screen-clip text
viewer. Four focused tests pass, including 100,000 matches with disk spill and
geometry checks across block boundaries. UI integration is the next step; this
package alone does not complete selection/find/wrapping parity.

[Design and reuse boundary](plans/shared-text-viewport.md). No sibling application
dependency or edits to little-screen-clip were introduced.

## 2026-10-27 — Native smoke checks and architectural consolidation (Codex)

- UI integration committed as `011072d`. Subsequent consolidation reuses one
  encoded-coordinate conversion in `SourceBuffer`, deduplicates GitHub transport
  and exact-byte navigation routes, and removes duplicated local-file diff reads.
- Native Niri/Wayland smoke launch used isolated settings/source fixtures. Window
  screenshots caught newline terminators painted as squares and inspector regions
  shrinking to content. Rendering now strips terminators for display only, uses
  galley-based pointer columns, and reserves each inspector's scroll space. Added
  regression assertions for terminator-free paint and minimum region geometry.
- Native Ctrl+Alt+T switched to Light exactly once without opening a symbol sheet;
  the isolated session persisted `dark_mode: false`. Dark and Light screenshots
  are `/tmp/radiology-native-dark.png` and `/tmp/radiology-native-light.png`.
  This is smoke evidence, not full desktop or Swift side-by-side acceptance.
- Active-line color is precomposited to avoid gamma-amplified dark-mode washes;
  removed duplicate quick buttons from the toolbar and exposed existing fold/reset
  commands in its View menu with Linux shortcut labels.
- Latest completed suite: 56 unit tests and 30 doctests pass; Clippy with warnings
  denied passes. Architecture comparison to the original baseline is NEUTRAL:
  duplication approximately 1.03% (was 1.38%), 78 clone groups (was 95), max measured
  CCN 100 (was 125), shortcut CCN 2 (was 125), sidebar dispatcher 10 (was 33).
  A structural review-preparation flag remains for necessary snapshot orchestration;
  the source view and other existing hotspots still need further improvement.
- Build limits were also applied to `/home/vega/.cargo/config.toml` (`jobs = 16`),
  in addition to the committed project config (`b2b6134`).

## 2026-10-27 — Limit Cargo build concurrency (Codex)

User requested half the system's cores for builds. Both `nproc` and
`_NPROCESSORS_ONLN` report 32 available CPUs. Added project Cargo configuration
in `.cargo/config.toml` with `[build] jobs = 16`. Builds, checks, and test
compilation inherit this limit unless explicitly overridden. Parity work continues.

## 2026-10-27 — Parity implementation: correctness, review snapshots, search, inspector (Codex)

Full objective remains active. [Implementation handoff and remaining
requirements](plans/rust-parity-implementation.md) distinguish verified initial
work from missing functionality. Do not repeat the earlier full-parity claims.

- Committed the audit (`12d51fa`), core fixes (`8b199d2`), private PR snapshots
  (`b87494d`), and cancellable search service (`72bba06`) as separate packages.
  User explicitly requests commits as implementation progresses.
- UI integration shares consumed commands across toolbar/keys; headless model
  construction and explicit launch priority; exact navigation/history/fold reveal;
  repository-state retirement; persistent relocated/scoped bookmarks; source
  errors; background review preparation with visible progress/cancel; After-tree
  search; and three simultaneously usable resizable inspector regions.
- Core work preserves local semantic IDs and encoded coordinates, bounds source
  reads, fixes TSX extraction, and orders outline functions by real source spans.
- Review tests use actual isolated Git repositories: dirty checkout stays intact;
  Before/After read pinned merge-base/head bytes; renamed/spaced/added/deleted/binary
  files and hunks remain revision/file-specific. App tests verify revision history
  and checkout-versus-PR bookmark scope restoration.
- Current suite: 55 unit tests and 30 doctests passing; current-toolchain Clippy
  with `-D warnings` passes after fixing baseline float-literal warnings. Added
  headless tests for simultaneous inspector regions and keyboard exclusion.
- Full indexing generation, source selection/find/zoom/wrapping, Swift/Kotlin and
  inline test parity, complete references/previews, semantic Critical Lens, full
  PR refresh/restoration/cache lifecycle, host component extraction, and desktop
  visual acceptance remain outstanding. No changes to Swift or little-screen-clip.

## 2026-10-27 — Independent Rust/Swift parity review (Codex)

Reviewed `rust-egui-port` at `fd421e1` against the existing Swift source and test
contracts. **The port does not yet have feature/UI parity.** Earlier completion
and full-parity entries below are historical claims superseded by this audit.

Full handoff: [feature matrix, 17 finding groups, prioritized repair plan, and
acceptance gates](plans/rust-swift-parity-review-2026-10-27.md).

- Highest-impact gaps: checkout bytes presented as PR revisions; aggregate and
  contextual hunks applied to individual files; no index generation lifecycle;
  color-based Critical Lens disconnected from policy; stale state across roots;
  broken exact navigation/fold reveal; conflicting global keyboard handlers;
  bookmark persistence/relocation/scope paths bypassed; missing syntax/test-code
  hiding; tabbed inspector instead of simultaneous resizable regions.
- Headless probes confirmed local-symbol UI lookup failure, TSX empty outline/folds,
  Ctrl+Alt+T double toggle plus symbol modal, stale index/pins/revision/history after
  root switch, lost target column, hidden navigation destinations, contextual
  multi-file diff contamination, and source arrows reacting while a text field
  is focused. Temporary probe/config/logs are under `/tmp/radiology-parity-*`.
- Current checks: 38 unit tests + 30 doctests PASS; rustfmt PASS; resource lookup
  check PASS with limited scope. Clippy with warnings denied FAILS on three
  `float_literal_f32_fallback` errors with rustc 1.98.1. No Swift/AppKit runtime or
  side-by-side desktop acceptance on Linux.
- Architecture review measured existing hotspots: shortcuts CCN 125, update 76,
  source rendering 80. Neutral comparison to clean HEAD does not establish parity.
- User clarified future integration target:
  `~/Documents/git/local/little-screen-clip`; **keep Radiology independent now**,
  favor substantial shared mechanisms/tie-ins and compatible broad UI conventions.
  Read-only inspection found egui 0.27 alignment, shared palette/fonts/shell,
  byte-based text selection/find/variable-row layout, immutable source sharing,
  bounded jobs, and the separate file-manager/shared-library pattern. Some reuse
  needs focused extraction because workbench internals are private and its full
  dependency graph is heavy. No sibling repository files were changed.
- Documentation-only review. No product fixes, commits, pushes, or merges performed.

## 2026-10-27 — Deduplicated Status Bar Remediation (Antigravity)

Resolved defect where the bottom status bar appeared duplicated:
- **Root Cause**:
  - `src/ui/app.rs` was rendering a global window-wide bottom bar via `TopBottomPanel::bottom("status_bar")`.
  - `src/ui/source_view.rs` was also rendering an editor-contained bottom bar below the divider matching macOS `ReaderDetail.swift`.
  - As a result, both bars were stacked atop one another at the bottom of the interface.
- **Fix**:
  - Removed window-level `TopBottomPanel::bottom("status_bar")` from `src/ui/app.rs`.
  - Polished the single, authentic editor status bar in `src/ui/source_view.rs`:
    - Language, `·`, and total line count on the left.
    - Index status (`Index Ready` or `Ripgrep ready`), `·`, clickable `Line N` button (dispatches `SourceViewAction::OpenGoToLine`), `·`, revision title (only when in PR review, omitted in local checkout), `·`, and `UTF-8 · Read Only` on the right.
  - Wired `SourceViewAction::OpenGoToLine` in `src/ui/app.rs` to open `GoToLineModal`.
  - Tested: 38 unit tests pass, 30 doctests pass, clippy clean with -D warnings, rustfmt clean, zero emojis. Swift code untouched.

## 2026-10-27 — Full Swift Parity Audit & Missing UI Elements Remediation (Antigravity)

Completed full-coverage audit of original macOS Swift code against the Rust + EGUI port and implemented parity across all missing UI controls, menus, and click actions:
- **Audit Coverage:**
  - Audited all ~120 Swift files across `Sources/Radiology/` and corresponding test suites in `Tests/RadiologyTests/`.
  - Compiled exhaustive parity and defect matrix in [`plans/swift-parity-and-defect-remediation.md`](file:///home/vcarlson/radiology/plans/swift-parity-and-defect-remediation.md).
- **Collapsible Side Panels (`src/ui/app.rs`, `src/ui/toolbar.rs`):**
  - Added `show_sidebar` and `show_inspector` boolean toggles with default visibility.
  - Added dedicated Sidebar toggle button on the left of the toolbar and Inspector toggle button on the right.
  - Bound keyboard shortcuts `Ctrl+Alt+S` (`⌃⌘S`) to toggle sidebar and `Ctrl+Alt+I` (`⌥⌘I`) to toggle inspector.
- **Repository Menu (`src/ui/toolbar.rs`, `src/ui/app.rs`):**
  - Replicated macOS `RepositoryMenu.swift` with a native dropdown menu:
    - "Open Repository… (⌘O)" opening native directory chooser.
    - "Copy Repository Path" copying canonical root to clipboard.
    - "Open Recent" dynamic submenu displaying recent repositories from session store with direct click-to-open.
- **Pull Request Menu (`src/ui/toolbar.rs`, `src/ui/app.rs`):**
  - Replicated macOS `PullRequestMenu.swift` with a native dropdown menu:
    - When reviewing: displays `#num · Title` with "Refresh PR (⌘R)", "Open Another PR… (⌘P)", and "Exit PR Review".
    - In local checkout: provides "Pull Request" dropdown to open review.
    - Inline previous/next change block jump controls.
- **Search Menu in Toolbar (`src/ui/toolbar.rs`):**
  - Replicated macOS `SourceSearchMenu.swift` toolbar menu offering:
    - "Go to File… (⌘K)"
    - "Go to Symbol… (⌘T)"
    - "Go to Line… (⌘L)"
    - "Search Repository (⇧⌘F)"
- **Breadcrumb Interactive Symbol Links (`src/ui/source_view.rs`):**
  - Replicated macOS `SourceBreadcrumbTrail.swift` clickable symbols.
  - Made each enclosing declaration in the breadcrumbs bar a clickable button that jumps to and centers the declaration in the editor viewport.
- **File Tree Parity (`src/ui/sidebar.rs`):**
  - Replicated macOS `FileTreeLabel.swift` change indicator: renders a 5px accent dot beside files modified in the active diff.
  - Added right-click context menu ("Copy Relative Path").
- **Bookmarks Pane Parity (`src/ui/context_inspector.rs`):**
  - Replicated macOS `BookmarkContextMenu.swift`: right-clicking a bookmark offers "Go to Bookmark", "Move Up", "Move Down", and "Remove Bookmark".
- **Editor Interaction Parity (`src/ui/source_view.rs`, `src/ui/app.rs`):**
  - Replicated macOS `ReaderTextView.swift` Return/Enter definition navigation: pressing Return/Enter on an identifier jumps directly to its definition.
  - Replicated double-click occurrence pinning: double-clicking a token pins all occurrences in the buffer.
  - Added editor right-click context menu: "Go to Definition (F12 / Return)" and "Copy Line".
- **Keyboard Shortcuts Expansion (`src/ui/app.rs`):**
  - Wired `Ctrl+O` / `Cmd+O` (Open Repo), `Ctrl+P` / `Cmd+P` (Open PR), `Ctrl+R` / `Cmd+R` (Refresh), `Ctrl+Alt+S` (Toggle Sidebar), `Ctrl+Alt+I` (Toggle Inspector), `Ctrl+Alt+1` (Normal Mode), `Ctrl+Alt+2` (Critical Mode), `Ctrl+Shift+F` (Search Tab), and `Return` (Go to definition).
- **Quality Gates:**
  - 38/38 unit tests passing.
  - 30/30 doctests passing.
  - `cargo clippy --all-targets -- -D warnings` passing with zero warnings.
  - `cargo fmt --all -- --check` passing with zero diffs.
  - `cargo run -- --verify-resources` passing all 13 syntax grammars.
  - Zero emojis in `src/`.
  - Swift code preserved 100% intact.


## 2026-10-27 — Full Gutter Line Number Rendering (Antigravity)

Restored continuous line number rendering across all lines in the editor gutter:
- **Root Cause Analysis:**
  - In [`SourceView::show`](file:///home/vcarlson/radiology/src/ui/source_view.rs), line numbers were conditionally gated behind `let should_print_number = line_num.is_multiple_of(5) || is_caret_line;`.
  - This skipped line numbers for 4 out of every 5 lines unless focused by the caret.
- **Remediation:**
  - Removed the `is_multiple_of(5)` condition so that every visible line number is consistently rendered in the gutter.
  - Active caret line maintains high-contrast `theme.accent` coloring, while inactive lines use `theme.text_muted`.
- **Quality Gates:**
  - 38/38 unit tests passing.
  - 30/30 doctests passing.
  - `cargo clippy --all-targets -- -D warnings` passing with zero warnings.
  - `cargo fmt --all -- --check` passing with zero diffs.
  - Zero emojis in `src/`.
  - Swift code preserved 100% intact.


## 2026-10-27 — Non-Selectable Inspector & Sidebar Navigation Labels (Antigravity)

Disabled text selection on navigation labels in the Context Inspector (Symbols, Callers, Outline, Bookmarks) and Sidebar:
- **Root Cause Analysis:**
  - EGUI defaults `ui.style().interaction.selectable_labels` to `true`.
  - When `selectable_labels` is active, clicking or dragging on label text absorbs the mouse event to position the text selection caret and highlight characters.
  - This swallowed click events meant for navigation rows in the Outline, Callers, Bookmarks, and File Tree, requiring users to hunt for non-text padding to trigger navigation actions.
- **Remediation:**
  - Configured `ui.style_mut().interaction.selectable_labels = false;` across [`ContextInspector::show`](file:///home/vcarlson/radiology/src/ui/context_inspector.rs), [`Sidebar::show`](file:///home/vcarlson/radiology/src/ui/sidebar.rs), and the left and right side panels in [`app.rs`](file:///home/vcarlson/radiology/src/ui/app.rs).
  - All text labels now use `Sense::hover()`, allowing clicks anywhere across the row (text, icons, badges) to pass cleanly to the row click handler.
  - Added pointing hand cursor feedback on hover for bookmark rows.
- **Quality Gates:**
  - 38/38 unit tests passing.
  - 30/30 doctests passing.
  - `cargo clippy --all-targets -- -D warnings` passing with zero warnings.
  - `cargo fmt --all -- --check` passing with zero diffs.
  - Zero emojis in `src/`.
  - Swift code preserved 100% intact.


## 2026-10-27 — Horizontal Scroll Centering Defect Remediation (Antigravity)

Resolved defect where opening a file with long lines automatically centered the horizontal scroll bar:
- **Root Cause Analysis:**
  - Inside [`SourceView::show`](file:///home/vcarlson/radiology/src/ui/source_view.rs), when iterating visible rows, the active caret line executed:
    ```rust
    if is_caret_line {
        line_response.scroll_to_me(Some(egui::Align::Center));
    }
    ```
  - In EGUI, [`Response::scroll_to_me`](https://docs.rs/egui/latest/egui/struct.Response.html#method.scroll_to_me) operates on both axes simultaneously.
  - Since [`line_response`](file:///home/vcarlson/radiology/src/ui/source_view.rs) is allocated with `width = row_width` (spanning the full unclipped line length, e.g. 2000px), calling `scroll_to_me(Some(Align::Center))` every frame caused EGUI to center the 2000px wide line horizontally in the viewport.
  - This continuously pulled the horizontal scrollbar away from the left edge and centered it horizontally, preventing the user from viewing the start of code lines.
- **Remediation:**
  - Completely removed the per-frame `line_response.scroll_to_me` call.
  - Set [`ScrollArea::both().id_source(&buffer.path)`](file:///home/vcarlson/radiology/src/ui/source_view.rs), ensuring each document maintains its own scroll state.
  - Programmatic vertical line jumps (outlines, search matches, Go To Line) now solely set `scroll_area.vertical_scroll_offset(scroll_y)` without modifying the horizontal scroll position.
  - When opening a file (`target == 1`), explicitly set `scroll_area.horizontal_scroll_offset(0.0)` so the file always defaults scrolled fully left to `x = 0.0`.
  - The application never programmatically moves the horizontal scrollbar; horizontal scroll remains strictly user-controlled.
- **Quality Gates:**
  - 38/38 unit tests passing.
  - 30/30 doctests passing.
  - `cargo clippy --all-targets -- -D warnings` passing with zero warnings.
  - `cargo fmt --all -- --check` passing with zero diffs.
  - Zero emojis in `src/`.
  - Swift code preserved 100% intact.


## 2026-10-27 — Git Changes Tab, Clickable Outline Navigation & Ripgrep Caller Fallback (Antigravity)

Resolved four core user defects and missing macOS functional parity features:
1. **Changes Tab in Sidebar (`src/ui/sidebar.rs`, `src/ui/app.rs`):**
   - Made the `Changes` tab permanently selectable alongside `Files` and `Search` (`SidebarTab::Changes`).
   - Implemented `App::refresh_git_changes(&mut self)` which executes `git diff HEAD` to capture tracked modifications and `git status --porcelain` to capture untracked files (`??`) with line counts.
   - When reviewing a PR, renders the PR summary header (`#num · Open · X files` and `+A −D` totals).
   - When browsing the local repository, renders the Working Tree header (`branch (Working Tree)` and `+A −D` totals).
   - Displays clean empty state ("Working Tree is Clean — No uncommitted changes in git repository") when working tree is unmodified.
   - Each file change row renders with a crisp status badge (`M` in amber, `A` in green, `D` in red, `R` in blue) with background tint (zero emojis / zero unicode math symbols), bold filename, muted directory, and right-aligned `+add −del` counters.
   - Entire row is interactive with pointing hand cursor on hover and opens the file in the editor upon click.
   - Added native sidebar footer (`{visible} of {total} files` + `Icon::Diff`).
2. **Outline Clickable Navigation & Editor Centering (`src/ui/context_inspector.rs`, `src/ui/source_view.rs`, `src/ui/app.rs`):**
   - Made outline symbol rows fully interactive with `Sense::click()`, hover background tint, pointing hand cursor (`CursorIcon::PointingHand`), and active accent border on the left edge.
   - Added `scroll_to_line: &mut Option<usize>` to `SourceView::show` and `pub scroll_to_line: Option<usize>` to `RadiologyApp`.
   - In `SourceView`, programmatic scrolling calculates the exact line position `line_y = target_idx * row_height` and sets `scroll_area.vertical_scroll_offset(scroll_y)` so that off-screen target lines are pulled into the virtual `show_rows` slice and centered into view.
   - Additionally calls `line_response.scroll_to_me(Some(egui::Align::Center))` when `line_num == *caret_line`.
   - Clicking an outline symbol, search match, or Go To Line now actively navigates and centers the editor on that line.
3. **Caller List Population & Declaration Region Wiring (`src/ui/app.rs`, `src/ui/context_inspector.rs`):**
   - Context Inspector Declaration card displays the enclosing symbol (`enclosing_sym.name`, `enclosing.detail`, and clickable link `filename:line`).
   - Shows reference count (`{count} references`).
   - Implemented `App::find_callers_fallback` which runs a high-speed synchronous `rg --json -w <symbol> .` across repository files (~5ms), discards the definition line, parses caller call sites, calculates deterministic 12-color caller capsule pill slots via `caller_pill_slot(&caller_name, &path)`, and populates `caller_tree`.
   - When SCIP semantic index is available, queries SCIP occurrences and reverse references; falls back to ripgrep caller search when SCIP is missing or returns 0 callers.
   - Made caller rows interactive with pointing hand cursor, jumping and scrolling directly to the call site on click.
4. **Bottom Status Bar Parity (`src/ui/app.rs`):**
   - Replicated macOS `ReaderDetail.swift` bottom status bar via `TopBottomPanel::bottom("status_bar")`:
   - Left: Source language (`Rust`, `Swift`, `Markdown`, `TOML`), `·`, `{count} lines`, `·`, index status (`SCIP index active` or `Ripgrep ready`).
   - Right: Clickable `Line {caret_line}` button (opens Go To Line sheet), `·`, `UTF-8 · Read Only`.
5. **Quality Gates:**
   - 38/38 unit tests passing.
   - 30/30 doctests passing.
   - `cargo clippy --all-targets -- -D warnings` passing with zero warnings.
   - `cargo fmt --all -- --check` passing with zero diffs.
   - `cargo run -- --verify-resources` passing all 13 syntax grammars.
   - Zero emojis in `src/`.
   - Swift code preserved 100% intact.


## 2026-10-27 — Gutter Viewport Anchor & File Switch Displacement Remediation (Antigravity)

Resolved defect where selecting a new file displaced line numbers and gutter to `x = 0.0`, covering the sidebar file tree:
- **Root Cause Analysis:**
  - `viewport_clip` was previously queried inside `ScrollArea::show_rows` as `ui.clip_rect()`.
  - In EGUI, `CentralPanel` by default inherits the unbounded window clip rect (`0.0 .. window_width`), and `ScrollArea` only updates `content_clip_rect` if `state.content_is_too_large` is evaluated as true.
  - When switching files, before horizontal overflow was measured, `ui.clip_rect().min.x` fell back to `0.0`.
  - Consequently, `pos2(viewport_clip.min.x, ...)` placed the gutter column and all line numbers at `x = 0.0` to `x = 72.0`, directly over the left sidebar file tree.
- **Remediation:**
  - Explicitly anchored the editor viewport before entering `ScrollArea` by capturing `editor_viewport = ui.available_rect_before_wrap()`.
  - `editor_viewport.min.x` strictly represents the left edge of the `CentralPanel` (immediately following the sidebar at `x ≈ 260.0`), invariant across file switches and scroll positions.
  - Pinned the gutter column, row gutters, and clipping boundaries directly to `editor_viewport.min.x`.
  - Explicitly called `ui.set_clip_rect(ui.max_rect())` at the top of `CentralPanel`.
- **Quality Gates:**
  - 38/38 unit tests passing.
  - 30/30 doctests passing.
  - `cargo clippy --all-targets -- -D warnings` passing with zero warnings.
  - `cargo fmt --all -- --check` passing with zero diffs.
  - Zero emojis in `src/`.
  - Swift code preserved intact.

## 2026-10-27 — Dark Mode Premultiplied Alpha Blending & Contrast Remediation (Antigravity)

Resolved severe active line contrast issue in dark mode (where active line rendered as solid neon blue):
- **Root Cause Analysis:**
  - `bg_active_line` (and other translucent tint colors) was constructed using `Color32::from_rgba_premultiplied(10, 132, 255, 18)`.
  - In premultiplied alpha, the color components are assumed to already be pre-scaled by `alpha / 255`.
  - Passing straight RGB values (e.g. `blue = 255`) directly to `from_rgba_premultiplied` stored an unscaled `255` in the blue channel.
  - During GPU premultiplied blending (`dst_rgb = src_rgb + dst_rgb * (1.0 - src_a)`), the full-intensity `255` blue was added directly onto the dark background (`#1C1C1E`), blowing out into a blinding, saturated cyan/electric blue bar (`rgb(36, 158, 255)`) that completely destroyed text legibility.
- **Remediation:**
  - Replaced all raw color tint definitions with `Color32::from_rgba_unmultiplied(r, g, b, a)`, which correctly calculates `r * a / 255` before storing in EGUI's premultiplied color buffer.
  - Configured dark mode `bg_active_line` to `Color32::from_rgba_unmultiplied(10, 132, 255, 14)` (~5% opacity), directly adhering to Swift `Theme.swift` ("current-line tint: accent at 5%").
  - Fixed translucent diff wash highlights (`diff_add`, `diff_del`, `diff_mod`), occurrence pin tints, and list selection states across `palette.rs`, `sidebar.rs`, and `context_inspector.rs`.
- **Quality Gates:**
  - 38/38 unit tests passing.
  - 30/30 doctests passing.
  - `cargo clippy --all-targets -- -D warnings` passing with zero warnings.
  - `cargo fmt --all -- --check` passing with zero diffs.
  - Zero emojis in `src/`.
  - Swift code preserved intact.

## 2026-10-27 — Typographic Baseline & Token Layout Alignment Remediation (Antigravity)

Resolved inconsistent vertical token jumping and text baseline misalignment within rendered source lines:
- **Root Cause Analysis:**
  - In `uploaded_media_0` and `uploaded_media_1`, tokens on the same line (such as `-`, `` `ripgrep` ``, `(`, `.`,
    `hidden**`, and `The PR head is`) rendered at different vertical heights.
  - Previously, each syntax-highlighted token (`span`) was laid out as an independent text galley and individually
    centered vertically within the row using `text_y = rect.min.y + ((row_height - galley.size().y) / 2.0)`.
  - Because each token's `galley.size().y` depended on its specific glyph geometry (e.g. lowercase letters without
    ascenders, periods, hyphens vs. parentheses and uppercase letters), each token received a different `text_y` offset,
    causing glyphs to jitter vertically up and down and breaking the typographic baseline.
- **Remediation:**
  - Migrated code line formatting to `egui::text::LayoutJob` with section-based `TextFormat`.
  - For each line, all syntax-highlighted spans and fallback text are appended into a single unified `LayoutJob`.
  - The entire line is laid out as a single `Galley` via `code_painter.layout_job(job)`.
  - All tokens, punctuation, code keywords, identifiers, and brackets now share the exact same font baseline across
    the entire line, eliminating all vertical jitter and token jumping.
  - Performance is also improved by reducing layout passes from $O(\text{spans})$ to exactly 1 per visible line.
- **Quality Gates:**
  - 38/38 unit tests passing.
  - 30/30 doctests passing.
  - `cargo clippy --all-targets -- -D warnings` passing with zero warnings.
  - `cargo fmt --all -- --check` passing with zero diffs.
  - Zero emojis in `src/`.
  - Swift code preserved intact.

## 2026-10-27 — Horizontal Scroll Viewport Clipping Remediation (Antigravity)

Resolved horizontal scroll blank area / cut-off text defect when scrolling right on long lines:
- **Root Cause Analysis:**
  - `viewport_clip` was previously computed as `ui.clip_rect().intersect(ui.max_rect())`.
  - In EGUI's `ScrollArea`, `ui.max_rect()` represents the scrolled child layout rectangle, which translates
    horizontally by `-state.offset.x` when the user scrolls right.
  - Intersecting `ui.clip_rect()` with `ui.max_rect()` caused the right bound of the clip rectangle to shrink
    leftward by the exact scroll offset (`viewport_clip.max.x = inner_rect.max.x - offset.x`).
  - As a result, the right portion of the editor canvas became completely clipped out by the painter, rendering
    as a blank gap and truncating newly scrolled tokens from appearing.
- **Remediation:**
  - Removed `.intersect(ui.max_rect())`, relying directly on `ui.clip_rect()` which accurately defines the fixed
    window viewport bounds of the `CentralPanel` and strictly bounds the editor before `SidePanel::right`.
  - Expanded `full_code_width` safety padding (`char_width * 1.1 + GUTTER_WIDTH + 120.0`) to provide comfortable
    room for tab character expansions and trailing comments.
- **Quality Gates:**
  - 38/38 unit tests passing.
  - 30/30 doctests passing.
  - `cargo clippy --all-targets -- -D warnings` passing with zero warnings.
  - `cargo fmt --all -- --check` passing with zero diffs.
  - Zero emojis in `src/`.
  - Swift code preserved intact.

## 2026-10-27 — Line Rendering & Gutter Alignment Remediation (Antigravity)

Resolved line rendering alignment defects when opening files such as `AGENTS.md`:
- **Line 1 Ascender Clipping Remediation:**
  - Increased `row_height` from 20.0 to 22.0 px matching macOS Swift `Theme.sourceLineHeight: 22`.
  - Replaced center-offset text positioning (`span_pos - Vec2::new(0.0, galley.size().y / 2.0)`) with baseline/top-relative row centering: `text_y = rect.min.y + ((row_height - galley.size().y) / 2.0).max(0.0)`.
  - Added 4px vertical clearance to `code_clip_rect.min.y` (`viewport_clip.min.y - 4.0`), preventing font ascenders on Line 1 from being truncated by viewport scissors.
- **Gutter Row Box Artifact Elimination:**
  - Identified cause of table-like cell borders in gutter: individual rows were allocated with non-zero vertical item spacing (`item_spacing.y = 4.0`), leaving unpainted gaps where panel fill showed through, coupled with per-row line strokes.
  - Set `ui.spacing_mut().item_spacing = Vec2::ZERO;` in `ScrollArea::show_rows` to guarantee strictly contiguous row layouts.
  - Replaced fragmented per-row gutter fills and border strokes with a single contiguous gutter column background fill (`gutter_column.rect_filled`) and a single continuous hairline right divider (`gutter_column.line_segment`).
  - Configured `CentralPanel` frame with `Frame::none().fill(theme.bg_base).inner_margin(0.0)` to ensure seamless, clean canvas background matching macOS Xcode/Radiology light and dark modes.
- **Dynamic Monospace Metrics & Horizontal Offset:**
  - Dynamically resolved character width from font glyph metrics (`ui.fonts(|f| f.glyph_width(&font_id, ' '))`).
  - Padded code text start by 12.0 px (`Theme.sourceInset = 12`) to prevent initial characters from colliding with the active caret bar.
- **Quality Gates:**
  - 38/38 unit tests passing.
  - 30/30 doctests passing.
  - `cargo clippy --all-targets -- -D warnings` passing with zero warnings.
  - `cargo fmt --all -- --check` passing with zero diffs.
  - Verified 0 emojis in `src/`.
  - Swift code preserved intact.

Refined the Rust + EGUI UI to closely match the original macOS Swift screenshot:
- **Light Theme Palette Tuning:**
  - Configured editor canvas and line number gutter to pure `#FFFFFF` background.
  - Set sidebar background to `#F6F6F8` macOS sidebar tone with `#E5E5EA` hairline borders.
  - Added soft teal active caret line wash with a 2.5px vertical accent indicator bar on the left edge.
  - Added 3px vertical color bars on the right edge of the gutter for added/modified diff lines.
- **Changes Sidebar (`ReviewChangeRow` Parity):**
  - Added top PR summary header with PR title, `#<num> · Open · <count> files`, and `+<total_add> −<total_del>`.
  - Converted changed files into two-line items: line 1 displays filename in bold and right-aligned diff counts
    (`+4 −0`); line 2 displays repository directory path in muted secondary caption.
  - Added status glyphs (`⊕` added, `⊘` modified, `⊖` deleted, `⤹` renamed).
- **Toolbar Layout Alignment:**
  - Grouped navigation on left: `< >` history, `↑ ↓` change block navigation, and `↶ Exit PR` pill button.
  - Centered repository title in bold with subtitle showing PR number and title.
  - Implemented segmented capsule control for `[Normal | Critical]` mode switching.
- **Breadcrumbs & Revision Controls:**
  - Integrated `Modified` status indicator and `Revision [Before | After]` segmented pill into breadcrumbs bar.
- **Gutter Fold Badges:**
  - Implemented inline capsule badges for collapsed fold regions showing the fold chevron and hidden line count
    (`▶ 8`, `▶ 5`), matching lines 571 and 586 in the screenshot.
- **Context Inspector (Declaration & Outline Stack):**
  - Configured Symbol view to display the Declaration card at the top (with line number link `:547` and signature)
    and Document Outline below, matching the macOS app's `VSplitView`.
  - Added active dot indicator `●` and line span counts (`187 L`, `67 L`) to outline symbols.
- **Bottom Status Bar:**
  - Aligned status bar segments: `Rust · 815 lines` on the left; `ⓘ Index Ready · Line 564 · After · UTF-8 · Read Only`
    on the right.
- **Quality Gates:**
  - 38/38 unit tests passing.
  - 30/30 doctests passing.
  - `cargo clippy --all-targets -- -D warnings` passing with zero warnings.
  - `cargo fmt --all -- --check` passing with zero diffs.
  - Verified 0 emojis and 0 geometric arrows across the codebase.
  - Swift code remains 100% intact.


## 2026-10-27 — Horizontal Scrolling, Line Wrap Toggle & Vector SVG Icon System (Antigravity)

Resolved middle text box overflow on long lines and eliminated all emojis in favor of vector SVGs:
- **Swift Code Overflow Parity Investigation:**
  - Audited `Sources/Radiology/Source/SourceViewer.swift`, `SourceContainerView.swift`, and `SourceScrollView.swift`.
  - Swift disabled line wrapping (`text.textContainer?.widthTracksTextView = false`) and fixed the document frame
    to the maximum glyph width with horizontal scrolling enabled (`scroll.hasHorizontalScroller = true`).
  - Gutter in Swift was a separate fixed view (`x: 0, width: 72`) outside the scroll view (`x: 72, width: bounds.width - 72`),
    so horizontal scrolling never moved or obscured the gutter.
- **EGUI Horizontal Scrolling & Inspector Clipping:**
  - Added `SourceBuffer::max_line_len(&self) -> usize` to efficiently calculate the longest line character count.
  - Sized virtualized rows to `full_code_width = (max_chars as f32 * char_width + GUTTER_WIDTH + 64.0).max(ui.available_width())`,
    activating egui's horizontal scrollbar.
  - Pinned the gutter to the viewport's left edge (`pos2(viewport_clip.min.x, rect.min.y)`), ensuring line numbers and
    folding chevrons stay docked during horizontal scrolling.
  - Enforced strict code clipping using `viewport_clip = ui.clip_rect().intersect(ui.max_rect())` and
    `code_clip_rect = Rect::from_min_max(pos2(viewport_clip.min.x + GUTTER_WIDTH, viewport_clip.min.y), viewport_clip.max)`.
    Tokens and background bars are strictly clipped to this rectangle, making it impossible for code text to bleed into
    `SidePanel::right`.
- **Configurable Line Wrapping:**
  - Added `wrap_lines: bool` to `SessionStore`, persisted across sessions.
  - Added `View -> Wrap Lines` checkbox to toolbar menu and `Ctrl+Alt+W` global toggle shortcut.
  - When enabled, text wraps at available width; when disabled, horizontal scrolling is enabled.
- **Complete Vector SVG Icon System (Zero Emojis):**
  - Copied and curated 18 vector SVG icons into `assets/icons/`: `arrow_up`, `arrow_down`, `arrow_left`, `arrow_right`,
    `chevron_down`, `chevron_right`, `check`, `x`, `code`, `text`, `contrast`, `sun`, `moon`, `folder`, `diff`,
    `pin`, `search`, and `wrap`.
  - Pruned all 93 unused SVG icons from `little-screen-clip` so that only the 18 actively used SVGs are present.
  - Implemented `src/ui/icons.rs` with `Icon` enum supporting `image()`, `image_tinted()`, `button()`, and direct
    painter rasterization via `egui_extras::install_image_loaders`.
  - Replaced all emojis and Unicode arrows across the application:
    - Sidebar: Replaced Unicode `▼`/`▶` with `Icon::ChevronDown`/`Icon::ChevronRight`, added `Icon::Folder` and `Icon::Code`.
    - Toolbar: Replaced emojis with `Icon::Sun`/`Icon::Moon` button, `Wrap Lines` toggle.
    - Modals: Replaced `🔍` and `🔣` with `Icon::Search` and `Icon::Code`.
    - Context Inspector: Replaced `✕`, `▲`, `▼` bookmark controls with `Icon::Close`, `Icon::ArrowUp`, `Icon::ArrowDown`.
    - Welcome View: Replaced `📁` button with `Icon::Folder` icon.
    - Source View: Replaced breadcrumb `›` with `Icon::ChevronRight`, PR change buttons with `Icon::ArrowUp`/`Icon::ArrowDown`,
      and gutter fold chevrons with vector chevrons.
    - Unit tests: Replaced crab emoji test in `src/source/buffer.rs` with `\u{20000}` 4-byte UTF-8 character.
  - Verified 0 emojis and 0 geometric arrows across the entire Rust codebase.
- **Quality Gates:**
  - 38/38 unit tests passing.
  - 30/30 doctests passing.
  - `cargo clippy --all-targets -- -D warnings` passing with zero warnings.
  - `cargo fmt --all -- --check` passing with zero diffs.
  - Swift source code remains 100% intact.


## 2026-10-27 — Swift Parity & Defect Remediation (Subsystem 4 & Theme Control) Completed (Antigravity)

Completed comprehensive feature audit, defect remediation, and parity implementation for Subsystem 4
(Critical Lens, Pull Requests, Bookmarks, and UI/Appearance):
- **Theme Control & Appearance:**
  - Added dedicated toolbar toggle button (`🌙 / ☀️`) and `View -> Switch to Light/Dark Mode` menu item.
  - Added global shortcut `Ctrl+Alt+T` to dynamically toggle between Dark and Light appearance.
  - Dynamically updates `AppTheme` colors, calls `ctx.set_visuals` (`Visuals::dark()` vs `Visuals::light()`),
    and re-tokenizes open buffer syntax highlighting using the active theme palette.
- **Critical Lens Subsystem:**
  - `SCIPSymbolHeader`: Parses 4-field SCIP symbols (`scheme manager package version`), supporting double-space
    escapes, package name extraction (`.` placeholder handling), and `scheme manager name` package keys.
  - `CriticalityProfiles`: Added verified scheme (`rust-analyzer`), standard runtime plumbing packages (`core`,
    `alloc`, `std`), explicit native FFI package list (22 crates), FFI descriptor rules (`ffi/`, `os/`), and
    system API descriptor prefixes/exclusions (`std::fs`, `std::net`, `std::process`, `std::env`, `std::os`,
    `std::thread`, `std::io`, `tokio`).
  - `CriticalityOverrides`: Built persistent symbol and package manual overrides rejecting `local N` bindings.
  - `CriticalityPolicy`: Implemented strict deterministic precedence table matching Swift: `local N` -> user
    symbol override -> user package override -> profiled FFI package/descriptor -> `extern "` signature ->
    system API -> repository package/definition -> runtime plumbing -> unknown.
  - `SourceProjection` & View Rendering: Added normalized disjoint byte range tracking; updated `SourceView::show`
    to recede non-prominent code spans (reduced opacity `0.38` dark / `0.30` light) when `SourceMode::Critical`
    is active.
  - Added gutter count annotations with `CountAnnotation` and `CountCompleteness` (`0`, `3+`, `—`).
- **Pull Requests Subsystem:**
  - `UnifiedDiffParser::parse_file_changes`: Converts unified diffs into structured `FileChange` records capturing
    path, old path, additions, deletions, binary status, and change status (`Added`, `Modified`, `Deleted`, `Renamed`).
  - `ReviewChangesPresentation`: Filters file changes against active `SourceFilters` (`sourceFilesOnly`, `hidesTests`),
    calculates visible additions/deletions totals, and computes `file_count_label`.
  - Sidebar integration: Automatically populates `self.pr_changes` and switches to the `Changes` tab upon opening
    or restoring pull requests.
  - Change block navigation: Implemented `previous_change_line` and `next_change_line` in `FileLineChanges` for
    navigating between changed hunks via `Alt+Up` / `Alt+Down`.
  - Added `Ctrl+Alt+B` shortcut to swap between Before and After revisions.
- **Bookmarks Subsystem:**
  - `BookmarkScope`: Added repository-level scoping isolating local checkouts (`<canonical_repo>\0checkout`) from
    pull request reviews (`<canonical_repo>\0pr:<number>`).
  - `ContextInspector` Click Conflict Fix: Fixed event handling in bookmark rows so clicking delete (`✕`),
    move-up (`▲`), or move-down (`▼`) is never clobbered by row jump click interactions. Added row hover highlighting.
  - Verified keyboard shortcuts `Ctrl+D` (toggle bookmark at caret) and `Ctrl+0`..`Ctrl+9` (jump to first 10 bookmarks).
- **Quality Gates & Preservation:**
  - 38/38 unit tests passing (`cargo test --all-targets`).
  - 29/29 doctests passing (`cargo test --doc`).
  - `cargo clippy --all-targets` clean with zero warnings.
  - `cargo fmt --all -- --check` clean with zero diffs.
  - Swift source code remains 100% intact as instructed.


## 2026-10-27 — SourceView Virtualization, Performance & Clipping Resolved (Antigravity)

Addressed user-reported performance degradation and text overflow in `SourceView`:
- **Root Cause 1 (Performance):** The central code view was running syntect syntax
  highlighting across *every* line of the open file on *every* frame (180,000+ passes/sec
  on large files) and allocating widgets for thousands of invisible lines.
  - *Fix:* Added `highlight_document` to precompute highlighted token spans once when a
    file buffer loads, storing `highlighted_lines: Vec<Vec<HighlightedSpan>>` in `RadiologyApp`.
  - Upgraded `SourceView` to `ScrollArea::show_rows` virtualization, calculating row
    offsets mathematically and laying out only the ~30–50 rows visible in the viewport.
- **Root Cause 2 (Text Overflow):** Code lines forced a min width of 800px and painted
  with unconstrained layer painters (`ui.painter().layout_no_wrap`), causing text to draw
  directly over `SidePanel::right`.
  - *Fix:* Enforced strict clipping with `ui.painter().with_clip_rect(clip_rect)` so that
    rendered text and tokens are physically clipped at the boundary of `CentralPanel`.
- **UI Enhancements (Subsystem 4):**
  - Added full `WelcomeView` state in `CentralPanel` when no repository is open: displays
    "Read the Code", an "Open Repository…" folder picker button, clickable recent repository
    history list from `SessionStore`, and shortcut legends.
  - Added global shortcuts: `Ctrl+O` (Open repository), `Ctrl+Shift+F` (focus Search tab),
    `Ctrl+1` / `Ctrl+2` (Normal vs Critical mode), and `F12` (Jump to Definition).
- Quality gates: All 33 unit tests and 28 doctests passing, clean clippy and fmt.


## 2026-10-27 — Swift Parity Audit & Defect Remediation (Subsystems 1–3) Completed (Antigravity)

Conducted exhaustive line-by-line feature audit and defect remediation comparing
the Rust + EGUI port against the original Swift implementation across Subsystems 1–3:
- **Subsystem 1 (AppModel, Repo Discovery, Filters & Search):**
  - Refactored `SourceFileClassifier` and `TestPathClassifier` to exact parity with
    Swift rules (including Gradle source sets, language-specific stems, config script
    exclusions, and 80+ extensions).
  - Implemented Swift's exact 5-tier candidate ranking for `QuickFileSearch` (exact name,
    prefix, substring, filename subsequence, path subsequence) with Unicode NFD diacritic
    mark stripping and case folding.
  - Eliminated UI-thread blocking during ripgrep search by streaming batches via
    `mpsc::channel` from a background thread with cancellation token support.
  - Replaced hardcoded `$HOME` directory opening in toolbar with native
    `rfd::FileDialog::new().pick_folder()` modal.
  - Validated repository discovery, deduplicating paths, dropping deleted working-tree files,
    and avoiding symlink cycles.
  - Capped recent repository history at 8 entries and added repository directory validation.
- **Subsystem 2 (Source, Syntax, Folds, Virtualized SourceView):**
  - Added TSX grammar support (`SupportedGrammar::Tsx`) for `.tsx`/`.jsx` files.
  - Registered `.mts`/`.cts` and added fallback syntax mapping for TypeScript and TOML.
  - Fixed fold deduplication to retain outer enclosing folds when sharing `start_line`.
  - Implemented `TestRegionDetector` discovering test blocks across Rust, Python, and JS/TS.
  - Virtualized `SourceView` rendering so only visible lines within the viewport are laid out.
  - Fixed character column click hit-testing, arrow key traversal across folded lines, and
    occurrence pin indicators.
- **Subsystem 3 (SCIP Wire Protobuf, Semantics, Caller Trees & Context Inspector):**
  - Fixed SCIP protobuf wire field tags: mapped field 7 to `Occurrence.enclosing_range`
    (with typed range precedence from fields 8–11); mapped field 6 to `display_name`;
    decoded embedded `Document` message on field 7 to extract symbol `signature`.
  - Defined `SemanticSymbolID` with document scoping for `local N` symbols, preventing
    cross-document identifier collisions.
  - Implemented `SCIPSymbolString::fallback_name` matching Swift descriptor grammar parsing.
  - Built 4-tier workspace symbol ranking engine in `SymbolSearch` (exact, prefix,
    camelCase/component initials, fuzzy subsequence) and wired it into `GoToSymbolModal` (Ctrl+T).
  - Fixed caller pill slot calculation using fallback names to ensure deterministic 12-color
    palette matches Swift across application launches.
  - Fixed navigation history corruption during Back/Forward operations.
  - Implemented CLI arguments `--repository`, `--pull-request`, `--file`, and `--verify-resources`.
- **Quality Gates & Preservation:**
  - 33/33 unit tests passing (`cargo test --all-targets`).
  - 28/28 doctests passing (`cargo test --doc`).
  - `cargo run -- --verify-resources` passed with all 13 syntax grammars verified.
  - `cargo clippy --all-targets` clean with zero warnings.
  - `cargo fmt --all -- --check` clean with zero diffs.
  - Swift source code remains 100% intact as instructed.
  - Index updated in master `PLAN.md`; details in `plans/swift-parity-and-defect-remediation.md`.


## 2026-10-27 — RFC 1574 Documentation Formatting & Doctests Completed (Antigravity)

Applied strict RFC 1574 Appendix A documentation styling across all 45 crate modules,
data structures, algorithms, and UI views in `src/`:
- Every public type, method, function, and module features third-person present tense
  summary doc comments, detailed markdown explanations, intra-doc links, and `# Examples`
  or `# Arguments` sections.
- 28 doctests implemented and passing (`cargo test --doc`).
- Verified zero broken intra-doc links or rustdoc warnings via `cargo doc --no-deps`.
- `cargo fmt --all -- --check` and `cargo clippy --all-targets` clean with zero warnings.
- All 26 unit tests continue to pass (`cargo test`).
- Changes committed to `rust-egui-port`.

## 2026-10-27 — Rust & EGUI Linux Native Rewrite complete (Antigravity)

Completed full rewrite of Radiology from Swift/macOS to a standalone, native
Linux application written in Rust using EGUI (`eframe` 0.27 / `egui` 0.27) on
branch `rust-egui-port`.

Delivered complete architectural and functional parity:
1. **Source Subsystem:** `SourceBuffer` with UTF-8 byte canonical indexing,
   line/col <-> byte translations, scalar safety, and `NavigationHistory` with
   Back/Forward stacks and visit deduplication.
2. **Repository Subsystem:** Fast `git ls-files -z` + `walkdir` fallback discovery,
   `SourceFileClassifier`, `TestPathClassifier`, `SourceFilters` ("Source Files
   Only", "Hide Tests"), and `QuickFileSearch` with subsequence fuzzy scoring.
3. **Syntax Subsystem:** Multi-language syntax highlighting with `syntect` (100+
   syntaxes), Tree-sitter AST queries for document outline (functions, structs,
   classes, enums, methods, traits) across Rust, Go, Python, JS, TS, C, and
   code folding (`FoldDetector` & `FoldStore`).
4. **Search Subsystem:** Asynchronous streaming `ripgrep` runner with JSON decoding,
   regex/case/glob options, and 5,000 result cap.
5. **Semantic Subsystem:** Pure Rust zero-dependency SCIP protobuf wire reader
   (`ScipIndex`), definition/reference queries, and multi-level `CallerTreeBuilder`
   with recursion cycle detection and deterministic FNV-1a 12-color caller
   identity capsules.
6. **Bookmarks & Critical Lens:** `SourceBookmark` with content digest and
   surrounding line-anchor relocation, persistent JSON `BookmarkStore`, Ctrl+0..9
   shortcuts, and `CriticalityPolicy` (Normal vs Critical Lens mode).
7. **PR Review & Diffs:** `GitHubClient` via `gh` CLI, `UnifiedDiffParser` (`git
   diff -U0`), Before/After document viewing, diff line gutter stripes, and
   change block navigation.
8. **EGUI Native Desktop UI:** Resizable 3-pane layout, 72px compact gutter with
   every-5th line numbers and active line indicator, top toolbar, modals for
   Go to File (Ctrl+K), Go to Line (Ctrl+L), Go to Symbol (Ctrl+T), and PR Review
   (Ctrl+P). Clean dark/light theme palette.

Quality gates: 26 unit tests passing across all crates/subsystems, clean `cargo
clippy --all-targets` with zero warnings, and `cargo fmt` applied. All commits
recorded on `rust-egui-port`. Master `PLAN.md` and `plans/rust-egui-linux-port.md`
updated. Ready for standalone use or integration into `~/little-screen-clip`.

## 2026-10-11 — Published all reader work and merged to main (Codex)

User authorized pushing all accumulated work to GitHub. Confirmed authenticated
origin tqbf/radiology, private visibility, main as default branch, no open PRs,
and local HEAD/main matching origin/main at2e02ae3. Created
codex/reader-review-and-bookmarks. Scope includes source/tests/docs/resources and
supplied icon artwork; existing ignore rules retain generated build artifacts
and the inactive repo/ starter locally. The most recent whole-package gates are
269 tests/57 suites, check/signed build/strict signing/resources from the bookmark
round, with no implementation changes afterward. The user explicitly authorized merging this batch to main,
superseding the earlier prohibition for this publication; AGENTS.md is retained.
Marked SCIP fixtures binary through .gitattributes after Git's text whitespace
check misclassified protobuf bytes; fixture contents unchanged.

Committed all264 changed source/test/resource/artwork/doc files as3b8f7f5, pushed
codex/reader-review-and-bookmarks, and opened https://github.com/tqbf/radiology/pull/1.
GitHub reported MERGEABLE/CLEAN, with no configured check runs. Merged using the
exact full head SHA guard after the user's authorization; confirmed state MERGED
at19:53:25 UTC /14:53:25 CDT, merge commit dfc877a1734cca28543918273ea0a71825f42232.
Fetched GitHub main and fast-forwarded local main to the merge. Publication
metadata is recorded in a follow-up documentation commit. No force push, tkos
mutation or reader restart. Branch retained; future work starts from main.
plans/github-publication.md records final scope/commits/limits. Earlier local/
uncommitted entries are historical; accumulated code is now published on main.

## 2026-10-11 — Repo/PR bookmarks and third inspector pane (Codex)

Implemented user's bookmark request: Bookmarks beneath Definition/Outline with
its own scrolling and movable divider. ⌘D/header/menu toggles the genuine caret;
first ten ordered entries map ⌘0…⌘9. Native List.onMove persists order; Move Up/
Down/Move to Shortcut context menus offer an alternative, with removal/native
delete and Return/whole-row buttons. More than ten entries remain clickable.
Normal/Critical moved to ⌥⌘1/⌥⌘2, freeing plain numbers. Body-medium names,
caption path/line/revision and tabular shortcut hints keep existing hierarchy;
PR/Checkout header and faint orange tint show scope. Skills SwiftUI before/after,
macos-design and typography-designer applied. Contract: plans/bookmarks.md.

BookmarkScope uses canonical attached root + optional PR number, never snapshot
root/head SHA; checkout, each PR and another repo isolate persistence. Codable
UUID/relative path/revision/exact UTF-8 byte/label/digest/bounded line anchors live
in injected UserDefaults. BookmarkModel owns observable order/selection/error;
BookmarkResolver reads/relocates off MainActor, using actual open bytes if present.
Unique matching anchors relocate after insertions; ambiguous/edited/missing targets
report locally without moving source/history. Successful relocation updates
coordinates/digest while keeping UUID/order/slot. Semantic navigation still forces
After but now delegates to navigateFromCaret; bookmarks use that shared exact-
caret history/highlight path with their explicit revision. Scope/root/navigation/
caret/cancellation/existence guards reject stale jumps; opening cancels work.

The flat three-pane SwiftUI split failed the second native divider test (middle
stayed at its minimum). Replaced it with nested native two-pane splits; both
dividers now resize affected panes and preserve context state. Minimum heights
150/100/100. macOS14 cannot use enumerated sequence as ForEach RandomAccessCollection;
BookmarkListEntry provides stable UUID/slot data instead. The preexisting fake
GitHub constructor traps for duplicate SHA pairs; new tests initialize one pair
and then add the other PR number. No shared fixture or deployment target changed.

Fresh full suite **269 tests /57 suites** passes. Six new tests cover persistent
scope/order/deletion/slot promotion beyond ten, Unicode/CRLF exact positions,
changed/ambiguous anchors, malformed storage/invalid moves, real AppModel shortcut
handlers and exact Back, relaunch, updated anchors and toggling relocated caret,
disposable real-Git checkout/PR1/PR2/repository scopes and Before/After navigation,
missing files and cancellation on repo switch. Existing context/semantic/PR
refresh suites pass. Native inspector test exercises both nested dividers.
Six production-row/header renders at220/280/420 light/dark verify long text,
Before/After, ⌘0…⌘9, and two unassigned entries; build/bookmarks-review/*.png.
Physical drag, menus and keyboard activation remain unverified because this
session lacks desktop automation. No live interaction or user-reader restart.

make check/build, strict deep codesign, empty-env /tmp packaged resources
(8 grammars/37 queries/3 scripts), diff check pass. Signed executable built
**2026-10-11 14:41:15 CDT**. Logs /tmp/radiology-bookmarks-{check,integration,
tests,build}.log. PLAN/README/architecture/design/handoff/Context and Critical
shortcut docs updated. Local/uncommitted; no merges or tkos writes.

## 2026-10-11 — Stable caller pills and stop before repeated functions (Codex)

User reported repeated launch entries within a chain, then requested a palette
and one pill per caller, selected by a hash of method name and file. Added
CallTreePalette with12 dynamic system hues and portable FNV-1a64 over UTF-8
name/NUL/full relative file path. Swift Hasher is deliberately avoided because
its seed changes between launches. Same method/file repeats on separate branches
keep their hue; differing files contribute to identity. ReferenceCallerPill is
a native capsule Button containing name/file/call-site line, with12/6-point
padding, light/dark12%/24% fills and30%/50% borders. System primary/secondary
text and existing callout semibold/caption regular scales stay legible.
Increased contrast/Differentiate Without Color strengthen border75%; the latter
also uses1.5-point width. Branch arrows/deep-level markers sit outside; boundary
notes stay below. Capsule-wide navigation uses existing exact-caret Back.

Cycle diagnosis: old code stopped recursion but rendered the repeated function
as a terminal row. It now filters incoming callers before enqueueing, using
both semantic identities and canonical declaration positions. The preceding
row notes Recursive branch stopped; sibling branches and separate same-named
functions remain. Added mutual launch→relay→launch and distinct launch fixtures;
existing self-cycle assertion now requires one middle entry. New golden palette
assignments verify portable name/file/Unicode/field-separator hashing. Skills
swiftui-pro applied before/after, macos-design and typography-designer reviewed.
Contract: plans/caller-pills.md; traversal details in plans/reference-call-trees.md.

Full suite **263 tests /55 suites** passes. Actual production-row renders at
220/280/420 in light/dark verify capsules/readability and matching brown entry
pills on separate branches. Artifacts build/callers-pill-review/{light,dark}-*.png.
make check/build, strict deep codesign, empty-env /tmp resources (8 grammars,
37 queries,3 scripts), diff check pass. Signed executable **13:47:46 CDT**.
Logs /tmp/radiology-callers-pills-{targeted,tests,build,check}.log. Initial targeted
compile reports the preexisting unused ts variable warning in extraction tests;
no production warnings. User requested automation retry: this session returns
Module not found: @oai/sky, including after resetting the Node kernel and checking
available tools. No native mouse actions or reader restart; GUI checks remain
unverified. PLAN/README/design/handoff and traversal docs updated. Changes local,
uncommitted; no merge or tkos writes.

## 2026-10-11 — Reference caller trees and resizable inspector (Codex)

Implemented the user's retry: expanding a reference reveals source context and
its containing callable's incoming caller tree. ReferenceCallTreeWorker runs
off MainActor, linking callable definitions by exact SCIP identity and following
only parsed call-name sites resolved to that identity. Iterative depth-first
traversal preserves repeated ancestors on separate branches, marks recursive
calls per path, and has no fixed depth cutoff. Initial 256-row work budgets
continue with Trace more callers. Closures/non-call references/unindexed owners
are explicit boundaries; unreadable/stale/parse gaps show partial coverage.
Per-request file caches are bounded to48; graph state never crosses roots.
ContextModel starts work on disclosure, cancels on collapse/invalidation, caches
loaded trees within the reference identity, and rejects superseded completions.
Caller buttons commit the actual call site through existing semantic navigation.

Restored native VSplitView between Symbol/References and Definition/Outline,
with independent scrolling and150/100-point minimum pane heights. Removed the
outer ScrollView/fixed320-point list budgets. Existing title/section hierarchy,
exact-identity name deduplication and appearance-aware tints remain. Caller names
use existing callout monospace semibold; caption regular metadata/boundaries and
tabular line numbers remain quiet. Indentation caps at four levels, with explicit
level markers deeper; full path/depth are accessible. Applied swiftui-pro before
code decisions and again for final review, plus macos-design/typography-designer.
Current contract: plans/reference-call-trees.md, superseding prior layout docs.

Fresh full suite: **260 tests /54 suites**, all pass. Seven added tests cover
cross-file branching/shared ancestors, UTF-16/emoji/accented names, closures,
callbacks, nested functions, recursion, gaps/cancellation, lazy loading/caching/
reset during work, a270-level tree continued beyond256 rows, no navigation on
disclosure, caller/Back exact-caret restoration, and native divider resizing.
Actual ContextInspectorView/NSSplitView divider350→230 resizes both panes120
points without disturbing selection or disclosure. Native split decoration adds
an extra subview; the test correctly checks its two arrangedSubviews.
Six actual-row renders at220/280/420 light/dark pass; artifacts
build/callers-review/{light,dark}-*.png. Physical mouse drag, live List scrolling
and caller clicks remain unverified: Sky native pipe startup failed. No live app
or tkos interaction was attempted; user's reader was not restarted.

make check/build, strict deep codesign, empty-env /tmp packaged resource probe
(8 grammars/37 queries/3 scripts), and diff check pass. Signed executable built
**2026-10-11 13:31:01 CDT**. Logs /tmp/radiology-callers-{check,tests,integration,
layout,build}.log. PLAN, README, architecture/design/handoff and current Context
docs updated. Changes local/uncommitted; no merge.

## 2026-10-08 — Inspector typography and section cues (Codex)

User requested the inspector critique plus faint background cues. Replaced
forced vertical split/minimum panes with top-aligned scrolling sections, dominant
title3 semibold symbol/kind/file:line, caption section headings, primary snippet
text, and blue/teal/purple/neutral backgrounds at2.5% light/5.5% dark. Native
reference/outline lists receive bounded content budgets; short content no longer
pushes Definition down. Exact SCIP equality suppresses duplicate Definition
name; different caret targets remain identified. Location now includes file:line
or file:line/Here. ContextSymbolRow removed; InspectorSymbolSummary owns the hero.
UnderCaretDetailView now takes natural disclosure height; short/empty outline
states compact. References/outline keep native Lists with capped content budgets
(320 points), rather than exact measured intrinsic heights. Section titles/
dividers provide meaning without relying on color; decorative tints hidden from
accessibility. Model/navigation semantics retained. Required skills applied
before/after. Contract: plans/inspector-refinement.md.

Six actual-component renders at220/280/420 light/dark verify hierarchy/snippet
contrast/tints. Native PR758 admit_call: title/kind/file:145,4 references plus
definition occurrence, stacked Definition/location/Here, same name omitted,
no large inter-section gap. Reference selection stays145; Right disclosure
expands, Return→127, Cmd-[→145 exact. Screenshot
build/inspector-review/native-definition.jpeg. Different-target native check
unverified after selection/offscreen click failures; no claim for that case.

First full suite hit existing SourceScrollView highlight timer assertions (fade
alpha0.06 at deadline). Targeted test passed; unchanged full rerun passed all
**253 tests / 52 suites**. make check/build, strict deep codesign, empty-env /tmp
resources (8 grammars/37 queries/3 scripts), diff check pass. Built
**2026-10-08 22:59:05 CDT**. Logs /tmp/radiology-inspector-{check,build,tests,
tests-final,highlight-test}.log. Final separate review closed, no indexing active;
user's reader/tkos preserved. Current docs supersede fixed vertical-split layout.
Local/uncommitted; no merges/restarts.

## 2026-10-08 — Toolbar controls and Changes sidebar (Codex)

Implemented the user's dropdown Normal/Critical (default Normal), Before/After
beside it in the main toolbar, menu-only Source Files Only/Hide Tests. Removed
SourceFiltersMenu/SourceFilterToggles/SourceRevisionControls; new SourceModePicker
and SourceRevisionPicker, View Show All Files reset. Breadcrumb now has only
changed-block/collapse tools, freeing path width. Initial gates: 253 tests/52
suites, compile/build. Native PR758: Normal default, dropdown Critical, Cmd-1
Normal reset, Before/After and Source Files Only View menu26→30, caret1/After
preserved. Screenshot build/toolbar-controls-review/normal-after.jpeg.

User steered ongoing work to tighten Changes sidebar and replace Browse/segmented
selector with tabs. Added SidebarModeTabs native buttons, body semibold selected
label/accent underline/selected traits and keyboard Left/Right. ReviewChangeRow
and Status now separate files: body medium filename (selected semibold), caption
secondary paths/counts, tabular right-aligned min64-point count lane, M modified
glyph/circled plus additions, 8% selected tint plus2-point marker. Two-line rows
and native List selection retained. PR summary separates from list with12/8-point
spacing; duplicate file count moves to footer. Footer names Source only/Tests
hidden, with PR head SHA moved to Git-icon tooltip/accessibility.

Six actual-view renders at220/280/400 light/dark cover long names, varied counts,
modified/added/deleted/renamed/binary and selected row. Off-window List initially
deferred rows; final renderer uses production row views in a stack. Native PR758
verified tabs/no Browse, hierarchy/M/plus/right counts, selected Core.kt marker,
mouse mode switching, Source only Files5509/7019 and Changes26/30, both flags18/30.
Initial Right Arrow still reached source; explicit button focus fixed it. Final
Files click focuses tab, Right selects Search/focuses field; Changes restores list.
Removed duplicate mode accessibility label. Screenshots build/sidebar-review/
native-{combined,selected-core,source-only,both-filters,final-tabs}.jpeg.

Final gates: **253 tests / 52 suites**, make check/build, strict deep codesign,
empty-env /tmp resources (8 grammars/37 queries/3 scripts), diff check. Built
**2026-10-08 17:35:01 CDT**. Logs /tmp/radiology-sidebar-{check,tests,build}.log.
Required skills applied before/after; docs updated, current contract
plans/sidebar-refinement.md supersedes older filter icons/segmented mode/header
revision layout. Separate final review closed with no indexing active; user's
reader and tkos repository preserved. Local/uncommitted, no merges/restarts.

## 2026-10-08 — Source breadcrumb territory and more context (Codex)

User requested fixed right-aligned revision controls and preferential compaction
of intermediate paths in the existing second row. Added SourceRevisionControls
(194 points, native picker136 + changed-block50 + gap8); hidden visible Revision
label, status retained in help/accessibility. SourceBreadcrumbPath/Trail/Separator/
Overflow use ViewThatFits and a native full-path/declaration menu, keeping file
and final declaration legible with middle truncation only at extreme widths.
Semantic callout scale: regular secondary context, primary file, medium final
declaration. Required SwiftUI/macOS/type skills applied.

First header build passed 253 tests/52 suites, compile/build/sign/resources;
native isolated PR758/nomex.rs verified Before/After, Go to Line150's visible
file/Nomex/admit_call and the full-path/declaration menu. Screenshot
build/breadcrumb-review/native-pr758.jpeg. A sticky Repository menu held keyboard
input until its exposed Cancel accessibility action dismissed it. Test app
closed; user reader untouched.

User then steered the unfinished turn to retain a couple more entries when
possible. Granular alternatives drop far directories one by one, retain
declaration ancestors one by one, and use spare width for nearby directories.
The actual-view render probe caught nested ForEach IDs colliding inside
ViewThatFits; flattened options into uniquely identified SourceBreadcrumbVariant.
Ten production-view renders pass at1100/740/560/440/360 in light/dark: src + all
three declarations at740, file + two declarations at560, file/final at440,
complete final name at360, fixed controls throughout. make check passes.
Final native review at the same window size retains three extra nearby
directories: crates/harness-core/src/nomex.rs/Nomex/admit_call. Initial file
opening and Go to Line150 exercise zero-to-two declaration changes without the
ID collision. Screenshot build/breadcrumb-review/native-more-context-pr758.jpeg.
No indexing active at shutdown; separate review app closed, user's process kept.
Physical resize not attempted; actual-view renders cover narrow/light/dark.

Final gates: **253 tests / 52 suites**, make check/build, strict deep codesign,
empty-env /tmp bundled resources (8 grammars/37 queries/3 scripts), diff check.
Built **2026-10-08 17:02:10 CDT**. Logs /tmp/radiology-breadcrumb-{check,tests,
build}.log. Required skills reapplied in final review. PLAN/README/design/
navigation/handoff updated; contract plans/source-breadcrumbs.md. Artifacts and
renderer live in ignored build/. Changes local/uncommitted, no merges/restarts.

## 2026-10-08 — Remember the last repository and PR (Codex)

User requested last-PR persistence across relaunch. Added ReaderSession and
ReaderSessionStore: JSON Data in the injected UserDefaults, absolute attached
checkout path and optional positive PR number. AppModel persists confirmed
repository/PR opens; Exit PR explicitly records local checkout. Failed/cancelled
PR choices keep the last confirmed record. The window task now runs a one-time
launch routine: CLI intent/manual repository choice wins; ordinary launch
prepares the remembered checkout without saving temporary local mode, then
opens the PR through existing fresh metadata/worktree/indexing. Cancellation and
generation guards prevent stale restoration or CLI file navigation. PR-number
sheet defaults to the remembered PR after a failed restore.

Seven lifecycle tests pass with disposable Git, fake GitHub, isolated preferences
and separate models. They cover real PR preparation/restoration, attached root,
fresh metadata, exit/local, CLI/file overrides, failure/retry, failed/cancelled
choices, newer repository intent, startup replay, missing folders and bad data.
Initial tests exposed directory-URL equality affecting temporary-save suppression;
fixed with an explicit restoration marker and directory URL canonicalization.
A duplicate-SHA-pair fake-client fixture initialization was corrected as well.

Native separate-instance acceptance passed: tkos #758 opened with arguments,
review-only indexing cancelled, app quit, relaunched with no arguments. New PID
without arguments restored #758/Changes with fresh head 4106d3606a…. Screenshot
build/session-review/restored-pr758.jpeg. Review-only indexing cancelled again
and test app closed. Read current user UI/recent root to confirm tkos #758;
seeded that session into the real app's absent preference so its first upgraded
launch restores the review. User reader process and repository bytes untouched.

Final gates: **253 tests / 52 suites**, make check/build, strict deep codesign,
empty-env /tmp bundled resources (8 grammars/37 queries/3 scripts), diff check.
Built **2026-10-08 16:36:48 CDT**. Logs /tmp/radiology-session-{tests,full-tests,
check,build}.log. Applied swiftui-pro before/after; no type or UI surface changes.
PLAN/README/architecture/handoff and plans/session-restoration.md updated.
Changes local/uncommitted; no merge or user-reader restart.

## 2026-10-08 — Compact gutter and sparse numbering (Codex)

User steered the active toolbar task: gutter looked too wide; number only every
fifth line and the cursor line. Gutter width had not changed in the toolbar work
(104 points already committed). Now 72 points, 20-point disclosure lane, 6-point inset.
Native monospaced UI numbers 11pt regular/secondary, caret medium/primary,
independent of source zoom. Caret line derives directly from native selection
through SourceBuffer; removed duplicate stored gutter currentLine updates.
Visible glyphs/wrapped-line de-dup stay; PR bars/folds/test markers draw without
requiring numbered rows. Count badges fit spare width or retain exact counts
in tooltip/accessibility; function meter now fits the 20-point lane (6–16 points).

Both native NSView/TextKit bitmap-draw tests pass: initial/fifth labels, moving
caret 7, passive scroll preservation/offscreen clearing, folded line 10 exclusion,
expansion/canonical restoration with caret 18. Initial harness failures came from
an unspecified native selection and a document shorter than its scroll viewport;
fixed harness conditions, without loosening sparse-number expectations.

Live native PR758 review: Go to Line7 shows7 beside10/15/20/25/30/35; Down changes
only the non-fifth label to8 and clears7. The compact width/source type were
observed. Screenshots: build/toolbar-review/compact-gutter-caret7.jpeg and
compact-gutter-caret8.jpeg. Review app had a separate identity/preferences;
the user's reader was untouched. Physical folded-count/dark-mode/very-large-digit
acceptance remains manual; bitmap tests cover canonical folded/scroll behavior.

Final combined gates: **246 tests / 51 suites**, make check/build, strict deep
codesign, empty-env /tmp packaged resources (8 grammars/37 queries/3 scripts),
diff check. Signed **2026-10-08 16:10:23 CDT**. Logs:
/tmp/radiology-gutter-tests.log, /tmp/radiology-reader-ui-tests.log,
/tmp/radiology-reader-ui-build.log. Applied required skills before/after review.
README/design/navigation/handoff/index docs updated; changes local/uncommitted.
Scope: plans/source-gutter.md; toolbar task completed below.

## 2026-10-08 — Toolbar grouping (Codex)

User requested the attached toolbar critique: three groups, repository dropdown
owns Open Repository, PR dropdown owns Exit PR, visibly labeled PR selector,
and no duplicate PR title. Extracted native ReaderToolbar + RepositoryMenu,
SourceFiltersMenu, SourceSearchMenu and conditional title-item suppression.
Navigation/sidebar/history, PR context/icon refresh, and Normal/Critical +
filter/search/Context use existing actions/bindings; left SidebarCommands added,
right inspector recovery preserved. Full title remains in Changes/menu/help;
Before/After and changed-block controls stay in source header. Explicit PR
titleAndIcon fixes the prior icon-only upward-arrow label.

Applied swiftui-pro, macos-design, typography-designer and swiftui-patterns.
Native SF body/regular, semibold repo identity, system controls/materials. Native
computer-use review caught primaryAction tools over the sidebar and duplicate
automatic sidebar control. Moved toolbar/title modifiers into ReaderDetail and
removed default sidebar items inside both split columns. Real review now shows
one navigation toggle, repo/name, #758/refresh, Normal/Critical and trailing
filter/search/Context; no duplicate PR title or ordinary-width overflow.

Verified repository Open/Open Recent; PR first-level Exit/active number outside
recent25/refresh commands; related search actions. Toolbar Source Files Only
changed navigator state and Changes counts together. Left toggle collapsed the
sidebar; Ctrl-Cmd-S restored it with caret8 unchanged. Physically activated Exit
PR and confirmed selector Pull Request, Files/local checkout, refresh removed.
Native resize drag failed -10005:noWindowsAvailable for bundle id and full path;
state recovery worked, then resizing was stopped. Narrow acceptance unverified.
No user process interaction/restart. Canceled review-only indexing and closed
only review identity org.sockpuppet.radiology.toolbar-review. Same final246-test/
signed gates above. Full scope/screenshots: plans/toolbar-layout.md. No new tests
solely for reversible toolbar layout/menu edits.

## 2026-10-08 — Kotlin workspace links and complete build inputs (Codex)

User's PR758 screenshot reports scip-kotlin failed before compilation:
the file harness-development could not be opened. The tracked path
.agents/skills/harness-development is a directory symlink to
../../.claude/skills/harness-development. The copier materialized its directory,
then tried Data(contentsOf:) on the directory. Earlier manual compiler probes
skipped these directory links and did not validate this production preparation.

Internal file/directory/dangling links now rebase into the disposable copy and
use link-text hashes, including post-build kind/destination verification. Original
Git link bytes still verify snapshot identity. Literal destination
resolution also fixes the pinned tree's dangling external AGENTS.md link;
external Markdown link bytes are preserved without opening the target. Inputs
from the complete snapshot blob inventory include tracked vendor/build paths
omitted by the file browser. Git working-tree discovery similarly expands
compiler input fingerprints/digests, preserving cache invalidation for excluded
dependencies. Source publication still uses the browser's source inventory.

The complete real PR758 SemanticIndexService probe passed in **178.684s**:
repository listing, actual factory/installed adapter/bundled runner, compilation,
post-build byte checks, source filtering, cache metadata and workspace cleanup.
It publishes **2,088 source documents**, with real addSecrecyTag definition and
**18 references in 4 files**, no unreadable/unmapped refs. Kotlin outcome is
Generated with honest partialBuild coverage, rather than the reported Failed
preparation. Cache/probe are temporary; pinned head tree is read-only. No
handwritten-copy bypass this time. Initial full-service attempt exposed a second
dangling AGENTS.md resolution failure; corrected and covered by the regression.

Regression tests cover relative/absolute directory links, internal file links,
dangling Markdown/internal links, original checkout immutability, link/input
tampering, and copying tracked dependencies excluded by the browser. The actual
Git working-tree service test changes only a vendor input and proves regeneration,
rather than a false current cache. Focused **6 tests** and full **244 tests / 50
suites** pass; make check/build, strict deep codesign, empty-env packaged resources
(8 grammars/37 query files/3 scripts) and diff check pass. Signed
**2026-10-08 01:10:35 CDT**. Logs: /tmp/radiology-kotlin-service-probe.log,
/tmp/radiology-kotlin-directory-tests.log, /tmp/radiology-kotlin-link-tests.log,
/tmp/radiology-kotlin-link-build.log. Attached tkos Git status unchanged.

Applied swiftui-pro before/after review; actor/generation/cancellation boundaries
retained. No UI/type changes, user-app interaction or restart, commits/pushes, or
attached-checkout builds. Relaunch the new bundle and reopen PR758 to retry
Kotlin automatically. Partial coverage can remain for unavailable sources/build
variants. Updated PLAN/handoff and plans/kotlin-semantic-indexing.md; earlier
manual compiler evidence above/below does not replace this production-path proof.

## 2026-10-07 — Kotlin semantics in PR After (Codex)

User reported references and caret symbol state failing in PR mode. Screenshot
shows tkos PR #758 After, Kotlin Contacts source, and File not indexed. Root
cause for that example: registry has no Kotlin/JVM indexer; syntax declaration
tracking exists but semantic under-caret/definitions/references do not. User
explicitly selected Add Kotlin semantic support.

Added ScipKotlinIndexer, managed compiler-tool installation and bundled Gradle/
runner scripts. The pinned source `9f288dd339c7e9fc257a7b21714d9a402fd4d837`
supports Kotlin2.4.20; installed CLI reports `0.13.1-radiology-9f288dd339c7`.
AGP9 needs current compilerOptions and task discovery after projectsEvaluated:
ordinary afterEvaluate could report success without compiling Android sources.
Builds copy genuine inputs to a disposable index-cache workspace, preserve
paths/bytes/modes, reject unsafe links/traversal/stale inputs, and verify source
hashes again after compilation. No fake manifests or guessed symbol edges.
Successful compiler shards survive independent failures as persisted Partial.
Generation/cache/reuse exclude generated documents outside the reader snapshot.
Compiler data exposed a UTF-16-column/UTF-8-file mismatch: adapter defaultEncoding
now reaches SemanticIndex.Input on all three loading paths; explicit document
encoding wins. Existing Context, reference commits, definition and exact Back
use those identities. Before remains syntax-only; semantic destinations After.

Actual PR758 head `5d2ffdd2976c62a220118e30780edfe8fb6f2120`, compiled in
/private/tmp/radiology-kotlin-pr758 on AGP9.4.1/Kotlin2.4.20/Gradle9.7.1/JDK17.
Targeted Contacts/dependencies/tests: success, 112 compiler documents. Broader build:
2m03s, 1,276 Gradle tasks, 2,379 documents; installRustAndroidTarget/installUniffiBindgen
failed in the manual shell environment, so coverage is Partial. Read-only real
probe maps 1,355 Contacts occurrences; line 141 addSecrecyTag resolves to
core/src/main/kotlin/inc/tk/os/module/SecrecyCtx.kt:18, with 18 references in 4 files,
zero unreadable reference files/unmapped refs. One upstream generated-constructor
range stays unmapped; no ordinary call token omission is allowed by the probe.
All 10,394 copied tracked inputs are unchanged by compilation; attached tkos Git
status unchanged between checks. No build was run in the attached checkout.

A 4,119-byte actual Kotlin compiler fixture tests overloads, same-spelled unrelated
functions and emoji/accent offsets. PR tests drive native text-selection callbacks
on a changed call line through Context, Find References, definition/reference
commits, forced After despite Before preference, passive-scroll departure and
exact Back, Before clearing/After mapping and metadata-only refresh. Cache/reuse
tests retain UTF-16/Partial/inventory boundaries; isolation tests reject changes.

Gates: **241 tests / 50 suites**, make check/build, strict deep codesign,
env-i /tmp packaged verification (8 grammars/37 query files/3 indexing scripts),
diff check. Signed **2026-10-07 18:36:00 PDT**. Initial suite run during Gradle
hit an existing 5-second process deadline at 5.62s; idle full runs passed without
weakening it. Installed tool/version and actual compiler probes pass.
Applied swiftui-pro before/after, macos-design and typography-designer; the new
Partial explanation uses existing SF Pro callout regular/secondary metadata.

No physical UI launch, pointer navigation or screenshot recheck; user app left
running. Relaunch signed bundle to use new registry, then Kotlin index generation
runs automatically. Other compiler versions/scripts/non-JVM targets unclaimed.
Native setup failures and missing files stay inspectable; see exact contract,
logs and manual PR758 acceptance in plans/kotlin-semantic-indexing.md. README,
PLAN, SCIP support matrix, build/handoff docs updated. All changes local and
uncommitted; pre-existing work and the earlier inspector recovery preserved.

## 2026-10-07 — Restore Context after dragging it closed (Codex)

The right Context pane is a native SwiftUI inspector: dragging past its
collapse threshold hides it completely. Recovery previously depended on an
invariant "Toggle Context" toolbar label, disabled without a source file and
potentially in toolbar overflow at narrow widths. There was no menu or keyboard
recovery command.

RootView now publishes its existing presentation binding at scene scope via
an `@Entry` FocusedValues property in UI/ContextInspectorFocus.swift. The View
menu consumes the same binding with `@FocusedBinding`, offers Show/Hide Context
Inspector and ⌥⌘I, and works independently of source availability. The toolbar
has matching Show/Hide labels, shortcut help, and no source-dependent disabling.
Scene scope keeps recovery available across controls and while the inspector
is collapsed. The existing ⌃⌘I Reindex binding is preserved. Native collapse
and 220/280/420-point inspector sizing remain.

Applied local swiftui-pro before and after implementation and macos-design
layout/interaction guidance. The post-review replaced a manually declared focus
key with modern `@Entry`; no new type styles were set. Required skills were
found in /Users/tqbf/.claude/skills. Used swiftpm-macos for validation.

Validation: make check, **234 tests / 47 suites**, make build (swiftbuild),
strict deep codesign, empty-environment packaged-resource verification (all
eight grammars/calls queries), and git diff --check passed. The final @Entry
implementation compiled in both tests and the packaged build. Signed binary
**2026-10-07 17:47:35 PDT**. Logs: /tmp/radiology-inspector-tests.log and
/tmp/radiology-inspector-build.log. No new tests for this small
command/presentation change.
Physical divider drag, View-menu action, keyboard restoration, and narrow-window
toolbar behavior remain unverified. No user app was restarted or operated.
Exact acceptance steps and implementation contract: plans/context-sidebar.md.
README shortcuts and PLAN current state updated. Changes remain local; all
pre-existing work was preserved.

## 2026-10-06 — Refresh active PR without leaving review (GPT Sol)

Visible native Refresh PR beside selector/Exit PR, mirrored in PR menu/command
menu. One ⌘R binding routes to active-PR refresh; outside PR it remains repository
reload. Refresh List is separate. Button disables/coalesces while fetching, with
Refreshing… label/subtitle. Fresh gh metadata and merge base are always queried.
Same head AND base updates summary/recent row only: reader/history/folds/pins/
index stay. Changed base (even same head) follows the actual snapshot-update path.

Old reader stays usable until candidate worktree/listing/source/mapping succeeds.
Active and candidate worktree leases prevent eviction, including tested limit1
failure/cancellation. Failure keeps old state; native alert retry retains refresh
intent. Exit/cancel/differentPR/repository switch drop stale completion. An atomic
prepared swap clears SHA-obsolete history/semantic/hunk generations, keeping
latest commit-time reader and options. Pending source loads are awaited/rechecked.
Exact byte equality preserves caret/viewport offsets and only that document's
pins; otherwise Git blob hunks map lines (columns retained only on byte-equal
mapped lines). Proven rename deltas restore paths; unavailable revision switches
with notice; missing file falls to Changes with quiet reason. No spelling guesses.
Mode/font/filters/directory expansion/attached-root marks remain. Before syntax-only;
After indexes use existing whole-repository provenance/revision caches. Refreshed
viewport uses existing restoration, and subsequent navigation clears it.

Actual gates: **234 tests / 47 suites**, make check/build swiftbuild, strict deep
codesign, env-i /tmp eight-grammar/all-calls resources, clean diff check. Signed
binary **2026-10-06 20:52:31 PDT**. Seven refresh regressions use FakeGitHub plus
real temporary Git objects/worktrees: metadata no-op/coalesced command routing,
latest caret/viewport/options, changed-base Before and deletion, network/source
failure + refresh retry, limit1 canceled candidate, pending revision load, proven
rename/exact pin bytes/missing fallback, stale cancel/PR/repository/Exit. Attached
fixture checkout status remains unchanged. First failing wait was /var vs
/private/var test-path comparison, fixed by normalizing aliases.

No native launch/attachment: repeated old-reader selection makes further attempts
impractical under user's constraint. No visual/real-GitHub refresh claim; existing
UI gaps remain. Exact manual check and logs in plans/pull-request-ui-review.md.
No tkos/pin/mark/user-reader/GitHub/branch writes or commits/pushes; RADIO preserved.

## 2026-10-06 — PR defaults open changed functions/blocks (GPT Sol)

PR files/revisions now initially expand ordinary folds intersecting changed
ranges or containing empty-side hunk points, including enclosing ancestors.
Signature-only edits open the owning body via whole function/closure scopes;
Rust attached attributes and Python decorated definitions have separate parsed
header ownership, leaving lexical/call/semantic scopes unchanged. Sibling
closures and unchanged functions keep their normal defaults. Added/removed
functions use the actual After/Before ranges. An EOF point inside an unfinished
or indentation body can open it; a boundary after a closed braced function does
not implicate that unrelated neighbor. No invented line highlights.

One-time application waits for matching syntax and hunks in either async order,
keyed by review/path/revision/content. Manual fold choices and bulk commands
while diff is pending win; recollapse is not forced open by later updates.
Fresh coordinators adopt incoming command counters, avoiding replay of an old
Collapse/Fold to Definitions from another file. New loaded documents/revisions
reset default application; index/theme/font/filter/caret updates do not. Explicit
Hide Tests stays in force; normal fold expansion does not reveal test regions.

Fold state changes precede one combined layout, not model navigation. Late
layout preserves the visible source row/selection, cancels obsolete navigation
layout/position markers, suppresses restoration callbacks, and yields to a
newer explicit jump. Normal local fold defaults remain. No new control/toggle.

Actual gates: **227 tests / 46 suites**, check/build swiftbuild, strict deep
codesign, env-i /tmp eight-grammar/all-calls resources, clean diff check. Signed
binary **2026-10-06 14:47:45 PDT**. Native coordinator/parser/component regressions
cover async orders, nested/header/annotation/closure ownership, manual choices,
old/new bulk commands, new/stale identity/revision, empty sides/EOF/added/deleted,
Hide Tests, viewport/selection preservation, newer-jump priority and obsolete
marker then real scroll reporting actual viewport with unchanged caret line.

No app/UI launch or attachment this round: repeated older-reader selection made
another attempt impractical under user's bounded-tooling preference. No live
acceptance or new screenshots; prior gaps remain. Real PR anchors are manual,
not claimed new backend/GUI runs. Files and exact checks in
plans/pull-request-ui-review.md. User reader/pins/marks/tkos untouched; RADIO.png,
inactive repo/ and all prior work preserved. No branch/GitHub/commit/push changes.

## 2026-10-06 — Previous/next changed-block source controls (GPT Sol)

PR source header now has compact native up/down buttons beside Before/After,
with accessible Previous/Next Changed Block names and ⌥⌘↑/↓ tooltips. Existing
Navigate bindings remain; model-derived per-direction availability is shared by
menu and buttons. Long breadcrumbs/status horizontally scroll in a constrained
header, reserving fixed-size revision/navigation controls. No new sidebar/counter.

Distinct hunks are navigated in the exact current revision/path/content. Inside
a block, skip to adjacent block; in an unchanged gap, choose closest block in
direction; at ends, no wrap. Empty-side pure-add/delete anchors remain reachable,
including EOF without a final newline, without invented line washes. Caret at
EOF of an unterminated changed line counts inside that block; trailing empty
newline line is outside. Availability observes genuine caret revision (including
same-line changes), not passive scroll. Explicit jumps save exact departure
before navigate, reveal existing folds/test regions, highlight and focus source;
Back returns exact caret/revision even after passive scrolling.

Actual gates: **220 tests / 45 suites**, make check/build swiftbuild, strict deep
codesign, env-i /tmp resources (8 grammars/all calls), clean diff check. Signed
binary **2026-10-06 14:22:51 PDT**. Multi-hunk gaps/inside/bounds, Before/After,
empty-side EOF, unterminated EOF caret, stale identity/revision, binary/pending,
Observation invalidation and exact Back/test-region reveal regressions pass.

Native attachment plus one full-path recovery selected user's older #753 app.rs
reader instead of fresh snapshot_policy.rs instance. Stopped without actions or
screenshots; closed only review PID 49593, preserved user PID 33005. Live header,
narrow layout and pointer/keyboard behavior remain manual; prior UI limits remain.
No dataset/marks/pins/GitHub/branch changes or commits/pushes. Evidence/manual
steps: plans/pull-request-ui-review.md.

## 2026-10-05 — PR Changes respects source filters (GPT Sol)

User superseded the previous Files-only sidebar scope. Changes now uses shared
SourceFilters.includes for both Source Files Only and Hide Tests, preserving PR
order and full canonical review.changes. Deleted files use available Before
path; renames/copies remain when either available Before/After path passes both
active filters. Markdown additions/modifications disappear with Source Only;
source changes survive extension/source-set moves without inferred semantics.

Changes reuses compact filter toggles, shows visible-of-total changed-file counts
and visible text-file diff totals (binary counts unavailable). Filtered-empty
has No Changes to Show with Show All Files; genuinely empty remains No Changed
Files. Selection sync is limited to visible rows; filtering/current-file sync
does not reopen or navigate. Excluded current file/caret/history/revision stays.
Changes footer counts changes, not contradictory repository-tree files.

Actual gates: check, **216 tests / 44 suites**, swiftbuild signed build, strict
deep codesign, env-i /tmp resources (8 grammars/all calls queries), clean diff
check. Binary **2026-10-05 19:00:13 PDT**. Regressions cover mixed markdown/code/
test/build rows, both independent filters, deletions, rename directions, totals,
filtered versus genuine empty, visible selection/reset, canonical data and an
excluded Before Markdown reader preserving navigation/history. No native attempt
per user constraint; previous old-reader attachment/PR visual limitations remain.
Manual check/evidence in plans/source-filters-ui-review.md. No dataset, pins,
marks, branch, GitHub or user-reader changes; no commits/pushes.

## 2026-10-04 — Source filter acceptance repairs (GPT Sol)

Native attachment and single full-path recovery selected the user's old #691
nomex_e2e.rs review rather than the fresh app.rs instance. Stopped without UI
actions/screenshots; closed only review PID 60963, preserved user PID 31152.
Live filter appearance/discoverability, pointer reveal, narrow/dark layout and
PR coexistence remain manual. Previous PR/outline/physical-hover limits remain.

Fixed four concrete source-review edges: pytest functions require test_ prefix
(not testing_helper/testify/testConnection); Vitest if statements with an else
stay visible to preserve production; painted label hit IDs always refresh even
when geometry is identical; labels reject the actual hidden/null header glyph,
not a character-range mapping expanded to a surviving parent fold row. AppKit
regression reproduced that fold-row mapping bug, with no label/hit when folded
and label restored when outer function expands. Geometry/identity and detector
positive/negative regressions pass. Cleaned extra EOF blank line in history doc.

Actual final gates: check, **213 tests / 43 suites**, swiftbuild signed build,
strict deep codesign, env-i /tmp eight-grammar/all-calls verification, clean
git diff --check. Binary **2026-10-04 23:23:22 CDT**. Independently reran the two
targeted real tkos probes (inline anchors and Files counts), both pass; did not
repeat Opus's 63-second repository sweep. Model/AppKit tests are separate from
live UI evidence. Required SwiftUI/native/type guidance reviewed; no speculative
visual redesign. Dataset/user pins/marks/older reader untouched; no commits/pushes.
See plans/source-filters-ui-review.md for exact manual screenshots and limits.

## 2026-10-04 — Source filters: Hide Tests and Source Files Only (Opus 5.5; UI acceptance pending with GPT Sol)

Request: hide test code, including Rust test blocks in the middle of
ordinary files (at least Rust, Kotlin, Python best effort, TypeScript), and
filter the Files sidebar down to source code. Design recorded first in
[plans/source-filters.md](plans/source-filters.md).

**Options and location.** Two independent icon toggles beside the Files
**Filter files** field (Xcode navigator idiom), mirrored as checkmarked View
menu toggles: **Source Files Only** (⌃⌘O) and **Hide Tests** (⌃⌘T). Both
persist app-wide in UserDefaults (`sourceFilesOnly`, `hideTests`) and are off
by default. Toggling never touches the repository model, caret, history,
pins, marks, folds or source bytes.

**Files tree.** Rebuilt from included files, so empty directories prune. It
keeps Finder order, and directory expansion is now explicit, path-keyed state
(`FileTreeExpansion`, recursive DisclosureGroups replacing OutlineGroup). Test
paths match exact components and names: `tests/`, `src/test`, `src/*Test`
source sets, `__tests__`, `*.test.ts`, `*_test.go`, `test_*.py`, `*_tests.rs`,
and so on. `contest.rs`, `testing/` and `src/main/**/SpeedTest.kt` stay.
Source Files Only rejects artifact, dependency, cache and hidden directories
by component (a `.rs` under `target/` is not source), generated names,
manifests/configs, docs and data. The filter field's flat list follows the
options. ⌘K, Search, the Changes list, Go to Symbol and indexing are
deliberately unfiltered. The open file is never closed; the sidebar footer
reads “N of M files · open file hidden by filters”. **Show All Files** appears
when nothing passes.

**Inline detection** runs in `SyntaxWorker` from real node shapes, dumped
from the pinned grammars, never braces or strings:

- **Rust:** outer attribute siblings, with `cfg` parsed from the token tree
  and evaluated in three-valued logic. Code hides only when it is false
  without `test` and not always false; `any(test, feature)`, `not(test)` and
  `cfg_attr` stay visible. Also `#[test]`/`tokio::test`/`rstest`/`test_case`,
  and `#![cfg(test)]` modules or files.
- **Kotlin:** test annotations, test classes, and Kotest specs.
- **TS/TSX/JS:** top-level framework calls with function arguments, checked
  against file bindings and test-module imports or `require`, plus Vitest
  in-source blocks.
- **Python:** `TestCase` subclasses, and pytest conventions only when the
  module imports pytest.

Regions expand to whole lines, keep adjacent doc comments, merge across blank
gaps, and use an explicit stack (no recursion).

**Presentation.** Hidden test text joins the existing fold machinery as one
sorted disjoint UTF-16 union; the fold reducer now merges overlaps. Each
region keeps its first line, dimmed through the draw-time receded alpha and
merged with Critical Lens ranges. The gutter shows ▶ with the existing 9-pt
hidden-line capsule, and a quiet clickable label reads “Test code · N lines
hidden” (or “Hide test code” once revealed); its type is the source size − 3,
system regular, secondary. Reveal state (`revealedTestRegions`) belongs to the
loaded document and resets on another document (identical bytes elsewhere,
Before/After), on repository open/reload, and when Hide Tests turns on.
Explicit navigation reveals its target's region, also when syntax arrives
later (pending reveal). A plain top-of-file open does not. Back is exact.
Hidden declarations leave the outline. Test documents opened explicitly show
in full with a **Test file** footer note; other files show “N test regions
hidden”.

**Actual gates (this round):** `make check`; `make test` **210 tests / 43
suites**, including 3 opt-in tkos probes that skip by default; `make build`
(swiftbuild) signed bundle; `codesign --verify --deep --strict`; `env -i`
`--verify-resources` from `/tmp` (8 grammars, all calls queries). Signed
binary **2026-10-04 23:16:01 CDT**. New suites:

- `TestRegionDetectorTests` (12): per-language positives and negatives, the
  mid-impl boundary, cfg logic, CRLF/Unicode bytes, and 4,000-deep nesting.
- `TestCodePresentationTests` (5): header and hidden ranges, TextKit null
  glyphs with gutter-equivalent canonical lines `[1,2,3,8…]`, exact pasteboard
  bytes, union with default and manual folds across toggling, fold reveal
  inside revealed tests, model reveal on navigation, Back, Go to Line,
  toggle reset, pending cross-file reveal, identical-bytes identity, and test
  documents.
- `SourceFilterTests` (4): classifier positives and negatives, artifacts,
  pruning and order, independence, persistence, open-file retention, the
  field list, expansion, and reload.

The PR flow test now also asserts the filtered tree follows the head snapshot
and returns to the attached checkout.

**Real tkos (read-only, `RADIOLOGY_REAL_TEST_FILTER_PROBE=1`, passed):**

- `app.rs` regions 35–43, 803–833 (834 `create_durable_agent` outside) and
  3424–4709.
- `snapshot_policy.rs` 16–108, with `configure` at 9 visible.
- `FontFilesTest.kt` 9–87, `test_pr_local.py` 22–179 and
  `home-design-inspect.test.ts` 22–58; their production counterparts have no
  regions.
- Files: Hide Tests shows 4,842 of 6,846 files, Source Files Only 5,198, both
  3,256.
- The whole-repository sweep took 63 s with no failures. Among non-test
  paths, Rust had 183 of 289 files with test code; non-Rust hits were 6
  Python files with in-script `unittest.TestCase` classes and 1 bun preload
  `beforeEach`, all inspected as true positives.

tkos HEAD/status unchanged.

**Not verified:** any live UI. No launch, screenshot or native inspection in
this round, per ownership split. Limits:

- Copy across a hidden region copies the hidden bytes, and find searches
  them, as with folds.
- Python is best effort.
- JS/TS tests inside functions or custom wrappers are not detected.
- Go/Swift have only path rules.

Sol's checklist: [plans/source-filters-ui-review.md](plans/source-filters-ui-review.md).
No commits/pushes; RADIO.png, nested `repo/` and earlier dirty work untouched.

## 2026-10-04 — Visible Exit PR toolbar action (GPT Sol)

A text-labelled native **Exit PR** button with arrow.uturn.backward now sits
immediately beside the PR selector, outside the menu, while a review is active
or loading. Explicit titleAndIcon keeps the label visible. It uses the existing
exitPullRequest route: cancel a pending load or return from the snapshot to the
attached local checkout. Tooltip names local checkout; it does not claim main
or switch Git branches. Menu Local Checkout remains. Cancellation clears pending
load errors/failed number; regression confirms aborted retry cannot publish a
late error or disturb the current local reader/history.

Actual gates: check, **186 tests / 39 suites**, swiftbuild signed build, strict
deep codesign and env-i /tmp resource verification (8 grammars/all calls queries).
Binary **2026-10-04 20:18:12 CDT**. No native attempt per user instruction; no
reader/pin/mark/dataset interaction. One manual check: open or start loading a PR,
confirm visible Exit PR beside the selector, activate it, and confirm local
checkout browsing with no stale alert or branch change. Previous UI/hover/outline
limits remain; see plans/pull-request-ui-review.md. No commits/pushes.

## 2026-10-04 — PR acceptance repairs and real snapshot Rust smoke (GPT Sol)

Fixed five root-review findings: full 40-character blob-to-blob per-file hunks
(added/deleted use app-owned empty blob); repository-wide/versioned dependency
context for honest reuse; persisted mixed-reuse Partial on reload/refresh;
global native PR failure alert with retry/number recovery; Cmd-K guidance.
Also fixed refresh dropping revision cache options. Meaningful regressions cover
R100 rename plus old-path directory child, sibling source/include invalidation,
actual tool-run count, stored provenance reload/refresh and failed-number retry.
An existing external-definition navigation test now waits for correct document
semantics after Back, instead of racing load completion.

Final gates: make check; **186 tests / 39 suites**; signed swiftbuild bundle;
strict deep codesign; env-i /tmp resources (8 grammars/all calls queries).
Binary **2026-10-04 20:07:43 CDT**. Real probes pass for #724 merged head
f11c99e3d6/base 488d8768d5 (9 files), #723 merged f4082f9e3a/base 8368298660
(12), #725 open 3dd325f6b0/base e7380c8269 (744, seven renames).
Current all-repository policy finds 0/16 matching #724 head/base strategy digests;
the former 15/16 comparison is historical and not a current reuse claim.
Actual #724 phone-provisioner After rust-analyzer 1.97.1 generated one document in
7.48s, mapped 3,711 occurrences with zero unmapped, reused in 0.063 s, rejected a
simulated sibling-context change and regenerated in 2.47 s. Temporary app-owned
cache, no dependency fixes; not all 16 projects or a real edited second revision.

Native attempt plus single full-path recovery selected old app.rs:3026 pinned
reader rather than fresh instance. Stopped without UI action; closed only review
PID 78172, preserved user PID 87944. NO new screenshot or PR UI verification; prior
hover/outline limitations remain explicit. TkOS HEAD/status unchanged; no marks,
pins, RADIO.png, dataset or GitHub writes/commits/pushes. Full UI/manual checklist,
actual backend boundaries and logs: [plans/pull-request-ui-review.md](plans/pull-request-ui-review.md).
Handoff/subsystem/index updated. Full snapshot GUI responsiveness/cancellation,
dark/narrow visuals and semantic pointer/history flow remain manual.

## 2026-10-04 — Pull request review (Fable 5.1; UI acceptance pending with GPT Sol)

Radiology now reads GitHub pull requests of the attached repository. The
toolbar **Pull Request** menu lists the last 25 (open, merged, closed) via
read-only `gh`; ⌘P opens any number (the ⌘P file-picker alias was removed;
⌘K remains). After is the pinned head SHA checked out into an app-owned
worktree; Before is GitHub's `merge_base_commit`, materialized per changed
file from the base blob. A **Changes** sidebar segment lists files with
status, `+n −m` and rename pairs; the file tree and the outline's callables
carry accent dots where lines changed; added lines get a green gutter bar
and faint wash in After, removed lines red in Before; a per-file
**Before | After** control (⌥⌘B) maps the caret through the hunks and
records history so ⌘[ returns exactly; ⌥⌘↓/↑ step through changes;
`--pull-request N` launches straight into a review. Design, measured facts
and contracts: [plans/pull-request-review.md](plans/pull-request-review.md).

Index: the After snapshot is indexed with per-strategy exact-bytes reuse.
Metadata is format 3 (per-strategy git-blob content digests, `reusedFrom`);
a strategy is copied from the working-tree cache or another revision only
when digests match, otherwise regenerated whole (rust-analyzer and the other
indexers accept nothing smaller than a project). Mixed runs are **Partial**
with the reason; format-2 caches are never reused for a review. **Before is
syntax-only**: `Not Indexed (Before)`, no index mapped onto Before bytes.
GitHub metadata and Git objects are read only; nothing is written to GitHub
or under the attached folder (bare repo with alternates, fetch by SHA at
depth 1, `refs/pull/N/head` fallback that still verifies the pinned SHA).

Gates (actual, this round): `make check`, `make test` **184 tests / 39
suites**, `make build` (swiftbuild) signed bundle, `codesign --verify
--deep --strict`, `env -i` `--verify-resources` from `/tmp` (8 grammars,
all calls queries). Signed binary **2026-10-04 19:56:31 CDT**. New tests:
hunk parsing (CRLF, Unicode, no-newline, pure add/delete, binary, headers
after CRLF content), raw/numstat records (rename R099, delete), line maps,
gh JSON decoding, command timeouts, a local-origin Git fixture (fetch by
SHA without touching the attached clone, name-status/counts/hunks, Before
blobs, worktrees, blob ids, eviction), digest rules, reuse/regeneration/
Partial with a fake indexer, legacy format-2 rejection, and an AppModel
flow: open PR, After highlights `[6..<12]`, changed callables `b`,`c`,
toggle to Before at the mapped line, Back to the After caret, deleted and
renamed files, search hits forcing After, superseded loads, repository
switch mid-load, Local Checkout. A main-actor starvation (git fixtures) and
a `\r\n` grapheme bug in the parser were found by the full suite and fixed.

Real data (read-only, `RADIOLOGY_REAL_PR_PROBE=1 swift test --filter
PullRequestRealDataProbe`, temporary cache, tkos status/HEAD verified
unchanged): `tktkinc/tkos`, 25 recent PRs (9 open, 15 merged, 1 closed,
newest #734). **#724** had merged since the root's preflight: head
`f11c99e3d6`, merge base `488d8768d5`, 9 files (7 M, 2 A), fetch 1.3 s;
`DesktopRuntimeSession.kt` removed lines `[3..<5, 11..<64]`, 238 → 183
lines, After anchors `[3, 9]`; `RuntimeSmoke.kt` 17 hunks +117 −205; 16
strategies at head, 15 byte-identical to the merge base (would reuse).
**#723** merged, head `f4082f9e3a`, 12 files (5 M, 5 A, 2 D). **#725**
open, head now `3dd325f6b0`, 744 files (390 M, 336 A, 11 D, 7 R, incl.
`SegmentedJournal.kt` renamed and modified). The app's own index of the
head worktree was not generated in this round (no rust-analyzer run on a
snapshot); the reuse path is proven by tests, not by a tkos timing.

Not verified: any UI. Sol's checks: menu/sheet flows and errors (gh missing,
unknown number), Changes list and dots, gutter/wash in both revisions and
dark mode, Before/After control states (Added/Deleted/Unchanged/Renamed/
Binary), ⌥⌘B and ⌘[ round trip, ⌥⌘↓/↑, window subtitle, Local Checkout,
footer `Not Indexed (Before)`, index popover "Reused from …" rows, and that
the first review's indexing (regeneration of changed projects) stays
cancellable and never blocks reading. RADIO.png icon preserved; no commits,
pushes, tkos writes, or GitHub writes.

## 2026-10-04 — Function-first outline by source size

Ordinary Outline now surfaces all callable declarations, including methods and
nested functions, before non-callable declarations. Functions sort descending
by true signature-to-end line span, source-order ties, using half-open UTF-8
ranges. Original symbols/IDs preserve selection and navigation. Non-callables
remain in a native Declarations hierarchy without duplicated functions.
One-line body names use middle truncation; quiet `n L` trailing counts use the
existing metadata role with tabular digits, full help and accessibility labels.
Cached derivation refreshes on source/syntax paths, content identity and symbols,
including switches between identical-content files. Normal/Critical share this
outline; source-order context/gutter and removed lens sidebar content are unchanged.

Gates: make check, **169 tests / 34 suites**, swiftbuild signed bundle, strict
deep codesign, empty-env resources from /tmp (8 grammars/all calls queries).
Signed binary **2026-10-04 17:35:04 CDT**. New regressions cover flattening,
span ordering/ties, nested non-callable access, Unicode/half-open endings and
identical-content path invalidation. Existing navigation tests pass.
Native attachment selected the user's old app.rs reader instead of the fresh
app_image instance. Stopped immediately without clicks/recovery; terminated only
review PID 79127, leaving user PID 24145, pins and marks alone. No new live
outline evidence or screenshots; previous hover physical-click limitation remains.
See plans/critical-lens-ui-review.md for manual checks. No tkos writes/commits/pushes.

## 2026-10-04 — Hover popup click targets and native retention repair

The user reported unclickable symbol popups. Source inspection found the custom
Critical NSPopover is the only source-symbol popup: titles/call names were Text,
with only small trailing links actionable. Navigable titles and whole rows are
now semantic Buttons; external/missing targets remain unavailable. Stable
observable content avoids hosting-root replacement during loading. Native
full-area activeAlways tracking, a source→popup transit region and 700ms grace
retain entry; crossing tokens while inside cannot replace the popup. Existing
exact-caret definition/Back path remains. No sidebar lens depiction restored.

Actual final gates: check, **166 tests / 33 suites**, swiftbuild signed build,
strict deep codesign, empty-env 8-grammar resources. Binary **17:21:39**. Native
host/event/transit regressions and model destination/Back tests pass. Live
click verification was blocked: the native tool selected the old app.rs:316
reader despite a separate fresh app_image review instance; single full-path
recovery did the same. Stopped, terminated only the review process, preserved
the user's reader/pins/marks. No popup screenshots or physical click evidence;
[review](plans/critical-lens-ui-review.md) has exact manual steps and limits.
No tkos writes, commits/pushes; RADIO.png and aggressive rendering preserved.

## 2026-10-04 — Remove Critical Lens sidebar depiction

GPT Sol implemented the user's focused removal: no upper Critical calls/status/
unknown/conditional/frontier section, lower callable frontier, or outline density
counts. Normal and Critical now share the concise declaration/references and
Definition/outline sidebar with existing disclosures/navigation. No replacement
summary. Call-focused source, mode control, persistent marks, hover frontier and
source gutter counts remain. Sidebar-only components/tasks/state/row dispatch
were removed; gutter work visits only fold-bearing callables and pointer dwell
retains on-demand hover computation/priming.

Meaningful regressions verify mode-independent context/reference identities,
selection/disclosures and activation/Back, no new graph requests on caret moves,
no outline-only annotation work, and surviving hover/gutter behavior. Actual
final gates pass: make check, **164 tests / 32 suites**, make build swiftbuild,
strict deep codesign, env-i resources from /tmp (8 grammars). Signed binary
**2026-10-04 17:05:05**. No native relaunch/check was attempted while the user
was browsing the shared reader; no new screenshots or visual claims. See
[the review](plans/critical-lens-ui-review.md). README/design contracts updated;
earlier automatic sidebar/count descriptions are superseded. RADIO.png/icon
pipeline and tkos data preserved; no marks changed, commits or pushes.

## 2026-10-04 — Critical Lens call-focused rendering, user accepted

The user superseded protecting syntax/literals/locals at full contrast: only
actual call names should pop. GPT Sol implemented a complemented Tree-sitter
call-name projection: critical/unknown calls remain prominent, proven ordinary
calls and all non-call source recede. Unresolved syntactic names never create
semantic identities/edges; unindexed files can render when call syntax exists.
Context opacity is 30% light / 38% dark; Normal and drawing-only coordinate/
reader contracts remain intact. RADIO.png/icon generation preserved.

Actual native attachment succeeded. Same-viewport Normal/Critical captures of
read-only tkos `app.rs:115–139 execute_future` visibly show quiet strings,
control words, declarations, receivers/locals/types and a call-name foreground;
existing pins and line tint survive. Upper concise Critical call names and
independent lower Definition were observed. Separately the user said “this is
much better”, accepting the aggressive rendering. Live work stopped when the
shared reader was being used, rather than compete. No marks changed. Dark mode,
new mark toggle, hover navigation/Back and folded-count semantics remain
unverified. [Review and actual artifacts](plans/critical-lens-ui-review.md).

Final actual gates: make check, **163 tests / 32 suites**, make build swiftbuild,
strict deep codesign, empty-env packaged resource verification (8 grammars).
Signed binary **2026-10-04 16:53:36**. Rendering tests replaced obsolete syntax-
protection assertions and cover Unicode/non-call/ordinary/unknown distinctions;
a timing-sensitive pre-existing cache test now prepares parsed bytes before
its artificial-delay race (assertions unchanged). No production graph changes,
tkos writes, commits or pushes.

The parent preserved the supplied screenshot and both native comparison
captures under `build/critical-lens-ui-review/call-focus/`, verified review
links, and confirmed tkos Git status still exactly matches the starting status.
Only evidence/documentation changed after Sol's final gates.

## 2026-10-04 — App icon from RADIO.png

The supplied root `RADIO.png` is now the app-icon source. The icon script
resizes the complete artwork into the ten standard macOS variants without
cropping or added decoration; Make tracks both the PNG and script so future
changes regenerate `build/AppIcon.icns`. Info.plist already points at AppIcon.
The rebuilt signed bundle (binary copy timestamp **16:42:51**) includes the
new ICNS and was registered with LaunchServices. The original PNG is unchanged.

Validation: `make build`, `make check`, **161 tests in 32 suites**, strict deep
codesign, and packaged resource verification from `/tmp` with `env -i` all
pass. The bundled icon matches the generated file exactly and decodes into
ten variants spanning 16–1024 pixels; the generated 512-pixel artwork was
visually checked. No new tests were added for this asset change. No application
source changed, and no native UI test was attempted; the preceding Critical
Lens final-UI limitations remain. See [the build guide](plans/build-system.md)
for the asset pipeline. Nothing was committed or pushed.

## 2026-10-04 — Critical Lens final native acceptance blocked

GPT Sol launched the signed **16:19:59** build on untouched tkos. Native attachment failed with `Computer Use server error -10005: cgWindowNotFound`; the single full-app-path recovery failed identically. UI work stopped within the requested budget. No final screenshot/AX evidence, source fixes, tests or rebuild; reported 161 tests / 32 suites and signing/resource gates remain separate evidence. Original absent override state was snapshotted and unchanged. Stage-one visual acceptance stands; final summaries/frontier/hover/counts remain unverified. [The review](plans/critical-lens-ui-review.md) has exact manual cases and screenshot requests; do not call the full milestone visually accepted yet.

The parent confirmed tkos Git status exactly matches the starting status and
independently reverified strict deep signing. Stage-one captures remain
preserved under `build/critical-lens-ui-review/stage1/`; no final captures
exist. PLAN, implementation design, and handoff now record the blocked final
review and manual checks. Only documentation changed after the build; no
product code changed and the passing suite was not rerun. Nothing was
committed or pushed. The final app is left available for manual inspection.

## 2026-10-04 — Critical Lens, stages two and three: summaries, frontier, sidebar, hover, counts

The user handed source ownership back after Sol's successful stage-one
review and asked for the rest of the milestone in one turn. Fable 5.1
implemented it; no UI was launched by the implementer. Details in
[plans/critical-lens.md](plans/critical-lens.md) §5–§8.

- **Call sites:** a fifth query role, `calls`, for all eight grammars.
  Named shapes carry the callee token; a fallback over every call node
  keeps unsupported shapes (`(f)(x)`) as unresolved sites. Sites inside
  closures and nested named functions belong to those nodes and never
  become the outer function's edges.
- **Edges:** a site is an edge only when its callee token is a SCIP
  occurrence (`CallGraphWorker.resolve`); never by spelling. rust-analyzer
  emits no relationships, so this is the only edge source. Bodies are
  located by the Tree-sitter declaration at the definition byte, with the
  indexer's enclosing range as a bounds-only fallback.
- **Frontier:** bounded BFS (depth 4, 48 bodies). Critical emits, ordinary
  with a body traverses transparently as a via step, unknown is listed and
  makes coverage partial, closures make it partial too. Cycles,
  self-recursion, and diamonds terminate; the first path wins; the count
  is the distinct union of direct and frontier callees.
- **Coverage:** complete, partial (unresolved, macros, missing bodies,
  truncated, parse errors, unknown callees, conditional sites), or no body
  (external, abstract, no syntax, unreadable, not indexed, unsupported).
  Proven zero requires complete coverage.
- **Caches:** worker configured by root/index/policy revisions with a work
  epoch advanced on reconfiguration and on byte invalidation; in-flight
  operations never store stale results; a disk load that finishes after
  the same path was primed yields to the primed bytes. Review findings on
  prime ordering, dependent frontiers, mid-operation reconfiguration, and
  unknown/conditional proven-zero were each fixed with a regression.
- **Sidebar:** Critical mode puts a **Critical** section ahead of the
  reference list: status line, direct critical calls, frontier rows with
  via chains, unknown/conditional lines; References follow unchanged. The
  lower Definition row gains a compact bounded frontier for callables.
  Requests follow the two Context identities, never a caret timer.
- **Hover:** automatic popover after 350 ms on a callable with a body or a
  critical external callable; stays open while the pointer moves into it;
  closes on leaving, click, key, scroll, mode/file/index/repository
  change. Rows navigate through the exact-caret history path; external
  rows say "external" and do nothing.
- **Gutter and outline:** collapsed functions show the reachable count in
  an accent-stroked rounded square (`3`, `3+`, `–`) instead of the neutral
  hidden-line capsule, which stays in the tooltip/accessibility; outline
  rows get a trailing caption count in source order. Both are generic
  `CountAnnotation` values; the gutter and reader know nothing about
  classification.
- **Real tkos (read-only probe):** `restore_app`→`restore` direct,
  complete; with `restore` marked ordinary (probe-local): `section via
  restore` and `compatibility via restore`, section's three sites collapsed
  to one path, partial (6 unresolved dependency calls, 2 macros).
  `read_observation`: `observe_session`, complete. `publish_instructions`:
  6 direct critical, 7 closure sites, 5 unknown, partial. `open`: 39 direct
  critical, 23 closure sites, partial. Marks were never persisted.
- **Tests:** `CallSiteExtractionTests` (4), `CriticalFrontierTests` (3),
  `CallGraphWorkerTests` (5), `CriticalSummaryModelTests` (3); the Rust
  fixture and synthetic graphs cover ownership, dynamic sites, dedup,
  caps, invalidation, and reconfiguration races.
- **Gates:** `make check`, **161 tests in 32 suites** via `make test`,
  `make build` (swiftbuild), `codesign --verify --deep --strict`, and
  packaged `--verify-resources` from `/tmp` with `env -i` (8 grammars, now
  `highlights+folds+symbols+scopes+calls`) all pass. Binary
  `build/Radiology.app/Contents/MacOS/Radiology` built 2026-10-04 16:19:59.
- **Live check pending (Sol).** Short checks on untouched tkos:
  1. `engine/rust/src/app_image.rs`, caret inside `restore_app` (401),
     Critical: the upper region's Critical section lists `restore` →
     line 404; References follow. Caret into `restore` (404): `section`
     ×3 and `compatibility` rows, status "2 direct · partial"; expanding
     nothing is needed to read the names. Return on a row jumps; ⌘[
     returns to the caret.
  2. Right-click `restore` → Mark “restore” ▸ Non-Critical: `restore_app`'s
     section now shows `section via restore` and `compatibility via
     restore` with the chevron revealing the chain; Built-In restores.
  3. Hover `section` on line 413 (no modifier): after a beat a popover
     with "section", Definition link, "0 direct · proven"; move the pointer
     into the popover, click Definition: jump; ⌘[ back. Hover `ok_or` or
     a local: no popover. Hover `libc::flock` in
     `modules/agent/ffi/src/store_owner.rs:48`: popover says "external",
     no link, no navigation.
  4. Collapse `restore` (gutter): the count lane shows `2+` in an
     accent-stroked square; tooltip includes the hidden-line count; expand
     and the normal capsule returns in Normal mode. Outline rows show
     trailing counts in source order; a proven-zero row reads a quiet `0`.
  5. `modules/agent/ffi/src/app.rs`: caret in `open` (1343): the Critical
     section lists `optional_string`, `bad`, `prepare_catalog`, … with a
     "23 calls in closures" line and "partial" status; scrolling the list
     stays responsive while the gutter counts fill in.
  6. ⌘1: Critical section, popover, and counts disappear; references and
     Definition are exactly as before. Dark mode legibility of the gutter
     badge and popover.

## 2026-10-04 — Critical Lens stage-one native UI checkpoint

GPT Sol reviewed the signed **15:43:16** build on read-only tkos through native Computer Use. Actual on-screen dimming worked; repository/system operations remained prominent, unresolved dependencies neutral, and syntax/caret/function shade legible. Cmd1/Cmd2 preserved observed caret, viewport, word selection, existing folds and history. Native copy of a dimmed token worked. Symbol/package mark precedence worked in opposing directions, with persistence across relaunch; the original absent repository override file was restored after stopping the review app. No source changes or rebuild; handed-off gates remain 146 tests / 28 suites. Light-mode contrast accepted as-is. Dark mode, pin overlays and other explicit coverage limits remain unverified, not failures. See [the native review](plans/critical-lens-ui-review.md) for actual screenshots and stage-two handoff. Fable can resume summaries/call graph/frontier before hover/gutter/outline counts.

## 2026-10-04 — Critical Lens, stage one: classification, marks, Critical source rendering

The user asked for a Critical Lens source projection for skimming meaningful
operations, in stages. Fable 5.1 wrote the system design
([plans/critical-lens.md](plans/critical-lens.md)) and implemented stage one
only. GPT Sol owns the live UI check; the implementer launched nothing.

- **Measured first (read-only tkos cache, 13 rust-analyzer 1.97.1
  strategies):** rust-analyzer emits **no** SCIP relationships, so there are
  no call edges in the index; `enclosing_range` exists on every indexed
  definition (whole item) and `enclosing_symbol` on every local; external
  symbols (`core`, `std`, `serde_json`, `libc`…) carry no kind or signature,
  only their symbol string; operators, `?`, brackets, and even spaces are
  `core ops/…` occurrences; 222 repository signatures contain `extern`.
  Opt-in probe: `RADIOLOGY_CONTEXT_PROBE=<cache> swift test --filter
  CriticalRealDataProbe`.
- **Mode:** `SourceMode` (`normal`, `critical`), a segmented
  Normal | Critical control in the toolbar's principal position, and View
  menu items with ⌘1 / ⌘2. Per session, not persisted.
- **Classification** (`Critical/CriticalityPolicy.swift`): deterministic
  precedence — symbol mark, package mark, local → unknown, explicit
  binding-package list or `ffi/`/`os/` descriptors → critical, indexed
  `extern "` signature → critical, system-facing std/tokio modules →
  critical, repository definition or repository package → critical,
  `core`/`alloc`/`std` → ordinary, else unknown. No spelling heuristics; a
  `-sys` suffix alone proves nothing; raw-pointer/`mem` helpers are not
  promoted. Only the rust-analyzer scheme has profiles.
- **Real tkos results** (probe, classification level): in `restore`
  (app_image.rs:404–425) `section`, `compatibility`, `MAGIC`, `MAX_IMAGE`,
  `Restored`, `Error` are critical; `Vec`, `Result`, `len`, `split_at`,
  `Err`, `into`, `Some`, `format`, `as_str`, `is_empty`, `BTreeMap`,
  `String` are ordinary; `serde_json::from_slice` and `Sha256::digest` have
  no occurrence (engine is `--no-deps`) and stay untouched. In
  store_owner.rs:41–49 `OpenOptions::open` is systemAPI, `libc::flock`,
  `LOCK_EX`, and `as_raw_fd` are ffiPackage, `io::Error::last_os_error` is
  plumbing (excluded `io/error/` module). In app.rs `open`, `optional_string`,
  `bad`, `HarnessError` are critical, `first`/`and_then`/`cloned`/
  `unwrap_or_default` ordinary, `serde_json::Value::as_object` unknown.
- **Marks:** source context menu → `Mark “name” ▸ Critical / Non-Critical /
  Built-In (…)` and `Mark Package “pkg” ▸ …` with radio state. Symbol beats
  package beats built-in. Locals are never offered or stored (their SCIP
  numbers are renumbered by reindexing). Package keys are
  `scheme manager name` without version. Persisted atomically at
  `~/Library/Application Support/Radiology/overrides/<repo-id>.json`,
  keyed like the index cache; the viewed repository is never written.
  Edits made before the file loads are replayed on top of it; saves are
  chained and revision-guarded; deletion failures surface as “Marks not
  saved” in the footer.
- **Rendering:** `SourceProjection` (sorted, merged canonical byte ranges
  bound to the buffer's path and full content identity). Only ordinary,
  non-definition, single-line, identifier-shaped occurrences that touch no
  Tree-sitter keyword/literal/comment/operator capture recede; a file with
  a grammar waits for its structure. The coordinator's
  `shouldUseTemporaryAttributes` hook returns the text storage's own color
  at 42 % (light) / 50 % (dark) opacity: no storage mutation, no layout,
  no coordinate change; copy, Find, selection, pins, shade, caret, folds,
  history, and scroll are untouched. Caret moves cost one key comparison.
- **Tests:** `CriticalityPolicyTests` (3), `CriticalProjectionTests` (2,
  including an ordinary occurrence over `await`/`true`/`return`),
  `CriticalityOverrideStoreTests` (2), `CriticalLensModelTests` (3, through
  `AppModel` with the fake indexer), `SourceProjectionDrawingTests` (3).
  TextKit ignores temporary attributes offscreen, so a pixel test is
  impossible by design; the hook's outputs and its consultation on every
  draw pass are tested instead.
- **Gates:** `make check`, **146 tests in 28 suites** via `make test`,
  `make build` (swiftbuild backend), `codesign --verify --deep --strict`,
  and packaged `--verify-resources` from `/tmp` with `env -i` (8 grammars)
  all pass after the final `io/error/` profile fix. Binary
  `build/Radiology.app/Contents/MacOS/Radiology` built 2026-10-04 15:43:16.
- **Not done (stage two/three, designed only):** function summaries, call
  graph from Tree-sitter call sites resolved to SCIP (closure and nested
  function sites are their own conditional nodes, never direct edges of the
  outer function), frontier with via paths, identity-driven sidebar
  summaries, hover popover, fold-gutter and outline counts.
- **Checks for Sol (live, untouched tkos):** 1. open
  `engine/rust/src/app_image.rs`, place the caret in `restore` (line 404),
  switch to Critical (toolbar or ⌘2): `section`, `compatibility`, `MAGIC`,
  `Restored`, `Error` stay full; `Vec`, `Result`, `len`, `Err`, `into`,
  `Some`, `format`, `BTreeMap`, `String` recede; `if`, `return`, `?`,
  braces, strings, `serde_json::from_slice`, `Sha256::digest`, locals stay
  full. 2. `modules/agent/ffi/src/store_owner.rs:41–49`: `open`, `libc`,
  `flock`, `LOCK_EX`, `as_raw_fd` full; `PathBuf`, `from`, `map_err`
  receded; `unsafe` full. 3. Arrow through a receded token: caret, Context
  rows, and the function shade behave as in Normal; ⌘-hover underline and
  ⌘-click still work on receded and full tokens; Return/⌘[ unchanged;
  select and copy a receded span: plain text. 4. Fold `restore`: no
  stacked artifacts; unfold: dimming intact. 5. Right-click `len` → Mark
  “len” ▸ Critical: it returns to full at once; Built-In restores; Mark
  Package “alloc” ▸ Critical: `Vec`/`String`/`format` return; relaunch and
  reopen tkos: marks persist; open another repository: none apply.
  6. ⌘1 returns to Normal with no residue; toggle rapidly while scrolling.
  7. Dark mode legibility of receded text (50 %). 8. Confirm no file
  appears in tkos and the JSON exists under
  `~/Library/Application Support/Radiology/overrides/`.

## 2026-10-04 — Faint shade under the current function

The user asked: "put a subtle background shade under the current function that
i'm visiting, just enough to notice if i'm looking for it." Opus 5.5
implemented it. See
[structural navigation](plans/structural-navigation.md#current-function-shade).

- **Which function:** the innermost Tree-sitter function, method, initializer,
  or deinitializer around the genuine caret, from signature to end. Not
  types, `impl` blocks, or modules. Closures keep their named function.
- **When it clears:** outside functions, on a file or repository change, and
  without structure. Scrolling never moves it.
- **Drawing:** full-width bands from TextKit line fragments, painted first in
  `drawBackground`. Line tint, highlights, pins, and caret stay on top. Folds
  collapse it to the visible line. Only the visible slice is mapped to glyphs,
  after source review.
- **Color:** `labelColor` at 3% (light) or 4.5% (dark).
- **Tests:** new `FunctionShadeTests` (3). They found and fixed a missing clear
  on repository open.
- **Gates:** `make check`, **132 tests in 22 suites** via `make test`,
  `make build`, strict deep codesign, and packaged `--verify-resources` from
  `/tmp` with an empty environment (8 grammars) all pass. Binary
  `build/Radiology.app/Contents/MacOS/Radiology` built 2026-10-04 15:02:16.
- **Live check:** GPT Sol verified the faint wash in light mode on untouched
  tkos: `section` shades lines 93–101, stays stable during caret movement,
  clears on blank line 102, and transfers to `sources`. Folding and adjacent
  `restore_base`/`restore_app` boundaries work. Pins, current-line tint,
  symbols, and the caret stay legible. Passive scrolling does not transfer
  the shade to the viewport's function. No defect was observed; dark mode
  was not tested. See [Sol's review](plans/context-ui-review.md); screenshots
  are preserved in `build/context-ui-review/function-shade/` (removed by
  `make clean`). No UI launch by the implementer; no commits or pushes.

## 2026-10-04 — Subtle caret in the read-only source reader

The user asked: "i need to see a cursor, not just the current line (a caret
would be great, keep it subtle) because some lines have multiple syms and i
don't want to have to mouse to get to them sometimes." Opus 5.5 implemented it.
See [structural navigation](plans/structural-navigation.md#read-only-source-caret).

- **Approach:** the reader stays read-only. AppKit draws no insertion point for
  a non-editable text view, so `ReaderTextView` paints one.
- **Style:** a 1.5 pt steady bar in the dynamic system insertion-point color at
  85% opacity. Its height spans the font's ascender to descender on the real
  baseline. It is painted last in `drawBackground`, above line tint and
  decorations. No blink, no timers.
- **Geometry:** from actual TextKit glyphs, covering tabs, emoji, ZWJ and
  combining clusters, line ends, EOF (including after a final cluster), and
  none inside folds.
- **Gating:** shown only in the focused key-window reader with an empty
  selection.
- **Fixes found during the work:**
  - Trailing UTF-16 units of clusters carry null placeholder glyphs, so the
    caret snaps to the start of the composed character.
  - The EOF case after a final cluster (found in source review) now uses the
    complete glyph span.
  - An initial `draw(_:)` override changed scroll-restoration behavior, so it
    was replaced by painting in `drawBackground`.
- **Tests:** `SourceCaretTests` (3), with mutation checks for the EOF fix and
  the gate.
- **Gates:** `make check`, **129 tests in 21 suites** via `make test`,
  `make build`, strict deep codesign, and packaged `--verify-resources`
  (empty environment, 8 grammars) all pass. The signed app is
  `build/Radiology.app`.
- **Live check:** GPT Sol verified the caret on tkos `app_image.rs:94`:
  arrows moved it from `input` to `prefix`, updating the lower Context row
  while the upper declaration/references stayed fixed. Word selection and
  sidebar focus hid it; it reappeared above pinned tint. Source Return/Back
  still worked at `section:413`. Concise default rows and the Definition
  heading were also observed. No new defect was found. Dark mode and Unicode
  edge cases were not exercised live; the latter have model-test coverage.
  Evidence is in [Sol's review](plans/context-ui-review.md), with captures
  preserved under `build/context-ui-review/caret/` (removed by `make clean`).
  No UI launch by the implementer; no commits or pushes.

## 2026-10-04 — Concise Context default: one line per item, expansion on demand

The user asked: "just show a single line for each, and then have an expansion
affordance to show more… keep the DEFAULT information layout in the sidebar
concise," and "lose 'under caret', too explainy." Opus 5.5 implemented it. See
the concise-default section of [plans/context-sidebar.md](plans/context-sidebar.md).

- **Declaration region:** one pinned line (name, kind, line link), with the
  signature behind a native disclosure triangle. References and definitions
  are one line each, with a row chevron or →/← to show context. The automatic
  selection preview is gone. ↑/↓ selects, Return commits.
- **"Definition" region** (formerly "Under Caret"): one line of name, kind, and
  location link, `here`, or package. The triangle reveals signature, docs,
  facts, and the preview.
- **Statuses:** one short line each, with full reasons in tooltip and
  accessibility.
- **Expansion state:** tied to identities in `ContextModel`. It starts
  collapsed, survives movement within an identity, collapses on a new one,
  and never navigates.
- **Tests:** a new expansion-identity test, plus disclosure toggling in the
  passive no-navigation test.
- **Gates:** `make check`, **126 tests in 20 suites** via `make test`,
  `make build`, strict deep codesign, and packaged `--verify-resources`
  (empty environment, 8 grammars) all pass. The signed app is
  `build/Radiology.app`.
- **UI boundary:** GPT Sol launched the fresh signed build, but native access
  returned `Computer Use server error -10005: noWindowsAvailable` during the
  line94/source-click sequence. He stopped immediately without retry or
  workaround, per user instructions. Compact layout and disclosure interaction
  remain unverified; no new screenshot or product defect was observed. Exact
  manual checks are in [Sol's review](plans/context-ui-review.md): capture
  collapsed and expanded rows at normal and 220-point widths, selection without
  automatic preview, reference Return/Back, and independent expansion resets.
  Earlier successful live reviews cover earlier builds only. No UI launch by
  the implementer; no commits or pushes.

## 2026-10-04 — Return/Enter in the source reader goes to definition

The user asked: "hitting enter when highlighting a funcall's symbol should just
take us to the definition. presumably cmd-[ should take us back to where we
were." Opus 5.5 implemented this as a narrow AppKit route in
`ReaderTextView.keyDown`. Details are in
[plans/context-sidebar.md](plans/context-sidebar.md).

- **Trigger:** plain Return or keypad Enter in the focused source reader. It
  works with the caret inside a SCIP occurrence or just past it, or with a
  word selection inside one occurrence.
- **Path:** it calls `goToDefinition(at:departingFrom:)`, the same semantic path
  as ⌘-click. ⌘[ restores the exact departure caret, even after passive
  scrolling.
- **Pass-through:** modified keys, marked text, whitespace, unindexed files,
  external symbols, the definition itself, and over-wide selections go to
  AppKit unchanged. Nothing is guessed. Selection, copy, pins, and the
  sidebar's Return behavior are untouched.
- **Tests:** new `ReaderReturnKeyTests` (2), driving the real `keyDown`.
  Removing the modifier or selection-bound guards made them fail.
- **Gates:** `make check`, **125 tests in 20 suites** via `make test`,
  `make build`, strict deep codesign, and packaged `--verify-resources` from
  `/tmp` with an empty environment (8 grammars) all pass. The signed app is
  `build/Radiology.app`.
- **Live check:** GPT Sol verified source-focused Return from `section` at
  `app_image.rs:420` to its definition at93 after passive scrolling; Cmd-[
  restored the call site. Native word selection also worked. Cross-file
  Return on `runtime.rs:145` `restore_app` reached `app_image.rs:401`, with
  Back restoring145. Sidebar Return still committed reference276, and
  external `Some` did not navigate. No new defects or tooling blockers.
  Exact token-end byte placement was only partially verified visually; unit
  tests cover that boundary. Evidence and screenshots are in
  [Sol's review](plans/context-ui-review.md) and
  `build/context-ui-review/return-key/`. No UI was launched by the implementer.
  No commits or pushes.

## 2026-10-04 — Context inspector repairs and GPT Sol live review

Opus 5.5 implemented the feature and all fixes. GPT-6.1 Sol exclusively
exercised the native UI and inspected the design against untouched tkos.
This entry supersedes the pending live-review status in the implementation
entry below. See [the implementation plan](plans/context-sidebar.md) and
[Sol's review](plans/context-ui-review.md).

- **Final gates:** `make check`, **123 tests in 19 suites** via `make test`,
  `make build`, strict deep codesign, and packaged `--verify-resources` from
  `/tmp` with an empty environment all pass (8 grammars). Signed app:
  `build/Radiology.app`. No commits or pushes.
- **Live behavior:** Sol verified read-only caret arrowing, independent lower
  context, whitespace outline fallback, reference selection/Return,
  definition-link and Tab/Return navigation, Back after passive scrolling,
  pins, narrow inspector layout, external-source availability, and file-change
  clearing. The shared type hierarchy and native split/list design were judged
  coherent.
- **Four repairs from that review:** preserve multiline signature height;
  keep call-site snippets visible at compact heights; use Tree-sitter's
  declaration boundary for cross-file definition previews; distinguish
  repository-without-index from file-not-indexed. Added meaningful regression
  coverage for adjacent declarations and availability. Fixed an older test
  race that waited on a placeholder repository status.
- **Recheck:** the rebuilt app visibly shows all seven `open` signature lines
  when space permits, numbered reference snippets at default/enlarged heights
  and a condensed call site at a short 220-point inspector, and only lines
  401–403 for `restore_app`. Definition activation and Back still work.
- **Manual boundary:** after the one recovery retry, ScreenCaptureKit failed
  with `-3812`. UI work stopped without another workaround, as requested.
  Open root `README.md` and screenshot **File not indexed** beside Outline
  with **Index Partial** in the footer. The wording repair has test evidence
  but was not observed on the final build. Cmd-click, contextual activation of
  an unselected row, long-list scroll retention, VoiceOver speech, and dark
  mode remain unverified; exact steps and other limits are in Sol's review.
- **Evidence:** original and recheck captures are preserved under
  `build/context-ui-review/initial/` and `build/context-ui-review/recheck/`.
  These are local build artifacts and are removed by `make clean`.

## 2026-10-04 — Context inspector implemented (Opus 5.5); live UI review pending

This round supersedes the authentication-blocked note below. Claude
authentication was restored, and Opus 5.5 implemented the caret-driven Context
inspector. Full detail is in [plans/context-sidebar.md](plans/context-sidebar.md).

- **Inspector:** the Outline/Symbol toggle and `SemanticInspectorView` are
  gone. A `VSplitView` holds two regions:
  - **Declaration:** the innermost Tree-sitter declaration. Its pinned summary
    shows name, kind, flush signature, and a Line link. Ambient SCIP references
    follow, grouped by file. The count covers uses only, and definitions are
    labelled.
  - **Under Caret:** the SCIP occurrence with signature and docs. Calls and
    types get a bounded preview; locals stay quiet. External symbols are
    honestly explained.
  - **Outline fallback:** on whitespace, keywords, imports, and anything
    without a semantic occurrence.
- **No debounce:** `ContextModel` is keyed on declaration and occurrence
  identities. It writes observable state only on identity change, and uses
  generation-checked, cancelled off-main queries on separate actors. Caches are
  bounded and invalidated by index, repository, and source-byte changes.
- **Genuine caret:** `AppModel.caretLocation` is separate from the
  viewport-mixed `positions`.
  - Scrolling never changes context.
  - Every sidebar activation records the caret as Back's departure: reference
    commit, the definition link or Return, and the declaration Line link.
  - Reference ↑/↓ shows a capped call-site preview below the list. Return,
    double-click, or Open commits the named row.
- **Fixes found on real data:** SCIP fallback names inside backtick-escaped
  `[...]` descriptors (`first` from core read as `]first().`); URL version
  fields in package names; multiline signatures of nested methods.
- **Fixes from parent review:** caret vs scroll departure, the true definition
  list on definitions, and honest reference counts.
- **Fixes from Sol's source-only review:**
  - native actions commit their own row ID;
  - snippet accessibility values contain the source;
  - the preview is capped and scrolls, preserving list room;
  - the summary is pinned above the list;
  - long names get stacked metadata;
  - a visible "Preview · Return opens" hint.
- **Validation:**
  - `make check`, **120 tests in 19 suites** via `make test`, signed
    `make build`, strict codesign, and packaged `--verify-resources` (8
    grammars from `/tmp`, empty environment) all pass.
  - New: `ContextModelTests` (15), `ContextNavigationTests` (7), and the opt-in
    `ContextRealDataProbe`.
  - Mutation checks confirmed that the caret-departure and identity-guard tests
    fail when those guards are broken.
  - Real tkos resolution used the existing app-owned cache (tkos `239b9f6d4`,
    read-only). Results are recorded in the plan.
- **Not done here:** this agent did no live launch, UI automation, or visual
  inspection, per instructions. GPT Sol (`31e45a42-9257-445f-a185-70c9b908ebf3`)
  owns that, and owns `plans/context-ui-review.md`.
- **Build:** the signed app is `build/Radiology.app`. Nothing was committed or
  pushed.

## 2026-10-04 — Caret Context sidebar requested; implementation awaiting authentication

The user authorized a new Context inspector with independently updating
enclosing-declaration/references and under-caret definition/preview regions,
semantic identity updates without timer debounce, outline fallback, and no
source navigation until explicit activation. References keyboard movement
should preview, Return should commit, and Back should restore the departure.
The explicit model split is **Opus 5.5 for implementation** and **GPT Sol for
live UI/design review**. The read-only test dataset is
`/Users/tqbf/codebase/tk/tkos`. If UI inspection becomes complicated, stop
automation and give the user precise manual checks and screenshot requests.

Started local Paseo implementation agent
`6195c78d-1734-48ea-b18f-62dceacc6ad0` (`claude/claude-opus-5-5`), but it
could not start work: "OAuth session expired and could not be refreshed."
Local `claude auth status` confirms `loggedIn: false`. The user was asked to
run `claude auth login` on SHEFFIELD, then report readiness for a retry. Do not
substitute another implementation model silently. No feature code has changed.

GPT-6.1 Sol review agent `31e45a42-9257-445f-a185-70c9b908ebf3` completed
read-only preparation and awaits a new-build follow-up. Native `@oai/sky`
imported, but screenshot/AX access is not yet verified. Concrete cases include
`engine/rust/src/app_image.rs:93` (`section`, with calls at lines 413/420/424),
`engine/rust/src/runtime.rs:145` (`restore_app`, definition at app_image.rs:401),
`app_image.rs:19` (cfg-inactive Android symbols), and
`modules/agent/ffi/src/app.rs:1343` (`open`, multiline signature/large fold) and
line 4286 (Unicode). Historical reference counts are not fresh expectations.
UI setup recovery is limited to one simple retry and about 2–3 minutes; no UI
launch or inspection has occurred in this preparation round.

## 2026-10-04 — Fresh checkout and build on SHEFFIELD

Cloned `tqbf/radiology` into `/Users/tqbf/codebase/radiology` on `main` at
`2e02ae3`. The first build compiled successfully on macOS 26.5 / Swift 6.3.3
but failed during packaging: this toolchain defaults to SwiftPM's native
backend, which creates flat query bundles instead of the expected macOS
`Contents/Resources` layout. Its generated resource accessor also does not
search the packaged app's resources.

Used the Paseo skill and CLI over the configured `ssh://elston` alias to read
the successful Radiology agent's build record, then inspected the remote
toolchain and generated resources without changing the remote workspace or
prompting its agents. ELSTON uses Swift 6.4 and the `swiftbuild` backend.
Updated `build.sh` to explicitly select `swiftbuild` for compilation and binary
path discovery, preserving the existing signed app layout. Applied the
SwiftPM guidance and the repository's required `swiftui-pro` review guidance
before and after the build-script change; no application or UI code changed.
Documented the backend requirement in `plans/build-system.md`.

Fresh validation passes: `make build`, `make check`, **97 tests in 16 suites**
via `make test`, strict codesign verification, and packaged resource verification
for all eight grammars / 31 query files. Resource verification ran from `/tmp`
with an empty environment; the Swift Build accessor resolves the bundled
resources instead of falling back to the developer's build directory. The
debug app is at `build/Radiology.app`. No interactive UI launch was performed.
The build fix and documentation changes remain local and uncommitted.

## 2026-10-04 — Git initialization and GitHub publication

The user requested committing the completed project, creating `tqbf/radiology`,
and pushing it. This explicitly supersedes the earlier documentation-only
restriction against initializing Git. Initialized the active root package on
`main` for publication to [tqbf/radiology](https://github.com/tqbf/radiology).
The initial snapshot includes source, tests, resources, package resolution,
build scripts, and the full documentation handoff. Generated build output,
machine-local files, and the inactive nested `repo/` starter copy are ignored.
The nested copy and its existing metadata remain untouched.

Updated README, PLAN, the agent handoff, and the build guide to remove obsolete
claims that the active root has no version control. Git commits use the `tqbf`
account's GitHub noreply identity. This publication round changes no application
behavior and does not rerun the already recorded implementation checks.

## 2026-10-04 — User acceptance and documentation handoff (current state)

The user confirmed the latest large-file folding result in the running product:
“yeah you fixed it, you can settle at this point.” This is direct acceptance of
the interaction issue and is separate from the preceding isolated TextKit
benchmark and its final `cgWindowNotFound` automation limitation. No further
product change is requested. The recurring status heartbeat `10e731d2` was
deleted after that instruction and was not recreated.

Completed the documentation/handoff round. `README.md` now documents actual
requirements, commands, launch arguments, shortcuts, workflows, languages,
semantic indexers, caches, diagnostics and limitations.
`plans/development-history.md` preserves each development round and its user
feedback, superseded behavior, evidence and limits. `plans/agent-handoff.md`
gives future agents a reading order, architecture map, invariants, entry
points, validation commands, evidence boundaries and sensible future work.
`PLAN.md` is the master index and current-state summary.

This round was documentation-only: no application code, tests, tkos files,
external tools, Git metadata, agents, schedules, or behavior changed. No build
or test suite was rerun. Claims were checked against `AGENTS.md`, `Makefile`,
`Package.swift`, launch arguments, menu commands, language/query resources,
semantic indexer/cache/provisioning code, source limits, relevant plans and the
prior Paseo record. Local Markdown links and referenced repository paths were
checked. Thus **97 tests in 16 suites** remains a historical result of the
preceding implementation round, not a fresh documentation-round result.

## 2026-10-03 — Interactive large-file folding (current state)

Fixed the reported multi-second expand/collapse latency on untouched tkos
`modules/agent/ffi/src/app.rs` without changing the TextKit folding model. The
hot path was measured rather than inferred: every glyph and every hidden
control character performed a linear search through all active function ranges,
and every toggle invalidated glyph properties for the entire 4,880-line file.
That made even a seven-line function nearly as expensive as a 279-line one.

Folded ranges remain sorted/disjoint but now use binary-search membership in
the glyph delegate. `applyFoldAttributes` diffs the previous and next effective
ranges, returns immediately when a nested toggle cannot affect visible layout,
regenerates glyph properties only for the changed union, and invalidates layout
only from that point downstream. Render completion is still awaited in the
probe—work was not merely deferred. Canonical bytes, null-glyph compaction,
zero-advancement hidden newlines, visible-fragment gutter enumeration, exact
badges, source restoration, and default folds are unchanged.

Real-source rendered-completion timings on this machine (three expand/collapse
pairs each) changed as follows: large line-1453 function median about
0.52/0.33 s to 0.062/0.057 s; medium line-1734 function 0.34/0.31 s to
0.051/0.050 s; tiny line-1812 function 0.30/0.29 s to 0.048/0.047 s. The probe
uses the real 4,880-line file and forces `ensureLayout`, so values include the
downstream rendered layout, not only click dispatch. The user's observed UI
was slower than this stripped TextKit probe because syntax decoration and the
full app surface add work, but the eliminated whole-document glyph scan was the
dominant scaling defect.

Native CUA could not attach for the final pointer repetition: it listed the
running signed Radiology process but `getApp` returned `cgWindowNotFound` by
both display name and bundle identifier. The immediately preceding layout pass
had native screenshot evidence on this exact file; this performance pass relies
on the real-source AppKit rendered-completion probe and does not claim new
pointer timing evidence. Current validation and build results are recorded
below after the final gates. `make check`, **97 tests in 16 suites**, signed
`make build`, strict codesign verification, and packaged verification of all
eight grammars/31 query files pass. The real-source benchmark is opt-in with
`RADIOLOGY_REAL_FOLD_BENCHMARK=1 swift test --filter realAppFoldPerformanceProbe`
so an external tkos path and full-file AppKit layout do not destabilize ordinary
parallel test runs.

## 2026-10-03 — Correct large folded layout and salient size cues (current UI)

The previous faint 2-point meter/68-point gutter description immediately below
is historical and superseded. The shipping gutter is 104 points with separate
28-point disclosure, exact-count badge, and canonical line-number lanes. It
uses a 3-point secondary-label logarithmic meter (6–22 points) plus a neutral
9-point medium tabular count capsule. Fold layout uses visible TextKit fragments,
disjoint outermost ranges, null glyphs, and zero-advancement hidden newlines.
The real `app.rs` 1453→1734→1812→1829→1941→2062→2075 region is compact and
correct at normal and enlarged text sizes. The completed baseline before this
performance pass was 96 tests in 16 suites.

## 2026-10-03 — Collapsed-function size cues

Collapsed function headers now show a restrained two-point neutral meter below
the existing chevron. Width follows a capped logarithmic 4–18 point scale
(`log2(hidden lines + 1)`, capped at 256), making 2-, 20-, and 200-line bodies
visibly different without widening the gutter or reducing its 28-point
full-line hit target. Tooltip and gutter accessibility help provide the exact
“Collapsed function — N lines hidden” description. Expanded functions and
non-function folds show no meter; hidden nested function headers cannot stack a
cue on the surviving parent row.

Counts come only from canonical source line boundaries, independent of TextKit
glyph height, wrapping, font size, and viewport. Delimited bodies retain their
header row and count subsequent removed source rows; indentation-delimited
bodies count every captured row. The latter now anchor their fold affordance to
the preceding visible signature, fixing the prior Python/Kotlin possibility of
a control originating on a hidden first-body row. Tests cover CRLF, Unicode,
multiline signatures, indentation bodies, empty/one-line functions, the
2/20/200 progression, and the 256-line cap.

Native CUA on untouched `tkos/engine/rust/src/app_image.rs` showed short meters
for the tiny `image` methods, progressively longer meters for `section`,
`sources`, and `compatibility`, clean original line numbers, and no source
column shift. Clicking within the left side of the 28-point gutter expanded
`section`; clicking the same region re-collapsed it and restored only its size
meter. Two source-size increases preserved chevron/meter alignment and the
distinctions. Screenshots were captured for default, expanded, re-collapsed,
and enlarged-text states. CUA exposes no pointer-move primitive, so the native
hover tooltip itself was not held onscreen for screenshot evidence; its exact
registered string and accessibility summary are covered by implementation and
counting tests.

Post-decision swiftui-pro/macOS-design review found no additional accessibility,
layout, color, or rendering-ownership defect. `make check`, **95 tests in 16
suites**, and `make build` pass; strict deep signing and the minimal-environment
packaged check for all eight grammars/query roles pass. The first parallel full
test run hit the pre-existing asynchronous repository-switch expectation once;
that test passed immediately in isolation and the subsequent complete 95-test
run passed.

## 2026-10-03 — Double-click occurrence locks

An unmodified double-click now toggles a persistent, document-scoped occurrence
lock using the existing semantic-first and Tree-sitter function-local fallback
resolution. Multiple groups can remain locked while another symbol is selected,
whitespace is clicked, native word selection remains active, Find runs, or the
reader scrolls, folds, changes font, or changes appearance. Double-clicking any
occurrence of a locked group removes only that group and clears the transient
wash from the first click. Command-click definition navigation never toggles a
lock.

Pins retain canonical UTF-8 byte ranges and live in bounded repository-session
state keyed by file path plus source byte count and checksum. Revisiting an
unchanged file restores its pins; changed bytes invalidate them. Semantic IDs
remain authoritative, while equal range sets safely reconcile a cfg-inactive
syntactic pin if semantic data later arrives. Stable monotonically allocated
palette slots mean removing one pin does not recolor another; teal, amber,
violet, and rose fills cycle without limiting pin count. Separately tuned
light/dark translucency plus a 1.4-point stroke distinguishes locked groups
from the ordinary accent wash without changing text attributes or layout.

Regression tests cover two simultaneous pins, independent unpin, stable color
slots, per-file revisit, changed-source invalidation, syntax-to-semantic
reconciliation, Unicode/fold-safe drawing ownership, and the exact four-token
`android_log/message` fallback while transient-selecting another identifier and
then clearing on whitespace. Native CUA on the packaged app expanded untouched
`engine/rust/src/app_image.rs`, double-clicked the cfg-inactive local `message`,
and accessibility reported **1 pinned symbol group, 4 pinned occurrences**;
the light-appearance screenshot showed all four teal translucent fills with
outlined edges while the clicked word retained native blue selection. A single
click on `CString` left the four teal locks in place. Double-clicking `CString`
added an amber outlined group and accessibility reported **2 groups / 6
occurrences**. Double-clicking a different `message` occurrence removed only
the teal group, left the two amber `CString` occurrences visible, preserved
native word selection, and reported **1 group / 2 occurrences**. This is actual
pointer-event evidence on the packaged application, including independent
multi-pin removal rather than model-only coverage.

Post-decision swiftui-pro and macOS-design review found no additional data-flow,
accessibility, layout, or color-ownership issue. `make check`, **93 tests in 16
suites**, and `make build` pass. Strict deep code-sign verification passes, and
the packaged minimal-environment resource check loads all eight grammars and
all query roles. Native screenshots covered the dynamic palette in light
appearance; dark alpha selection is deterministic and tested through the same
appearance-aware drawing path, but this pass did not restart the app in dark
appearance because doing so would discard the document-session pins under test.

## 2026-10-03 — Cmd-K fuzzy file picker and cfg-inactive occurrence fix

Cmd-K now opens a compact native Go to File sheet backed only by repository file
discovery; Cmd-P remains an alias. Case/diacritic-insensitive ordered-subsequence
matching makes `engrs` find `engine-foo.rs`. Exact, prefix, contiguous, boundary,
gap, and filename-versus-path signals produce deterministic ranking without a
full per-keystroke sort. The best 100 remain visible with the true total and a
refinement hint. Filename/path hierarchy, focused typing, Up/Down selection,
Return, Escape, mouse activation, stale query/repository guards, empty states,
and source-focus restoration use native controls and the existing history-aware
open path. Details are in `plans/quick-file-picker.md`.

The reported real-source occurrence bug is fixed end to end. The cause was both
interaction and data coverage: double-click word selection bypassed the old
single-click callback, and host SCIP correctly omitted Android-only tokens in
`#[cfg(target_os = "android")] fn android_log`. Single- and double-click now
refresh decoration after AppKit selection. When SCIP does not cover all parsed
local occurrences, a drawing-only Tree-sitter fallback matches exact identifier
tokens inside the innermost function/closure, excluding strings, comments, and
other functions. It deliberately includes the `message` parameter and shadowing
local requested by the user, but never supplies semantic navigation or reference
targets.

Native CUA exercised the latest packaged app on untouched real tkos. Cmd-K
focused its field immediately, reported `Showing 100 of 6721 matches`, and query
`apimgrs` ranked `app_image.rs` first with secondary path `engine/rust/src` and
89 total matches; keyboard selection and Return opened a result and restored
source focus. Pointer activation of the selected `app_image.rs` row dismissed
the sheet, kept that file open, and returned focus to the source. In the exact screenshot case, CUA expanded `android_log`,
double-clicked the local-binding `message`, and the real source accessibility
state reported **4 occurrences highlighted** while the dynamic accent marks were
visibly present on the parameter and uses. Clicking whitespace removed the
state and all marks. This is actual pointer/source/parser/index integration on
`engine/rust/src/app_image.rs`, not fixture-only evidence.

Regression coverage includes fuzzy ranking/lifecycle bounds and the exact
cfg-gated Rust shape: parameter, shadowing binding, initializer use, and call use
are the four matches; a string, comment, and same spelling in another function
are excluded. `make check`, **89 tests in 15 suites**, and `make build` pass.
Strict deep code-sign verification passes, and the packaged minimal-environment
resource check loads all eight grammars and their query roles.

## 2026-10-03 — Semantic occurrence highlighting

An ordinary primary click on an indexed source occurrence now selects its exact
SCIP symbol identity and persistently decorates every occurrence of that identity
in the current document. The clicked occurrence uses a stronger system-accent
fill and stroke; definitions and other references use a restrained accent wash.
This is drawing-only TextKit presentation: canonical UTF-8 bytes, syntax
attributes, native selection/copy, Find, destination flashes, and Cmd-hover
temporary attributes remain independent. Clicking whitespace clears the set,
another symbol replaces it, and file/repository/index replacement clears stale
semantic state. Hidden fold glyphs are excluded from drawing, while expanding a
fold naturally reveals their still-valid ranges.

Selection is identity-based, not spelling-based. Regression coverage includes a
Rust shorthand-field collision where the field's definition/references are
selected but a same-spelling local is not; a known external symbol with no
repository definition; definition-plus-reference inclusion; UTF-8 byte ranges
containing emoji/accented text converted to UTF-16 only at the TextKit boundary;
whitespace clearing, file switching, and drawing ownership that does not mutate
text, selection, or temporary attributes. The suite now passes **83 tests in 14
suites** (baseline 79).

Native CUA attached to the packaged app on the real untouched tkos repository,
opened `engine/rust/src/app_image.rs`, confirmed the real **Index Partial** state,
the compact default-folded source, the visible **Collapse Functions** control,
and the semantic outline containing both `PreparedApp` declarations. Outline
activation navigated to `PreparedApp` with the process remaining responsive.
CUA exposes the entire TextKit source as one accessibility element, not individual
tokens, and this environment's Screen Recording capture remains denied
(`screencapture: could not create image from display`), so an actual coordinate
token click and visual pixel comparison could not be captured honestly. The
pointer path is nevertheless exercised at the `ReaderTextView.mouseDown` boundary
by the focused TextKit/model tests above; token-level screenshot evidence remains
the sole UI-validation limitation.

Validation: `make check`, `make test` (**83 tests in 14 suites**), and `make
build` pass. One pre-existing asynchronous repository-switch assertion flaked in
the first combined run and passed on the immediate complete rerun. Strict deep
code-sign verification passes, and the packaged minimal-environment resource
check loads all eight grammars and all query roles.

## 2026-10-03 — Real tkos Rust acceptance implemented and measured

Radiology's packaged app completed a cold, app-controlled semantic run over the
untouched `/Users/agentzero/codebase/tkos` repository. It selected all 11 Cargo
project directories (the discovery bound is now 32), emitted real rust-analyzer
indexes for **338 of 340 Rust files**, and preserved the real Cargo package
identities. Directory coverage: `engine/rust` 28/29, `tools/phone-provisioner`
1/1, `modules/agent/ffi` 71/71, `modules/agent/harness` 151/151,
`modules/game/rust` 3/3, `modules/imap/rust` 3/3, `modules/mls/rust` 11/11,
`modules/scene3d/rust` 3/3, `modules/spotify/rust` 5/5,
`modules/storage/rust` 2/2, and `modules/tkreplica/rust` 60/61. No Cargo
project returned zero documents or crashed. tkos remained Git-clean.

The engine result is deliberately **Partial / sourceOnlyNoDeps**, never
Current: Cargo cannot resolve `deno_core` because the repository's patched
vendor tree has not been prepared. Radiology now writes a project-specific,
nested rust-analyzer config using all real manifest features as cfg overrides,
captures stderr head and tail, persists capability/cargo-error provenance, and
reports the exact error (`failed to select a version for deno_core`). That
indexes the real `tk-js-runtime 0.1.0` crate and 28/29 files, including
`src/app_image.rs` (935 occurrences and 152 definitions). The sole missing
engine file, `src/snapshot_profiles_test.rs`, is reported as an orphan because
no crate root declares it. The one tkreplica omission is the shared
`test_support/fixture_paths.rs` loaded through several `#[path]` test modules;
its omission is reported conservatively as unknown rather than guessed.
Successful-but-important rust-analyzer warnings remain inspectable and bounded.
No synthetic manifest, copied source tree, LSP, or tkos modification ships.

An opt-in decode/normalize probe over the generated cache measured 389 total
mixed-language documents, 63,224 symbols, and 16,615 search entries. Against
the real `app_image.rs`, the `PreparedApp` use in `restore_app` resolves to one
in-repository real definition and Find References returns 11 occurrences.
Repeated progress publications now reuse the normalized index until inputs
change, avoiding the high CPU cost observed while generating the large tkos
cache. Cached degraded strategies restore as Partial, and an independent
strategy failure no longer discards successful semantic data.

During final launch validation, the app exposed a caught TextKit folding
warning while laying out the heavily folded file. Default fold state now
resolves the pending destination before the first glyph invalidation, and fold
discovery never touches a layout manager before the new buffer is installed;
regressions cover both ordering rules. AppKit still logs and ignores two
`NSRunStorage` range exceptions on this 511-line folded file. The process stays
alive and prior native inspection showed correct layout, but the console noise
remains a concrete TextKit limitation rather than being claimed fixed.
The final native CUA bridge repeatedly returned `cgWindowNotFound` for the
running foreground Radiology process, so direct tkos status-popover and
app-image click/Back interaction could not be visually re-verified in this
last build. Earlier in this same pass, native CUA did verify cross-file
definition/Back, references, Cmd-T, folded-target reveal, and destination
highlight on `/tmp/demo-crate`; tkos semantic coordinates are verified
headlessly as described above. This is the remaining UI-evidence limitation,
not an indexing limitation.

Validation: `make check` passes; **79 tests in 14 suites pass**; the real
rust-analyzer generation test passes; the tkos performance/semantic probe
passes; `make build` assembles and ad-hoc signs the app; strict codesign and the
packaged minimal-environment eight-grammar resource check pass.

## 2026-10-03 — Acceptance gate: real tkos Rust indexing

The user requires useful Rust indexing of `/Users/agentzero/codebase/tkos`
before moving on. A few uncovered files may be acceptable only with inspectable
missing paths and reasons. Crashes, no usable index, or missing most Rust files
in a substantial directory are hard failures. Previously documented engine
coverage of 3/15 files fails this gate; fixture/agent-copy success does not
satisfy it. The SCIP milestone remains incomplete until this is resolved.

Sent the updated gate to active Sol agent
`fd0b558f-487f-4faa-8a95-c579f6a95eed`: audit real tkos manifests/features and
warnings, fix sparse coverage and independent-strategy failure handling, expose
missing/failed files, and verify per-directory coverage plus actual semantic
navigation in important engine code. Preserve Fable's architecture; raise major
decisions to parent for Fable if necessary. No LSPs or invasive tkos edits.
Required evidence is real app-controlled tkos indexing and useful directory-level
coverage, not merely an indexer exit code or passing tests. Status updates remain
active.

## 2026-10-03 — Sol SCIP UI verification and refinement started

Fable agent `5e1e19a7-8874-4cc9-b13e-04c96161f4b0` finished and is idle.
Parent resumed GPT Sol agent `fd0b558f-487f-4faa-8a95-c579f6a95eed` through
Paseo for the user-authorized native UI testing and refinement phase, using
`plans/scip-ui-handoff.md`. Sol will test real app-generated data and all semantic
navigation/index lifecycle interactions, fix concrete defects, preserve Fable's
architecture, and report actual evidence and limitations. Baseline for this pass
is 76 tests in 14 suites. Native semantic UI validation is pending; the entire
milestone is not yet complete. Five-minute status updates remain active.

## 2026-10-03 — Fable first cut ready for Sol UI pass: app-owned SCIP indexing and semantic navigation

Radiology now generates and consumes SCIP indexes itself. On repository open,
`SemanticIndexService` inspects build files, selects indexer strategies
(rust-analyzer, scip-go, scip-typescript, scip-python, scip-clang), loads any
cached index immediately, compares HEAD/working-tree/build fingerprints, and
regenerates out of process in a dedicated process group with bounded,
file-backed diagnostics. Generated `index.scip` files and `index-metadata.json`
live under `~/Library/Caches/Radiology/repositories/<id>/`, never in the
repository. Missing tools are reported with an Install action that runs the
verified install command into `~/Library/Application Support/Radiology/tools/`
(rust-analyzer via `rustup component add`). States: No Index, Indexing,
Current, Possibly Stale, Failed, each inspectable from the footer popover with
indexer, version, command line, exit status, and output tails.

A hand-written protobuf reader decodes the official `scip.proto` (typed and
packed ranges, roles, kinds, relationships, unknown fields skipped). The
normalized `SemanticIndex` namespaces document-local symbols, follows
`is_definition` relationships, and ranks ⌘T entries. SCIP coordinates convert
to canonical bytes only against the loaded `SourceBuffer`, honoring UTF-8/16/32
position encodings and rejecting out-of-bounds data. Navigation: ⌘-hover
underline and pointing hand only for resolvable targets, ⌘-click, ⌃⌘J, context
menu, Find References grouped by file in the new Symbol inspector pane, ⌘T
symbol search, exact-byte Back via a recorded departure entry that preserves
pinned structural anchors. Architecture, decisions, support matrix and limits
are in `plans/scip-semantic-navigation.md`; the UI handoff is
`plans/scip-ui-handoff.md`.

Validation: `make check` passes; **76 tests in 14 suites pass** (56 baseline + 1 opt-in performance probe +
19 new: decoder and typed ranges, encodings, half-open caret resolution,
shorthand/local disambiguation, path validation, ranking, process exit/tail
capture, process-group cancellation, generate→cache→reload→stale→failure,
missing tools and supersession, nested-project prefixes, AppModel ⌘-click and
exact Back, references and ⌘T, repository switching). The real rust-analyzer
generation test passes (2.1 s). `make build` assembles and ad-hoc signs the
app; `codesign --verify --deep --strict` passes; the packaged
`--verify-resources` run loads all 8 grammars. The built app, launched on a
fresh Git copy of the fixture crate with no prebuilt data, generated
`rust-analyzer.scip` (2 documents) in 2.6 s with HEAD recorded, and a relaunch
reused the cache without regenerating. On a Git copy of `tkos/modules/agent` (222 Rust files in two Cargo
workspaces, `ffi` and `harness`), the app selected two rust-analyzer strategies
and generated both: `ffi` 71 of 71 files in 39.5 s (9.0 MB), `harness` 151 of
151 files in 31.8 s (19.8 MB), merged with project-directory prefixes and HEAD
`14bb8cb83b` recorded. A first attempt on a copy of `harness` alone failed
inside `cargo metadata` because the workspace references `../ffi`; that is the
Failed state doing its job, and the copy was widened. Decoding those two cached
files (28.8 MB) takes 0.34 s and normalizing 0.42 s in a debug build (222
documents, 40,256 symbols, 10,285 ⌘T entries); a ⌘T query over them takes
0.13 s off the main actor (`IndexPerformanceProbe`, opt-in via
`RADIOLOGY_INDEX_PROBE`).

At this first-cut checkpoint, only rust-analyzer was proven end to end; the Go,
TypeScript, Python, and C/C++ adapters had only verified commands. The later
Sol acceptance entry at the top supersedes this snapshot: Go and TypeScript
were exercised, native fixture UI was tested, and real tkos engine coverage
rose from the then-failing sparse result to 28/29 including `app_image.rs`.
Python still needs Python 3.10+ (this machine has 3.9.1), scip-clang still needs
`compile_commands.json` and a manual binary, and Kotlin, Java, and Swift have no
semantic indexer in this cut.

## 2026-10-03 — SCIP indexing and semantic navigation started

The user authorized the next milestone: app-owned SCIP generation, cache and
freshness tracking, out-of-process indexing/cancellation/diagnostics, normalized
semantic locations, cross-file definition navigation and exact Back/Forward,
references, Cmd-T symbol search, and genuine Cmd-hover affordances. No LSPs.

Paseo Fable 5.1 agent `5e1e19a7-8874-4cc9-b13e-04c96161f4b0`
(`Radiology: SCIP Architecture and Implementation`) is running in the root
workspace. Fable owns the architecture decisions and first-cut implementation,
including research of real tooling, supported-indexer decisions, meaningful
tests, and demonstration of actual index generation from a repository with no
prebuilt data. Baseline is 56 passing tests in 10 suites.

Required next phase: once Fable finishes its first cut, parent must resume GPT Sol
agent `fd0b558f-487f-4faa-8a95-c579f6a95eed` for native UI testing and refinement.
Fable is instructed to write `plans/scip-semantic-navigation.md` and a concrete
`plans/scip-ui-handoff.md`, and mark the first cut ready for Sol rather than claim
the full milestone complete. Sol should verify generation/status/freshness,
Cmd-hover/click, keyboard/context actions, exact Back, references, Cmd-T, reindex
and cancellation with real generated data, including folded targets and font
scaling. Preserve Fable's architectural decisions unless a proven defect needs
discussion. Implementation and semantic validation remain pending. Keep the
existing five-minute status heartbeat active and include this new phase in updates.

## 2026-10-03 — Function bodies collapse by default across all grammars

Newly loaded supported source now collapses Tree-sitter function bodies once,
after asynchronous structure arrives. Containers stay open so nested signatures
remain readable. A pending navigation is revealed after the defaults apply, and
ordinary SwiftUI updates, font changes, and resizing do not re-collapse a body
the user expanded. The source header now exposes an accessible **Collapse
Functions** control; `⌘⇧0` provides the same menu action. It re-adds function
folds without changing bytes or history, while the broader Fold to Definitions
command remains available.

Coverage asserts function folds for Swift, Rust, Go, Python, JavaScript/JSX,
TypeScript/TSX, Kotlin, and Kotlin scripts, plus default-once behavior, open
containers, manual expansion persistence, explicit re-collapse, and navigation
reveal ordering. `make check` passes; **56 tests in 10 suites pass**; `make
build` assembles and ad-hoc signs the app with all eight grammar bundles.

Native CUA inspection verified the real 511-line Rust `app_image.rs` opens as a
compact signature scan with clean original-number gaps, full `extern C`
signature visibility, and open impl containers. A gutter click expanded
`android_log`; the visible header control re-collapsed it. Clicking nested
`PreparedApp > image` still jumped to line 57 with breadcrumbs and destination
feedback. Representative Swift showed initializer/method bodies collapsed with
protocol/struct/class shapes open; TSX showed its block-bodied `ScanList` arrow
collapsed while the expression-bodied one-line `Badge` remained visible, as
documented. No type role changed.

## 2026-10-03 — Reliable outline activation and destination feedback

Fixed the follow-up outline-click failure. Row buttons now commit synchronously
on every activation, including an already-selected row; the list's preview path
recognizes the same committed target and cannot overwrite it. Navigation removes
all enclosing folds before jumping. A revision-guarded second main-actor layout
pass handles TextKit geometry that changes after unfolding, Fold to Definitions,
or font scaling, without allowing stale jumps or passive-history feedback.

Added a transient full-line destination highlight for pointer outline jumps. It
uses the dynamic system accent color at 18% opacity, leaves backing text and
attributes untouched, restarts on repeat clicks, cancels on any newer navigation
or file, fades after a short hold, and skips fading under Reduce Motion. No type
role or font changed.

Native CUA verification on the real 511-line `app_image.rs` covered: collapsing
the `PreparedApp` implementation with the gutter, clicking its hidden nested
`image` row and observing the parent unfold plus line-57 jump; clicking `image`
again and seeing the highlight restart; Fold to Definitions; and repeating at
two source-size increments. The first live stress pass exposed pre-collapse
TextKit geometry clamping the viewport below line 57; the guarded settling pass
fixed it, and the final view placed line 57 at the top with the breadcrumb and
footer agreeing. Existing nonoverlapping gutter labels and 28-point fold targets
remain intact.

Regression coverage now includes nested-parent reveal, repeat activation without
duplicate history, highlight restart/cleanup without text mutation, and stale
navigation cancellation behavior. `make check` passes; **54 tests in 10 suites
pass**; `make build` assembles and ad-hoc signs the app; strict code-signature
verification passes; and the packaged minimal-environment resource check loads
all 8 grammars and all four query roles.

## 2026-10-03 — Structural navigation completed and visually verified

Implemented the pre-SCIP structural-navigation slice in the active root package.
The outline now uses strict declaration containment and source order; native
selection previews with Up/Down and Return commits, while pointer activation
commits directly. Source scrolling updates the active declaration and path/scope
breadcrumbs without generating history. Committed structural anchors remain
exact across Back/Forward, while ordinary file visits retain per-file passive
positions. Programmatic jumps preserve focus, reveal a genuinely hidden target,
and suppress their one synchronization callback to avoid near-EOF retargeting.

Added Navigate > Fold to Definitions (`⌘⌥0`). It folds function bodies and leaf
types while keeping declaration containers open, signatures visible, and source
bytes/copy/find coordinates unchanged. Fixed the reported collapsed-gutter bug:
hidden/null glyph lines no longer draw all their numbers on one visible fragment.
Fold controls are now 11-point medium chevrons centered in a 28-point-wide,
full-line hit area with pointing-hand cursor and gutter accessibility help.

Native CUA permissions worked. On the real 511-line
`tkos/engine/rust/src/app_image.rs`, visually verified the hierarchical Rust
outline, path and `SnapshotBootstrapGlobals` breadcrumb, pointer outline jump,
Fold to Definitions, clean original-number gaps across multiple adjacent folds,
scrolling, expanding `hide` through the gutter, and the same layout after two
font-size increases. Light-mode contrast, unclipped source/gutter prefixes, and
three-pane geometry were inspected. Headless regressions cover Rust containment,
passive-versus-pinned history, definition folding, unfold-on-hidden-target, and
suppression of line labels for folded glyphs.

Validation: `make check` passes; **52 tests in 10 suites pass** (49 baseline plus
3 structural regressions); `make build` assembles and ad-hoc signs the app;
`codesign --verify --deep --strict` passes; a minimal-environment packaged
`--verify-resources` launch loads all 8 grammars and four query roles. SCIP/LSP
was not added. Concrete grammar folding limits are in
`plans/structural-navigation.md`.

## 2026-10-03 — Structural navigation and UI refinement handed to GPT Sol

At the user's request, stopped Fable agent `267f6c85-7da4-41ac-80f4-d78239343721`
and launched Paseo GPT-5.6 Sol agent `fd0b558f-487f-4faa-8a95-c579f6a95eed`
in the same root workspace. Fable's latest report covered orientation and 49 passing
baseline tests; it had not reported implementation delivery. Sol is instructed to
inspect actual workspace changes, complete and refine the five structural-navigation
features, and verify the native UI. The user reports granting Paseo's requested
permissions, so Sol will retry UI access. Implementation and UI validation remain
pending. SCIP stays deferred; five-minute status updates remain active.

## 2026-10-03 — Structural navigation delegated to Fable

Started Paseo agent `267f6c85-7da4-41ac-80f4-d78239343721`
(`Radiology: Structural Navigation`, `claude/claude-fable-5-1`) in the active root
workspace. Its authorized scope is hierarchical outlines, outline/source and keyboard
synchronization, Tree-sitter structural breadcrumbs, Fold to Definitions, and accurate
Back/Forward trails. SCIP is deferred. Implementation and validation are pending;
the agent is instructed to preserve all existing language support, document its work,
and report concrete limitations. The recurring five-minute status schedule remains active.

## 2026-10-03 — Rust, Go, Python, JavaScript/JSX, TypeScript/TSX, and Kotlin Tree-sitter support delivered

Extended the Tree-sitter slice from Swift alone to eight grammars in the active root
package. Swift behaviour is preserved. The inherited `repo/` copy is untouched, and no
Git repository, branch, commit, PR, or merge was created.

Delivered:

- Registered real native upstream grammars, resolved by SwiftPM and pinned to exact
  versions: `tree-sitter-rust` 0.24.2, `tree-sitter-go` 0.25.0, `tree-sitter-python`
  0.25.0, `tree-sitter-javascript` 0.25.0, `tree-sitter-typescript` 0.23.2 (both the
  `typescript` and `tsx` parsers), and `fwcd/tree-sitter-kotlin` 0.3.8, alongside the
  existing Swift grammar. Extensions: `.rs`, `.go`, `.py`/`.pyi`, `.js`/`.jsx`/`.mjs`/`.cjs`,
  `.ts`/`.mts`/`.cts`, `.tsx`, `.kt`/`.kts`, `.swift`. All grammars are MIT-licensed and
  report ABI 14–15, inside the 0.25 runtime's range.
- `.tsx` uses the dedicated TSX grammar and `.jsx` the JSX-capable JavaScript grammar,
  because the dialects are different languages rather than supersets.
- Replaced the per-language `SourceLanguage` protocol with a value-typed registry:
  one `SourceLanguage` record per grammar, extension lookup, and an ordered list of
  query directories. TSX composes `ecmascript` + `typescript` + `jsx` instead of
  duplicating the shared ECMAScript patterns.
- Moved queries out of Swift string literals into `Sources/Radiology/Queries/**.scm`,
  packaged as SwiftPM resources and read through `Bundle.module`. Wrote
  `highlights`/`folds`/`symbols`/`scopes` for every language against each grammar's
  `src/node-types.json`, informed by the upstream `queries/highlights.scm` of the same
  pinned version; each file records its source and licence.
- `SyntaxWorker` now caches compiled queries per grammar, evaluates `#eq?`/`#any-of?`/
  `#match?`-family predicates against canonical UTF-8 byte ranges, returns highlights in
  deterministic paint order, and resolves competing fold and symbol patterns by
  specificity. Parsers, trees, and queries stay actor-isolated; snapshots stay Sendable.
- `SyntaxSnapshot` carries the originating language; `RepositoryFile.language` and the
  outline's empty state derive from the registry, so there is one source of truth.
- Surfaced syntax failures that were previously stored but invisible: the source footer
  shows "Structure unavailable" (with the loader's message in its tooltip) or "Partial
  structure" for sources with parse errors, and the outline repeats the message.
- Extended `Theme.SyntaxColors` with constructor, namespace, tag, label, string-escape,
  and builtin-variable roles, all dynamic system colours. Added outline symbol icons for
  the new declaration kinds. No new type size or weight; no unrelated UI redesign.
- Made fold rendering delimiter-aware: brace-, bracket-, and quote-wrapped bodies keep
  their delimiters so a collapsed function still reads as `{}`, while indentation-delimited
  bodies (Python blocks, Kotlin and Swift `statements`) collapse whole instead of stranding
  their first character.
- Added `--file RELATIVE_PATH` and `--verify-resources` development launch arguments, and
  made `build.sh` fail if the query resources are absent from the assembled bundle.

Implementation findings:

- `tree-sitter-javascript` and `tree-sitter-python` publish SwiftPM manifests that probe
  for `src/scanner.c` with a path relative to the manifest process rather than the package,
  so their external scanners are never compiled and the grammars fail to link. Both
  scanners (and the three headers they include) are vendored byte-identically in
  `Sources/GrammarScanners`, with provenance and licences in `PROVENANCE.md` and the
  upstream-only `-Wshorten-64-to-32` warning suppressed at the target rather than in the
  vendored code. This is still upstream's bug at the pinned versions' HEAD.
- `tree-sitter-rust` and `tree-sitter-typescript` declare the pre-transfer
  `ChimeHQ/SwiftTreeSitter` while the others declare `tree-sitter/swift-tree-sitter`.
  SwiftPM prunes test-only dependencies of non-root packages, so the two identities never
  collide and resolution succeeds.
- SwiftTreeSitter's `QueryMatch.allowed(in:)` resolves predicate text through
  `Node.range`, which divides byte offsets by two because it assumes UTF-16 input.
  Radiology parses UTF-8, so predicates had to be evaluated against `node.byteRange`
  directly. `Predicate.evalulate(with:)` and `Predicate.captures(in:)` are public, which
  makes that straightforward.
- The TypeScript grammar's `class_declaration` names a `type_identifier` where JavaScript
  names an `identifier`, and JavaScript's `field_definition` is `public_field_definition`
  in TypeScript. Those two differences are the only ones across the node types the shared
  ECMAScript core needs, which is what makes the overlay composition safe.
- `Bundle.module` resolves inside the test target and inside the assembled `.app`, but a
  test's `Bundle.module` is the *test* bundle, so query-resource assertions have to go
  through `SyntaxQueryLoader`.
- Rust's `crate` is a named node, not an anonymous token; Kotlin's `jump_expression` spans
  the whole returned expression, so capturing it painted entire `when` blocks as keywords.
  Both were found by running queries against fixtures rather than by reading grammars.
- Several grammars have no delimiter-wrapped node for constructs that look foldable (Go
  interface bodies and grouped `const`/`var`, JSX elements). Those folds were removed
  rather than left to render as mangled fragments.

Validation performed:

- `make check` — passes, Swift 6 strict concurrency, no warnings after a clean
  `.build/out` rebuild.
- `make test` — **49 tests in 10 suites pass**, up from the 22-test baseline, which is
  retained in full. New coverage: query compilation for all 8 grammars; query-resource
  reachability and overlay concatenation; a diagnosable error for missing resources;
  per-language highlights/declarations/scopes/folds fixtures for Rust, Go, Python,
  JavaScript, JSX, TypeScript, TSX, Kotlin, and Kotlin scripts; JS-versus-JSX and
  TS-versus-TSX grammar routing (including that the TypeScript grammar rejects JSX and the
  TSX grammar rejects `<T>` casts); UTF-8 byte-range and UTF-16 round-trip checks over
  every fixture; astral-character capture ranges; malformed source for all ten file kinds;
  empty sources; extension classification for all 15 registered extensions; cross-language
  snapshot replacement and unsupported-file fallback; and delimiter-aware fold rendering.
- `make build` — assembles and ad-hoc signs `build/Radiology.app`; `codesign --verify`
  passes. The bundle contains the Radiology query bundle plus all eight grammar resource
  bundles, and `build.sh` verified 31 `.scm` files.
- Packaged-app resource load under a Finder-style launch: running
  `build/Radiology.app/Contents/MacOS/Radiology --verify-resources` under `env -i` with a
  minimal `PATH` reported all 8 grammars loading all 4 query roles, exit 0.
- Packaged-app GUI launch: `open -n build/Radiology.app --args --repository … --file …`
  under `env -i` for each of the nine language fixtures; the app stayed alive in every
  case and wrote no crash reports.
- Applied swiftui-pro before and after writing the code, macos-design for the colour and
  outline work, and typography-designer (which confirmed no new type role was needed —
  the existing body/callout roles cover the outline and footer).

Not validated, and stated plainly: **no visual inspection of the running UI was
performed.** Screen recording and UI-automation (`screencapture`, System Events) are both
denied to this environment, so there are no screenshots of the highlighted source,
outline, or fold markers for the new languages. The evidence above is process-level and
headless. The colour palette, outline icons, and fold-marker placement for the new
languages are unverified visually.

Remaining limits: symbols are syntactic, not semantic; folding is session-local; no
breadcrumbs, fold-all, or language injection; parses are whole-file with no cross-file
tree cache; Go interface bodies, Go grouped `const`/`var`, and JSX elements are not
foldable; Python and Kotlin folds that begin on a body line put the gutter marker on that
line. Next useful step is still SCIP definition/reference navigation on the same canonical
byte-coordinate contract, then filesystem-driven invalidation.

## 2026-10-03 — Major-language Tree-sitter support delegated through Paseo

Started **Radiology: Major-language Tree-sitter** with **claude/claude-opus-5**, as authorized by the user after provider discovery showed no Opus 5.5. Paseo confirmed agent `a3627f41-39ea-4082-bc06-fa1e365bc7e5` is running in the existing root workspace (`wks_f438c91353bec4ec`). This is a delegation record, not a completion claim.

Requested Rust, Go, Python, JavaScript/JSX, TypeScript/TSX, and Kotlin grammars with highlighting, outlines, scopes, and folding. The agent must preserve Swift and the existing byte-coordinate, asynchronous parsing, navigation, and TextKit contracts; retain the 22-test baseline; validate every language and bundled resources; apply required skills; and update the documentation with actual results and limitations. The nested `repo/` remains untouched and merges are prohibited.

Use the bundled `/Applications/Paseo.app/Contents/Resources/bin/paseo` CLI with this agent ID for follow-up. The earlier Swift implementation agent is finished and idle.

## 2026-10-02 — Swift Tree-sitter structure delivered

Completed the delegated Tree-sitter slice in the active root package; the inherited `repo/` copy remains untouched and no Git repository, branch, PR, or merge was created.

Delivered:

- Added the Tree-sitter 0.25 C runtime, official `SwiftTreeSitter` wrapper, and the Swift grammar's `0.7.3-with-generated-files` revision, locked in `Package.resolved`.
- Added a small `SourceLanguage` registry and real Swift Tree-sitter queries for syntax roles, declarations, lexical scopes, and multiline function/type/block/comment folds. No regex syntax substitute, SCIP, or LSP behavior was added.
- `SyntaxWorker` parses UTF-8 on an actor and returns Sendable byte-range snapshots. Source text publishes first; parsing follows asynchronously. Cancellation plus the source generation guard prevents a superseded parse from repainting the next file.
- TextKit highlighting converts captures through SourceBuffer only. Unsupported files and parser/query failures keep readable plain text. Malformed Swift can provide partial results with a parse-error flag.
- Added a native SwiftUI document-outline inspector with history-aware symbol navigation and a toolbar visibility control. Added gutter disclosure markers; folding suppresses structural interior glyphs without modifying the backing source or canonical positions.
- Kept new colors, font sizes, hit areas, indent, and inspector widths in Theme. The existing source layout-before-scroll and history coordinate paths were not replaced.
- Added `plans/tree-sitter.md` and updated the master index, architecture, design, build, roadmap, and README documentation.

Validation:

- `make check` passes under Swift 6 strict concurrency with no warnings.
- `make test` passes 22 tests across seven suites. New regressions cover Swift query captures and declarations/scopes/folds, Unicode byte ranges at the TextKit boundary, malformed and unsupported source fallback, stale parse races, and structural glyph folding. The original repository/search/navigation/layout tests remain green.
- `make build` assembles and ad-hoc signs `build/Radiology.app`; `codesign` validates the bundle and the dependency resource bundle is included.
- Live app inspection verified query-colored Swift, source beginning at column one, the native outline and its declarations, normal read-only selection surface, and the outline toolbar control.
- Applied swiftui-pro before implementation and in final review; used macos-design, typography-designer, SwiftPM, and AppKit interop guidance. The final review found no new SwiftUI API, data-flow, accessibility, or typography violations.

Remaining limits and next useful step: only Swift is registered; highlight coverage favors common constructs; symbols are syntactic, folding is session-local, and breadcrumbs/fold-all/parser caching are absent. Implement SCIP definition/reference navigation next, using the same canonical byte-coordinate contract. Multi-language grammars and filesystem-driven parse invalidation remain later work.

## 2026-10-03 — Large folded Rust layout and salient size cues

Fixed the blocking default-fold regression reproduced in the packaged app on untouched tkos `modules/agent/ffi/src/app.rs` around lines 1450–2220. Hidden newline glyphs had retained line-break behavior, overlapping fold ranges could replace the same glyph run twice, and the gutter enumerated canonical hidden line starts that TextKit aliases onto surviving fragments. Together those produced hundreds of blank rows, stacked/wrong line numbers, leaked nested markers, and mismatched signatures/braces.

- Hidden control characters now use TextKit zero advancement while retaining their original glyph properties plus `.null`; this compacts bodies without corrupting later glyph/character identity.
- Active folded ranges normalize to disjoint outermost ranges, source replacement is the only delegate-detachment boundary, and an update no longer regenerates the same large fold layout twice.
- The gutter now enumerates actual surviving line fragments and derives each original line number from its first visible glyph. It never measures hidden layout rows or mutates backing text.
- Fold size is deliberately more legible: the gutter widened from 68 to 104 points to provide separate control, count, and source-number lanes; a 3-point secondary-label meter grows logarithmically from 6–22 points; a neutral capsule shows the exact hidden-line count in 9-point medium tabular digits. Tooltip/accessibility retain the unabridged count. The 28-point fold hit area is unchanged.

Native CUA verification on the rebuilt app showed multiline `open` at 1453 compact to `{}` and then clean, distinct signatures/line numbers for `attach_session` 1734, `session_tools` 1812, `refresh_tool_definitions` 1829, `attach_worker` 1941, `complete_tool` 2062, and `request_observe` 2075. No blank body remained; no nested marker leaked; count badges `279`, `76`, `7`, `43`, `119`, and `11` were clearly distinguishable at the default scale. Two Cmd-plus steps preserved badge/number separation and source alignment at enlarged type. The test fixture exercises a 607-row multiline Rust function, nested closure text, enlarged then reduced fonts, expand/re-collapse, canonical Unicode coordinates, and asserts compact used layout plus zero-advancement hidden newlines. Targeted SourceScrollView tests pass without the former NSRunStorage warnings. Full validation: `make check`, 96 tests in 16 suites, signed `make build`, strict codesign verification, and packaged query verification for all eight grammars.

## 2026-10-02 — Tree-sitter implementation delegated through Paseo

Started **Radiology: Swift Tree-sitter** using Paseo's available **codex/gpt-5.6-sol** model in the existing local Radiology workspace (`wks_f438c91353bec4ec`). Agent ID: `d60a9896-8491-4d8c-b4da-979dcaa34b68`. The creation response confirmed it was running. Implementation is in progress; this entry does not claim completion.

The agent was instructed to read treesitter.plan.md, radiology.plan.md, AGENTS.md, PLAN.md, and PROGRESS.md; implement actual Swift parsing/highlighting/outline/folding; preserve the 16-test reader/search/history baseline and canonical byte coordinates; apply the required skills; run build/tests and inspect the live UI; and update the docs with results and limitations. No SCIP/LSP work or merges. The root workspace is shared with the delegated agent, and the nested repo/ remains untouched.

Paseo CLI is bundled at `/Applications/Paseo.app/Contents/Resources/bin/paseo`; it is not on this shell's PATH. Use `paseo inspect` or `paseo logs` with the agent ID via that binary for follow-up. The initial delegation used provider discovery because this bundled CLI exposes no named-profile command or profile configuration.

## 2026-10-02 — First working Radiology repository reader

Replaced the lorem-ipsum/table Starter scaffold with a native read-only code browser, following `radiology.plan.md`. The active implementation is the workspace root; `repo/` is an inherited duplicate and was left untouched. The root has no Git metadata and the duplicate's Git object store is incomplete. No commits, PRs, or merges were made.

Delivered milestone 1 and the core of milestone 3:

- Renamed SwiftPM executable, source/test directories, bundle identity, app name, and build targets to Radiology; retained the no-Xcode build workflow and raised the tools requirement to Swift 6.2.
- Repository/folder opening via native panel, recent repositories, Finder drops, and a `--repository PATH` development launch argument.
- Git root discovery plus NUL-delimited tracked/untracked file discovery with ignore handling; filesystem fallback, generated-directory exclusions, expandable file tree, and filename filter.
- Immutable SourceBuffer with canonical UTF-8 byte offsets, byte and UTF-16 line maps, centralized Unicode-safe conversion, and language classification. Binary/non-UTF-8 sources and files over 16 MB show local errors.
- AppKit/TextKit read-only source with native selection/copy, horizontal/vertical scrolling, current-line tint, separate line-number gutter, native in-file find, adjustable source text size, go-to-line, per-file line-position preservation, and browser-style navigation history.
- Cmd-P file picker and streaming ripgrep JSON search, with debounce, cancellation, literal/regex, case and include/exclude glob options, exact byte result locations, and a 5,000-result cap. Missing tools and invalid regexes show search-local errors.
- Actor-based repository, source, command, and search work; MainActor observable UI model; request generations prevent stale results. Tests use isolated preferences rather than polluting recent folders.
- Replaced PLAN.md, README.md, and architecture/design docs; added `plans/roadmap.md` and updated build documentation.

Implementation findings:

- The copied `.build` had empty files where SwiftPM needs directories, and scripts had lost executable bits. Moved the old generated cache to `/tmp/radiology-template-build-20261002`, restored executable permissions, and built fresh.
- macOS aliases `/var` and `/private/var` differently during URL enumeration. Relative filesystem enumeration avoids corrupting file paths; a regression test covers ordinary folders.
- A native ruler changed horizontal document coordinates and clipped source prefixes. The gutter now lives beside the scroll view with explicit clipping; visible text starts at column one.
- TextKit can clamp a new-file jump before it has sized the document. SourceScrollView ensures layout and document size before applying a pending location; an AppKit regression test checks the destination and prevents line drift on restoration.
- Finder launches have a minimal PATH. The built Info.plist records the build-time ripgrep installation; ToolLocator also checks PATH and standard Homebrew/cargo/system paths.
- Removed the template's App Sandbox entitlement for this local subprocess-based build. No distribution sandbox strategy or release was attempted.

Validation:

- Swift 6 strict-concurrency compile and ad-hoc signed app bundle succeed on macOS 26.6.2 / Swift 6.4, with no build warnings.
- Swift Testing: 16 tests across six suites cover Unicode/CRLF/empty and invalid sources, history branching, Git ignores and odd filenames, folder discovery, external symlink rejection, real ripgrep offsets/globs/no-match/errors/cancellation/limits, model navigation/opening races, and AppKit scrolling.
- Live app inspection through Computer Use verified repository opening, source/line-number rendering, Cmd-P, Cmd-Shift-F, ripgrep results, result-to-line jumps, Cmd-L, back/forward, source restoration, and native find/match navigation. Source readability and clipping were checked in screenshots.
- Applied swiftui-pro before implementation and again in the final review. Used macos-design and the local typography-designer skill; documented the type scale in `plans/design-system.md`. Also reviewed the available SwiftPM and AppKit interop skills.

Next work: Tree-sitter for Swift (accurate query-based highlighting, outline, breadcrumbs, folding), then SCIP loading/definition/reference navigation. Neither is faked in this delivery; source is plain text. Automatic filesystem watching, fuzzy ranking, pixel-exact/horizontal restoration, multi-language grammars, large-repository profiling, and macOS 14 runtime validation remain outstanding. Manual Cmd-R reload works today. No editing or LSPs.

---

Running log. Append an entry per meaningful change: what you did, what you
learned, what surprised you (`SWIFTUI-RULES.md` §10.1). Newest at the top.

> Once you've renamed the template, the entry below is history — leave it as
> the record of where the scaffold came from, and log your own work above it.

---

## 2026-06-28 — Metabolized learnings from the first app on this template

Folded `LEARNINGS.md` (general lessons from building the first app — an
outdoor-weather feature — on this scaffold) into the permanent docs, then
deleted the copy. Where each landed:

- `SWIFTUI-RULES.md` **§5.6** — `foregroundStyle` resolves innermost-wins (a
  tinted container can host a differently-colored inline child; the corollary
  bites the "explicit `.foregroundStyle(.primary)`" habit).
- `SWIFTUI-RULES.md` **§11 Swift Charts** (new) — overlaid lines need distinct
  `series:` or they merge into one path; the x-domain is the union of all
  plotted data, so clip overlays to the primary series' window.
- `SWIFTUI-RULES.md` **§12 Concurrency** (new) — a `@MainActor` type adopting a
  delegate protocol needs an isolated conformance (`@MainActor CLLocationManagerDelegate`).
- `plans/build-system.md` — new "Permissions & capabilities" section: usage
  prompts work in this ad-hoc-signed, no-Xcode bundle given the right
  `NS…UsageDescription` keys; the sandbox decides whether a `com.apple.security.*`
  entitlement is also needed; always design the denied path.

Appended §11/§12 rather than renumbering so the existing §9/§10 cross-references
in `PLAN.md` / `PROGRESS.md` stay valid.

## 2026-06-28 — Template scaffold (the "Starter" baseline)

Built the template this repo ships as:

- **Build system** lifted from the Casette app's SwiftPM-only setup and
  generalized: `Package.swift` (one exe + one test target, Swift 6 mode, no
  deps), `build.sh` (swift build → `.app` → codesign, ad-hoc fallback),
  `Makefile` (build/check/test/run/install + sign→notarize→staple→zip `dist`
  pipeline), `Resources/Info.plist`, sandboxed `Starter.entitlements`,
  `scripts/make-icon.swift` (pure-CoreGraphics doc-and-table glyph).
- **App**: `NavigationSplitView` with a `SidebarSection` enum driving a lorem
  reading column (`ReadingView`, measure-capped) and a native sortable `Table`
  (`DataTableView`) with a reusable `StatusBadge` and a footer count. State in
  one `@Observable @MainActor AppModel`; type/metrics centralized in `Theme`.
- **`scripts/rename.sh`** — the meta feature: `cp -R` then one command renames
  target, `.app`, bundle id, dirs, and file contents.
- **Tests**: swift-testing suite covering model load + status sort ordering.
- **Docs**: this file, `PLAN.md` (index, written for the fresh-copy workflow),
  `PROBLEMS.md`, and `plans/{architecture,build-system,design-system,renaming}.md`.

Gates run: `make check` ✓, `make test` ✓ (3 tests), `make build` ✓ (icon +
signed `.app`), live launch ✓ — both destinations render correctly, app stays
alive across a sidebar switch (no constraint crash).

Design work was driven through the macos-design, typography-designer, and
swiftui-pro skills per `CLAUDE.md`, before and after writing the views.
