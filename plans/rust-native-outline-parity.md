# Swift and Kotlin declaration outlines

## Contract and implementation — 2026-11-01

The standalone Rust reader now consumes the bundled Swift application's symbol
queries for Swift and Kotlin outlines through the existing cached folding query.
The existing Kotlin query adapter handles the linked kotlin-ng grammar. This
replaces the smaller language-specific AST match arms. At this increment other
languages retained their existing paths; the later expansion below replaces them
except C. No dependency, host integration, worker or UI state was added.

Swift protocol requirements, initializers and deinitializers have exact captured
names, alongside existing types, functions, extensions and aliases. In particular,
a deinitializer is named `deinit`, rather than accidentally taking a name from its
body. Kotlin includes file/type properties, enum cases, secondary constructors,
initializer blocks and named companions, alongside types, aliases and functions.
Local variables are excluded by the original query scope; nested named functions
remain included. This matches those query rules, not every possible language
construct or a completed grammar acceptance claim.

Full declaration ranges form a source-ordered containment tree. Selection ranges
retain the captured original bytes, including Unicode and escaped identifiers;
headers retain complete signatures. Presentation uses existing outline roles,
glyphs, navigation and source coordinates. Public `SymbolKind` gains `Initializer`,
`Deinitializer` and `Property`; shared `is_callable` includes lifecycle routines so
existing function-first outline presentation and scoped reader behavior apply.
Future embedding hosts can reuse these facts without depending on the app shell.

Query execution uses the existing bounded native matcher and cancellation checks.
The outline has a 65,536-declaration limit. Failed queries publish no partial
outline; `DocumentSyntax::parse_error` reports the failure while preserving other
completed analysis facts. Cancellation rejects publication. The convenience
`SymbolExtractor::extract` returns an empty outline on failure; callers needing
diagnostics use `DocumentSyntax::analyze`.

## Verification

Three regression tests cover exact Swift/Kotlin names, byte selection ranges,
hierarchy, lifecycle kinds, headers, local-property exclusion, escaped names and
cancellation followed by complete fresh extraction of 3,000 declarations. The
initial Swift negative exposed missing protocol requirements and a deinitializer
incorrectly named `clean`. The first Kotlin fixture failed parser setup; that
trial is excluded as evidence of outline behavior.

The workspace passes 564 app tests (seven ignored), two binary tests, four font
tests, 13 text tests and 37 doctests. Strict Clippy, all-target check, binary build,
rustdoc with warnings denied, formatting and diff checks pass.

Architecture baseline: `before-native-declarations` at `a103340`. Product and tests
add 249 NLOC, ten functions and two files. The old AST `parse_node` drops from
CCN 6 to 4; maximum CCN remains 100 and dependency cycles remain zero. Health rises
about 0.15. The automated comparison labels an existing nine-line outline/test
scanner clone as a regression because its file changed. A complete normalized
multiset comparison of file pairs, format, line/token counts and whitespace-normalized
fragments confirms all 91 clone groups unchanged: zero added and zero removed.

Four final native flows pass: Swift and Kotlin at both 1× and 2×. Swift clicks
navigate to exact `deinit` and alias name bytes; Kotlin checks constructors and a
Unicode property. Each verifies selected-name and whole-file clipboard hashes,
unchanged fixture/binary hashes and zero child exits. Dark/light captures were
inspected. Kotlin 2× also uses outline Home/End to reveal offscreen rows. These
small fixtures prove these declarations, not broad language-corpus acceptance.

Early native trials are excluded: reading structure was still changing panel
geometry, the pointer helper rejected fractional coordinates, whole-window OCR
missed labels, hovered tooltips covered names, and offscreen rows needed reveal.
Final checks wait for declarations, find the actual divider, use integer pointer
coordinates, crop OCR, allow focus frames, clear tooltips and reveal rows. No
product code changed during native acceptance.

Archive: `~/.local/state/radiology-qa/native-declarations-20261101-a103340`.
`accepted.json` selects `final5-1.0-swift`, `final3-2.0-swift`,
`final3-1.0-kotlin`, and `final6-2.0-kotlin`; each contains action logs, hashes,
captures and exact clipboard files. Workspace/gate logs, parser probe, controllers,
fixtures, architecture reports, full clone audit and source provenance are archived.
Requested SwiftUI/macOS/typography skills are absent from the installed skill
roots; existing presentation roles and typography were reused without a new scale.

## Remaining grammar acceptance

The specific compact-member defect below is repaired by the
[2026-11-02 scanner patch](rust-kotlin-scanner.md), with complete raw trees and
pinned upstream corpus coverage. The following records the earlier finding.

