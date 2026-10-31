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
