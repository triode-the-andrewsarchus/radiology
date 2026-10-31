# Native source colors

Swift, Kotlin, JavaScript, TypeScript and TSX/JSX use native Tree-sitter highlight
captures for source colors.
The existing shared parse produces the immutable capture snapshot; no parser runs
in painting or theme recoloring. Other languages retain their Syntect appearance.
Missing/failed native facts fall back to basic Syntect/plain colors, with a
recoverable Basic syntax colors status and error detail when extraction failed.

## Query and ownership reuse

The query registry shares highlight source between native coloring and syntactic
occurrence exclusions. Swift uses its shipped query with the native nil-node
adaptation. kotlin-ng uses native literal/comment/keyword names plus declaration,
call, type, property, constant, label and annotation roles. Contextual boolean/null
identifiers use text predicates. Captures remain syntactic, never semantic facts.

The bounded capture visitor consolidates match limits, fact limits and native
predicate execution for highlights and occurrences. SourceIdentity centralizes
path, typed revision and digest checks for native colors, occurrences and calls.
Failed captures publish no partial native color set and do not disable independent
fold/call analysis. No new grammar dependency, sibling runtime or UI setting is
introduced.

NativeHighlights retains canonical byte ranges, capture names and query order.
A sweep paints wider/earlier captures first and lets nested/later captures win,
matching Swift's published order. Half-open ends expire before following source
text. Output spans cover original line bytes, including Unicode and CRLF, without
normalization. Source typography stays regular monospace. Role colors follow the
same desktop palette categories; exact macOS dynamic RGB values are not claimed.
Swift's shipped query leaves escape nodes without their own style; Kotlin's
explicit escape captures receive the orange role.

The host carries the capture Arc into exact-buffer recoloring requests. Such
requests reuse the source Arc and publish no new DocumentSyntax. Mismatched path,
revision or digest rejects the captured facts; theme-only requests do not read
checkout bytes or rebuild structural/occurrence analysis. Retirement is checked
before and during recoloring. Parsing/query interruption remains bounded by the
existing stage/fact limits rather than a new fine-grained cancellation mechanism.

## Verification

Full gates pass 318 app tests (one external-Clang fixture ignored), eight shared
reader-text tests, 30 doctests, fmt, strict Clippy, build and rustdoc. Tests cover
native query compilation, full Unicode/CRLF line coverage, role colors in both
themes, escaped text, nested/later precedence and expiry, Kotlin native-node roles,
limits with independent syntax facts intact, cancellation, identity rejection,
and retained-buffer theme publication with no syntax parse/read. Existing all-
grammar occurrence, fold, graph, source and navigation tests remain green.

Architecture against `before-native-highlights`: +434 NLOC, two Rust files,
14 functions, unchanged average/max complexity and 91 clone groups. Duplication
falls by 0.01 percentage points. No new hotspot or dependency cycle. Two initial
clones were removed by sharing the query visitor and typed source identity.
REGRESSION classification refers only to the two unchanged source-view/context-
inspector test fragments present in the baseline.

Broader language corpus, accessibility, scaling and visual acceptance remain open,
as do PR refresh mapping, source/graph/context caches, background/index/PR lifecycle
and sibling palette/chrome integration. Full Rust/Swift parity is not complete.


Native evidence lives in `/tmp/radiology-native-colors/`: `swift-dark.png`,
`swift-light.png`, `kotlin-folded-light.png`, `kotlin-light.png`, `kotlin-dark.png`
and `build-evidence.json`. Both CRLF fixtures contain a Unicode comment, a type,
property, function, parameter, string, call, number and return. Swift's expanded
body keeps native roles after theme recoloring. Kotlin's folded header and
expanded body both retain native type/function roles; the formerly plain body
shows keyword/property/string/call/number colors in both appearances. This is
Rust-native acceptance against shipped role rules, not a pixel comparison with
macOS dynamic colors.

Final binary SHA-256 is `bf7747648fec835711f923fdcb996357a1a485e1aafb4972554ba76ec696b0dc`, based on `50a73c6`
plus the capture preparation and source-loader integration. Native app PID 115583
and private Weston/Niri exited zero; persistent pointer helpers were intentionally
terminated. The external driver uses the sibling compositor helper only for
acceptance. Product builds require no sibling checkout.

## ECMAScript family parity — 2026-10-30

The audit found TypeScript/TSX still routed through Syntect's JavaScript fallback,
even though the shared query registry already included TypeScript type captures.
NativeHighlights now accepts JavaScript, TypeScript and TSX grammars as well as
Swift/Kotlin. The existing source loader automatically selects these exact-source
captures and retains them for recoloring. No new parse, renderer or syntax asset
package is introduced. Syntect/plain fallback remains available on query failure.

Query assembly now includes the existing Swift JavaScript-only field overlay and
JSX tag/attribute overlay for JavaScript/TSX, matching Swift's dialect composition.
TypeScript retains its type/modifier overlay. The same source serves occurrence
exclusions and coloring; compile tests cover the enabled grammars. Native colors
remain syntactic decoration, never substitute SCIP identity or navigation.

A failing-before regression exposed a.ts lacking native captures. The final shared
language matrix covers Swift/Kotlin/JS/TS/MTS/CTS/TSX/JSX, both themes, type/modifier/
function/property/tag/attribute/string/comment roles, exact contiguous original
Unicode/CRLF span bytes, regular source typography, cancellation and path/content/
revision rejection. The existing asynchronous loader test now covers Kotlin,
TypeScript, TSX and JavaScript with retained capture/source Arcs, no new syntax
parse and foreign-capture fallback. Earlier language tests were consolidated;
the test total is not inflated by the added matrix cases.

A native private 1280×900 Wayland run verifies a TSX document containing an
interface, readonly property, typed function/string and JSX element. Dark and light
captures retain their native roles, outline and caret line. Copy All reproduces
the original CRLF bytes exactly; fixture hashes stay unchanged and app/Weston/Niri
exit zero. Durable captures, driver, binary/fixture hashes and copied bytes:
`~/.local/state/radiology-qa/typescript-highlights-20261030-9a5f99a` with SHA256SUMS.
This is representative 1× TSX evidence, not every dialect/corpus/scale acceptance.
The external QA driver uses sibling compositor helpers; product builds remain
standalone and have no sibling checkout dependency.

Workspace passes 443 app tests (seven ignored), four font tests, eight text tests
and 33 doctests, strict Clippy/build/fmt and warning-denied, inspected rustdoc.
Architecture against before-native-typescript-highlights is neutral: no new clone
group, cycle or maximum complexity increase. A real duplicated assertion block
was removed by sharing byte checks and the language matrix. Broader corpora and
resource-verifier coverage remain open; full accessibility is deferred.
