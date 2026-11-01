# Ordered source keyboard input

Pointer gestures and deferred measurement now extend this same queue; see
[ordered source gestures](rust-source-gesture-order.md) for the current contract.

## Failure and implementation

Native 2× font acceptance exposed a source-reader batching bug: a rapid
Home → Shift-End → Copy sequence could leave the previous clipboard unchanged.
The source viewport processed Copy before navigation and consumed only one
navigation event in each frame. Multiple navigation/selection events arriving in
the same frame lost their order or were discarded at frame retirement.

The existing command dispatcher now exposes its ordered event-consumption step
for both pressed keys and platform Copy events. The source viewport resolves its
owned inputs through that mechanism and applies every action in arrival order.
Each movement updates the caret immediately so the following movement/selection
uses the new position. The final caret action is published to the app once.
Unrelated events remain available to their existing owners. Shaped navigation,
selection geometry and exact-original-byte copying reuse the existing code.

## Validation — 2026-10-30

The regression test `coalesced_navigation_selection_and_copy_follow_input_order_at_each_scale`
uses the installed reader fonts at 1×, 1.5× and 2×. It covers Home/Shift-End/platform
Copy, repeated Right/Shift-Right/Ctrl-C across multibyte text, and Select All/Copy
in one frame. The test fails against the previous implementation and passes with
the fix. Logs: `/tmp/radiology-key-order-{before,after}.log`.

Workspace tests pass: 433 app tests (seven ignored), three reader-fonts tests,
eight reader-text tests and 31 doctests. Strict Clippy, build and formatting pass.
Logs: `/tmp/radiology-key-order-{workspace,clippy,build}.log`.
Architecture comparison against `before-source-key-order` is neutral, with no new
hotspots, code clone groups or dependency cycles. The dispatcher is extended rather
than duplicated.

Native verification uses the actual rebuilt app in a private Weston/Niri session
at 2×. Rapid Home/Shift-End/Ctrl-C copies the entire emoji line exactly; the same
sequence also copies the mixed CJK comment exactly. Clipboard reads precede screen
captures because Niri screenshots replace the clipboard with PNG data. Source
bytes remain unchanged. `/tmp/radiology-fonts-native-2.0/acceptance.json` and
`SHA256SUMS` record copy checks, screenshots and zero app/compositor exit statuses.
The app closes normally through Niri; interrupting the idle QA driver's stdin
loop runs its cleanup block and reaps the children. This does not establish a
complete keyboard/accessibility audit or broad source-corpus acceptance.


## Caret-sensitive reader commands share source order — 2026-11-01

At 5cc24d2, a native 2× sequence and a real-app frame probe confirm that Ctrl+D,
Right, Ctrl+D executes both global toggles before source movement, leaving no
bookmarks. Fixing only global deferral exposes another case: with explicit source
focus, egui moves focus before the retained movement reaches the canvas. A trace
shows pending source input retired for lost focus. Implicit source focus does not
have that failure. Merely setting the focus filter at request time is insufficient:
egui 0.27 requires the widget to have had focus in the preceding frame.

The existing pre-frame inspector capture now also protects a live source owner.
Its private event buffer is named reader_input to reflect both owners; there is
no second raw queue. Tab retains native traversal, and modal/context-menu ownership
still takes precedence. Source owns input when explicitly focused or when its live
canvas is present and no text control owns keyboard input.

A shared source_command resolver reuses existing shortcut matching for Bookmark,
Go to Definition and Find References. The app leaves those events in place for
the existing document-bound source queue. Movement, selection and pointer geometry
resolve first. At a command, that queue emits the shared ReaderCommand action and
yields to application dispatch; the remaining ordered input resumes on repaint.
No callback into partially borrowed application state or new command execution
pipeline is introduced. Command handlers, availability, persistence and semantic
lookup remain shared with menu/inspector paths. Explicit navigation reuses source
input cancellation immediately; focus/document/zoom/manual override and modal
retirement retain their existing rules. Modal-opening and other global command
groups keep their current ownership paths; this does not certify every mixed
command/pointer/focus sequence.

Real-app tests at 1×/1.5×/2× with explicit and implicit focus retain bookmarks at
canonical bytes 3 and 5 of Unicode/CRLF source, then copy the exact later selection
from the same batch. Tests also move into an indexed occurrence before References
or Definition and verify navigation retires a later source bookmark command.
A >1 MB deferred-layout fixture verifies the command waits for shaped movement,
yields before Copy and resumes the exact CRLF/Unicode selection. Existing document,
zoom, focus and pointer-override retirement checks now include a queued command.
The raw pre-change probe is replaced in the passing suite by ordered assertions
that drain the command boundaries; it does not require all side effects in one
paint. Removing application deferral fails with zero bookmarks; removing source
pre-frame capture separately loses the second explicit-focus bookmark.

Final workspace passes 530 app tests (seven existing ignored), two binary, four
font, 13 reader-text and 36 doctests. Strict Clippy/all-target check/build,
warning-denied rustdoc, fmt and diff checks pass; the new action's rendered docs
are inspected. Architecture adds 173 NLOC including tests/fixtures and ten
functions, with no new hotspot/cycle/max-CCN increase. Shared indexed-gap and
Unicode deferred-command fixtures remove two genuinely duplicated test workflows.
The final raw warning concerns an unchanged eight-line source-view clone; a full
multiset comparison proves all 91 baseline clone groups identical.

Native 1×/2× runs now preserve a rapid Bookmark/Right/Bookmark sequence at adjacent
Unicode columns. Exact independent clipboard hashes verify both slots through
restart with invalid/duplicate private configuration, then removal of only the
second slot. The dialog is closed before the rapid source sequence, preserving
the separate closing-frame isolation contract. Native imported-index fixtures at
both scales move from whitespace into alpha, pin its references (Follow Caret is
visible), navigate to a.rs byte/column 3, reject a later source bookmark, and return
with Back to b.rs's call column. Fixed-length selection and full-source copies prove
those destinations. These are controlled imports, not additional real-indexer
coverage. Initial indexed QA expected Shift+End to copy only the header, but the
collapsed declaration correctly yields its canonical hidden-body range. Those
validation failures are retained and excluded; final QA uses eight selected
characters plus complete source bytes, independent of folded End behavior.
Captures inspected, final binary/fixture hashes verified, and all children exit
zero. Final native binary hash also matches the final post-interruption build.

Evidence: `~/.local/state/radiology-qa/source-global-input-20261101-5cc24d2`.
Broader mixed-owner/command/pointer corpora, reader performance, grammar/index,
PR/cache and final Swift UI/standalone/reuse acceptance remain open. Full
accessibility and actual sibling adoption remain deferred.
