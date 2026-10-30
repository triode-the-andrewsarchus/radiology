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
Full gates pass 246 application tests, 5 reader-text tests, 30 doctests, fmt and
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
the query limits. Native acceptance is recorded below. Parsed and
validated call trees, broader context metadata and full parity remain open.

The configured skill locations still lack swiftui-pro, macos-design and
typography-designer. Swift source and established reader type scales remain the
UI reference; no sibling runtime dependency is introduced.

## Native acceptance — 2026-10-30

Final code commit `ad79ecc` was built and exercised as PID 3688886 in a private
1280×900 Wayland session. Evidence, screenshot hashes and the executable SHA-256
are in `/tmp/radiology-reference-results-native/build-evidence.json`. These are
local acceptance artifacts, not required runtime inputs.

The 518-site fixture includes UTF-16 coordinates after an emoji, definitions,
multiple uses, a missing file, an invalid range and 513 extra source files.
The initial query shows 511 references plus one definition, one unmapped site,
one unreadable file and four omitted sites. Load More shows all 515 references
plus the definition, retaining the selected/open source excerpt and caveats.
Narrow rows retain the highlighted match with cropping markers. Single-click
selection followed by Right expands source context. After Load More, two Down
keys select the second-file use, and Return opens `b.rs` at line 5. Back restores
`a.rs` at the exact use after the emoji while retaining explicit reference focus.
Find References was invoked from the native source context menu in this run.

Screenshots: `accepted-limited.png`, `accepted-expanded-before-more.png`,
`accepted-complete-after-more.png`, `accepted-selected-use.png`,
`accepted-return-b-use.png`, and `accepted-back-a-exact.png` in that directory.
Radiology, Weston and Niri exited zero; persistent test pointers were intentionally
terminated during cleanup. The sibling project supplies only the private test
harness, with no product dependency.

A hardware watchpoint under rust-gdb isolated the Load More focus failure to
egui list registration surrendering focus during a coalesced press/release.
`focus-gdb.log` records the stack. The regression test failed with that exact
one-frame input sequence before the fix; deferred focus restoration now passes
the test and native Down/Return acceptance. No debugger instrumentation or
system security changes were added to the product.
