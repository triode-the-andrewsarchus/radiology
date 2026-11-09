# Swift behavior-suite evidence index — 2026-11-05

Reference inventory at Rust `fbf6872`:81 root Swift files,66 ordinary suite files,11 opt-in probe files and four support files;334 annotated test functions before parameter expansion. Fixture source files are separate. File-level categories do not imply that every test in an ordinary suite runs unconditionally. The hashed function inventory is retained in the evidence archive for the Return checkpoint.

This is an index of source and existing acceptance evidence, **not proof that every Swift assertion has been reconciled**. Each linked checkpoint states its actual tests, native/compiler boundaries and remaining limits. Reading a filename or passing the Rust suite alone does not establish parity. The current pass rechecked reader navigation/selection and found the Return mismatch repaired in [its checkpoint](rust-reader-return-selection.md). Continue the remaining groups against their individual assertions; keep discovered gaps in scope.

## Reader state, selection and navigation

Rust entry points: [tests.rs](../src/ui/app/tests.rs), [positions.rs](../src/ui/app/tests/positions.rs), [focus.rs](../src/ui/app/tests/focus.rs), [viewport.rs](../src/ui/source_view/viewport.rs), [history.rs](../src/source/history.rs).

Evidence: [rust-reading-positions](rust-reading-positions.md), [rust-reader-return-selection](rust-reader-return-selection.md), [rust-source-status-layout](rust-source-status-layout.md).

| Swift suite | Annotated functions |
| --- | ---: |
| [AppModelTests](../Tests/RadiologyTests/AppModelTests.swift) | 6 |
| [NavigationHistoryTests](../Tests/RadiologyTests/NavigationHistoryTests.swift) | 1 |
| [ReaderSessionTests](../Tests/RadiologyTests/ReaderSessionTests.swift) | 7 |
| [SourceCaretTests](../Tests/RadiologyTests/SourceCaretTests.swift) | 3 |
| [SourceScrollViewTests](../Tests/RadiologyTests/SourceScrollViewTests.swift) | 12 |
| [LineNumberGutterTests](../Tests/RadiologyTests/LineNumberGutterTests.swift) | 2 |
| [FunctionShadeTests](../Tests/RadiologyTests/FunctionShadeTests.swift) | 3 |
| [ReaderReturnKeyTests](../Tests/RadiologyTests/ReaderReturnKeyTests.swift) | 2 |
| [ReaderFunctionFoldKeyTests](../Tests/RadiologyTests/ReaderFunctionFoldKeyTests.swift) | 4 |
| [SourceFunctionNavigationTests](../Tests/RadiologyTests/SourceFunctionNavigationTests.swift) | 1 |
| [OccurrencePinningTests](../Tests/RadiologyTests/OccurrencePinningTests.swift) | 4 |

FunctionShadeTests: all three groups now have [shading evidence](rust-function-shading-audit.md)
and [pending-context/recovery evidence](rust-pending-source-context.md). The latter
closes immediate context clearing and validates real blocked native reads.

ReaderFunctionFoldKeyTests and SourceFunctionNavigationTests: all five contracts
are reconciled in the [function audit](rust-function-contracts.md), including
nested prepared publication, manual recollapse, Unicode selections and native
Caps Lock/modifier/editable-field behavior.

LineNumberGutterTests: both contracts and SourceScrollViewTests' two meter
assertion groups are reconciled in the [gutter checkpoint](rust-gutter-parity.md).
That checkpoint adds wrapped-continuation and crowded-count evidence; other scroll
assertions retain their linked checkpoint-specific boundaries.

SourceCaretTests: all three contracts are reconciled in the
[caret checkpoint](rust-caret-parity.md), including rendered geometry, focus/menu
ownership and native selection/editor transitions at both scales.

ReaderSessionTests: all seven contracts are reconciled in the
[session recovery checkpoint](rust-session-recovery.md), with controlled GitHub
failure and native no-argument relaunch at both scales.

OccurrencePinningTests: all four named assertions are reconciled in the
[reader-host/pin audit](rust-reader-host-audit.md). Other rows in this group retain
their checkpoint-specific scope.

NavigationHistoryTests' single contract and SourceScrollViewTests'11 ordinary
assertion groups are now reconciled in the
[scroll/history checkpoint](rust-scroll-history-contracts.md). It maps each
function to Rust tests and platform equivalents; the absolute-path opt-in
performance probe is explicitly unrun.

