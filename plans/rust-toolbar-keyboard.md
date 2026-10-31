# Toolbar menu keyboard ownership

## Behavior and boundary

Repository, PR, View and Search now use the same owned menu component as the
reader context menus and source breadcrumb overflow. The command lists remain
live borrowed presentation; they return existing ReaderCommands to the host.
There is no second keyboard resolver, parser, renderer or new dependency.
PullRequestMenu::show remains usable independently; the full toolbar adds its PR
anchor to the same group as the other three menus.

Pointer press opens the popup. Press/release and keys in a coalesced opening
batch remain owned by that popup; release cannot reopen a completed action.
A recent, registered toolbar anchor reserves opening keys before egui handles
focus. Keyboard focus on a toolbar button supports Down/Up/Enter/Space opening,
with first/last enabled selection. Left/right switch between top-level menus;
Right enters a selected submenu and Left returns from it. Hovering another
anchor replaces the active popup. Home/End, arrows, Tab, activation, disabled
rows and Escape reuse existing menu behavior.

All current anchors are registered before menu dispatch. A handoff to an earlier
anchor preserves the remaining key queue and refreshes its frame ownership, so
it can complete on the next paint. The regression that found this expired queue
is retained in the full-app suite. A newly opened anchor paints first, preventing
the previous popup remaining visible behind it. Closing requests a repaint.
Changed menu scopes retire old choices; actions are rebuilt from current data.

Activation/dismissal restores the previous live focus. Pointer-opened menus return
to the reader; a keyboard-opened menu returns to its toolbar button. Direct
Sidebar/Inspector/theme/history/mode controls preserve their prior focus before
the host dispatches the command, allowing commands that open an input or sheet
to take their own focus afterward. No asynchronous source publication requests
focus.

Shared popup rows support checked/disabled states, tooltips and non-action status
labels. The viewport-bounded popup scrolls the keyboard-selected row into view;
scrollbars are not extra keyboard stops. PR choices retain current/missing review
metadata, Local Checkout, recent states, independent refresh/cancel commands,
retained-list/error messages and the 400-character error bound. Long titles wrap.
The current checkout/review is checked and skipped during action traversal.

## Verification (2026-10-31)

Five new menu-bar regressions cover cross-menu/submenu routing,
coalesced opening/release, toggle/Escape/scope retirement, four keyboard opening
keys and hover replacement. Two real-app regressions cover View → wrap → source
copy and View → left to PR dialog → Cancel → source. The existing PR tests now
exercise the actual popup instead of a separate contents renderer, including
25-row scrolling, disabled choices, loading/retention/errors and tooltip lifetime.
An additional keyboard → wheel → keyboard regression waits 120 frames for the
smooth-scroll tail: it fails before the one-shot reveal repair, then passes.
Repeating reveal every frame had pulled the viewport back after wheel input
expired. A 16-frame exploratory check was too early and is not negative evidence.
Row reveal now runs only for owned keyboard input, preserving manual scrolling.
The shared raw-menu test host is reused by three component fixtures.

Restoring the prior toolbar implementation makes the new View keyboard regression
fail at the wrap-setting assertion; the repair passes. Workspace passes 480 app
tests (seven ignored), two binary tests, four font tests, eleven text tests and
34 doctests. Strict Clippy, workspace build, fmt and diff checks pass.

Architecture comparison is NEUTRAL: no new clone, hotspot or cycle. The full
clone multiset audit retains all 92 groups with zero additions/removals. Toolbar
show CCN falls 34 → 22; PR presentation is expressed as shared menu data, and its
standalone show is CCN 1. Toolbar-specific routing lives in the primary-anchor
adapter; the general key dispatcher remains below the hotspot threshold. Test
setup/gesture assertions reuse the existing GUI test boundaries.

Final private native 1×/2× runs verify, without an extra source click:

- Go To Line 5, View → Home/Down/Enter toggles Wrap, then exact source-line copy.
- View → Right to Search → Go To Line 6 opens the existing modal.
- View → Left to PR → Open Pull Request → Escape returns to exact source copy.
- Repository → Copy Repository Path returns the exact canonical private path;
  End/Right opens the recent-repository submenu, Left/Escape dismisses it.
- Direct Inspector toggling preserves source selection/copy; final light-mode
  captures retain the reader and inspector layout.

Each of four copies per scale follows PNG clipboard replacement and requires the
expected SHA-256. Dark View, Search, PR dialog and repository submenu captures,
plus final light captures, were inspected. Fixture hashes are unchanged and all
application/compositor processes exit zero. Native tests use a private gh shim;
no account or remote review action is invoked. The 25-choice scrolling, wheel/keyboard transition and hover
replacement checks are headless UI coverage, not claimed as native acceptance.

Archive: `~/.local/state/radiology-qa/toolbar-keyboard-20261031-c421d03` contains
native inputs/output/captures, exact copies, fixture/binary provenance, negative/
positive tests, architecture reports, source patch and SHA256SUMS. The initial
mapping-only 1× trial is excluded. Capture helpers borrow the sibling's QA scripts
only; the standalone product has no sibling checkout/runtime dependency.

## Remaining acceptance

Active menu-title styling still needs polish: the native Search handoff retains
the original View button focus ring while Search owns the popup.
Broader rapid transitions among search/inspector/reader owners and source-bound
menu enabled states still need the final audit. Other long-label surfaces,
large-file/Back/scroll/wrap acceptance, broader grammar/index and PR/cache cases,
and the requirement-by-requirement Swift parity audit remain required. Full
assistive accessibility and actual sibling adoption remain deferred by the user.
