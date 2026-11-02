# Compact inspector summaries and bounded region content

## Behavior — 2026-11-01

The declaration region now starts with the declaration name and disclosure, then
kind/location metadata on one line. This follows Swift's `DeclarationSummaryView`
and `InspectorSymbolSummary`: a pinned identity above references, with signatures
behind disclosure. The redundant Declaration heading and separate kind row are
removed. Existing body/small typography, glyphs, links, signature selection/copy
and exact byte navigation are reused.

Summary content scrolls within the height left after reserving reference heading,
coverage and a file/row. Its scroll identity includes exact source/declaration and
explicit reference ownership; changing the source declaration resets disclosure
and reveals the top again. Framework scroll areas do not add drag surfaces ahead
of the original keyboard controls. Scroll wheels and scrollbars remain available;
selectable signature text retains its own drag interaction.

Reference coverage has a separate, bounded scrolling area taking at most a third
of the remaining reference space. Long unmapped/unreadable/omitted notices and Load
More cannot consume the entire reference viewport. Existing result virtualization,
selection, disclosure, keyboard routing, actions and source validation are reused.
The declaration panel minimum increases from 100 to 160 logical points so its
compact summary, reference heading/coverage and first file/reference can coexist.
The outline and bookmark regions retain independent sizing and scrolling. The
standalone app's existing 800×500 minimum window is unchanged.

No new dependency, worker, public reader API or persistent application state is
introduced. Scroll positions use egui's existing per-control state. The requested
SwiftUI/macOS/typography skills remain absent from installed skill/plugin roots;
no new type scale or widget system was invented.

## Evidence

The original regression paints the References heading from y=94 to y=108 in a
100-point clipped pane. The existing lifecycle test now checks compact/expanded,
returned and explicitly pinned summaries at three theme/scale/height combinations.
It verifies name/header/Follow Caret clipping, keyboard disclosure, signature
scrolling, and retirement when leaving the declaration. A new three-scale case
checks the name, counted heading and first reference while coverage is incomplete,
then scrolls to and activates Load More through its existing action.

The existing reference hit/results builders are shared through UI test support;
one shared clipping assertion replaces duplicate geometry checks. A breadcrumb
focus test now locates the name above and left of the actual source viewport's
right/top edges: the compact inspector legitimately paints the same name higher
than the old hard-coded y=65 cutoff. The source focus surface begins after the
gutter, so its left edge is not a breadcrumb bound. This fixes test targeting,
not source navigation behavior.

Initial trials exposed egui's default minimum scroll extent and leading drag-area
keyboard stop; bounded summary/coverage areas now allow a zero minimum extent and
avoid that extra stop. Early fixture construction failed strict Clippy, and a
new duplicate assertion was removed through the shared helper. These intermediate
trials are excluded as accepted product evidence.

The final workspace passes 570 app tests (seven ignored), two binary tests, four
font tests, 13 text tests and 37 doctests. Strict Clippy, all-target check, binary
build, rustdoc with warnings denied, formatting and diff checks pass.

Four final native flows pass against one final binary: loaded and indexed readers
at 1× and 2×, with the divider dragged to its minimum. Loaded cases disclose and
scroll signatures, copy the exact original signature through its context menu,
visit another declaration, return with retired disclosure and copy the whole
source. Indexed cases pin explicit references, commit the first definition,
verify its exact name and whole source by clipboard, then use Follow Caret.
Fixture/binary hashes remain unchanged and all children exit zero. Dark/light
captures were inspected. Earlier harness waits incorrectly matched the explanatory
phrase “structure browsing”; later OCR trials confused the References heading
with body text or merged its count. Those trials are excluded. Final locators
select the upper reference heading and recognize its count-bearing text.

Architecture baseline: `before-inspector-minimum-height` at `d74a4ce`. Product and
tests add 183 NLOC and two functions, with no new files, hotspots, maximum CCN or
cycles. Health changes about -0.02. The complete normalized multiset comparison
confirms all 91 clone groups identical, with zero added or removed. The automated
regression warning flags only the existing 12-line modal/reference test clone
because its reference-list file changed. The new duplicate geometry assertion was
removed before acceptance.

Archive: `~/.local/state/radiology-qa/inspector-region-sizing-20261101-d74a4ce`.
`accepted.json` selects `native-v2-1.0-loaded`, `native-v2-2.0-loaded`,
`native-v4-1.0-indexed` and `native-v3-2.0-indexed`. Controllers, exact clipboard
bytes, captures/action logs, hashes, workspace/gate logs, negative/debug probes,
architecture reports/full clone audit and source diff are retained.

## Scope still open

This closes the observed small-region clipping and coverage starvation cases.
Broader inspector corpora, exact ownership across every menu/overlay, type/spacing
alignment and final whole-app Swift acceptance remain part of the parity goal.
Full accessibility and actual little-screen-clip adoption remain deferred.
