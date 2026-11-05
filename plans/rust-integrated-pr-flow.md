# Integrated pinned PR workflow — 2026-11-05

## Contract and checked-in coverage

`integrated_changed_files_refresh_and_cache_pressure_preserve_pinned_reader_transactions`
uses the existing private real-Git review fixture, refresh helper, source workers
and actual app frames. Added/removed/renamed paths publish exact After/Before bytes;
Select All/Copy uses the accepted reader. Rename Before resolves the original path.
A binary open reports an error while preserving the same buffer and history entry.

Five moving heads include a real path rename and inserted line. Refresh preserves
the caller's mapped line/column and pinned contents while the attached checkout
retains its deliberately different bytes. A retained snapshot keeps the oldest
worktree readable under cache pressure. Releasing it and publishing another head
permits eviction; reopening the original summary rematerializes its source into a
new managed tree without replacing the other app's accepted reader.

This is one integrated regression using existing mechanisms, with no production
runtime changes. Existing focused refresh, navigation and job-retirement tests
remain the evidence for other stale-result/history transitions.

## Native acceptance

The standalone binary runs in private Weston/Niri sessions at 1× and 2×. Each run
creates a local origin, seven review heads and two bases, plus a dirty attached
clone with deliberately incorrect source. Git fetch/materialization and scip-clang
0.4.0 are real; GitHub metadata is controlled `gh` JSON. This verifies the native
review workflow without claiming a live GitHub API check.

The accepted workflow exercises:

- Cold Clang generation and tool-free warm reopen; F12 reaches the exact pinned
  cross-file definition, rather than checkout bytes.
- Before source and the visible Before-not-indexed limitation; semantic commands
  preserve that reader instead of applying After coordinates.
- Added, deleted, renamed and binary Changes rows, unavailable revision toggles,
  exact full-text clipboard checks and retained source on the binary error.
- Refresh through an actual caller rename and changed implementation, fresh Clang
  navigation, then a moving merge base while reading Before.
- An identical-input new commit reuses the project record without another compiler
  driver invocation, retaining explicit revision provenance.
- More heads exercise managed-tree eviction. Reopening the evicted old revision is
  tool-free for source and honestly shows no semantic index after its normalized
  record has also been evicted. Explicit Generate restores the old index; F12 then
  reaches its original implementation and full Copy matches exactly.

Both managed trees and recognized normalized semantic records retain three entries.
They have separate retention policies. Source rematerialization does not promise
that an evicted semantic record survives. The final explicit restoration invokes
Clang again in these runs: four driver invocations and 21 total version/probe/driver
calls per scale; before restoration there are three drivers and 16 total calls.
The generated fallback report covers four documents/three translation units, with
zero missing paths. The footer retains the conservative Partial generated index
label for inferred build defaults; this is not authoritative project build coverage.

The attached source hashes, HEAD, refs and Git status remain unchanged. App,
restarted app, Weston and Niri exit zero. Captures and exact clipboard payloads
retain the before/after source and native outcome evidence. No typography or
production UI changes are made; the requested SwiftUI/design/type skills remain
unavailable in this environment.

## Harness corrections and limits

Earlier attempts exposed assumptions in QA rather than runtime bugs:

- A short caller with an inserted comment fell below Git's rename similarity
  threshold. Stable context now creates a real rename, verified through Git.
- Creating a new stamp file changes the conservative complete build input set.
  Reuse is checked with an empty commit; stamp changes supply later cache pressure.
- A mapped window can precede completed repository opening. The driver now awaits
  published review state before sending Generate.
- The initial reopen check incorrectly expected an evicted normalized record to
  survive. Final checks require honest unavailable status and explicit restoration.
- The app test originally expected immediate eviction with only three retained
  trees. A new head now creates actual pressure after the old lease is released.

These corrections preserve the documented semantics. This checkpoint does not
claim real remote authentication, every compiler/build system, all possible stale
job schedules, or full Swift parity. Heavy compiler outcomes, remaining Swift
behavior reconciliation and final standalone/reuse acceptance remain open.

## Verification and architecture

The full locked workspace passes 761 app tests (nine opt-in ignored), two binary,
13 font and 22 text tests, plus 59 doctests. Strict Clippy passes. The integrated
regression was rerun on November 5 and passes; formatting, diff and the locked
resource smoke/build pass too. Cargo retains 16 jobs on this 32-logical-CPU host.

Against `before-integrated-pr-flow` at `40f3ec4`: +115 NLOC/one test, test CCN4,
average/max CCN unchanged at 3.39/132, no additional hotspot or dependency cycle.
All 125 normalized file-pair/fragment clone instances and 1335 duplicated lines
are unchanged (Counter comparison, including multiplicity). Eight structural
findings remain. The NEUTRAL comparator's changed Python chokepoint ranking is
in untouched Swift reference scripts and is not introduced by this test.

Evidence archive:
`~/.local/state/radiology-qa/integrated-pr-flow-20261105-40f3ec4`.
It includes fixture/driver/actions, accepted and earlier diagnostic runs, captures,
clipboard bytes, cache reports/tool calls, source/Git hashes, checks and baselines.
