# Shared source viewport primitives

`crates/reader-text` is a focused, GUI-independent library integrated into
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
anchors. Variable-height geometry, bounded fragments, and byte selection are now wired
into `SourceViewport`; local-find rendering uses the same shaped fragment geometry.
The broader requirement ledger still governs full acceptance.

Tests cover rapid worker replacement, Unicode/CRLF byte navigation, same-path
content replacement, closing/reopening on another document, and repeated Return
through the real shared shortcut and TextEdit focus paths. Native smoke checks
showed two highlighted matches, next-match movement, and an invalid-regex error
using isolated settings; screenshots are under `/tmp/radiology-native-find*.png`.

## Measured, selectable viewport (2026-10-27)

`SourceViewContent` groups borrowed frame data. `SourceViewport` owns focus,
selection, source identity, visible fragments, geometry, and scroll state. The
source chrome delegates layout/input/rendering rather than holding a fixed-row
painter monolith. It uses `reader-text`'s bounded 4 KiB fragments, row sums, and
original-byte selection directly. Wrapping measures at most 64 offscreen rows or
2 ms per frame; visible rows and navigation targets are measured on demand.
Scroll anchors follow the source row as estimates change. Highlight spans carry
line-relative byte ranges so fragment rendering binary-searches its first token.

Pointer hit testing, Shift/drag selection, visual Up/Down movement, horizontal and
Home/End/Page navigation, source caret strokes, selection/find/pin highlights, and
copy all use original bytes and shaped galleys. Horizontal movement skips CRLF
terminators and folded rows without placing the caret inside line endings.
Navigation targets account for the exact caret column and wrapped continuation.
Find adopts a newly revealed match as selection once; subsequent caret movement
is independent. Source input is excluded when another control or modal owns it.

Source zoom is independent of interface typography: default 14 points, bounded
8–32, persisted in settings. Menus and Ctrl++/Ctrl+-/Ctrl+Alt+= use shared reader
commands. Metadata keeps its own compact scale. Native wrap/zoom evidence is
`/tmp/radiology-native-viewport-wrap-settled.png`; clipboard comparison verified
all 1,131 original Unicode/CRLF bytes. Headless tests cover wrapped hit testing,
drag-copy across CRLF, folded copy, focus exclusion, long-line navigation and zoom.

Still open: persistent per-document fold/pin states, fully syntax-aware fallback
occurrences and unlimited pin groups/legend, enclosing-function body shade,
destination feedback, broader scaling/light/dark acceptance, background source
analysis, and host palette/font extraction. The fallback pin path currently uses
whole lexical identifiers within a parsed enclosing function; comments/strings
and shadowed bindings still need AST-aware occurrence resolution.
