# Background source loading

`source::loader::SourceLoader` prepares immutable source on the existing bounded
blocking pool. This backend is implemented and tested; UI navigation is not yet
routed through it. Do not treat this document as proof that file opening is
already nonblocking.

## Implemented contracts

- `SourceRead` captures attached-checkout storage, an immutable pinned review, or
  an existing source Arc. All paths are validated as repository-relative.
- `SourceTarget` resolves exact original bytes, line/scalar columns, strict encoded
  index positions or relocated bookmark anchors after reading the target source.
- `PreparedSource` contains shared bytes, structural facts, highlights, exact
  destination, captured appearance, and file-specific diff data. Existing-buffer
  recoloring omits new structural facts and preserves Arc identity.
- `LatestRead` owns cancellation and the sole completion receiver. Replacing,
  stopping or dropping it retires publication, including queued jobs. Source
  preparation and local find use the same mechanism.
- At most four blocking read/analysis jobs are active. Source preparation checks
  retirement between stages and highlighting checks between source lines.
- In-flight native parsing and Git subprocesses currently finish their stage;
  they are not forcibly interrupted. Process timeouts/cancellation and native
  parser progress cancellation remain open. Repository discovery, existing SCIP
  decoding and repository-wide Git refresh remain separate outstanding work.

## Reader integration requirements

1. Keep the current source, caret, pins, folds and revision until replacement is
   ready. Publish all new source-derived facts together; show pending path/cancel
   and recoverable failures. A failed request preserves the current reader.
2. Repository or review snapshot transitions retire source jobs. New navigation
   supersedes older jobs; returning to the current document cancels a pending
   replacement without re-reading the current source.
3. Commit history only when a destination is available. Back/Forward must avoid
   premature history cursor changes, preserve departure movement while loading,
   and leave history intact on failure or cancellation.
4. Route bookmarks, index-encoded definitions and revision line mapping through
   `SourceTarget`; do not synchronously pre-read files to calculate destinations.
5. Retain appearance identity. Theme changes must retire stale colors and share
   immutable bytes for recoloring. A pending file request must not be replaced by
   a recolor of the old source, losing navigation intent.
6. Plain default file opens must not reveal a first test region accidentally;
   explicit byte/line/history/search/definition destinations should reveal it.
7. Integration tests must poll real asynchronous completions. Do not add a test
   configuration that makes production workers synchronous.

## Evidence

Real-source tests rapidly replace a request, verify exact Unicode/indexed byte
coordinates and one-time completion, report a missing file, and recolor the same
Arc after deleting its backing file. Existing local-find worker replacement and
focus tests pass through the shared retirement mechanism. The full suite has
74 application tests, 5 reader-text tests and 30 doctests passing, with formatting
and Clippy warnings denied.
