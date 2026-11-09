# Critical graph worker contracts

Reference: all five `CallGraphWorkerTests` groups, compared with Rust `091cd94`
and this checkpoint. The first group was mapped in the
[summary/hover checkpoint](rust-critical-summary-contracts.md#first-worker-group).
This pass reconciles the other four using existing mechanisms and stronger tests.
No production code, public API, UI or typography change was needed.

## Lifecycle equivalence

Swift's actor owns mutable configuration, primed files, summary/frontier caches
and epochs. Rust's `CallGraphWorker` owns one immutable `GraphInputs` configuration;
source, storage, index, syntax or policy changes replace the entire worker.
Files and callee summaries are cached within that request. Root frontiers are
recomputed; Rust does not expose Swift's whole-frontier cache/build counters.
This is an implementation difference, not a claim of identical cache performance.
Caret-only app frames preserve graph-input identity; they do not schedule another
annotation sweep. Remaining performance/corpus acceptance stays open.

The displayed source/call facts prime the worker before disk access. File reads
use captured checkout or pinned Git storage; semantic provenance and encoded
coordinates still reject stale indexes. `LatestRead` retirement cancels work and
drops its publication receiver. Old worker caches are separate allocations and
cannot populate replacement caches. Existing held annotation-batch tests force
late publication after replacement; the old counts cannot overwrite new policy.

## Assertion mapping

| Swift group | Rust evidence and boundary |
| --- | --- |
| `resolvesSitesLocatesBodiesAndHonorsMarks` | Previous checkpoint uses the same checked-in Rust-analyzer wire fixture as Swift: one area call, repository classification, exact cross-file destination/site count, complete status, cached no-call area body, ordinary traversal/proven zero and external no-body result. |
| `changedCalleeBytesInvalidateSummariesAndDependentFrontiers` | New `same_length_callee_replacement_retires_dependent_summaries_with_the_worker` changes open to read without changing byte length. The old request keeps its coherent snapshot without reading more bytes. A new request with the stale digest reports a partial lower bound, never proven zero. A replacement index/worker keeps the root's helper call but changes its transitive critical result to read and binds the callee body to the new digest. Deleting disk afterward preserves the replacement's cached result/read count. Identical repeated requests reuse loaded files/summaries; there is no mutable in-place prime or cross-request frontier cache to invalidate. |
| `reconfigurationDuringALoadNeverPopulatesTheNewCaches` | Extended `cancelled_semantic_reads_reap_pinned_blob_children` starts a real pinned-blob read through an isolated Git wrapper and confirms its child PID is live. While blocked, a replacement source/index/policy worker successfully summarizes primed viewer bytes. The old read remains live until explicitly cancelled, then joins with cancellation, is reaped within the existing two-second bound, and cannot change the replacement's answer. A retry on the old worker after release also succeeds, proving cancellation did not cache an unreadable body. Existing stale-source/new-policy worker tests and forced late annotation publication separately verify changed decisions and publication retirement. |
| `byteInvalidationDuringAnInFlightLoadNeverResurrectsOldBytes` | The same live-read test prepares changed viewer bytes first, supplies old call facts and requires NoSyntax. Replacing the call facts with matching parsed structure succeeds immediately while the old disk read remains blocked. Its body digest equals viewer bytes; querying again after old-read cancellation returns the identical accepted result. Missing-structure answers are scoped to the discarded immutable worker, so they cannot survive the upgrade. Existing displayed-buffer precedence test additionally changes disk bytes and requires the primed source digest. |
| `unresolvedAndDynamicSitesMakeCoveragePartial` | New `dynamic_and_unresolved_sites_remain_partial_with_an_explicit_empty_document` uses Swift's source with a dynamic function-pointer call and unresolved helper. An explicit empty semantic document (equivalent to Swift's empty DocumentSemantics input) yields no invented edges, a real parsed body, exactly two unresolved sites and 0+ rather than proven zero. Missing structure yields NoSyntax; matching facts restore the partial result. Existing abstract/stale/Before/encoding tests preserve distinct reasons and guards. An absent semantic document remains NotIndexed, just as nil semantics does in Swift; this test does not claim syntax invents index targets. |

The real in-flight test extends the existing subprocess/wrapper fixture shared by
previews, references, callers and Critical Lens. It does not change the parent
process PATH or add a production delay/test hook. All four consumers still prove
live-child cancellation/reaping and subsequent recovery. The callback-free
immutable worker remains usable by independent hosts.

## Verification and architecture

Archive: `~/.local/state/radiology-qa/critical-worker-20261109-091cd94/`.
`worker.log` passes all eight worker tests; `live-read.log` records the isolated
live-child/structure-promotion workflow for all four consumers. All 56 Critical
service tests, nine app/popup checks, strict all-target workspace Clippy, formatting
and diff checks pass. The prior
checkpoint's full workspace passed 854 app tests and all 65 doctests; no new full
workspace run is claimed for this test-only checkpoint.

Baseline `before-critical-worker-20261109` to `final-critical-worker-20261109`:
+155 measured NLOC/two test functions, unchanged measured files, average/max CCN
3.41/132, 36 hotspots and dependency cycles. Complete normalized clone multisets
retain all 123 instances with zero additions/removals; the comparator's warning
points to the old worker/frontier assertion clone. Shared source and fixture
mechanisms are reused without new product abstractions. The baseline includes
the previous checkpoint's newly committed Markdown, explaining its one additional
measured clone versus the earlier pre-documentation baseline.

All five worker groups now have explicit mappings. The three drawing groups,
other behavior suites, final whole-app/native acceptance, known parallel timing
limits and Android remain open. Accessibility and actual sibling adoption remain
deferred; ordinary keyboard support and standalone operation remain required.
