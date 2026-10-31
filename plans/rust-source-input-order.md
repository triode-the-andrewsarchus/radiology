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
