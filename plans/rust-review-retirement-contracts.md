# Integrated review failure and supersession contracts

Reference: `PullRequestReviewTests.errorsAndSupersededLoadsNeverPublishStaleReviews`,
reviewed against Rust `85cf94c`. This completes the four-function integrated review
mapping alongside [metadata](rust-github-metadata-contracts.md),
[browsing](rust-pr-browsing-contracts.md) and
[changed-block navigation](rust-change-block-flow-contracts.md).

## Combined request lifecycle

The existing isolated CLI recovery test now invokes `run_cli_supersession` before
its recovery driver. It uses real Git snapshots and the actual app review commands,
with a private `gh` executable, PATH, cache/config and settings. No parent process
environment or user GitHub configuration changes. Existing snapshot, checkout
context, frame/click, self-relaunch and child-start helpers are reused.

| Swift requirement | Rust evidence |
| --- | --- |
| Failed request identifies its number and keeps checkout | Opening 999 with a generic offline diagnostic preserves checkout; the notice names 999. The red test exposed the missing number. The shared failure recorder now prefixes `PR #999:` while retaining operation and complete underlying error. |
| Retry clears error and repeats the same request | A painted Try Again click fails again; exactly two view requests have occurred. A third retry reaches a held live gh child with pending number 999 and no failure. |
| Exit a pending retry without losing the reader/history | Actual Exit command cancels the held request; source Arc and history equal the departure, no review/error/pending request survives. The preceding checkout-exit checkpoint also proves saved restoration identity is cleared while generic Cancel retains it. |
| Repository replacement retires the initial review load | A held PR 1 request is replaced by opening a non-Git folder containing a.txt. The accepted root is that folder, no review/error lands, and actual rendered frames poll the list until Unavailable. Source/repository settling alone does not await the independent recent-list worker. |
| Newer PR number wins | While another PR 1 child is live, PR 2 is requested through `open_pull_request`. The full accepted summary is PR 2's, with no error. |
| Reload stays in the accepted review/head | After selecting pinned source, the normal Refresh command requests PR 2 again and retains its source Arc/history and head browsing root. |

Both 1× and 2× runs require the exact view-number sequence
`999, 999, 999, 1, 1, 2, 2`. The three held children exec sleep directly;
the test waits for their actual PIDs before transitions and proves they have
been reaped afterwards. It does not substitute a fixed delay for request startup.
The fake CLI supplies deterministic summaries/errors; no live GitHub request is
claimed. Error 999 models a failed lookup rather than asserting a particular
GitHub server message.

The existing both-scale `retired_refresh_results_cannot_reopen_cancelled_exited_or_replaced_review_scopes`
also forces completed successful snapshot publication after retirement, verifying
both cancellation and closed receivers. The new real CLI flow exercises earlier
metadata cancellation; together these cover the distinct ownership boundaries.

## Implementation and verification

Runtime change: shared `record_review_failure` adds its already owned PR number to
the diagnostic. No new production functions, workers, public types or dependencies.
The exact operation-message test now expects the prefix for both open and refresh.
The checkout-exit implementation is recorded in
[recovery notes](rust-review-recovery.md#checkout-exit-during-preparation--2026-11-09).

Archive: `~/.local/state/radiology-qa/review-supersession-20261109-85cf94c/`.
`red.log` records the missing-number assertion, `cli.log` the passing both-scale
combined and prior recovery flows, and `architecture.json` the neutral comparison.
The full serial workspace run passes 844 app tests (11 existing ignored), two
binary, 13 font and 22 text tests plus all 65 doctests. Strict all-target Clippy, build, warning-free API
docs, formatting and diff checks pass. Architecture adds 113 measured NLOC and one
test driver, no files or clone groups;
average/max complexity stays 3.41/132 with 36 existing hotspots. The reported
exchange of equal-score chokepoints concerns untouched Python files.

This maps all four ordinary PullRequestReviewTests functions; it does not establish
whole-app parity. Other unreconciled suites, final native acceptance, recorded
parallel timing limits and Android remain open. No fresh compositor or Swift/AppKit
run is claimed. Keyboard remains required; full accessibility is deferred.
