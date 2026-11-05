# Saved session recovery — 2026-11-05

Rechecked all seven `ReaderSessionTests.swift` contracts against the Rust app.
Two red regressions showed that Rust discarded a temporarily unavailable root and
persisted Local Checkout over a confirmed PR during startup preparation.

Session validation now checks structure, leaving availability to asynchronous
discovery and its existing Retry/Dismiss UI. The existing typed repository request
carries whether its PR is a saved restoration; checkout publication retains that
confirmed number until an explicit local choice or accepted review replaces it.
Startup runs once and yields to existing repository work. The PR prompt uses the
accepted review, then the saved PR for the same attached root, then recent reviews.
There is no additional session store or failed-number cache.

## Swift contract reconciliation

| Swift assertion group | Rust evidence |
| --- | --- |
| Relaunch attached repository and confirmed PR | `saved_review_identity_survives_checkout_preparation_failure_cancel_and_fresh_restore`; native no-argument restart, fresh metadata call and exact After source |
| Exit PR remembers local checkout | Same app regression exits through `ExitPRReview`, loads a new session/app and verifies no pending or accepted PR |
| Explicit launch arguments win | Existing `explicit_launch_root_supersedes_saved_review_without_starting_it`; native initial explicit root/PR/file; existing review publication/navigation tests |
| Failed restoration supports retry | New app regression retains disk PR through preparation/failure/cancel, prefills the prompt, retries through successful review publication; native controlled offline failure and keyboard retry |
| Failed/canceled PR choices retain confirmed session | Same regression accepts 758, fails 999 and cancels 888 while preserving accepted review and on-disk 758 |
| Newer repository choice wins; launch runs once | Updated gated discovery regression retires the startup root/PR intent and verifies a repeated launch cannot replace the newer reader |
| Missing repository and malformed preferences recover | New missing-root regression retains root/PR, displays discovery failure, accepts another root; broken JSON, relative path and PR zero produce an empty session |

App tests are in `src/ui/app/tests/repository_open.rs`; older explicit launch and
exit/navigation evidence remains in `src/ui/app/tests.rs`. The native fixture uses
real Git commits/worktrees and controlled GitHub metadata/failure, without a live
GitHub service. Native 1×/2× runs each verify four exact clipboard payloads including
the retry input, unchanged attached files/HEAD/refs/status and binary hash, and zero
child exit statuses. Screenshots confirm readable error and retry presentation.

## Verification and architecture

Locked workspace: 779 app tests pass (11 opt-in ignored), two binary tests,
13 font tests, 22 text tests and 60 doctests. Strict Clippy, rustdoc, formatting,
binary build and bundled-resource smoke pass with the existing 16-job Cargo limit.

Baseline `before-session-restore-recovery`: +144 NLOC and two test functions;
average/max CCN remain 3.39/132, duplication stays at 125 instances/1335 lines,
with no new hotspot, cycle or structural issue. A repeated invalid-payload test
sequence was consolidated into one payload loop. Final comparison is neutral;
equal-score ranking churn between two untouched Python chokepoints is unrelated.

Evidence: `~/.local/state/radiology-qa/session-recovery-20261105-3bd3e69/` contains
red/green/full-check logs, native driver and acceptance scripts, captures, payloads,
fixture/binary hashes and architecture reports. Build/working caches stay under
`~/.cache/radiology-qa-session-recovery/`, outside `/tmp`.

This closes the session contracts above. Remaining Swift assertion reconciliation
and SDK-backed Android success remain open; actual sibling adoption and full
accessibility are deferred. Reader-host APIs and dependency boundaries are unchanged
from the independent-host audit; runtime changes here have fresh native acceptance.
