# Normalized PR index retention

## Behavior

After publishing a fresh PR semantic cache, keep at most three recognized
current-format revision records for that canonical review-store identity. Keep the
just-published revision regardless of timestamps, then the two newest publications
by file modification time, breaking ties by path. This matches the existing Rust
managed-worktree count. It is publication recency, not a read-updated LRU. Cache-only
loads remain read-only and do not invoke tools or update timestamps.

Checkout records, other repository/review-store identities, unsupported formats,
unreadable or malformed JSON, unknown filenames and symbolic links remain intact.
Ownership requires the current format, exact canonical identity root, a recorded
revision, and the existing SHA-256 filename derived from that root/revision. The
metadata scan validates JSON syntax but does not reconstruct semantic maps or
claim to validate every old semantic occurrence. The existing 256 MiB per-record
read/write limit and chunked cancellable decoding still apply. Unrecognized files
are outside the retention count, so this is not a total disk-quota guarantee.
Existing cache paths, keys and format 3 stay compatible; no migration is needed.

Stale inputs, failed project outcomes and compiler-reported partial builds do not
trigger eviction. Successful indexes with incomplete source coverage can still be
retained/pruned normally. Empty/stale indexes already skip publication. Cache
cleanup errors become the existing report warning while the new accepted file
remains published. Cancellation is checked during scanning, bounded JSON reads and
between removals; it does not roll back older files already evicted.

## Ownership and reuse

Normalized serialization/publication now holds the same shared `.project-cache.lock`
lease used by project generators. It uses the existing bounded/cancellable reader
lease acquisition, then releases it before trying an exclusive maintenance lease.
A live writer defers maintenance without blocking it; a later successful publication
retries. Consequently recognized records can temporarily exceed the count while
writers overlap. Exclusive idle maintenance also removes abandoned regular files
with the existing `.radiology-index-stage-` prefix at the normalized-cache root.
It never follows stage or record symlinks and does not remove input workspaces.

An already-open record descriptor remains readable after POSIX unlink; decoded
indexes are independent in-memory values. Retention does not hold an old index or
snapshot lease alive. This does not promise that an evicted revision remains
available for a later disk reload; it can be regenerated explicitly.

The normalized filename hash was extracted from the existing path function and
reused for ownership checks. The existing no-follow file lease, bounded cache
reader/JSON decoder, atomic stage publication and report warning path are reused.
No new daemon, cache format, dependency or sibling runtime requirement is added.

## Evidence — 2026-10-30

Three retention regressions use the existing private Git review fixture and valid
serialized cache fixtures. They cover current-record protection despite old mtime,
newest-two retention, checkout/foreign/unknown/malformed/unsupported preservation,
symlinks, abandoned stages, stale/failed/partial/cancelled requests and publication
through `save`. An actual external shared-lock holder prevents deletion of live
stage/history files; cancellation reaps it, idle retry prunes, and a descriptor
opened before eviction still decodes the original record afterward.

Idle assertions retry the documented deferred operation, accounting for unrelated
concurrent fork/exec briefly inheriting a close-on-exec lease descriptor. The test
holder uses `flock --no-fork` so the runner owns and reaps its actual PID.
The existing large-JSON cancellation and live serialization/recovery tests pass
with the shared lease. Real Rust and C/C++ lifecycle cases also pass after adding
the normalized publication guard. The native PR generation/restart/eviction check below now passes; broader corpus
acceptance remains open.

Full workspace gates pass 428 app tests (seven ignored), eight shared tests and
30 doctests; strict Clippy, formatting and workspace build pass.

Architecture comparison against `before-normalized-revision-retention` is neutral:
no new hotspots, cycles or clone groups. Logs:
`/tmp/radiology-normalized-retention-{tests,workspace,real,build}.log` and
`/tmp/radiology-normalized-retention-final-clippy.log`.

## Native acceptance — 2026-10-30

Backend `84d320c` was built and run in a private 1280×900 Weston/Niri session.
PR #7 uses local controlled gh/tool responses, a dirty attached checkout and four
real pinned Git commits with identical trees. Each head is refreshed and published
through the app. The first run invokes the controlled SCIP producer; subsequent
heads reuse its exact-input/version-qualified project record. Coverage remains
honest: one document and one missing source, with no cache warning. This is native
lifecycle evidence, not another real compiler acceptance claim.

A separate process holds the shared cache lease while the fourth head publishes.
All four normalized records and a QA active-stage stand-in remain intact. After
release, native Update removes the oldest normalized record and the stage, keeps
the current plus two recent heads, and preserves unknown notes. Reload Index and
restarting/reopening the review restore the current index without another tool
invocation. Explicit repository launch first opens checkout scope as designed;
the review is reopened through its command rather than relying on session restore.

Reopening the evicted first head shows no semantic index, with no checkout-import
leakage. Explicit Generate reuses the retained project data, restores that head's
normalized record and evicts the next oldest publication. The original generating
revision remains valid provenance for the independent project record even after
its earlier normalized copy was evicted. Final disk records are heads 1, 3 and 4.
Only the initial SCIP production run occurs; five later observed tool invocations
are version probes for project reuse. Reload/restart add none.

All four commits exist in the private object store and have the same Git tree.
Viewed checkout HEAD/status/all regular-file bytes remain unchanged, including
proof against the initial committed files and original dirty source. The private
producer sees exact pinned After bytes; its workspace and all build-input/stage
paths are removed. Both app instances, both compositors and the lease holder exit
zero. The sibling capture helper is used for QA only; the app's standalone runtime
and dependency boundaries are unchanged.

Evidence: `/tmp/radiology-normalized-retention-native/acceptance.json` and
`manifest.json` (SHA-256 of captures, logs, fixtures, drivers and the tested binary).
Key captures: `generated-after.png`, `head-4-live-writer-generated.png`,
`idle-update-pruned.png`, `cached-reload.png`, `reopened-review-cached.png`,
`evicted-head-no-index.png`, `evicted-head-regenerated-from-project.png` and
`final-accepted-reader.png`. The record-state JSON files preserve before/deferred/
after sets. No production source changes or fresh automated test rerun were needed
in this evidence increment. Broader corpus/recovery and final UI/accessibility/
scaling/font/sibling acceptance remain open.
