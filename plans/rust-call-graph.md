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