AppModelTests: all six ordinary functions are now reconciled in the
[app-model checkpoint](rust-app-model-contracts.md), with an assertion-level table.
It combines new both-scale app publication/grammar/feedback tests with the
[root-choice fix](rust-empty-repository-reader.md) and
[passive-trail fix](rust-passive-history-trail.md). Other suite groups retain their
checkpoint-specific limits.

## Bookmarks

Rust entry points: [store.rs](../src/bookmarks/store.rs), [tests.rs](../src/ui/app/tests.rs), [bookmark_list.rs](../src/ui/bookmark_list.rs).

Evidence: [rust-bookmark-pane](rust-bookmark-pane.md).

The [bookmark command checkpoint](rust-bookmark-command-parity.md) fixes declaration
labels and bounds numbered commands without restricting UUID list activation. It
adds focused app evidence. The navigation functions are reconciled below; the
three model/storage functions are now reconciled in the
[model/storage checkpoint](rust-bookmark-model-contracts.md).

| Swift suite | Annotated functions |
| --- | ---: |
| [BookmarkTests](../Tests/RadiologyTests/BookmarkTests.swift) | 3 |
| [BookmarkNavigationTests](../Tests/RadiologyTests/BookmarkNavigationTests.swift) | 3 |

All three BookmarkNavigationTests functions are now reconciled in the
[combined workflow checkpoint](rust-bookmark-navigation-contracts.md). It records
the real Git/app boundaries; the model/storage checkpoint completes BookmarkTests.

## Repository, filters, Files reveal and quick navigation

Rust entry points: [tests.rs](../src/repository/discovery/tests.rs), [filters.rs](../src/repository/filters.rs), [quick_search.rs](../src/repository/quick_search.rs), [files.rs](../src/ui/app/tests/focus/files.rs).

Evidence: [rust-repository-opening](rust-repository-opening.md), [rust-unicode-file-search](rust-unicode-file-search.md).

| Swift suite | Annotated functions |
| --- | ---: |
| [RepositoryTests](../Tests/RadiologyTests/RepositoryTests.swift) | 3 |
| [SourceFilterTests](../Tests/RadiologyTests/SourceFilterTests.swift) | 4 |
| [FilesSidebarRevealTests](../Tests/RadiologyTests/FilesSidebarRevealTests.swift) | 4 |
| [QuickFileSearchTests](../Tests/RadiologyTests/QuickFileSearchTests.swift) | 5 |

All three RepositoryTests functions are reconciled in the
[repository checkpoint](rust-repository-contracts.md), including the ordinary-read
symlink fix. All four SourceFilterTests groups are reconciled in the
[source-filter checkpoint](rust-source-filter-contracts.md), with loaded-document
presentation boundaries explicit. The filename-ordering follow-up is linked below.
Other rows retain their
current audit scope.

All five QuickFileSearchTests functions are reconciled in the
[filename/picker checkpoint](rust-file-order-and-picker-contracts.md), including
exact Unicode ranking, bounded totals and selection/reset assertions.

The four FilesSidebarRevealTests groups have an explicit
[accepted-source outcome mapping](rust-files-reveal-contracts.md), with real
bookmark/ripgrep and nested Before app workflows and platform boundaries recorded.

## Repository search

Rust entry points: [ripgrep.rs](../src/search/ripgrep.rs), [session.rs](../src/search/session.rs), [results.rs](../src/ui/sidebar/tests/results.rs).

Evidence: [rust-repository-search](rust-repository-search.md),
[six-function contracts](rust-search-contracts.md). All six ordinary SearchTests
functions have explicit mappings; cancellation now reports an error at the shared
service boundary, while UI Stop retains its prior presentation policy.

| Swift suite | Annotated functions |
| --- | ---: |
| [SearchTests](../Tests/RadiologyTests/SearchTests.swift) | 6 |

## Pinned review, refresh and changes

Rust entry points: [diff.rs](../src/pull_request/diff.rs), [tests.rs](../src/pull_request/review/tests.rs), [review_refresh.rs](../src/ui/app/tests/review_refresh.rs), [fold_defaults_tests.rs](../src/syntax/fold_defaults_tests.rs).

