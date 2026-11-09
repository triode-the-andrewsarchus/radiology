# C/C++ compiler and fallback assertion reconciliation

Reference: all five CScipTests and three ClangFallbackTests groups. All eight have
explicit Rust evidence, bringing compiler/personality reconciliation to 17/35.
The remaining eighteen are IndexingTests (10), KotlinIndexingTests (6) and
PRKotlinNavigationTests (2); inspector/trait and final whole-app acceptance remain.
The next lifecycle audit must address Swift's automatic open/stale-cache refresh:
Rust currently restores/imports caches on open and leaves generation explicit.
This is an observed remaining difference, not an accepted scope exclusion.

## Assertion mapping

| Swift group | Rust evidence |
| --- | --- |
| realCompilerIndexResolvesCAndCppOverloadsAfterUnicode | recorded_clang_overloads decodes the exact checked-in four-document scip-clang fixture, verifies producer and UTF-8 columns, maps every C++ occurrence, preserves distinct render overload identities and exact definition lines 1/4 (zero-based), resolves C add to math.c and counts the two non-definition references. The unsupported extern-C add(value) caret remains empty, never guessed by name. |
| compilationDatabaseDiscoveryIncludesHiddenBuildOutputsAndPinsSnapshots | clang_generation_uses now checks both exact build/compile_commands.json and native/cmake-build-debug/compile_commands.json locations with a native CMake marker, in tracked/untracked/Git-free contexts; shallow root evidence wins. Existing ignored-database and bounded CMake inventory tests cover browser exclusions. clang_fallback_review uses a real pinned Git tree and verifies an ignored working-checkout database cannot contaminate it. Rust's fallback strategy/visible explanation supplies guidance when a real database is absent rather than copying Swift's no-index build hint. |
| clangErrorsRemainPartialEvenWhenTheProcessSucceeds | The service diagnostic matrix now includes both exact finished-TU strings and the warning-only string. Error diagnostics/positive counts with usable SCIP remain Partial even at full document coverage; zero counts/warnings do not fabricate failures. |
| realClangServiceDiscoversBuildDatabaseCachesCoverageAndLeavesSourcesUntouched | New opt-in real_clang_swift_fixture runs the exact source fixture plus unused.cpp through configured→defaults→configured generation with the pinned real tool. Configured output has four documents and the exact unused.cpp missing path; defaults has five. Every phase stays Partial for its actual reason, restores the same coverage from its own normalized cache, preserves UTF-8/C definition identity and all authored bytes/database bytes, and leaves no source-side object/index/supplementary outputs. |
| compilerOutputSupportsReaderDefinitionReferencesAndExactBack | recorded_clang_reader imports the exact compiler artifact into the actual app, resolves add at its original byte, reports two references plus one separately counted definition, goes to math.c and Back to the exact departure byte at 1×/2×. This is recorded compiler data through real app actions, not a claim of a freshly compiled app pipeline; fresh compiler/service behavior is proved separately above. |
| switchingRepositoriesCancelsLargeDefaultPreparationPromptly | New service case starts the exact 1,500-function inventory, observes the live best-effort preparation phase, switches to a README-only repository and asserts completion under two seconds. The retired worker leaves no input-copy directory, compiler invocation or JSON cache and cannot publish a late result. |
| defaultInputsInferAvailableHeadersAndPreferRealBuilds | swift_default_fixture uses the exact three units and layout header, checks sorted case.C/main.c/main.cpp entries, GNU++20/GNU11, inferred support/include, real database precedence, header-only fallback and literal ./-fplugin=helper.cpp. Existing seven generator cases retain larger bounds, cycles and unusual-name coverage. |
| noDatabaseStillProducesNavigableCompilerIdentityAndCachedBestEffort | Re-executed real_scip_clang_fallback uses the exact increment/layout/missing_header pattern with additional Unicode/CRLF, validates cross-file canonical definition/reference coordinates, compiler diagnostics and Partial generation, project reuse, warm tool-free cache, unchanged source and private database cleanup. Existing service tests ensure even full fallback coverage stays Partial. |

## UI and architecture change

The existing generation-progress owner now reports “Preparing best-effort C/C++
inputs…” before fallback inference, or “Preparing compilation database…” before
configured rebasing. Previously that work retained the preceding tool-version
message. The status now explains the work and exposes the real cancellation phase.
It uses the same progress mutex, worker, controls and cancellation path. No new
state, UI font/style, cache format, dependency, helper or publication path.

Architecture +334 NLOC/five test functions, average/max CCN 3.43/132 and unchanged
36 high-complexity functions/one cycle. All 125 normalized clone instances remain
identical, zero additions/removals. The raw touched-file clone and equal-ranked
untouched Python chokepoint alerts are pre-existing. Necessary status selection
adds two branches to the existing execution owner; no additional mechanism.

## Verification

Archive: ~/.local/state/radiology-qa/clang-contracts-20261109-8d1b04f/.
The retained scip-clang 0.4.0 executable matches the official pinned SHA-256
06fd18c576f979a726c651594644ec4a35db4f471f2160b3f72eb89fa6001784;
no download or install was required. The new real configured case and both existing
real fallback/response-file lifecycle cases pass. All 110 regular indexing tests
pass, with twelve opt-in ignores (including the new real case); both recorded
semantic/app tests, exact defaults and live preparation cancellation pass too.
Strict all-target Clippy, normal build and warning-free docs pass. The complete
serial workspace gate passes 919 app tests (twelve opt-in ignores), binary/font/
text checks 2/13/23 and all 67 doctests (59+4+4). It includes preparation status
and every recent explicit-reuse/source/personality change. No native screenshot or full external-corpus rerun is claimed
by these source/app-frame/service checks. Final integrated acceptance remains open.
