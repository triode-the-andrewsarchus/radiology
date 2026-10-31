# Context-menu Escape dismissal

The later [keyboard menu component](rust-context-menu-keyboard.md) supersedes this
Escape-only implementation while preserving the regression and historical native
evidence below.

The [appearance native pass](rust-reader-appearance.md) proved that file context
menus stayed visible after standalone Escape and a 400 ms wait. Egui 0.27 handles
Escape for toolbar menus but its context-menu path does not implement that action.

All four reader context-menu owners—Files, source, bookmarks and references—now
use one shared command helper. It uses egui's public `read_response` and
`context_menu_opened` queries to verify the actual menu owner. The existing
pre-frame input hook reserves only the open menu's unmodified Escape before list
handlers or framework focus handling can consume it. A small context-local owner/
dismiss intent then closes that menu without running its action closure. Closed
or missing owners leave raw input alone. Ordinary click actions and outside-click
closing remain egui-owned. This reuses the ordered event dispatcher and introduces
no dependency, custom popup renderer or application-global focus policy.

The helper also handles Escape in prepared UI input for hosts exercising the menu
component directly. An embedding host must forward Radiology's raw input hook for
the same pre-frame list ownership used throughout the reader. Existing focus and
selection stay in place through Escape; the fix does not implement complete menu
keyboard traversal, every popup transition or general focus restoration.

## Validation — 2026-10-30

The regression test fails against ordinary egui context-menu behavior and passes
with the repair. It opens a real egui popup, protects a focused reader owner from
pre-frame Escape dispatch, verifies closure without invoking contents/actions,
checks focus retention and closed-menu input passthrough, then reopens and invokes
an ordinary action. All four call sites use the helper.

Workspace gates pass 437 app tests (seven ignored), four font tests, eight shared
text tests and 33 doctests, strict Clippy, build and formatting. Logs:
`/tmp/radiology-menu-escape-{before,after,workspace,clippy,build}.log`.
Architecture baseline `before-context-menu-dismissal`: max CCN 100, 25 hotspots,
0.72% duplication, 92 clone groups and no cycles stay unchanged. The sole flagged
12-line test fragment in reference_list/modals already exists in the baseline;
the menu change touches neither copy. The production behavior is shared once.

Native 1× dark acceptance opens and dismisses file, source and bookmark menus with
standalone Escape. The complete file-menu-dismissed PNG is byte-identical to the
reader capture before opening that menu. The bookmark and source remain visible
and intact; a bookmark hover tooltip after dismissal is distinct from the closed
menu. Reopening the file menu and clicking Copy Relative Path copies the exact
Japanese/emoji filename and closes normally. Source fixture hashes stay unchanged;
app and both private compositors exit zero. No native reference-menu check is
claimed; its call-site wiring and the shared component path are covered by code
and tests. Broader keyboard/accessibility and all-state acceptance remain open.

Durable evidence:
`/home/vega/.local/state/radiology-qa/menu-escape-20261030-254336c`.
It includes the driver, binary/fixture hashes, before/acceptance JSON, captures,
exact clipboard bytes, gate logs, zero exit statuses and SHA256SUMS. Working output:
`/tmp/radiology-menu-escape-native-1.0`. An initial immediate clipboard probe ended
when text was unavailable; its `-clipboard-trial` directory is excluded. The
accepted run waits for native publication before reading text clipboard data.
