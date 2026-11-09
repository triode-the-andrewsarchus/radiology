# Source language registry and C/C++ syntax contracts

Reference: all five SourceLanguageRegistryTests groups and all four CSyntaxTests
groups. Comparison begins at 51aaa44. These nine groups are explicitly reconciled
out of the source/syntax section's 66; the other 57 still require assertion mapping,
although broad corpus/runtime evidence already exists. Compiler, inspector and
final integrated/native/standalone acceptance remain open.

## Registry and source filtering correction

The parser already recognized C++ template implementation files, but the source
filter omitted tpp/tcc. The red SourceFileClassifier test fails on templates.tpp.
Both extensions now belong to the existing source set, matching Swift. The actual
app filter/refresh scenario includes reader.tpp and reader.tcc at both scales:
the files survive Source Files Only and Hide Tests, path queries still filter
correctly, refresh retains reader/history/preferences and source inventory changes
remain exact. This adds no classifier, extension utility or parsing path.

The registry's red display check fails on View.tsx = TSX. Shared metadata now uses
TypeScript JSX and Plain text, matching Swift's user-facing labels. The existing
ECMAScript grammar matrix and uncovered-index-language expectation use that same
metadata. The actual mixed-language app corpus also checks the painted TSX footer
at both scales; its existing theme, wrap, copy and history checks remain. No font,
weight, dimension or layout algorithm changes are required.

| Swift registry group | Rust evidence |
| --- | --- |
| extensionsResolveToTheRightGrammar | test_language_detection checks all 30 exact Swift paths/aliases, including uppercase .C, plain .h, Gradle kts and C++ template/header suffixes, with a native grammar for every case. |
| pathExtensionMatchingIsCaseInsensitiveAndIgnoresDirectories | The same matrix includes deep/Nested.Dir/Main.RS and Scan.SWIFT. Uppercase .C remains the deliberate C++ exception before case folding. |
| unregisteredFilesHaveNoGrammar | All six Swift paths, including Makefile, JSON and shell, have no native structural grammar. Rust retains its additional non-structural display metadata. |
| everyLanguageHasAUniqueIdentifierAndDisjointExtensions | parsed_languages returns exactly ten unique IDs and ten unique grammars. Rust's single static extension match replaces Swift's mutable extension collections; all aliases are checked, and strict compiler/Clippy checks reject unreachable duplicate patterns. |
| fileClassificationNamesParsedAndPlainLanguages | The six exact Swift display examples match; main.go is parsed and README.md is not. App footer and index-gap checks use the shared metadata. |

## C and C++ assertion mapping

The existing shared Swift query resources and recursive declarator handling pass
these assertions without new parser implementation. New checks use the existing
checked-in Swift corpus and declarations helper rather than another fixture/parser.

| Swift C group | Rust evidence |
| --- | --- |
| cProvidesStructureAndRecursiveDeclaratorNames | c_and_cpp_swift_fixtures checks clean sample.c, Scan/Modality/ScanID/TWICE/titles/render kinds, exact titles selection and its callable body fold. Parsed measure/callback sites remain, fake_call in a comment is absent. The native color-role matrix checks int and measure in the exact fixture, in both themes; the existing corpus checks both LF/CRLF. |
| cppRetainsMethodsTemplatesQualifiedCallsAndLambdaOwnership | The same test checks imaging, Scan, render, ~Scan, operator+=, ScanID and titles in sample.cpp. Render owns its own fold; one nested closure owns one site, and twice is excluded from render's direct calls. Load directly owns render and twice. The exact fixture's template keyword is checked through native colors. |
| plainHeadersParseBothCAndCppWithoutFunctionPointerOutlines | The exact reader.h source parses cleanly as C++, retains Callback and read, and does not outline handler as a function. All selection ranges are checked by the existing declarations helper. |
| newAndDeleteRetainUnknownConstructionAndDestructionSites | The exact allocation.cpp source has a clean parse and exactly two dynamic sites, both with absent callee names. No semantic construction/destruction edge is fabricated. |

Rust's typed Struct/Class distinctions refine Swift's type capture. Native color
verification tests the displayed semantic role rather than exposing a new raw
capture API. Callable ownership is checked through the shared CallSites graph;
no separate implementation of direct/conditional-call extraction is introduced.

## Verification and architecture

Archive: ~/.local/state/radiology-qa/source-contracts-20261109-51aaa44/.
Both label and source-filter regressions fail before repair. Focused registry,
classifier, native-outline, calls and highlights checks pass (2/2/13/4/7), as do
the index-gap check and both app filter/corpus checks. Strict all-target Clippy,
normal build and warning-free docs pass. A fresh complete serial workspace gate
follows this checkpoint and must also verify the preceding caller-keyboard repair;
no full success is claimed here before that run finishes.

Architecture: +248 NLOC, three focused test functions, no new source file or
runtime helper. Average/max CCN stays 3.42/132, high-complexity sites stay 36 and
cycles are unchanged. Clone count stays 125 with zero normalized multiset additions
or removals. A draft capture assertion increased an already-large corpus test's
complexity; the final change instead extends the existing dedicated color-role
matrix with the same fixtures. Raw equal-ranked chokepoint differences concern
untouched Swift Python functions. Source behavior stays in the existing owners.

Next: SourceBuffer, outline, call-site extraction and the other syntax/test-region
contracts, followed by compiler/inspector reconciliation and final acceptance.
Full accessibility and actual sibling integration remain deferred. Android SDK
success and documented parallel-run limitations remain open.

The combined production gate now passes at 2e089e9; the [buffer/outline checkpoint](rust-buffer-outline-contracts.md#verification-and-structure) records the complete results and subsequent test-only additions.
