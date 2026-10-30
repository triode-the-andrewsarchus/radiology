# Bounded source diff and pinned-blob reads

Implemented 2026-10-30, starting with `dd62028`. SourceLoader now passes its actual
LatestRead cancellation flag into checkout diffs and review blob reads. These
operations reuse GitRead and the existing process-group runner; there is no
additional executor, output-reader thread or temporary stdout file.

## Request and publication contract

One cooperative 30-second deadline covers source read, analysis, colors and diff
preparation once the worker starts. Checks run between stages and during line
highlighting. Each Git subprocess gets the remaining budget; cancellation,
timeout and output failures kill its process group and reap the child. [Native analysis](rust-native-analysis-cancellation.md) now uses progress callbacks.
Sorting, bounded helper operations and individual filesystem syscalls still finish
their current operation before checking the deadline.

PreparedSource remains one atomic bundle. A failed diff/blob read reports a
recoverable, path-labeled source error and retains the accepted reader/history.
Replacing/cancelling a request retires its publication and signals its subprocess.
Recoloring still reuses immutable bytes/captures and does not read Git or disk.
SourceRead::read and ReviewSnapshot::read retain their synchronous public APIs,
with a bounded default deadline; callers with a job flag use the internal
cancellable path. [Semantic source consumers](rust-analysis-source-cancellation.md)
now propagate the same request flag/deadline through previews, references, caller
traces and Critical Lens, including cache-safety and live-child checks. Broader PR
preparation/fetch cancellation remains a separate task.

## Shared checkout hunks

File opens and checkout refresh use the same exact-source hunk preparation.
It validates current disk content before/after the patch, rejects a moving HEAD,
and uses a captured commit as the diff base. The child environment forces literal
Git pathspecs, so brackets/wildcards in actual filenames cannot select other files.
External diff and text conversion are disabled. Complete stdout is bounded at
16 MiB per command; overflow is an error, never a truncated parsed patch.

A file absent from HEAD, or untracked in the current index, gets full-file added
hunks. This includes `git rm --cached` followed by leaving the file on disk: HEAD
still contains that path, but its displayed bytes are untracked and must not get
the index deletion's hunks. Plain folders without Git do not get added-file washes.
Changed/missing disk bytes suppress hunks for a stale captured source. Existing
HEAD verification also guards the broader checkout refresh snapshot; sequential
commands do not promise an atomic filesystem snapshot.

## Pinned source

Review reads operate only in the private object store at the pinned Before/After
commit. `cat-file -s` validates the advertised size, and bounded `cat-file blob`
returns raw source bytes. The actual count must match metadata before UTF-8,
binary and file-size validation constructs a SourceBuffer. The same cancellation
flag and remaining deadline cover both subprocesses. Attached checkout contents
and index state do not supply pinned source bytes.

## Verification

Full workspace tests pass 339 application tests, one external-Clang fixture
ignored, eight reader-text tests and 30 doctests. Strict Clippy, formatting, build
and rustdoc pass. The added index-deletion case passes again after its table-driven
fixture consolidation. Architecture comparison with `before-source-process-cancellation`
is neutral: no new hotspot, clone group or dependency cycle.

New fixtures exercise literal bracket/Unicode paths against a colliding glob
filename, untracked and index-deleted source on first open, moving HEAD, changed
disk bytes, interrupted/oversized patches, retired/expired reads, pinned metadata
mismatch/oversize, bounded raw blob output and running blob deadlines. Existing
review fixtures continue proving pinned bytes differ from dirty checkout bytes.

Native acceptance uses private Weston/Niri, a real Git fixture and a child-only
Git wrapper that holds/fails the b.rs diff. With a.rs accepted, opening b.rs shows
the existing progress/Cancel row and retains a.rs. The held child is confirmed
live before Cancel, then confirmed absent after it. Forced failure reports
`b.rs: Git diff failed` with the old reader intact; selecting b.rs again succeeds.
An earlier run additionally returns to the already displayed a.rs while b.rs is
pending and verifies that replacement kills/reaps the held child immediately.
The sibling capture helper is used only for QA; the app remains independent.

Final corrected build: app PID 260196, held child PID 260723, all app/Weston/Niri
exits zero. Artifacts live in `/tmp/radiology-source-process-final-native`:
`settled.png`, `pending-diff.png`, `cancelled.png`, `failed-diff.png`, `retry-source.png`,
`cancellation.json` and `manifest.json`. The driver is
`/tmp/radiology-source-process-final-driver.py`. Tested binary SHA-256:
`4e997d2e57fdc5f2c01db232a799ac38213747346cbb5034d3bd04b0b918d9b0`.
The manifest records hashes for every screenshot, cancellation evidence and driver.

Earlier `dd62028` run: `/tmp/radiology-source-process-native/manifest.json`, app
PID 252558, cancelled child 253030 and replaced child 253687, all reaped/exit checks
successful. Its binary SHA-256 is
`b4136e2dfb7d87a3ef0fee4a8f78af2161639412cf40b12b569b5985f415f588`.
The index-deletion refinement was followed by the corrected-build native run above.
