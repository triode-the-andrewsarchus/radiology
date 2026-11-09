# Inspector and trait assertion reconciliation

All fifteen functions across ContextInspectorLayoutTests, InspectorPaneTests,
TraitImplementationContextTests and TraitImplementationTests now have explicit
mappings. The existing implementations pass the contracts; this checkpoint adds
missing interaction/integration evidence and shares test input setup. It changes
no production source, preferences format, public API, typography or UI controls.
Overall parity remains unproven until the final integrated/native/standalone/reuse
acceptance and complete requirement-ledger review; Android SDK-backed success is
still separate and unverified.

## Inspector layout and preferences (five functions)

| Swift function | Rust evidence |
| --- | --- |
| nativeDividerResizesBothPanesWithoutChangingContext | New focus::panes::inspector_divider_drags runs actual app pointer input at 1×/2× with ordinary references and trait context. It locates the real framework resize controls, drags both References/middle and Bookmarks/middle boundaries by sixty points away from size limits, and checks more than forty points of actual region-height change. Source Arc, caret, Back history, reference target/selection, trait target/selection and bookmark selection remain exact. Each case starts with its own render context; scale-restored panel sizes are allowed to begin at a minimum/maximum. Rust uses egui regions rather than AppKit split views. |
| bookmarksRemainIndependentOfCodePathsSidebar | New code_paths_sidebar_keeps_bookmark_preference_and_saved_reader_state_independent imports a real saved trace and bookmark, hides both sidebars, and issues Show Code Paths twice for both enabled/disabled bookmark choices. Only the left sidebar reopens; bookmark choice persists on disk. Re-enabling Bookmarks reveals the inspector while retaining the Code Paths sidebar, trace identities, bookmark data, source identity and history. The retired Swift UserDefaults key does not belong to Rust's session format; no fourth visible region exists in the typed pane list. |
| preferencesPreserveExistingDefaultsAndPersistEveryChoiceIncludingNone | Session inspector tests cover absent old-session fields, partial serde defaults, all sixteen preference combinations, both trait-availability states, serialization and actual SessionStore reload. The real-app matrix now additionally reloads session.json after every mask, including none. Show All persists; reset/root changes retain host-owned choices. |
| everyPaneCombinationReflowsNativelyAndPreservesContext | The both-scale real-app sixteen-mask matrix checks actual References, Definition/Outline and Bookmarks headings, empty/unavailable recovery, full-height single Bookmarks, stable selected reference/caller expansion/bookmark/source/caret/history. Framework regions are composed only for active slots; one region has no divider. The new pointer test additionally exercises actual resizing after composition. Rust's compact References minimum is 160 points and Bookmarks minimum 100, preserving the earlier accepted region cleanup. |
| traitControlsSelectImplementationDefinitionFallbackOrNeither | Existing focus::traits::trait_pane_preferences covers all sixteen masks: trait rows replace the Definition slot only while enabled/available; otherwise Definition/Outline appears if selected. A trait-only layout remains valid, an unavailable trait-only layout offers recovery, and hidden row keyboard ownership retires without losing selection. The typed visibility test covers no-trait fallback for every mask; actual session persistence is asserted above. |

## Trait catalog (five functions)

| Swift function | Rust evidence |
| --- | --- |
| realRustIndexResolvesAliasesQualifiedBlanketAndNegativeImpls | semantic::trait_impls::real_rust_trait_catalog uses the exact recorded Rust fixture and checks Alpha/Beta/Wrapper order, Alias signature, where bound, right::Render/Other identity, negative Marker, Empty absence, no unresolved/gaps, two parsed documents and no SCIP relationship data. Duplicated contributions retain three explicit left blocks. |
| persistentCacheInvalidatesForSourceAndSemanticIdentity | trait_cache_revalidates asserts cold two/warm zero parses, two persisted cached documents, one changed-source parse, semantic remap to right::Render yielding Alpha/Other, role/encoding changes and corrupt-cache recovery. Failed/unsafe derived-cache destinations do not fabricate source writes. |
| excludesUntrustedCoordinatesAndReportsCoverageGaps | trait_catalog_reports checks excluded implementations, one unindexed source, unknown inventory coverage, two explicitly mismatched source digests, three unresolved joins and unreadable paths; cancellation remains explicit. Coverage detail distinguishes unverified imported semantics from known generated provenance. |
| syntaxExcludesBoundsArgumentsCommentsAndMacroTokens | rust_impl_structure uses the literal Swift syntax corpus: exactly Wrapper<T>, Negative and Concrete; trait terminal names A/B/A, negative flag and unsafe signature. Both full DocumentSyntax and the shared implementation-facts path agree, with cancellation covered. |
| navigationRejectsChangedSourceEvenAtSameLength | The real catalog test resolves an exact row then replaces Alpha with equal-length Gamma and rejects it. Actual-app trait_activation_rejects additionally keeps reader/history unchanged on failure and retires pending activation when row/index ownership changes. Restoring exact bytes permits activation. |

