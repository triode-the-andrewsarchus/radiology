# Passive reading positions and structural history — 2026-11-05

Update: the four remaining app-model groups recorded below are now reconciled in
the [six-group app-model checkpoint](rust-app-model-contracts.md). Other Swift
suites and tool-validation limits remain open.

Swift AppModelTests.passiveScrollingDoesNotReplaceCommittedStructuralTrail starts
with an implicit file visit, changes its reading position, commits a structural
stop, scrolls again and navigates elsewhere. Back must restore the committed stop
and then the earlier reading position. AppModel.navigate updates an implicit entry
from positions; navigateFromCaret additionally records exact semantic departures.

The new hosted-app regression fails against 6749ef4: after passive scrolling to
row 151, committing an outline destination and opening another file, the second
Back returns byte 0 instead of byte 3143. The previous structural-history fix kept
fixed stops but still used the caret for every departure.

## Ownership and behavior

Existing NavigationIntent now explicitly marks structural reading departures.
Normal file, public navigate/jump, outline, local symbol, search/line and PR
change/revision navigation use that policy. A departing implicit visit updates
from the existing accepted ReadingPositions byte; a fixed structural stop remains
fixed without adding incidental caret movement. A history-less accepted reader
still acquires its first departure before recording a destination.

Definition/reference/trait, bookmark and Code Path navigation retain the existing
exact-caret transaction. For those routes, an implicit entry updates from the
caret, while a changed departure is added after a fixed structural anchor. Back
and Forward step through existing entries without adding another departure.

Thus structural and semantic moves preserve their respective Swift histories:

    structural: implicit reading position → committed stop → next destination
    semantic:   fixed stop → changed exact caret departure → destination

The existing private history flag is renamed history_tracks_position to describe
both reports. ReadingPositions exposes its accepted active byte through a private
query; no second position cache or location type is introduced. Publication,
failure/cancellation, identical-visit deduplication and scope teardown retain their
existing owners. No dependency, persistence or reader-host API change is needed.
The public navigate documentation now calls out its structural reading trail.

## Tests and native evidence

At 1×/2×, outline_commits_keep_implicit_reading_positions_and_fixed_structural_stops
uses 300 Unicode comment rows, real viewport top restoration and an actual
source-bound OutlineAction::Activate. Caret stays on line 1 during passive scroll.
A later passive report cannot replace the fixed stop. Opening b.rs and stepping
Back restores the outline name and then the original row 151 byte exactly.

Existing exact-departure tests now exercise bookmark navigation explicitly,
including failure/cancel/Back/Forward. The pointer-pinning regression uses the
actual Go to Definition command and verifies its exact moved-caret Back. The
pending plain-file test expects the fixed structural stop to survive while the
accepted reader remains interactive. Other app/history/reading-position tests
continue passing, including a manually history-less reader and outline previews.

Native private-compositor checks are driven interactively at both scales. A real
Git checkout fixture has uniquely numbered Unicode prefix rows. The workflow:

1. Open other.rs through Ctrl+K and save bookmark 0 with Ctrl+D.
2. Open scan.rs as an implicit file visit and scroll with the real virtual pointer.
3. Inspect the first visible numbered row and the unchanged line 1 caret footer:
   row 22 at 1× and row 29 at 2×.
4. Click area in the outline, focus its source name, move Right once, and open
   bookmark 0 with Ctrl+0.
5. Back restores the exact departure, the fixed outline name and the earlier
   reading row; Forward restores the outline name, departure and bookmarked file.

Seven exact clipboard checks per scale prove the distinct rea/area suffixes,
uniquely numbered passive row, both Forward suffixes and the complete 1553-byte
scan.rs/33-byte other.rs sources. Copied selections do not overwrite the fixed
history entries. Captures independently show the scroll/caret/outline states.
Prepared source hashes, Git HEAD/refs/status and binary hash remain unchanged;
all accepted child exits are zero. verify-native.py independently rechecks the
saved clipboard files, manifests and hashes after both apps exit.

Durable evidence, native driver/verifier, red regression and architecture counters:
~/.local/state/radiology-qa/passive-trail-20261105-6749ef4/.
Working fixtures: ~/.cache/radiology-qa-passive-trail/native-{1.0,2.0}-n1/.
The accepted manifests record observed pointer coordinates and reading rows.
This checkpoint supersedes the older Go to File exact-departure demonstration:
normal file moves now preserve the structural trail, and bookmarks prove the
separate exact-caret behavior.

Workspace 796 application tests/11 ignored, two binary, 13 font,22 text and 60
doctests pass. Strict Clippy, rustdoc warnings, formatting and standalone build
pass. No new ignored-tool acceptance or Swift runtime execution is claimed.

## Architecture and remaining work

Baseline before-passive-history-trail at 6749ef4 versus after-passive-history-trail:
+69 measured NLOC and two functions (one regression and the active-byte query),
same 666 files, average/max CCN 3.40/132, 36 high-complexity functions, 123 clone
instances/1323 repeated lines, one cycle and eight structural issues. Full clone
fragment/file multisets have zero additions/removals. record_departure is CCN 8,
finish_source_interaction 7, current_byte 1. Raw comparison only flags an unchanged
clone in the modified app test module. The origin distinction is required domain
behavior and reuses the existing intent and accepted-source transaction.

This reconciles the named passive-trail assertion and refines the previous history
checkpoint to match Swift's structural/semantic split. Remaining AppModel groups
(latest navigation, repeated pointer activation, superseded parse and language
replacement) still need their explicit evidence pass. Other Swift suites and
SDK-backed Android success remain open. Full assistive accessibility and actual
little-screen-clip adoption remain deferred.
