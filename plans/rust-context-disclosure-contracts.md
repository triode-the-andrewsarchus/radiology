# Context disclosure and explicit reference focus

Reference: Swift ContextModelTests.expansionBelongsToIdentityAndStartsCollapsed
and explicitReferencesOverrideTheDeclarationUntilFollowCaret, plus declaration
identity's indexRevision ownership. Rust comparison begins at `127cdc4`.
Sixteen of eighteen ContextModel groups now have mappings; excerpt windowing and
availability still await their final reconciliation.

## Reproduced UI defect

The real app test opens area's signature with its actual chevron, replaces the
index with an equal immutable clone and paints another frame. The signature stays
open, even though definition and reference state retire. Swift's declaration
identity includes the index revision; Rust's renderer key contains only source
identity and selection range. The new assertion fails at that replacement
boundary (`red.log`) after the preceding disclosure assertions pass.

The app now resets its existing DeclarationSummary whenever semantic query inputs
change. The same existing source/index ownership check already drives metadata
and reference retirement. There is no second revision counter, renderer field,
public input parameter, new command or helper protocol. Renderer documentation
now tells independent hosts to reset on index replacement as well as repository
replacement. Same-input caret movement never reaches the reset; independent
same-declaration disclosure remains intact.

## Assertion mapping

The scenario extends the current real Swift fixture app matrix at 1×/2×. It uses
actual summary text/clip geometry to click the 16-point chevron, rather than
exposing or mutating the renderer's private expanded flag.

- At radius, declaration/definition/reference disclosures start closed. Opening
  the signature, definition details and one real call row leaves invalid rows
  ignored. Moving within radius preserves all three; moving to self collapses
  only definition details. Moving into new collapses declaration/reference state
  and the new token's details. Reference expansion reverses on false and can open
  again. Reset plus reopening leaves everything collapsed.
- A separate index replacement at the same source/declaration now closes the
  signature, completing the discovered identity gap beyond the basic Swift test.
- Actual Find References pins Circle from lib.rs. Moving into area's body in
  shapes.rs updates the enclosing declaration while preserving the exact explicit
  target and immutable Circle row Arc. Follow Caret through the inspector action
  releases it and restores area's one use/one definition. Reader Arc and history
  do not change during that action.

The pointer test derives coordinates from the painted summary row, and existing
keyboard-disclosure/signature-copy/link tests remain part of focused verification.
The UI keeps its previous fonts, weights, dimensions and controls. No Swift source
or typography scale changes; the unavailable design-skill catalog remains as
recorded in earlier checkpoints.

## Verification and structure

Archive: `~/.local/state/radiology-qa/context-disclosure-20261109-127cdc4/`.
Both app checks and five declaration-summary checks pass after the fix. Strict
all-target Clippy, ordinary build and warning-free docs pass. A fresh full workspace
run is deliberately deferred to the following excerpt checkpoint, where it will
verify both production changes together; no fresh full-run pass is claimed here.

Measured change: +150 NLOC/three test functions/one scenario file, one production
reset statement and an ownership documentation update. Average/max CCN 3.42/132,
36 high-complexity sites and dependency cycles are unchanged. Complete normalized
clone multisets are compared by content and multiplicity; raw touched-file flags
refer to the existing app/tests.rs clone, not the new scenario.

Next: cropped source markers/limits and availability, then remaining reference/
call-tree/palette and other suite reconciliation plus native/standalone acceptance.
Known Android/license and parallel-run limitations still apply. Full accessibility
and actual little-screen-clip adoption are outside the current agreed scope.
