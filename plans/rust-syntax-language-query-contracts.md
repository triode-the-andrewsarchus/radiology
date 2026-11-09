# Language syntax and embedded-query contracts

Reference: twelve SyntaxLanguageTests groups and five SyntaxQueryTests groups.
Comparison begins at 1a235ff. These bring source/syntax reconciliation to 37/66;
SyntaxWorker (9), test presentation (7) and test regions (13) remain, followed by
compiler/inspector and final integrated/native/standalone/reuse acceptance.

## Reproduced Kotlin fold defect

The exact sample.kt language assertion failed because it has a single import and
a blank separator, while Rust emitted no Imports fold. The new reference matrix
also failed on blank.kt, expecting bytes 0..17 and receiving no fold. The old
sibling-run implementation omitted separators and grouped across blank boundaries.

Rather than changing expectations to fit Rust, the audit built the exact reference
parser from Package.resolved: fwcd Kotlin e1a2d5ad1f61f5740677183cd4125bb071cd2f30,
and Tree-sitter da6fe9beb4f7f67beb75914ca8e0d48ae48d6406 (0.25.10). The installed
Cargo runtime's .cargo_vcs_info.json matches that exact runtime revision. The probe
extracts real import_list byte ranges; Swift's existing multiline filter is then
applied. This is the pinned parser on this Linux host, not execution of AppKit.

