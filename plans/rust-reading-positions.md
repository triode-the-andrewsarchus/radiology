# Per-file reading positions — 2026-11-05

## Contract and implementation

Swift `AppModel.openFile` restores `positions[location.positionKey]`; its
`updatePosition` records the latest reader report. `SourceViewerCoordinator`
reports both caret changes and passive scrolling. A later scroll replaces the
reading anchor without moving the live caret. The scrolling callback uses x=0,
including visual rows inside wrapped source lines. Exact navigation and Back
continue to use their own caret locations. New repositories and refreshed PR
snapshots clear the memory; refresh explicitly seeds the mapped current viewport.

Rust now follows this contract through the existing accepted-destination
transaction. Only implicit file opens request memory restoration; explicit jumps,
bookmarks and history destinations retain their requested bytes. ReadingPositions
reuses DocumentMemory, keyed by path/revision and content digest, and stores only
an optional UTF-8 byte. Failed or canceled publications cannot overwrite it;
changed content invalidates the old anchor. Repository and PR snapshot installs
reset it. The current refresh viewport is seeded separately from the mapped caret.
Initial paint does not erase a remembered column, while later viewport changes
record the source anchor. Memory remains ephemeral, as in Swift.

The measured viewport reuses its existing shaped hit testing to report the first
visible visual row. Restore uses the existing deferred reveal machinery with
Center, Nearest and Top alignment instead of a keyboard boolean. Soft-wrap
boundaries prefer the following visual row, preventing a one-row upward drift.
Restored destinations reveal enclosing folds/test regions through existing paths.
The host and shared text/font crates retain their existing boundaries; no new
persistence service, worker, dependency or public API is introduced.

## Evidence

Evidence is retained outside the repository at
`~/.local/state/radiology-qa/reading-positions-20261105-ae94f33`;
temporary runs are in `~/.cache/radiology-qa-reader-positions`.

- Original implicit reopen regression fails at byte0 instead of408. The wrapped
  regression separately fails because the old viewport reports source-line start.
- Two memory tests cover caret-column/scroll precedence, explicit targets,
  Before/After identity, changed digest and UTF-8 clamping.
- Four actual-app tests cover implicit reopen, independent exact Back, failed and
  canceled reads, changed bytes, repository reset and refresh viewport/caret mapping.
  Wrapped Unicode restoration runs at1×/2×, both within and beyond the first4096-byte
  fragment; its source is10864 bytes. These are app frames, not native wrap evidence.
- An existing fold test now checks remembered line2/column1 through Reload. It also
  opens another fold before changing bytes, proving that stale manual folding is
  discarded while the explicitly restored caret's enclosing function stays exposed.
- Full locked workspace:769 app tests,11 opt-in ignored; two binary,13 font,22 text
  and59 doctests pass. Final focused position tests, strict workspace/all-target
  Clippy, locked build, embedded-resource smoke and format/diff checks pass.
- Native1×/2× runs use private Wayland, real Git and a controlled PR metadata fixture.
  Raw keyboard file switching restores line80/column5, verified by exact UTF-8 Copy.
  Passive wheel scrolling then changes the remembered anchor to line104/column0;
  reopening restores that row, again verified by Copy and inspected captures.
  Attached source hashes, HEAD/refs/status and executable hashes remain unchanged.
  App/compositor exits are zero. The intentionally persistent pointer helpers are
  terminated at cleanup. Early harness JSON/pointer-presence/exit assumptions are
  retained as diagnostics; no production repair was needed for those assumptions.

Architecture baseline `before-reader-position-memory`: +325 NLOC, ten functions,
no added hotspot, unchanged average/max CCN3.39/132,125 clone instances/1335 duplicate
lines, one cycle and eight structural findings. Repeated test setup uses existing
isolated_source, a shared settle-and-paint helper and one file-roundtrip helper.
The full normalized clone Counter has zero additions/removals. Raw diff-aware
reports flag unchanged old test clones and rank noise in untouched Swift Python;
those are not introduced regressions. No typography/style changes are made; the
requested SwiftUI/macOS/typography skills remain unavailable in this environment.

## Remaining goal

This closes the identified reading-position gap, not the entire parity objective.
Continue reconciling remaining Swift behavior fixtures, obtaining SDK-backed Android
success evidence, and performing the final clean standalone/resource/library reuse
audit. The app remains independent; actual sibling adoption and full accessibility
remain deferred by the user's scope.
