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
