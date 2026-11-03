# Code Paths standalone sidebar and saved collections

The standalone Rust app now presents saved Code Paths as a left navigation peer.
The existing [format/store](rust-code-paths.md), [resolver](rust-code-path-resolution.md)
and [guarded source transactions](rust-code-path-app.md) remain the shared backend.
This feature adds no runtime dependency on little-screen-clip or new worker pool.
The full port still has the other open items in the upstream parity ledger.

## Interaction contract

Repository > Import Code Path… (Ctrl+Shift+P) opens a multiline editor, with a UTF-8
text-file chooser. Return inserts a newline; composition belongs to the editor.
Tab visits the editor, chooser, Cancel and enabled Import action. The dialog reuses
ModalInput's pre-frame ownership and shared focus ring. Parse or save errors retain
the authored text and current reader. Accepted saving disables competing edits and
dismissal; success closes through the existing modal boundary, selects the imported
trace and reveals the left sidebar. It does not reveal the right inspector.

View > Show Code Paths (Ctrl+Alt+P) and the peer tab reveal the same content. Existing
View-menu keyboard ordering is preserved. The four tab titles are measured; narrow
panes use vector icons with full-title tooltips. Left/Right move between focused tabs.

The sidebar has a saved-trace picker, Import, Copy/Delete/reload actions, trace-count
footer and numbered source steps. Function names and paths use the existing measured
label helpers; descriptions wrap without a line cap. Arrow/Home/End select a row;
Return or a click opens it. Open Source is also available in the row context menu.
Selection alone does not move the source caret. Switching tabs preserves the trace
and step. Missing/ambiguous declarations remain visible navigation errors.

Conditional steps initially open their first source definition. The alternatives menu
names each definition's source line and re-resolves the selected alternative through
the guarded transaction. The menu does not imply an active build configuration.
Back, fold/test reveal, destination feedback and Before/After source ownership remain
with the ordinary source transaction. Bookmarks and Context pane preferences remain
independent.

## Storage and lifecycle

CodePathSession runs one collection operation at a time using LatestRead and the
existing bounded read pool. It captures the attached canonical root and a presentation
epoch. An accepted save is not retired by close/open: the next scope load waits for
it, including an A→B→A round trip. Late rows cannot replace another scope. Background
write errors remain reportable even after leaving their scope.

The accepted collection changes only after successful persistence. Parsing, record
copying, serialization and file I/O run outside rendering. The caller cannot start
another mutation while busy. Invalid input and corrupt archives preserve both current
selection and stored data. Reload supports explicit storage repair. Same-title traces
and repeated source steps retain independent UUIDs; Copy exports authored text, not
saved identities or source coordinates. PR views use the attached checkout collection,
not the materialized head tree. Standalone shutdown calls flush_code_paths; embedded
hosts have the same public shutdown hook. This is in-process serialization, not a new
cross-process transaction guarantee.

The file chooser reads a canonical regular file on a worker, checks metadata and a
maximum-bytes-plus-one bounded read, then validates UTF-8 before replacing editor text.
Both the chooser and parser enforce the original2MiB byte cap; no arbitrary collection
size cap was introduced. Source files and private archives remain separate.

The step list reuses reader-text RowHeights, including its sibling-derived variable-
height geometry. Only the visible range is shaped/painted, with exact wrapped row
heights recorded as they become visible. There is no duplicate list layout engine.

## Verification

Eleven new tests cover serialized import/delete/load/relaunch, delayed scope switching,
read-only failure preservation, corrupt-store recovery, background errors and shutdown;
real UI import/step/variant/Copy/Delete/relaunch actions at1×/2×; editor Return, IME,
Tab submission, shortcuts, sidebar/inspector separation and stable selection; bounded
UTF-8/regular-file imports; and10,000-step visible rendering with complete wrapping
and End selection. The existing gated caret test now waits for worker startup before
asserting cancellation, eliminating its scheduler assumption. Shared app fixtures keep
private settings outside source.

Workspace validation:717 app tests pass, seven pre-existing ignored, with two binary,
13 font and21 text tests. All55 doctests and strict Clippy/check/build/rustdoc/fmt/diff
checks pass. Rendered collection/modal/app shutdown API contracts inspected.

Native private Wayland runs at1× and2× import a Unicode trace, open main and both
conditional definitions, navigate Back, copy all84 original UTF-8 trace bytes, switch
to light appearance and relaunch with saved trace/step UUIDs intact. Go to Line field
Copy verifies18 and25 exactly, replacing unreliable footer OCR. Captures are reviewed;
source/binary hashes are unchanged during each run, core app/compositor exits are zero.
Hover helpers are intentionally terminated during owned cleanup. Native QA tooling
imports the sibling's read-only capture helpers; the app does not depend on them.

