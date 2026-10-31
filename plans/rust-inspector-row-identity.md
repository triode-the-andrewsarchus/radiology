# Inspector row activation identity

## Contract — 2026-10-31

Outline and bookmark pointer presses belong to the row originally pressed.
Reordering bookmarks, replacing a bookmark, or publishing different source between
press and release cannot open a replacement target. A fresh click on the new row
still navigates. This follows Swift's identity-bound `OutlineRow` actions and
bookmark identities; keyboard preview/commit and scoped persistence keep their
existing semantics.

Outline interaction IDs now include the same exact path/revision, content digest
and name range already used by `OutlineTarget`. Bookmark interaction IDs include
the persisted UUID. Both use the existing `commands::row_activated` live-rectangle
check, also used by References, Changes and Search. This matters when egui's hit
test describes the preceding frame but a stable row has moved. No navigation,
bookmark-storage or parser abstraction was added.

## Verification

New tests reproduce navigation to replacement content with the previous
renderers. They cover bookmark reorder/replacement, outline content/path/revision
replacement at the same selection offset, rejection of the old press and fresh
pointer activation. A third test drags the original bookmark identity below the
second row and verifies a Move action with no Open action. The fixture uses the
application's non-selectable-label style; default selectable egui labels otherwise
own their own pointer gestures. GDB confirmed this distinction during diagnosis.

Workspace passes 507 application tests (seven ignored), two binary, four font,
eleven reader-text and 34 doctests. After sharing the bookmark test fixture,
all four bookmark tests and six outline tests pass again, as does strict Clippy.
All-target check, build, formatting and diff whitespace checks pass.

Architecture baseline `before-inspector-row-identity` is at `c91800b`. The final
comparison reports REGRESSION for 90 → 91 clone groups and 0.65% → 0.66%
duplication. Fragment inspection identifies the added group as seven lines of
standard egui hover allocation followed by explicit-ID interaction, shared with
Search. This is the required API sequence for preserving layout while binding an
existing row identity; it adds no duplicated application policy. A separate
repeated test fixture was consolidated. Average CCN remains 3.35, maximum 100,
high-complexity functions 24 and dependency cycles zero. NLOC increases by 135,
primarily regression tests; no dependency or public API change.

Final native 1×/2× runs create two bookmarks, click each row and verify its exact
source line (473-byte Unicode call and 13-byte `target` call), drag the first below
the second and inspect persisted order `[2, 5]`. Narrow dark/light and Follow Caret
captures remain bounded. Fixture hashes are unchanged and all final app/compositor
exits are zero. Automated replacement tests establish between-frame retirement;
the native runs establish real pointer activation and saved drag order.

Evidence archive: `~/.local/state/radiology-qa/inspector-rows-20261031-c91800b`.
The first native trial reached both correct bookmark destinations and saved the
reorder but its harness searched the wrong store filename pattern. That trial
was stopped and its private app/compositors terminated; final acceptance uses
fresh fixtures and the actual bookmarks directory.

## Remaining scope

Broader mixed-owner and menu/source event ordering, mixed-language/pinned-review
visuals and parser explanations, large-file/reader performance, grammar/index/PR/
cache corpus checks and the final Swift/reuse audit remain open. Full assistive
accessibility and actual sibling adoption remain deferred; keyboard shortcuts,
focus and navigation remain required.
