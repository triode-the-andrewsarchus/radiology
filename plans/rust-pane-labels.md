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

## Welcome recent paths — 2026-11-02

The subsequent welcome-screen audit found a distinct long-path defect. Swift's
`WelcomeView` bounds Recent Repositories to 500 points and gives each path one
middle-truncated line. Rust used unbounded wrapping links. At 800×500, the actual
app regression measures 25 lines for the first path, pushing later entries outside
the window. Native previous-binary captures at 1×/2× confirm the final entry cannot
be reached on screen.

Welcome links now reuse `text_fit::middle`, `bounded` and `explanation`. Each label
fits the smaller of the available pane and 500 points, retaining both ends. Only
display CR/LF become spaces; callbacks still open the original repository path,
including Unicode, spaces and newlines. Existing body type, link color, hover and
keyboard-capable Link widgets remain in place. Hover uses the existing bounded
path explanation. No font role, state, helper, worker, dependency, public API or
sibling code was added. Requested Swift/macOS/type skills remain unavailable in the
installed catalog; the Swift view and established shared label policy are the
reference for this layout-only change.

The actual-app matrix covers all eight recent paths, 800×500/1100×800/1400×900,
1×/2× and dark/light. Rows stay single-line, within 500 points and inside the
window. Clicking the last row opens its exact canonical directory and reads its
original CRLF source. The identical test fails before the change and passes after.
Native old/current 1×/2× at the minimum size prove all eight new labels visible in
both appearances; the final recent repository opens through the pointer, then
Ctrl+K selects its source and fresh Select All/Copy verifies its unique exact
Unicode/CRLF payload. Captures are inspected; fixtures/binaries stay unchanged and
all app/compositor exits are zero.

Workspace: 639 app tests pass, seven ignored/zero filtered; two binary, 13 font,
21 text and 42 doctests pass. Strict Clippy/check/build/rustdoc with shaping,
formatting/diff checks and copied standalone resource verification pass.
Architecture baseline `before-welcome-recent-paths` at `7c0eb47`: NEUTRAL, +77 NLOC
and one regression function, unchanged files/max CCN/110 clone groups/cycles;
average CCN 3.34→3.35, health approximately -0.01. The production change extends the
existing loop with shared measured labels instead of adding another UI mechanism.

Evidence: `~/.local/state/radiology-qa/welcome-recents-20261102-7c0eb47`.
The broader feature/UI/corpus audit and native IME-engine acceptance remain open.
Full accessibility and actual sibling adoption retain their deferred scope.
