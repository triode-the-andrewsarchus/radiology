# Review fold-default contracts

Reference: all seven annotated functions in
[ReviewFoldDefaultsTests.swift](../Tests/RadiologyTests/ReviewFoldDefaultsTests.swift),
reviewed against Rust `00cc20c`.

## Publication model

Rust prepares source bytes, structural syntax, exact per-file hunks and initial
folds together in `source::loader::prepare_loaded`, then publishes that bundle at
one source transaction boundary. The corresponding Swift tests exercise separately
arriving syntax and diff inputs. Rust satisfies their ordering/identity outcome
through captured immutable inputs and latest-request ownership. There is no
independent late-diff callback to replay over an accepted document.

`FoldStore::activate` restores only the exact path/revision/content identity.
`apply_prepared_defaults` accepts initial choices only for an uninitialized
document. Manual choices, including an entirely expanded set, survive a same-source
revisit. Review installation resets document memory at a new PR/snapshot boundary.
Appearance completion reuses the accepted source Arc and supplies no syntax or
initial folds, so recoloring cannot replay initial fold policy.

Bulk commands execute on invocation, rather than persisting Swift's view command
sequence counters. Revision navigation exposes its destination function. The new
app test deliberately navigates an unrelated tail row when checking restoration,
so explicit reveal does not hide a fold-memory failure or produce a false failure.

## Assertion mapping

| Swift function | Rust evidence |
| --- | --- |
| `signatureChangeOpensOwnBodyAndAncestorsButNotSiblingClosures` | New `review_signature_closure_and_added_body_changes_expand_only_their_owners` uses the Swift nested/multiline signature/two-closure/quiet-function fixture. Both revisions check all five callable folds. Signature changes open the nested body and ancestor only; closure changes open only that closure and ancestor. Added/deleted body cases use genuinely empty opposite sides and verify the closed sibling set and exact text. Existing `pr_annotations_and_body_changes_expand_only_their_owners_and_ancestors_once` directly checks ancestor range membership. |
| `attachedAttributeAndDecoratorOnlyEditsOwnTheFunctionNotItsClosures` | Strengthened `python_decorator_and_rust_attributes_belong_only_to_their_declarations` retains the Python decorator and attached Rust attribute/comment cases, adds an annotated Rust function with two anonymous closures, and checks every other callable body stays folded. Annotation ownership starts at the changed line. |
| `asynchronousOrdersAndManualChoicesApplyOnlyOnce` | The immutable loader bundle supplies syntax and changes together; `replacements_publish_only_current_source_and_report_read_failures` checks initial folds belong to the replacement source. Existing initial-default tests explicitly reapply after a manual recollapse and preserve it. New `review_fold_choices_survive_late_appearance_and_retired_reload_but_reset_for_new_scopes` holds an actual appearance result, changes folds through a bulk command, publishes it and checks the manual state, source Arc, caret, viewport and history at both scales. |
| `bulkCommandBaselineDoesNotReplayOldCommandsButNewCommandsOverride` | The new app workflow invokes CollapseFunctions on After while recoloring is held, switches to fresh Before defaults, invokes FoldToDefinitions there, then returns to After's remembered manual choices. Prior command invocations do not replay into the other revision. Existing callable/default and definitions-command tests exercise explicit changes, including container/variant behavior. |
| `identityAndRevisionGuardsAndLocalDefaults` | The new app test changes PR number with identical commits/bytes and checks fresh review defaults. Existing `refresh_tracks_rename_preserves_latest_caret_and_exact_pins_and_resets_old_fold_memory` verifies path/head replacement retires expanded choices; `fold_choices_restore_on_return_and_reload_but_reveal_destinations_and_reject_changed_bytes` checks exact-source reuse versus changed bytes/new root. The annotation/body matrix uses different Before and After hunks and checks checkout ignores PR folding policy. Loader replacement tests prove path-bound syntax/fold publication. |
| `emptySidesExpandContainingBlocksButNotClosedEOFNeighborsOrTestRegions` | Existing `empty_side_anchors_do_not_open_closed_neighbors_but_open_indentation_eof_is_owned` covers initial/interior/closed-boundary/EOF anchors and unterminated versus terminated Python indentation. New added/deleted nested fixture cases cover the corresponding nonempty side's body. `changed_test_functions_stay_hidden_by_the_independent_test_presentation` proves expanding a changed body leaves test-region hiding independently active. |
| `lateExpansionPreservesViewportAndNewerExplicitJumpWins` | Atomic preparation avoids a later accepted-source fold expansion. The new app workflow holds appearance completion while scrolled to a tail row and checks source/caret/history/top remain unchanged. It then holds a real reload containing prepared defaults, makes a newer same-document jump and forcibly submits that obsolete completion; the source Arc, newer caret/top/history and manual folds remain unchanged at both scales. Existing reading-position tests verify passive user scroll and navigation anchors, including latest viewport reports during held PR refresh. |

## Verification

Archive: `~/.local/state/radiology-qa/review-folds-20261109-00cc20c/`.

- `owners.log`, `fold-defaults.log`: initial nested/attribute checks pass.
- `lifecycle-final.log`: both-scale held-result/revision/new-PR workflow passes.
  Its earlier failure was a test expectation conflicting with deliberate explicit
  navigation reveal, corrected by selecting an unrelated tail destination.
- `folds-final.log`: 48 fold-related tests pass after the final empty-side matrix.
- `loader.log`: seven source-loader tests pass.
- `positions.log`: six reading-position tests pass.
- `revisions.log`: the existing revision-line-map workflow passes after adopting
  the shared scaled-review harness.
- `clippy.log`: strict all-target workspace Clippy passes; formatting passes.

This checkpoint changes tests and test-only fixture visibility/reuse. It does not
change runtime code or public APIs. The preceding `00cc20c` checkpoint's full
serial workspace run passed 836 app tests and 64 doctests; that result is historical,
not a claim that the two newly added tests ran in a fresh full-suite invocation.
The affected suites and all-target compile/lint checks provide current evidence.

## Architecture

Baseline `before-review-fold-contracts-20261109` to
`final-2-review-fold-contracts-20261109`: +113 measured NLOC/two test functions/no
new files; average/max CCN 3.41/132 and 36 high-complexity functions unchanged.
The complete normalized clone multiset remains unchanged at 122 instances and
1,317 duplicated lines; one dependency cycle and eight structural issues remain.
An initial duplicate setup was replaced by the existing `review_context` fixture,
now visible to sibling app tests and reused by both the new lifecycle workflow and
the revision mapping workflow. Equal-score raw chokepoint ranking changes are in
untouched Python files. No runtime mechanism, dependency or worker was introduced.

All seven fold-default groups now have explicit mappings, with ordering differences
explained above. The separate four-function PullRequestReview integration audit,
final native acceptance, known parallel timing limits and Android remain open.
