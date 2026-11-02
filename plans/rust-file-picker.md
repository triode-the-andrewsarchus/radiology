# Rust file picker parity

## Search foundation accepted — 2026-11-02 (`ec900a8`)

Repository inventories are immutable `Arc<[RepositoryFile]>` snapshots. Opening a
replacement repository publishes a fresh snapshot only on acceptance; failure or
cancellation retains the previous inventory identity. Review acceptance publishes
its inventory in the same form. Hosts assigning `all_files` convert a Vec with
`.into()`; slice readers continue to work through dereferencing.

The existing bounded file ranking implementation now supports cancellation before
normalization, after normalization, every 128 candidates and before publication.
Cancelled work returns no partial matches. The public synchronous search method
still uses the same ranking implementation with cancellation disabled. Zero limits
retain truthful totals, and the existing visible limit remains 100.

## Picker UI and lifecycle — 2026-11-02

The public `QuickFilePickerModal` stays at `ui::modals::QuickFilePickerModal`;
its implementation now lives in a focused submodule. `show` takes a shared
`Arc<[RepositoryFile]>` inventory. Default and Clone remain available; cloning
copies query/selection and creates independent worker and interaction ownership.

Query or inventory changes retire the existing `LatestRead` job and its publication
channel, clear stale rows and selection, and launch bounded ranking on the existing
four-worker read pool. There is no new thread pool or dependency. Empty/loading,
failed search and retry states disable Open until a current selection exists.
Closing cancels work. Immutable snapshots make per-frame inventory comparison
constant time and keep worker inputs alive independently of repository replacement.

The 620×430 logical-point picker follows Swift's broad layout: filename in Body,
directory in muted Small, fixed two-line rows, 100 visible results, truthful total/
limit notice, separate No Files/No Matches feedback and Open/Cancel controls.
Only viewport rows are shaped; shared measured text fitting bounds long Unicode
paths without changing their canonical destinations. Row IDs include popup,
query, inventory and path ownership, and activation checks the live rectangle.

Mixed keyboard events retain their order: text reaches the query before the next
arrow/Return; repeated arrows each move selection. Return waits for that exact
query's current result and discards its closing suffix. Query/inventory replacement
retires queued activation. Selection is clamped before activation; shared list
movement and reveal keep the filename and directory visible. Escape cancels even
while a query or queued Return is pending. The existing app modal boundary retains
source keyboard ownership on the closing frame and restores source/prior focus.

## UI verification

Seven focused regressions cover narrowing+Return from selection 99, repeated arrows
past 12 and viewport clipping, mixed text/key order, in-flight query/inventory/
cancellation retirement, failure/Return/retry, empty states and long Unicode rows
at both scales, and pointer release across replaced results followed by a fresh
click. Existing real-app opening-batch, history, source-load focus and later-find
focus assertions now await actual asynchronous picker publication through frames.
Fresh workspace: 590 application tests passed, seven ignored; two binary, four
font, 13 text and 37 doctests passed. Strict Clippy, all-target check, binary build,
rustdoc with warnings denied, formatting and diff checks pass.

Architecture against `before-file-picker-ui`: default comparison says REGRESSION
because shifted line locations in edited files are flagged as new duplicates.
Full normalized clone audit proves zero additions and one old picker clone removed
(110→109). +605 NLOC including seven regressions and shared frame polling; 23 added
functions, one focused module, max new CCN 14, no new hotspot/cycle, health +0.13.
This records the tool warning without hiding it or changing analysis scope.

Native final runs v4 at 1× and v5 at 2× pass over 241 real files: the capped
100/240 query, Down×28 opening file0028, selection 50 narrowed to file0007 plus
Return, No Matches/Escape, long Unicode path/Return, empty-query 100/241 and
pointer Cancel, then light-theme pointer Open. Seven fresh exact source copies
per run verify navigation and source focus after Open/Escape/Cancel. Fixture and
binary hashes remain unchanged and all child exits are zero. Captures were
inspected for list reveal, path bounds and light/dark layout. Early v1/v2 OCR
misses, v3 QA output filename overflow and v4 2× Open OCR miss are excluded from
acceptance; the final 2× pointer targets the observed Open button position.
No product rebuild or source edit occurred between accepted runs.

Evidence: `~/.local/state/radiology-qa/file-picker-ui-20261102-ec900a8`.
Broader modal/symbol-picker and corpus acceptance remains part of the open parity
goal; this file-picker slice does not establish whole-application parity.

## Foundation verification

Fresh workspace run: 583 application tests passed, seven ignored; two binary,
four font, 13 text and 37 documentation tests passed. Strict Clippy, all-target
check, binary build, rustdoc with warnings denied, formatting and diff checks pass.
Cancellation regression stops after partial ranking and verifies fresh result
order/counts; the repository transaction test checks retained/replaced Arc identity.
Architecture against `before-file-picker-search`: NEUTRAL, +51 NLOC, two functions,
no new hotspot or cycle. All 110 clone groups are unchanged by normalized audit.
No additional dependency or worker pool is introduced.

Evidence: `~/.local/state/radiology-qa/file-picker-foundation-20261102-699a55e`.
Full accessibility and actual sibling integration remain deferred; broader parity
remains open.

## Shared picker components — 2026-11-02

The symbol-picker integration now reuses the file picker's ordered input, window,
query/body/footer geometry, measured two-line rows and scrolling through concrete
shared components. Selected details use primary text color; other rows retain
muted metadata. All seven file-picker regressions and fresh native v6 flows at
1×/2× pass, including seven exact source copies each. See the
[symbol-picker integration evidence](rust-symbol-picker.md#integration-verification)
for the final binary, full test totals and architecture audit.
