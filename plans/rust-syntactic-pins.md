# Syntax-aware occurrence pins

Swift's fallback is a drawing-only same-spelling group in the innermost captured
function or closure. It excludes comment, string, number and keyword highlight
spans. It intentionally includes shadowed spellings and nested declarations
inside the chosen outer scope. It provides no binding identity and must never
feed definition/reference/caller navigation. Outside a callable, no syntactic
group is available. Complete SCIP groups take precedence; a syntactic group wins
when SCIP omits some of its local spelling ranges, including cfg-inactive code.

Follow-up: [transient occurrence selection](rust-occurrence-selection.md) now reuses
the same resolver for ordinary source clicks and adds outlined pin styling.

## Parsed facts and publication

`SyntacticOccurrences` is prepared during the existing shared background parse,
independent of call-query availability. The query registry reuses shipped scopes
and highlights (including TypeScript overlay), with native Kotlin/C exclusion
captures and the Swift nil-node adaptation. All nine native grammar queries
compile. Excluded spans use the same shared interval-union routine as byte fold
projection; a duplicate union implementation was removed.

The immutable facts hold callable ranges and source-ordered spelling ranges.
Tokens follow Swift's byte rules, including high bytes and underscore, rejecting
an ASCII-digit first byte. Excluded overlaps are filtered before grouping. The
clicked spelling's indexed ranges are bounded to the smallest containing callable.
There is no regular-expression scan or new parse on pin clicks. Path, typed
revision and content digest must all match the displayed buffer. Repository
transitions retire facts; theme-only preparation retains the current facts.

Query match/fact limits and token limits reject the entire preparation on failure.
Failures leave unrelated source/folds/calls usable and show a recoverable
Occurrence highlighting unavailable status with details. Unsupported grammars
produce no fabricated whole-file lexical group.

Pinned-group choice follows the Swift policy above, while preserving existing
range reconciliation, colors, unlimited groups, document return and successful-
source publication rules. A second click on another occurrence toggles the same
group. [Transient selection](rust-occurrence-selection.md) now shares this resolver for
ordinary single-click highlighting and exact active-range styling.

## Verification

Full gates pass 309 app tests (one external-Clang fixture ignored), eight shared
reader-text tests and 30 doctests, fmt, strict Clippy, build and rustdoc. Tests
cover all native grammars, exact Unicode/CRLF, whole spellings, comments/strings/
raw strings/numbers/keywords, nested functions/closures, intentional shadowed
spelling grouping, stale path/revision/digest rejection, unsupported grammars,
limits with independent calls intact, and partial versus complete semantic pin
identity. Existing pin persistence and source/navigation tests remain green.

Architecture against `before-syntactic-pins` adds 358 NLOC, two files and 15
functions; average/max complexity and 91 clone groups remain unchanged. No new
product hotspot or cycle. An initial extraction hotspot was separated into query
collection and token preparation; a new duplicate union was replaced with shared
reuse. REGRESSION flags refer to six baseline test fragments in touched app/
source-view files, already recorded in the baseline. Broader source/graph caches,
background/index/PR lifecycle, accessibility/scaling/visual acceptance and sibling
palette/chrome reuse remain open. Full parity is not complete.


Native evidence is `/tmp/radiology-syntactic-pins-native/`: `expanded.png`,
`partial-scip-pin.png`, `string-rejected.png`, `comment-rejected.png`,
`closure-pin.png`, `inactive-occurrence-unpins.png` and `build-evidence.json`.
A real imported SCIP document reports only the first clone call. Double-clicking
that call pins both code spellings, including the cfg-gated second call, while
comment/string double clicks add no group. A closure parameter/body produces its
own two-range group. Double-clicking the unindexed clone occurrence removes its
original group without affecting the closure pin. Final binary SHA-256 is
`870cdaad31c6f37338903cc65a229ba09541f71755d8849941758af8f3fcaba9`, based on `46cc9c8` plus this change.
App PID 4171407 and private Weston/Niri exited zero; persistent pointer helpers
were intentionally terminated. The external driver uses the sibling compositor
helper only for testing; no sibling runtime dependency was introduced.
