# Reader context-menu keyboard ownership

## Repaired behavior

Native acceptance at 0566596 showed Down moving the source caret behind an open
menu. The four reader menu owners now use one private component: Files, source,
bookmarks and reference rows. It renders ordinary egui buttons in menu frames,
with one action model for keyboard and pointer input. This replaces their separate
egui 0.27 pointer-only menu closures; toolbar menus remain separate.

Up/Down and Tab/Shift+Tab select enabled entries with wraparound; Home/End select
boundaries. Right opens a submenu, Left returns to its parent, and Enter/Space
activate the selected action. Right does not activate an ordinary leaf. Escape
closes the hierarchy without acting. Disabled entries and separators are skipped.
A checkmark shows the current radio choice independently of keyboard selection.
Submenus fit within the screen and open leftward when the right edge is crowded.

Source Copy Selection retains Ctrl+C/platform Copy behavior when enabled. Other
reader shortcuts pass through the existing dispatcher, which dismisses the menu
before acting. Ordinary text/edit events are owned by the popup instead of
changing source, lists or filter fields behind it. Existing reader focus remains
intact during keyboard traversal; pointer controls do not take focus. Closing
restores a still-live previous control if focus changed. Outside presses dismiss
and remain available to the clicked control.

## Ownership and architecture

State is local to the egui context. It stores the response owner, semantic scope,
open submenu path, selected index, geometry and ordered pending keys. Menu actions
are built lazily from the current owner model and are never cached in that state.
Source scope includes document/revision/content identity; bookmark scope uses its
UUID; references use the source site; Files uses its path. Repository publication
retires the menu. Missing/replaced owners and changed semantic scopes cannot
retarget an old popup to another source. Changed enabled states are checked again
at activation.

The app's raw-input hook reserves menu events before framework focus handling and
reader/list handlers. SourceViewport also guards its own keyboard path, supporting
prepared-frame embedding. Hosts driving the whole reader should forward the raw
hook before egui processes the frame. No dependencies, sibling runtime or public
provider framework were added. Existing command, clipboard, reference, bookmark,
critical-mark and navigation handlers still execute the selected actions.

A native 2× pass found an additional race: Down could arrive in the same frame as
the right-click and reach the source before a popup owner existed. A short-lived
opening reservation now captures keys after the secondary press, preserves their
order and transfers them to the claiming menu on release. An unclaimed release
expires; it cannot consume future reader keys indefinitely. Closing also retires
that reservation. Temporary tracing used to identify the lost key was removed.

## Evidence — 2026-10-31

The source regression fails when its popup guard is removed: line 2 becomes line
3. Fixed behavior leaves caret/selection unchanged, copies the context line on
Enter and returns the next Down to the reader after close. A separate opening-batch
regression fails before the reservation because Down/Enter remain in raw input;
now right-click, Down and Enter together copy line 2 without reader navigation.

Six actual-frame component tests cover disabled skipping, root/child wrap,
Home/End, Tab/Shift+Tab, Left/Right, Enter/Space/Escape, pointer action, Copy,
focus return, outside dismissal, current-model updates, owner retirement, semantic
scope replacement and global-shortcut availability. The earlier Escape/reopen
regression remains active. All four owners share the tested mechanism.

The 1× native pass checks source Copy Line and subsequent reader navigation,
Files Copy Relative Path, Critical Lens persistence, bookmark Move to Shortcut
reordering by identity, reference menu navigation and a global Go to Line dialog.
It precedes the final scope, indicator and opening-race refinements. The final
clean binary is checked at narrower 1024×768 logical resolution with 2× rendering:
rapid right-click/Down/Enter copies exactly `    alpha();`, batched End/Up/Right
opens the intended symbol submenu, keyboard and pointer radio choices persist,
and light/dark panes show distinct checkmarks and selection with leftward fit.
Earlier failed/diagnostic 2× attempts are excluded from final acceptance.

All accepted fixtures retain their original hashes and app/Weston/Niri exit zero.
Durable captures, copy bytes, policy/bookmark records, binary/fixture hashes,
drivers, regression logs and SHA256SUMS:
`~/.local/state/radiology-qa/menu-keyboard-20261031-0566596`.
Compositor helpers come from the sibling QA tooling only; the app stays standalone.

The final workspace passes 453 app tests (seven ignored), two resource-checker
binary tests, four font tests, eight text tests and 33 doctests, strict Clippy,
build, fmt and diff checks. Architecture adds no clone group, dependency cycle,
hotspot or maximum-CCN increase. The new rendering hotspot and duplicated gesture
setup detected during implementation were simplified. Its remaining 12-line
modal/reference-test clone flag is unchanged baseline code (reference lines
711–722 before the menu wiring edit).

Toolbar-menu traversal/focus and the broader reader/corpus/PR/cache/final Swift
acceptance queue remain open. This is ordinary keyboard work; full accessibility
and actual little-screen-clip adoption remain deferred by the user.
