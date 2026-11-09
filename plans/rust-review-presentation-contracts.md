# Review Changes presentation contracts

Reference: all three functions in
[ReviewChangesPresentationTests.swift](../Tests/RadiologyTests/ReviewChangesPresentationTests.swift),
reviewed against Rust `5247cb6`.

## Gap and repair

A real app-frame regression failed in Before at 1×: the renamed row had no
selection stroke. The app stores the old source path while the Changes row uses
the new canonical path. The renderer compared these strings directly even though
the shared presentation already exposed revision-aware `selection_id`.

The app now supplies its accepted source revision to `Sidebar::show`. Changes
resolve the selected path through the already-filtered presentation before drawing
rows. This uses the existing model's Before/After availability rules, so a Before
rename selects the new canonical row, an unavailable side cannot select a row,
and a filtered-out change remains unselected. Files still receive the exact
source path; checkout Changes keep their path-based selection.

No stored selection state, reverse alias map or duplicate rename logic was added.
The reader, navigation history and canonical diff are unchanged by projection.
The existing file-query exclusion for Changes is preserved: source/test toggles
are shared, while the Files path query does not hide Changes rows.

## Assertion mapping

| Swift function | Rust evidence |
| --- | --- |
| `sharedFiltersKeepOrderAndOnlyVisibleTotals` | `presentation_filters_keep_order_visible_totals_revision_selection_and_distinct_empty_states` checks the exact six Swift paths in all four source/test toggle combinations, plus the rename case. Ordered paths, total canonical count, visible additions/deletions and count labels are explicit. Canonical input remains equal to its saved value. |
| `renameAcceptsEitherAvailablePathAndDeletionUsesBefore` | The same matrix uses Swift's three rename/deletion boundary examples, checks all four expected visible rows, and exercises old/new selection plus wrong-revision and filtered/unavailable sides. `changes_selection_follows_both_rename_revisions_without_moving_the_reader` opens a real pinned rename in each revision, checks the actual selected row's painted stroke at both scales, and preserves source Arc/history/path. |
| `filteredEmptyDiffersFromNoChangesAndResetRestoresSelection` | The model checks hidden README selection/totals, reset selection, unavailable Before and genuinely empty status. `changes_filter_reset_restores_visible_totals_without_reader_departure_and_empty_pr_has_no_reset` prepares an actual README-only PR and an equal-base/head empty PR. At both scales it checks visible and hidden counts/totals, distinct painted empty messages, clicks Show All Files, and verifies restored rows/default filters with identical accepted source Arc, history and canonical diff. A genuine empty PR does not offer the reset button. |

The app reset workflow also proves that a nonmatching Files path query leaves the
README Changes row visible. Existing result-row tests cover Space activation and
pressed-target retirement, while existing width/theme/scale tests cover bounded
labels and compact row geometry. Those tests run in the full workspace suite.
No new keyboard shortcut or typography scale is introduced by this selection fix.

## Host API

`Sidebar::show` gains `selected_revision: Option<ReviewRevision>` immediately after
`selected_path`. A host rendering pinned source supplies its accepted buffer's
revision; a checkout host supplies `None`. The canonical row mapping stays inside
the shared renderer and model, rather than requiring every host to reproduce it.
Rustdoc documents the new argument and warning-free API docs are verified.

## Verification

Archive: `~/.local/state/radiology-qa/review-presentation-20261109-5247cb6/`.

- `selection-red.log`: missing selected rename stroke at 1× Before.
- `selection-green.log`: both-scale Before/After app selection passes.
- `changes.log`: 20 matching tests pass, including the rendered filter/empty flow.
- `model.log`: final combined presentation matrix passes.
- `clippy.log`, `build.log`, `docs.log`: strict all-target workspace Clippy,
  normal build and warning-free docs pass; formatting passes.
- `workspace.log`: the full locked offline serial run passes 836 app tests/11
  existing ignored, two binary/13 font/22 text tests and all 64 doctests.

## Architecture

Baseline `before-review-presentation-20261109` to
`final-review-presentation-20261109`: +259 measured NLOC/three test functions/no
new files; average/max CCN remain 3.41/132, with 36 high-complexity functions.
All 122 clone instances and 1,317 duplicated lines remain unchanged, including
the full normalized file-pair/fragment multiset with multiplicities. One cycle
and eight structural issues remain unchanged. The new test initially duplicated
path-projection assertions; the final table drives both ordinary and renamed
fixtures through the same checks. The raw comparator still flags an unchanged
seven-line clone in the touched sidebar file; full multiset comparison proves
no new instance. New behavior reuses `selection_id`, and the app tests reuse the
Git fixture, review opener, actual frame/input helpers and source/history owners.
No dependencies, new workers, persisted state or sibling-app coupling were added.

All three presentation groups now have explicit mappings. Separate
PullRequestReview and ReviewFoldDefaults assertion audits, final native integration
acceptance, known parallel timing limits and Android remain open.
