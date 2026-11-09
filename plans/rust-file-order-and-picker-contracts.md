# Natural filename ordering and file-picker contracts — 2026-11-08

The recorded filename-ordering difference is repaired, and all five ordinary Swift
QuickFileSearchTests functions now have explicit Rust evidence. Numeric names now
sort naturally in discovery inventories, directory-first trees, flat Files query
rows, pinned review inventories and equal-score quick-picker tie breaks.

## Shared name ordering

The checked-in Swift RepositoryWorker, FileNode and final quick-search comparison
use localizedStandardCompare. [Foundation's implementation](https://github.com/swiftlang/swift-corelibs-foundation/blob/main/Sources/Foundation/NSString.swift)
combines numeric, case-insensitive and width-insensitive comparison with forced
ordering. Rust previously used lowercase lexical keys or raw path bytes. Both new
regressions fail before the change: folder10/file10 precede folder2/file2, and two
equal-score picker candidates use lexical rather than numeric path order.

A private name_order module now owns one ICU collator. Numeric ordering uses
[ICU's Unicode decimal-digit collation](https://docs.rs/icu_collator/2.4.0/icu_collator/preferences/enum.CollationNumericOrdering.html).
Secondary strength keeps accents significant while treating case/width as secondary
name variants; original bytes break collation ties deterministically. The comparator
never changes a source path. Existing fuzzy tiers, boundary/gap penalties, path
length and visible-result limits retain their priority; only their final tie break
changes. Changes-list/source traversal order and machine cache keys are untouched.

The collator captures the first nonempty LC_ALL, LC_COLLATE or LANG value once,
strips POSIX encoding/modifier suffixes and converts underscore separators to BCP47
hyphens. C/POSIX, invalid or absent tags use ICU's language-neutral fallback.
Direct parameterized tests avoid changing process-global environment and verify
German/Swedish tailoring. Numeric ordering is always enabled for filename lists.
This follows the intended locale/numeric behavior; it does not certify identical
Apple/ICU weights for every locale and punctuation combination.

ICU collator 2.4 and a direct declaration of the already-resolved locale-core 2.4
supply the mechanism. The lockfile adds seven supporting packages; it does not
upgrade existing package versions. Compiled collation data avoids a runtime file
or system ICU dependency for this feature. The independent reader-text/font crates
do not gain these dependencies. No public API, worker, persisted format or sibling
coupling is added. The existing picker-completion fixture is reused by the new app
workflow instead of a second wait implementation.

## Swift QuickFileSearchTests mapping

| Swift function | Rust evidence |
| --- | --- |
| orderedSubsequenceFindsTheUsersExampleAndRejectsOutOfOrderText | test_quick_search_subsequence now checks the exact engine/rust/engine-foo.rs example with engrs/ENGRS and rejects srgne, retaining the prior buffer example. |
| rankingFavorsFilenameExactPrefixAndContiguousBeforeGappyAndPathOnly | test_ranking_favors_filename_tiers uses the identical five-path scan fixture and exact full output order. The new equal_rank_candidates_use_numeric_path_order_as_the_final_tie_break adds a real tie with file10 and file2x; both have equal scoring fields before path comparison. |
| unicodeCaseFoldingAndStablePathDisambiguationWork | test_diacritic_folding now checks the exact a/CAFÉ.swift then b/Café.swift order, not just two matches. Existing sharp-S/sigma/ligature/decomposed-name tests and both-scale app navigation keep their original paths and exact bytes. |
| visibleResultsAreBoundedButTotalCountRemainsTruthful | cancelled_ranking_discards_partial_matches_and_fresh_search_keeps_order_and_totals checks 4096 matches, 100 returned rows, truthful total/limited status and a zero-visible-limit response. The existing rendered picker test independently verifies the 100-of-240 footer and virtualization. |
| selectionResetsAndClampsAcrossResultChanges | New both-scale selection_clamps_at_result_edges_and_clears_when_inventory_becomes_empty checks the first of three paths, repeated Down/Up clamping at each edge, an empty replacement and disabled Return. Existing narrow-query/Return and gated query/inventory/cancel tests verify reset and stale-publication retirement. |

The picker processes ordered key batches across frames. The first selection test
asserted before its queue drained; using the existing settled helper fixes the
test, with no product change claimed from that preliminary failure.

## Integrated verification

The app workflow checks natural discovered order and flat filter rows at both
scales, commits the first empty-query picker result and copies the exact Unicode/
CRLF source. A real Git-backed PR snapshot checks its numbered inventory order,
then query/Return loads exact pinned After text. The existing finished-picker wait
is shared with the earlier Unicode app workflow, avoiding a timing assumption.

Collator and tree tests cover file/folder numbers, Arabic-Indic and full-width digits,
integers beyond machine range, accents, leading-zero/case/canonical-equivalence
ties, invalid tags, language-neutral defaults and locale tailoring. A temporary
copied test executable runs both collation tests from an otherwise empty working
directory and is deleted afterward; only its log remains. This verifies compiled
data use, not a new native desktop or full source-export acceptance run.

Focused repository/app/modal tests, strict workspace Clippy and a locked offline
binary build pass. The locked offline serial workspace run passes 816 app tests
(11 opt-in ignored), two binary, 13 font, 22 text tests and 61 doctests. There is no build or
source mutation while its executable is live. Logs and architecture details:
~/.local/state/radiology-qa/file-order-20261108-68603bb/.

Architecture before-file-order-20261108 → after-file-order-20261108: +256 measured
NLOC, one private module and eight functions (two runtime helpers, six tests).
Average/max CCN remain 3.40/132; the 36 high-complexity functions, 123 clone instances,
1323 duplicate lines, one cycle and eight structural issues are unchanged. Full
normalized clone multisets have zero additions/removals. The raw comparison flags
old clones in touched test files and unchanged Python ranking churn. New collator
setup/comparison CCNs are 1/2; the five presentation paths share their policy.

## Remaining work

Continue the other ordinary Swift suites and final integrated native/standalone
acceptance. The prior parallel timing limits and successful SDK-backed Android
indexing remain open. Full accessibility and actual little-screen-clip adoption
stay deferred.
