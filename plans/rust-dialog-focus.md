# Dialog keyboard focus — 2026-11-05

The pending-context checkpoint fixed Go to Line after native reuse exposed premature
initial-focus completion. Inspection found the same ownership mistake in the
shared File/Symbol query, Pull Request prompt and Code Path import editor.
Visibility or issuing a focus request does not prove the editor has acquired
keyboard focus, especially while the native window is inactive or editing is
explicitly disabled.

Each existing dialog now completes its existing initial-focus flag only when its
TextEdit response is enabled and has focus. Egui 0.27.2 makes its invisible window
measurement pass non-interactive through Ui::set_visible(false), which also
sets enabled=false. Response::has_focus additionally requires native input focus.
The explicit visibility condition in Go to Line was therefore redundant and is
removed. There is no shared wrapper, extra state, public API or dependency change.
Once initial focus succeeds, deliberate Tab navigation retains ownership.

## Regression evidence

Three new tests fail against the previous implementation and pass after the fix:

- The shared File/Symbol query does not complete initial focus in inactive frames;
  activation allows exact Unicode query text.
- The PR prompt waits for activation, accepts 42, and keeps Cancel focused after
  two Tabs so Enter closes rather than submits.
- The import editor waits through both inactive-window and active-but-saving
  frames, accepts multiline Unicode authoring after enabling, and does not steal
  Tab focus from Choose Text File when more text arrives.

All use the existing ModalInput owner before Context::run, with a small test frame
helper extracted from the previous Go to Line regression. That regression remains
active. The older direct-render PR test now finishes the invisible measurement
pass before delivering Tab and explicitly asserts acquisition first. The hosted
PR regression covers actual application input delivery. Existing file/symbol
ranking, composition, ordered input and submission tests continue running.

The 23 focused modal tests pass. Workspace verification covers 792 application tests
(11 ignored), two binary tests,13 font tests,22 text tests and60 doctests. Strict
workspace/all-target Clippy and formatting pass. No new native capture is claimed
for these controls; this checkpoint exercises the real egui input/focus pass and
hosted modal lifecycle, supplementing the prior native Go to Line checkpoint.

Evidence is retained outside /tmp under
~/.local/state/radiology-qa/dialog-focus-20261105-168eb13/ (red/final-focused logs,
workspace, Clippy and architecture counters).

## Architecture and remaining work

Baseline before-dialog-focus at 168eb13 versus after-dialog-focus: +122 measured
NLOC, four functions (three tests and the shared test frame), same 666 files,
average/max CCN 3.40/132,36 high-complexity functions,125 clone instances/1335
repeated lines, one cycle and eight structural issues. Full clone fragment/file
multisets have zero additions/removals. The import renderer changes CCN 13→14;
final comparison is NEUTRAL with no new problems. Existing ModalInput and shared
picker query are reused; no second focus/input mechanism is introduced.

This fixes a keyboard defect found during parity review. It does not establish
completion of the remaining Swift assertion inventory. SDK-backed Android success
also remains pending the previously requested SDK license/install decision. Full
assistive accessibility and actual little-screen-clip adoption remain deferred.
