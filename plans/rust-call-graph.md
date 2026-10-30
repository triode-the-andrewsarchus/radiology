# Rust call summaries and critical frontiers

## Implemented backend (2026-10-30)

The existing shared Tree-sitter parse now captures callable scopes alongside call
sites using Swift scope resources (C supplies its function-definition capture).
Scopes retain exact declaration/name bytes and body presence, including abstract
signatures. A source-ordered stack sweep assigns each site to its innermost
callable; nested functions and closures never become unconditional outer edges.
All nine linked grammars compile and basic function/body ownership is tested.
Parse-error state is retained independently from query support.

`critical::callees::CalleeResolver` is shared by source projection and graph
summaries. It matches a single-line SCIP occurrence starting at the exact callee
name, prefers the whole-token occurrence and otherwise retains the existing
same-start fallback. Document encodings, source digest, local identity scope and
Before isolation are preserved. No spelling search creates an edge.

`critical::graph::summarize` selects an exact parsed callable scope. Direct calls
are deduplicated by SCIP identity with the first source site and total site count.
Macro expansion, unresolved sites, unknown callees, syntax errors and conditional
sites make coverage partial. Directly nested callable nodes have separate direct
site counts; calls in deeper nested closures also prevent an outer false zero.
No-body results distinguish abstract, missing syntax, unsupported language and
missing/stale semantic data. Bodies carry exact path/revision/digest/range.

`critical::frontier::build` traverses ordinary repository callees breadth first,
stops at critical/unknown leaves and ignores external ordinary plumbing. It uses
Swift's defaults (depth four, forty-eight additional bodies), deduplicates cycles,
self recursion, diamonds and direct/indirect duplicates, and retains the shortest
stable via chain. Missing repository bodies and bounds explicitly make coverage
partial. Unknown identities are counted once as gaps and never counted as critical.
Existing CountAnnotation produces proven counts, lower bounds (`0+`) and no body
(`—`). Providers and cancellation can abort without publishing partial success.

## Verification

Seven new summary/frontier tests cover UTF-16 after non-ASCII/emoji, repeated
sites, nested ownership, deeply conditional execution, macros/dynamic calls,
abstract declarations, parse errors, stale/Before/unsupported encodings, graph
cycles/diamonds/shortest paths, external/unknown leaves, missing bodies, depth/body
bounds, provider errors and cancellation. Full verification passes 177 application
tests, 5 reader-text tests, 30 doctests, fmt, Clippy, binary build and rustdoc.
One external Clang test remains ignored by default. Architecture comparison is
IMPROVED: duplication down 0.01 percentage points, no new hotspot/clone/cycle,
max CCN unchanged; the projection hotspot is resolved by shared callee resolution.

## Integration still required

This commit provides the verified graph backend. It is not yet requested by the
reader. A worker must load canonical indexed bodies using the captured checkout
or pinned-review SourceRead, prime the displayed immutable bytes, validate source
provenance, cache by content identity, and retire publications on source/index/
policy changes. Hover and fold-count requests should reuse that worker without
reintroducing the removed automatic Critical sidebar. Native count/hover acceptance
and broader callable fixtures remain required. The old references-derived caller
tree is not replaced or relabeled as a validated parsed call graph here.

## Snapshot worker and fold-count service (2026-10-30)

`CallGraphWorker` now loads canonical indexed definitions through captured
`SourceRead` checkout or pinned-review storage. Displayed immutable bytes/call facts
prime the worker and win over disk; other bodies are parsed once per request.
Each instance owns one captured index/policy/source configuration. Request-local
files retain a coherent read snapshot; no cache crosses configuration changes.
Source provenance and exact encoded name coordinates are checked before summary
construction. Missing, abstract, unsupported and stale bodies retain no-body
reasons. Additional source is limited to 64 MiB and 512 file entries; summaries
cache at most 512 identities. Resource exhaustion explicitly truncates the count.

`CriticalAnnotations` owns a replaceable fold-only sweep on the existing bounded
read pool. It shares one worker across function roots, computes no inspector
sections or outline counts, and publishes exact-source/revision/digest-bound
counts and full coverage explanations. No current UI requests this service yet.
Publication is one completed batch; progressive six-function publication and
hover requests remain integration work.

Six new tests verify cross-file ordinary traversal, displayed-buffer precedence,
request-local cache coherence, policy replacement, source budgets, stale data,
abstract/external results, cancellation, pinned Git objects despite changed
checkout bytes, Before isolation, fold scope, exact annotation identity and retired
publication. Full checks pass 183 application tests, 5 reader-text tests,
30 doctests, fmt, Clippy and rustdoc. Architecture retains one ten-line test
assertion clone shared with frontier tests; no new product clone/hotspot/cycle,
maximum CCN unchanged and average CCN down 0.02. Native I/O/Tree-sitter cancellation
is still checked around each operation rather than interrupting it internally.

## Source fold-count integration and acceptance (2026-10-30)

The standalone reader now requests the annotation service in Critical mode when
an index and displayed source are available. Source/index/policy/repository/mode
transitions retire old sweeps and publications; Before never consumes the After
index. The previous usable index/count remains visible while a replacement index
loads, and replacement publication starts a fresh count request. Caret movement
schedules no graph work. Color projection remains a separate job, so graph reads
do not block call-focused source presentation. Shutdown cancels count work.

Collapsed callable folds show an outlined reachable count in Critical mode;
partial coverage reads `n+`, proven zero/no-body recede. Normal mode retains hidden
line counts. Non-callable folds retain their ordinary presentation. Count tooltip
and accessible button text retain hidden-line totals plus coverage reasons. The
existing gutter now has separate disclosure/count/line-number lanes (104 points,
matching the Swift layout), reusing existing fonts and colors. Clicking either
disclosure or count still toggles the fold. No inspector or outline count section
was added.

Application verification covers policy recomputation with source/history/folds
preserved, mode removal/restoration, replacement-index recomputation, Before
isolation, document changes and repository retirement. A pointer replay verifies
the actual painted count differs from hidden-line count, activates the same fold
and preserves source bytes/caret. Full checks pass 185 application tests,
5 reader-text tests, 30 doctests, rustfmt, Clippy and binary build.

Private native Weston/Niri UTF-16 fixture acceptance verifies Normal's two hidden
lines, Critical's `1+` (system call plus unresolved site), expansion by clicking the
count, a source-menu Non-Critical decision yielding `0+`, and the tooltip retaining
two hidden lines plus partial/unresolved coverage. Normal restoration retains the
fold. Screenshots are under `/tmp/radiology-count-native/`: `normal-hidden-lines.png`,
`critical-count.png`, `expanded-from-count.png`, `partial-zero-and-detail.png` and
`normal-restored.png`. App and both compositors exit zero. The reusable sibling
acceptance harness is invoked only for testing; no runtime dependency was added.
Architecture retains short test assertion/fixture clones and pre-existing footer
label sequences; no new product hotspot or cycle, max CCN remains 100 and average
CCN is down 0.02 from before-graph-worker. Duplication remains about 0.7%.

Still required: pointer-dwell hover summaries/navigation/grace behavior,
progressive fold-sweep publication and broader corpus/large-file/cancellation
acceptance. Parsed graph facts do not yet replace the references-derived caller
tree. Full Rust/Swift parity remains open.
