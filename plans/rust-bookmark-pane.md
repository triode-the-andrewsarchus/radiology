# Rust bookmark pane contract and acceptance

The Swift references are `BookmarksPane.swift`, `BookmarkRow.swift`,
`BookmarkContextMenu.swift`, and `Bookmarks/BookmarkModel.swift`. The Rust pane
remains independently hosted; it does not import little-screen-clip at runtime.

## Implemented behavior

- The header toggles the current source line through the same reader command as
  Ctrl+D. The button reflects anchor-resolved current-location membership and is
  disabled without ready source. Repository/PR preparation disables management.
- Rows show one bounded primary label, secondary path/line and optional Before or
  After revision; the first ten show Ctrl+0 through Ctrl+9 on the right. The
  tooltip includes the original column. Anchor/context text remains in the
  persisted navigation model rather than expanding each visible row.
- One list focus owns unmodified Up/Down, Home/End, Return, Delete/Backspace and
  Escape before egui's automatic focus movement. Selection does not navigate;
  pointer activation and Return commit through the existing asynchronous anchor
  resolver. Escape clears selection. Tab, modified keys, releases and global
  definition commands remain available to their owners.
- Selection and queued open/remove/reorder actions use UUIDs. Removing the
  selected entry chooses an adjacent remaining entry, so ordered Delete/Return
  operates on the updated list. Invalid identities and slots do nothing.
- Dragging inserts before/after a visible row, preserving other entries' order.
  Context menus offer Go, Move Up/Down, Move to Shortcut and Remove; unavailable
  directions/current shortcut slots are disabled. All mutations save through the
  existing scoped store. Shortcut numbers still resolve the current slot.
- Repository/scope loading retires list state. A stale UUID cannot resolve to a
  different bookmark simply because its former slot is now occupied.
- `ScrollArea::show_rows` paints only visible fixed-height rows. Keyboard
  selection reveals itself while preserving an already-visible scroll position.
  Header membership prefilters unrelated line anchors before relocation scans;
  complete relocation/request caching remains in the broader cache queue.

## Structure and validation

`BookmarkStore` owns identity lookup, stable slot insertion and current-location
resolution. `ui/bookmark_list.rs` owns list presentation/focus/selection;
`ui/app/bookmarks.rs` binds commands to existing scoped persistence and navigation.
The context inspector delegates this region rather than carrying bookmark menus
and mutation policy itself. No new dependency or generic list framework was added.

At `c00308e`, workspace checks pass 277 application tests, 5 reader-text tests and
30 doctests; the external-Clang-dependent test remains intentionally ignored.
Formatting, Clippy with warnings denied, binary build and rustdoc pass. New
coverage includes invalid/stale identities, stable slot ordering, relocated
current-line lookup, rapid Home/Delete/Return with exact destination and persisted
order, disabled pending-review mutation, scope retirement, focus/modifier/release
isolation, and visible-row/reveal behavior with 1,000 bookmarks.

Architecture baseline `before-bookmark-controls` is at `71984e1`. Comparison has
no new product hotspot or dependency cycle; average CCN remains 3.34, maximum
100, duplication 0.70%, health approximately 82.69. The tool reports REGRESSION
for clone groups 90 → 91. The additional detected group is an existing pair of
keyboard-test destination assertions made detectable after consolidating raw-key
construction; it adds no product duplicate mechanism. Other reported touched-file
clones predate this change. New NLOC is +609 for the store, complete pane,
application binding and tests. This small test-only finding remains documented.

## Native evidence

The private Wayland fixture is `/tmp/radiology-bookmarks-native-2`, driven by
`/tmp/radiology-bookmarks-native-2-driver.py`. It reuses little-screen-clip's
external test/compositor helper; product builds remain standalone. Initial
captures exercise intermediate layout versions and are not final acceptance.

Final hashes and build checkpoints are recorded in `build-evidence.json` in that
fixture directory. All app/restarted-app and compositor exits are zero; pointer
helper termination is intentional. The final binary SHA-256 is
`80b76da05c3b44940929d988c4ce36d0e9e9cc1166b8c2a8007268e8ce9ee9d5`.
PID 3873238 and its restarted PID 3881943 execute that binary. Final evidence:

- `accepted-final-keys.png`: Home/Down/Return selects the second bookmark and
  opens b.rs. The source and row shortcut labels agree.
- `accepted-final-delete.png`: Delete removes the selected b.rs entry; the scoped
  JSON contains only the original a.rs UUID.
- The header re-adds the displayed b.rs location. Dragging its row above a.rs
  persists b.rs at slot 0 and a.rs at slot 1; `accepted-final-reorder.png`.