## Trait context and service (five functions)

| Swift function | Rust evidence |
| --- | --- |
| followsTraitMembersAndImplBodiesWithStableSelection | semantic::trait_context::compiler_fixture_context checks trait declarations, members/default bodies, Alias and qualified/blanket/negative impl bodies plus the literal Render bound reference. Ordinary struct/function/inherent declarations yield no trait. Actual-app trait_context_retains preserves same-trait selection, clears index replacements, shows empty/negative contexts and clears Before. |
| servicePrecomputesPaneNavigationPreservesCaretAndCacheReloads | generation_and_cache_publish_verified_trait_catalogs checks production generated enrichment, prepared/verified provenance, five rows, persisted reload with the tool removed, derived cache reconstruction and a subsequent warm two-document derived-cache hit. Actual app primary-click/Arrow/Return/Back checks preserve exact source departures at both scales. The compact list paints only implementation rows, not file-heading rows, and virtualizes large lists. |
| nativeTraitPaneHasIndependentScrollingAndResizableDivider | The new divider matrix includes the real trait reader at both scales and preserves selected trait row. The existing compact-trait list test independently checks 220/280/420 widths, dark/light at both scales, virtualized 1,000 rows, selection reveal, bounded text and compact <=24-point text. Existing bookmark scroll survives split/single layouts. Independent scroll/focus IDs stay owned by the existing list instances; no shared source scroll state is introduced. |
| beforeAndIndexReplacementClearTraitSelections | Actual app trait_context_retains and trait_activation_reads_pinned_after exercise index replacement selection retirement and Before clearing; model/context tests reject Before, stale bytes, wrong language, ambiguous/unresolved/local impl joins and cancellation. Root/index ownership updates reset results through the existing context owner. |
| serviceDoesNotEnrichStaleCachedCoordinatesWhenRegenerationFails | New index_lifecycle::failed_automatic_trait_refresh starts real automatic generation from the exact trait corpus, confirms prepared verified five-row enrichment, changes Alpha to equal-length Gamma and forces an actual exit-9 refresh failure. Published stale catalog contains no trait rows, reports the changed implementation path as excluded, retains the current lib reader and reports gaps. Restored bytes recover all five rows. Rust uses per-document digests, so unchanged trusted lib coordinates remain eligible; Swift's coarse stale state excludes both documents. No changed-coordinate implementation can be activated. |

## Verification

Archive: ~/.local/state/radiology-qa/inspector-contracts-20261109-0649a1b/.
All six pane app cases, twenty-two trait-filtered cases, the bookmark drag/cancel
regression, session preference case and strict all-target Clippy pass. New shared
begin_drag sends held press/movement through actual app frames; existing bookmark
cancel and inspector resize own their distinct release/cancel steps. This removes
the initial duplicated six-line setup rather than duplicating a pointer driver.
The full serial workspace gate passes 942 app tests, twelve opt-in ignores,
binary/font/text 2/13/23 and all 68 doctests (60+4+4). These are real-widget frame checks, not a new
native-desktop acceptance claim. Earlier scoped native evidence remains linked in
rust-inspector-region-sizing.md, rust-inspector-panes.md and rust-trait-inspector.md.

Architecture compares NEUTRAL: +177 measured NLOC/net three analyzed functions,
no new file, average/max CCN 3.43/132, all 36 high-complexity functions/one cycle and
all 125 normalized clone instances unchanged (zero additions/removals). No new
production dependency, state owner, worker, cache writer or reader abstraction.
The requested SwiftUI/macOS/typography skills remain unavailable locally; source
changes here are Rust tests only and use established fixtures/controls/type styles.
