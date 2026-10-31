# Bounded Changes and repository Search rows

## Contract

Changes retains Swift's two-line presentation: status, prioritized filename and
quiet counts, followed by a directory or rename origin. Filenames and details
reuse measured middle elision; trailing counts get a bounded share of the pane.
Binary changes say `binary`. Help retains the status, complete new/old paths and
counts. Review/branch headings are limited to two rows with full-title help.

Repository Search retains one middle-elided path/line row and up to two code
preview rows. Preview text uses the existing 12-point monospace role and tail
elision. The virtual list's fixed stride derives from the actual metadata/code
font metrics and padding. Text painting is clipped to that row. Source data is
unchanged: clicking returns the original SearchMatch, including its exact byte,
line/column and complete preview.

The shared text-fit module now shapes a UTF-8-safe prefix of at most 4 KiB into
bounded tail-elided rows. This bounds preview work even for long ripgrep lines;
it does not truncate the stored match or loaded source. Existing font/appearance
roles are reused, with no parser, source renderer or dependency added.

Rows use explicit egui interaction IDs bound to their path or exact match.
Pointer activation also checks the current interaction rectangle: egui can have
hit-tested the previous frame before a result moves. A changed/reordered list
cannot transfer a pending press to another item or activate an item that has
moved out from under the release. Normal fresh clicks and Tab/Space activation
remain available. There is no additional input queue or persistent owner state.

## Verification (2026-10-31)

Three added UI regressions cover:

- 180/260/420-point panes in both themes and 1×/2× scales, including running Search,
  a long unbroken review title, deep Unicode paths, compact virtual row geometry
  and path suffix visibility on the first line.
- Exact full navigation actions from fitted captions, binary metadata, two-row
  preview elision and the 4 KiB shaping bound. The preview budget lands inside a
  repeated multibyte emoji; stored match fields still compare equal in full.
- Press → reorder → release cancellation for both lists, followed by a successful
  fresh click and real Tab/Space activation. The fixture uses the application's
  read-only label interaction style.

Saved pre-fix regressions show Changes expanding a 180-point pane to 974.10706
points and Search result strides reaching 380 points. Another negative shows an
old Changes press activating `assets/image.dat` after row replacement. All three
regressions pass with the repair. The running-search controls already fit the
covered widths, so their behavior is retained.

Workspace passes 486 app tests (seven ignored), two binary, four font, eleven
reader-text tests and 34 doctests; strict Clippy/build/fmt/diff pass. Architecture
comparison is neutral with no new clone, hotspot or cycle. Geometry validation
is shared across the layout matrix, and result fixtures/action expectations are
reused across activation checks.

Final private native 1×/2× runs use a real local Git fixture with a nested rename,
a modified binary and a long Unicode source comment. Real ripgrep returns a
normal line and a long preview. From an independently opened peer, clicking the
Changes filename and Search result opens the full nested path. Header copy then
returns the expected complete relative path, and the Search destination is line
5. Explicit source clicks followed by selection/copy return the exact Unicode
source bytes in dark/light mode. Each of four text offers per scale follows PNG
clipboard replacement and requires its expected SHA-256.

Captures cover 260-point and narrowed panes, restoration, two-row preview elision,
rename origin and binary metadata in dark/light mode. Fixtures remain unchanged;
all application/compositor exits are zero. The initial grayscale readiness probe
sampled a green diff stripe despite a loaded source view; that failed QA trial
is excluded. The corrected probe samples the neutral line-number gutter. No
source-loading failure or performance stall is inferred from the failed probe.
Native PR-title truncation and list publication/reordering are not claimed;
those have headless UI coverage.

Archive: `~/.local/state/radiology-qa/result-panes-20261031-d3cd8f0` contains
negative/positive gates, native inputs/captures/copies, Git/fixture/binary
provenance, architecture comparison, source patch and SHA256SUMS. Sibling scripts
are used only as QA capture helpers; the product remains standalone.

## Remaining acceptance

Inspector/other long-label surfaces, broader result/reader focus and enabled
states, large-file/Back/scroll/wrap cases, broader grammar/index and PR/cache
fixtures, and the requirement-by-requirement final Swift audit remain open.
This increment does not certify automatic source-focus return from result rows
or the working-tree rename hunk mapping. Full assistive accessibility and actual
sibling adoption remain deferred by the user.
