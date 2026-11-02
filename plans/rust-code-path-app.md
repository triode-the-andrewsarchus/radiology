# Code Paths app navigation and source ownership

Code Paths now has an asynchronous app navigation API, using the existing bounded
read workers and source/history transaction. The [standalone sidebar/import workflow](rust-code-path-sidebar.md)
is now integrated and verified separately. Hosts can select a validated owned trace with select_code_path,
open a step/conditional variant with go_to_code_path_step, poll through poll_source
and observe loading, accepted step, destinations and visible errors. The format/store
APIs remain independent of egui; eventual little-screen-clip adoption is deferred.

## Publication and source contract

The request captures the canonical attached root, exact review snapshot, selected
trace object, displayed source object and caret. New navigation cancels the pending
step immediately, including a request returning to the same destination. Trace
replacement cancels even when saved UUIDs are equal. Root opening, scope/review/source
replacement and observed caret movement retire late success and error publication.
The source widget records movement throughout its ordered input batch: Right/Left
in one frame retires the request even when final caret coordinates equal the start.
No new background pool, navigation history, source file reader or parser was added.

The resolver's buffer and DocumentSyntax stay paired. Existing prepare_loaded now
accepts optional already-computed structure, so Code Paths does not parse its source
again. The same preparation still calculates colors, exact-source change information
and initial folds outside rendering. Ordinary file/review loads pass no structure and
retain their existing parse/recolor behavior. An appearance change recolors the
accepted buffer from retained captures without changing the resolved destination.

A ready step publishes atomically through the existing PendingNavigation and
publish_source path: exact departure history, hidden-test/fold reveal, function-body
expansion, destination feedback and symbol/reference context retain their meanings.
Reviews resolve After regardless of displayed Before; Back restores the exact Before
byte/revision. Checkout resolution prefers a matching displayed immutable buffer even
when the on-disk file has changed. Step selection and conditional alternatives publish
only after source preparation succeeds. Missing/ambiguous/preparation/invalid-variant
failures retain the source, caret and history, and expose a step-specific error.
Cancel uses the ordinary source loading control. Sidebar visibility is not an owner;
switching tabs must preserve the selected trace and step when presentation is added.

## Evidence

Nine app tests cover exact checkout and Before/After Back, displayed-buffer authority,
first/explicit conditional variants, invalid variants, missing/ambiguous source,
explicit cancellation, queued stale success/error, same-ID trace replacement, source/
review/root replacement and appearance changes. Gated completion receipts establish
that results are queued or cancelled, instead of relying on sleeps to infer a race.
The real app-frame Right/Left batch test runs at1× and2×. The existing app-frame
harness is reused; its small test-helper visibility changes add no runtime API.
Initial tests used a nonexistent Back convenience and inaccessible fixture helper;
they now exercise the existing command and shared fixtures. The initial color test
also assumed a highlight row for an empty trailing line; it now compares exact light
native capture colors, which is the behavior under test.

cargo run --example code_path_app_probe -- ROOT exercises the actual app model and
source/history lifecycle on the reference seL4 checkout, with private temporary
settings. All91 steps open; all93 conditional destinations in22 files match the
independent resolver and Git blob digests/byte/line coordinates. The probe sends the
reader's Back shortcut and verifies91 exact departure restorations. The reference
checkout stays clean at d419a119a61a9b42edf61fb79ad08de45248daa7. This is app model and
keyboard routing acceptance; there is no native Code Paths sidebar/import acceptance
claim yet. The previous resolver unit separately verifies native C++ reader rendering.

Full workspace:706 app tests pass, seven existing ignored; two binary,13 font,21 text
and54 doctests pass. Strict Clippy/check/build/rustdoc/fmt/diff gates pass. Rendered
app selection/navigation/observation API documentation inspected. Required SwiftUI/
macOS/typography skills remain absent from installed skill locations; no Swift UI,
font scale or new custom presentation surface is changed in this unit.

QA: ~/.cache/radiology-qa-code-path-app.
Archive: ~/.local/state/radiology-qa/code-path-app-20261102-c1c3f07.

## Architecture audit

Baseline before-code-path-app at c1c3f07. Reuse searches and source review located
LatestRead, source preparation/publication, exact history and ordered source input.
Initially extracted preparation duplicated a signature/check block; extending the
existing prepare_loaded removed it. The probe's26-CCN main was separated into CLI,
step transaction and variant verification, with maximum new-function CCN11. These
functions perform distinct verification responsibilities rather than forward calls.

| Dimension | Before | After |
| --- | --- | --- |
| Analyzed NLOC |98,143 |98,795 (+652, including tests/probe) |
| Functions /files |4,102 /631 |4,131 /634 |
| Average /maximum CCN |3.39 /132 |3.39 /132 |
| High-complexity functions |33 |33 |
| Clone groups /cycles |126 /1 |126 /1 |

Raw comparison reports REGRESSION for pre-existing clones touching test/helper files
and shifting untouched Python chokepoint ties. Full path/fragment/token comparison
against the baseline proves zero added/removed clone identities. The two initial
new clone groups and excessive probe complexity were eliminated; no new cycle or
hotspot remains.

## Subsequent integration

[Saved collections and the standalone sidebar](rust-code-path-sidebar.md) now bind
background load/import/delete to the navigation API, including numbered steps,
conditional choices, Copy/Delete, UTF-8 file/paste import and shortcut handling.
Native1×/2× dark/light/relaunch acceptance passes. Desktop-portal chooser and real
IME-engine acceptance remain separate from the synthetic editor/input coverage.
Full Swift parity remains incomplete; see the master ledger for indexing and help.
