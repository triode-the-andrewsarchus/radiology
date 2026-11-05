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

## Bookmarks

Rust entry points: [store.rs](../src/bookmarks/store.rs), [tests.rs](../src/ui/app/tests.rs), [bookmark_list.rs](../src/ui/bookmark_list.rs).

Evidence: [rust-bookmark-pane](rust-bookmark-pane.md).

| Swift suite | Annotated functions |
| --- | ---: |
| [BookmarkTests](../Tests/RadiologyTests/BookmarkTests.swift) | 3 |
| [BookmarkNavigationTests](../Tests/RadiologyTests/BookmarkNavigationTests.swift) | 3 |

## Repository, filters, Files reveal and quick navigation

Rust entry points: [tests.rs](../src/repository/discovery/tests.rs), [filters.rs](../src/repository/filters.rs), [quick_search.rs](../src/repository/quick_search.rs), [files.rs](../src/ui/app/tests/focus/files.rs).

Evidence: [rust-repository-opening](rust-repository-opening.md), [rust-unicode-file-search](rust-unicode-file-search.md).

| Swift suite | Annotated functions |
| --- | ---: |
| [RepositoryTests](../Tests/RadiologyTests/RepositoryTests.swift) | 3 |
| [SourceFilterTests](../Tests/RadiologyTests/SourceFilterTests.swift) | 4 |
| [FilesSidebarRevealTests](../Tests/RadiologyTests/FilesSidebarRevealTests.swift) | 4 |
| [QuickFileSearchTests](../Tests/RadiologyTests/QuickFileSearchTests.swift) | 5 |

## Repository search

Rust entry points: [ripgrep.rs](../src/search/ripgrep.rs), [session.rs](../src/search/session.rs), [results.rs](../src/ui/sidebar/tests/results.rs).

Evidence: [rust-repository-search](rust-repository-search.md).

| Swift suite | Annotated functions |
| --- | ---: |
| [SearchTests](../Tests/RadiologyTests/SearchTests.swift) | 6 |

## Pinned review, refresh and changes

Rust entry points: [diff.rs](../src/pull_request/diff.rs), [tests.rs](../src/pull_request/review/tests.rs), [review_refresh.rs](../src/ui/app/tests/review_refresh.rs), [fold_defaults_tests.rs](../src/syntax/fold_defaults_tests.rs).

Evidence: [rust-byte-change-navigation](rust-byte-change-navigation.md), [rust-pr-refresh-restoration](rust-pr-refresh-restoration.md), [rust-integrated-pr-flow](rust-integrated-pr-flow.md).

| Swift suite | Annotated functions |
| --- | ---: |
| [ChangeBlockNavigationTests](../Tests/RadiologyTests/ChangeBlockNavigationTests.swift) | 3 |
| [UnifiedDiffTests](../Tests/RadiologyTests/UnifiedDiffTests.swift) | 5 |
| [PRRefreshTests](../Tests/RadiologyTests/PRRefreshTests.swift) | 7 |
| [PullRequestReviewTests](../Tests/RadiologyTests/PullRequestReviewTests.swift) | 4 |
| [ReviewChangesPresentationTests](../Tests/RadiologyTests/ReviewChangesPresentationTests.swift) | 3 |
| [ReviewFoldDefaultsTests](../Tests/RadiologyTests/ReviewFoldDefaultsTests.swift) | 7 |
| [ReviewRepositoryTests](../Tests/RadiologyTests/ReviewRepositoryTests.swift) | 4 |

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

## Code Paths

Rust entry points: [tests.rs](../src/code_paths/resolver/tests.rs), [tests.rs](../src/code_paths/store/tests.rs), [tests.rs](../src/ui/app/code_paths/tests.rs), [ui_tests.rs](../src/ui/app/code_paths/ui_tests.rs).

Evidence: [rust-code-path-resolution](rust-code-path-resolution.md), [rust-code-path-app](rust-code-path-app.md), [rust-code-path-sidebar](rust-code-path-sidebar.md).

| Swift suite | Annotated functions |
| --- | ---: |
| [CodePathNavigationTests](../Tests/RadiologyTests/CodePathNavigationTests.swift) | 2 |
| [CodePathSymbolTests](../Tests/RadiologyTests/CodePathSymbolTests.swift) | 8 |
| [CodePathTests](../Tests/RadiologyTests/CodePathTests.swift) | 5 |
| [CodePathUITests](../Tests/RadiologyTests/CodePathUITests.swift) | 1 |

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
