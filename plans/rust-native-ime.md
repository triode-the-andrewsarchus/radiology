# Native IME acceptance

## Candidate placement — 2026-11-02

The standalone app uses egui/eframe0.27.2 and winit0.29.15 on Wayland.
A private Weston headless desktop hosts Niri, Fcitx5 5.1.23 and Anthy5.1.11
with Anthy9100h dictionaries. Packages are extracted under the QA cache; a
Bubblewrap mount namespace supplies their compiled dictionary paths and private
configuration. No system packages or desktop input configuration are changed.
A private D-Bus session and XDG directories contain the test services.

`wtype` bypasses Niri's input-method keyboard grab, so Latin text entered that way
is **not** engine acceptance. The retained QA Weston module sends key events on
the headless compositor's keyboard to its nested Niri client. Fcitx's key trace
and the app's actual `zwp_text_input_v3` trace prove the complete engine path.
The module and input FIFO exist only in that private desktop; host input is unused.

The real test exposed a candidate panel anchored at the import editor's bottom.
The pinned egui-winit frontend uses `IMEOutput.rect` rather than `cursor_rect`.
After rendering all app owners, Radiology now supplies the existing caret rectangle
as the native area. There is one normalization point, no added state or dependency.
Future eframe hosts embedding these widgets need the same boundary correction
until their integration uses the caret rectangle itself.

At1×/2×, real keys spell `nihongo`, yielding incremental Japanese preedit; two
Spaces show the candidate list, Up selects 日本語 and Return commits it. The import
editor remains open and copies exactly nine UTF-8 bytes without an extra newline.
Go to File repeats exact commit/copy; its candidate panel follows the caret when
the native app becomes an820×650 floating window. Captures inspect both scales.
Source hashes remain unchanged; app/Weston/Niri exit zero, engine retires by SIGTERM.

The new actual-app-frame regression fails before the fix and passes after. It
covers single/multiline owners, preedit/commit, resize,1×/2× and disabled IME after
closing. Full locked workspace:758 app tests pass, nine ignored; two binary,13 font,
21 text and59 doctests. Strict Clippy/build, formatting and diff checks pass.

Architecture against `before-native-ime-cursor`: +60 NLOC, one test function,
average/max CCN unchanged3.37/132;125 clone instances/1335 duplicated lines,
one existing cycle and eight structural findings unchanged. The comparison reports
REGRESSION because existing `render` gains the necessary focused-IME conditional
(CCN36→37). This is one boundary guard, with no new abstraction or duplicated
mechanism. Its duplicate warning is an existing shifted test fragment: the full
normalized fragment/file-pair Counter is identical across all125 instances.
The untouched Swift Python chokepoint rank change is measurement variation.

Evidence: `~/.local/state/radiology-qa/native-ime-cursor-20261102-6200866`, including
package URLs/checksums, QA driver/module, real protocol/key traces, captures,
clipboard bytes, source/executable hashes, exit statuses and architecture records.
The initial1× probe and later native checks are distinct; only after-cases use the fix.

## Open real-engine findings

Cancellation is **not accepted**. After two Escapes clear Anthy's conversion and
reading, the engine emits a Wayland done with no preedit/commit. winit translates
this to `Ime::Preedit("", None)`, but egui-winit0.27.2 ignores that event. Cancelled
marked text stays in the editor and its composition state can remain active.
The next repair belongs at the backend translation boundary, not in individual
modal shortcuts. Existing synthetic streams include explicit clear/end events and
therefore could not prove this real cancellation path. Re-run native cancel,
commit/Return and resize ownership at both scales after repairing it.
