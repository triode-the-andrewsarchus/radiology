# Background source loading

`source::loader::SourceLoader` prepares immutable source on the existing bounded
blocking pool. The reader now routes file opens, bookmarks, encoded definitions, revision
changes and new-document history navigation through this worker. Theme changes
recolor immutable current bytes or restart the pending destination with the new
appearance. Repository discovery and index imports are also worker-owned now;
see [repository opening](rust-repository-opening.md) and [index lifecycle](rust-indexing-parity.md).

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
  parser progress cancellation remain open in the source-loader path. Discovery
  and checkout refresh now use bounded process-group cancellation; SCIP imports
  are worker-owned with finer decode interruption still on the lifecycle queue.

## Reader integration invariants (implemented)

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

## Reader verification (2026-10-28)

`RadiologyApp::poll_source` is available to embedded hosts and called by the native
frame loop. A pending strip names the destination and offers cancellation while
all current source-derived facts remain attached to the old buffer. Publication
commits history and source together. Back/Forward advance their cursor only on
success; cancellations/errors leave it intact. Departure coordinates are captured
at publication, including movement made while another file was preparing.
Returning to the current document retires a pending replacement. Repository and
changed review snapshots retire queued source work. Pinned reviews/highlighters
are shared through Arc; no synchronous bookmark/definition/revision pre-read is
retained. Index byte conversion and definition requests use one encoding lookup.

The existing reader tests now poll real asynchronous completions. Added tests cover
retained source/history during pending/cancel/failure, latest departure movement,
current-document and repository supersession, pending navigation across appearance
changes, recoloring after deleting the backing file, and default opens versus
explicit navigation into a first test region. Latest suite: 78 application tests,
5 reader-text tests, 30 doctests; formatting and Clippy warnings denied pass.

Native Niri smoke used an isolated settings directory and a 100,000-statement
fixture. `/tmp/radiology-native-source-loading-pending.png` shows the opening strip
with the prior source/outline intact. The file picker remained usable; returning
to the current source retired the replacement, and a subsequent appearance change
published correct colors. `/tmp/radiology-native-source-loading-theme-settled.png`
retains the resulting source. This is smoke evidence, not a benchmark or a claim
that every analysis stage is immediately interruptible.

Repository discovery, checkout branch/status/diff refresh and SCIP imports now run
on workers. Native parser/analysis stages and source-loader Git child processes
still finish their current stage after retirement; finer interruption and source-
loader process deadlines remain required. Context/graph/cache and broader index/PR
lifecycle work remain on the full parity queue.
