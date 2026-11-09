# Integrated pinned browsing contracts

Reference: `PullRequestReviewTests.opensAPullRequestBrowsesAfterTogglesBeforeAndExits`
and the exact-caret portion of `changedBlockJumpsRevealTestsAndBackUsesCaretNotPassiveScroll`,
reviewed against Rust `325a169`.

## Reproduced gaps

1. Unchanged files incorrectly offered Before. A command regression entered a
   Before source load, and the painted button regression dispatched Before when
   no change record existed. `ReviewSnapshot::resolve` now rejects Before navigation
   for unchanged paths with an After-byte hint. Revision buttons use the same
   After-only interpretation when no change exists. Added/deleted/modified/renamed
   rules still use the existing `path_for_revision` model.
2. Revision switches and changed-block jumps used structural reading-anchor
   departures. The real scroll regression reported `(80, 0)` on Back instead of
   the actual caret `(1, 2)`. Both actions now use the existing exact-caret intent,
   consistent with semantic jumps. Ordinary file navigation retains its reading
   trail policy. No alternate history store or navigation mechanism was added.

The public navigation resolver's error contract now explicitly covers unchanged
Before requests. The low-level immutable object reader retains its base-object
capability for analysis; UI/host navigation goes through the resolver. Unavailable
revision commands remain no-ops. An explicit unavailable navigation reports the
error while retaining the accepted source/history and starting no read.

## Combined browsing proof

`pinned_browsing_keeps_snapshot_inventory_revision_history_rename_filters_search_and_exit_in_agreement`
uses a real private Git snapshot with Swift's `a`/`b`/`c` source body, an appended
unchanged scrolling tail, added/deleted files, unchanged Cargo manifest and an
edited Markdown rename. It runs at both scales using actual app frames.

| Swift browsing assertion | Rust evidence |
| --- | --- |
| Attached identity survives; head inventory is browsed | Source-only checkout rows include removed/old-rename paths; review rows instead include added/new-rename paths. Attached `repository_root` stays fixed, search root is the private head, and pinned base matches. Rust separates attached identity from the review source root rather than changing that identity field to a cache path. |
| Review starts in Changes with empty history/source | Combined workflow checks the accepted empty entry. The earlier fresh-entry regression also checks query reset, explicit launch file intent, unchanged metadata preservation and new-PR reset. |
| Revision availability for modified/added/deleted/unchanged paths | The combined matrix checks both sides of each, including After-only Cargo. Extended `unavailable_revision_commands_keep_the_reader_without_starting_a_failed_read` opens each natural side, tests explicit/current revision commands, resolver refusal and rejected explicit navigation with source/history retained. The painted revision-button matrix now includes no-change cases. |
| Exact After hunks and changed callable identities | After bytes match the head; highlights are exactly `[6..12]`; actual painted outline markers belong to `b` and `c`, with no marker for `a`, at both scales. Existing outline revision tests retain removed/edited/added/unchanged and checkout coverage through the shared paint helper. |
| Corresponding Before line, source-only semantics, exact Back caret | A real caret report and passive scroll to line 80 precede both Next Change and revision switch. Targets reach line 6; Before bytes/highlights match `[6..7]`, Before semantic availability is RevisionNotIndexed and definition is unavailable. Back restores each exact After caret/column and revision, not line 80. Existing indexed-source reference and pinned generation tests also prevent After semantic data from applying to Before. |
| Deleted and renamed natural sides/status | Deleted source reads exact Before bytes and reports Deleted. The edited rename resolves After's path to the old Markdown path for Before and returns its exact base text. Existing raw metadata/status-title and revision-choice matrices cover rename status/control details. |
| Filters preserve an excluded Before rename and canonical diff | While the renamed Markdown Before source is open, source-only/test filters exclude its row, but reset preserves the same source Arc, history and canonical changes. |
| A search hit forces After despite a remembered Before choice | Actual ripgrep searches the private head for a line absent from Before and the dirty checkout. Clicking the actual result opens the new path's After bytes at the exact match offset. |
| Exit restores checkout scope, inventory and recent entries | Exit returns Files with review/change/revision state cleared, retains the same attached root and recent summary, and restores the checkout's source inventory. HEAD, status and deliberately different attached source bytes remain unchanged across both-scale runs. |

The semantic data remains repository-owned in Rust while per-source availability
prevents Before queries. The audit maps user-facing availability and navigation,
not a requirement to delete the reusable After index when viewing Before.

## Reuse and architecture

The prior local painted-outline checker is now a shared test helper parameterized
by labels and scale. New and existing tests share it. The app-level single-result
search helper reuses the existing service completion waiter and is shared with the
Files reveal workflow. Scaled checkout context reuses the existing reader/open
helpers for the earlier fresh-entry and new browsing workflows. This removes two
draft clones instead of duplicating complete source/search/paint setup.

Baseline `before-pr-browsing-20261109` to `final-2-pr-browsing-20261109`:
+285 measured NLOC/three measured test functions/no new files. Average/max CCN
remain 3.41/132, with 36 high-complexity functions, one dependency cycle and eight
structural issues. The complete normalized clone multiset remains unchanged at
122 instances and 1,317 duplicated lines, including multiplicities. Raw flags
refer to an unchanged eight-line touched app-test clone and equal-score Python
ranking movement. Production changes reuse the resolver, revision model and
navigation intent; no new runtime function, worker, dependency or saved state.

## Verification

Archive: `~/.local/state/radiology-qa/pr-browsing-20261109-325a169/`.

- `unchanged-red.log`: unchanged Before command incorrectly starts a read.
- `buttons-red.log`: no-change Before button incorrectly dispatches.
- `caret-red.log`: reported caret plus actual passive scroll returns line 80
  instead of caret line 1/column 2. The earlier `browsing.log` used raw caret-field
  assignment; the final regression deliberately reports actual source interaction
  before scrolling, so the red proof does not depend on bypassing the observer.
- `browsing-final.log`: final both-scale integrated workflow passes.
- `reviews.log`: 44 review-related checks pass before the final scale-helper
  adjustment. Files reveal, prior outline and revision-button final checks pass
  in `files.log`, `outline.log` and `buttons-green.log`.
- `clippy.log`, `build.log`, `docs.log`: strict all-target workspace Clippy,
  normal build and warning-free docs pass. Formatting/diff checks pass.
- `workspace.log`: full locked offline serial run passes 842 app tests (11
  existing ignored), two binary/13 font/22 text tests and all 65 doctests.

Together with the metadata checkpoint, this maps two of four PullRequestReview
functions. The changed-block group benefits from the caret fix but still needs
its combined test-region/EOF/stale-input mapping. The retired-load group also
remains open, alongside other unreconciled suites, final native acceptance, known
parallel timing limits and Android. Do not infer overall parity completion from
this browsing workflow.

Follow-up: [changed-block contracts](rust-change-block-flow-contracts.md) map that
third group with hidden-test/EOF keys and actual source-result ownership checks.
The retired-review-load group remains open.
