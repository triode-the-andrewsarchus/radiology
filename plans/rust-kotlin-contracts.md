# Kotlin and PR Kotlin assertion reconciliation

All six KotlinIndexingTests and both PRKotlinNavigationTests functions now have
explicit Rust mappings. The unchanged recorded Kotlin compiler fixture remains the
identity/encoding reference; controlled Gradle/aggregate boundaries do not claim
fresh Kotlin compilation. Prior real multi-module JVM compilation and missing-SDK
failure are recorded separately in rust-gradle-android-acceptance.md. Successful
Android SDK-backed generation remains unverified.

## Complete mapping

| Swift function | Current Rust evidence |
| --- | --- |
| realCompilerIndexPreservesOverloadsAndUTF16AfterEmoji | app::tests::kotlin's existing four-case matrix maps every ranged Reader occurrence, resolves integer/string read calls after café/emoji to distinct symbol identities and exact Store line/byte locations, and returns one reference/one definition with zero unmapped or unreadable results and no Noise hit. |
| strategyUsesWholeGradleBuildAndDoesNotInventProjectsFromKotlinFiles | kotlin_build_roots_cover_members_without_inventing_loose_source_projects uses the literal Swift path inventory. One root Kotlin/Java strategy owns all nested modules with exact Gradle evidence; standalone Kotlin sources and an empty build manifest create no strategy. Both source extensions are accepted. The actual wrapper always runs in the existing isolated workspace. |
| cacheAndReuseRetainUTF16ColumnsAndPartialCompilerCoverage | The existing PR matrix checks three-document complete and two-document partial generation, filters compiler-only Noise outside inventory, and retains UTF-16 availability/partial status through tool-free reload and Update. The new explicit_kotlin_checkout_reuse case copies the four exact indexed inputs to an independent checkout, supplies the original cache/root/label through the production library API, and asserts two documents, partialBuild, absent Noise, UTF-16 and the integer overload's exact Store definition. It then deletes the source cache, clears reuse sources and tools, and reloads solely from destination-owned cache records. Aggregation remains at exactly one original invocation throughout. |
| isolatedInputsPreserveBytesModesAndNeverWriteThroughSymlinks | The shared workspace cases assert exact CRLF bytes, copied executable 0755 mode, changed-mode detection, relative/absolute internal-link rebasing, source preservation after copy edits, snapshot blob mismatch rejection, invalid paths and external source-link failure. These are language-independent copy/identity contracts; actual Kotlin wrapper tests exercise the same workspace owner. |
| directoryLinksAreRebasedAndTrackedBuildInputsSurviveBrowserFiltering | internal_links_are_redirected checks relative directory alias, absolute file alias and internal missing links, while external Markdown becomes literal target text. It now writes through the rebased directory alias and proves copied-input verification fails while original source verification/bytes remain intact. Git inventory checks retain tracked hidden .cargo/build inputs beyond browser exclusions; snapshot checks compare symlink blob IDs, and unsafe parent symlinks cannot write into source. |
| workingTreeServiceCopiesTrackedDependenciesAndInvalidatesTheirCache | The new tracked_kotlin_dependency case commits the recorded Kotlin sources and vendor/dependency/input.txt into a real Git checkout. Vendor is absent from the actual app's file inventory. The controlled Gradle process reads the dependency from the private copied workspace and records first then second version after only that tracked file changes. Reload regenerates without a reader-inventory refresh, with three compiler documents, no reuse/failure, three aggregate invocations total including initial review generation, and exact source bytes preserved. Git status contains only the authored dependency edit. |
| changedBlockCaretReferencesAndDefinitionBackRemainInAfter | The existing both-scale complete/partial matrix accepts a real pinned Git review, checks its changed-line overlay, moves the caret by a real source key without changing Back, and resolves both overloads through Ctrl+Alt+R and F12. Stored Before preferences are overridden by accepted semantic destinations in After; both direct definitions and committed reference rows return by Back to the exact departure byte. The attached checkout deliberately has unrelated bytes. |
| beforeClearsSemanticsAndAfterRemapsUsingCachedUTF16Contract | The same app matrix reads read(1) in Before, clears semantic inputs/under-caret references, disables semantic actions and visibly shows Before not indexed while retaining parsed declarations. After restores the integer identity and zero unmapped occurrences; tool-free cache reload and unchanged-snapshot publication preserve caret/history/identity in After. This matches Swift's controlled metadata publication; it does not assert a fresh remote request. |

The existing kotlin_review fixture owns all new Kotlin tool boundaries; its input
list is shared by review setup and tracked-checkout validation. No second generator,
compiler wrapper, fixture protobuf builder or runtime mechanism was introduced.

## Verification and scope

Archive: ~/.local/state/radiology-qa/kotlin-contracts-20261109-b904c6f/.
The three actual-app cases pass; the pre-existing large matrix includes four defined
1×/2× complete-dark/partial-light combinations. All 116 indexing cases pass (twelve
opt-in real-tool ignores), and strict all-target Clippy passes. The combined serial
workspace run passes 939 app tests (twelve opt-in ignores), binary/font/text 2/13/23
and all 68 doctests (60+4+4). No new native-desktop run is claimed;
prior recorded-fixture native proof is linked from rust-kotlin-pr-fixture.md.

Compiler/personality is now 34/35 mapped. Only the exact real-Rust generation
function remains in that category; inspector/traits fifteen, SDK-backed Android and
final integrated/native/standalone/shared-host acceptance remain open. Actual
little-screen-clip adoption and full assistive accessibility stay deferred.

Architecture compares NEUTRAL to before-kotlin-contracts-20261109: +175 measured
NLOC/net three test functions, no new file; avg/max CCN 3.43/132, all 36 existing
high-complexity functions, one cycle and all 125 normalized clone instances
unchanged (zero additions/removals). The lone ranked Python alert is in an untouched
source and is not a new chokepoint. This checkpoint changes tests only.
