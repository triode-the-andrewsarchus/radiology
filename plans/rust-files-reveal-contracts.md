# Files reveal assertion review — 2026-11-08

The four Swift FilesSidebarRevealTests functions now have an explicit outcome
mapping to Rust's accepted-source presentation. This pass extends existing app
regressions with actual bookmark and search-result routes, nested deleted Before
source and preservation of unrelated disclosures. Existing runtime meets these
workflows; production behavior, APIs and dependencies are unchanged here.

## Swift outcome mapping

| Swift function | Rust evidence |
| --- | --- |
| presentationsRevealFilteredAndBeforeOnlyFilesWithoutChangingFilters | files_current_exception_preserves_inventory_filters_and_exact_source now uses the same src/main.swift, tests/nested/Spec.swift and docs/guide.md categories. A filtered loaded Spec appears in the flat list and src/tests tree while the underlying filtered inventory contains only main.swift. Inventory Arc and filters remain unchanged. A newly loaded unlisted file gets the sole exception; a missing read retains that accepted row instead of inventing a missing entry. The real nested Before regression below verifies an entry outside the After inventory. |
| expandingAncestorsPreservesOtherFolders | The long reveal test starts with other expanded and b/z collapsed. A bookmark to z/deep expands only its ancestors, keeps other and does not insert its leaf. A later search opens b/deep while preserving other and the manually collapsed z/deep. A top-level source jump leaves the entire expansion set unchanged. |
| nativeFilesListTracksBookmarksAndSearchAndPreservesTheirCaret | files_reveal_centers_committed_jumps_and_preserves_passive_scroll_and_caret now prepares two 130-file branches. A real bookmark command restores byte 5 and centers its row at both scales. Clicking the current row preserves source Arc/caret/history. Actual wheel input and a source caret move do not recenter; an explicit same-file jump does. Real ripgrep with an exact include glob finds b/deep/file044.rs at byte 19; clicking its result uses the app's search route. Returning to Files expands/reveals it while preserving exact source Arc, byte and full history. |
| nativeFilteredListRevealsBeforeOnlyPRFileAndKeepsItsRevision | files_deleted_before_row_keeps_revision_caret_and_filters_when_clicked builds real nested base/head commits with src/old.rs deleted in After. At both scales, a nonmatching query plus both filters still shows the selected Before exception. Clearing the query shows the tree and expands src. Clicking the current row in both presentations retains Before, byte 1, source Arc, filters and history. Attached HEAD/status stay unchanged and the checkout file remains absent. |

The Rust tests inspect actual egui row geometry and state; they do not execute
AppKit or assert NSTableView row numbers. Centering is checked in logical points
with the established rendered-app bounds; source bytes/history are checked exactly.
The public app's accepted-source boundary supplies the presentation. Swift's pure
FilesSidebarPresentation initializer and Rust's combined app/tree representation
are not identical APIs; this evidence does not claim optimistic pending-row
construction before a source read is accepted.

The earlier [Files implementation/native checkpoint](rust-files-navigation.md)
records compositor acceptance at both scales and the current-row handoff policy.
This checkpoint adds no new native run or SwiftUI execution claim.

## Test design and reuse

The nested Before fixture initially changed filter fields directly while its
source was already accepted, leaving the cached tree from before the query edit.
The corrected test uses filters_changed, the same path used by actual sidebar
edits, then checks both flat and tree presentations in one parameterized loop.
No production defect is claimed from those preliminary row-lookup failures.

Real search completion reuses the existing search/session.rs test wait helper,
now available under cfg(test) as finish_search. Session tests and both prior
search-edit waits share it with Files. Its five-second deadline and actual worker
polling remain intact. A newly introduced duplicate wait and the old repeated
focus-test wait are removed without a second renderer, fake search result or
runtime synchronization mechanism.

## Verification and architecture

Three extended Files app regressions pass, including the both-scale workflows.
All 26 search-filtered tests pass after the helper consolidation, covering its
session and app callers. Strict workspace/all-target Clippy and format/diff checks
pass. This checkpoint is test-only; it does not claim a fresh full workspace run
or binary build. The preceding runtime checkpoint passed 816 app tests and all
61 doctests. Logs and complete architecture counters are retained under
~/.local/state/radiology-qa/files-reveal-contracts-20261108-ce2ca57/.

Architecture before/after-files-reveal-contracts-20261108: +97 measured NLOC,
no new module or net function. Average/max CCN remain 3.40/132 with the same
36 high-complexity functions, one cycle and eight structural issues. Full normalized
clone multisets show zero additions and one removal: 123→122 clone instances and
1323→1317 duplicated lines. The final raw result is IMPROVED. The gain comes from
sharing actual search completion and parameterizing the two Before presentations,
while expanding the evidence rather than removing required assertions.

## Remaining work

Continue repository SearchTests and the other ordinary Swift groups, maintaining
native/standalone boundaries and final integrated acceptance. Pending-source
presentation remains within the separately documented loading contracts; this
checkpoint proves accepted-source reveal behavior. Parallel timing limits,
SDK-backed Android success and machine-specific probe limits stay explicit.
Full accessibility and actual little-screen-clip adoption remain deferred.
