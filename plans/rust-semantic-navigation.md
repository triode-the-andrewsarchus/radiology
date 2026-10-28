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

## Integrated entry points and acceptance

Workspace symbol selections carry scoped symbol identity; caller rows carry the
original encoded reference range. Both use the same destination path as F12,
including canonical definition resolution, encoding conversion, fresh source
reads and generated-index digest checks. Before review uses its document outline
instead of exposing After workspace symbols. Unsupported encodings do not
advertise semantic availability.

The document/workspace symbol picker shares one rendering and input path, renders
only visible rows, preserves keyboard visibility, and handles query text plus
Enter in the same frame. The outline traversal borrows its input rather than
recursively cloning nested trees.

Validation: 128 application tests, 5 reader-text tests and 30 doctests pass, as do
fmt, Clippy with warnings denied and the binary build. Regression tests cover
keyboard/pointer identity, same-frame query selection, 1,000-row keyboard reveal,
exact caller ranges, UTF-16 destinations after emoji, newer same-path cache
reloads, stale fresh bytes retaining source/history, absent provenance and Before.

Native acceptance in a private 1280×900 Weston/Niri session verified workspace
selection by keyboard and pointer, caller navigation, and F12 returning to the
exact target after emoji. This used an imported fixture SCIP file with explicit
UTF-16 encoding; compiler-indexer acceptance is separate. Captures are local
artifacts under `/tmp/radiology-semantic-navigation-native/` (`keyboard-target.png`,
`caller-reference-target.png`, `definition-return.png`, `pointer-target.png`). App,
Niri and Weston exited zero; the user's desktop session was untouched.

Architecture comparison against the implementation baseline remains REGRESSION:
duplication falls from 1.38% to 0.74%, maximum CCN from 125 to 100, and cycles remain
zero. Picker consolidation removes a reported show hotspot and duplicate product
branches. Remaining reported clones are syntax fixtures and GUI test input/text
extraction. The current update function grows from CCN 51 to 53 due to two required
encoding-availability checks; review/workspace/generation orchestration flags
remain. These are recorded debt, not a clean architecture verdict. No new worker,
job protocol, host dependency or alternate navigation pipeline was introduced.

## Remaining full parity work

This boundary does not provide parsed call graphs, reference snippets, independent
inspector keyboard selection, source definition previews, persistence of viewport/
pins/folds, validated index caches, review indexing or complete language queries.
The independent Rust/Swift review remains the full requirement ledger.
