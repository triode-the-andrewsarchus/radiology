# Source pointer definition navigation

## Behavior

Swift's ReaderTextView handles Command-click independently of ordinary selection.
Rust now implements its Linux Ctrl-click equivalent through the existing
NavigateToDefinition action and indexed asynchronous navigation. It does not move
the source caret to the clicked token before recording departure history.

A primary Ctrl/Command press over a resolvable source occurrence shows the link
cursor and emits its exact SemanticSymbolID. It preserves selection/caret, stops
any drag and dismisses the hover. Shift/Alt combinations retain ordinary selection
behavior. The existing ReferenceTarget resolver supplies typed local/global
identity, encoding-aware byte lookup, source freshness and Before isolation;
find_definition_for_id determines actual destination availability. Missing,
unresolved, stale and Before targets fall back to ordinary pointer selection.
There is no lexical definition guessing or second navigation pipeline.

Source context menus reuse the same definition target and action. Go to Definition
is disabled without a destination; its shortcut hint includes Ctrl+Alt+J/F12.
Copy Selection is disabled for an empty selection. Find References and Critical
Lens marking keep their independent availability rules.

## Validation — 2026-10-30

The actual egui pointer regression fails before the change: Ctrl-click moves the
caret and produces CaretMoved instead of navigation. It now covers global and
file-local symbols, a UTF-16 coordinate after an emoji, retained departure caret
and selection, stale source digests, source-only Before, unresolved symbols and
Shift/Alt combinations. The source harness now supplies the actual input modifier
state and semantic snapshot to the view. Pointer-driven disabled menu tests prove
unavailable actions neither run, copy text nor dismiss the popup.

Workspace gates pass 445 app tests (seven ignored), two binary resource-checker
tests, four font tests, eight text tests and 33 doctests, strict Clippy/build/fmt,
warning-denied/render-inspected rustdoc and diff checks. Architecture adds no clone
group, cycle, hotspot or maximum-CCN increase. Duplicate graph-input setup was
removed through the existing reader test harness. The comparison still flags two
old context-inspector/source-view clone fragments: all source_view code is byte
identical to its baseline except the NavigateToDefinition documentation sentence.

Native private 1280×900 dark Wayland evidence verifies Ctrl-click from a departure
caret on b.rs line 1 to alpha's definition in a.rs. Back restores b.rs line 1,
column 4; Shift+End/Copy returns exactly `eta() {`. The source menu navigates the
same indexed call. A whitespace menu disables definition and copy; clicking the
disabled action leaves a byte-identical PNG. Fixtures stay unchanged and app,
Weston and Niri exit zero. This uses a controlled unverified SCIP import, not
another real-indexer claim. Durable screenshots, hashes, copy bytes and drivers:
`~/.local/state/radiology-qa/control-click-20261030-9449b3e` with SHA256SUMS.
QA helpers use sibling compositor tooling; production remains standalone.

## Remaining keyboard defect observed during native acceptance

Down moves the underlying source caret from line 2 to line 3 while its context
menu remains open. Escape then closes the menu. This is captured in
menu-arrow-ownership.png versus source-menu-disabled.png. Full context-menu key
ownership/traversal is not fixed by pointer navigation and remains a required
keyboard repair. Broader reader/corpus/PR/cache and final Swift acceptance also
remain open; full accessibility and actual sibling adoption are deferred.
