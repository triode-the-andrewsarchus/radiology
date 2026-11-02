# Inspector keyboard ownership and global commands

## Accepted behavior — 2026-10-31

Inspector controls and global shortcuts resolve each key in event order. Opening a
modal stops later keys from navigating a reference or caller behind it. Opening
Find or repository Search hands subsequent text to that query. A pending caller
focus request cannot steal focus from the newly opened query. Hiding the focused
inspector returns focus to the source viewport.

The standalone app reuses the command shortcut table, ordered input collector,
local focus guards and existing inspector action handlers. Buttons and keyboard
commands share the inspector action dispatcher. Inspector input is retained before
egui's automatic focus processing, then dispatched before asynchronous publication
can replace its visible targets. Repository reset clears that queue. Tab remains
with egui traversal. Public component APIs remain unchanged.

A modal's first window pass can measure invisible, disabled content. Its opening
keyboard input is retained through that pass and replayed on the next requested
frame, after the text field registers. This avoids accepting the first unfiltered
file when Ctrl+K, query text and Return arrive together. PR input retains its
one-time focus request after the field registers, preserving Cancel traversal.

## Verification

Four new real-app RawInput tests failed against the previous routing and pass with
the change: modal prefix from reference/caller focus; file-picker opening text plus
Return with exact Back departure; caller-to-Find text handoff; and repository Search
opening text. The Find case includes a pending caller focus request. Existing
bookmark and outline tests now run the complete raw-input/render lifecycle. The
PR toolbar cancellation test includes the requested replay frame and still checks
modal closure, restored source focus and no review launch.

Workspace: 504 application tests pass, seven ignored; two binary, four font,
eleven reader-text and 34 doctests pass. Strict Clippy, all-target check, build,
formatting and diff whitespace checks pass.

Architecture baseline: `before-cross-owner-key-order` at `752e115`. Comparison
reports REGRESSION for 89 → 90 clone groups. Full fragment/file-pair multiset
inspection identifies one added six-line clone containing test history assertions;
no existing clone was removed and no production clone was added. These explicit
outcome assertions stay local to their tests rather than introducing a test-only
navigation abstraction. Average CCN is 3.36 → 3.35, maximum remains 100,
high-complexity functions remain 24 and dependency cycles remain zero. The new
input module replaces separate batch dispatch paths; it does not duplicate a
shortcut table or add a dependency.

Native 1×/2× runs show Find receiving `target`, repository Search receiving
`needle`, and the file picker opening `a.rs` after shortcut/text/Return are sent
in one driver invocation. Caller navigation, Back, narrow dark/light presentation
and exact 473-byte Unicode copy remain intact. Both runs leave fixture hashes
unchanged and all application/compositor exits are zero. RawInput tests establish
exact event coalescence; native timing is not used to infer frame boundaries.

Evidence logs and native runs are archived outside the repository under
`~/.local/state/radiology-qa/owner-order-20261031-752e115`.

## Remaining acceptance

This increment covers inspector-to-modal/query transitions. Arbitrary interleaved
pointer/Tab/global/source gestures, multiple text-owner changes in one event batch,
and menu-local/global ordering still need acceptance. It does not establish full
keyboard or Swift UI parity. Mixed-language and pinned-review visuals, lifecycle
and error recovery, large-file/reader performance, grammar/index generation,
PR/cache validation and the final requirement-by-requirement Swift/reuse audit
remain open. Full assistive accessibility and actual sibling adoption are deferred;
keyboard shortcuts and focus/navigation remain required.

## Pointer/keyboard frame handoff — 2026-11-01

A real-app RawInput regression confirms that selecting a reference and pressing
Right in the same batch selects the row without expanding it. The pre-frame
keyboard capture runs before UI painting applies the click action and its focus
change. Reversing those events must instead move the source caret before selecting
the reference; routing every key to the final owner would lose that distinction.

The raw hook now stages mixed pointer-button/keyboard batches at their first
input-kind transition. The suffix remains in one ephemeral event vector and is
prepended to new input on the next automatically requested repaint. Thus earlier
keys execute before a later click, and later keys see the click's registered owner.
This adds one frame per transition without a timer or sleep. The existing source
queue still handles source-only gestures directly: when source owns keyboard input
and every button position is inside its registered response rectangle, staging is
bypassed. `SourceViewport::focus_id` is now crate-visible to reuse that identity.
Modal/menu input continues through the existing owners. No alternate command or
pointer-action dispatcher, widget hit cache, worker or dependency was added.

