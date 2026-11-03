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

## Cancellation finding and repair — 2026-11-02

The candidate-placement checkpoint initially found cancellation was **not accepted**. After two Escapes clear Anthy's conversion and
reading, the engine emits a Wayland done with no preedit/commit. winit translates
this to `Ime::Preedit("", None)`, but egui-winit0.27.2 ignores that event. Cancelled
marked text stays in the editor and its composition state can remain active.
Existing synthetic streams include explicit clear/end events and therefore could
not prove this real cancellation path.

The backend is now pinned locally with a narrow translation repair, original
archive/file/license checksums and a reconstructable patch. See
[vendor provenance](../vendor/egui-winit/PROVENANCE.md). An empty Preedit or Disabled
while composing emits clear-then-end; nonempty preedit is handled even with a hidden
cursor. Commit inserts the final text exactly once after winit's preceding clear.
The existing translation is extracted into one private function to exercise this
boundary without a native window. No modal-specific workaround or public API is added.

Both new backend regressions fail against the extracted original logic and pass
with the repair. They cover clear-without-commit, hidden cursors, following commits,
duplicate clears, Disabled, visible empty preedit and restarted composition.
The archive reconstructs byte-for-byte with `ime.patch`; unchanged inputs/licenses
match their hashes. Cargo.lock changes only the source/checksum of this existing
package, with no version or transitive dependency changes.

Final native1×/2× workflows preserve the exact16-byte café/emoji/newline prefix after
Escape clears conversion and preedit; Backspace clear retains the exact25-byte
committed draft. The navigation picker separately preserves its `boot` prefix after
real engine cancellation. Candidate Up/Return commits 日本語 and keeps the picker open;
a subsequent ordinary Return opens 日本語.C. The import editor receives no spurious
newline or submission. Active candidate placement and preedit remain correct while
resizing the actual native window in **both** import and navigation controls at both
scales. Relevant candidate/cancel/committed/destination captures were inspected.
All source/executable hashes remain unchanged; app/Weston/Niri exit zero, engine
retires intentionally by SIGTERM. Private packages/configuration remain QA inputs.

Final locked workspace passes758 app tests/nine ignored, two binary/13 font/21 text
tests and59 doctests. The two backend tests run explicitly because the pinned
package is outside the workspace. Strict workspace and backend Clippy, build,
resource smoke, workspace/vendor formatting and diff checks pass.

Architecture distinguishes importing the already-used upstream package from the
local repair. The full baseline adds1844 NLOC/57 functions across three source files;
1758 NLOC/54 functions are unchanged upstream code. Three imported upstream hotspots
(CCN16/21/18) explain the full comparison's REGRESSION. No new design duplicates an
existing backend: Cargo substitutes this one package for the registry copy.
Against the unmodified import, the patch is NEUTRAL: +86 NLOC/three functions
(one translator/two tests), no new above15 function. Average/max CCN3.38/132,
all125 clone instances/1335 duplicated lines, one existing cycle and eight structural
findings are unchanged. Baselines and full clone Counters are retained; untouched
Swift Python rank changes are measurement variation.

Evidence: `~/.local/state/radiology-qa/native-ime-clear-20261102-77cc03a` contains final
drivers, real key/protocol traces, exact clipboard payloads, native captures,
hashes/statuses, package pins, failing/passing tests and architecture records.
This accepts the actual-engine gate for the app's Linux Wayland frontend using
Anthy. It does not claim other engines/platform backends, restoration of a selection
replaced before cancelling, full accessibility or the remaining integrated corpus,
PR, stress and final standalone/reuse audits.
