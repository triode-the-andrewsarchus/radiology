# Rust semantic navigation

## Coordinate boundary (2026-10-28)

`SemanticIndex` uses a single canonical ordering of valid definition starts for
workspace search labels, direct definitions and relationship fallback. Definition
selection is independent of SCIP document order. Caller nodes preserve their
original reference `ScipRange`; their line field is presentation only.

Packed/typed SCIP ranges reject negative or reversed coordinates. An explicitly
present invalid typed range does not fall back to conflicting packed coordinates.
Empty ranges remain valid. Large positive line numbers can be represented without
signed overflow; source resolution still checks actual file bounds. Unknown
position encodings return no encoding instead of silently becoming UTF-8.

`SourceTarget::accepts_source_identity` is used by the source cache fast path and
worker destination resolution. A cached buffer with a different expected digest
is a cache miss: the worker reads current bytes before checking the destination.
This supports a newly generated index while an older same-path buffer remains
visible. Changed fresh bytes fail without replacing source/history.

Current workspace validation, including UI integration in progress: 128 application
tests, 5 reader-text tests and 30 doctests, fmt, Clippy and binary build pass. UI
consumers are the next commit: workspace selection must preserve scoped symbol
identity, references must preserve ranges, both must share encoded/digest-checked
source requests, and Before must not expose After workspace symbols.

## Remaining full parity work

This boundary does not provide parsed call graphs, reference snippets, independent
inspector keyboard selection, source definition previews, persistence of viewport/
pins/folds, validated index caches, review indexing or complete language queries.
The independent Rust/Swift review remains the full requirement ledger.