Evidence: [diff contracts](rust-diff-contracts.md) plus
[change metadata/identity](rust-change-metadata-contracts.md) map all five
UnifiedDiffTests and all three ChangeBlockNavigationTests groups. Other suites
below still need their own reconciliation. Additional evidence:
[rust-byte-change-navigation](rust-byte-change-navigation.md), [rust-pr-refresh-restoration](rust-pr-refresh-restoration.md), [rust-integrated-pr-flow](rust-integrated-pr-flow.md).

| Swift suite | Annotated functions |
| --- | ---: |
| [ChangeBlockNavigationTests](../Tests/RadiologyTests/ChangeBlockNavigationTests.swift) | 3 |
| [UnifiedDiffTests](../Tests/RadiologyTests/UnifiedDiffTests.swift) | 5 |
| [PRRefreshTests](../Tests/RadiologyTests/PRRefreshTests.swift) | 7 |
| [PullRequestReviewTests](../Tests/RadiologyTests/PullRequestReviewTests.swift) | 4 |
| [ReviewChangesPresentationTests](../Tests/RadiologyTests/ReviewChangesPresentationTests.swift) | 3 |
| [ReviewFoldDefaultsTests](../Tests/RadiologyTests/ReviewFoldDefaultsTests.swift) | 7 |
| [ReviewRepositoryTests](../Tests/RadiologyTests/ReviewRepositoryTests.swift) | 4 |

The [PR refresh lifecycle audit](rust-pr-refresh-contracts.md) and
[owned recovery](rust-review-recovery.md) map all seven refresh groups, including
real CLI retries of the failed target. The separate integrated-review suite
listed above remains open. The
[per-file isolation checkpoint](rust-blob-diff-isolation.md) explicitly maps
`ReviewRepositoryTests.renamedBlobDiffExcludesAChildOfTheReusedOldPath` (one of four
repository functions). The follow-up [repository audit](rust-review-repository-contracts.md)
now maps all four, strengthening older-clone fetch, merge-base/bytes and exact
worktree/cache evidence. The [Changes presentation audit](rust-review-presentation-contracts.md)
now maps all three presentation groups and repairs the Before-rename highlight.
The [review fold-default audit](rust-review-fold-contracts.md) maps all seven fold
groups with atomic-source ordering evidence and both-scale held-result workflows.
The separate integrated-review suite remains open. The
[fresh-entry checkpoint](rust-pr-review-entry.md) repairs two assertions in its
browsing flow. [GitHub metadata contracts](rust-github-metadata-contracts.md) now
map `decodesGhSummariesAndTimesOutSlowCommands` (one of four integrated groups);
[integrated browsing](rust-pr-browsing-contracts.md) maps
`opensAPullRequestBrowsesAfterTogglesBeforeAndExits` and repairs unchanged Before
availability/exact-caret returns. The
[changed-block navigation](rust-change-block-flow-contracts.md) now maps
`changedBlockJumpsRevealTestsAndBackUsesCaretNotPassiveScroll` through keyboard and
atomic source ownership checks. [Retired review loads](rust-review-retirement-contracts.md) now map
`errorsAndSupersededLoadsNeverPublishStaleReviews` through the real isolated CLI
workflow and existing forced-publication ownership tests. All four groups are mapped;
other suites and final native acceptance remain open.

## Critical Lens and hover

Rust entry points: [tests.rs](../src/critical/worker/tests.rs), [tests.rs](../src/critical/frontier/tests.rs), [tests.rs](../src/critical/projection/tests.rs), [tests.rs](../src/critical/annotations/tests.rs), [tests.rs](../src/critical/hover/tests.rs), [viewport.rs](../src/ui/source_view/viewport.rs).

Evidence: [rust-critical-projection-reuse](rust-critical-projection-reuse.md), [rust-critical-marks](rust-critical-marks.md), [rust-call-graph](rust-call-graph.md).

