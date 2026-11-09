# Final Swift/Rust requirement audit — 2026-11-09

Reference: main 7edfc79db6343707e4138c414f53f30db5de2411, including the upstream
publication incorporated on November 2. Production snapshot: a5fd157; the following
7ea44ee checkpoint records its completed runtime verification without changing code.
The original [October 27 review](rust-swift-parity-review-2026-10-27.md) and
[upstream expansion](rust-upstream-parity-2026-11-02.md) retain the requirements;
this document records their disposition rather than replacing the scope.

Desktop feature and broad UI implementation is accepted for the evidence below.
SDK-backed Android success remains **unverified**; overall completion is therefore
not claimed. Full accessibility and actual little-screen-clip adoption are deferred
by the user's instructions. Keyboard shortcuts remain required and verified.

## Original findings

The [detailed R1–R17 implementation ledger](rust-parity-status-2026-11-02.md#original-finding-reconciliation)
identifies production owners and concrete assertions. The table here gives each
finding its final evidence boundary. Historical counts in earlier checkpoints
refer to their own snapshots; the current full gate is 943 app checks/twelve
opt-in ignores, binary/font/text 2/13/23 and 68 doctests. Both serial and sixteen-job/
sixteen-test-thread runs pass. [Fresh runtime evidence](rust-final-runtime-acceptance.md)
refreshes affected paths after the last runtime fix.

| Requirement | Disposition and authoritative evidence |
| --- | --- |
| R1 pinned review source | Accepted: real Git/native PR checks distinguish pinned After, merge-base Before and dirty attached checkout. Both-scale actual Clang generation/navigation and exact Copy pass. |
| R2 file-specific exact changes | Accepted: zero-context per-file overlays, added/deleted/rename/binary cases, moving-head/base mapping and changed-block app contracts. See [diff contracts](rust-diff-contracts.md). |
| R3 repository state retirement | Accepted: transactional source/repository/review ownership, cancellation and stale-publication app tests; native superseded load retains the accepted reader. |
| R4 compiler/index lifecycle | Desktop adapters/personality lifecycle accepted: automatic generation, tools, isolation, provenance, failure retention, cancellation, reuse and tool-free restart. Actual small compiler runs and real 67-profile seL4 union retain their scopes. Successful Android SDK/Gradle compilation remains unverified. |
| R5 semantic identities/encoding | Accepted: document-local identity, half-open ranges, UTF-8/16/32 conversion and exact navigation; actual compiler fixtures supplement recorded facts. |
| R6 Critical Lens | Accepted: semantic policy/projection, parsed calls, persisted marks, frontier/hover and fold counts; [Critical contracts](rust-critical-model-projection-contracts.md) identify actual assertions and bounded graph behavior. |
| R7 exact navigation/history/reveal | Accepted: shared source transaction preserves byte/revision/departure, mapped refresh and folded destinations; native PR, source, seL4 and Code Path checks exercise actual Back/Copy. |
| R8 keyboard/focus | Accepted: exact commands, modal/menu/edit ownership, Return/keypad routing, composition and ordered input. Real IME and native keyboard acceptance remain distinct from frame tests. |
| R9 bookmarks | Accepted: durable mutations, anchors and independent checkout/PR scopes, actual row/shortcut/drag flows and restart. [Contracts](rust-bookmark-navigation-contracts.md). |
| R10 grammars/structure/test hiding | Accepted: native language queries and recovered C/C++/Kotlin declarations, AST test regions and original-byte projections. Exact fixture assertions plus representative native recovery/folding pass. |
| R11 bounded asynchronous work | Accepted for representative contract: cancellable reads/processes, bounded normalization/paint and actual 5.85 MB/60,002-line workflows. No universal timing or arbitrary external-corpus guarantee is inferred. |
| R12 semantic reference/caller meaning | Accepted: document/index identity, parsed call validation, explicit reference independence and bounded expansion. All Context and caller-tree mappings link assertion-level checkpoints. |
| R13 simultaneous inspector regions | Accepted: actual both-scale resize drags, independent scrolling, persisted pane combinations, under-caret context and trait rows. [Inspector/trait contract](rust-inspector-trait-contracts.md). |
| R14 source interaction | Accepted: original-byte selection/Copy, find, wrap, zoom, pins and caret. Fresh native stress verifies exact bytes after wrap/zoom/reset, line 60001 navigation and Back. |
| R15 operational controls | Accepted: source/file/search/change controls, progress/errors, PR chooser, Stop, globs, Code Paths, Help and file-manager handoffs. Actual portal chooser/Cancel/retry/import/restart passes separately. |
| R16 restoration/recovery | Accepted: seven saved-session contracts, explicit launch precedence, unavailable-root retry, failure/cancel retention and exact refresh mapping; native warm restart and failure flows supplement app tests. |
| R17 evidence and honest limits | Current full serial/parallel, fresh committed export, independent hosts and affected native gates pass. Fixture/process/frame/native/compiler boundaries and unverified Android/external-corpus limits remain explicit. |

## Upstream additions and reuse

Current-function x folding, committed Files reveal, persistent inspector choices,
trait implementations, Code Path storage/sidebar/import/resolution, C/C++ recovery/
best-effort compilation, personalities/seL4 and complete keyboard Help each have
linked contracts in the [upstream matrix](rust-upstream-parity-2026-11-02.md).
Later source, inspector/trait, compiler and final runtime checkpoints close that
matrix's historical pending notes for desktop behavior. Actual IME-engine and
portal acceptance are documented separately in the status ledger; they are not
inferred from unit tests.

The app builds independently from a committed archive. Full-reader and asset/text
external consumers use shared reader-text/reader-fonts and existing public reader
APIs; copied binaries run with export sources unavailable. The asset/text consumer
has no app/GUI/Wayland/portal dependency. No sibling product dependency or edit is
introduced. [Host boundary](rust-reader-host-audit.md),
[fresh export](rust-final-standalone-acceptance.md),
[latest refresh](rust-final-runtime-acceptance.md).

Architecture remains an explicit constraint: the latest production fix reuses the
existing index job predicate and reduces coordinator CCN 15→13, with no new worker,
state owner, API or dependency. Its normalized comparison retains all 125 clone
instances without additions, average/max CCN 3.43/132, 36 high-complexity functions
and one existing cycle. This is evidence about the delta, not a claim of zero
historical debt. No cleanup is added solely to improve a metric.

## Complete ordinary-suite traceability

The inventory rechecks all 81 root Swift file hashes against the original recorded
inventory: 66 ordinary suites/317 annotated functions, eleven optional probe files/
seventeen functions and four support files. Reference bytes are unchanged. The
following index connects every ordinary suite to its assertion-level checkpoints;
58 distinct contract documents are used. Some ordinary-suite functions are also
conditional, including the absolute-macOS fold performance probe.

This is not execution of Swift/AppKit on Linux. Counts and name searches establish
traceability, not behavioral proof by themselves. Each contract supplies the actual
assertions, Rust tests, real tools and native evidence. The final archive retains
function-map.json with every method, reference hash, contract hash and seventeen
exact Rust test links for methods previously described without literal Swift names.
Archive: ~/.local/state/radiology-qa/final-requirement-audit-20261109-a5fd157/.

| Swift suite | Functions | Assertion-level contract |
| --- | ---: | --- |
| [AppModelTests](../Tests/RadiologyTests/AppModelTests.swift) | 6 | [app-model-contracts](rust-app-model-contracts.md) |
| [BookmarkNavigationTests](../Tests/RadiologyTests/BookmarkNavigationTests.swift) | 3 | [bookmark-navigation-contracts](rust-bookmark-navigation-contracts.md) |
| [BookmarkTests](../Tests/RadiologyTests/BookmarkTests.swift) | 3 | [bookmark-model-contracts](rust-bookmark-model-contracts.md) |
| [CScipTests](../Tests/RadiologyTests/CScipTests.swift) | 5 | [clang-contracts](rust-clang-contracts.md) |
| [CSyntaxTests](../Tests/RadiologyTests/CSyntaxTests.swift) | 4 | [source-language-contracts](rust-source-language-contracts.md) |
| [CallGraphWorkerTests](../Tests/RadiologyTests/CallGraphWorkerTests.swift) | 5 | [critical-worker-contracts](rust-critical-worker-contracts.md) |
| [CallSiteExtractionTests](../Tests/RadiologyTests/CallSiteExtractionTests.swift) | 4 | [callsite-extraction-contracts](rust-callsite-extraction-contracts.md) |
| [CallTreePaletteTests](../Tests/RadiologyTests/CallTreePaletteTests.swift) | 1 | [call-tree-contracts](rust-call-tree-contracts.md) |
| [ChangeBlockNavigationTests](../Tests/RadiologyTests/ChangeBlockNavigationTests.swift) | 3 | [diff-contracts](rust-diff-contracts.md) |
| [ClangFallbackTests](../Tests/RadiologyTests/ClangFallbackTests.swift) | 3 | [clang-contracts](rust-clang-contracts.md) |
| [CodePathNavigationTests](../Tests/RadiologyTests/CodePathNavigationTests.swift) | 2 | [code-path-contracts](rust-code-path-contracts.md) |
| [CodePathSymbolTests](../Tests/RadiologyTests/CodePathSymbolTests.swift) | 8 | [code-path-contracts](rust-code-path-contracts.md) |
| [CodePathTests](../Tests/RadiologyTests/CodePathTests.swift) | 5 | [code-path-contracts](rust-code-path-contracts.md) |
| [CodePathUITests](../Tests/RadiologyTests/CodePathUITests.swift) | 1 | [code-path-contracts](rust-code-path-contracts.md) |
| [ContextInspectorLayoutTests](../Tests/RadiologyTests/ContextInspectorLayoutTests.swift) | 1 | [inspector-trait-contracts](rust-inspector-trait-contracts.md) |
| [ContextModelTests](../Tests/RadiologyTests/ContextModelTests.swift) | 18 | [context-declaration-contracts](rust-context-declaration-contracts.md), [context-disclosure-contracts](rust-context-disclosure-contracts.md), [context-fallback-contracts](rust-context-fallback-contracts.md), [context-lifecycle-contracts](rust-context-lifecycle-contracts.md), [context-signature-contracts](rust-context-signature-contracts.md), [context-window-contracts](rust-context-window-contracts.md), [memory-definition-previews](rust-memory-definition-previews.md), [preview-structure-reuse](rust-preview-structure-reuse.md) |
| [ContextNavigationTests](../Tests/RadiologyTests/ContextNavigationTests.swift) | 9 | [context-navigation-contracts](rust-context-navigation-contracts.md) |
| [CriticalFrontierTests](../Tests/RadiologyTests/CriticalFrontierTests.swift) | 3 | [critical-frontier-contracts](rust-critical-frontier-contracts.md) |
| [CriticalLensModelTests](../Tests/RadiologyTests/CriticalLensModelTests.swift) | 4 | [critical-model-projection-contracts](rust-critical-model-projection-contracts.md) |
| [CriticalProjectionTests](../Tests/RadiologyTests/CriticalProjectionTests.swift) | 3 | [critical-model-projection-contracts](rust-critical-model-projection-contracts.md) |
| [CriticalSummaryModelTests](../Tests/RadiologyTests/CriticalSummaryModelTests.swift) | 4 | [critical-summary-contracts](rust-critical-summary-contracts.md) |
| [CriticalityOverrideStoreTests](../Tests/RadiologyTests/CriticalityOverrideStoreTests.swift) | 2 | [critical-policy-contracts](rust-critical-policy-contracts.md) |
| [CriticalityPolicyTests](../Tests/RadiologyTests/CriticalityPolicyTests.swift) | 3 | [critical-policy-contracts](rust-critical-policy-contracts.md) |
| [DocumentOutlineTests](../Tests/RadiologyTests/DocumentOutlineTests.swift) | 3 | [buffer-outline-contracts](rust-buffer-outline-contracts.md) |
| [FilesSidebarRevealTests](../Tests/RadiologyTests/FilesSidebarRevealTests.swift) | 4 | [files-reveal-contracts](rust-files-reveal-contracts.md) |
| [FunctionShadeTests](../Tests/RadiologyTests/FunctionShadeTests.swift) | 3 | [function-shading-audit](rust-function-shading-audit.md) |
| [HoverPopoverInteractionTests](../Tests/RadiologyTests/HoverPopoverInteractionTests.swift) | 2 | [critical-summary-contracts](rust-critical-summary-contracts.md) |
| [IndexReuseTests](../Tests/RadiologyTests/IndexReuseTests.swift) | 3 | [index-reuse-contracts](rust-index-reuse-contracts.md) |
| [IndexingPersonalityTests](../Tests/RadiologyTests/IndexingPersonalityTests.swift) | 5 | [personality-contracts](rust-personality-contracts.md) |
| [IndexingPersonalityUITests](../Tests/RadiologyTests/IndexingPersonalityUITests.swift) | 1 | [personality-contracts](rust-personality-contracts.md) |
| [IndexingTests](../Tests/RadiologyTests/IndexingTests.swift) | 10 | [indexing-contracts](rust-indexing-contracts.md), [real-rust-contract](rust-real-rust-contract.md) |
| [InspectorPaneTests](../Tests/RadiologyTests/InspectorPaneTests.swift) | 4 | [inspector-trait-contracts](rust-inspector-trait-contracts.md) |
| [KotlinIndexingTests](../Tests/RadiologyTests/KotlinIndexingTests.swift) | 6 | [kotlin-contracts](rust-kotlin-contracts.md) |
| [LineNumberGutterTests](../Tests/RadiologyTests/LineNumberGutterTests.swift) | 2 | [gutter-parity](rust-gutter-parity.md) |
| [NavigationHistoryTests](../Tests/RadiologyTests/NavigationHistoryTests.swift) | 1 | [scroll-history-contracts](rust-scroll-history-contracts.md) |
| [OccurrencePinningTests](../Tests/RadiologyTests/OccurrencePinningTests.swift) | 4 | [reader-host-audit](rust-reader-host-audit.md) |
| [PRKotlinNavigationTests](../Tests/RadiologyTests/PRKotlinNavigationTests.swift) | 2 | [kotlin-contracts](rust-kotlin-contracts.md) |
| [PRRefreshTests](../Tests/RadiologyTests/PRRefreshTests.swift) | 7 | [pr-refresh-contracts](rust-pr-refresh-contracts.md) |
| [PullRequestReviewTests](../Tests/RadiologyTests/PullRequestReviewTests.swift) | 4 | [change-block-flow-contracts](rust-change-block-flow-contracts.md), [github-metadata-contracts](rust-github-metadata-contracts.md), [pr-browsing-contracts](rust-pr-browsing-contracts.md), [review-retirement-contracts](rust-review-retirement-contracts.md) |
| [QuickFileSearchTests](../Tests/RadiologyTests/QuickFileSearchTests.swift) | 5 | [file-order-and-picker-contracts](rust-file-order-and-picker-contracts.md) |
| [ReaderFunctionFoldKeyTests](../Tests/RadiologyTests/ReaderFunctionFoldKeyTests.swift) | 4 | [function-contracts](rust-function-contracts.md) |
| [ReaderReturnKeyTests](../Tests/RadiologyTests/ReaderReturnKeyTests.swift) | 2 | [reader-return-selection](rust-reader-return-selection.md) |
| [ReaderSessionTests](../Tests/RadiologyTests/ReaderSessionTests.swift) | 7 | [session-recovery](rust-session-recovery.md) |
| [ReferenceCallTreeTests](../Tests/RadiologyTests/ReferenceCallTreeTests.swift) | 7 | [call-tree-contracts](rust-call-tree-contracts.md) |
| [RepositoryTests](../Tests/RadiologyTests/RepositoryTests.swift) | 3 | [repository-contracts](rust-repository-contracts.md) |
| [ReviewChangesPresentationTests](../Tests/RadiologyTests/ReviewChangesPresentationTests.swift) | 3 | [review-presentation-contracts](rust-review-presentation-contracts.md) |
| [ReviewFoldDefaultsTests](../Tests/RadiologyTests/ReviewFoldDefaultsTests.swift) | 7 | [review-fold-contracts](rust-review-fold-contracts.md) |
| [ReviewRepositoryTests](../Tests/RadiologyTests/ReviewRepositoryTests.swift) | 4 | [review-repository-contracts](rust-review-repository-contracts.md) |
| [SearchTests](../Tests/RadiologyTests/SearchTests.swift) | 6 | [search-contracts](rust-search-contracts.md) |
| [SemanticIndexTests](../Tests/RadiologyTests/SemanticIndexTests.swift) | 11 | [semantic-index-contracts](rust-semantic-index-contracts.md) |
| [SemanticNavigationTests](../Tests/RadiologyTests/SemanticNavigationTests.swift) | 4 | [semantic-navigation-contracts](rust-semantic-navigation-contracts.md) |
| [SourceBufferTests](../Tests/RadiologyTests/SourceBufferTests.swift) | 4 | [buffer-outline-contracts](rust-buffer-outline-contracts.md) |
| [SourceCaretTests](../Tests/RadiologyTests/SourceCaretTests.swift) | 3 | [caret-parity](rust-caret-parity.md) |
| [SourceFilterTests](../Tests/RadiologyTests/SourceFilterTests.swift) | 4 | [source-filter-contracts](rust-source-filter-contracts.md) |
| [SourceFunctionNavigationTests](../Tests/RadiologyTests/SourceFunctionNavigationTests.swift) | 1 | [function-contracts](rust-function-contracts.md) |
| [SourceLanguageRegistryTests](../Tests/RadiologyTests/SourceLanguageRegistryTests.swift) | 5 | [source-language-contracts](rust-source-language-contracts.md) |
| [SourceProjectionDrawingTests](../Tests/RadiologyTests/SourceProjectionDrawingTests.swift) | 3 | [critical-drawing-contracts](rust-critical-drawing-contracts.md) |
| [SourceScrollViewTests](../Tests/RadiologyTests/SourceScrollViewTests.swift) | 12 | [scroll-history-contracts](rust-scroll-history-contracts.md) |
| [SyntaxLanguageTests](../Tests/RadiologyTests/SyntaxLanguageTests.swift) | 12 | [syntax-language-query-contracts](rust-syntax-language-query-contracts.md) |
| [SyntaxQueryTests](../Tests/RadiologyTests/SyntaxQueryTests.swift) | 5 | [syntax-language-query-contracts](rust-syntax-language-query-contracts.md) |
| [SyntaxWorkerTests](../Tests/RadiologyTests/SyntaxWorkerTests.swift) | 9 | [syntax-worker-contracts](rust-syntax-worker-contracts.md) |
| [TestCodePresentationTests](../Tests/RadiologyTests/TestCodePresentationTests.swift) | 7 | [test-presentation-contracts](rust-test-presentation-contracts.md) |
| [TestRegionDetectorTests](../Tests/RadiologyTests/TestRegionDetectorTests.swift) | 13 | [test-region-contracts](rust-test-region-contracts.md) |
| [TraitImplementationContextTests](../Tests/RadiologyTests/TraitImplementationContextTests.swift) | 5 | [inspector-trait-contracts](rust-inspector-trait-contracts.md) |
| [TraitImplementationTests](../Tests/RadiologyTests/TraitImplementationTests.swift) | 5 | [inspector-trait-contracts](rust-inspector-trait-contracts.md) |
| [UnifiedDiffTests](../Tests/RadiologyTests/UnifiedDiffTests.swift) | 5 | [diff-contracts](rust-diff-contracts.md) |

## Supplemental exact-method links

These seventeen ordinary methods already had descriptive contract coverage. This
appendix records their exact Swift names and current Rust test declarations so a
future audit does not mistake absent literal names for absent behavior. Multiple
session methods share one regression because it checks the complete lifecycle.

| Swift method | Rust declaration |
| --- | --- |
| NavigationHistoryTests.backForwardRestoreLocationsAndNewVisitsDiscardFuture | [test_navigation_history](../src/source/history.rs) |
| OccurrencePinningTests.multiplePinsPersistAndUnpinIndependentlyWithoutRecoloring | [pin_groups_have_no_eviction_limit_and_removal_does_not_recolor_survivors](../src/source/selection.rs) |
| OccurrencePinningTests.pinsArePerFileSurviveRevisitAndInvalidateOnChangedBytes | [document_restoration_requires_identical_content_and_revision_and_clears_ephemeral_state](../src/source/selection.rs), [occurrence_pins_restore_on_return_and_invalidate_only_after_successful_changed_source](../src/ui/app/tests.rs) |
| OccurrencePinningTests.semanticArrivalCanToggleTheEquivalentFallbackGroup | [syntactic_pins_include_cfg_omissions_but_prefer_complete_semantic_identity](../src/ui/app/tests.rs) |
| OccurrencePinningTests.androidMessagePinsFourTokensThenWhitespaceAndAnotherSelectionDoNotRemoveIt | [lexical_pins_use_whole_identifiers_and_the_enclosing_function](../src/ui/app/tests.rs) |
| ReaderReturnKeyTests.returnFollowsTheSemanticCallAtTheCaretOrSelection | [return_uses_one_utf16_occurrence_and_preserves_selection_or_falls_through](../src/ui/source_view/viewport.rs), [source_return_rejects_a_selection_crossing_an_occurrence_boundary](../src/ui/app/tests/focus.rs) |
| ReaderReturnKeyTests.returnNavigatesAndBackRestoresTheExactCaretAfterScrolling | [source_return_follows_word_or_end_caret_and_back_uses_the_swift_departure](../src/ui/app/tests/focus.rs) |
| ReaderSessionTests.relaunchRestoresTheAttachedRepositoryAndConfirmedPR | [saved_review_identity_survives_checkout_preparation_failure_cancel_and_fresh_restore](../src/ui/app/tests/repository_open.rs) |
| ReaderSessionTests.exitPRRemembersLocalCheckoutOnTheNextLaunch | [saved_review_identity_survives_checkout_preparation_failure_cancel_and_fresh_restore](../src/ui/app/tests/repository_open.rs) |
| ReaderSessionTests.explicitLaunchArgumentsOverrideTheSavedSession | [explicit_launch_root_supersedes_saved_review_without_starting_it](../src/ui/app/tests.rs) |
| ReaderSessionTests.failedRestoreKeepsThePRForRetry | [saved_review_identity_survives_checkout_preparation_failure_cancel_and_fresh_restore](../src/ui/app/tests/repository_open.rs) |
| ReaderSessionTests.failedAndCancelledPRChoicesDoNotReplaceTheConfirmedSession | [saved_review_identity_survives_checkout_preparation_failure_cancel_and_fresh_restore](../src/ui/app/tests/repository_open.rs) |
| ReaderSessionTests.newerRepositoryChoiceWinsAndLaunchOnlyRunsOnce | [newer_repository_choice_retires_launch_intent_and_repeated_launch_is_ignored](../src/ui/app/tests/repository_open.rs) |
| ReaderSessionTests.missingRepositoryAndInvalidPreferencesHaveSafeRecovery | [missing_saved_repository_keeps_recovery_identity_and_invalid_sessions_remain_empty](../src/ui/app/tests/repository_open.rs) |
| SourceCaretTests.caretFollowsTypesetGlyphsAcrossTabsEmojiLineEndsAndEOF | [insertion_caret_geometry_follows_tabs_graphemes_line_ends_and_eof](../src/ui/source_view/viewport.rs) |
| SourceCaretTests.caretIsHiddenInsideFoldedText | [insertion_caret_skips_folded_bytes_and_empty_documents](../src/ui/source_view/viewport.rs) |
| SourceCaretTests.caretShowsOnlyForAnEmptySelectionInTheFocusedReader | [insertion_caret_requires_empty_selection_reader_ownership_and_active_window](../src/ui/source_view/viewport.rs) |

## Conditional external probes and remaining work

| Swift probe family | Current evidence and explicit boundary |
| --- | --- |
| CRealDataProbe (2) | Actual Clang small corpora, representative C/C++ and real seL4 evidence. Dependency-heavy external repositories retain documented Swift/Clang limitations; no universal compilation claim. |
| CodePathRealDataProbe (2) | Supplied real seL4 trace: 91 steps/93 destinations through app navigation, plus native sidebar/import/restart acceptance. |
| ContextRealDataProbe (1), CriticalRealDataProbe (1) | Optional diagnostic dumps against a separately supplied tkos cache. Context's eighteen groups and Critical assertions are reconciled; no claim that this machine executed the author's private cache/paths. |
| IndexPerformanceProbe (1) | Optional supplied-cache decode/search observations, with an additional PreparedApp assertion for tkos. Current decoder/navigation/cache and stress evidence establish portable behavior; that exact private cache and timing are not claimed. |
| KotlinRealDataProbe (2) | Recorded real PR Kotlin navigation and actual multi-module JVM generation pass. SDK-backed Android PR generation remains unverified and is required before overall completion. |
| PullRequestRealDataProbe (2) | Real Git/Clang and controlled gh metadata, pinned review/cache/reuse/invalidation native flows pass. This is not authenticated remote GitHub testing. |
| Sel4IndexProbe (1), Sel4PersonalityProbe (1) | Real 67-profile preparation/compiler union, original coordinates, isolated service generation and tool-free cache reload; native navigation/Reindex at both scales. |
| TestFilterRealDataProbe (3) | Exact checked-in AST/path fixtures and real supported-language filters are accepted. The original absolute /Users/tqbf/... tkos sweep/line anchors are external-corpus-specific and not claimed executed here. |
| TraitImplementationRealDataProbe (1) | Source-verified exact trait fixture and service cache cold/warm equivalence, stale-source failure recovery and native trait rows pass. The author's optional external cache/timing is not claimed. |

The ordinary SourceScrollView realAppFoldPerformanceProbe also depends on an
absolute macOS path and has no fixed portable timing threshold. Shared fold/viewport
contracts and the fresh large/long-source native flows verify the portable product
requirements; no AppKit performance equivalence is asserted.

Outstanding required work is successful SDK-backed Android generation/navigation
(or an explicit user deferral), followed by recording its result and rerunning any
affected gates if it exposes a defect. SDK package/license authorization is pending;
silence is not permission. Keep the goal active. No additional missing major desktop
feature was identified by this ledger pass. A future upstream update or newly
observed defect must be reconciled without weakening the original requirements.
