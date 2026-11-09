# Test-code presentation contracts

All seven Swift TestCodePresentationTests groups are reconciled. The exact shared
Rust source (including the final newline and both astral characters) is stored in
src/syntax/test_fixtures/test-presentation.rs and used by model, real viewport and
app navigation tests. This checkpoint changes tests/fixtures only.

| Swift group | Rust evidence |
| --- | --- |
| presentationKeepsTheHeaderLineAndHidesTheRestExactly | swift_presentation_retains checks one region at 3–7, the exact indented header, hidden initial LF, exact Unicode body suffix and byte end on line 7. Helper ownership hides while production remains. Reveal removes the hidden interval. Loading identical bytes at copy.rs resets reveal; changed bytes retire the old header. Rust's app/document owners qualify path/content instead of adding a second identity field to this presentation. |
| textKitHidesTestLinesKeepsCanonicalNumbersCopyBytesAndFolds | swift_test_projection runs the real viewport at 1×/2×: exact visible original line sets for hiding, default function folds, manual production expansion, off/on preference and navigation into the helper. Copy All equals every original byte before/after. The existing hidden_test_headers test also clicks the actual painted label and verifies hidden source is not painted, then is painted after reveal. Rust projects canonical fragments instead of nulling TextKit glyphs. |
| labelHitIdentityChangesEvenWhenGeometryDoesNot | test_labels_retire replaces // a with // 🚀 at a different path. Label geometry stays identical, but an actual click dispatches only the new UTF-8 region ID. |
| nestedTestLabelCannotPaintOrHitAFoldedProductionRow | The same both-scale test folds the enclosing production function: the nested label is neither painted nor clickable at the old position. Expanding the parent restores it. |
| unionMergesOverlappingTouchingAndContainedRanges | reader-text union_merges_swift uses the exact six intervals and exact two merged results. The actual viewport combines fold/test intervals through that shared projector. |
| explicitNavigationRevealsHiddenTestCodeAndBackStaysExact | swift_test_navigation uses the exact fixture and normal app navigation: hides helper rather than production, reports the departure as Swift's updatePosition does, navigates into tests, returns Back to the exact byte without rehiding, toggles the region, reveals via the same endpoint as Go to Line and verifies preference off/on preserves source, caret and the complete history. Existing outline_explanations and hide_tests_preserves tests cover actual outline/shortcut presentation and unrelated fold retention. |
| pendingRevealDefaultOpenIdentityAndTestDocuments | swift_test_pending starts with a byte-zero hidden region, requests cross-file navigation while the old source remains displayed, then checks exact target/reveal after async publication. Plain-open identical bytes at another path hide again. Path-classified integration tests and inner-cfg test-only files remain readable. |

The reader intentionally keeps an empty EOF insertion row after a final newline.
The comparison uses nonempty canonical fragments because Swift's first-visible-
glyph enumeration omits that row. Every nonempty source row matches the exact
Swift expectations; source and trailing newline are unchanged. Header styling and
controls continue through existing header/is_hidden consumers; no new UI state,
converter, presentation cache or runtime helper was added.

## Verification

Archive: ~/.local/state/radiology-qa/test-presentation-20261109-6995b84/.
All 67 real viewport tests, all three presentation-model tests, both new app
navigation cases, all 23 reader-text tests and its four doctests pass. The new
label identity/fold cases pass independently too. Strict workspace/all-target
Clippy, formatting and diff checks pass. Latest full production verification
remains c20eba6 (901 app tests/all 66 doctests); no new whole-workspace result is
claimed for these later test-only additions.

Architecture adds 250 measured NLOC/nine functions: six test cases plus three
uncompiled declaration functions in the exact source fixture. Average/max CCN
3.42/132, 36 hotspots and one cycle; all 125 normalized clone instances unchanged,
zero additions/removals. An initial duplicate identity setup was consolidated
into a two-document matrix. Remaining raw warnings identify unchanged pre-existing
clones and equal-ranked untouched Python chokepoints.

Source/syntax reconciliation reaches 66/66. This is one section, not an overall
parity claim. Compiler lifecycle/personality assertions (35 groups), inspector/
trait assertions (15), final integrated/native/standalone/reuse acceptance and
Android SDK-backed success remain. Full accessibility and actual sibling adoption
are deferred.
