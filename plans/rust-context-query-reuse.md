# Bounded Context query reuse

Reference: the preview/reference reuse assertions in
`ContextModelTests.underCaretIdentityIgnoresMovementWithinOneOccurrence` and
`declarationReferencesSurviveUnderCaretChangesAndKeepTheirSelection`, compared
with Rust `9868830` and this checkpoint. Complete reconciliation of the eighteen
ContextModel groups remains open; this checkpoint closes their missing result
reuse mechanism rather than claiming the entire suite complete.

## Missing behavior and correction

Rust already retained the current token and reference target while the caret
moved within their identities. Leaving and returning to a previously loaded
definition or declaration, however, always restarted source work. Both new
regressions failed before implementation: an excerpt worker and a reference scan
were live again instead of immediately presenting the previously validated data.

The two existing query owners now share one small internal cache mechanism:

- Definition previews retain up to 64 exact target excerpts.
- Declaration/explicit reference targets retain up to 24 immutable result sets.
- Each cache additionally limits retained payload to 32 MiB. Oversized results
  remain usable as active results, but are not retained for reuse. Replacing a
  cached key with an oversized result removes its old cached value.
- Reads preserve insertion order. Replacing a value preserves its position.
  Oldest insertions retire when either bound is exceeded.

Reference results use shared Arc storage: changing targets and restoring cached
rows does not clone large hit collections on the UI thread. Each cached result
also retains its query limits, so Load More continues from the accepted budget.
Selection and disclosures reset when changing targets, while movement within the
same declaration retains the same rows and selection.

There is no new worker, job protocol, disk cache, configuration option, dependency
or sibling-app tie-in. Queries still prepare on the existing blocking workers,
check cancellation, and publish through their existing retired channels. Failed
queries do not populate caches. An abandoned request cannot publish an entry.

## Ownership and invalidation

Shared source-query inputs and internal cache ownership now live together in
`source/queries.rs`, independently of preview rendering. The public
`source::preview::SourceQueryInputs` path remains a re-export with unchanged
fields and behavior. Preview and reference consumers therefore use the same
captured-storage mechanism without a preview/cache module dependency cycle.

A cache scope includes the exact immutable index, checkout root or pinned review
owner, source revision and known source contents. Repository/index/revision
replacement or loss of inputs clears all entries. Changing documents within the
same scope preserves reuse when known contents are unchanged. Review storage and
checkout storage cannot share entries; replacement review owners conservatively
start a fresh scope.

Source observations include displayed buffers and files read by excerpts,
reference queries and caller traces. Accepting changed bytes for any previously
observed path invalidates derived cached results, including the first reader
visit to a file previously seen only by a preview/caller. Unreadable files retain
an unknown-content marker: a later successful open invalidates cached partial
coverage even when another declaration was active in between.

The cache performs no synchronous filesystem reads. Unobserved checkout edits
follow the existing explicit-import/accepted-source ownership policy. Active
queries and navigation continue their independent source validation; caching does
not manufacture index freshness or bypass destination validation.

## Evidence

| Boundary | Verification |
| --- | --- |
| Definition result reuse | Existing replacement/cancellation test now prepares both targets, clears the visible target and returns to each without a new worker. The initially retired target must still perform its first real preparation. Reset clears retained excerpts. The redundant draft fixture was merged into this test. |
| Declaration result reuse | New backend regression changes declarations and returns immediately to the accepted rows without a running scan. Old row selection/disclosures stay cleared for the new identity. |
| Actual Context publication | Both-scale app regression uses the real Swift wire fixture. Moving through Circle retains exact Context/disclosure, outline Arc and reference Arc. Area's preview ends on lines 7–10. Returning to Circle polls only metadata publication and requires an already-loaded cached excerpt, never starting/polling an excerpt worker. Moving through area retains selected references; visiting new clears selection and returning restores the identical immutable area result Arc immediately. |
| Bounded retention | Tests cover insertion order, replacement, entry eviction, byte eviction, oversized new/replacement results and zero capacity. |
| Scope and source identity | Tests cover unchanged replacement buffers, cross-document reuse, accepted edits, changed root/index/revision, no inputs, pinned storage versus checkout and a changed preview-only file in the same pinned revision. |
| Partial coverage | New shared-symbol test caches an unreadable b.rs, visits another declaration, then opens real b.rs bytes. The old partial result is rejected and fresh definition/reference counts are restored. |
| Expanded results | Extended Load More test preserves real source/caller disclosures while expanding, then leaves and returns to the exact result Arc and all three saved query limits. |
| Existing safety boundaries | Source tests, reference-context tests, real pinned-child cancellation and app/workspace checks retain stale-source, Before, replacement-index, caller-only first-visit, pending-buffer rebinding and failed-read validation. |

## Validation and architecture

Archive: `~/.local/state/radiology-qa/context-query-reuse-20261109-9868830/`.
Initial red regressions are retained. Early full runs were intentionally interrupted
for the Load More budget correction and query-ownership extraction; they are not
final acceptance. The final focused checks/build/docs/full run are recorded below.

The new shared mechanism replaces the reference owner's private source ledger and
its duplicate observation routine. Existing query-input fields/read behavior move
without adding a public API. Existing bounds, validated results, workers, snapshots
and publication are reused. The draft duplicate preview test was merged into the
existing replacement test while keeping both cancellation and reuse assertions.

Architecture comparison adds 511 measured NLOC/13 net functions/two files.
Average/max CCN remain 3.42/132, with 36 hotspots and unchanged dependency cycles.
Complete normalized clone multisets retain all 124 instances with zero additions/
removals. Raw touched-file flags identify existing preview, result and app test
clones; the draft's genuinely new preview-fixture clone was eliminated by merging
the two substantive test flows.

Final focused verification passes 36 source tests (including pinned-child
cancellation), twelve reference-context tests and the both-scale real-fixture app
test. Strict all-target workspace Clippy, normal build and warning-free docs pass;
the generated public SourceQueryInputs page retains its old path and three fields
and points to the new shared implementation. The final locked offline workspace
run passes 879 application tests (11 existing opt-in ignored), two binary tests,
13 font tests, 22 text tests and all 66 doctests. Compilation uses -j 1 and tests run
serially; normal half-core Cargo configuration remains 16 jobs. The separate
parallel timing/linking investigation remains open.

## Remaining parity work

The complete ContextModel assertion mapping remains open, including lifecycle,
fallback, formatting and Unicode cases not fully reconciled here. Swift's
SemanticQueryWorker also retains up to four parsed destination structures across
different uncached preview targets; Rust still extracts structure for each missed
definition preview. That parsed-structure reuse remains an identified follow-up.
Reference/call-tree/palette, source/syntax, compiler/indexing and inspector suites,
final native/standalone acceptance, known parallel timing/linking limits and
Android remain open. Full accessibility and actual sibling integration are deferred.