The linked kotlin-ng 1.1.0 grammar reports a missing `_class_member_semi` for valid
compact `interface Reader { fun read(value: Int): Int }`. The newline form and
explicit `;` form parse without errors; compact enum declarations also parse.
This is a confirmed grammar acceptance gap, not repaired by rewriting source or
inserting bytes. Recoverable error trees follow the existing analysis policy and
do not automatically populate `parse_error`. Broader Swift/Kotlin corpus coverage
and final side-by-side Swift acceptance remain open. Full accessibility and actual
little-screen-clip adoption remain deferred per user scope.

## Shared declaration queries for the remaining native languages — 2026-11-01

Rust, Python, Go, JavaScript, TypeScript and TSX now use the same native outline
extractor as Swift/Kotlin. The old hand-selected match arms are removed; C retains
its existing AST path. This closes concrete omissions found against bundled Swift
queries: Rust trait requirements, unions and macros; Python module assignments and
type aliases; Go package names, interface methods, aliases and file-scope values;
JavaScript generators and named function/arrow initializers; TypeScript abstract
types, signatures, properties, enums and namespaces. Local non-callable values stay
excluded according to the query rules. Public `SymbolKind` gains `Union` and `Macro`.
Rust statics follow the bundled query's Constant role; Go struct/interface bodies
retain useful specific kinds. Generic impl target names retain their exact bytes.

The shared query cache combines TypeScript additions with ECMAScript symbols and
folds for both TS and TSX. TypeScript interface/object/enum/namespace bodies gain
structural fold facts. The Rust adapter narrows a bare enum case capture to its
identifier: the bundled query captures the entire enum body, which otherwise
incorrectly parents assigned cases under the first bare case. Original Swift
resources are retained unchanged. This correction is covered by sibling/hierarchy
and exact full-range assertions, beyond just checking flattened names.

Duplicate declarations use the Swift worker's most-specific-kind ranking, so a
named arrow function matched by both function and generic variable queries remains
one callable row. Source order, captured selection bytes, cancellation, bounded
matching, existing inspector and standalone host boundaries remain shared.
Signatures for named function/arrow values stop before their actual body, using
the existing header helper; they do not expose a whole initializer body as a
signature. No dependency, worker, parallel parser or persistent UI state is added.

Five language regression cases fail against the old implementation and establish
the omissions above. Exact names/ranges, hierarchy, kind preference, file/local
scope, generic impl names and closure headers are checked. TypeScript and TSX
also verify four multiline structural type folds. Existing outline assertions
now expect TypeScript properties and the bundled Rust static classification.

The final workspace passes 569 app tests (seven ignored), two binary tests, four
font tests, 13 text tests and 37 doctests. Strict Clippy, all-target check, binary
build, rustdoc with warnings denied, formatting and diff checks pass. The closure
signature and enum-hierarchy assertions both fail before their respective fixes.

Ten final native flows pass on the same final binary: Rust, JavaScript, TSX, Go
and Python at both 1× and 2×. Each verifies two outline destinations by exact
selected-name copies, exact whole-file copies, unchanged fixture/binary hashes and
zero child exits. Targets include the Rust macro/union, Unicode and nested JS
closures, TS interface method/enum case, Go interface method/constant, and Python
alias/module value. Dark/light captures include the corrected enum siblings.
Rust 2× OCR misread the macro label; the controller locates that row between its
observed Bits and VALUE neighbors, and exact source copies verify the result.
Initial pre-signature/pre-enum runs are retained as development evidence only;
`accepted.json` selects the ten final `radiology-shared-outline-accepted-*` runs.

Architecture baseline: `before-shared-outline-queries` at `59b50cb`. Product and
tests add 140 NLOC and six functions (five tests and the shared rank helper), with
no new files, hotspots, maximum CCN or cycles. Health is effectively unchanged.
All 91 clone groups remain identical after complete normalized multiset comparison.
The comparison's regression label only flags the existing nine-line outline/test
scanner clone touching a modified file; no clone was added or removed.

Archive: `~/.local/state/radiology-qa/shared-outline-queries-20261101-59b50cb` contains
final controllers, fixture JSON, action logs, clipboard bytes, hashes, captures,
workspace/gate logs, negative probes, architecture reports/audit and source diff.
This closes the listed declaration gaps, not broad grammar/corpus or whole-app
parity. The native resized inspector also exposes an existing minimum-height
cleanup: the References heading can clip when the declaration region is shrunk.
That sizing defect is now repaired by [compact summaries and bounded regions](rust-inspector-region-sizing.md). Broader reader/index/PR/cache/UI acceptance remains open. The linked
compact Kotlin interface limitation was subsequently repaired by the
[scanner patch](rust-kotlin-scanner.md). The app stays independent;
full accessibility and actual sibling adoption remain deferred.
