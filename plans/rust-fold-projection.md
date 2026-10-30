# Canonical-byte fold projection

Swift's `SourceViewerCoordinator.hiddenRange` retains opening/closing ASCII
brace, bracket, parenthesis or quote delimiters and hides the interior, including
newlines. Non-delimited captures are hidden whole. The gutter places those
captures' disclosures on the preceding physical row. Rust now uses the same
source-byte rules rather than removing complete physical lines.

## Shared text contract

`reader-text::projected_fragments` unions overlapping/adjacent hidden intervals
before walking visible spans. Hidden newlines join surviving prefixes and suffixes,
so a function displays as `{}` and a type's collapsed header omits nested method
text. Each ProjectedFragment retains ordered canonical spans, a source envelope,
row ownership and bidirectional source-byte/display-character mapping. The bound
counts visible UTF-8 bytes, not the potentially enormous hidden source envelope.
No hidden body is copied into layout, and no GUI/runtime dependency is added to
the standalone shared package. Empty visible rows, CRLF and Unicode remain exact.
Nonempty hidden intervals must be valid UTF-8 source ranges; the API documents
that precondition and its panic behavior. Character mapping rounds an interior
UTF-8 byte back to a visible scalar boundary.

The viewport combines manual-fold and hidden-test ranges in the same projection.
Syntax styles are read for each surviving span's original source line. Shaping,
measurement, hit testing, visual movement, caret/active row, function shade,
destination feedback and selection/find/pin decorations use projected positions.
Find reads only surviving canonical spans, with duplicate cross-span matches
removed. Selection/copy still use original endpoints and original source bytes.
Navigation, outline previews and find reveal hidden exact bytes, including a
hidden suffix on a row that also contains a visible declaration header.

## Keyboard focus correction

The new pointer/arrow/copy regression exposed egui moving focus away from the
canvas during arrow navigation. GDB showed the source keyboard handler was no
longer called for Select All after ArrowLeft. The existing inspector arrow/Escape
focus filter is now named `retain_navigation_keys` and reused by the canvas at
registration and pointer focus. Tab remains available. No competing shortcut
router or synthetic clipboard behavior is added.

## Verification

Full gates pass 303 app tests, one external-Clang fixture ignored, eight shared
reader-text tests and 30 doctests; fmt, strict Clippy, build and rustdoc pass.
Projection tests cover nested union, CRLF/Unicode joins, hidden byte rejection,
visible-byte bounds across large hidden regions, empty rows, and equivalence to
unhidden source fragments. Reader tests cover joined Unicode tail hits, Home,
left/right across a hidden body, Select All/copy of exact original bytes, Python
header controls, whole-body hiding and exact reveal. Existing folds, Hide Tests,
source navigation, PR, destination and source-find tests remain green.

GDB before/after evidence is `/tmp/radiology-fold-select-gdb.log` and
`/tmp/radiology-fold-select-fixed-gdb.log`. The fixed run reaches the Select All
branch after arrows, passes the exact-copy assertion and exits normally. The DAP
adapter is unavailable; the installed GDB launched its own test process.

Architecture against `before-fold-projection` adds 336 NLOC, one shared source
file and 21 functions, keeps average/max complexity unchanged, lowers duplication
by 0.01 percentage points and keeps 91 clone groups. No new hotspot or dependency
cycle. REGRESSION classification refers to existing test fragments in touched
files: source-view/context-inspector filters and a modal/reference-list fixture.
All three pairs are present in the baseline. The projection mechanism is necessary
for noncontiguous canonical rows; existing host layout, focus filtering and source
ownership mechanisms are reused.

Broader wrapping/scaling/accessibility and cross-language visual acceptance, PR
refresh mapping, caches/background/index lifecycle and sibling palette/chrome
reuse remain open. These tests do not establish full Rust/Swift parity.


Native artifacts are `/tmp/radiology-fold-projection-native/`: `rust-folded.png`
shows separate method bodies as `{}` with the Unicode tail retained;
`outer-collapsed.png` shows only `impl Demo {}`. `unicode-tail-end.png` records
Home/End with the caret on original source line 3 (and the real tail pin). The
Python collapsed/expanded pair shows a disclosure on `def first():`, with no
surviving first/last statement characters when closed. Kotlin expanded/collapsed/
reexpanded captures show the control on preceding row 3, all import text hidden,
and trailing comment preserved. Original newlines outside non-delimited captures
remain visible, including their empty row. No newline normalization is performed.

`build-evidence.json` hashes all eight captures and final binary
`7efa6e4a5e2782bcef24b59d2251112160c1f1c489d9a63028b9617906835cab`, based on `9f8649c` plus the shared primitive
and host integration. Native app PID 4145721 and private Niri/Weston exited zero;
persistent pointer helpers were intentionally terminated. This external driver
uses the sibling compositor helper only for acceptance; product builds need no
sibling checkout. Full native scaling/theme/accessibility parity is still open.
