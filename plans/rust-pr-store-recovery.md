# Private PR-store writer ownership and recovery

Implementation `2fe0b11`, pushed on 2026-10-30.

## Writer ownership

Every snapshot preparation acquires one advisory lease on the stable
review-store.lock file beside the private bare repository. Waiting checks the
request's existing flag and total deadline every bounded interval. Cancellation
or timeout releases the wait without mutating the store. Closing a guard releases
its descriptor; the lock file is never deleted or explicitly unlocked. Its stable
inode lets separate app processes serialize against the same store.

Preparation Git children explicitly inherit the locked descriptor. The shared
runner clears CLOEXEC only inside the child's pre-exec hook using fcntl; parent
flags remain unchanged. Other commands retain the normal CLOEXEC policy and public
process APIs retain their signatures. A surviving Git child therefore keeps the
lease after its parent exits. A new preparation waits until all inherited holders
close it, instead of deleting a lock beneath a live writer. Normal cancellation
still kills the owning process group and reaps its direct child.

The lease covers initialization, fetch, merge-base lookup, worktree registration,
enumeration and patches through successful snapshot assembly. Pinned read-only
source requests remain independent and can keep the accepted reader usable.
Git-store operations now live in a cohesive review/store.rs module; ownership and
cancellable acquisition live in review/lease.rs.

## Avoiding shared Git lock conflicts

A new bare repository initializes in a unique review-init temporary directory,
then publishes by rename only after successful Git completion and a control check.
A valid existing bare store is validated and reused without another git init.
A legacy directory missing HEAD is preserved under a review-recovery quarantine
before a new complete store is published. Symlink/non-directory stores fail without
being followed. Existing directories with HEAD but invalid Git metadata report an
error rather than being discarded.

Fetch disables FETCH_HEAD writes, automatic maintenance and commit-graph writes.
Known commits must be commit objects. If a direct SHA fetch finishes nonzero,
fallback fetch targets a unique refs/radiology/requests ref. The exact fetched
commit is verified against pinned metadata and the temporary ref is removed.
A moved ref fails even if it brought other useful objects. Cancellation, timeout
and I/O failures still abort immediately. Legacy config.lock, FETCH_HEAD.lock and
FETCH_HEAD contents can remain untouched while normal local fetch proceeds.

Before creating another tree, Git-store recovery inspects registered worktree paths.
Only a missing path under this cache's revisions/<valid-sha>/tree-<suffix> namespace
qualifies for forced removal of an abandoned registration, including a locked one.
Present paths, symlinks and foreign paths are retained. Normal worktree pruning
follows. No unknown global Git lock file is deleted.

## Automated evidence

Six added tests bring the full suite to 371 app tests passing (one external-Clang
fixture ignored), eight shared reader-text tests and 30 doctests. Formatting,
strict Clippy, build and rustdoc pass.

Tests verify cancellable/deadline-bounded lease contention, stable lock inode on
release/reacquisition, child-only descriptor inheritance with parent CLOEXEC
preserved, serialized replacement after a live materializer, locked missing-tree
repair, and unchanged accepted pinned bytes. Real local fetch succeeds with legacy
config/FETCH_HEAD locks and their exact bytes preserved. Additional cases preserve
legacy initialization data in quarantine and validate request-ref fallback, moved
heads and the absence of FETCH_HEAD publication. Existing interruption tests now
cover initialization as well as fetch, worktree creation and patches.

Commands:

```sh
cargo test --workspace
cargo test --lib recovery_tests -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
cargo fmt --all --check
```

Logs: `/tmp/radiology-pr-store-{recovery-tests,runtime,all-tests,clippy,build,doc}.log`.
Architecture baseline `before-pr-store-recovery` compares neutral with no new
problems, clone groups or cycles. Health changes 78.8→79.2. Store stages remain
below the function complexity threshold, and the shared runner preserves ordinary
callers without a second process engine or duplicated stdout collection.

## Native parent-crash acceptance

A private 1280×900 Wayland session opens the dirty checkout and starts PR #7.
A controlled materialization child, PID 600354, owns descriptor 24 pointing at the
private review-store.lock. The harness deliberately kills app PID 600060 with
SIGKILL. The child remains alive with that inherited descriptor.

App PID 600452 opens the same repository and requests #7. Its reader remains
available while preparation waits. A separate process receives WouldBlock when
trying the lease, and the worktree-attempt log still contains only the old writer:
no second materialization overtakes it. Explicit Cancel retires only the waiting
request and retains the reader; the surviving writer stays alive.

The harness then kills and reaps that owned orphan using a subreaper. A fresh
request acquires the released lease and displays PR #7 After from pinned bytes.
The second materialization is recorded. Checkout HEAD, dirty status and source
bytes compare unchanged. The reopened app, Weston and Niri exit zero; the first
app's -9 status is the intentional crash. Pointer-helper termination is intentional.
The old crash tree remains on disk: present-tree reuse/retention is separate work,
and this sequence does not claim it was evicted.

Artifacts under `/tmp/radiology-pr-store-native/`: `manifest.json`,
`child-lease-before-crash.json`, `child-survived-crash.json`, `waiting-evidence.json`,
`cancel-isolation.json`, `writer-retirement.json`, `recovery-result.json`,
`final-worktrees.txt`, and screenshots `initial-reader.png`,
`old-writer-pending.png`, `new-writer-waiting.png`, `waiting-cancelled-reader.png`,
`recovered-review.png`. The manifest hashes these, checkout evidence, runtime/full
logs, fixture/driver sources and the tested binary. App SHA-256:
`2b882958ec169917b955125dd21105a55378067eba65621ccbce7dc46426fe33`.
Driver: `/tmp/radiology-pr-store-native-driver.py`; fixture:
`/tmp/radiology-pr-store-native-fixture.py`. Configuration/cache are isolated;
sibling helpers supply QA capture only. The app remains independently built.
Temporary artifacts are local evidence. No network PR writes occur.

## Remaining limits

The lease serializes cooperating clients of this private cache; manual Git commands
or older clients that ignore it are outside that ownership protocol. Tested legacy
lock recovery concerns config/FETCH_HEAD and missing owned worktrees, not every
possible corrupt Git metadata or lock state. Existing invalid-HEAD stores fail
safely; HEAD-less quarantine is preserved and not yet bounded by eviction policy.
Filesystem work and cleanup are cooperative rather than hard real-time operations.

Present complete and crash-left trees remain retained. Repeated unchanged-head
preparation still creates an independent tree. Proven tree reuse, active-snapshot
leases and bounded eviction remain the next cache requirements, alongside broader
index provenance/provisioning/pinned After indexing and PR refresh restoration.
The [full parity queue](rust-parity-implementation.md) remains open.
