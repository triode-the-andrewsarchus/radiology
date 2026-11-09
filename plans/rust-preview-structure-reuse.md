# Parsed destination reuse for definition previews

Reference: `ContextModelTests.crossFilePreviewsEndAtTheirTreeSitterDeclaration`
and Swift SemanticQueryWorker's four-entry structure cache, compared with Rust
`a9f0459` and this checkpoint. This maps that one ContextModel group. The complete
remaining seventeen groups still need assertion reconciliation; the earlier
[result reuse](rust-context-query-reuse.md) closes a mechanism required by the
first two groups without claiming their whole suite complete.

## Missing behavior and correction

Rust bounded previews at their real Tree-sitter declaration, but each distinct
uncached target extracted the destination's symbols again. The new actual-worker
regression loaded two adjacent definitions from the same checkout file and counted
two extractions instead of one (`red.log`).

The existing preview worker now retains four exact destination structures through
the shared QueryCache. Keys use path, revision and content digest; existing captured
storage/index/source ownership scopes still apply. An immutable Arc symbol slice
supports each target's declaration-end lookup. Requests for different symbols in
unchanged destination bytes reuse that slice. Source reads and index-coordinate
validation still happen before lookup, preserving fresh-read and provenance guards.
The existing per-cache 32 MiB retained-payload limit also bounds these entries;
oversized structures remain usable for the current query without being retained.

Source/index/root/revision replacement retires structure ownership alongside the
existing result cache. Changed observed destination bytes invalidate old structure
before lookup. The cache retains symbols and bounded metadata, not another copy
of whole destination source files or Tree-sitter's mutable parser state.

## Worker ownership and cancellation

Structure lookup/extraction runs on the existing preview blocking worker. The UI
only exchanges an Arc at ownership boundaries and clones it for a request; it
never acquires the structure mutex. Parsed data is immutable after insertion.

The pre-existing Critical Lens checked mutex-wait loop is now one shared private
background helper. Critical Lens supplies its existing cancellation check; preview
structure supplies its existing source cancellation/deadline check. Both retain
2 ms polling and poisoned-lock recovery. No new pool, job protocol, dependency,
public API or platform UI mechanism is introduced.

Checks run before lock acquisition, during contention, before lookup, during
symbol extraction and payload accounting, and before the result is used. A query's
30-second source deadline includes lock waiting and parsing. Existing LatestRead
channels still retire abandoned excerpt publications. Tests also hold the mutex
while requesting/cancelling a preview on the UI side, requiring no lock acquisition
and no extraction there.

## Contract evidence

| Assertion | Evidence |
| --- | --- |
| Cross-file adjacent definitions | New loader test uses a real checkout read, a separate displayed host and exact indexed coordinates. restore_app yields lines 4–6; restore yields 7–10. Both end at their own closing brace, are not truncated, and share one extraction despite distinct uncached targets. Unicode source remains intact. This maps the Swift cross-file declaration-bound group. |
| Unsupported grammar | The same table includes a real notes.txt destination with validated source identity. Its preview contains one/two and ends before the blank line, retaining the existing fallback. |
| Four-entry bound | Unit tests prepare five distinct files, require a hit for a retained file, then require a new extraction for the evicted oldest file. Shared QueryCache tests separately verify payload bounds. |
| Changed bytes | The same path gains a multiline body; the next end location becomes line 3 and extraction count increases once. An unchanged repeat hits that new structure. |
| Contended cancellation/deadline | While the mutex is held, cancellation and a deadline expiring during the wait return errors with zero extractions. Releasing it permits a successful cold extraction. |
| UI request/cancel | A preview request and cancellation both complete while the worker mutex remains held; the displayed phase is None, the old publication channel is retired and extraction count stays zero. |
| Existing source safety | Eleven preview checks retain real pinned-child cancellation/reaping, primed source, Before/stale/bad-coordinate rejection, source deadlines, UTF-16/long-line handling and retired excerpt publication. |
| Shared Critical Lens wait | All 56 Critical checks pass with the same wait/cancellation behavior after the old private loop moves to the shared helper. |

The adjacent/unsupported cases share one test table rather than duplicate fixture
and request/assertion blocks. A test-only preparation helper keeps the existing
pure source-validation tests on the same worker preparation code with a cold
structure cache. Test-only extraction counts do not add product state.

## Verification and architecture

Archive: `~/.local/state/radiology-qa/preview-structure-20261109-a9f0459/`.
The reuse regression failed with count two instead of one before implementation.
The original direct cancellation fixture supplied borrowed inputs; its test helper
now captures the same immutable fields for the worker path. Focused eleven-preview
and 56-Critical checks pass, followed by strict Clippy/build/docs and full workspace
verification. The test-only case consolidation receives a final focused check
after the workspace run; runtime code is unchanged by that consolidation.
The locked offline full workspace run passes 882 application tests (11 existing
opt-in ignored), two binary tests, 13 font tests, 22 text tests and all 66 doctests.
The final consolidated preview table passes all eleven checks afterward, along
with strict all-target workspace Clippy and formatting/diff checks. Normal build
and warning-free docs pass. Compilation uses -j 1 and tests run serially; normal
half-core Cargo configuration remains 16 jobs. Previously recorded parallel
timing/linking limits remain open.

Architecture adds 258 measured NLOC/five net functions/one focused file. Most
new code is the worker memo and three regressions; declaration-end logic and the
existing checked-lock loop move without duplication. Average/max CCN remain
3.42/132 with 36 hotspots and unchanged cycles. Complete normalized clone multisets
retain 124 instances, zero additions/removals after consolidating the genuinely
new duplicated fixture block. Raw touched-file clone flags remain existing debt.
Existing query scope/bounds, extraction, source reads, workers and publication are
reused. The shared helper also removes the Critical-only lock implementation.

The subsequent [memory-preview checkpoint](rust-memory-definition-previews.md)
closes the identified cold same-file worker gap using accepted source/outline
facts and maps the complete Unicode-caret group. Other ContextModel groups remain
open.

The remaining ContextModel/reference/call-tree/palette assertions, source/syntax,
compiler/indexing and inspector reconciliation, final native/standalone acceptance,
known parallel timing/linking investigation and Android remain open. Full
accessibility and actual sibling-app integration stay deferred.
