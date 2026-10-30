# PR preparation cancellation and deadlines

Implementation: `a651a8b`, pushed on 2026-10-30.

## Ownership and subprocesses

Review preparation uses the shared LatestRead owner. Explicit Cancel, replacement,
repository opening, reset and app drop retire publication and signal the worker's
flag. The accepted reader, history, semantic data and review remain in place while
preparation runs or fails. Cancel and failure discard pending launch-file intent.
A worker uses one two-minute deadline for GitHub metadata, private-store setup,
fetches, merge-base lookup, materialization, file enumeration and patch collection.
Queued work checks retirement before starting. Local filesystem work/cleanup may
finish its current operation before the next cooperative check.

GitHub and Git use the existing POSIX process runner, which kills the child group
and reaps its direct child on interruption, I/O failure or unwind. New run_collect
reuses its observer to retain complete stdout up to 16 MiB while diagnostic capture
remains independently bounded. GitRead reuses this collector instead of maintaining
another copy. Oversized output fails rather than publishing a truncated result.
Nonzero required Git/GitHub commands retain bounded stderr diagnostics. GitHub
requests disable interactive prompts; availability probes have five-second budgets.
Existing public synchronous APIs remain available with bounded default deadlines.

Only a completed nonzero command qualifies for fetch fallback. Cancellation,
timeout, spawn, output-limit and I/O errors propagate immediately. Enumeration
and change traversal check the same flag/deadline, followed by a final publication
check. GitHub JSON parsing has a post-parse check and bounded input.

## Snapshot materialization

The private bare store initializes idempotently, allowing retry after interrupted
initialization. Its canonical absolute path also supports relative cache arguments.
Each attempt creates an independently owned temporary worktree under its pinned
head directory. It becomes persistent only after files, patches and the final
control check succeed. Failure removes that attempt's tree; it cannot remove an
accepted snapshot, including another tree for the same head. Later preparation
runs worktree prune before materialization to clear eligible abandoned registrations.
The attached checkout is never switched or used for review source bytes.

## Automated verification

Four added tests bring the suite to 357 app tests passing (one external-Clang
fixture ignored), eight shared reader-text tests and 30 doctests. Formatting,
strict Clippy, build and rustdoc pass.

The runtime cases observe a live PID before cancellation/timeout for GitHub
metadata, fetch, worktree materialization and patch loading. They require ESRCH
after the request returns, reject fallback after interruption, verify removal of
temporary trees and unchanged accepted/checkout bytes, then prepare and read a
fresh snapshot successfully. Metadata cases also check fresh, malformed, nonzero
and oversized responses. Shared collector tests preserve bytes beyond diagnostic
truncation and reject excess output. Model tests prove cancelled jobs cannot
publish errors and failures retain the reader/history/session while clearing intent.

Commands:

```sh
cargo test --workspace
cargo test --lib pull_request::review::cancellation_tests -- --nocapture
cargo test --lib ui::app::tests::reviews -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
cargo fmt --all --check
```

Logs: `/tmp/radiology-pr-process-{runtime,all-tests,clippy,build,doc}.log`.

## Native verification

A private 1280×900 Wayland session opens a dirty checkout with accepted source and
semantic data. The existing PR prompt starts a controlled local gh child. Cancel
retains the reader and reaps PID 471356. A second request pauses fetch; its live
PID 473098 is observed before Cancel and is reaped afterward. The reader, checkout
branch and semantic data remain displayed. A fresh request opens PR #7 After from
its pinned head, showing the second helper call absent from the checkout.
Cancelling a refresh reaps metadata PID 474022 and retains that accepted PR reader.
Checkout HEAD, dirty status and exact source bytes compare unchanged. The app,
Weston and Niri exit zero; pointer-helper termination is intentional.

Artifacts: `/tmp/radiology-pr-process-native/manifest.json`, `initial-reader.png`,
`metadata-pending.png`, `metadata-cancelled.png`, `fetch-pending.png`,
`fetch-cancelled.png`, `review-recovered.png`, `review-refresh-pending.png`,
`review-refresh-cancelled.png` and `child-results.json`. The manifest hashes captures,
fixture/driver sources, checkout evidence, runtime/full-test logs and both binaries.
Tested app SHA-256:
`4a6e5f0f61c1374423d6d70b8c69deaa9f648a0f395ef87702c07c9abd4dd2c0`.
Driver: `/tmp/radiology-pr-process-native-driver.py`; fixture:
`/tmp/radiology-pr-process-native-fixture.py`. Configuration and cache directories
are isolated. Sibling capture helpers remain QA tooling, with no app dependency.
This uses local Git and controlled gh responses, without network PR mutation.
Native cancellation/recovery and backend timeout tests provide separate evidence;
no native two-minute timeout run or exact UI response latency is claimed.

## Architecture and remaining scope

Baseline: `before-pr-process-cancellation`. The comparison labels a regression
because adding a test-module declaration touches a large existing test file.
All four flagged clone fragments are byte-identical to the baseline, including
coordinates 124/1614, 219/1185, 909/986 and 1679/1717 in src/ui/app/tests.rs.
Clone groups and dependency cycles do not grow. The renamed preparation entity
retains a pre-existing Valknut warning: Lizard CCN improves 15→14, cognitive 27→26;
Valknut's own CCN changes 13→14 with explicit cancellation checks. Shared collection
removes duplicated runner-consumer logic. No new accidental structural regression
is evidenced by the flagged findings.

[Private-store writer ownership](rust-pr-store-recovery.md) now serializes preparation
and surviving children, stages initialization and handles tested legacy/missing-tree
recovery. Complete cache retention/eviction and broader corrupt-store recovery remain open. Worktree pruning is not a
promise to recover every interrupted Git metadata/lock state. Successful repeated
preparation currently retains independent trees, including unchanged-head refreshes;
reusing proven complete trees belongs with cache provenance/retention work.
The [recent PR chooser](rust-pr-chooser.md) is now integrated and accepted natively.
Pinned After indexing, exact refresh caret/fold/pin/rename
mapping, notice/retry behavior and final UI/scaling/accessibility acceptance remain
on the [full parity queue](rust-parity-implementation.md).
