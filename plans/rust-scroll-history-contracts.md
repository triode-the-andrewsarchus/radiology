# Scrolling, folding and history contracts — 2026-11-05

Compared the individual assertions in NavigationHistoryTests and SourceScrollViewTests
against the current Rust implementation. The existing runtime supports these reader
contracts; this checkpoint strengthens evidence without adding runtime code or APIs.
The Swift AppKit/TextKit implementation is not executed on Linux. Equivalent visible
behavior, canonical bytes and input ownership are checked through Rust's actual
app, projected viewport and source preparation paths.

## New and expanded evidence

NavigationHistory's existing test now updates the first location to byte 100 before
leaving it, checks identical visits do not add entries, restores that updated
location, follows Back/Forward, truncates a forward branch with a new visit, and
checks Back at the beginning returns None. This covers the single Swift history
contract, including updateCurrent rather than only ordinary visits.

The app reading-position regression now uses 300 Unicode source rows and restores
row 151 three times at 1×/2×. It checks the top byte does not drift on returns, Back
restores the exact departure caret independently of the passive viewport anchor,
and restoring top zero does not move the semantic caret.

A new rendered regression uses the Swift multiline Rust signature, eight prefix
rows,607 body rows including an inline nested closure, and sibling declarations.
At1×/2×, parsed default folding leaves fewer than 25 projected rows and a document
height below 28×25, with visible braces and no hidden body text. Expanding makes
the document more than 20 times taller. Changing 18pt to 13pt and recollapsing remains
compact. Keyboard Select All/Copy returns the complete original source.

The Python disclosure regression now uses the Swift Unicode list/return fixture
at both scales. The whole indentation-delimited body disappears and exact reveal
restores both lines; full Copy remains byte-for-byte original.

The existing occurrence paint test now starts with an actual nonempty Unicode
selection after document installation. Painting transient and pinned occurrences,
folding, and clearing all decorations preserve that selection and original source;
Copy still returns café. Dark/light fill and active-stroke checks remain in place.
This checks the same independence from selection and backing text that Swift
checks through TextKit temporary attributes; Rust paints separate egui shapes.

The landing-feedback test now runs at both scales and restarts a partially faded
cue on another wrapped Unicode row. It verifies the new byte/full-strength band,
subsequent automatic clock expiry, unchanged selection/source and explicit preview
cancellation. Existing DestinationFeedback tests check first-paint timing,
repaint delays, animation-disabled expiry, revision/digest ownership and cancel.
No wall-clock sleep is needed to test the animation schedule.

## Swift assertion reconciliation