Window-focus loss drops the staged suffix; accepted repository replacement clears
it alongside existing reader input. New events follow the earlier suffix. Pointer
movement alone does not create a focus boundary. This contract belongs to the
normal eframe raw-input hook; prepared keyboard-only dispatch remains unchanged.
The source bypass uses the previous registered source rectangle; overlay and wider
mixed-pane interaction acceptance remain open.

The old renderer fails the corrected same-batch regression because no disclosure
opens. The initial label lookup assumed leading whitespace; final tests derive
text from the actual ReferenceHit and reuse one gesture fixture. Both event orders
now pass at 1×/1.5×/2×, with exact caret/selection/disclosure assertions and a zero
repaint delay proving progress is scheduled. Additional app tests verify queued
Right followed by fresh Left, focus-loss retirement and repository retirement.
Existing modal/global/focus and source-gesture tests pass. Workspace passes 558 app
tests (seven ignored), two binary, four font, 13 text and 36 doctests. After sharing
test setup, all nine pointer-related tests and strict Clippy/check/build/rustdoc/
fmt/diff pass.

Architecture adds 133 NLOC including tests and four functions (one stage method,
two tests and a shared test gesture). Stage CCN 11; app render/dispatch complexity,
max CCN, hotspot count and dependency cycles stay unchanged. Sharing the gesture
removes the initially added test clone; the full normalized multiset contains the
same 91 groups as the baseline. The comparator's touched old test clone warning
is pre-existing. Health changes -0.01.

Four native 1×/2× navigation and preview-failure/retry flows pass after removing the
previous selection-to-keyboard pause. Navigation copies the exact 473-byte Unicode
destination and Back restores the original call-site copy. Failure/retry removes
and restores only private fixture bytes and preserves exact accepted source.
Final binary/fixture hashes match and every final child exits zero. Actual native
same-frame grouping is not logged; the RawInput tests prove that ordering directly.

Archive: `~/.local/state/radiology-qa/pointer-key-20261101-ed4f546`.
Broader overlays, mixed owners, source/performance/language/semantic/PR/cache/UI and
final Swift/standalone/reuse acceptance remain open. Accessibility and actual
sibling adoption stay deferred.

## Layer-aware source ownership — 2026-11-01

The source-only bypass now requires both the source response rectangle and its
interaction layer. A foreground control can lie inside the source rectangle
without owning source gestures. The check reuses the same `layer_id_at` policy
already used by source gesture capture; absent layer information retains the
existing fallback. Event staging, modal/menu policy and the source gesture queue
otherwise remain unchanged. No new product function, state, API or dependency.

The regression hosts a real foreground egui TextEdit above the actual app's source
viewport and verifies that the field lies inside its rectangle on a different
layer. Click, Ctrl+A and replacement text arrive in one RawInput batch. The old
rectangle-only path leaves originalreplacement because source consumes Select
All. The layer-aware path yields replacement at 1×/1.5×/2×; source caret and accepted
buffer remain unchanged and the staged queue drains. The shared real-app frame
helper now accepts a test-only overlay callback. This is an embedding/interaction
regression, not a new floating editor in the standalone product.

Initial trials omitted egui's logical command modifier and failed with either
guard. The corrected negative uses both physical Ctrl and logical command and
still fails with the old guard. Its script restores final product code in a
finally block. Workspace passes 559 app tests (seven ignored), two binary, four
font, 13 text and 36 doctests. Strict Clippy/check/build/rustdoc/fmt/diff pass.
Architecture +67 NLOC including tests, two functions (test/frame extension), stage
CCN 11→12 for necessary layer ownership. No new hotspot/max-CCN/cycle; all 91 clone
groups unchanged after full normalized audit. The touched old test clone warning
is pre-existing; health is effectively unchanged.

Four native 1×/2× reader/reference navigation and preview failure/restore/retry
flows retain exact source/destination/Back copies, unchanged binary/fixture hashes
and zero exits. They validate the existing standalone flows; floating-field layer
ownership is proved by the real-app frame regression. An initial 2× native copy
read the sentinel before Copy completed; excluded. The final 2× controller polls
for the expected payload without replaying keys.

Archive: `~/.local/state/radiology-qa/layer-handoff-20261101-d28aa81`. Accepted
native roots are `layer-handoff-1.0`, `layer-handoff-fresh-2.0` and
`layer-handoff-failure-{1.0,2.0}-indexed`. Source-only raw input interleaving and
broader real hover/overlay/mixed-owner corpus checks remain audit targets, along
with reader/language/index/PR/cache/UI and final Swift/standalone/reuse acceptance.
Full accessibility and actual sibling adoption remain deferred.
