# Critical Lens projection and classification reuse

## Contract and Swift reference

Swift CriticalityWorker retains eight exact-source projections and memoizes symbol
classifications within the current index/policy revision. The Rust CriticalLens
service now retains an eight-entry LRU with a 16 MiB estimated allocation budget
for syntax facts and projection ranges. It also retains at most 4,096 classification
decisions/8 MiB of estimated identity-key storage for the current scope. Oversized
entries still compute correctly but are not retained. Weight estimates include
vector/string capacities and structural overhead; these are not allocator-wide
memory limits. One current policy remains outside the projection-entry budget.

A projection matches exact path, source SHA-256, Before/After/checkout revision and
complete parsed CallSites facts. A new Arc with equal facts can reuse the entry;
unsupported, changed or stale syntax cannot reuse another parse's presentation.
The semantic index is identified by a Weak pointer, avoiding index retention and
pointer-reuse ambiguity. Any index identity or override change resets both caches.
Repository/index teardown calls clear; Normal mode uses stop, retaining valid memo
entries for mode toggles and source revisits. Appearance/font choices do not enter
the key because the projection contains canonical bytes, not colors or geometry.

## Ownership and reuse

Lookup, equality checks, policy preparation, classification and projection work run
on the existing bounded read-worker pool. The UI only schedules/retires requests.
A worker-owned mutex serializes cache access; waiting polls cancellation every two
milliseconds. Projection checks cancellation between calls. Current publication
still uses LatestRead, so retired jobs cannot replace the displayed projection.
Clearing replaces the cache owner without waiting on a worker-held lock.

Pure projection::make remains available and uses a request-local bounded memo. The
service reuses that same computation with its scope-owned memo, preserving callee
resolution and policy precedence. Debug tracing reports cache_hit and source path
under radiology::critical::service::cache for measured acceptance, with no product
UI counters. The cache retains parsed facts/ranges, not source buffers, PR snapshots
or source-storage leases. Call-graph frontiers/counts remain request-owned because
other checkout files can change independently of displayed bytes; projection reuse
does not bypass those source validation paths.

## Automated evidence — 2026-10-30

Four service tests cover fresh parses on revisits, mode toggles, different paths,
bytes, revisions and syntax support; independent policy and index replacement;
weak index lifetime; LRU entry eviction; cumulative byte-budget eviction; oversized
facts; and cancellation while a cache lock is held. A classification test exercises
both count and byte bounds independently, including an oversized first key and
correct uncached decisions. App integration proves mode toggle/file-return reuse
through the real reader command pipeline while retaining index-arrival invalidation.

Full workspace gates pass 424 app tests (three ignored), eight shared tests and
30 doctests, build and rustdoc. After refining test tables and budget assertions,
47 Critical Lens tests, the reader integration test, strict Clippy and formatting
pass; build/rustdoc pass again after the final API documentation edit. Architecture
adds no hotspot/cycle/clone growth. Four reported app-test fragments are byte-identical
to baseline before-critical-projection-reuse. Test cases share request/setup helpers.

Logs: /tmp/radiology-critical-cache-{tests,app-tests,all-tests,clippy,build,doc}.log.
Native revisit/policy evidence follows. Broader graph/corpus/runtime, UI/scaling/
accessibility/font and sibling acceptance remain open within the parity objective.
