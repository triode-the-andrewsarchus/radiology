# Integrated changed-block navigation contracts

Reference: `PullRequestReviewTests.changedBlockJumpsRevealTestsAndBackUsesCaretNotPassiveScroll`,
reviewed against Rust `8d29a34`.

## Keyboard and presentation proof

New `pinned_change_keys_reveal_tests_keep_exact_departures_and_reach_only_the_eof_anchor`
uses the existing real Git snapshot fixture, scaled review context, actual source
focus and input/frame helpers. The source follows Swift's production function,
hidden attributed test function and final deletion. Its unchanged tail is longer
so the complete app window can genuinely scroll to source row 30 at both scales.
This checks observed viewport movement rather than assigning a synthetic anchor.

| Swift assertion | Rust evidence |
| --- | --- |
| Availability follows the caret, independently of passive scrolling | At source line 3/column 5 both directions target the surrounding blocks. A real Right key moves to column 6 while targets remain correct. Returning the caret and scrolling to row 30 changes the reading anchor without changing navigation targets. Rust derives availability directly in each query/frame; there is no separate Observation invalidation cache. |
| Next Change reveals hidden test code | Actual Alt+Down reaches line 22/column 0, removes that byte from hidden test ranges and exposes its fold, while the global Hide Tests preference stays enabled. |
| Back restores exact caret, not passive top | Back returns line 3/column 5. The prior browsing checkpoint fixed this through the existing exact-caret history intent. |
| Inside a block skips that block | At line 2/column 5 Previous is unavailable and Alt+Down reaches the test block at line 22. |
| EOF deletion remains reachable without becoming an added line | The next Alt+Down reaches the exact source byte length. Next is then unavailable, Previous targets the test block, and changed-line ranges exclude the trailing EOF row. The real Git patch has three hunks, including the empty After side. Repeating Next leaves history unchanged. The earlier unterminated-EOF regression covers the other newline shape. |
| Binary or unavailable hunks disable navigation | Both raw states produce no targets; Alt+Down/Up preserve source Arc, EOF caret and history. |
| Before uses its own hunk side | After a real revision switch, Alt+Down from Before line 3 reaches Before line 22, retaining exact base text and revision. |
| A new file cannot inherit old hunks | Opening an unchanged source after the Before navigation publishes its natural After side with no hunks/targets. Next remains idle with source/history retained. |

## Identity and ordering proof

Swift's independently supplied ReviewHighlights carry path/revision/content
identity. Rust prepares source bytes, syntax, full per-file hunks and initial folds
in one source-loader transaction. Its request's publication channel owns the whole
bundle; there is no separate late-highlight channel.

New `retired_pinned_source_results_cannot_publish_wrong_revision_or_changed_content_hunks`
uses real prepared Git results and the existing LatestRead sender boundary. It
holds a genuinely completed result in a worker, awaits that worker starting,
replaces the request and then forces the old sender to publish despite cancellation.
Both cancellation and closed-channel checks must be true. Only the replacement
result is available, with its exact source/revision/hunks/defaults and no extra
completion left behind.

Two cases isolate ownership:

- A real mode-only Git change offers Before and After with identical bytes. The
  fixture asserts both modes and equal blob IDs. Replacing Before with After tests
  revision identity without relying on different content to reject the result.
- Replacing After with a newer source uses the same path/revision and a different
  content digest. The new hunks and expanded changed-body defaults must belong to
  the replacement rather than the retired mode-only snapshot.

These tests prove stale-context rejection through the supported source-loader
publication path. They do not assert that arbitrary direct writes to public app
fields acquire provenance checks. Binary/nil inputs are separately tested at the
navigation boundary. This maps the Swift stale-input outcomes at Rust's actual
ownership boundary rather than adding a redundant independently arriving wash.

## Verification and architecture

Archive: `~/.local/state/radiology-qa/change-block-flow-20261109-8d29a34/`.

- `keys.log`: initial both-scale keyboard workflow passes.
- `ownership.log`, `ownership-final.log`: real deferred results are rejected after
  replacement; the final version also checks mode-only fixture validity.
- `change-final.log`: nine change-related tests pass, including the final keyboard
  flow and unchanged-file handoff.
- `loader.log`: eight source-loader tests pass before final fixture assertions.
- `clippy-final.log`: strict all-target workspace Clippy passes. Formatting and
  diff checks pass.

This checkpoint changes only test code and registration. No runtime implementation
or public API changes. The prior checkpoint's full serial run of 842 app tests and
65 doctests remains historical evidence; the two added tests and affected suites
were run directly, without claiming a fresh full-suite result.

Baseline `before-change-block-flow-20261109` to `final-change-block-flow-20261109`:
+251 measured NLOC/two test functions/one focused app-test file. Average/max CCN
remain 3.41/132 with 36 high-complexity functions, one cycle and eight structural
issues unchanged. The full normalized file-pair/fragment clone multiset remains
unchanged at 122 instances/1,317 duplicated lines, including multiplicity. The
raw touched-file flag is an existing eight-line app-test clone; equal-score Python
ranking movement is unrelated. Fixtures, parsing, worker channels, source focus,
frame/key helpers and completion polling are reused. The tree-edit fixture adds
an actual unchanged-file handoff case, keeping path ownership in the same flow.
No test-specific production visibility or injection API was introduced.

Three of four PullRequestReview groups now have explicit mappings with the
metadata/browsing checkpoints. The retired-review-load combined group remains
open, along with other unreconciled suites, final native acceptance, known
parallel timing limits and Android.