| Swift SourceScrollViewTests function | Rust evidence / platform boundary |
| --- | --- |
| realAppFoldPerformanceProbe | Opt-in benchmark requires a developer's absolute macOS app.rs path. It has no latency threshold. Not run here; the portable607-body fixture checks compaction and repeated expansion/recollapse, and prior gutter native evidence checks the same large-body workflow. |
| collapsedFunctionMetersUseCanonicalHiddenLinesAndALogarithmicScale | Existing hidden_counts and collapsed_function_meters tests plus the gutter checkpoint: exact canonical counts, monotonic/logarithmic/capped widths and9999+ labels. |
| functionMeterHandlesIndentationUnicodeMultilineSignaturesAndEmptyBodies | Existing hidden_counts tests and gutter checkpoint cover CRLF/Unicode, preceding Python header and zero-line empty bodies. |
| aFreshDocumentJumpSizesTheWholeFileBeforeRestoringPosition | Expanded implicit_file_reopens test checks300 rows, row 151, repeated exact top restoration, passive caret independence and top zero. Distant-reveal tests check a fresh destination is visible and stable while other rows measure. Rust virtualizes rather than laying out the whole file upfront; the restored destination must still remain stable. |
| structuralFoldUsesSourceBufferConversionAtTextKitBoundary | Existing folded_delimiters_and_unicode_tail test checks braces, hidden source, Unicode canonical hit/arrows, selection and full Copy in wrapped/unwrapped modes. Sparse-gutter tests check hidden canonical rows do not acquire numbers. Rust uses byte projections directly, with no TextKit conversion boundary. |
| sixHundredLineMultilineRustFunctionCompactsWithoutBlankFragments | New multiline_large_function test checks607 body rows, compact height/row count, visible braces, expansion ratio, font change and re-collapse at both scales. No TextKit null-glyph/line-break hooks are applicable. |
| indentationDelimitedFoldHidesItsWholeRange | Expanded indentation_disclosure test checks the full parsed hidden range, disappearing Unicode body lines, exact reveal and original Copy at both scales. |
| foldToDefinitionsKeepsContainersOpenAndNavigationRevealsTargets | Existing fold-query fold_to_definitions test checks nested declaration containers; app fold_to_definitions command preserves caret/history. Current-function/nested navigation tests reveal enclosing body and parent while preserving unrelated siblings; see function-contract audit. |
| defaultFunctionFoldsApplyOnceAndExplicitCollapseRestoresThem | Existing fold-choice reload/revisit and current-function tests preserve manual expansion, apply initial callable defaults once and allow explicit re-collapse/reveal. FoldStore's initialized document state owns the once-only rule. |
| defaultFoldStateDoesNotTouchTextKitBeforeNewBufferIsInstalled | SourceLoader prepares syntax/default folds with the new immutable buffer. App acceptance installs them synchronously before the next paint. Existing source-loader publication and pending-context app tests retain the accepted source/folds while another destination prepares or fails; no TextKit mutation stage exists. |
| destinationHighlightRestartsAndClearsWithoutChangingText | Expanded landing-feedback test plus DestinationFeedback hold/fade/restart/retirement tests described above. SourceViewport schedules the next repaint from the feedback frame. |
| semanticOccurrenceDecorationDoesNotMutateTextOrSelection | Expanded transient-occurrences test checks an existing nonempty selection through paint/fold/clear, original text and exact Copy; existing pin-host audit covers source ownership. |

This records acceptance of the 11 ordinary scrolling assertion groups and the single
history contract through equivalent Rust mechanisms. It does not claim execution
of the opt-in machine-specific probe or AppKit implementation. Existing checkpoint
links remain authoritative for their native acceptance boundaries:
[gutter](rust-gutter-parity.md), [reading positions](rust-reading-positions.md),
[function navigation](rust-function-contracts.md),
[pending publication](rust-pending-source-context.md), and
[reader-host/pins](rust-reader-host-audit.md).

## Validation and architecture

Workspace 793 application tests/11 ignored, two binary, 13 font,22 text and 60
doctests pass; strict workspace/all-target Clippy and formatting pass. The final
viewport subset passes 71 tests. Logs and full clone counters are retained under
~/.local/state/radiology-qa/scroll-contracts-20261105-32ca0fa/.
No new native screenshot or compiler-tool acceptance is claimed in this checkpoint.
Initial local test failures were fixture setup issues: selecting before initial
source installation and requesting preview after a flash (preview intentionally
cancels it). Corrected tests use the installed source and preview-before-flash,
matching the host lifecycle. The large braces capture first reveals its row below
the prefix so it is actually inside the viewport.

Architecture baseline before-scroll-contracts at 32ca0fa versus after-scroll-contracts:
+116 measured NLOC and two functions (one regression and one shared test Copy
setup). All production code is unchanged. Existing Reader::frame, parser,
FoldStore, feedback, history and app fixtures are reused. The repeated keyboard
Copy setup in the two strengthened fold tests is consolidated in Reader::copy_all.
Average/max CCN stay 3.40/132; 36 high-complexity functions,125 clone instances/1335
repeated lines, one cycle and eight structural issues remain. Complete clone
fragment/file multisets have zero additions/removals. The raw comparison reports
REGRESSION for an unchanged clone touching the expanded viewport test module and
an equal-score ranking swap between untouched Python chokepoints; neither is a
new source dependency or clone. The earlier two new test clones were removed by
sharing their Copy setup.

Other Swift suites still require assertion-level reconciliation. SDK-backed
Android success remains pending the earlier license/install decision. Full
assistive accessibility and actual little-screen-clip adoption remain deferred.
