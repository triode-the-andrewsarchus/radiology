# Standalone keyboard reference

The toolbar's Help > Keyboard Shortcuts and Ctrl+/ open the same bundled
[Linux keyboard reference](keyboard-shortcuts-linux.md), including on the welcome
screen. It covers the actual Rust global bindings, source selection/movement,
bookmarks, Code Paths, reference/caller/definition/outline/trait controls, sidebar
tabs, menu navigation and import-editor Return behavior. Desktop window management
stays with the compositor. The Swift document remains the macOS reference; the
Linux document records intentional modifier differences rather than advertising
unimplemented macOS window commands.

## Presentation and ownership

KeyboardShortcutsModal is a public, reusable presenter alongside the existing
navigation dialogs. Embedded hosts use the same ModalInput capture/replay boundary
and own their backdrop/focus-return policy. ReaderCommand::OpenKeyboardShortcuts
and ActiveModal::KeyboardShortcuts connect the standalone app to that presenter.
No dependency, second modal manager, font scale or color palette was added.

The compiled-in document is rendered with existing heading/body styles and strong
key labels. Paragraphs wrap; explanations are selectable. The bounded centered
sheet keeps Copy Reference and Done outside its scroll area, including at 640×480.
Up/Down, Page Up/Page Down and Home/End scroll. End uses egui's measured cursor
target, with the same immediate scrolling policy as existing lists. An initial
oversized offset produced a blank body in native QA; a regression now verifies
the complete final paragraph is visible and Home restores the introduction after
narrow reflow. Geometry comes from the rendered document, without an invented
maximum offset or separate cached document extent.

Copy Reference copies the exact bundled Markdown. Done initially owns focus;
Tab/Shift+Tab cycles the two footer controls. Pointer or keyboard Done and Escape
use the normal modal close path. Opening, scrolling, composition and closing
batches cannot reach the source or other background controls. Prior source
selection, caret, history, buffer identity and focus survive dismissal.

## Acceptance

Two app tests exercise source-focused keyboard opening, welcome Help-menu opening,
exact reference copy, Tab/Return, boundary scrolling/reflow at both scales and
closing-batch retirement. The shared measured welcome fixture also serves the
existing bookmark-state test. The final workspace passes 755 app tests (nine opt-in
tests ignored), two binary tests, 13 font tests, 21 text tests and 59 doctests.
All 57 focus tests pass, as do strict Clippy, build, rustdoc, formatting, resource
smoke and rendered public API checks.

Native 1×/2× runs verify Ctrl+/, Help-menu arrows/Return, Home/End/Page Down,
exact 6,732-byte reference copy, pointer and keyboard Done, Escape, light/dark
and exact source-selection copying before/after. Corrected bottom and light
captures were visually reviewed. Source/executable hashes stay unchanged; core
processes exit zero. Initial sparse OCR omitted the visible Done button, so its
pointer acceptance uses the observed footer position; the blank-End product bug
was independently fixed and verified. These are separate findings.

Against before-shortcut-help at fe436a8, the change adds 223 NLOC, five functions
and two analyzed files. Average/max CCN remain 3.37/132, the 33 functions above15
remain unchanged, and duplication stays at125 groups/1,335 lines/0.84%. Normalized
clone identities have zero additions/removals; cycles remain one and structural
issues eight. The raw comparator flags an existing seven-line clone in the touched
focus test file. Shared welcome setup removes the initially introduced clone.

Evidence: ~/.local/state/radiology-qa/shortcut-help-20261102-fe436a8.
Broader corpus/PR/stress and actual desktop-portal/IME-engine acceptance remain
part of the master parity work. Full assistive accessibility is deferred.
