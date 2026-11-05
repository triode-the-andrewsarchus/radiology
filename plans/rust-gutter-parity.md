# Compact canonical gutter and function meters — 2026-11-05

Comparing the Swift gutter and SourceScrollView meter assertions found Rust still
numbered every row, reserved 104 points and lacked function-size meters/count caps.
A red rendered-output test demonstrates the old width; an additional red case
shows scrolling into a long wrapped continuation removed its canonical label.

The gutter is now 72 points, with numbers every fifth canonical source line plus
the caret line. Passive scrolling preserves the caret and only paints its number
when visible. The first visible fragment of each logical line owns one label,
including a partially visible wrapped continuation or horizontally scrolled line.
Hidden body rows remain unnumbered and expanded labels keep original line numbers.

Collapsed callable bodies now show the Swift logarithmic meter: a six-point base,
ten-point logarithmic span and a cap at 256 hidden rows. Normal count capsules cap
their label at `9999+`; tooltips retain the exact total. Critical annotations use
the existing reachable-count label and outlined rounded-square presentation. They
continue to replace the ordinary line count rather than sharing its meaning.

Count badges are measured against the actual sparse line-number bounds. A crowded
badge is omitted while its disclosure/meter and exact tooltip remain available.
The existing 12-point number and 10-point count fonts remain interface metadata,
independent of source zoom; a 20-point minimum row protects those controls at small
source sizes. Ordinary 14-point source rows retain their prior minimum height.

`FoldRange::hidden_line_count` derives canonical rows from original byte bounds,
not cached line metadata: delimiters survive on the header, while indentation
captures remove all captured rows. CRLF, Unicode, multiline signatures, empty
bodies and invalid ranges have explicit checks. The existing fold model now owns
this reusable calculation. Gutter paint/measurement lives in `viewport/gutter.rs`,
beside input/selection; viewport painting retains visible-row geometry and source
interaction. No second fold state, cache or dependency was introduced.

## Contract evidence

| Swift assertion group | Current Rust evidence |
| --- | --- |
| `LineNumberGutterTests.sparseNumbersFollowNativeCaretAndNeverTheScrolledViewport` | `sparse_gutter_numbers_follow_caret_and_keep_canonical_folded_rows`: width72, fifth/current labels, caret movement, offscreen caret suppression, passive-scroll identity and one label on a long Unicode continuation at 1×/2× |
| `LineNumberGutterTests.foldedRowsStayUnnumberedAndExpandedNumbersRemainCanonical` | Same rendered regression: header5/current18 survive collapse, hidden10 disappears, expanded10/18 retain canonical identities |
| `SourceScrollViewTests.collapsedFunctionMetersUseCanonicalHiddenLinesAndALogarithmicScale` | `collapsed_function_meters_scale_cap_and_keep_exact_count_targets`: sizes2/20/200/256/2000/10200, monotonic/capped painted meter widths, compact label and real count-click expansion |
| `SourceScrollViewTests.functionMeterHandlesIndentationUnicodeMultilineSignaturesAndEmptyBodies` | `hidden_counts_use_original_byte_ranges_for_delimiters_indentation_and_empty_bodies`: canonical CRLF and Python Unicode multiline-header counts; empty/invalid ranges return zero |
| Crowded badge recovery | `large_line_numbers_omit_crowded_badges_without_losing_meter_or_disclosure`: five-digit line number, omitted capsule, retained clickable meter and exact source/caret at both scales |

The meter test also exercises source font sizes8/14/32. The existing Critical-count
pointer regression continues to prove reachable counts differ from hidden lines
and activate the same fold. Existing shared-header identity, Unicode Copy and
deferred pointer/navigation tests pass. Two older tests now derive text origin
from the gutter constant and compare zoom heights after bounded remeasurement;
their canonical navigation and caret assertions remain intact.

Native real-Git PR fixtures use a multiline Rust signature, 607 body rows and a
nested closure. At both scales, the 608-row capsule and meter stay compact, clicking
the actual badge expands the body, and dark/light/zoom captures remain readable.
Collapsed, expanded and zoomed Copy each match all10012 original UTF-8 bytes;
attached files, Git HEAD/refs/status and executable hashes stay unchanged. Accepted
child exits are zero. Screenshot pixel checks separately verify badge disappearance
after expansion; native input uses measured badge coordinates.

## Validation and architectural review

The locked workspace passes 786 app tests (11 opt-in ignored), two binary tests,
13 font tests, 22 text tests and 60 doctests. Strict Clippy, rustdoc, formatting,
binary/resource smoke and the separate current-crate reader host pass. The host
checks focus, viewport/caret separation, exact Copy and stale-source rejection at
1×/2× with the reduced gutter.

Baseline `before-gutter-parity`: +289 measured NLOC, one cohesive gutter module and
eight reported functions. Average/max CCN remain3.40/132; no added hotspot, cycle or
structural issue. Gutter functions have CCN1–10; viewport row publication is15.
Full file-pair/fragment clone counters retain all125 instances/1335 lines with no
addition/removal. Raw diff warnings identify unchanged old word-navigation tests
and equal-score ranking churn in untouched Python scripts, not new duplicate logic.

Evidence is retained at `~/.local/state/radiology-qa/gutter-20261105-4ae63d9/`;
working fixtures/logs stay under `~/.cache/radiology-qa-gutter/`, outside `/tmp`.
This reconciles both gutter tests and the two meter assertion groups. Other
SourceScrollView assertions retain their existing checkpoint-specific evidence;
remaining Swift reconciliation and SDK-backed Android success are still open.
Full accessibility and actual sibling adoption remain deferred.
