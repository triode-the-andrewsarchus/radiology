# Shared project inspection — 2026-11-02

## Behavior and reuse

Index Projects, tool controls and generation now use the same build-input discovery.
A tracked `build/compile_commands.json` is detected before Generate Index even
though the browser excludes build directories. Git inputs remain tracked/unignored
files; non-Git inputs retain the existing bounded walk and exclusions. This does
not discover ignored/untracked generated databases or external build directories.

`service::inspect_inputs` extracts GenerationRequest's existing read-only inventory
boundary for reuse without a cache, copied workspace or tool execution. The UI's
existing LatestRead worker passes cancellation and owns the review Arc/lease.
Pinned reviews use verified exact head inputs. Review installation now sets the
new review before scheduling inspection; the new regression caught the old order
reading the attached checkout after this shared boundary was introduced.
Repository replacement still retires the old worker through existing ownership.

Inspection errors clear the old plan, show a persistent truthful project-panel
failure and publish complete diagnostics through the existing error notice.
Dismissing that notice stays dismissed; repository reload retries inspection.
Successful inspection/generation or repository reset clears the failure state.
The misleading claim that only generation checks complete inputs was removed.
No new dependency, worker mechanism or cache requirement. No sibling changes.

Swift's initial plan also uses browser paths, while generation has a broader input
copy stage. This change makes Rust's controls consistent with its already existing
generation contract; it does not claim broader Swift C++ corpus completion.

## Verification

- RED: actual app regression fails before the fix because scip-clang is absent
  despite the tracked nested database. The same test passes after the fix and
  verifies browser exclusion, detection before generation, rendered project-panel
  state, malformed Git discovery failure, cleared stale plan and reload recovery.
- Pinned app regression covers a manifest absent from the attached checkout,
  verified head inputs, wrong-root rejection, pre-cancellation, modified pinned
  bytes, retained displayed source, dismissed diagnostics and unchanged checkout.
- Existing unfiltered-plan/repository-replacement regression remains green.
- Full workspace: 625 app tests pass, seven ignored; two binary, 13 font, 19 text
  and 40 doctests pass. Strict workspace Clippy/check, standalone binary build,
  rustdoc with denied warnings, formatting and diff checks pass. Rendered public
  inspection docs checked. Copied executable verifies embedded resources outside
  the repository (exit zero).
- Actual egui frames cover project-panel messages. No new native compositor
  acceptance or real compiler corpus acceptance is claimed for this increment.

## Architecture review

Baseline `before-shared-index-inspection` at d387597: 86,698 NLOC, 3,659 functions,
average CCN 3.34, maximum 132, 109 clone groups and no dependency cycles.
Final delta: +122 NLOC/four functions including two regressions, unchanged average
and maximum CCN, no new hotspot/cycle, health -0.00. Inventory logic is moved into
one shared boundary rather than duplicated. LatestRead, existing error publication,
head verification and generation request ownership are reused.

The first measurement identified one new duplicate test sequence; independent
assertion ordering now follows each test's behavior and removes that duplicate.
Final default comparison still reports REGRESSION for four existing app-test
clones touched by line shifts. A normalized file-pair/whitespace-stripped-fragment
Counter matches all 109 old groups exactly, with zero additions or removals.
No production hotspot is worsened; the increased test code exercises discovery
failure and pinned-root integrity rather than a second inventory implementation.

Evidence archive: `~/.local/state/radiology-qa/shared-index-inspection-20261102-d387597`.
Broader generated database discovery, real Android/Gradle/C++ corpora and the final
Swift feature/UI acceptance remain open. Accessibility and sibling adoption retain
the user's deferred scope.
