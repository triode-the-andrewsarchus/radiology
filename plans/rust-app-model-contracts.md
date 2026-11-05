# App-model assertion reconciliation — 2026-11-05

All six ordinary functions in Swift AppModelTests now have explicit Rust evidence.
The earlier root-selection and passive-history checkpoints repaired actual gaps;
this checkpoint adds tests for the four remaining groups. Existing runtime meets
those remaining contracts. Production code, dependencies and public APIs are
unchanged here.

The new cohesive app/tests/model.rs module uses the actual app, source loader,
outline actions, input hook and egui renderer at both scales. It shares existing
reader/session fixtures. A small first_outline_target test helper synchronizes the
accepted source and validates Home's source-bound preview token; the prior passive
history test uses the same helper instead of duplicating that setup.

## Swift function mapping

| Swift AppModelTests function | Authoritative Rust evidence |
| --- | --- |
| latestNavigationWinsAndBackRestoresScrollLocation | New latest_navigation_restores_reported_positions_and_retires_another_pending_file uses the same first/second/third text and bytes 13/6. Real Down keys report line 3; B.swift and Back restore byte 13/line 3. A new B request is superseded by immediate A byte 6 navigation; loading/pending ownership retires, the accepted Arc remains and frames retain byte 6/line 2. |
| passiveScrollingDoesNotReplaceCommittedStructuralTrail | The passive-trail regression and native workflow preserve an implicit reading position, a committed outline stop and independent semantic departure. Later passive scroll does not replace the committed stop; Back/Forward restore each role. |
| repeatedOutlineActivationAlwaysIssuesAJumpAndRestartsHighlight | New repeated_pointer_outline_activation test uses Activate with highlight feedback and a controlled egui clock. Repeated same-target activation after expiry and during fading returns a full-strength band without changing history. Clearing an existing selected fn range proves it actually places the jump. Swift's internal revision increments are represented by these observable effects. |
| aNewRepositoryClearsOldStateAndWinsAnOpeningRace | The empty-reader regression confirms an old discovery worker is running, accepts a newer real root, then releases the old result. Root/inventory stay current, source/history stay empty, and no repository error appears. Native plain/explicit-file launch and refresh evidence is linked below. |
| supersededParseCannotRepaintTheNextFile | New superseded_swift_preparation_and_language_changes test prepares 2000 old Swift functions, verifies old1999 was actually parsed, holds that completed result, starts Old again and immediately requests Current.swift. Current owns the accepted source/symbols/colors/folds. Publishing the held retired result afterward cannot replace its Arc, native captures or fold facts. |
| switchingLanguagesReplacesTheSnapshotAndItsGrammar | The same test checks Swift Current facts → Rust main plus one trait implementation → unsupported NOTES.md. Native captures are bound to their exact source; Swift captures do not match Rust. Notes clears outline/folds/trait facts and prior structure/fold errors, renders No Outline and remains exactly copyable as # notes plus LF. |

The two existing fixes are documented in
[empty repository reader](rust-empty-repository-reader.md) and
[passive/structural history](rust-passive-history-trail.md), with their native
acceptance boundaries and exact-source evidence. This table closes the six-group
app-model review, not the remaining Swift suite inventory.

## Verification details and boundaries

The retired result is held at the existing loader publication boundary after real
parsing completes. The test also exercises ordinary Old→Current request
replacement. It does not pretend an impossible retired channel can bypass
LatestRead ownership during an active successor request. Source-loader replacement
and bounded-worker retirement tests remain active; Current's post-publication
ownership guard is checked directly with the held value.

The language test explicitly seeds prior recoverable structure/fold errors before
opening plain text, so clearing them is checked rather than inferred from an
already-empty error field. Rust keeps an empty DocumentSyntax value for unsupported
formats; Swift uses nil. Empty derived facts, no stale errors, No Outline and exact
readable/copyable source are the relevant equivalent behavior.

The feedback test observes full-strength, half-faded and expired painted bands.
After repeated activation, Copy no longer returns the old selected fn range,
proving the jump was issued despite identical history. Another activation restarts
while fading. Keyboard Commit clears the cue and does not flash, matching Swift's
onKeyPress Return path; pointer Activate uses highlight:true. No wall-clock sleep
is needed for that animation test. Both scales run every group.

No new native run is claimed for this test-only checkpoint. The linked root and
history checkpoints contain current app/native evidence; this checkpoint supplies
the direct app-publication/grammar/controlled-clock assertions. AppKit/SwiftUI are
not executed on this Linux host.

Workspace: 799 app tests/11 ignored, two binary, 13 font, 22 text and 60 doctests pass.
The three model tests and shared passive-outline caller pass again after helper
consolidation; strict workspace/all-target Clippy and formatting pass. Logs and
architecture counters:
~/.local/state/radiology-qa/app-model-20261105-9e5c6c1/.

## Architecture and remaining scope

Baseline before-app-model-contracts at 9e5c6c1 versus after-app-model-contracts:
+209 measured NLOC, one test module and four test-only functions (three regressions
and the shared outline fixture). Same average/max CCN 3.40/132, 36 high-complexity
functions, 123 clone instances/1323 duplicate lines, one cycle and eight structural
issues. Complete clone fragment/file multisets show zero additions/removals. New
regression CCNs are 3/6/4; shared target setup is 2. The raw comparison flags an
unchanged clone in the modified parent test module. The initially introduced
outline-setup clone was removed through actual fixture reuse.

Continue the other ordinary Swift suites against individual assertions. The
machine-specific opt-in probes retain their recorded limits; SDK-backed Android
success still awaits the earlier license/install decision. Full assistive
accessibility and actual little-screen-clip adoption remain deferred.
