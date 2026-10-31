# Persistent Rust semantic indexes

## Backend contract (2026-10-28)

Generated normalized SCIP data, exact source SHA-256 digests, the build-input
inventory and complete bounded coverage/diagnostic report are persisted together
in a format-versioned JSON entry. Canonical repository paths have distinct SHA-256
cache keys even when an embedded host shares a cache parent. Cache writes stay
outside the viewed repository, stream through a bounded buffered writer, sync and
atomically replace one file. Failed, empty, cancelled or stale generations never
replace the previous entry. A write failure retains usable in-memory semantics and
reports a cache warning.

`IndexGenerator::start_cached` uses the existing cancellable worker ownership; it
never invokes tools, copies inputs or creates directories. Hosts can distinguish
missing, loaded, failed and cancelled cache requests through `CacheLoadStatus`.
Original source digests survive reload, including stale reloads. Changed inputs
are explicitly stale; source destinations still reject differing bytes. Metadata
is never relabeled with a fresh digest. Missing caches can fall back to explicitly
unverified repository imports.

Cache reads reject unknown formats, foreign roots, inconsistent inventory/plan/
coverage, duplicate/escaping document paths, unsupported encodings, malformed
ranges and missing/invalid source digests. The shared SCIP/cache binary reader
rejects final symlinks/nonregular files, limits reads to 256 MiB and checks
cancellation between chunks. Input fingerprints share generation's byte/type/mode
identity calculation and read limits without materializing workspaces.

Tests cover partial-report and scoped-definition round trips with tools removed,
source/build/inventory drift, original digest retention, distinct repository
roots, malformed metadata/ranges/paths, truncated JSON, symlink/oversized entries,
failed generation preserving prior cache and write errors preserving memory data.
Read-only fingerprints match copied-workspace identities and detect mode/link
changes while enforcing limits and cancellation.

Backend validation: 133 application tests, 5 reader-text tests, 30 doctests, fmt,
Clippy with warnings denied and binary build. Architecture comparison removes
initial production-reader duplication and new CCN hotspots; the final comparison
still reports REGRESSION for a six-line test-fixture clone (77 vs 76 groups).
Maximum CCN remains 100 and there are no new dependency cycles.

## Integration and remaining Swift requirements

Repository opening and Reload Index now load the private cache first, with a
missing/invalid-cache fallback to repository SCIP imports. Cache failure remains
visible alongside the import's explicitly unknown freshness. Generation and cache
loads share a captured request and existing job ownership; cancellation suppresses
fallback, and repository/review transitions retire pending publications. Reindex
still forces a new tool run. A persistence warning is visible in Index Projects.

Application integration tests cover reopening with the fixture indexer removed,
stale source, corrupt-cache import fallback, cancelled cache loading and switching
repositories before publication. Final verification: 133 application tests,
5 reader-text tests, 30 doctests, fmt, Clippy with warnings denied and binary build.
Architecture findings remain the one fixture clone documented above.

Native acceptance in a private Weston/Niri 1280×900 session generated an entry,
removed the fixture indexer, restarted to show Cached index, changed source and
restarted to show Stale cached index with semantic availability disabled for the
changed bytes. Corrupting the entry and supplying a repository SCIP file showed
both Cached index unavailable and Imported index / freshness unknown. Captures
are local `/tmp/radiology-cache-native-acceptance/{generated,reopened-without-indexer,stale-reopen,corrupt-cache-import-fallback}.png`.
The final app and both compositors exited zero. The first temporary driver trial
closed stdin early and terminated its owned app during cleanup; it was terminal
before the successful acceptance session started. Fixture output does not prove
compiler-indexer correctness or Swift visual parity.

Per-project incremental cache reuse, cross-revision reuse, validated tool-version
refresh policy, retention controls and pinned PR cache namespaces remain required.
Swift's cache is a behavioral reference; these entries are Rust-private normalized
data, not a promise of Swift disk-format compatibility.

## Normalized revision retention — 2026-10-30

[Retention](rust-normalized-revision-retention.md) now keeps the newly published PR
revision and two recent owned records per review-store identity. It shares the
project writer lease, defers while busy, preserves unknown/foreign data and leaves
cache-only reads unchanged. Automated cross-process/publication/open-reader checks
pass; final native PR retention acceptance remains open.
