# Call-site extraction assertion reconciliation

Reference: all four CallSiteExtractionTests groups. Comparison begins at 5769978.
These are test-only additions to existing call queries and Critical Lens summaries.
With buffer/outline and registry/C mapping, 20 of 66 source/syntax groups are now
reconciled; the remaining five suites contain 46 groups. Compiler, inspector and
final integrated/native/standalone/reuse acceptance remain open.

## Assertion mapping

| Swift group | Rust evidence |
| --- | --- |
| everyFixtureReportsNamedCallSites | swift_corpus_call_sites uses exactly SyntaxFixtures.names: all twelve checked-in fixtures. Every query is supported without error and supplies named calls. Names are nonempty original slices contained by their full expression; anchors are sorted and full expression ranges are unique. The existing corpus parse/highlight/reader checks remain separate evidence for each language and line ending. |
| rustShapesMacrosAndOwnership | swift_rust_fixture uses the exact sample.rs. format/println are macros; to_string/render/default are calls; closure inside println's token tree is absent and every site is named. Render's body directly owns format/println; main owns to_string/default/render/println. Both render scopes exist, and the trait requirement has no body. Existing summary returns Abstract for it; main has one conditional closure with zero parsed sites. |
| dynamicCalleesClosuresAndNestedFunctionsStayHonest | swift_dynamic_callees uses the exact outer(f,g)/adder/inner/helper source. Both parenthesized f/g sites are dynamic and unnamed; outer's ordered direct names are two dynamic entries, adder and inner. The existing graph summary reports closure/function conditionals with one site each and the exact inner name. Only two real helper sites exist, excluding the comment and string. |
| otherGrammarsNameMethodAndFunctionCalls | The corpus matrix checks the exact Go Sprintf/append, Swift append/joined, Python open/read and Kotlin max/mutableListOf names. TypeScript and every other fixture also provide nonempty named call sites. |

These assertions inspect the same CallSites.scopes/sites consumed by source
projection, incoming callers and graph summaries. The existing graph test helpers
supply synthetic semantic metadata when invoking summarize; this proves structural
conditional ownership, not real compiler-generated edges or cross-file navigation.
No second direct/conditional-call algorithm or extra runtime API was introduced.
The queries remain compiled from the shared Swift resources.

## Verification and architecture

Archive: ~/.local/state/radiology-qa/callsite-contracts-20261109-5769978/.
All five call-query tests and five graph-summary tests pass, along with strict
all-target Clippy and formatting. Production code is unchanged. The last complete
workspace run at 2e089e9 passed 891 app tests and all 66 doctests; later buffer/
outline and these call-site assertions have focused evidence rather than a new
full-run claim. See [buffer/outline verification](rust-buffer-outline-contracts.md#verification-and-structure).

Architecture +204 NLOC/three focused test functions, no new files or runtime
helpers. Average/max CCN stays 3.42/132, with no added hotspot or cycle. Clone
instances remain 125 with zero normalized multiset additions/removals. Reuse is
through existing corpus constants, CallSites, indexed/summary test helpers and
the production summarize path. The architecture comparison is neutral.

Next source suites: SyntaxLanguageTests (12), SyntaxQueryTests (5), SyntaxWorkerTests
(9), TestCodePresentationTests (7) and TestRegionDetectorTests (13). Full accessibility
and sibling integration remain deferred; Android SDK success and prior parallel
run limitations remain open.
