# Test-region detector contracts

All thirteen Swift TestRegionDetectorTests groups are reconciled against the
existing Rust detector. Fifteen exact source variants are checked in
src/syntax/test_fixtures/test-region-contracts.json; a separate test generates the
4,000-level source. Multiline literals preserve Swift's stripped indentation and
lack of a trailing newline. No runtime, dependency or API change was needed.

| Swift group | Rust evidence |
| --- | --- |
| pytestRequiresUnderscoreAndVitestElseRemainsProduction | Only test_connection is hidden at 5; a Vitest guard with a production else remains visible. |
| rustTestHelperInTheMiddleOfAnImplEndsExactlyBeforeProductionCode | Exact 2–7 range and indentation, closing brace included, following LF excluded. Rust detection stores regions rather than a separate Swift evidence enum. |
| rustCfgLogicHidesOnlyCodeAbsentFromEveryProductionBuild | Exact 1–4 and 18–19 ranges; production-reachable, disabled, cfg_attr and similarly named predicates stay visible. |
| rustTestAttributesModulesDocCommentsAndInnerAttributes | Exact 3–10 and 13–22 spans include attached documentation, async/rstest attributes and inner cfg, without the intervening production declaration. |
| rustFileWithInnerCfgTestIsATestOnlyDocument | Exact inner-cfg file reports test-only with no inline regions. |
| rustRegionsUseCanonicalBytesAfterUnicodeAndCRLF | Exact 3–6 span and byte slice after astral/accented characters; excludes the following CR. |
| kotlinTestClassesFunctionsAndSpecs | Exact 10–23 and 26–28 spans retain production methods while recognizing annotated classes, StringSpec and qualified KDoc-owned tests. |
| pythonUnittestClassesAndPytestOnlyWhenImported | Exact two sources yield 9–11 and 3–5/10–18; pytest imports, decorators, async tests and whitespace merging are exercised. |
| typescriptFrameworkCallsAndInSourceBlocks | Exact bun:test/describe.each/Vitest/Deno source yields 6–17. |
| typescriptProductionCallsSharingTestSpellingStayVisible | Exact relative-import/local-binding/callback lookalikes yield no regions. |
| tsxAndCommonJSRequireFromTestModules | TSX yields 3–5; node:test require yields only line 3 and excludes the relative production require. |
| deeplyNestedGeneratedCodeIsWalkedWithoutRecursion | Exact generated depth 4,000 yields 2–3 and the exact cfg/module bytes. |
| languagesWithoutDetectionReportNothing | Exact Go test-spelled source has no regions or document-level gating. |

The table-driven test also checks canonical scalar-safe slices and header/line
agreement for every detected region. Existing native-grammar parse validation is
reused. Both new tests live beside the existing language-specific detector tests;
no alternate parser, detector or generic fixture framework is introduced.

Verification archive: ~/.local/state/radiology-qa/test-region-20261109-47b6c30/.
All seven detector tests, all 82 syntax tests, strict workspace/all-target Clippy,
formatting and diff checks pass. The latest complete production gate remains
c20eba6 (901 app tests and all 66 doctests); this checkpoint is test-only.
Architecture adds 62 NLOC/two tests, no runtime helper; average/max CCN stays
3.43/132, 36 hotspots and one cycle unchanged. All 125 normalized clone instances
are identical, zero added/removed. Raw comparator warnings concern an existing
outline/detector clone and equal-ranked untouched Python chokepoints.

Source/syntax mapping is now 59/66. Seven test-presentation groups remain, followed
by compiler/inspector reconciliation and final integrated/native/standalone/reuse
acceptance. Accessibility and actual sibling integration remain deferred.
