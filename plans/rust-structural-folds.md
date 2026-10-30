# Typed structural folds and Fold to Definitions

## Implemented behavior (2026-10-30)

Fold to Definitions now replaces the current collapsed set with callable bodies
and type bodies containing no nested declarations. Containers exposing methods,
variants, properties or other captured declarations stay open. Comments, imports
and generic blocks are expanded by the command. Collapse Functions remains a
union operation that preserves unrelated manual folds. Both commands mark the
current document initialized, so source defaults cannot replay over user choices.
File-return restoration uses the existing exact-document memory.

## Parsed facts and reuse

The existing shared Tree-sitter parse now runs a fold/declaration query role
alongside the independent call/scope role. `syntax::queries` extracts the previous
call registry into one independently cached role loader. Rust, Python, Go,
ECMAScript/TypeScript/TSX and Swift reuse the shipped Swift fold and symbol queries.
Kotlin adapts identifiers, blocks, comments and imports for kotlin-ng. C uses
explicit native-grammar captures. No new parser invocation, runtime or dependency
is added, and no sibling checkout is needed to build/run Radiology.

Each FoldRange retains its exact byte span, typed role, callable header ownership
and nested-declaration containment. Roles use Swift's precedence: callable, type,
comment, imports, block. Canonical exact-range duplicates keep the stronger role;
for the current single-disclosure-per-physical-row gutter, same-row candidates
prefer that role and then the larger extent. This makes a same-line method body
win over its enclosing type without hiding unrelated later methods. Multiline
signatures remain outside body folds. Old text-prefix comment guessing is removed:
Rust raw strings containing comment-like lines are not block comments.

Declaration containment comes from symbol-query ranges, including Rust enum cases
and Kotlin properties absent from the current outline model. It does not infer
semantics from spelling or a type's display label. Callable owner metadata reuses
the existing annotated-range mechanism. Kotlin lambda folds own their own literal,
so an edit to one callback does not open sibling callbacks in the same call.
Initial PR folding now consumes these exact typed spans directly rather than
reconstructing bodies from line pairs. Defaults depend on successful fold facts,
independently of call-query coverage.

Folding queries enforce match/fact limits. Failed folding yields no partial fold
set, leaves calls/source usable, and reports a recoverable Folding unavailable
footer with error detail. Unknown languages remain unfolded. The complete source
bundle and existing request retirement prevent stale query errors/folds crossing
source or repository/review transitions.

## Verification

Six query tests cover all nine linked grammars, parsed roles and exact bytes,
method/variant/property containment, command replacement, comment-string impostors,
callable-vs-block precedence, fact-limit failure with independent calls intact,
and Kotlin sibling-closure ownership. A standalone app test verifies command
routing, preserved bytes/caret/history and file-return choices. Existing defaults,
pinned-review, source, graph, hover and pin tests pass. Gates: 210 application
tests, 5 reader-text tests, 30 doctests, rustfmt, Clippy with warnings denied,
binary build and rustdoc. One external Clang test remains ignored by default.

Private Weston/Niri acceptance invokes the real View menu. The resulting source
keeps impl/module method headers, enum variants, comments and imports visible,
collapses the leaf struct and callable bodies, and restores the result after
switching to b.rs and returning. Artifacts are under
`/tmp/radiology-definition-folds-native/`: `view-menu.png`,
`folded-to-definitions.png`, `definition-folds-restored.png`. App and both
compositors exit zero. `initial.png` records the temporary source-loading state
before query preparation completed, not the final reader acceptance image.

Architecture comparison reports lower duplication (-0.01 percentage points) and
average CCN (-0.01), unchanged max CCN (100), no new product hotspot/cycle, and
short app-fixture/existing-footer clone reports. It still classifies REGRESSION
because those reports touch modified files; the result is not claimed neutral.
The shared registry replaces duplicated role setup and typed policies replace
outline-based inference, with no generic scheduling layer added.

## Remaining parity limits

The gutter still exposes one fold per physical header row. Exact multiple-fold
controls on the same row, Kotlin import grouping, broader C++ grammar support,
outline/query unification, large-file/scaling/light-mode acceptance and PR-refresh/
late-data viewport mapping remain open. The grammar fixture checks establish the
supported cases, not exhaustive language coverage. Full Rust/Swift parity remains
open.
