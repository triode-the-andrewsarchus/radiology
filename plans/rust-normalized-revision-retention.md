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
the normalized publication guard. A final native PR generation/restart/eviction
check and broader corpus acceptance remain open.

Full workspace gates pass 428 app tests (seven ignored), eight shared tests and
30 doctests; strict Clippy, formatting and workspace build pass.

Architecture comparison against `before-normalized-revision-retention` is neutral:
no new hotspots, cycles or clone groups. Logs:
`/tmp/radiology-normalized-retention-{tests,workspace,real,build}.log` and
`/tmp/radiology-normalized-retention-final-clippy.log`.
