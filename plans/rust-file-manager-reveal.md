# Files-row desktop reveal

## Contract and reference

Swift's FileTreeLabel offers Reveal in Finder for files and directories. Rust now
adds **Reveal in File Manager** to both Files-row context menus, retaining Copy
Relative Path for files and making it available for folders too. The right-clicked
row owns the action; it does not open a source, alter selection/history or toggle
folder expansion. Existing egui menu controls and type styles are reused.

The checked-in design guidance was reviewed before and after the UI decision.
The requested swiftui-pro/macos-design/typography-designer skills remain absent
from the configured local skill/plugin roots. No new typography scale, palette or
custom control was introduced.

The desktop handoff uses the primary
[freedesktop FileManager1 interface](https://wiki.freedesktop.org/www/Specifications/file-manager-interface/):
ShowItems selects supplied file or directory URIs in their parent folders. Its
session-bus name and object path are the standard org.freedesktop.FileManager1
and /org/freedesktop/FileManager1. The URI-array argument is JSON-quoted ASCII,
which uses valid GVariant string-array syntax. No shell evaluates any filename.
The existing url crate handles file URI encoding, including Unicode/control bytes.

## Ownership and integration

A small egui-independent desktop module plans a literal ProcessCommand and sends
it using the existing bounded subprocess runner. The app uses the existing
LatestRead worker/publication mechanism, polls without blocking a frame and places
errors in the existing recoverable notice with complete Details/Copy. The client
has a five-second budget, cancellation and reaping. Only the short-lived gdbus
RPC client belongs to that process group; an independently activated file manager
owns its own lifetime. No content-opening fallback is substituted for selection.

This requires gdbus and a desktop service implementing FileManager1. Missing clients,
unsupported/unavailable services and rejected requests report visible errors.
No new Rust dependency, daemon, sibling checkout dependency or runtime is added.
The sibling app's existing external-launch lifecycle was inspected: validation is
off-thread and accepted external applications must survive reader retirement. Its
workbench-specific launcher is not imported into this standalone reader. This
module can later be extracted or adapted alongside the shared process boundary.

Checkout Files reveal the attached directory entry; PR Files reveal the matching
private After tree, consistent with their inventory even when Before is displayed.
The worker holds the ReviewSnapshot until the RPC completes, protecting its tree.
Repository/review replacement retires late results; accepted desktop requests are
not undone. Ordinary tree-cache retention still governs a private tree after the
handoff, so keeping an external browser open does not pin that tree indefinitely.

Relative paths use the existing SourceLocation validator. Canonical parent paths
must stay within the selected root; the final entry is checked without following
its symlink so selection addresses that entry, including a dangling link. Missing
entries and escaping ancestor links fail before launching a client. This does not
claim descriptor-level protection from unrelated filesystem changes during a
later desktop application's access.

## Evidence and remaining acceptance — 2026-10-30

Three backend tests cover literal URI round trips for file/folder and quote,
Unicode, newline, backslash and command-like names; internal/external ancestor
links; missing/traversing paths; successful client responses, nonzero diagnostics,
missing clients, timeout and cancellation. A reader integration verifies that
errors retain the accepted buffer and repository teardown drops late results.
Workspace gates pass 432 app tests (seven ignored), eight shared tests and 30
doctests, strict Clippy, formatting and build.

Architecture baseline before-file-manager-reveal: no new hotspot/cycle/clone
growth. Four reported clone groups in the touched app-test file are pre-existing;
all eight exact fragments match HEAD byte for byte and none intersects the added
test. Logs: `/tmp/radiology-reveal-{tests,app-tests,workspace,clippy,build}.log`.
The full workspace run preceded the final plain-language error-summary wording;
three focused backend tests, strict Clippy and build pass again afterward.
Actual gdbus-to-service and native file/folder/PR context-menu acceptance remain
open; the backend fixtures are not proof of a desktop file-manager window.
