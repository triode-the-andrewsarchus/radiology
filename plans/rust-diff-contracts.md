# Unified diff parsing and revision mapping contracts

Reference: [UnifiedDiffTests.swift](../Tests/RadiologyTests/UnifiedDiffTests.swift),
[ChangeBlockNavigationTests.swift](../Tests/RadiologyTests/ChangeBlockNavigationTests.swift)
and the checked-in [Swift implementation](../Sources/Radiology/PullRequest/UnifiedDiff.swift).
Reviewed from Rust `03ec79b`.

This checkpoint reconciles the first three of five UnifiedDiffTests functions and
all three ChangeBlockNavigationTests functions. Raw change metadata and document
identity are still separate, explicitly open assertion groups.

## Confirmed differences and repair

Two direct regressions fail before the production changes:

- `@@ -x +1 @@` silently became an empty nonbinary change set.
- A line removed by `@@ -3,8 +2,0 @@` mapped from Before line 3 to After line 2.
  Swift maps it to line 3, after the deletion gap. Inside a replacement, Swift also
  targets the first changed line rather than retaining an offset within the hunk.

`UnifiedDiffParser::parse` now returns `Result<FileLineChanges, DiffParseError>`.
The typed error retains the offending header. Existing header/range helpers are
reused; numeric bounds reject ends/empty-side anchors outside the signed mapping
coordinate space before ranges are used. Binary markers must occupy their own
lines, so source text mentioning `Binary files` does not erase text hunks. The
existing `GIT binary patch` extension remains supported.

All checkout, pinned review and refresh callers propagate errors through their
existing request result paths. Review changes/counts/maps are updated only after
parsing succeeds. There is no fallback to empty hunks for malformed output.
Existing tests/examples explicitly handle valid fixture Results.

`FileLineChanges::map_line` remains the one shared mapping function for revision
switching and review refresh. Changed lines target the first destination line;
an empty destination targets the line after its anchor. Existing handling of an
empty source-side anchor and accumulated shifts outside hunks is retained.
No parallel mapping mechanism or UI special case was added.

### Public API migration

`parse` previously returned `FileLineChanges` directly. Hosts must now use `?`,
match its Result or explicitly unwrap trusted fixtures. `DiffParseError` is
available from `radiology::pull_request` and its `diff` module. The API documentation
includes a runnable valid/malformed example. This deliberate signature change
makes the Swift error contract available to future little-screen-clip callers.
No dependency, worker, persisted format or framework coupling changed.

## Assertion mapping

| Swift function | Rust evidence |
| --- | --- |
| `parsesHunkHeadersAndSkipsEverythingElse` | `headers_counts_ranges_and_binary_markers_match_swift_fixtures` uses the same three Kotlin headers, omitted count, zero-count side, content/no-final-newline marker, exact hunk fields, Before/After ranges, anchors and totals 22/33. Existing single-hunk parser fixture remains. |
| `pureAdditionsDeletionsBinaryAndCRLF` | The same fixture matrix checks addition/deletion fields, CRLF Unicode content and a header immediately after CRLF content. It checks real binary marker forms, empty input and source-text marker rejection. Existing empty-side byte-navigation tests verify absent highlights and reachable points. `malformed_hunk_headers_are_reported_instead_of_discarded` checks invalid fields, negative/multiple counts, valid-prefix-plus-invalid-tail rejection and numeric range bounds. |
| `lineMapShiftsOutsideHunksAndLandsOnChangesInside` | `line_map_lands_on_changes_and_after_empty_gaps_in_both_revisions` checks all 16 Swift line/revision expectations and an additional interior replacement line. Prior zero-length-anchor tests remain. Real app revision-switch regression below verifies the command path. |
| `insideBlocksSkipsCurrentAndGapsChooseClosestWithoutWrap` | Existing `byte_navigation_matches_inside_gap_and_unterminated_eof_boundaries` uses the same 30-line source, three blocks and five inside/gap/edge caret cases. Existing binary/empty model handling uses the shared navigation block path; no new navigation implementation is needed. |
| `EOFOnUnterminatedChangedLineBelongsToCurrentBlock` | The same existing byte-navigation test checks unterminated `changed` EOF versus the trailing empty line of `changed\n`. |
| `emptySideAtEOFWithoutFinalNewlineRemainsReachableWithoutHighlights` | Existing `empty_byte_targets_are_points_deduplicate_clamped_eof_and_ignore_binary_changes` checks both revisions on `one\ntwo`, no changed ranges, a reachable EOF point, no repeated target, deduplicated clamped points and no binary targets. |

