# Source filter state and assertion review — 2026-11-08

All four ordinary Swift SourceFilterTests functions now have explicit Rust evidence.
The combined app workflow exposed and repaired a disclosure-state loss: accepting
a refreshed listing reset manually expanded folders, even when the attached root
had not changed.

## Swift assertion mapping

| Swift function | Rust evidence |
| --- | --- |
| testPathsAreRecognizedByComponentsAndNamesNeverBySubstring | Existing test_test_classifier now includes all 27 positive and 18 negative Swift paths: exact test components, Gradle source sets, filename conventions and lookalikes that must remain production code. Earlier Rust cases remain too. |
| sourceFilesExcludeArtifactsDependenciesGeneratedDocsAndManifests | Existing test_source_classifier now includes all 21 positive and 39 negative Swift paths, including source/script/header variants, generated outputs, dependencies, hidden paths, manifests, config files and assets. Earlier Rust cases remain. Together the two functions exercise 127 distinct classification cases, including all 105 Swift examples. |
| treeIsPrunedOrderedAndBothOptionsCombineIndependently | New reference_filter_combinations_prune_empty_directories_and_preserve_tree_order uses the same thirteen-file fixture. Default filtering keeps every file. All three active combinations compare the complete directory-first flattened tree with Swift's expected paths, prune empty directories and check includes_path against filter_files. |
| modelFiltersTheTreeOnlyAndKeepsReaderStateAndPreferences | New both-scale filters_preserve_reader_preferences_and_manual_directory_expansion_through_refresh uses the same five-document app fixture. It checks immutable inventory/source ownership, unchanged history and selected README, independent options/query results, preference restoration and clearing, refreshed six-file inventory and four source candidates, retained manual disclosures, rendered guide.md after clearing filters, same-root plain reopening and clearing on another root. |

The path cases are ordinary Rust test data copied from the checked-in reference;
compilation does not depend on the Swift source tree or a Swift runtime. Existing
Rust cases were retained and compared against the prior source inventory.

## State repair and presentation boundaries

Swift AppModel.openRepository resets FileTreeExpansion only when the canonical
attached root changes. Its explicit expansion model retains paths through filtering,
refreshes and same-root snapshots. Rust's accept_repository previously called
reset_repository_state unconditionally and discarded expanded_dirs. The new app
regression fails before the fix at the retained-src assertion.

Acceptance now determines whether the discovered canonical root matches the
accepted root, moves the existing disclosure set through the reset in that case,
and installs it afterward. The same identity comparison chooses refresh source
restoration. It does not copy the set, add another cache or change worker ownership.
Collapsed folders stay collapsed. Different-root opens retain the existing reset;
plain same-root opens still wait for an explicit file choice. Source preparation,
index discovery, history and review lifecycle retain their existing transactions.

Rust's file_tree is the displayed tree, including the previously accepted current-
document exception. Swift keeps its filtered model tree separate from its sidebar
presentation. Consequently, this checkpoint compares filtered inventory counts to
Swift's visibleFileCount and verifies tree pruning separately without an open-file
exception. It also verifies that the loaded README remains readable and that its
path fails the active filter. It does not remove the intentional loaded-document
row or count it as a matching candidate. See the
[Files presentation contract](rust-files-navigation.md).

No dependency, public API, persistent format, type scale, background worker or
sibling-app coupling changes. The fix extends the existing root acceptance path.

## Verification and architecture

Final focused repository/app checks, strict Clippy, formatting and locked offline
binary build pass. The final serial workspace run passes all 810 app tests
(11 opt-in ignored) in 243.62 seconds, plus two binary, 13 font, 22 text tests
and 61 doctests. Full workspace verification runs serially because of the two
previously recorded concurrent timing failures, documented in the
[repository checkpoint](rust-repository-contracts.md). Its output and the final
focused checks are retained under
~/.local/state/radiology-qa/filter-contracts-20261108-8309581/.
The focused group was rerun after Clippy's if/else clarity correction and the
additional retained classifier cases. An initial serial run overlapped that test-
binary rebuild: two tests that relaunch current_exe failed to spawn the replaced
artifact (NotFound); the other 808 app tests passed. This is a verification-workflow
error, not a product or concurrency-limit failure. The final full run keeps its
compiled executable unchanged. No new native desktop run is claimed.

Architecture before-filter-contracts-20261108 versus
after-filter-contracts-final-20261108: +340 measured NLOC, mostly expanded fixture
data/assertions; two added test functions, no new module or runtime function.
Repository acceptance adds seven measured lines and goes from CCN 5 to 6. Average/
maximum CCN remain 3.40/132, with 36 high-complexity functions, 123 clone instances,
1323 duplicate lines, one cycle and eight structural issues. Full normalized clone
multisets have zero additions/removals. The raw classification is NEUTRAL; an
unchanged equal-score Python ranking moves without a source edit. The lower
percentage of duplicate lines reflects more test data, not removed duplication.

## Remaining work

Continue the other ordinary Swift suites and final integrated acceptance. Swift's
localized-standard name ordering versus Rust's lowercase lexical ordering also
needs a numeric-name fixture; the source-filter reference matrix does not test that
case. The previously recorded concurrent timing failures remain open. Successful
SDK-backed Android indexing and machine-specific probe limits remain explicit.
Full accessibility and actual little-screen-clip adoption stay deferred.
