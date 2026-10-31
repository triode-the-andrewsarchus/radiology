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
The native protocol/menu acceptance below now passes. Neither backend fixtures
nor the controlled receiver claim a real desktop file-manager window.

## Native protocol/menu acceptance — 2026-10-30

Backend `7cfdd12` was built and exercised in a private 1280×900 Weston/Niri session.
An independently owned private session bus and Python/Gio FileManager1 receiver
isolate the desktop handoff from the user's normal desktop. The app runs the real
gdbus binary; no fake client parses or forwards its arguments. The service receives
actual typed ShowItems calls, proving that the serialized URI array is accepted by
the real GVariant parser and D-Bus signature. This is controlled-receiver evidence,
not a claim that a Nautilus/Dolphin window was inspected.

Native checks pass:

- Revealing unselected `b.rs` selects that exact URI while `a.rs` remains the reader.
- Folder rows expose both context actions; revealing the folder does not expand it.
- A nested filename containing spaces, single/double quotes, Unicode, semicolon and
  `$(touch sentinel)` reaches the receiver exactly, with no shell-created file.
- A rejected selection reports the plain-language notice and full service failure
  in Error Details, retaining the source and selection.
- An After-only Cargo.toml reveals the private After-tree URI from both After and
  Before readers. The file does not exist in the attached checkout.
- During a slow selection, a file click still navigates. The five-second client
  timeout reports Details without disrupting that new reader. The receiver
  completes its delayed reply later and remains alive.
- A timed request is retired by Exit PR. The checkout capture occurs 1.164 seconds
  after dispatch, before the five-second client deadline and seven-second server
  completion. The delayed reply introduces no error into the checkout reader.

Nine calls are recorded, including the controlled rejection and slow-request
experiments. The final timed retirement is proven specifically by
`retirement-order.json` and the `retire-case-*` captures listed in the manifest;
other exploratory retirement captures are not acceptance evidence. All viewed
checkout regular-file bytes, HEAD and dirty status compare unchanged. The app,
receiver, private bus and both compositors exit zero. The sibling capture helper
is QA only and creates no sibling app dependency.

Evidence: `/tmp/radiology-reveal-native/acceptance.json`, `manifest.json`,
`reveal-requests.jsonl`, source before/after proofs and exit statuses. The manifest
lists accepted captures explicitly and hashes them, the logs/proofs, receiver,
fixture chain, driver, capture helper and tested binary. No production change or
fresh automated test rerun was needed for this evidence increment. Broader desktop
implementations, UI/accessibility/scaling/font and sibling reuse acceptance remain
part of the full parity audit.