| Swift suite | Annotated functions |
| --- | ---: |
| [CallGraphWorkerTests](../Tests/RadiologyTests/CallGraphWorkerTests.swift) | 5 |
| [CriticalFrontierTests](../Tests/RadiologyTests/CriticalFrontierTests.swift) | 3 |
| [CriticalLensModelTests](../Tests/RadiologyTests/CriticalLensModelTests.swift) | 4 |
| [CriticalProjectionTests](../Tests/RadiologyTests/CriticalProjectionTests.swift) | 3 |
| [CriticalSummaryModelTests](../Tests/RadiologyTests/CriticalSummaryModelTests.swift) | 4 |
| [CriticalityOverrideStoreTests](../Tests/RadiologyTests/CriticalityOverrideStoreTests.swift) | 2 |
| [CriticalityPolicyTests](../Tests/RadiologyTests/CriticalityPolicyTests.swift) | 3 |
| [SourceProjectionDrawingTests](../Tests/RadiologyTests/SourceProjectionDrawingTests.swift) | 3 |
| [HoverPopoverInteractionTests](../Tests/RadiologyTests/HoverPopoverInteractionTests.swift) | 2 |

[Policy and override storage](rust-critical-policy-contracts.md) now reconcile all
three CriticalityPolicyTests and both CriticalityOverrideStoreTests groups,
including indexed menu evidence, invalid-load session fallback and real deletion
permission errors. The other Critical Lens rows retain their existing evidence
and still need remaining assertion-level reconciliation.

[Frontier reconciliation](rust-critical-frontier-contracts.md) maps all three
CriticalFrontierTests functions, including conditional lower-bound normalization,
root/nested unknown deduplication, macro/cap chains and a painted conditional hover
status. Other model, worker and hover rows still need their remaining audit.

[Model/projection reconciliation](rust-critical-model-projection-contracts.md)
now maps four CriticalLensModelTests and three CriticalProjectionTests groups.
It fixes unsupported-source publication and strengthens combined app, persistence,
real-fixture and byte-ownership checks. Summary/hover are reconciled below;
drawing and the remaining worker groups retain their separate audit.

[Summary/hover reconciliation](rust-critical-summary-contracts.md) now maps all
four CriticalSummaryModelTests and both HoverPopoverInteractionTests groups,
including no-consumer scheduling, real-fixture count/reference stability, actual
gap/padding hit regions and presented-anchor ownership during replacement dwell.
The first CallGraphWorkerTests group uses the same wire fixture; its remaining
four groups and all three drawing groups still require their separate audit.

[Worker reconciliation](rust-critical-worker-contracts.md) now maps all five
CallGraphWorkerTests groups. Immutable workers retire dependent summaries;
same-length changes, stale digests, real in-flight viewer/structure replacement,
cancellation/reaping and unresolved/dynamic partial coverage pass. The three
SourceProjectionDrawingTests groups retain their separate audit.

[Drawing reconciliation](rust-critical-drawing-contracts.md) maps all three
SourceProjectionDrawingTests groups through actual both-scale/theme frames,
syntax color bounds, selection/pin layers, canonical CRLF copy, folds and source
replacement. All nine suites in this section (29 annotated groups) now have
explicit mappings; other sections and final acceptance remain open.

## Code Paths

Rust entry points: [tests.rs](../src/code_paths/resolver/tests.rs), [tests.rs](../src/code_paths/store/tests.rs), [tests.rs](../src/ui/app/code_paths/tests.rs), [ui_tests.rs](../src/ui/app/code_paths/ui_tests.rs).

Evidence: [rust-code-path-resolution](rust-code-path-resolution.md), [rust-code-path-app](rust-code-path-app.md), [rust-code-path-sidebar](rust-code-path-sidebar.md).

| Swift suite | Annotated functions |
| --- | ---: |
| [CodePathNavigationTests](../Tests/RadiologyTests/CodePathNavigationTests.swift) | 2 |
| [CodePathSymbolTests](../Tests/RadiologyTests/CodePathSymbolTests.swift) | 8 |
| [CodePathTests](../Tests/RadiologyTests/CodePathTests.swift) | 5 |
| [CodePathUITests](../Tests/RadiologyTests/CodePathUITests.swift) | 1 |

[Code Paths reconciliation](rust-code-path-contracts.md) now maps all sixteen
groups in these four suites. Combined saved checkout/PR/relaunch/refresh,
canonical storage and failure preservation, exact symbol variants and both-scale/
theme narrow layouts have explicit evidence. The old ambiguous-file test wrote
outside the repository; its corrected fixture now requires the real diagnostic.
Final whole-app/native acceptance and the sections below remain open.

## Semantic/context/reference identities

