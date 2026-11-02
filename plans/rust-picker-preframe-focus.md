# Picker focus before frame processing — open investigation, 2026-11-02

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
No picker correction is included in the footer commit.

## Next work

Audit and reuse the existing app raw_input_hook/modal capture boundary before
introducing another input mechanism. The standalone app already captures modal
input before egui processing; direct picker hosts/tests currently supply raw keys
through Context::run and then capture inside show. Establish the correct reusable
host contract and preserve opening-frame text, repeated arrows, asynchronous query
publication, Tab/Shift+Tab, explicit primary/Cancel Return and Escape. Do not make
the tests green by changing expected navigation or allowing unrelated button focus.

Evidence is archived with source-footer-20261102-b5b8385: baseline/current test logs,
DAP failure and GDB route/focus/transfer logs. This is a required keyboard/focus
follow-up, separate from deferred full accessibility.