- `accepted-final-restart.png`: the same persisted order survives another native
  restart, with the current a.rs location reflected in the header.
- Earlier `shortcut-menu.png` verifies current-slot disabling; choosing Ctrl+0
  moves a.rs from slot 1 to slot 0 in the persisted JSON. This menu code is the
  same in the final build.

## Remaining acceptance and parity work

Outline selection/preview/commit and hierarchical keyboard behavior are now
integrated; see [outline acceptance](rust-outline-pane.md). Retain the final
light/dark/scaling, pinned-PR revision labels,
long-name/mixed-language corpus. Large-list drag-edge scrolling, dedicated region
diagnostics and persisted exact-location/validity behavior are now implemented and
validated in the follow-ups below. These checks do not assert full bookmark/inspector
or final Swift visual parity.

The requested swiftui-pro, macos-design and typography-designer skills are absent
from the configured skill roots. Checked-in Swift views and the Rust Body/Small
styles provide the comparison; no new type scale was introduced.


## Numbered toolbar menu and current-location toggle — 2026-10-31

View now exposes the existing ten numbered slots in Go to Bookmark, with current
labels, Ctrl+digit hints and disabled empty/unavailable slots. Its toggle changes
to Remove Bookmark at the saved location. Both use existing command dispatch,
readiness and BookmarkStore navigation/persistence. Rendered slot-9 traversal and
native 1×/2× add/remove/menu-return checks pass; see the
[command-state contract and evidence](rust-keyboard-audit.md#view-availability-and-numbered-bookmark-commands--2026-10-31).
Broader model/location/deduplication and large-list/review corpus acceptance remain
open. Full accessibility is deferred by the user.


## Exact locations, persisted validity and relocation refresh — 2026-10-31

The Swift model identifies a bookmark by path/revision/canonical byte. At 8ccd476
Rust instead resolves bookmarks against the current line during control painting:
toggling another column on that line removes the first entry. Rust also decodes
all records without Swift's validity/UUID filtering, and successful navigation
does not refresh relocated coordinates/digest in the persisted entry.

BookmarkStore::at_location now compares exact canonical bytes, path and revision.
It clamps incoming bytes to scalar boundaries like source navigation, without
line scans, anchor relocation, digest formatting or per-entry source work while
painting. Distinct columns on one line remain independent. SourceTarget::Bookmark
still resolves anchors in the existing worker. Publication checks the UUID still
exists before changing source/history, then refreshes the accepted bookmark using
the existing constructor, preserving UUID, current label and current slot. It
saves only changed metadata. A removal during preparation retires the result
without resurrecting the entry or navigating. Reordering while preparing keeps
the current slot through UUID lookup.

Loading retains the first valid record per UUID in persisted order, using the
existing repository-path validator and positive-line rule. Rust's unsigned
coordinate types reject negative positions. Individual malformed records are
skipped, allowing the other records to recover; loading never rewrites the file.
Distinct UUIDs at the same location remain distinct, as in Swift's load filter.
Scope paths, serialization format and save behavior are unchanged. No new public
API, worker, cache, dependency or sibling coupling.

Tests cover independent Unicode/CRLF columns, scalar-boundary clamping, path/
revision separation, first-valid UUID ordering, invalid paths/zero lines/negative
positions/bad UUIDs, read-only load, round-trip save and PR scope isolation.
The existing app relocation test now checks stable UUID/label, new line/digest,
current-location state and persisted refresh. A real Remove action during pending
bookmark loading preserves the exact prior source/history. Restoring old store
or publication code separately fails these regressions. Workspace passes 526 app
tests (seven existing ignored), two binary, four font, 13 reader-text and 36
doctests. Strict Clippy/check/build, warning-denied rustdoc, fmt/diff pass; rendered
store docs inspected. Architecture adds 158 NLOC including three tests; no new
hotspot/cycle/max-CCN increase. Four raw touched-file clone warnings are unchanged:
all 91 baseline clone groups match in a complete multiset audit.

Native 1×/2× runs create bookmarks at adjacent Unicode scalar columns 3 and 4,
whose canonical bytes differ by two. Fresh clipboard sentinels prove each suffix
before and after restart. The private config is deliberately seeded with an
invalid path, negative byte and duplicate UUID; both valid slots still navigate
exactly. Removing the second exact location saves only the original first UUID,
which remains navigable. Source fixture/final binary hashes and both reader plus
compositor zero exits are verified; captures inspected. These final checks place
explicit frame boundaries between independent actions.

**New confirmed remaining input defect:** an earlier rapid 2× sequence loses the
two bookmarks. A separate real-app frame probe with stable source focus confirms
Ctrl+D, Right, Ctrl+D in one batch executes global toggles before source movement
and produces zero entries. The failing probe and native trial are retained as
product evidence, not a passed test or an OCR-only harness failure. Model/restart
acceptance above does not establish mixed global/source event ordering. The next
repair must preserve that sequence through the source input pipeline, including
pending geometry and focus/document retirement. The probe is retained outside the
passing suite until that repair. Modal closing-frame input isolation remains a
separate required contract.

Evidence: `~/.local/state/radiology-qa/bookmark-model-20261031-8ccd476`.
Broader bookmark anchor bounds/cancellation, region-specific navigation errors,
review/long-name/large-list corpora, remaining command ownership and final Swift/
standalone/reuse acceptance remain open. Full accessibility and actual sibling
integration remain deferred.


## Mixed source/bookmark command follow-up — 2026-11-01

The confirmed batch defect above is now repaired: source pre-frame capture and
the existing ordered queue retain Bookmark/Right/Bookmark, including deferred
movement and explicit/implicit focus. Native 1×/2× rapid sequences and real-app
1×/1.5×/2× checks pass. Definition and References use the same caret-sensitive
command boundary; explicit navigation retires later source commands.
[Implementation, command boundaries, validation and limits](rust-source-input-order.md#caret-sensitive-reader-commands-share-source-order--2026-11-01).
Other mixed-owner/pointer/command corpus and full parity acceptance remain open.


## Bounded anchors and cancellable relocation — 2026-11-01

Swift omits anchor/context lines over its display-line limit and checks task
cancellation every 256 lines. Rust previously copied entire source lines into
bookmark records and scanned matches without cancellation. Creation now shares a
bounded-line helper using the existing 2,000-byte limit, including UTF-8/CRLF
boundaries. Unchanged source still resolves its exact byte even with no anchor;
changed source with an omitted/oversized anchor fails conservatively.

Relocation checks cancellation at entry, every 256 lines and before publication.
The existing source worker supplies its cancellation/deadline predicate through
SourceTarget; its post-resolution check preserves cancelled/timed-out diagnostics.
Public resolve remains compatible. The scan retains at most two raw and two
context-qualified candidates, instead of all matching lines. Two qualified matches
prove ambiguity immediately. A unique raw match still wins without requiring
context; multiple matches require one complete bounded-context match. Omitted
oversized context cannot prove uniqueness. Saved scalar columns use existing strict
coordinate conversion, rejecting invalid columns instead of clamping silently.

Five new tests cover byte limits, legacy oversized anchors, unchanged-source
navigation, contextual ambiguity, Unicode columns, cancellation before exact and
mid-scan resolution, source-target propagation and failed navigation preserving
source/history/bookmarks. Restoring old creation or removing scan/source-target
cancellation separately fails regressions. A shared reader fixture replaces new
duplicated test setup. Workspace passes 535 app tests (seven ignored), two binary,
four font, 13 reader-text and 36 doctests. Strict Clippy/check/build/rustdoc/fmt/diff
pass. Architecture adds 174 NLOC including tests/fixture; no new hotspot/cycle or
maximum-CCN increase. Raw touched-file clone warnings are audited against baseline.
The constructor CCN falls to 1; cancellable resolution is CCN 15. The aggregate
health score falls by 1.24 with the additional cancellation/context policy; no
artificial code moves or new dependency/cache/worker/public API are introduced.

A 6,291,502-byte native fixture at 2× writes a 366-byte bookmark JSON record, returns
to exact unchanged bytes without an anchor, reports the changed-source anchor
error, and preserves the complete accepted source while private disk bytes differ.
The fixture is intentionally edited then restored; before/modified/restored hashes
are recorded. Final binary hashes and zero exits pass. At 1× the record and error
checks pass, but the subsequent Select All/Copy fails twice without a debugger.
A delayed debugger run passes. A second GDB trace from before the error transition
reproduces failure: source retains keyboard ownership, but Copy receives selection
None. The previous reveal's committed flag clears a newer Select All range while
its geometry settles. This is a separate confirmed reader-selection lifecycle bug;
these bookmark changes do not claim to repair it. Traces, failed runs and the
small-reader focus probe are retained. Fix selection retirement at the new
navigation boundary before accepting the complete unprofiled 1× flow.

Evidence: `~/.local/state/radiology-qa/bounded-bookmark-20261101-8918bfb`.
Broader reader timing/interaction, bookmark-region diagnostics, grammar/index/PR/
cache corpus and final Swift UI/standalone/reuse acceptance remain open. Full
accessibility and actual sibling integration remain deferred.


## Selection-lifecycle follow-up — 2026-11-01

The 1× Copy failure above is now repaired at the source navigation boundary.
Old selections retire before new input; pending reveal settling no longer clears
a later Select All. The redundant committed flag is removed. Both failing
regressions and complete unprofiled 1×/2× source/error/copy flows now pass, including
all 6,291,502 original bytes while disk bytes differ.
[Implementation, tests, native evidence and limits](rust-reader-performance.md#retire-old-selection-before-new-input-not-during-reveal-settling--2026-11-01).


## Scoped navigation diagnostics — 2026-11-01

Swift's `BookmarksPane` places `navigationError` below the Bookmarks header. Rust
now follows that ownership: immediate path rejection, cached resolution failure
and worker failure report against the still-present bookmark UUID. Failure opens
the inspector so the status is visible, preserves accepted source/history, and
leaves unrelated global diagnostics intact. Removed identities ignore late errors;
removal, repository/PR scope reset and a valid retry retire the status. An empty
numbered slot does not dismiss an existing failure. Storage errors remain global.

The existing `ErrorNotice` supplies the inline title, bounded diagnostic summary,
Details and Dismiss controls. Its complete selectable/copyable Details window now
uses its owning UI's persistent ID, separating global and bookmark notices. No
new worker, dependency, cache, storage format or public API is introduced. Shared
reader fixtures avoid duplicated app setup. Existing typography roles are reused.

Validation passes 541 app tests (seven ignored), two binary, four font, 13 text
and 36 doctests, plus strict Clippy/check/build/rustdoc/fmt/diff checks. New tests
cover missing/invalid targets, unrelated global errors, preserved reader/history,
hidden inspector recovery, retry, empty slots, removal and scope retirement. A
270-point pane exercises long Unicode/multiline summaries, full Details/Copy and
Dismiss at dark 1× and light 2×. The scaled test settles egui's pending scale before
measuring. Restoring global-only routing fails two new regressions; omitting the
inline region fails the painted test. Restoring production passes 25 bookmark tests.

Architecture comparison adds 190 NLOC including tests and ten functions, with no
new maximum complexity, hotspot or cycle. Its raw REGRESSION lists touched-file
clones; the full normalized fragment/file-pair multiset confirms all 91 baseline
groups unchanged, with zero additions/removals. Health changes +0.02.

Fresh private native 1×/2× runs preserve and copy all 6,291,502 accepted source bytes
after a controlled disk mutation, retain the 366-byte bookmark record, reopen the
hidden inspector and show the error in Bookmarks. Details copies the exact complete
message. Dismiss removes the status/window; another attempt reproduces the failure.
Restoring disk bytes and retrying clears the error, finishes loading and copies the
exact destination suffix `LONG_END`. Captures, binary/fixture hashes, restored edits
and all child exits are verified. Earlier OCR checks confused generic semantic
unavailability or split the status text; an earlier retry capture stopped at
Opening. Those preliminary trials are retained but excluded from final acceptance.
Final checks crop the Bookmarks region and observe loading completion before copying.

Evidence: `~/.local/state/radiology-qa/bookmark-notice-20261101-d4249db`.
Broader mixed-owner input, reader/corpus performance, language/index/PR/cache and
final Swift feature/visual/standalone/reuse acceptance remain open. Full accessibility
and actual sibling integration remain deferred.


## Held-drag edge scrolling and cancellation — 2026-11-01

The virtualized Rust list could only reorder within its visible rows. It now
scrolls while an existing bookmark drag is held inside the list's top/bottom
24-point edge bands. Speed increases with proximity to the edge, uses logical
points and elapsed frame time, and caps a frame's time contribution at 50 ms.
The existing ScrollArea owns offsets and clamping; existing UUID payloads and
Move actions own reordering/persistence. Holding never commits a move. No new
persistent state, timer, worker, dependency, public API or sibling coupling.

The pointer must remain down in the enabled, unobscured list viewport and its
identity must still exist in the store. Leaving that viewport, releasing, disabling
the region or retiring the identity stops requesting edge movement. Drag start
focuses the existing list keyboard owner; Escape clears only its bookmark payload
through ordinary app dispatch. Releasing after Escape cannot reorder or navigate.
Right-click selection keeps its previous focus policy. Row CCN remains 15; the
new scrolling helper is CCN 7 and cancellation is CCN 2.

A 1,000-entry regression initially fails with offset 0→0. It now scrolls over 400
logical points in either direction at 1×/1.5×/2×, never commits while held, drops
the original identity at an initially off-screen slot, and stops on release. A real
app frame test covers drag focus, Escape, subsequent release, unchanged ordering,
source identity and history. Removing focus transfer or cancellation separately
fails it; restored production passes all 27 bookmark-related tests. Full workspace
passes 543 app tests (seven ignored), two binary, four font, 13 reader-text and
36 doctests. Strict Clippy/check/build/rustdoc/fmt/diff pass.

Architecture adds 140 NLOC including tests and five functions. No new hotspot,
cycle, maximum complexity increase or clone group. The raw touched-file clone
warning is the unchanged seven-line bookmark/sidebar group: normalized full
multiset audit retains all 91 baseline groups with no addition/removal. An initial
row change increased CCN to 17; grouping selection/focus by gesture restored 15
before final verification, without changing right-click behavior.

Final native private-window checks at 1×/2× seed 100 records from one real bookmark,
then restart and hold each direction for 2.5 seconds. The first identity moves
from slot 0 to 27; the last moves from 99 to 72/71. Storage is byte-identical while
held. Escape keeps persisted ordering unchanged and at least two independently
recognized row positions remain stationary through release. After another restart,
Ctrl+0/Ctrl+9 copy the exact source suffix belonging to their reordered identities.
Viewed fixture and final binary hashes, inspected captures and all eight child
exit statuses per scale pass. QA alone imports sibling capture helpers and uses
a locally extended private pointer helper; application dependencies are unchanged.
Preliminary captures are retained: their strict OCR label-set check failed when
a normal release tooltip obscured a label, although the shared positions were
unchanged. Final QA validates common visible anchors and records their coordinates.

Evidence: `~/.local/state/radiology-qa/bookmark-drag-20261101-2c7c3cc`.
Remaining acceptance includes pinned-review/mixed-language/long-name inspector
corpora, other cross-owner interactions, broader reader/index/PR/cache behavior
and final Swift UI/standalone/reuse audit. Full accessibility and actual sibling
integration remain deferred.


## Empty scope guidance and readable unavailable states — 2026-11-01

Swift distinguishes an absent scope from an empty bookmark collection. Rust now
shows No Repository with repository-opening guidance until a root is accepted,
and No Bookmarks with Ctrl+D/reordering guidance in an accepted checkout or PR.
It no longer labels an unopened session Checkout or suggests bookmarking a current
location that does not exist. Adding an entry replaces the explanation with rows;
removing the last entry restores it. Existing Body/Small roles and the shared
three-row bounded explanation renderer are reused.

`InspectorContent.scope` is now `Option<&str>`: callers pass None before a repository
is accepted and Some(title) for checkout/PR context. App rendering derives this
from the existing accepted root, without filesystem work or extra persistent state.
This is an explicit presentation API adjustment; bookmark storage is unchanged.
Empty guidance renders outside whole-pane dimming, while the header action requires
both region and source readiness. Nonempty lists retain their existing disabled
behavior during repository/review transitions. No new production function, worker,
cache, dependency or type scale. Shared empty-repository test setup replaces a
duplicate fixture and retains the existing Critical Lens assertions.

The real app regression first fails on missing No Repository. It now covers absent
root, accepted empty repository, populated root/source, shortcut Add and last-entry
removal at 1×/1.5×/2×. Narrow 180×120-point component frames cover checkout/PR/absent
scopes, three-row guidance bounds and disabled/enabled header Add at dark/light
scales. Full workspace passes 545 app tests (seven ignored), two binary, four font,
13 reader-text and 36 doctests. Strict Clippy/check/build/rustdoc/fmt/diff pass.
Architecture adds 108 NLOC including tests and three test functions/fixtures, with
no new hotspot, cycle, max complexity or clone group. Raw touched-file clone flags
are unchanged: normalized multiset audit preserves all 91 baseline groups.

Six native private-window cases cover absent root, empty repository and loaded
source at 1×/2×. Captures verify correct headings/guidance and normal text contrast;
Ctrl+D and the observed header plus cannot create bookmarks without a source.
Loaded source accepts keyboard Add, last removal and header Add; complete source
copy remains byte-exact. Fixture/binary hashes, captures and all child exits pass.
Earlier QA trials confused the header with No Bookmarks and read Checkout as
Cneckout. They are retained and excluded; final QA selects the earliest header,
normalizes that observed OCR substitution and retains raw OCR/captures. The initial
app fixture also assumed repository opening left its source empty; production
auto-opens a file, so the final regression uses a genuinely empty root.

Evidence: `~/.local/state/radiology-qa/bookmark-empty-20261101-d6e762a`.
Broader inspector/input/reader/language/index/PR/cache corpus and final Swift
feature/visual/standalone/reuse acceptance remain open. Full accessibility and
actual sibling adoption remain deferred.
