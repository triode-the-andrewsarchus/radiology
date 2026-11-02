# Shared modal input before frame processing — 2026-11-02

## Accepted host contract

The four failures are resolved through a shared public `ui::modals::ModalInput`
boundary. Hosts own one instance for the dialog lifetime, call `capture_input`
before egui begins a frame, then render through `show`. The standalone app now
uses this same implementation, extracted from its previous ModalFocus buffering.
The class reuses keyboard classification/consumption, initial gesture retirement,
first-measurement deferral, ordered replay through each Tab, and closing input/
pointer/scroll cleanup. The app retains backdrop layering and focus-return policy.
No dependency, worker, late focus repair or second event-routing implementation.

Direct file/symbol host tests now use this public boundary before Context::run,
rather than handing raw arrows to egui and capturing them only inside picker.show.
Their original destination/query expectations remain unchanged. A test that reused
a closed symbol dialog now measures the new lifetime and clicks its current row;
opening gestures are intentionally retired just as in the standalone app.
A shared test frame helper avoids duplicating host setup across both picker suites.
The new raw-host regression checks opening text/Return and trailing-text retirement,
Tab→primary, Tab→Cancel, Shift+Tab→Cancel, empty background editing, raw-key capture
and closing-owner reset. The compiled public rustdoc example and rendered API docs
cover the integration sequence. Existing picker show signatures remain unchanged.

Verification: all 16 file/symbol tests and the new direct-host matrix pass. Full
workspace now passes 631 app tests (seven ignored, zero filtered), two binary,
13 font, 19 text and 41 doctests. Strict Clippy/check/build/rustdoc/fmt/diff pass;
copied standalone resource verification passes. Native standalone 1×/2× results
and final artifact identity are recorded in PROGRESS.md and the evidence archive.
The original unsupported bare raw-host sequence below remains useful for diagnosis:
rendering alone cannot retroactively cancel native focus navigation.

Architecture baseline `before-shared-modal-input` at e3cfeed: 87,243 NLOC,
3,670 functions, average CCN 3.34, max 132 and 109 clones. Final delta +129 NLOC/
five functions including test support and regression, one shared UI file; maximum/
average unchanged, no new hotspot/cycle, health +0.00. Default REGRESSION flags
three old file-picker test clones at shifted lines; normalized file-pair/whitespace-
stripped-fragment audit matches all 109 groups unchanged. The raw capture/replay/
cleanup code is moved out of the app, which now delegates, rather than copied.

Evidence: `~/.local/state/radiology-qa/modal-input-20261102-e3cfeed`.
Broader host/owner/Unicode transitions and final Swift parity acceptance remain open.

## Reproduction

Four existing unit tests fail on both the footer worktree and unmodified b5b8385:

- file_picker::mixed_text_navigation_and_return_preserve_order
- file_picker::repeated_keys_reveal_beyond_twelve_and_enter_discards_its_text_suffix
- symbol_picker::document_picker_virtualizes_rows_and_reveals_keyboard_selection
- symbol_picker::workspace_limit_metadata_and_mixed_repeated_keys_preserve_order

Run `cargo test --lib ui::modals:: -- --test-threads=2` in either tree. The clean
parent reproduction is ~/.cache/radiology-footer-baseline; source is unchanged.
Its test command used CARGO_TARGET_DIR pointing at the main target directory to
reuse dependencies. This can rebuild local crates when switching manifests.
Parent and current modal-only runs each pass 12/fail four. Wrong outcomes include
Close/None instead of Open and symbol 990 instead of 999. Timing can affect whether
the initial query has fully settled before tests supply their raw key batch; do
not infer a source-footer dependency from a previously passing whole-suite run.

## Debugger evidence and rejected trial

The debugging-code skill was consulted. The local dap wrapper rejected the binary
plus test arguments, and lldb-dap was not found. Existing GDB batch breakpoints
provided the runtime evidence; sessions terminated without installing tools.

At PickerInput::route (picker.rs:69), first ArrowDown has query focus, the second
ArrowDown and following Enter do not, with the same query ID. The Enter branch
then returns the key to egui controls. At egui memory.rs:445, Focus::end_frame has
already queued FocusDirection::Down and moves focus to another widget. During the
trial, the query's filter already retains vertical arrows at that point: setting
it in query rendering does not erase native navigation queued at begin_frame.

A trial adding commands::retain_navigation_keys after TextEdit was tested and
failed the same four cases; it was reverted. Do not repeat that alone as a fix.
No picker correction is included in the footer commit; the shared boundary above
was implemented afterward.

## Boundary requirements motivating the implementation

The investigation required auditing and reusing the app raw_input_hook/modal
capture boundary before introducing another mechanism. The standalone app already captures modal
input before egui processing; the old direct picker hosts/tests supplied raw keys
through Context::run and then captured inside show. The shared contract preserves opening-frame text, repeated arrows, asynchronous query
publication, Tab/Shift+Tab, explicit primary/Cancel Return and Escape. Do not make
the tests green by changing expected navigation or allowing unrelated button focus.

Evidence is archived with source-footer-20261102-b5b8385: baseline/current test logs,
DAP failure and GDB route/focus/transfer logs. The shared input boundary resolves
this keyboard/focus follow-up; full accessibility retains its deferred scope.

## Modal IME event ownership

On 2026-11-02, the remaining input classifier gap was reproduced: CompositionStart,
CompositionUpdate and CompositionEnd bypassed keyboard_event, so a direct host's
previously focused background field received Japanese composition behind a modal.
The standalone app also selected a.rs instead of the composed 日本.rs query. Both
regressions fail with the previous classifier and pass after the fix.

The existing classifier now includes those three egui 0.27 event variants. Shared
modal capture, ordered picker replay, pointer/key handoff and closing cleanup use
that classifier; there is no new IME state machine, event queue or dependency.
Composition is delivered to the existing TextEdit implementation. Public modal
capture documentation now explicitly includes IME text.

The direct Context::run regression keeps a focused background TextEdit, opens the
file picker, composes 日本 and submits the exact Japanese path. It covers complete
same-frame composition, commit-only delivery, pre-edit updates across frames, and
cancel/empty-update followed by a new composition. Composition text following Return
is retired; background bytes and committed query remain exact. The actual-app
regression opens the picker from a focused sidebar search, composes/submits the
Japanese filename and verifies the loaded bytes and unchanged sidebar query.

Workspace: 633 app tests pass, seven ignored/zero filtered; two binary, 13 font,
19 text and 41 doctests pass. Strict Clippy/check/build/rustdoc/fmt/diff, rendered
public capture docs and copied standalone resources pass. These are actual egui
composition-event tests; no native IME engine, OS candidate panel or platform
composition-protocol acceptance is claimed. Broader input/Unicode/platform and
final Swift acceptance remain open.

Architecture baseline before-modal-ime-input at 4a5cb7c: overall NEUTRAL, +134 NLOC/
two regression functions, same 517 files, average CCN 3.34/max 132 and 109 clone
groups. No new hotspot/cycle, health -0.01. Runtime change is confined to the shared
classification match; test code exercises host isolation and Unicode destinations.
Evidence: ~/.local/state/radiology-qa/modal-ime-20261102-4a5cb7c.
