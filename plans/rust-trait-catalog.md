# Explicit Rust trait implementation catalog

This is the shared data foundation for the upstream trait implementation feature.
The catalog/extraction/cache/resolver APIs are implemented and verified. Automatic
index-publication enrichment, caret-context selection and the inspector list are
still required; this commit does not claim the trait UI is available.

## Source and identity contract

`syntax::rust_impls` extracts explicit, well-formed Tree-sitter `impl_item` blocks.
It selects only the terminal implemented trait name, peeling generics and qualified
paths. Bounds, generic arguments, inherent impls, comments and unexpanded macro
`token_tree` contents cannot create trait rows. Negative/unsafe impls and exact
constraints/type spelling survive. Full `DocumentSyntax` gets the same facts from
its existing parse; catalog building has a lightweight cancellable pass with no
highlight/fold/call work. Parser errors remain coverage gaps, not false empty proof.

`semantic::trait_impls::TraitImplementationIndex::build` reads immutable source
through the existing checkout/pinned-After storage API. It maps SCIP ranges through
`SemanticIndex::byte_range`, preserving document/default column encoding and source
provenance. A header requires exactly one distinct nonlocal, nondefinition reference
at the terminal token's exact byte range. No spelling join or SCIP relationship
fallback is used. Duplicate contributions do not duplicate source blocks.

Rows retain original impl/name ranges, signature, implementing type, line, negative
flag and full-source SHA-256. `resolve` re-reads the selected storage, validates the
entire digest and range, and returns canonical bytes/revision; same-length changes
are rejected. Checkout symlinks leaving the root are rejected before source reads.
Pinned reads request After directly from owned Git objects. Before context must be
cleared by the future UI/model rather than mapped against After semantics.

`SemanticIndex` has an immutable Arc catalog slot, unprepared by default. This keeps
one accepted semantic snapshot and prepares direct-host reuse; merely constructing
an index from SCIP does not silently read a repository or run enrichment on the UI.

## Cache and coverage

The optional app-owned derived cache lives outside the source root. Format/extractor
version1 keys every document by exact SHA-256 source bytes, default/document encoding,
and ordered semantic identities, roles and coordinates. Cached rows still require a
fresh source read and provenance check. Source or semantic changes reparse only
changed records; corrupt/unsupported caches rebuild. Cache read/write failures affect
speed only, and atomic temporary-file replacement avoids publishing partial JSON.
The format version must change when extractor semantics change.

Caller-supplied exclusions and source-stale generated documents cannot contribute.
Coverage separately records unresolved headers, unreadable files, parser/coordinate
errors, excluded files and Rust files missing from the semantic index. Unknown
repository inventory uses None, never a fabricated zero. Imported semantics retain
an explicit unknown-freshness flag and explanatory text. Even with no known gaps,
coverage is limited to explicit indexed source: macro-generated/inactive impls may
be absent. Work counters distinguish parsed/cache-hit documents without UI promises.

## Evidence and limits

- The existing upstream scip-rust-traits fixture has no symbol relationships. Its
  actual compiler index resolves Alias/qualified/blanket impls to left::Render,
  keeps same-named right::Render separate, retains negative Marker and no rows for
  Empty. Duplicate occurrences leave each source block unique.
- Cold/warm builds verify2 parses then zero parses/two cache hits. Source edits,
  semantic remapping, definition roles and encoding changes invalidate relevant
  records. Corrupt/unwritable/in-repository cache paths preserve correct results.
- Exact UTF-8/16/32 cases include a Unicode prefix before the trait token. Ambiguous,
  local, definition-only, overlapping and unsupported-encoding sites are rejected.
- Tests exercise explicit exclusions, missing/unindexed/unknown inventory, stale
  source identities, external symlinks, partial parser recovery and cancellation
  after real processing starts. Activation rejects a same-length Alpha→Gamma edit.
- Final workspace: 661 app tests pass, seven existing ignored; two binary,13 font,
  21 text and48 doctests pass. Strict Clippy/check/build/rustdoc, formatting and diff
  gates pass; rendered public extraction/catalog/resolver API pages inspected.
  This foundation has no new native UI acceptance claim.

## Architecture review

Baseline `before-trait-implementation-catalog` at597fcc0. Reuse lookup identified
SCIP mapping, SourceRead, source digests, cancellation, atomic temp files and cache
path validation. Record cache eligibility and extraction are cohesive operations;
the main build loop has no new complexity hotspot. Exact-alias fixture lookup
reuses one coordinate-based helper rather than duplicated test mutation setup.

| Dimension | Before | After |
| --- | --- | --- |
| Analyzed NLOC | 94,390 | 95,288 (+898, including tests/docs) |
| Functions / files | 3,977 /607 | 4,006 /611 |
| Average /maximum CCN | 3.38 /132 | 3.38 /132 |
| High-complexity functions |33 |33 |
| Clone groups /cycles |125 /1 |125 /1 |

Architecture result is NEUTRAL; no new clones, hotspot or cycle. No new dependency,
worker pool or parser library was introduced. Actual publication will use existing
workers and accepted index lifecycle rather than parallel trait-specific ownership.

## Remaining wiring

Enrich generated, validated-cache and imported indexes before publication, using the
full inventory, cache directory and correct checkout/review storage. Keep imports'
unknown freshness and exclude stale contributions. Then implement trait/member/impl
caret targeting, stable selection, compact rows in the existing middle pane, keyboard
activation and source-verified navigation with generation/index/row guards. The pane
preference and slot policy already exist; see `rust-inspector-panes.md`.

Evidence: ~/.local/state/radiology-qa/trait-catalog-20261102-597fcc0.
The broader goal stays in `rust-upstream-parity-2026-11-02.md` and the parity ledger.
