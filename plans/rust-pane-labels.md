# Bounded file labels and compact source breadcrumbs

The native-proven Files/sidebar and source-header filename overflow is repaired.
Long/deep file rows stay within their pane, and the header now compacts its source
trail using the Swift presentation policy. Automatic focus return and the broader
UI acceptance queue remain open.

## Reference and behavior

The references are [source-breadcrumbs.md](source-breadcrumbs.md),
[design-system.md](design-system.md) and the production Swift
SourceBreadcrumbPath/Variant/Trail/Overflow views. The requested SwiftUI/macOS/type
skills are unavailable in this environment; those checked-in specifications and
views supply the design contract.

A shared private text_fit helper measures the actual UI font and middle-elides
labels on UTF-8 boundaries. It preserves the current type scale and both ends of
names, including extensions; it does not shrink the font to fit. Files rows keep
their original model paths for opening, context actions and full-path help.
Indentation is capped before it can force deep rows beyond the pane. Selected
backgrounds and changed-file markers retain their existing behavior.

The source header gives tools their trailing space before laying out its flexible
breadcrumb. The measured variants first retain the entire enclosing-declaration
chain while dropping leading directories, then drop outer declarations. Nearby
path context fills whatever space remains. File plus final declaration survive
when they fit; extreme widths prioritize the final declaration, or the filename
outside a declaration, before middle elision. There is no extra header row or
horizontal breadcrumb scrolling.

An ellipsis exposes the complete relative path and every enclosing declaration.
It uses the existing shared keyboard-owned menu component: the path action copies
its original bytes, and declaration actions use existing canonical-byte jumps.
The menu scope includes source content and declaration identity. Visible
breadcrumbs keep direct declaration jumps. Full-path help is an intentional
floating tooltip; it can cross a pane boundary without expanding that pane.

Rust retains its existing Before/After actions in the fixed trailing header area.
The visible Revision/status captions give way to action/status help; change-block
and Collapse Functions actions remain protected. Collapse is unavailable without
a callable fold. The source footer's Go to Line hint now uses Ctrl+L. The final
Swift/host chrome audit, including toolbar arrangement, is not closed by this
breadcrumb increment.

## Validation

The exact final file-row regression fails against the old implementation: a
196-point effective minimum panel expands to 535.8439 points. It passes with the
repair at configured widths 180, 260 and 420. The minimum comparison uses an
empty-panel control because egui includes frame margins in its effective minimum;
it does not demand a smaller panel than the framework itself creates.

Four added UI tests cover long/deep pane bounds; elided file open/full-path copy;
source priority labels and fixed review/change controls at 360/440/560/740/1100
points in both themes at 1×/2×; and direct/overflow jumps to every enclosing
declaration plus exact Unicode path copy. Existing behavior tests remain intact.
Workspace passes 467 app tests (seven ignored), two binary checker tests, four
font tests, eleven text tests and 34 doctests. Strict Clippy/build/fmt/diff pass.

Architecture comparison reports REGRESSION solely because two unchanged test
clones touch the modified source_view.rs file. The saved baseline contains the
same 8- and 7-line fragments. A full fingerprint comparison finds 92 groups before
and after, zero new and zero removed. There is no new hotspot/cycle/max-CCN
increase; SourceView::show drops from CCN 21 to 8 through the cohesive header
component. An actual new test-setup clone was consolidated. Unrelated test
collection refactors were set aside instead of chasing old modified-file flags.
The raw comparison and clone audit are retained with the evidence.

Native private Niri runs at 1× and 2× use a 170-byte Unicode filename and real Rust
module/impl/function declarations. Captures show the sidebar/source/inspector
boundaries intact in dark and light mode. Both sidebar and header menus copy the
exact full filename; screenshot clipboard clearing and expected SHA checks prove
fresh payloads. Ellipsis keyboard navigation reaches target at line 4 and the
outer declaration at line 1, confirmed in the caret/footer/inspector captures.
Fixtures remain unchanged and app/compositors exit zero. The older before image
uses a different fixture; it is qualitative overflow evidence, not a pixel-diff
comparison. The controlled before/after regression uses the identical final test.

Durable source/binary/fixture hashes, captures, driver, tests and SHA256SUMS:
`~/.local/state/radiology-qa/pane-labels-20261031-c9fd9e1`.

## Remaining work

Automatic return of keyboard focus from modal/toolbar/breadcrumb commands,
traversal and rapid owner changes remain required. Changes/Search/inspector
long-label cases, broader wrapping/scroll/scaling and large-file performance,
long-line Back, grammar/index/PR/cache corpus and the final requirement-by-
requirement Swift audit also remain open. This change verifies Files and source
breadcrumbs rather than every text-bearing surface. Full accessibility and actual
little-screen-clip adoption remain deferred; the app remains standalone and adds
no dependency.
