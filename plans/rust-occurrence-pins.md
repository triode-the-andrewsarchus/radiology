# Rust occurrence pin parity

## Session store (2026-10-28)

The store supports unlimited groups. New groups advance a palette slot; removal
never recolors survivors or evicts older pins. Colors may repeat. The active
source's groups are retained by position key (path plus checkout/Before/After)
and exact SHA-256 content identity. Returning to identical bytes restores them;
changed content discards the old pins. Ephemeral highlights never cross documents.
Repository/review reset clears the session; this is in-memory state, matching the
Swift pin store, not disk persistence.

Ranges are sorted/deduplicated, and equivalent ranges reconcile lexical groups
that later acquire semantic identity. Group identity and display spelling are
separate. Clear Current preserves other documents and color progression; Clear
resets the entire repository session.

Backend regression tests cover twelve groups without eviction, stable colors,
restoration/revision/content isolation, changed-then-reverted bytes, ephemeral
reset and lexical/semantic range reconciliation.

## Reader integration and acceptance

Successful source publication activates the pin document; failed/cancelled reads
leave the visible document's pins intact. Source labels use the clicked spelling,
while identities continue to select the group. A compact horizontal scrolling row
shows each group's color, spelling and count; clicking removes that group. Clear
Pins affects only the active document. Long labels are shortened in the row and
retain full hover details. This is a Rust reader control, not a claim that Swift
has the same legend row. Existing type scale/palette and source action routing
are reused without a sibling checkout dependency.

Application regression tests cover five groups, file departure/return, failed
reads, changed bytes and retained other-document groups. Headless pointer tests
exercise actual remove/clear widgets. Private native Weston/Niri checks verify
five highlighted groups (palette cycling), file switching/restoration, middle-group
removal without recoloring and Clear Pins removing the row/highlights. Local
captures: `/tmp/radiology-pins-native/{five-groups,restored-five-groups,removed-middle-group,cleared-groups}.png`.
App and both compositors exit zero.

Validation: 147 application tests, 5 reader-text tests, 30 doctests, fmt, Clippy
with warnings denied and binary build pass. The pre-existing real-Clang test is
ignored by default; this pin work does not change that compiler adapter. Architecture
comparison has unchanged maximum CCN 100 and no new product clone/hotspot/cycle;
it still reports REGRESSION for two GUI test input-replay clones (8 and 7 lines).

AST-aware fallback occurrence scope (comments/strings/shadowed bindings), full
occurrence decoration parity, content-bound fold restoration and exact PR-refresh
restoration remain open. The unlimited/session restoration work does not close
the full source presentation or Swift parity requirement.

## Shared document activation (2026-10-30)

Pins now share exact-document activation memory with fold choices. Identity,
revision/digest invalidation, unlimited groups, stable colors and ephemeral reset
behavior are preserved by the existing tests.
[Shared contract and verification](rust-fold-restoration.md).