Rust entry points: [tests.rs](../src/semantic/context/tests.rs), [index.rs](../src/semantic/index.rs), [caller_tree.rs](../src/semantic/caller_tree.rs), [tests.rs](../src/semantic/incoming_calls/tests.rs), [references.rs](../src/ui/app/tests/focus/references.rs).

Evidence: [rust-semantic-navigation](rust-semantic-navigation.md), [rust-reference-context](rust-reference-context.md), [rust-inspector-previews](rust-inspector-previews.md).

| Swift suite | Annotated functions |
| --- | ---: |
| [ContextModelTests](../Tests/RadiologyTests/ContextModelTests.swift) | 18 |
| [ContextNavigationTests](../Tests/RadiologyTests/ContextNavigationTests.swift) | 9 |
| [SemanticIndexTests](../Tests/RadiologyTests/SemanticIndexTests.swift) | 11 |
| [SemanticNavigationTests](../Tests/RadiologyTests/SemanticNavigationTests.swift) | 4 |
| [ReferenceCallTreeTests](../Tests/RadiologyTests/ReferenceCallTreeTests.swift) | 7 |
| [CallTreePaletteTests](../Tests/RadiologyTests/CallTreePaletteTests.swift) | 1 |

[Semantic caret/selection corrections](rust-semantic-identity-contracts.md) map the
shorthand-field SemanticIndexTests group using the same wire fixture, plus targeted
caret tie/whole-file and invalid-path regressions. Remaining groups in this section
still need reconciliation; tool/root metadata is now retained below.

[SCIP metadata reconciliation](rust-scip-metadata-contracts.md) maps the complete
real-index decode SemanticIndexTests group, including document encodings, function
signature/kind and Unicode docs. Tool records survive decoding/merging/reuse/cache
and an independent host. The subsequent
[complete index contract mapping](rust-semantic-index-contracts.md) reconciles all
eleven groups, including real cross-file/local identities and caret targets,
external/Unicode selection, search ranking, strict coordinate validation,
stale/malformed inputs and typed/unknown wire fields. Other suites in this section
still need their own reconciliation.

[Generated semantic navigation](rust-semantic-navigation-contracts.md) maps all
four SemanticNavigationTests groups through the actual fake-tool generation
pipeline, exact-caret Back/Forward, complete Circle references, workspace symbol
activation and root replacement. Explicit external/definition-here errors now use
the same destination choice as Context, independently of metadata publication.
Other Context/navigation/reference/call-tree suites in this section remain open.

[Context navigation](rust-context-navigation-contracts.md) reconciles all nine
ContextNavigationTests groups, including the repaired declaration-link departure,
real painted distant scrolling, passive Context/preview ownership, named/keyboard/
caller activation, exact Back/Forward, accepted-source retirement and file coverage.
The separate eighteen ContextModel groups and reference/call-tree/palette suites
remain open.

[Context query reuse](rust-context-query-reuse.md) closes missing cached result
reuse identified in the first two ContextModel groups. Backend and both-scale
real-fixture app checks verify stable token/declaration ownership and immediate
preview/reference restoration, with bounds and source/scope invalidation. This is
not yet a complete mapping of the eighteen ContextModel groups. Parsed-structure
reuse across distinct uncached previews is now closed in the checkpoint below.

[Parsed destination structures](rust-preview-structure-reuse.md) maps
ContextModelTests.crossFilePreviewsEndAtTheirTreeSitterDeclaration through real
checkout reads, exact adjacent bounds, plain-text fallback and one extraction for
two targets. Four-entry reuse, changed bytes and contended deadline/UI cancellation
checks pass. The remaining ContextModel groups are reconciled separately below.

[Same-file memory previews](rust-memory-definition-previews.md) maps the complete
ContextModelTests.utf16CaretOffsetsResolveOneIdentityAcrossUnicode group. Metadata-
only app publication proves immediate bounded previews, stable identity through
all valid café offsets, quiet locals, no invented surrogate/string target, source
reload and unchanged history. Prepared ownership and shared worker validation are
checked independently. Two ContextModel groups are mapped; sixteen remain open.

[Context declaration/identity contracts](rust-context-declaration-contracts.md)
map the complete first two ContextModel groups: stable occurrence/declaration
identity, cached exact previews and declaration reference rows/selection through
caret changes. The enclosing summary now prefers the index's Method kind.
Both-scale real-fixture and both-scale/theme summary checks preserve parser
fallback and pin metadata. Four ContextModel groups are mapped; fourteen remain.

