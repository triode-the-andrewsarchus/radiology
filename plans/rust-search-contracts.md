# Repository search contracts

Reference: [SearchTests.swift](../Tests/RadiologyTests/SearchTests.swift), six ordinary
functions, and the checked-in [Swift worker](../Sources/Radiology/Search/SearchWorker.swift).
Reviewed from Rust `685ab93`. This checkpoint reconciles those six functions;
it does not establish overall parity for the remaining suites.

## Behavior and repair

The shared Rust runner previously returned `Ok(false)` for cancellation, which is
indistinguishable from an uncapped successful search to another host. Swift throws
`CancellationError`. The direct public-runner regression reproduces the discrepancy
(`cancelled search reported success: Ok(false)`) before the repair.

The existing `SearchError` now includes `Cancelled`. The runner checks cancellation
before launch, kills/waits through its existing process cleanup during streaming,
and checks again after completion, including a cancellation from the final short
batch callback. Completion uses one ordered match: cancellation wins, a reached
limit or normal/no-match exit succeeds, and other exits retain the diagnostic.
Already delivered batches stay with their caller. A genuinely empty uncancelled
query still succeeds without launching ripgrep.

`RepositorySearch::stop` still retires its publication channel immediately and
retains already presented rows. Thus an intentional UI Stop does not publish a
cancellation error. The existing generation/channel mechanism handles this; no
second worker, queue, status flag, public method or dependency was introduced.
External callers that exhaustively match `SearchError` must handle the new variant.

## Six-function reconciliation

| Swift function | Rust evidence |
| --- | --- |
| `decodingPreservesByteOffsetsForUnicodeAndMultipleMatches` | `unicode_submatches_keep_byte_columns_locations_and_distinct_ids` uses the same emoji/two-match offsets (35/39), line 3, byte columns 6/10, distinct IDs and canonical locations. Both text and base64 JSON forms preserve the path and CRLF preview. The earlier decoder fixture remains. |
| `argumentsProtectLeadingDashPatterns` | Extended `test_search_options_args` checks the exact `--`, `--help`, `.` suffix, adjacent include/exclude glob arguments and fixed-string/regex plus sensitive/insensitive flags across all four combinations. |
| `realRipgrepSearchFindsUnicodeLiteralAndHonorsGlob` | Extended `real_search_reports_glob_matches_and_regex_errors` runs real rg over emoji/lowercase and uppercase lines plus an excluded text file, asserting both exact paths, lines, columns and file byte offsets. Existing whitespace/glob matrix covers twenty immediate/debounced cases. |
| `noMatchesAreNotAnErrorAndInvalidRegexIsReported` | The same session test now explicitly transitions populated results → absent query with no error/no limit/empty results → invalid regex error → empty query clearing. |
| `resultLimitStopsTheProcessAndCapsDeliveredMatches` | `streaming_search_caps_submatches_and_reports_cancellation` searches 6,000 matches, crosses the 5,000 cap inside a three-match JSON record, returns limited and checks every delivered path/line/byte offset. The awaited search must finish within five seconds. |
| `cancelledSearchPropagatesCancellation` | The same direct-runner matrix asserts `SearchError::Cancelled` before launch, from a ten-result final batch and from the first forty-result streaming batch of a 10,000-line file. It checks exact delivered counts and offsets, with bounded completion. Existing `replacement_cannot_publish_a_retired_request` independently gates a live worker and proves cancellation plus rejected retired-channel publication. |

The decoder and streaming matrices each reuse one fixture/loop rather than copying
launch or assertion blocks. Session and app tests continue to use the shared
`finish_search` test helper.

## Verification

Evidence archive: `~/.local/state/radiology-qa/search-contracts-20261109-685ab93/`.

- `cancellation-red.log`: direct cancellation reproduces the old successful result.
- `search-final.log`: all 28 search-filtered library tests pass, including existing
  sidebar/app request ownership and the new shared-runner checks.
- `clippy.log`, `build.log`, `rustdoc.log`: locked offline strict workspace Clippy,
  normal app build and workspace API docs with warnings denied pass. Generated
  `SearchError` and `SearchRunner` HTML contains the new variant and its contract.
- `workspace.log`: locked offline serial workspace verification passes 818 app
  tests (11 existing ignored), two binary tests, 13 font tests, 22 text tests and
  all 61 doctests. All artifact-producing checks above finished before starting
  it, preserving self-relaunch test executable ownership. Formatting and diff
  checks also pass.

Architecture baselines `before-search-contracts-20261109` and
`after-search-contracts-20261109`: +114 measured NLOC, two test functions, no runtime
function or module added. Runner CCN 15→14; average/max remain 3.40/132; all 36
existing high-complexity functions, one cycle and eight structural findings remain
unchanged. Full normalized clone multisets retain all 122 instances/1,317 duplicate
lines with zero additions/removals. The raw comparison flags the unchanged
nine-line base64 decoder clone solely because its file was touched. An intermediate
new test-launch clone was removed by the single streaming matrix; it is absent
from the final comparison. Detailed vectors are in `architecture.txt`.

## Limits and remaining work

No GUI behavior or appearance changed here. Prior native 1×/2× search/result/error
and exact-source workflows remain documented in
[rust-repository-search](rust-repository-search.md); they were not rerun for this
service checkpoint. These tests use real installed rg without silently skipping;
they do not execute Swift/AppKit or independently inspect child PIDs. Completion
awaits the existing runner's kill/wait paths. Full accessibility remains deferred.

The next review is the three ChangeBlockNavigationTests and five UnifiedDiffTests
functions. Source inspection already identifies two unreconciled differences:
Rust silently ignores malformed `@@` headers while Swift throws, and Rust's
`map_line` preserves offsets inside replacement hunks or uses an empty side's
preceding anchor while Swift lands at the first changed line or the line after a
deletion gap. The subsequent [diff checkpoint](rust-diff-contracts.md) supplies
red/green evidence and repairs both differences. Separate raw metadata and document
identity assertions remain open there.

Other Swift suites, broader final integration, the previously recorded concurrent
semantic/selection timing limits and SDK-backed Android acceptance remain open.
Radiology remains standalone; actual little-screen-clip adoption is deferred.
