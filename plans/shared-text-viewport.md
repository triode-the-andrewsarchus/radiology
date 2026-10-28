# Shared source viewport primitives

`crates/reader-text` is a focused, GUI-independent library being integrated into
Radiology's reader. It stays inside this repository so standalone builds do not
require a sibling checkout, workbench/daemon, media decoders, or GPU libraries.
It is also a concrete extraction boundary for eventual little-screen-clip reuse.

The variable-height geometry and fixed-record search-spool mechanisms come from
little-screen-clip's existing text preview implementation. No little-screen-clip
files have been modified; migrating that host onto the same package remains an
integration step. This is not a claim that two applications already share the
runtime implementation.

## Contracts

- Fragment boundaries always land on UTF-8 scalar boundaries. Canonical source
  bytes and line numbers survive continuation rows, wrapping, folding, and CRLF.
- `ByteSelection` stores original-source endpoints, so copy includes hidden rows
  and exact line endings even when neither endpoint is currently visible.
- `RowHeights` maintains variable heights using block Fenwick sums, with bounded
  per-update work. Width, scale, font height, or row count invalidates estimates.
- `MatchSpool` keeps complete regex/literal search results, spilling past 1 MiB
  instead of silently capping local-find matches. The host reads individual
  matches or ranges intersecting bounded visible source fragments.
- Source bytes, cancellation, focus, presentation colors, and application
  lifecycle remain host-owned. The package has no egui/eframe dependency.

Current tests verify Unicode/CRLF fragmentation, selection over original bytes,
variable row geometry across block boundaries, dense searches that spill to disk
without losing count/coordinates, and Unicode, regex-error and cancellation cases.
These tests do not yet establish the integrated source viewport's acceptance.

## Integrated local find (2026-10-27)

`SourceFind` owns background jobs over `Arc<SourceBuffer>` and retires obsolete
receivers. The GUI-only `SourceFindPanel` owns visibility, query focus, active
match, and a `(path, revision, content identity)` key. Shared reader commands
provide Ctrl+F, Ctrl+G, and Ctrl+Shift+G; Return/Shift+Return navigate while retaining
query focus. Options changes retire the old query; regex/storage failures display
in the strip. Modal sheets disable find input and defer match navigation.

The current source painter queries only visible source ranges and uses galley
coordinates to mark normal and active matches, including Unicode and zero-width
anchors. Variable-height geometry, bounded fragments, and byte selection are
still awaiting full viewport integration; local find does not establish that
wrapping, long-line layout, selection/copy, or source zoom meet acceptance yet.

Tests cover rapid worker replacement, Unicode/CRLF byte navigation, same-path
content replacement, closing/reopening on another document, and repeated Return
through the real shared shortcut and TextEdit focus paths. Native smoke checks
showed two highlighted matches, next-match movement, and an invalid-regex error
using isolated settings; screenshots are under `/tmp/radiology-native-find*.png`.