Earlier attempts retain diagnostic logs/captures: a missing private runtime parent,
fractional pointer coordinates, held-hover process handling, tooltips covering a row,
and OCR misreading18/25 or Copy case. Full test retries also retain a corrected scope
fixture, preserved View-menu order, a transient existing deep-call timeout under load,
and rustdoc linking against an artifact replaced by overlapping builds. Final library/
binary tests and documentation gates run separately to avoid that artifact race.
The large-list test warms the pending2× viewport change before checking actual pane
geometry; it keeps its visible-row bound and exact last-step assertion.

Evidence: ~/.cache/radiology-qa-code-path-sidebar.
Archive: ~/.local/state/radiology-qa/code-path-sidebar-20261102-1bd346c.

## Architecture review

Baseline before-code-path-sidebar at1bd346c. Initial new clones in storage/UI fixtures
were eliminated through shared loaded-collection and source/settings-isolated fixtures.
The import footer and reader input ownership now have cohesive shared operations,
removing the initial excessive modal/dispatcher complexity. No new parser, background
pool, archive writer, source navigation/history implementation or dependency was added.

| Dimension | Baseline | Accepted |
| --- | --- | --- |
| Analyzed NLOC |98,795 |100,119 (+1,324, including tests) |
| Functions /files |4,131 /634 |4,183 /640 |
| Average /maximum CCN |3.39 /132 |3.39 /132 |
| High-complexity functions |33 |33 |
| Clone groups /dependency cycles |126 /1 |126 /1 |

The raw tool still labels touched pre-existing test/sidebar clones and shifting
untouched Python chokepoint ranks as REGRESSION. A normalized baseline fragment/path/
token audit finds zero added or removed clone identities. The actual new clone groups
and new hotspots from the first implementation were removed.

## Remaining acceptance and port work

The Code Paths standalone workflow and real GTK desktop-portal file selection
are implemented and accepted as described below. An actual IME engine/candidate
window still needs environment-specific acceptance; synthetic composition and
byte-bound reading/pasted native import are already verified.
A native saved-trace PR workflow and broad unusual-input/appearance stress acceptance
remain useful follow-ups alongside the port's existing broader corpus/input QA.

C/C++ fallback, indexing personalities/seL4 and authored keyboard help are now
implemented and accepted in their dedicated checkpoints. Use the [current parity
reconciliation](rust-parity-status-2026-11-02.md) for remaining integrated work.
Full accessibility and actual sibling adoption remain deferred by scope.

## Actual desktop portal acceptance — 2026-11-02

The real installed xdg-desktop-portal/GTK backend now passes at native 1× and2×,
through the app's actual rfd chooser. A private D-Bus session and XDG config/data/
state directories select only the GTK FileChooser backend; GTK settings use memory
and local GIO. No replacement portal, injected result or simulated file read is
used for this acceptance.

Starting with authored editor text, Tab/Return opens Choose Text File. Cancelling
the GTK chooser preserves the exact draft (verified by clipboard bytes). Retrying
and selecting an actual path containing café returns the complete UTF-8 trace,
including its crab character and final newline. Editor Select All/Copy matches the
original file bytes. Three Tab presses and Return submit Import through the real
footer, then atomic storage preserves exact title/description and stable identities
across restart. Code Paths reopens the saved trace; no chooser remains mapped.
Source and executable hashes stay unchanged, app/restarted app/compositor exits
are zero, and portal/hover helpers terminate intentionally during cleanup. Final
chooser/editor/restored-trace captures were visually reviewed.

This run also exposed the singleton label '1 steps'; it now uses the same singular/
plural pattern as the collection footer and displays '1 step'. All41 Code Path
tests, strict Clippy/build and the final native portal runs pass after that small
UI change. The preceding complete workspace at3add1c3 passed757 app tests (nine
opt-in ignored), two binary/13 font/21 text tests and59 doctests. No new tests or
mechanisms were added for the label change. Architecture against
before-code-path-step-label is NEUTRAL: +1 analyzed NLOC, no new functions, clone
or cycle growth; the existing presenter CCN changes14→15.

Earlier driver trials are diagnostic: GTK selection uses its visible Select
control, and sparse OCR had mistaken the Import heading for the footer action.
The accepted runs activate Choose/Import with the existing keyboard focus ring
and use observed GTK Cancel/Select positions. The product chooser returned valid
text before the driver was completed; no backend repair was needed.

Evidence: ~/.local/state/radiology-qa/code-path-portal-20261102-3add1c3.
Actual IME-engine/candidate-window and integrated PR/stress acceptance remain in
the master ledger. Source composition injection does not claim IME-engine proof.
