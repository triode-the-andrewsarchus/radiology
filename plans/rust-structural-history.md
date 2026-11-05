# Preserve committed structural history — 2026-11-05

Reviewing Swift AppModelTests and AppModel+Semantics exposed a history ownership
gap. Swift distinguishes an implicit file visit that follows reading position
from a committed structural anchor. Its exact-caret navigation keeps that anchor
and adds a changed departure after it. Rust previously replaced the current entry
on every caret movement and departure, erasing a prior outline/bookmark/line stop.

A new actual-app regression fails before the fix: after committing byte 13 in the
Unicode café fixture, ArrowRight changes the saved history entry to byte 14. That
is not a new navigation and must not replace the committed stop.

## Resulting behavior

The existing app/history transaction now keeps one private history_tracks_caret
flag. Successfully accepted implicit file visits follow the caret as before;
explicit destinations and Back/Forward stops remain fixed. Caret movement still
updates reading positions and semantic context. Passive scrolling remains separate
from both the semantic caret and committed navigation history.

When new navigation succeeds, record_departure updates a caret-tracking implicit
entry or visits the exact departure after a fixed anchor. NavigationHistory's
existing identical-visit deduplication avoids extra entries when the caret is
already at the anchor. The accepted destination is then visited normally:

    structural anchor → changed exact departure → destination

Back and Forward step through that trail without inserting a new departure while
stepping. This matters because a history target is selected before asynchronous
source preparation; inserting a step during publication would invalidate that
target. The departure is recorded only at successful publication, so failed or
canceled requests preserve the trail and its ownership mode. Repository/review
teardown resets the flag with the existing history reset.

No new history container, location type, persistence, public API, dependency or
reader-host mechanism is introduced. NavigationIntent already distinguishes
implicit restoration from explicit destinations, so it also selects this policy.

## Evidence

At 1×/2×, committed_structural_stops_survive_caret_motion_and_keep_exact_departures
uses real hosted reader frames and ArrowRight. It checks the retained anchor,
missing-file failure and cancellation with no history mutation, exact departure
on first Back, original anchor on second Back, both Forward destinations and
unchanged Unicode source.

The previous pinning-after-pointer-motion regression expected the overwritten
anchor. It now checks that pinning still refreshes alpha metadata and the real
caret while retaining the committed beta stop. A subsequent navigation and Back
prove the moved alpha departure was retained exactly, rather than dropping that
coordinate to preserve the older stop.

The full workspace passes 794 application tests/11 ignored, two binary, 13 font,
22 text and 60 doctests. Strict workspace/all-target Clippy, formatting and a fresh
standalone binary build pass. Existing reading-position, Return, bookmark, nested
Code Path, repository/review refresh, failure and keyboard suites remain active.

Native private-compositor tests pass at 1×/2× with the current binary and a real
Git PR fixture. Go to Line 61 commits a stop; Right moves one character; Go to File
opens other.rs. The first Back restores column 1, the second column 0. Forward
restores column 1 and then the other file. Shift+End/Copy returns distinct exact
suffixes (seven versus eight leading spaces), proving the positions independently
of the screenshots. Four clipboard checks per scale pass. Captures show the
correct source/caret/footer; attached source hashes, Git HEAD/refs/status and the
binary hash remain unchanged, and all accepted child exits are zero.

Durable logs, red regression, native captures/manifests, runner and architecture
counters are under
~/.local/state/radiology-qa/structural-history-20261105-b4ea88f/.
Native working folders are radiology-qa-pending-context/native-{1.0,2.0}-h2 under
~/.cache. The first runner attempt only failed to parse a diagnostic click message
as JSON; its children exited zero. The corrected runner accepts that existing
message format, and both accepted runs complete. No fixture source is rewritten.

## Architecture and remaining review

Baseline before-structural-history at b4ea88f versus after-structural-history:
+67 measured NLOC, one new regression, same 666 files, average/max CCN 3.40/132,
36 high-complexity functions, 125 clone instances/1335 duplicate lines, one cycle
and eight structural issues. Complete clone fragment/file multisets show zero
additions/removals. record_departure/finish_source_interaction/apply_destination
have CCN 5/7/12. The raw comparison flags existing clones in the modified app test
module and an equal-score ranking swap between untouched Python chokepoints;
neither is a new clone, dependency or hotspot.

The app-model audit remains in progress. In particular, Swift's repository-opening
race test expects an empty reader/history after discovery, while Rust currently
auto-opens README or the first source. That existing initial-selection difference
is recorded for the next review rather than counted as a reconciled assertion.
Other Swift suites and SDK-backed Android success also remain open. Full assistive
accessibility and actual little-screen-clip adoption remain deferred.
