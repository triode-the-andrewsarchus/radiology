# Exact-byte change-block navigation — 2026-11-05

## Gap and repair

Swift `ChangeBlockNavigation` computes targets against exact source bytes. Empty
hunk sides are points; they do not create changed-line highlights. Targets beyond
the line count clamp to EOF, duplicate EOF targets collapse, and EOF belongs to an
unterminated changed last line while a trailing empty line remains outside it.
Inside a block, Previous/Next skip that block; gaps choose the nearest block without
wrapping. Binary changes have no navigation targets.

Rust's line-based calculation could not distinguish an empty-side point from other
columns on its line. An actual-app red regression using the Swift synthetic EOF
hunk shows the source already reaches byte7 but Next remains enabled with line3 as
its target. SourceBuffer already clamps oversized lines to EOF; availability must
therefore use the resulting byte rather than the requested line.

FileLineChanges now exposes `navigation_at_byte`, reusing SourceBuffer conversion
and the existing hunk-side projection. Empty ranges remain points instead of being
expanded into a fictitious line. The bounded byte calculation deduplicates clamped
anchors and implements the EOF/inside/gap rules above. The existing public line API
remains available and documented with its one-based coordinate space.

App command dispatch and toolbar availability use the new method, as do source
header arrows. The header receives the full caret byte, so its availability agrees
with commands at different columns on one line. Navigation still commits through
the existing destination/history transaction, preserving exact Back, fold/test
reveal and destination feedback. No new worker, cache, dependency or public type.

## Evidence

Durable archive:
`~/.local/state/radiology-qa/change-block-bytes-20261105-a2d0af3`.
Temporary execution:
`~/.cache/radiology-qa-change-eof`.

- Two unit tests reconcile the Swift inside/gap/end cases, terminated versus
  unterminated EOF, both revision sides, duplicate EOF anchors, an interior empty
  point, binary changes and absence of invented empty-side changed highlights.
- An actual-app raw Alt+Down test reproduces and fixes the synthetic unterminated
  EOF case. At EOF navigation becomes unavailable, another key does not change
  history, and Back restores the original caret. This is a synthetic hunk contract
  check, not a claim that ordinary Git emits that exact EOF deletion shape.
- Native1×/2× checks use real Git middle deletions (After) and insertions (Before)
  with controlled PR metadata. From the end of the empty-side anchor line, raw
  Alt+Up reaches its beginning; exact Copy verifies the source slice. Back restores
  the end-column departure. Four exact Copy checks per scale, inspected captures,
  unchanged attached file/HEAD/ref/status and executable hashes; core processes
  exit zero. These are native interior-point checks, separate from synthetic EOF.
- Full locked workspace:774 app tests/11 opt-in ignored, two binary,13 font,22 text,
 60 doctests. Strict workspace/all-target Clippy, locked build, resource smoke,
  format/diff and rustdoc generation pass. The new public example runs and the
  generated method documentation is checked. Cargo remains limited to16 jobs.
- Architecture comparison is NEUTRAL: +139 NLOC/four functions (one production,
  three tests), new byte method CCN8, unchanged36 hotspots/max132 and average3.39.
  Full normalized clone Counter is unchanged125/1335 with zero additions/removals;
  cycle and structural counts stay one/eight. Raw chokepoint rank changes concern
  untouched Swift Python, not this implementation.

## Remaining work

Continue reconciling other Swift behavior fixtures and SDK-backed Android success,
then perform the clean standalone/resource/library-host audit. Native captures also
show the Before status footer wrapping “Read Only” at the smaller logical window;
the [subsequent metadata layout repair](rust-source-status-layout.md) closes that issue. Actual sibling adoption and full assistive
accessibility remain outside the current user scope.
