# Open repositories through native file and folder drops

## Contract — 2026-11-02

Swift `RootView.dropDestination` takes the first URL, opens a directory directly,
and opens a file's parent directory. Rust now follows that contract through the
existing asynchronous repository transaction. The welcome view advertises folder
drops. The normal default-document selection applies; dropping a file does not
introduce a separate direct-file navigation policy.

`RadiologyApp::open_dropped_path` is available to independent hosts. Metadata
validation and discovery run on the repository worker, not the GUI thread.
Missing paths report the existing repository error while retaining the accepted
reader/session. Cancel and replacement use existing job retirement. Successful
publication uses the same complete repository reset, source selection, bookmark
scope and background index/search ownership as other repository opens. egui raw
drops are consumed once and only the first item is considered; byte-only items
are ignored rather than silently choosing a later path.

## Native Wayland delivery and reuse

The linked winit 0.29 Wayland implementation does not forward file-drop events.
An egui-only handler would therefore leave the standalone Linux feature broken.
`desktop::file_drop` supplies the missing native receiver on eframe's existing
Wayland display. It owns an independent protocol queue, accepts `text/uri-list`
with Copy only, and publishes the latest completed result. It does not copy or
move dropped files. URI parsing accepts the first local file URL, preserving
percent-encoded original path bytes; remote URLs and malformed payloads do not
open roots.

The receiver uses one dedicated event thread so waiting for desktop events does
not occupy an analysis worker or add idle GUI repaints. Nonblocking offer reads
retain at most 64 KiB and have a three-second deadline. Protocol I/O handles
WouldBlock/Interrupted and writable readiness. The native queue synchronizes its
read guard with winit through libwayland. Offers, seats and data devices are
released; app exit stops and joins the receiver before eframe closes its display.
Unsafe boundaries are limited to the borrowed display and the initialized pollfd.
The receiver does not disconnect the borrowed display.

The Linux-specific dependencies were already in Cargo.lock through the windowing
stack; no package versions were changed. `from_session` remains idle and has no
native receiver: embedding hosts can supply drops through the public method and
own their own desktop integration. The native adapter is confined to the standalone
frontend. There is no sibling checkout or workbench dependency.

Before adding the native mechanism, existing Radiology discovery/process helpers
and little-screen-clip's `file_manager/native_drag` were inspected. The sibling
uses the same foreign-display/independent-queue and URI/copy protocol approach,
but combines bidirectional dragging, file transfers, source identities and
workbench jobs. Importing that subsystem would contradict the standalone boundary.
This focused receiver keeps repository opening in the existing model and leaves
future host adoption at the explicit host input boundary.

## Verification

Two app cases exercise actual egui dropped-file frames and direct host requests:
folder and file-parent selection, first-item semantics, one-time consumption,
Unicode/spaces/semicolon paths, default README selection, cleared history,
persisted accepted root, missing-path retention, Cancel and newer-root retirement.
The existing repository transaction suite remains green. Removing only the frame
hook makes the new app regression fail with “native dropped-file input was
ignored”; the hook is restored before final checks.

Two protocol cases check URI-list comments/CRLF, escaped local paths, localhost,
first-invalid/remote rejection, malformed UTF-8, incremental nonblocking reads,
EOF, byte bounds and deadline failure. Workspace passes 578 app tests (seven
ignored), two binary, four font, 13 text and 37 doctests. Strict Clippy, all-target
check, binary build, rustdoc with warnings denied, formatting and diff checks pass.
A copied standalone executable passes resource verification outside the checkout.

Two final native v4 runs pass at 1×/2× on private Weston/Niri. A frozen copy of the
sibling's built file manager is the actual drag source. Each run drops a directory
from the welcome state, then a file into an already open reader and verifies its
parent root/source. Reader protocol logs show two URI receives and two finishes
per run. Clipboard bytes match the accepted source exactly after Go to Line;
light/dark captures are inspected. Fixture and Radiology binary hashes stay
unchanged and every child exits zero, with no receiver failure/protocol panic.
The source binary is identified by hash and used only for QA, not by the product.

Initial QA trials are excluded: the shared workbench-only window matcher did not
recognize the file-manager app ID, and OCR included the source caret as a leading
bracket. Final matching uses owned PIDs and normalized OCR. v3 verified an
intermediate build; only v4 is final accepted evidence. Initial unused-result and
Clippy unit-expression warnings were corrected before final gates.

Architecture baseline `before-native-repository-drop` at `e0f9c63`: NEUTRAL,
+567 NLOC including tests, +22 functions and one code file. No new hotspot, maximum
CCN, cycle or clone group; native event/IO functions have maximum CCN 10. All 91
clone groups are unchanged by full normalized multiset audit. Health +0.13.
Requested SwiftUI/macOS/typography skills remain absent; existing UI type roles
and repository status presentation are reused.

Archive: `~/.local/state/radiology-qa/repository-drop-20261102-e0f9c63`.
`accepted.json` selects the two native v4 runs; actions, captures, clipboard,
protocol/exits/hash evidence, controller, fixtures, workspace/gate/negative logs,
standalone report, architecture/clone audit and staged source diff are retained.
Broader grammar/corpus, reader/key/UI, indexing/PR/cache and final Swift parity
acceptance remain open. Full accessibility and actual sibling adoption remain
deferred per user scope.