The new app test
`revision_switches_land_on_first_changed_lines_and_after_deleted_gaps` prepares a
real Git snapshot and runs the actual revision command at 1×/2×. It checks a deleted
Before line → line after gap, an interior Before replacement → first After line,
and an interior After replacement → first Before line. Rendered app frames retain
exact Unicode/CRLF buffers, expected revision/caret and no error. Attached HEAD,
status and dirty checkout bytes remain unchanged.

The existing checkout Git failure matrix now includes malformed output and checks
the diagnostic reaches its caller. `malformed_patch_does_not_replace_pinned_changes_or_counts`
uses the existing fake-Git fixture at the review boundary and checks error identity
plus unchanged changes/diffs. The fake executable is scoped per fixture; process
PATH and user repositories are untouched.

## Verification and architecture

Archive: `~/.local/state/radiology-qa/diff-contracts-20261109-03ec79b/`.

- `diff-red.log`: two failures above, with six earlier diff tests passing.
- `pull-request-final.log`: all 36 pull-request-filtered tests pass, including
  parser, bounds and pinned-rejection tests.
- `app-review.log`: all 18 app review/refresh-filtered tests pass, including the
  new both-scale revision switches and existing moving-reader/merge-base mapping.
- `checkout.log`: extended checkout failure matrix passes.
- `clippy.log`, `build.log`, `rustdoc.log`: locked offline strict workspace Clippy,
  normal app build and warning-free API docs pass. Generated parser/error HTML
  was inspected for the Result, error contract and runnable migration example.
- `workspace.log`: locked offline serial workspace passes 823 app tests/11
  existing ignored, two binary/13 font/22 text tests and all 62 doctests. No source
  edits or recompilation occurred while its self-relaunch executable was live.
  Formatting and diff checks pass.

Baselines `before-diff-contracts-20261109` → `after-diff-contracts-20261109`:
+249 measured NLOC, five test functions, no runtime helper/module added. One typed
error is added to the existing diff module. Average CCN 3.40→3.41, max 132; all 36
high-complexity functions, one cycle and eight structural findings remain unchanged.
Parser/map/range CCNs are 8/8/7, below the hotspot threshold. Full normalized clone
multisets retain all 122 instances/1,317 duplicate lines, zero additions/removals.
Raw comparison is NEUTRAL; reported top-ten Python chokepoint swaps are equal-score
ranking churn in untouched files. Full vectors are in `architecture.txt`.

## Remaining scope

The two remaining UnifiedDiffTests functions, `rawRecordsCoverAddModifyDeleteRenameAndNumstat`
and `locationsCarryRevisionsAndPositionKeys`, are not closed by parser/navigation
evidence. In particular Rust review discovery currently uses name/status records
and leaves the public FileChange blob-ID fields empty; Swift raw records carry
those IDs. Pinned source reads already use commit/path identity, but that is not
proof of the separate metadata assertion. Reconcile this explicitly next.

Other review/refresh/presentation suites and the broader Swift corpus still need
assertion-level review. Prior native source/revision/change-navigation checks are
in [rust-byte-change-navigation](rust-byte-change-navigation.md); this checkpoint
adds app frames and does not claim a fresh compositor or Swift/AppKit run. No type
scale or layout changes are introduced; unavailable SwiftUI/macOS/type skills are
recorded in earlier checkpoints. Full accessibility and actual sibling-app adoption
remain deferred. Final integration, recorded parallel timing limits and SDK-backed
Android acceptance remain open. Radiology remains standalone.
