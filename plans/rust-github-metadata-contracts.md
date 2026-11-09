# GitHub summary metadata contracts

Reference: `PullRequestReviewTests.decodesGhSummariesAndTimesOutSlowCommands` and
`PullRequestSummary.jsonFields` in the Swift reference, reviewed against Rust
`0e00a4c`. This follows the explicit gap in
[fresh PR entry](rust-pr-review-entry.md).

## Retained data and shared requests

Rust's summary previously ignored `author`, `isCrossRepository` and `mergeCommit`.
The red roundtrip regression using Swift's open/merged fixture confirmed the
missing author. The model now retains:

- `author: Option<PullRequestAuthor>`, exposing the GitHub login;
- `is_cross_repository: bool`;
- `merge_commit: Option<PullRequestMergeCommit>`, exposing the reported object ID.

The two small metadata types are exported alongside `PullRequestSummary` for host
reuse. They mirror the fields consumed by Swift, including nullable author/merge
objects. Extra GitHub author properties remain outside the typed model. Serde
rejects malformed field types and incomplete non-null nested objects. Missing
fields preserve older serialized records with None/false defaults. Timestamp
strings retain their exact GitHub value; this change adds no date library.

List and view now use one private complete JSON-field constant, matching Swift's
request set. List also requests the base/head IDs that its previous reduced field
set omitted. Both operations use the existing bounded child runner, cancellation,
deadline, output validation and disabled interactive prompting. There is no second
parser, process adapter or metadata request.

## Assertion mapping

The first of the four PullRequestReviewTests groups is now explicitly mapped:

| Swift assertion | Rust evidence |
| --- | --- |
| Open/merged numbers, state title and merged classification | `summary_metadata_roundtrips_open_merged_and_legacy_records` uses Swift's two summary examples and checks numbers, Open/is_open and is_merged. |
| Nullable author/merge object, login, cross-repository flag and merge ID | The same test checks typed access and serialized values, full roundtrip equality, legacy missing fields, and six malformed nested/flag cases. |
| Full SHA accepted, abbreviated SHA rejected | The fixture's head ID and abbreviation are checked through the shared object-ID predicate also used by pinned preparation. Existing SHA-1/SHA-256 raw/preparation matrices remain applicable. |
| Bounded slow command fails promptly | Existing `github_metadata_children_cancel_timeout_and_recover_without_accepting_bad_output` calls `interrupt_gh`, whose actual child sleeps 30 seconds. It exercises explicit cancellation and a two-second deadline, checks prompt interruption/reaping within three seconds, then verifies recovery, malformed/failed/oversized output. This proves the same timeout outcome with Rust's captured shared control. |

`list_and_view_request_complete_metadata_through_the_shared_child_runner` executes
an actual private shell child for list, open view and merged view. It records argv,
checks operation/number, checks the complete eleven-field JSON set and verifies
received typed summaries. It also checks the runner's prompt-disabled environment.
It does not use a live GitHub account or claim remote authentication coverage.

The existing both-scale `metadata_refresh_coalesces_commands_and_preserves_reader_fact_owners`
now changes title/state and all new fields, then compares complete summaries in
the active PR, review snapshot and recent list. It retains the accepted source,
outline/native/call/index Arcs, pins, caret and history. No new refresh/remember
mechanism was necessary; those owners already clone the complete shared model.

## Host migration

Rust struct literals must now supply `author`, `is_cross_repository` and
`merge_commit`. Deserialization of old records remains compatible through defaults.
The lone in-tree manual fixture was replaced by the existing summary factory.
Hosts may import `PullRequestAuthor` and `PullRequestMergeCommit` directly from
`radiology::pull_request`; the summary rustdoc includes a runnable serde example.
Generated docs expose all three field anchors and build without warnings.

## Verification

Archive: `~/.local/state/radiology-qa/gh-metadata-20261109-0e00a4c/`.

- `metadata-red.log`: existing model loses the fixture's `author.login`.
- `pull-request.log`: 44 focused tests pass before the final shared SHA assertion.
- `refresh.log`: extended both-scale metadata refresh passes.
- `clippy.log`, `build.log`, `docs.log`: strict all-target workspace Clippy,
  normal build and warning-free API docs pass; formatting passes.
- `workspace.log`: full locked offline serial run passes 841 app tests (11
  existing ignored), two binary/13 font/22 text tests and all 65 doctests.

Architecture baseline `before-gh-metadata-20261109` to
`final-gh-metadata-20261109`: +140 measured NLOC/two test functions/no new files.
Average/max CCN remain 3.41/132, with 36 high-complexity functions, one dependency
cycle and eight structural issues. The complete normalized clone multiset is
unchanged at 122 instances and 1,317 duplicated lines. Raw flags refer to an
existing eight-line clone in touched app tests and equal-score untouched Python
chokepoint ranking movement. Typed metadata adds no runtime functions; list/view
reuse the field constant and child runner, while refresh reuses existing ownership.
No dependency, worker or new persisted store was added.

This maps one of four integrated PullRequestReview groups. The browsing,
changed-block and retired-load combined mappings remain open, together with final
native acceptance, other unreconciled suites, known parallel timing limits and
Android. The fresh-entry gap is repaired in the linked checkpoint, and the separate
repository/presentation/fold suites retain their explicit mappings.
