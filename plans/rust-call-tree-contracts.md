# Reference caller trees and identity colors

Reference: all seven ReferenceCallTreeTests groups and the one CallTreePaletteTests
group. Comparison begins at b959b1f. The ContextModel audit is complete separately;
source/syntax, compiler, inspector and final whole-app acceptance remain open.

## Loaded disclosure reuse

The first red check collapsed and reopened a loaded reference, then checked its
caller phase before polling. Rust restarted the trace, unlike Swift's retained
loaded tree. ReferenceContext now moves a completed tree and its continuation
limits into QueryCache on collapse, and consumes that result on reopen. The
existing renderer sees the loaded tree immediately, without a new trace worker.
Source excerpt preparation remains separate and may prepare again on reopen.

The cache uses the existing 24-entry/32 MiB payload bounds. Keys contain path,
original SCIP range and validated source digest. QueryCache gains an owned take
operation that also releases accounted bytes. Existing reference-context ownership
clears closed results with target/index/root/revision/source retirement; observed
excerpt source changes also clear them. Caller-only source identities continue
through the existing reference cache's accepted-source invalidation. The cache
contains trees and limits, with no worker, loader, source bytes or index Arc.

Pending traces and failed continuations are excluded from completed-result reuse;
collapse drops their publication channels. Reopening those starts a fresh attempt.
A loaded continuation's expanded limits move with its tree. Tree accounting covers
row capacity, names, paths, notes, scoped symbols, notices and source identities.
Source identity accounting is shared with reference results, removing a new
six-line clone rather than adding another budgeting rule. The built-in UI controls,
font sizes and layout are unchanged.

## Swift assertion mapping

| Swift group | Rust evidence |
| --- | --- |
| tracesEveryBranchAndStopsAtHonestBoundaries | incoming_calls parsed_branches uses the exact three-file UTF-16 fixture, asserting all nine names/depths, callback/closure/no-caller notes, entry.rs path/line, canonical café range and no notices/continuation. |
| recursionAndContinuationPreservePaths | row_limit_expansion compares every short DFS prefix, then the recursion matrix checks Swift's exact self-call source, explicit stopped/no-caller boundaries and an available sibling path. |
| missingFilesStaleCoordinatesAndCancellationAreExplicit | missing_changed_and_invalid_sites removes entry.rs, edits boundary.rs and adds invalid coordinates; notices retain the file/identity/coordinate reason. initial_identity checks cancellation, changed initial digest and Before refusal. Known captured provenance rejects changed bytes before stale-coordinate use, a stronger Rust guarantee. |
| launchAppearsOnceInAMutuallyRecursiveChain | recursion matrix checks the same launch/relay/entry/other topology, every row/depth, stop on relay, sibling continuation, prefix expansion and unique scoped identities along each branch. Its leaf is named café, also exercising Unicode. |
| distinctFunctionsNamedLaunchAreNotMistakenForACycle | identical_offsets matrix uses two middle callables in a.rs/b.rs with the same offsets. Both document-local and distinct workspace-global symbols retain their separate entry/other branches without a recursion note. Labels differ from Swift's launch aliases; callable identity and file separation are the asserted contract. |
| deepTreesHaveNoDepthCutoffAndCanContinuePastTheInitialBudget | traversal matrix contains 301 rows rather than Swift's 270: the initial 256 end at depth 255, all 301 end at caller299/depth 300, the initial prefix is unchanged and continuation ends. |
| disclosureLoadsLazilyAndResetDiscardsWork | independent_disclosures starts empty, drops pending work on collapse, restores loaded trees immediately, keeps sibling state independent and retires on index replacement. Both-scale real Context fixture checks total_area and immediate loaded reuse. Existing Context disclosure/lifecycle reset checks retire app ownership; the caller-only first-visit case now collapses before an accepted source edit and requires a fresh trace. |
| identityColorsStayStableAcrossLaunchesAndIncludeTheFile | caller_tree determinism now includes all seven exact portable Swift assignments: file-sensitive launch identities, nested path, café and ab/c versus a/bc. Existing FNV-1a method/NUL/path implementation already matches; no color algorithm change. |

The worker uses an iterative DFS and canonical source-validated call sites; the
legacy bounded index-only CallerTreeBuilder is not substituted as evidence for
these checks. Error strings retain Rust's detailed I/O/provenance messages rather
than pretending to produce Swift platform error text.

## Verification and structure

Archive: ~/.local/state/radiology-qa/caller-disclosure-20261109-b959b1f/.
Focused checks cover reference ownership, bounded cache insertion/consumption,
both-scale real-fixture app behavior, worker traversal and exact palette slots.
All 12 reference-context checks, three source-query/cache checks, two both-scale
app checks, nine incoming-call checks and the exact palette check pass. Strict
all-target Clippy, normal build and warning-free docs also pass. A fresh serial
workspace gate will follow this commit; no new full-run result is claimed yet.

Architecture adds 141 NLOC and four small runtime functions: owned cache removal,
shared source-identity accounting, tree payload accounting and completed-tree
extraction. No new file, dependency cycle or high-complexity site; average/max CCN
stays 3.42/132 with 36 high-complexity sites. Clone instances remain 125. Complete
multiset comparison has one shortened existing test setup/assertion fragment in
place of its prior longer fragment; this is not a new duplicate runtime mechanism.
The genuinely new source-identity accounting clone was removed by shared reuse.
Raw chokepoint differences are equal-ranked untouched Swift Python functions. The preceding full workspace run at b959b1f passed
889 application checks and all 66 doctests; it does not prove this later change.

No new dependency, public input or parallel query controller is introduced.
Repository/index/source retirement stays in the existing owner. Full accessibility
and actual sibling adoption remain deferred; Android SDK-backed success and the
previously documented parallel-run limits remain open.

The subsequent full workspace run exposed a keyboard-owner regression (888 app
passes, one failure, eleven ignores). [Caller keyboard retirement](rust-caller-keyboard-retirement.md)
repairs it while retaining immediate cached data. Its focused checks pass; the
following combined gate must establish fresh full success.
