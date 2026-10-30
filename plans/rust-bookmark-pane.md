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

Outline selection/preview/commit keyboard behavior is still separate unfinished
work. Also retain the final light/dark/scaling, pinned-PR revision labels,
long-name/mixed-language corpus, desktop accessibility and large-list drag-edge
scroll acceptance. Bookmark navigation errors currently use the application error
surface rather than Swift's dedicated bookmark-region status. Swift's persisted
entry validity/deduplication and exact bookmark-location semantics merit a further
cross-language audit; this pane work does not assert full bookmark-model parity.

The requested swiftui-pro, macos-design and typography-designer skills are absent
from the configured skill roots. Checked-in Swift views and the Rust Body/Small
styles provide the comparison; no new type scale was introduced.