[Context fallback contracts](rust-context-fallback-contracts.md) map four more
ContextModel groups: real SCIP implementation links, blank/keyword/absent-structure
fallback, quiet indexed locals and explanatory external-source details. The
existing real-fixture app matrix drives all cases at both scales, including fully
visible explanations and actual reload after the absent-structure input. Eight
ContextModel groups are mapped; ten remain.

[Nested signature presentation](rust-context-signature-contracts.md) maps
ContextModelTests.nestedMultilineSignaturesStopAtTheBodyAndReadFlush. The exact
Swift source paints flush after keyboard disclosure at both scales/themes and
with LF/CRLF; raw-copy bounds and source selection survive shared limiting.
The exact escaped impl/core-URL, numeric serde_json version and local-package
cases additionally map escapedImplDescriptorsAndURLVersionsReadCleanly through
existing Context preparation. Ten groups are mapped; eight remain unaudited.

[Context lifecycle ownership](rust-context-lifecycle-contracts.md) maps the four
stale-publication/reset/index-replacement/changed-source ContextModel groups. Real
result endpoints reject late metadata across caret, reset, root and index changes;
identical-source roots cannot share row ownership, and source edits refresh both
excerpts and definition snippets under one index. Fourteen groups have mappings;
explicit focus, scalar-window snippets, availability and expansion remain open.

[Context disclosures and explicit focus](rust-context-disclosure-contracts.md)
map expansionBelongsToIdentityAndStartsCollapsed and
explicitReferencesOverrideTheDeclarationUntilFollowCaret. Actual chevrons,
Find References and Follow Caret drive both-scale checks. Index replacement now
retires the signature using existing semantic-input ownership. Sixteen groups
are mapped, leaving scalar-window excerpts and coverage.

## Source, syntax, projection and test regions

Rust entry points: [buffer.rs](../src/source/buffer.rs), [test_code.rs](../src/source/test_code.rs), [native_tests.rs](../src/syntax/outline/native_tests.rs), [tests.rs](../src/syntax/fold_queries/tests.rs), [test_detector.rs](../src/syntax/test_detector.rs), [corpus.rs](../src/ui/app/tests/corpus.rs).

Evidence: [rust-integrated-source-corpus](rust-integrated-source-corpus.md), [rust-native-outline-parity](rust-native-outline-parity.md), [rust-inline-test-parity](rust-inline-test-parity.md).

| Swift suite | Annotated functions |
| --- | ---: |
| [CSyntaxTests](../Tests/RadiologyTests/CSyntaxTests.swift) | 4 |
| [CallSiteExtractionTests](../Tests/RadiologyTests/CallSiteExtractionTests.swift) | 4 |
| [DocumentOutlineTests](../Tests/RadiologyTests/DocumentOutlineTests.swift) | 3 |
| [SourceBufferTests](../Tests/RadiologyTests/SourceBufferTests.swift) | 4 |
| [SourceLanguageRegistryTests](../Tests/RadiologyTests/SourceLanguageRegistryTests.swift) | 5 |
| [SyntaxLanguageTests](../Tests/RadiologyTests/SyntaxLanguageTests.swift) | 12 |
| [SyntaxQueryTests](../Tests/RadiologyTests/SyntaxQueryTests.swift) | 5 |
| [SyntaxWorkerTests](../Tests/RadiologyTests/SyntaxWorkerTests.swift) | 9 |
| [TestCodePresentationTests](../Tests/RadiologyTests/TestCodePresentationTests.swift) | 7 |
| [TestRegionDetectorTests](../Tests/RadiologyTests/TestRegionDetectorTests.swift) | 13 |

## Compiler lifecycle and personalities

Rust entry points: [real_tools.rs](../src/semantic/indexing/service/tests/real_tools.rs), [reuse.rs](../src/semantic/indexing/service/tests/reuse.rs), [tests.rs](../src/semantic/indexing/personality/tests.rs), [personality.rs](../src/ui/app/tests/personality.rs), [kotlin.rs](../src/ui/app/tests/kotlin.rs).

Evidence: [rust-indexing-parity](rust-indexing-parity.md), [rust-indexing-personalities](rust-indexing-personalities.md), [rust-gradle-android-acceptance](rust-gradle-android-acceptance.md), [rust-kotlin-pr-fixture](rust-kotlin-pr-fixture.md).

