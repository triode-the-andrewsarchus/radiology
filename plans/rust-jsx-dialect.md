# JavaScript JSX versus TypeScript JSX routing

## Contract and implementation — 2026-11-02

Swift `SourceLanguageRegistry` registers `.js`, `.jsx`, `.mjs` and `.cjs` with the
JavaScript grammar and JavaScript metadata. `.tsx` uses the TypeScript JSX grammar;
`.ts`/`.mts`/`.cts` use TypeScript. Its `SyntaxLanguageTests` explicitly checks JSX
inside JavaScript and the different interpretation of angle-bracket assertions.

Rust previously grouped `.jsx` with `.tsx`. This mislabeled JSX as TSX and silently
parsed a TypeScript variable annotation in a JavaScript JSX file. The registry now
routes JSX through the existing JavaScript branch. This also selects the already
bundled ECMAScript/JavaScript/JSX queries, without TypeScript-specific overlays.
The public Tsx grammar description now names TypeScript JSX accurately. TSX keeps
its distinct parser and current label.

No new grammar, dependency, query, worker, utility or UI state is introduced.
The shared registry updates source footer metadata and coarse index-language
reporting together. Unconfigured JSX now contributes JavaScript, rather than
being indistinguishable from a TSX file. Selected TypeScript/JavaScript adapters
still cover it through existing family/extension rules. Cache display metadata
is already derived on restore, so no format migration is needed. Source bytes,
semantic coordinate encodings and navigation contracts remain unchanged.

## Evidence

One dialect matrix covers 13 cases in LF/CRLF: valid JSX in JavaScript aliases,
uppercase JSX, typed JSX in JSX versus TSX, ordinary TypeScript module aliases,
JSX rejected by the TypeScript-only parser, and a valid TypeScript angle assertion
that is not a valid TSX construct. It checks raw tree error flags, registry
metadata, alias equivalence and coupled reader facts. The old route fails first
because it accepts the typed JSX source. The existing indexing-language matrix
now requires the separate JavaScript family in its unconfigured mixed inventory.

The resource verifier adds a real JSX component with a function call inside
its attribute expression. This exercises parse, outline, folds, calls, occurrences
and native colors through the actual reader. A copied executable passes all
resource checks outside the checkout. The existing ten Swift corpus fixtures and
native color matrix continue to run in the workspace suite.

Workspace: 582 app tests pass, seven ignored; two binary, four font, 13 text and
37 doctests pass. Strict Clippy/check/build/rustdoc/fmt/diff gates pass.

A native old-build 1× control reproduces the TSX label and missing warning for the
typed `.jsx` file. Final patched runs at 1×/2× verify JavaScript labels, a warning
for that typed JSX file, and clean TSX for a byte-identical `.tsx` copy. They check
valid JSX tag/attribute/string/keyword pixels in light/dark, exact CRLF/Unicode
source copies, exact `View` selection, both Back transitions, unchanged fixture/
binary hashes and zero child exits. Representative old/new/error/light captures
were inspected. These checks prove the tested dialect boundaries, not complete
validation by a compiler.

Early 1× QA attempts are excluded: full-window OCR missed the short filename and
misread footer names, including after appearance changes. Tight, inverted-when-dark
header/footer crops fix the observation without changing product code. The 2× v2
run already passed; final acceptance selects it with the v4 1× old/new runs.

Architecture baseline `before-jsx-dialect-routing` at `df36eaf`: NEUTRAL, +52 NLOC,
one test function, unchanged max CCN 132 and zero dependency cycles. The existing
vendored scanner hotspot is unchanged. All 110 clone groups are unchanged by full
normalized multiset audit, zero added/removed; health changes about -0.003.
Existing type roles and layout are reused. Requested SwiftUI/macOS/typography
skills remain absent from installed roots.

Archive: `~/.local/state/radiology-qa/jsx-routing-20261102-df36eaf`.
`accepted.json` selects the v4 1× old/new runs and v2 2× new run. Fixtures, exact
copies, captures, role counts, source/binary hashes, actions/exits, negative and
final logs, standalone report, architecture/clone audit and source diff are retained.
Broader language/build corpora, reader/key/UI, indexing/PR/cache and final Swift
parity acceptance remain open. Full accessibility and actual sibling adoption
remain deferred per user scope; the application stays standalone.
