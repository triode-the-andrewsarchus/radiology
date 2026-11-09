# Bookmark navigation assertion review — 2026-11-08

All three ordinary Swift BookmarkNavigationTests functions now have a combined
Rust app workflow at both display scales. This follows the declaration-label and
numbered-command fixes in [the prior checkpoint](rust-bookmark-command-parity.md).
The runtime already satisfies these navigation contracts; this checkpoint changes
only tests and documentation.

## Assertion mapping

| Swift function | Current Rust regression |
| --- | --- |
| shortcutsUseReorderedBookmarksAndBackRestoresTheExactCaret | reordered_shortcuts_restore_exact_departures_and_refresh_relocated_unicode_bookmarks: real Ctrl+D and Ctrl+0 input, passive top independent of caret, exact Back to another file/column, reordered slot activation, independent app relaunch loading saved order, inserted Unicode/CRLF source relocation with retained identity/label/order, persisted refreshed coordinates and toggle removal. The previous twelve-entry command test supplies the populated out-of-range slot boundary. |
| checkoutEachPRAndAnotherRepositoryKeepIndependentListsAndRevisions | checkout_two_pull_requests_and_another_root_restore_independent_bookmark_lists: real Git fixture and prepared snapshots, local checkout entry, independent PR 7 After/Before entries, empty PR 8 then its own entry, return to PR 7, exact bookmark destinations and pinned source bytes, Back to Before, exit to the original checkout list, another root's empty list, reopening checkout and restoring PR 8's list. Scope uses the attached root rather than the extracted head tree. |
| missingFilesAndSupersededJumpsLeaveTheReaderAlone | missing_bookmark_and_retired_preparation_preserve_the_current_reader_and_new_scope: actual missing-file failure retains the accepted source Arc, caret, full history and bookmark identity; another request superseded by accepted root discovery cannot leak status/source/history. Separately, a genuinely completed old bookmark result held before publication cannot populate the newer empty scope. |

The fresh app created for persistence restoration uses the same isolated settings
path and opens its repository without selecting a source. It verifies ordered
records, not just a serialized count. PR fixtures use actual Git objects and the
normal ReviewSnapshot::prepare/install/publication path. They bypass GitHub lookup;
no live network or GitHub refresh claim is made here. The attached checkout's
observed source remains checkout-only while pinned reads return the exact Before
or After strings.

Back restores source coordinates independently of list selection. The first test
initially asserted that history must also select a different bookmark row; that
was an overreach beyond the Swift assertion and was corrected. No product defect
is claimed from that preliminary test failure.

## Publication and test architecture

The stale-result test exercises ordinary failed-request replacement, then
separately holds a real successful preparation at the existing publication
boundary. It does not invent delivery through a retired worker channel during an
active successor. The existing source-loader replacement/retirement tests remain
responsible for that transport boundary.

The existing app-model parse test's bounded preparation wait is now shared with
the bookmark publication test. It still polls the real loader with a ten-second
failure deadline. A local location assertion keeps source comparisons consistent;
a shared empty-reader assertion covers both failed and successful retired cases.
These helpers introduce no alternate renderer, fake source loader or runtime API.

Architecture before-bookmark-workflows-20261108 → after-bookmark-workflows-20261108:
+223 measured NLOC, no new module, six test-only functions (three regressions and
three helpers). Average/max CCN remain 3.40/132, with the same 36 high-complexity
functions, 123 clone instances/1323 duplicated lines, one cycle and eight structural
issues. Normalized fragment/file-pair multisets contain zero added/removed clones.
An initially repeated empty-scope assertion was consolidated. The raw comparison
still flags an unchanged clone in the touched parent test module and equal-score
ranking changes between untouched Python functions. Full clone and metric
comparisons distinguish those findings from actual additions.

## Verification and remaining scope

Workspace passes 804 app tests/11 ignored, two binary, 13 font, 22 text and
60 doctests. The five bookmark tests pass again after empty-scope assertion
consolidation; strict workspace/all-target Clippy and formatting pass. Full
workspace results and architecture details are retained under
~/.local/state/radiology-qa/bookmark-workflows-20261108-075c95e/.
This is app/rendered and Git-backed evidence. It does not claim a new native desktop
run, SwiftUI execution or a change to the reusable reader's public surface.

Continue the separate BookmarkTests model/storage functions and remaining Swift
suite assertions. Existing [model/anchor/native evidence](rust-bookmark-pane.md)
remains useful, but this checkpoint does not equate it with an assertion-by-assertion
closure of every model test. Successful SDK-backed Android indexing remains open;
full accessibility and actual little-screen-clip adoption stay deferred.