| Swift suite | Annotated functions |
| --- | ---: |
| [CScipTests](../Tests/RadiologyTests/CScipTests.swift) | 5 |
| [ClangFallbackTests](../Tests/RadiologyTests/ClangFallbackTests.swift) | 3 |
| [IndexReuseTests](../Tests/RadiologyTests/IndexReuseTests.swift) | 3 |
| [IndexingTests](../Tests/RadiologyTests/IndexingTests.swift) | 10 |
| [KotlinIndexingTests](../Tests/RadiologyTests/KotlinIndexingTests.swift) | 6 |
| [PRKotlinNavigationTests](../Tests/RadiologyTests/PRKotlinNavigationTests.swift) | 2 |
| [IndexingPersonalityTests](../Tests/RadiologyTests/IndexingPersonalityTests.swift) | 5 |
| [IndexingPersonalityUITests](../Tests/RadiologyTests/IndexingPersonalityUITests.swift) | 1 |

## Inspector regions and trait implementations

Rust entry points: [panes.rs](../src/ui/app/tests/focus/panes.rs), [traits.rs](../src/ui/app/tests/focus/traits.rs), [tests.rs](../src/semantic/trait_impls/tests.rs), [tests.rs](../src/semantic/trait_context/tests.rs).

Evidence: [rust-inspector-region-sizing](rust-inspector-region-sizing.md), [rust-trait-inspector](rust-trait-inspector.md).

| Swift suite | Annotated functions |
| --- | ---: |
| [ContextInspectorLayoutTests](../Tests/RadiologyTests/ContextInspectorLayoutTests.swift) | 1 |
| [InspectorPaneTests](../Tests/RadiologyTests/InspectorPaneTests.swift) | 4 |
| [TraitImplementationContextTests](../Tests/RadiologyTests/TraitImplementationContextTests.swift) | 5 |
| [TraitImplementationTests](../Tests/RadiologyTests/TraitImplementationTests.swift) | 5 |

## Opt-in external probes

These11 reference files require particular repositories, caches, GitHub metadata or tool environments. Existing bounded Rust/native/real-tool evidence is linked above; it does not claim every external Swift corpus or platform run.

| Probe | Annotated functions |
| --- | ---: |
| [CRealDataProbe](../Tests/RadiologyTests/CRealDataProbe.swift) | 2 |
| [CodePathRealDataProbe](../Tests/RadiologyTests/CodePathRealDataProbe.swift) | 2 |
| [ContextRealDataProbe](../Tests/RadiologyTests/ContextRealDataProbe.swift) | 1 |
| [CriticalRealDataProbe](../Tests/RadiologyTests/CriticalRealDataProbe.swift) | 1 |
| [IndexPerformanceProbe](../Tests/RadiologyTests/IndexPerformanceProbe.swift) | 1 |
| [KotlinRealDataProbe](../Tests/RadiologyTests/KotlinRealDataProbe.swift) | 2 |
| [PullRequestRealDataProbe](../Tests/RadiologyTests/PullRequestRealDataProbe.swift) | 2 |
| [Sel4IndexProbe](../Tests/RadiologyTests/Sel4IndexProbe.swift) | 1 |
| [Sel4PersonalityProbe](../Tests/RadiologyTests/Sel4PersonalityProbe.swift) | 1 |
| [TestFilterRealDataProbe](../Tests/RadiologyTests/TestFilterRealDataProbe.swift) | 3 |
| [TraitImplementationRealDataProbe](../Tests/RadiologyTests/TraitImplementationRealDataProbe.swift) | 1 |

Support files: GitFixture, ObservationFlag, PreparedClangIndexer, SyntaxFixtures.

## Concrete remaining gates

1. Finish assertion-level reconciliation in the indexed groups; repair differences rather than marking a whole group complete from its links.
2. Successful SDK-backed Android debug/unit-test generation remains unverified. No local SDK was found. A private Platform36/Build Tools36.0.0/command-line setup is prepared; downloading requires the user’s authorization to accept Google’s SDK license. Missing-SDK failure acceptance already passes.
3. [Clean build/resource/external-host acceptance](rust-reader-host-audit.md) now passes for this audited state; rerun affected gates after further runtime/API/resource changes. Actual little-screen-clip adoption and full assistive accessibility remain deferred.
