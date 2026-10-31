# Ordered source gestures across deferred measurement

The distant-click defect recorded in [unwrapped geometry](rust-unwrapped-fragment-geometry.md)
is repaired. The saved failing regression now runs as a passing viewport test.
Retained source gestures are integrated. Toolbar/modal return and bounded source
labels now have their own acceptance records; broader cross-owner checks remain.

## Ownership and implementation

The existing ordered source-input queue now carries pointer gestures alongside
navigation, selection and Copy. The private viewport/input.rs module owns capture,
retirement and dispatch; it reuses projected fragments, UnwrappedRows, bounded
layout/hit testing, ByteSelection and the existing semantic definition resolver.
No parser, text renderer, dependency or parallel keyboard resolver was added.

Outside a pending provisional reveal, a gesture stores its projected row and document-local coordinates, original screen
point, modifiers and time. Width estimates never become guessed byte targets.
If the selected fragment's prefix is unmeasured, dispatch waits at that event and
prioritizes its frontier. Later release, selection and Copy remain behind it.
Once ready, the same bounded shaped galley resolves the exact canonical byte.
[Immediate provisional reveal](rust-reader-performance.md#reveal-before-complete-prefix-measurement--2026-10-31)
instead captures the displayed fragment and fragment-local coordinates, retaining
its exact hit as prefix estimates refine. It shares the same bounded galley and
ordered dispatcher.
Ctrl-click resolves the current semantic target at dispatch and preserves the
departure selection/caret. Its queued release cannot retarget a new document.

Adjacent drag motions coalesce without crossing a key/button barrier. Stationary
pointers outside the viewport continue extending selection during vertical
autoscroll. Real scrollbar response rectangles and test-label controls are
excluded from source capture. Fragment widgets leave keyboard focus with the
stable source canvas. Hover remains immediate and separate from committed input.

Source/projection/font changes, focus loss, manual scrolling, new host navigation
and disappearance of the owner retire pending input. A new pointer interaction
supersedes unresolved keyboard-only navigation. Ordered pointer gestures keep
their own release and following keys. Deferred menu invocation also retires on a
new press or explicit global dismissal.

Source menus now use the existing shared component once per canvas, with exact
byte/content scope and an explicit opening point. The opening reservation holds
its captured keys while the source target measures. A live owner renews it each
frame; absent owners expire. The same menu key classifier supports both the app's
raw hook and hosts that supply prepared egui frames. Menu actions are rebuilt
from the current source/index rather than cached before measurement.

## Verification

Six added viewport tests cover the original lost click, shaped Unicode click/
Shift-click/drag plus ordered Copy, deferred Ctrl-click departure state, deferred
menu Down/Enter for raw/prepared hosts, menu retirement at eight ownership
boundaries, and scrollbar/stationary-autoscroll behavior. A separate linear
bounded-layout reference checks the exact bytes independently of UnwrappedRows.
Existing navigation retirement tests now also cover a manual source click.
All 35 source-viewport tests pass.

Workspace gates pass 463 app tests (seven ignored), two binary checker tests,
four font tests, eleven text tests and 34 doctests. Strict Clippy, build, fmt and
diff checks pass. Architecture comparison against before-deferred-source-gestures
is neutral with no new clone, hotspot, dependency cycle or maximum-CCN increase.
Initial retirement/capture functions exceeded complexity thresholds; separating
pointer lifetime from host navigation and motion queuing removes those flags.

Final private Niri runs at 1× and 2× verify exact `_END` pointer selection/copy,
right-click followed by Down/Enter, cross-line CRLF drag copy, Copy All, and source
keys after an explicit click back from the toolbar in light mode. Each expected
clipboard SHA is checked; fixtures remain unchanged and all child exits are zero.
Captures were inspected. The final runs wait for a painted source gutter before
input and separate toolbar dismissal/focus frames. They prove those explicit
paths, not automatic focus return or input during the loading state.

Durable evidence, source/binary/fixture hashes, driver scripts, assertions, logs
and SHA256SUMS are under:
`~/.local/state/radiology-qa/source-gestures-20261031-0fd76ed`.
The archive includes excluded exploratory trials so their failures are not
silently counted as final acceptance.

## Findings still requiring work

- The long-name Files/source-header defect is now repaired by
  [bounded labels and source breadcrumbs](rust-pane-labels.md), with exact-path
  copy/jump and native 1×/2× evidence. Other long-label surfaces remain in the
  broader UI audit.
- [Modal return](rust-modal-focus.md) and [toolbar ownership](rust-toolbar-keyboard.md)
  now repair the reported source-focus gaps with native 1×/2× acceptance. Broader
  rapid cross-owner batches remain in the final keyboard audit.
- [New marker/hash-gated 6 MB checks](rust-reader-performance.md) now establish
  exact full-source, End suffix and cross-file Back bytes, and preserve selection
  across a wrap toggle at 1×/2×. Immediate anchored unwrapped reveal now has
  native visual and exact pointer-hit evidence at both scales. Wrapped reveal,
  broader performance, many-line, scroll and zoom acceptance remain open. The earlier roughly 6 MB trials render, but their initial loading/CPU and clipboard
  observations do not establish a specific performance defect: some probes ran
  before source readiness, and the old clipboard reader accepted stale offers.
  The corrected driver waits for the expected payload hash. Do not treat those
  excluded trials as a passing performance test or a diagnosed parser stall.

The DAP adapter was unavailable and ordinary attachment was denied by ptrace
policy. A separate private QA child temporarily allowed attachment; installed GDB
sampled its main thread waiting in the event loop and detached. That ordinary
fixture trace does not diagnose the large-file trial. The private child exited
zero, the DAP session was stopped, and no system policy or product dependency was
changed.

Remaining Swift/corpus/index/PR/cache and standalone/reuse acceptance stays in the
[master implementation queue](rust-parity-implementation.md). Full accessibility
and actual little-screen-clip adoption remain deferred.