The [pinned grammar](https://raw.githubusercontent.com/fwcd/tree-sitter-kotlin/e1a2d5ad1f61f5740677183cd4125bb071cd2f30/grammar.js)
and [scanner](https://raw.githubusercontent.com/fwcd/tree-sitter-kotlin/e1a2d5ad1f61f5740677183cd4125bb071cd2f30/src/scanner.c)
explain why comments and explicit semicolons affect where an import header ends.
The actual probe corrects the initial inference that comments always terminate a
group. All eighteen inputs/ranges are checked in at
[src/syntax/test_fixtures/kotlin-import-ranges.json](../src/syntax/test_fixtures/kotlin-import-ranges.json).

The existing Kotlin query still captures parsed imports. Extraction now collects
those bounded facts, reconstructs their groups from parsed siblings and at most
two trailing line separators, and sends each range through the existing make_fold.
Blank-delimited runs stay separate. Comments attach according to semicolon/header
ownership, including the reference's CRLF endpoints. Strings and error/declaration
siblings cannot become imports. Same-line imports and single imports without a
blank separator remain non-collapsible. No second grammar, parser or source rewrite
is shipped. Fold precedence insertion is shared rather than duplicated.

The old Unicode/CRLF example now correctly yields two folds rather than one:
first import/comment/blank rows 4..6, then import/trailing-comment rows 9..10. Its
semicolon-terminated middle import has no multiline fold. The previous expectation
that every trailing comment stayed outside the fold was wrong for the pinned
Swift parser. Declaration rows remain outside both groups.

A 20,000-space separator test interrupts grouping at a latched stop pulse and
returns cancellation, never a partial successful group. Parsed imports still
count against the original fact limit before grouping. The shared gutter test now
covers Python bodies and Kotlin import separators at both scales: actual click
collapses the range, Copy All preserves the original source while folded, reveal
restores exact text, and source bytes never change. The existing non-delimited
header rule matches Swift's LineNumberGutter; no new gutter algorithm is added.

## Language assertion mapping

| SyntaxLanguageTests group | Rust evidence |
| --- | --- |
| everySupportedDialectProvidesFunctionBodyFolds | Structural fixture matrix checks Callable folds for all twelve exact fixture paths, including C/C++ and kts. |
| rustHighlightsDeclarationsScopesAndFolds | Exact raw keyword/function/macro/builtin/doc-comment captures; Scan/Modality/Render/helpers/MAX_SCANS kinds; nested render, closure scope, Callable/Type folds. |
| goHighlightsDeclarationsScopesAndFolds | Exact func/scan/string/append roles, with New explicitly excluded from function.builtin. Parsed module/struct/method/function, closure and Imports/Callable folds remain. |
| pythonHighlightsDeclarationsScopesAndFolds | Exact def/async/@dataclass/range/self captures, Scan/load and nested render, Class structure and Type/Callable folds. |
| javaScriptHighlightsDeclarationsScopesAndFolds | Exact class/render/#secret/console/MAX_SCANS captures, Scan/render/load/double declarations, closure and Type fold. |
| typeScriptHighlightsDeclarationsScopesAndFolds | Exact interface/readonly/string captures, Renderer/Modality/Level/Low/Helpers/Scan kinds and Type/Callable folds. |
| kotlinHighlightsDeclarationsScopesAndFolds | Native keyword/namespace/type/constant roles retain exact fun/radiology.scan/String/XRAY/return tokens; Scan/Modality/Titles/constructor/XRAY structure, no local parts property, closure and repaired import fold. kotlin-ng normalizes keyword/type subcategories to the same displayed color families. |
| kotlinScriptsUseTheSameGrammar | Registry/dialect evidence selects Kotlin for kts; exact script has collect, Report and scanRoot with a clean native parse and Callable fold. |
| jsxUsesTheJavaScriptGrammarAndTagsElements | Existing dialect matrix selects JavaScript; exact ul/Badge/className captures and ScanList declaration pass. |
| tsxUsesTheTsxGrammarAndKeepsTypeScriptStructure | TSX selection, exact ul/data-count/number captures and Props interface pass. |
| jsxAndTsxNeedTheirOwnGrammars | Existing ECMAScript matrix checks JSX rejection by TypeScript and inverse angle-bracket assertion rejection by TSX, with both line endings. |
| javaScriptGrammarAlsoParsesJsxInsidePlainJsFiles | The exact plain-JS export-const source parses cleanly and captures main/className as JSX roles. |

Rust retains type/protocol structure in the typed outline forest rather than
putting non-callable types into CallSites.scopes. The latter deliberately stores
only callable ownership. This is a representation difference, not discarded
reader/breadcrumb structure. Shared declarations helpers check canonical ranges
and clean parses; no duplicate syntax model was introduced.

## Query packaging assertion mapping

| SyntaxQueryTests group | Rust evidence |
| --- | --- |
| everyRegisteredLanguageCompilesItsQueries | registered_grammars_embed_all_roles checks all ten registered grammars across Calls/Folds/Occurrences/Highlights. Folds includes actual symbol and fold captures; Calls includes callable scopes. Role caches remain independently compiled and thread-safe. |
| overlayLanguagesConcatenateEveryQueryDirectory | Exact combined TSX source contains template_string, predefined_type and jsx_opening_element markers from all three query families. |
| queryResourcesAreReachableThroughTheModuleBundle | Rust embeds sources with include_str instead of loading a module resource directory. All real roles compile, and the copied executable's headless resource check succeeds from an empty working directory. |
| missingQueryResourcesReportADiagnosableError | A private complete Git export plus current Rust overlays removes only swift/highlights.scm. Locked offline cargo check fails with that exact path and os error 2. The original resource hash is verified unchanged. Rust's closed grammar enum has no dynamic Unpackaged language constructor; a missing shipped query fails at build time. |
| resourceVerificationReportsEveryGrammar | Binary resource tests assert the exercised grammar set equals all ten registered grammars. Copied executable verifies eleven native path probes (JS/JSX share one grammar) and five fallback formats, with exit zero and no FAIL. The existing negative native probe proves fallback coloring cannot masquerade as complete structure. |

## Verification and structure

Archive: ~/.local/state/radiology-qa/syntax-language-20261109-1a235ff/.
All 76 syntax checks, two binary resource checks, the both-scale gutter matrix,
strict all-target Clippy, normal build and warning-free docs pass. Query cache,
parser/query cancellation and long-separator cancellation are covered. The fresh complete serial workspace run passes 901 app tests, eleven existing
opt-in ignores, binary/font/text checks 2/13/22 and all 66 doctests. Full compilation
uses -j 1 and tests use --test-threads=1 for the previously documented parallel
limitations; normal Cargo builds retain the requested half-core 16-job setting.

The archive keeps the C probe harness, pinned source/hash/runtime manifests, input
files and reference ranges, plus packaging results. Its 23 MB generated parser,
reference executable, copied app and private exported checkout were removed after
verification; only small text evidence is retained. No new /tmp probe tree remains.

Architecture +413 NLOC/eight net functions: five test functions and three net
runtime functions after replacing the old group walker and sharing insertion.
One small JSON reference fixture is added; no new source-code file, dependency,
query controller, public API or UI field. Average CCN changes 3.42 to 3.43; maximum
stays 132, high-complexity sites stay 36 and cycles are unchanged. Necessary group
boundary handling replaces the old incorrect whole-run shortcut. All 125 clone
instances are identical by complete normalized multiset comparison. Raw flags
refer to an old viewport fragment and equal-ranked untouched Python chokepoints.

No Swift source or typography changes. Full accessibility and actual sibling
integration remain deferred. Android SDK success, prior parallel-run limitations
and the remaining whole-app/native/standalone/reuse gates stay open.
