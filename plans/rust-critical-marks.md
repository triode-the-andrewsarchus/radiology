# Critical Lens decisions in Rust

## Storage and replay contract (2026-10-30)

`critical::overrides::OverrideStore` stores one bounded JSON file per canonical
repository root under an explicitly configured private settings directory. SHA-256
root keys and a root field isolate repositories; source-root aliases resolve to
the same decisions. Construction and missing-file loads create nothing. Storage
inside the viewed repository, foreign roots, unsupported formats, corrupt/oversized
JSON and nonregular/final-symlink inputs are rejected. The Rust disk schema is
private; this does not promise Swift JSON format compatibility.

Nonempty snapshots serialize to a synced temporary file and replace atomically.
Empty snapshots delete the file. Failed writes retain the previous file and report
an error. Clones share save ordering: reserve a revision when accepting an edit,
before scheduling its snapshot. A delayed lower revision cannot overwrite a newer
attempt, including after deletion or a failed newer attempt. This ordering applies
to the store and its clones, not independent processes/stores.

`MarkEdit` separates stable symbol and version-independent package identities from
decisions. Critical/Ordinary persist; None/Unknown restores built-in behavior.
Locals and empty identities never persist. Loaded snapshots are normalized to the
same rules. Edits can replay on top of a delayed load without replacing unrelated
stored decisions. `CriticalityOverrides::default` now correctly selects format 1
instead of derive(Default)'s accidental 0.

The implementation extends the existing regular-file, cancellation-aware bounded
reader and reuses workspace destination-path resolution. Index cache paths now use
that same resolver, removing their duplicated ancestor traversal. No additional
writer thread, runtime, database, configuration write or dependency was introduced.

## Verification

Seven tests cover atomic round trips, no idle writes, root/alias isolation,
normalization, obsolete and concurrent save ordering, deletion, failed newer saves,
corrupt/foreign/unsupported data, bounded/symlink input, cancellation and pending
edit replay. Full validation passes 164 application tests, 5 reader-text tests,
30 doctests, fmt, Clippy with warnings denied and binary build. The external-Clang
acceptance test remains ignored by default. Architecture comparison retains a
seven-line fixture clone; no new product clone/CCN hotspot/cycle is reported.
Maximum CCN remains 100, with about 0.7% duplication.

## Session lifecycle (2026-10-30)

`MarkSession` uses replaceable background loads and a single coalescing durable
writer on the existing worker pool. Pre-load edits replay over stored decisions;
repository switches and Drop retain accepted edits. Controlled shutdown can drain
with a deadline. Errors remain repository-scoped and failed saves retain session
decisions. Exact-source menu targets reject locals, Before and mismatched bytes.
SCIP package headers and cached repository ownership are shared with the index,
avoiding a whole-index scan during menu rendering.

Four lifecycle tests cover delayed loads/switches, rapid decisions, shutdown,
failure isolation and Drop; a semantic menu test covers identity guards and
cached ownership. The complete working tree passes 169 application tests,
5 reader-text tests, 30 doctests and Clippy. UI integration remains under
verification and is deliberately separate from this backend commit.

## Remaining reader integration

This backend is implemented; current reader projection requests still use empty
overrides and no source marking menu is connected yet. Integration must:

- Capture the canonical root/private store as repository session state and retire
  stale load publications through the existing LatestRead mechanism.
- Apply user decisions immediately, retain/replay edits accepted before load
  completion, and save only the merged snapshot after loading.
- Keep durable writes independent of cancellable reads/repository switches.
  Coalesce pending snapshots per root while a single bounded-worker writer runs;
  return only current-session save failures to UI. Store revision reservation
  prevents a delayed old request from rolling back later decisions.
- Refresh the exact-source Critical Lens projection when loaded/edited decisions
  change, preserving source/caret/history/folds and using the existing policy.
- Add Normal/Critical source context-menu actions for Critical, Ordinary and
  Built-In at symbol/package scope. Exclude locals, Before and mismatched semantic
  source; capture the full SCIP identity, not identifier spelling. Show defaults
  and save errors without making a failed save undo the current session decision.
- Verify delayed loads, rapid edits/repository transitions, failure feedback,
  app restart and actual native menu/projection interaction before accepting UI.

Call graphs/frontiers, hover/count/fold annotations and full Rust/Swift acceptance
remain separate open requirements. Persisted decisions are not a completion claim
for those workflows.
