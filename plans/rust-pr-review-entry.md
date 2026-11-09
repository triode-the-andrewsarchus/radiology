# Fresh PR review entry

Reference: `PullRequestReviewTests.opensAPullRequestBrowsesAfterTogglesBeforeAndExits`
and `AppModel.beginOpening(root:review:)` in the Swift reference, reviewed against
Rust `f5c7c5d`.

## Gaps and behavior

A rendered regression failed because Rust automatically opened the checkout's
previous file (or a first changed/inventory file) and populated review history.
Swift enters Changes with source, selection and history cleared, waiting for an
explicit file choice. A second regression showed the checkout's Files query
survived entry, while Swift also clears that query.

The fresh-review branch of `accept_review` now installs the immutable snapshot,
clears the Files query, and opens source only when an explicit pending file intent
exists. It reuses the existing installation reset for source, history, syntax,
folds, reader positions, pins and Changed-line state. The public `active_revision`
field is `None` until source is selected, rather than prepopulating After. Source/test preference toggles
remain intact. The existing No File Selected view supplies sidebar/Ctrl+K guidance.
No new empty-state UI or navigation state was introduced.

Selecting a changed file then creates the first review-local history entry, without
a stale checkout departure. Explicit launch/restoration file intent still resolves
against the review (including a deleted file's natural Before side), beginning at
that file's start rather than inheriting an unrelated checkout byte.

Same-number/same-commit metadata refresh retains the accepted reader/history/query.
Changed-head/base refresh keeps the existing prepared restoration transaction.
Changing PR number enters a fresh Changes view even for identical commits. Failed
preparation continues to preserve the old reader until successful entry; a retry
that opens a new PR now lands in this empty review, while a same-PR retry preserves
its accepted source. Existing recovery tests now distinguish these outcomes.

## Evidence

Archive: `~/.local/state/radiology-qa/pr-review-entry-20261109-f5c7c5d/`.

- `entry-red.log`: auto-open violates the fresh review's empty-source assertion.
- `query-red.log`: checkout Files query incorrectly survives review entry.
- `entry-final.log`: `fresh_review_entry_waits_for_file_choice_and_explicit_destinations_still_open`
  passes at both scales. It begins with a checkout reader and Files query, verifies
  empty source/history/selection/change state and retained source-only preference,
  clicks the actual Changes row, checks exact pinned bytes and first-history
  semantics, then exercises metadata refresh, another PR and explicit deleted-file
  intent. The painted empty-reader guidance is checked too.
- `reviews.log`: 12 review app tests pass before the final query adjustment.
- `review-flows.log`, `bookmarks.log`: 44 review-related and 39 bookmark tests pass
  before that adjustment, including private real CLI retry/recovery workflows.
- `clippy-final.log`, `build-final.log`: strict all-target workspace Clippy and
  normal build pass after the final change; formatting passes.
- `workspace.log`: full locked offline serial run passes 839 app tests (11
  existing ignored), two binary/13 font/22 text tests and all 64 doctests. This run
  precedes the final one-line empty-source revision ownership adjustment and its
  assertion; `review-final.log` reruns the affected review checks afterward.
  Build, strict Clippy and warning-free API docs are rerun for that adjustment.

Older tests that need a reader now make an explicit file choice after review
acceptance. This preserves their semantic/fold/bookmark assertions while matching
the new entry behavior. Shared `open_review` helpers already carry explicit file
intent. No public Rust signature, dependency, worker or persisted schema changed.

## Architecture

Baseline `before-pr-review-contracts-20261109` to
`final-3-pr-review-entry-20261109`: +66 measured NLOC/one test function/no new files.
The production path removes checkout-caret and first-file fallbacks, replacing
them with the existing explicit-intent path. Average/max CCN remain 3.41/132, with
36 high-complexity functions, one dependency cycle and eight structural issues.
All 122 clone instances/1,317 duplicated lines and the full normalized file-pair/
fragment multiset with multiplicity remain unchanged. The raw comparator flags
an existing eight-line clone in touched app tests; there is no new clone instance.

## Remaining integrated review audit

This repairs the fresh-entry and query assertions in the second of four Swift
PullRequestReviewTests functions. It does not declare that function or the whole
suite reconciled. Existing repository, revision, metadata, change-navigation,
refresh/recovery and native PR documents provide substantial evidence for the
remaining assertions, but need an explicit combined mapping.

The first function also expects richer GitHub metadata (author, cross-repository
flag and merge-commit ID) that Rust currently discards. Reconcile that model/client
contract next, then complete the combined browsing/changed-block/retired-request
mapping. Final native acceptance, other unreconciled suites, known parallel timing
limits and Android remain open. The earlier all-seven fold and all-three Changes
presentation mappings remain valid with the updated explicit source choice in
scope-reset tests.

Follow-up: [GitHub metadata contracts](rust-github-metadata-contracts.md) retain
those discarded fields and map the first integrated test group. The three combined
flow mappings remain open.
