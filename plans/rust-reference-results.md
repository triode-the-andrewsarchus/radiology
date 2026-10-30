# Source-backed reference results

## Query contract

Swift's SemanticQueryWorker lists definitions and uses grouped by file, showing
actual source text. Its reference count excludes definitions, and unmapped or
unreadable sites remain visible as coverage caveats. Rust's new reference-results
query implements this contract independently of the inspector controller.

`SourceQueryInputs` is shared with source previews: captured checkout/pinned-review
storage, exact displayed bytes and an immutable semantic index. The shared read
path primes the displayed file without reading disk. Each other file is read once
per query. Definitions include every direct destination, or every definition
relationship destination when direct definitions are absent. Coordinates are
sorted/deduplicated by path and range, preferring the definition role.

Every listed hit has a validated canonical UTF-8 range, original encoded range,
bounded source-line text and precise visible emphasis. The preview line window
is shared, including UTF-16/UTF-32 validation, CRLF handling, long Unicode lines
and first-line portions of multiline occurrences. Row indentation is removed
without removing the indexed match. File groups retain source SHA-256 identities.

Coverage accounts for all unique indexed sites:

`indexed = listed definitions + listed references + unmapped + unreadable sites + omitted`

Unmapped includes stale generated source identities, unsupported encodings and
invalid/out-of-bounds/empty ranges. Unreadable paths retain bounded error detail
and site counts, including invalid repository paths. Before and cancelled queries
return no results. Imported indexes retain their existing unknown-freshness status.

Default limits accept 64 MiB of source, 16 MiB of retained rows/text/path metadata
and 512 attempted file reads. A sorted prefix is returned with an exact omitted
count; limits can be doubled to request more. Empty display lines still consume
row memory. Reads retain the existing per-file 16 MiB cap and native read/cancel
boundaries; finer interruption remains a broader lifecycle requirement.

## Verification and integration status

Nine new query tests cover counts/grouping, exact UTF-16/emoji matches, duplicate
roles, document-local IDs, stale/range/encoding/unreadable coverage, expandable
bounds, primed bytes, cancellation/Before, all relationship definitions,
multiline/long Unicode sites, pinned Git objects and empty-line memory bounds.
Full gates pass 244 application tests, 5 reader-text tests, 30 doctests, fmt and
Clippy; rustdoc is checked. The architecture comparison against
`before-reference-results` reports no new product hotspot/cycle, unchanged maximum
CCN and a short shared source-read branch report.

The reader now consumes these results. The controller shares one captured query
configuration with definition/reference previews, retires pending reads on source
changes, and keeps valid selection/disclosure state through expanded queries.
All accepted source reads enter the source-identity ledger, even when their sites
are unmapped. Opening a previously unreadable file refreshes coverage once.

The inspector groups source-line rows by file, marks definitions separately and
counts only real uses in its References header. Measured glyph positions keep
the indexed match visible in a narrow row. Group headers and expanded excerpts
share the existing variable-height virtualization. Right-click Go to Reference,
double-click and Return reuse exact indexed navigation; single click remains
selection only. Coverage and Load More controls expose skipped sites and expand
the query limits. Native acceptance is underway. Parsed and
validated call trees, broader context metadata and full parity remain open.

The configured skill locations still lack swiftui-pro, macos-design and
typography-designer. Swift source and established reader type scales remain the
UI reference; no sibling runtime dependency is introduced.
