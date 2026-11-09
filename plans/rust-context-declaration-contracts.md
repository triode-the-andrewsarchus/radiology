# Context identity and enclosing-declaration contracts

References: Swift `ContextModelTests.underCaretIdentityIgnoresMovementWithinOneOccurrence`
and `declarationReferencesSurviveUnderCaretChangesAndKeepTheirSelection`, compared
with Rust `a7dfad6` and this checkpoint. Together with the cross-file preview and
Unicode-caret groups, four of eighteen ContextModel groups are mapped; fourteen
remain, along with the separate reference/call-tree/palette and other audits.

## Missing UI behavior

The expanded real-fixture app test failed because the enclosing `area` declaration
was painted as Function. Swift's DeclarationContext prefers a nonzero index kind,
so that method is shown as Method. Rust already passed the exact summary identity
and SCIP kind through InspectorContent but only used the kind for explicit pins.

The declaration summary now receives that existing kind when the summary has an
indexed identity and uses the shared semantic kind formatter. Zero metadata falls
back to the parser title. Unknown numeric kinds remain visible by number. The
existing explicit pinned-name/kind branch still owns its own metadata; no new
lookup, state, index query, public field/API, worker or dependency is added.
No type sizes, fonts, geometry or Swift source changes. The required SwiftUI/macOS/
typography skills are absent from the available skill catalog and local skill
search; the existing Rust summary rendering and style are retained.

## Complete group mapping

| Swift group | Rust evidence |
| --- | --- |
| Stable under-caret occurrence identity | The shared Swift rust-analyzer fixture is opened through the real Rust app at 1×/2×. Circle resolves in total_area, with its exact shapes.rs definition preview. Arrowing through every remaining token column keeps the same captured identity, metadata, expanded state, outline Arc, enclosing declaration, reference target and immutable reference-row Arc; no metadata/preview/reference job restarts. Moving to area changes occurrence identity, preserves the declaration/rows and produces the exact untruncated lines 7–10. Returning to Circle publishes the identical cached preview and identity at metadata acceptance with no excerpt worker. |
| Declaration references and selection | In area, the exact source signature is `pub fn area(&self) -> f64` and the actual painted summary says Method. Rows contain one use and one definition. The use is src/lib.rs line 8, with exact area match emphasis; its expanded preview contains lines 6–10. Moving through r/self/radius/PI/whitespace yields multiple caret identities but retains the exact area declaration, target, immutable row Arc, selected use and disclosure excerpt without a reference query. Entering new clears the selection and changes target. Returning to area restores the same cached rows immediately, without a scan, and keeps selection cleared. |

The backend cache checkpoint previously established the required reuse mechanisms;
this completes the first two groups' assertion mapping rather than treating cache
presence alone as proof. The expanded app test replaces partial checks in place.
A shared assertion for declaration/target/row ownership removes actual duplicate
logic while preserving independent occurrence-identity checks.

## Verification and architecture

Archive: `~/.local/state/radiology-qa/context-declaration-20261109-a7dfad6/`.
The new painted Method assertion fails on the previous runtime (`red.log`). Both
app checks and all four declaration-summary checks then pass. The summary checks
cover both scales/themes, Method, zero-kind Function fallback, unknown-kind labels
and explicit pins retaining their independent kinds. Existing signature-copy,
keyboard disclosure and exact-link checks remain green. Strict all-target Clippy,
normal build and warning-free docs pass. The full locked/offline workspace run
passes 886 app tests (eleven existing opt-in ignores), two binary/13 font/22 text
tests and all 66 doctests. The final test-only consolidation/strengthening is
verified by focused app checks and strict Clippy afterward; production code is
identical to the full-run snapshot. Full-run compilation uses -j 1 with serial
tests; ordinary half-core Cargo jobs=16 remains unchanged.

Architecture adds 99 measured NLOC/two functions/no files. Average/max CCN remain
3.42/132, with 36 hotspots and dependency cycles unchanged. Full normalized clone
multisets are identical: 124 instances, zero additions/removals after sharing the
actual new duplicate assertion. Raw flags refer to an unchanged existing inspector
clone and equal-ranked untouched Python chokepoints.

Remaining work: fourteen ContextModel groups, reference/call-tree/palette,
source/syntax, compiler/indexing and inspector reconciliation, then final native/
standalone acceptance. Known parallel timing/linking limits and Android remain
open. Full accessibility and actual little-screen-clip adoption remain deferred.
