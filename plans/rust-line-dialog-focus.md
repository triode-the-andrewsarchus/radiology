# Go to Line and scoped modal keyboard focus

## Swift contract and confirmed gaps

Swift AppModel.showGoToLine fills lineInput with currentLine. GoToLineSheet focuses
the numeric field once, displays the source line range, and offers Cancel and Go.
Submission routes through SourceBuffer.byteOffset(line:), which clamps to the
source bounds. Rust previously opened an empty field, requested focus every frame,
showed an always-enabled Jump action, and duplicated the prompt's footer controls.
The actual-app prefill regression failed before the repair: empty rather than 2.

The first focus test also exposed a shared modal boundary defect. take_modal_input
removes Tab before egui's begin-frame focus traversal. Replaying it only while
rendering a modal was too late to traverse controls. Fixing just the field's repeated
focus request did not repair app-level Tab. Component-only PR prompt tests had
missed this because their RawInput reached egui directly.

Native old controls at 1× and 2× confirm the empty prompt and ignored Tabs: after
editing line 3, Tab/Tab/Return navigates there instead of canceling at line 2.
The earlier intermittent empty Go to Line capture motivated this audit, but its
specific cause was not reproduced. It is not used as proof of this repair.

## Implementation and reuse

GoToLineModal::new(current_line) initializes a per-instance prompt. Both the command
and source-footer route now use the same command handler. The field receives focus
and selects its initial number through egui's existing text-edit state; initialization
finishes only on a visible pass. Invalid or overflowing signed integers keep the
prompt open, disable Go and show concise guidance. Invalid Return retains field
focus. Signed 64-bit integers clamp to 1..line_count, including zero/negatives;
valid Return or Go resolves the existing navigation transaction. Default remains
available for an initially empty standalone prompt. Construct instances through
new or Default; focus state is private to the dialog.

The existing picker footer now supplies consistent range/status metadata and
right-aligned Cancel/Go or Cancel/Open controls to line, PR, file and symbol prompts.
No new fonts, type scale, palette or layout framework is introduced. Requested
macOS/SwiftUI/type skills remain unavailable in this environment as recorded in
the original review; the local Swift source and existing shared UI are the reference.

A shared footer focus function cycles among the query, enabled primary action and
Cancel, skipping disabled primary actions and wrapping in both directions. It uses
this frame's actual response IDs and a fixed array; there is no cached widget
identity or extra owner registry. RawInput hosts keep egui's native Tab processing;
app-replayed Tabs use the explicit cycle because that native pass has already run.

ModalFocus replays input only through the next Tab boundary, repainting for the
remaining batch so later text or Return follows the new owner. Closing the modal
retires its remaining keys and restores the prior owner. PickerInput routes Return
to the focused footer button when the query no longer owns it, instead of committing
a highlighted result behind Cancel. Existing worker/result publication and query
navigation stay in their existing path. No dependency, worker pool, Cargo setting,
sibling checkout dependency or second application navigation mechanism is added.

## Verification

Three new real-app regressions cover all four modal owners, forward/backward focus
cycles, disabled primary skipping, batch Shift+Tab/Return and retirement of later
source keys, current-line prefill, exact retained Unicode/CRLF selection, signed
bounds/overflow/invalid/empty input, disabled-button pointer clicks and correction,
opening-frame typed/pasted submission at 1×/1.5×/2×, exact destination and Back.
The existing modal-prefix ownership test now enters invalid text before Return:
with a valid prefill, Return intentionally accepts the current line. A shared
scaled-reader fixture removes repeated app/context setup without changing coverage.

Workspace: 619 app tests pass, seven ignored; two binary, 13 font, 19 text and
40 doctests pass. Doctests use four test threads to avoid the previously observed
/tmp linking peak. Strict Clippy/check/rustdoc/build/fmt/diff pass. Rendered public
constructor docs inspected; a copied standalone executable verifies all resources.

Native v1 old/new runs at 1×/2× use the same five-line Unicode/CRLF fixture and
private Wayland/clipboard targets. Old controls reproduce both defects, with two
positive source copies and two verified empty-field checks each. New runs pass
14 exact fresh clipboard checks each: selected prefill/current line, Cancel versus
submission, default Return, invalid retention/correction, negative/high clamping,
replacement typing, file/symbol/PR Shift+Tab cancellation, source selection/focus
restoration and light appearance. Captures inspected, fixture/binary hashes stay
unchanged, and every final app/compositor process exits zero.

Archive: ~/.local/state/radiology-qa/line-dialog-20261102-12e1269.

## Architecture review and remaining audit

Against before-line-dialog-parity: +280 NLOC/seven functions including three tests
and their shared fixture; no new maximum-CCN hotspot or dependency cycle, health
-0.02. The default comparison flags four old clones touching modified files as
REGRESSION. A complete normalized fragment/file-pair comparison verifies all
109 clone groups unchanged, zero new or removed. Two initial new test clone groups
were removed by consolidating overlapping tests and sharing actual fixture setup.
The focus primitive lives with existing picker query/footer/input components and
is consumed by all four dialogs; no parallel control family was introduced.
Measured production CCN stays bounded: line show 7→9, PR show 11→9, shared
footer 3 unchanged, Return routing 5 and scoped Tab handling 9.

This closes current-line line prompting and the confirmed scoped Tab/Return gaps.
Other modal pointer/focus transitions, broader inherited bindings, Unicode/UI text,
typography/spacing, language/indexer/PR corpus coverage and the requirement-by-
requirement final Swift audit remain open. Full accessibility and actual sibling
adoption remain deferred by user scope.
