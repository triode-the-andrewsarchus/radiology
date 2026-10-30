# Rust PR tree reuse and retention

The Rust app reuses private PR worktrees only after verifying their exact pinned
Git contents. This extends the [private store recovery contract](rust-pr-store-recovery.md).
The viewed repository is never the cache or the materialization destination.

## Ownership and lifetime

Each generated tree has an external format-1 metadata record and a shared advisory
reader lease. A published review snapshot retains that lease; cloning the snapshot
keeps it alive. Consumers must retain the snapshot while using its head-root path.
Preparation and eviction hold the private store writer lease. Eviction also needs
an exclusive tree lease, protecting readers across processes.

The retention target is three managed trees. Active readers may exceed that target.
After successful preparation, idle trees are evicted in least-recently-used order.
Dropping the final snapshot schedules bounded, nonblocking-writer maintenance;
busy stores and cleanup failures defer work until a later operation. This is a
best-effort target rather than an immediate hard bound. Unknown, malformed and
unmanaged legacy entries are preserved, not adopted or counted as safe eviction
candidates. Failed new preparations clean up their own temporary tree and metadata.

## Exact reuse proof

Verification checks the real tree root, private Git common directory, pinned HEAD,
and every tracked file against raw Git tree entries. Shared repository identity
code hashes file or symlink bytes and checks executable modes, supporting SHA-1
and SHA-256 Git blob IDs. Missing files, extra entries, changed modes and symlink
substitutions reject reuse even when Git status or index flags hide the change.
Gitlinks accept an empty placeholder directory, not initialized submodule contents.
Checks honor cancellation and deadlines throughout walking and hashing.

Private checkout attributes disable text/EOL conversion, filters, working-tree
encoding and ident expansion. Checkout also disables hooks and automatic CRLF
conversion; the newly materialized tree is verified before publication. This
policy applies only to the private store. Fresh stores select the pinned commit's
object format; existing stores are not silently migrated between object formats.

## Verification and remaining acceptance

Automated gates passed on 2026-10-30: 377 app tests (one ignored), eight shared
reader tests, 30 doctests, formatting, strict Clippy, build and rustdoc. Six cache
tests cover exact reuse, concealed modifications, active leases and LRU retention,
mode/symlink/extra/missing/HEAD changes, malformed metadata and abandoned staging,
checkout conversions/hooks, and SHA-256 Git objects. Existing cancellation and
store recovery tests remain green.

Architecture comparison reports a regression for an unchanged baseline ten-line
critical-test fragment because its file received a synthetic snapshot lease field.
The fragment itself is unchanged; clone groups remain 90, duplication falls from
0.61 to 0.60 percent, and no new complexity hotspot or dependency cycle appears.

## Native acceptance

A private 1280×900 Wayland session opened six distinct local PR revisions. Opening
#7 twice incremented its use record without another materialization. A separate
process held #7's reader lease while the app moved through #8–#11: #8 and #9 were
evicted, leaving #7, #10 and #11. Releasing that external lease and opening #12
evicted #7 and retained #10–#12. Each observed settled cache held at most three trees.

The active #12 tree was modified after setting Git assume-unchanged; diff-files
reported clean. Reopening #12 rejected it and produced a separate verified tree
with the committed bytes. Restarting and opening #12 reused that replacement:
seven materializations total for six revisions plus the corruption replacement.
Checkout HEAD, dirty status and source bytes remained unchanged. Both app runs and
both compositors exited zero. Screenshots show the pinned After reader working.

Hashed artifacts: `/tmp/radiology-pr-tree-cache-native-accepted/manifest.json`,
stage JSON records, `materializations.jsonl`, `child-results.json`, and
`pr-seven.png`, `eleven-held.png`, `twelve-repaired.png`, `reopened-twelve.png`.
Driver and fixture scripts use sibling capture helpers for QA only; the app has no
sibling runtime dependency. Cross-process protection is exercised by an independent
lease holder, not a second graphical app. Active-reader overflow beyond three is
covered by automated tests. This does not establish final UI parity. Remaining broader work includes indexing provenance
and provisioning, pinned PR indexing, precise refresh restoration, corrupt-store
recovery, and visual/accessibility/scaling/font and sibling reuse acceptance.
