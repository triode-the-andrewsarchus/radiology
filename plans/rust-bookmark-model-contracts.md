# Bookmark model and storage contracts — 2026-11-08

The three Swift BookmarkTests groups now have assertion-level Rust evidence,
completing the bookmark review alongside the prior
[navigation workflow checkpoint](rust-bookmark-navigation-contracts.md).
This pass fixes selected-entry deletion and an invalid-index overflow, and adds
one shared group-ordering operation to the reusable bookmark store.

## Swift assertion mapping

| Swift function | Rust evidence |
| --- | --- |
| scopePersistenceReorderingAndDeletionKeepShortcutOrder | scoped_group_order_survives_relaunch_and_deletion creates the same twelve-line list, moves the last identity to slot zero, verifies slots zero/nine, then performs the original-index group move [0,2]→5 and verifies the two group positions. Checkout, PR 1, PR 2 and another root persist independent records; relaunch restores the full ordered list, deletion shifts slot zero, exact-location toggle removes the existing identity and the independent checkout label survives. The app deletion regression now verifies selection clearing. The prior navigation checkpoint covers actual app scope transitions. |
| unicodeLocationsRelocateByAnchorAndRejectAmbiguousChanges | The prior navigation workflow verifies an interior byte after emoji in Unicode/CRLF source, inserted-line relocation, persisted coordinates and toggle removal. relocation_uses_unique_anchor_or_bounded_context_and_rejects_invalid_columns verifies ambiguous copies, changed anchor text, revision mismatch and invalid columns. Existing bounded/cancellable tests retain exact unchanged-source and retirement checks. |
| damagedStorageAndInvalidMovesDoNotCreateBookmarks | Persisted-record recovery now covers wholly invalid JSON, wrong archive shape and truncated arrays; loads stay empty and leave bytes untouched before a valid save recovers. Move tests cover missing identities, empty index sets, mixed valid/invalid group indices, out-of-range destinations and extreme adjacent-move indices without mutation. Negative indices are unrepresentable in Rust's usize API. The app's existing empty-scope/source-readiness and keyboard empty-state checks prohibit adding before a source is accepted. |

## Runtime changes and reuse

Swift clears selectedID when its selected bookmark is removed. Rust previously
selected a neighbor, so Delete followed by Return could navigate to an entry the
user had not selected. BookmarkList::removed now clears that identity and its
owned error. It no longer needs the old index or remaining store. App toggle and
Remove share this existing path; the redundant lookup and parameters are removed.
The prior Delete→Return test now explicitly presses Down before Return when it
intends to select and open the next entry.

BookmarkStore::move_indices implements Swift's original-list insertion-boundary
semantics. It treats incoming indices as a set, preserves moved entries' original
relative order, accepts append at list length and validates the complete request
before mutation. Unsorted/duplicate inputs, backward moves and append are tested.
Existing move_to delegates to this operation while retaining its UUID/final-slot
API and no-op behavior. The standalone list continues to select/drag one identity;
this adds the grouped store contract without introducing a second selection model.

move_down now bounds-checks before adding one to an index. The old usize::MAX
call panicked in debug builds; it is now a no-op, matching other invalid moves.
Both this regression and selected deletion failed before the fixes. No dependency,
worker, persisted schema, cache, lifecycle or sibling-app tie-in is added.

## Validation and architecture

The focused bookmark suite and full workspace pass: 805 app tests/11 ignored,
two binary, 13 font, 22 text and 61 doctests. Strict Clippy, warning-denied rustdoc,
format and binary build pass. The deletion test uses actual app frames with
Home/Delete/Return at both scales and proves there is no pending source jump,
selection, source-Arc replacement or history change. Keyboard Down/Return remains
usable in the existing command-order regression. Whole workspace, strict Clippy,
warning-denied rustdoc and build logs are retained under
~/.local/state/radiology-qa/bookmark-model-20261108-f7a7879/.
The public group-move rustdoc contains a runnable example, and the generated HTML
method section was inspected. No fresh native desktop run is claimed here.

Before/after bookmark-model-20261108 architecture vectors: +115 measured NLOC,
no new module, two added functions (one group operation and one app regression).
Average/max CCN stay 3.40/132, with 36 high-complexity functions, 123 clone
instances/1323 duplicate lines, one cycle and eight structural issues. Full
normalized fragment/file-pair multisets have zero additions/removals. The raw
comparison flags unchanged touched-file clones and equal-score ranking churn in
untouched Python helpers. New group-move CCN is five; deletion drops four NLOC and
two parameters; app toggle drops four NLOC. The ordering primitive is necessary
for the original grouped-move contract and reused by the ordinary single move.

## Remaining scope

Bookmark behavior reconciliation is complete for these six Swift test functions,
with the app/native boundaries explicitly recorded across the linked checkpoints.
This does not prove all remaining Swift suites or certify pixel equivalence.
Continue repository/filter discovery and other behavior audits, maintaining the
standalone/reader-host gates after runtime changes. Successful SDK-backed Android
indexing, other ordinary suites and machine-specific probe limits remain open.
Full accessibility and actual sibling adoption remain deferred.
