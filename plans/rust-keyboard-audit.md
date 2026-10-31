# Rust keyboard command audit

## Scope and reference

Compared the current Swift `Sources/Radiology/UI/ReaderCommands.swift` and
`BookmarkCommands.swift` with the Rust command dispatcher, toolbar, indexing
controls and application command handlers on 2026-10-30. This audit concerns
ordinary keyboard operation. Full assistive accessibility is deferred by the user.
It does not certify every native focus transition or menu traversal state.

## Repaired gaps

Swift exposes Go to Definition as a named Navigate command and a dedicated
modified shortcut. Rust previously offered Return and F12, but neither a dedicated
modified shortcut nor a Search-menu entry. **Ctrl+Alt+J** now dispatches the existing
GoToDefinition command; **Search → Go to Definition** exposes the same action.
Return and F12 remain available. Text entry keeps Ctrl+Alt+J, and modal dialogs
keep their input exclusive.

Swift also exposes Reindex through a shortcut. Rust now uses **Ctrl+Shift+I** to
force generation without project reuse. Ctrl+Alt+I remains the Context inspector
toggle. Swift's Command+Control+I and Command+Option+I cannot both map to Linux
Ctrl+Alt+I. The existing Reindex button includes the shortcut in its hover help.
Generation commands now ignore repeated requests while a generation/cache load
or tool installation is active and ignore requests without an available repository.
The explicit Cancel action remains the way to stop running generation.

These changes reuse ReaderCommand and the existing application handlers; there is
no second navigation/index pipeline, new dependency or sibling runtime coupling.

## Current Linux mapping

| Workflow | Binding |
| --- | --- |
| Open repository / file / symbol / line / PR | Ctrl+O / K / T / L / P |
| Back / Forward | Ctrl+[ / Ctrl+] |
| Find / next / previous | Ctrl+F / Ctrl+G / Ctrl+Shift+G |
| Repository search | Ctrl+Shift+F |
| Go to definition | Ctrl+Alt+J, F12, or Return in source |
| Pin references | Ctrl+Alt+R |
| Reindex every project | Ctrl+Shift+I |
| Refresh repository or current PR | Ctrl+R |
| Previous / next change | Alt+Up / Alt+Down |
| Toggle Before/After | Ctrl+Alt+B |
| Sidebar / inspector | Ctrl+Alt+S / Ctrl+Alt+I |
| Normal / Critical Lens | Ctrl+Alt+1 / Ctrl+Alt+2 |
| Source Files Only / Hide Tests | Ctrl+Alt+O / Ctrl+Alt+H |
| Wrap lines / appearance | Ctrl+Alt+W / Ctrl+Alt+T |
| Toggle bookmark / open numbered bookmark | Ctrl+D / Ctrl+digit |
| Fold to Definitions / Collapse Functions | Ctrl+Alt+0 / Ctrl+Shift+0 |
| Larger / smaller / reset source text | Ctrl++ / Ctrl+- / Ctrl+Alt+= |

The table records the actual dispatcher mappings, not a claim that every workflow
has received a new native acceptance pass. Existing workflow-specific evidence is
linked from the [implementation queue](rust-parity-implementation.md).

## Regression evidence

- An actual egui key event through `RadiologyApp::handle_shortcuts` navigates an
  indexed call in b.rs to a.rs line 1, column 3. Back restores b.rs line 2, column 5.
  Removing the new definition binding fails because the selected file stays b.rs.
- Exact modifier, text-entry suppression, modal exclusion and inspector/reindex
  collision checks cover the new bindings.
- A pointer-driven egui Search popup test clicks the rendered definition entry
  and receives the shared GoToDefinition command.
- A controlled executable holds the first indexing process while a second actual
  Reindex key event is dispatched. After release, exactly one invocation and one
  indexed document publish. Removing the running-job guard produces two invocations.
- Unavailable-repository generation/update commands start no work and report no
  spurious cache error.
- Workspace tests pass: 442 application tests (seven ignored), four font tests,
  eight text tests and 33 doctests. Strict workspace Clippy, build, fmt and diff
  whitespace checks pass. Cargo retains 16 build jobs and unchanged dependencies.
- Architecture baseline `before-navigation-shortcuts`: no new clone groups,
  dependency cycles or maximum-CCN increase. The comparison's four app-test clone
  warnings exactly match retained baseline fragments (152/1702, 247/1273,
  982/1074 and 1767/1805). They are not introduced by this change.

## Native acceptance and same-frame modal repair

A private 1280×900 Wayland run of pushed `cd4c119` verifies the dedicated
Ctrl+Alt+J binding and Search-menu action against a controlled two-document SCIP
import. Both navigate b.rs's alpha call to a.rs's definition. Back restores b.rs
line 2, column 5: Shift+End/Copy returns exactly `lpha();`. This is explicit
unverified-import fixture coverage, not an additional real-indexer claim.

Two rapid Ctrl+Shift+I presses invoke the controlled indexer once and publish two
documents from one project. Ctrl+Alt+I hides/restores the inspector independently.
With Go to Line already shown, Ctrl+Alt+J does not navigate behind it. Source,
manifest and imported index hashes stay unchanged; app, Niri and Weston exit zero.
Captures, driver, fixture hashes, binary hash, invocation record, copied text and
exit statuses are retained with SHA256SUMS at:
`~/.local/state/radiology-qa/keyboard-20261030-cd4c119`.
The driver reuses sibling QA compositor helpers; application builds and runtime
remain standalone. The screenshots are 1× dark appearance evidence.

Inspection during this pass found a separate batching defect: take_shortcuts
resolves the whole frame before commands run. If Ctrl+L, Ctrl+Alt+J and Ctrl+T
arrive together, the first opens Go to Line, but the next navigates behind it and
the last replaces it with Go to Symbol. The application now rechecks modal state
before each command and stops the batch when a modal opens. Already-consumed
reader commands are discarded; later ordinary text events remain in egui's input
stream. No new command resolver or queue is introduced.

The actual-frame regression fails before the guard and passes after: Go to Line
remains active, source loading stays idle and b.rs line 2/column 5 is unchanged.
This deterministic regression covers same-frame ownership; the native captures
above cover separate-frame ownership. The repaired workspace passes 443 app tests
(seven ignored), four font tests, eight text tests, 33 doctests, strict Clippy,
build, fmt and diff checks. Architecture comparison against
`before-modal-batch-guard` adds no clone group/cycle/max-CCN increase; the same four
flagged baseline fragments remain unchanged.

## Still open

[Context-menu traversal and input ownership](rust-context-menu-keyboard.md) now
cover all four reader owners, nested actions and opening-frame batching. Toolbar
menu traversal and broader focus acceptance remain in the final keyboard pass. Existing context-menu Escape evidence is recorded
[separately](rust-context-menu-dismissal.md). Broader indexing/grammar corpora,
reader wrapping/scroll/scaling, PR/cache recovery and the final Swift requirement
audit remain required. Full accessibility and actual little-screen-clip adoption
remain deferred.


## Search-menu command availability — 2026-10-31

Swift ReaderCommands disables file/symbol/repository search without a repository,
line/find commands without a source, and definition/references without an eligible
caret target. At 7d4eb14 Rust's Search popup enables every item; Ctrl+K/T/L can
open empty dialogs, and Ctrl+Shift+F switches to a repository search with no root.

Search items now receive current availability from RadiologyApp::command_enabled;
the same predicate guards execute_command for pointer/menu/global-shortcut paths.
Repository commands reuse repository_interaction_ready. Local commands reuse
source_location_ready, so a pending source publication cannot open a dialog on
outgoing bytes. Semantic commands reuse the existing cached definition_identity
and source/index Arcs, verify the current caret remains in the cached range, and
require a canonical definition for Go to Definition. Drawing introduces no new
occurrence scan, parser, cache, worker or dependency. Dispatch refreshes the
existing target before checking, preserving a preceding caret move's destination.
Redundant local-find guards now live in the shared predicate. Other command groups
retain their existing handlers and availability; this is not a claim of complete
View/PR/indexing/keyboard acceptance.

Actual egui shortcut tests reject empty-reader file/symbol/line/search dialogs.
Accepted-source tests cover loading, no source, ready navigation, whitespace,
indexed targets and a caret move immediately before explicit references. A painted
menu test changes availability after opening, rejects a stale row click and uses
Home/Return to skip disabled commands. Removing the dispatch guard or menu enabled
state separately fails its regression. Final workspace passes 521 app tests
(seven ignored), two binary, four font, 13 reader-text and 36 doctests. Strict
Clippy, all-target check, build, warning-denied rustdoc, fmt and diff checks pass.
Architecture adds 153 NLOC including tests; command_enabled CCN is 7 and
execute_command falls from 14 to 13. No new hotspot/cycle/max-CCN increase.
The raw comparison flags four touched-file test clones; a complete multiset audit
proves all 91 baseline clones unchanged (zero added/removed).

Native private Wayland empty/loaded-reader runs at 1×/2× verify disabled clicks,
empty-reader shortcuts and available Go to Line navigation. The loaded fixture is
unindexed: definition remains disabled, while Go to Line opens and line-two copy
returns the exact original `// NATIVE_SECOND`. Captures are inspected; final
binary/fixture hashes and zero child exits are verified. Indexed command execution
is covered by the app suite and earlier dedicated native definition acceptance;
these four native runs do not add real-indexer coverage. An initial OCR selector
matched both toolbar and sidebar Search labels; its failed trial is retained and
excluded, then corrected by limiting the toolbar search region. Preliminary
pre-cache-reuse runs are retained separately from final-binary acceptance.

Evidence: `~/.local/state/radiology-qa/command-availability-20261031-7d4eb14`.
Broader enabled states, cross-owner sequences, source/grammar/index/PR corpus and
final Swift UI/standalone/reuse acceptance remain open. Full accessibility and
actual little-screen-clip integration remain deferred.


## View availability and numbered bookmark commands — 2026-10-31

At eb0ab96 View still enables folds and bookmarks without source/targets, labels
an existing bookmark as an add action, and omits Swift BookmarkCommands' ten
numbered menu entries. Source mode segments also accept input without source.
View now uses the same command_enabled predicate as Search and dispatch. Folding
requires a ready source and an applicable parsed fold; Collapse Functions requires
a callable fold. Bookmark toggling and source mode require ready source. Numbered
navigation requires the current repository/review and an occupied slot. Show All
Files requires nondefault filters, including Rust's path query. Global appearance,
source-size and filter preferences retain their existing availability.

The bookmark toggle now says Remove Bookmark at an already bookmarked location.
Go to Bookmark exposes all ten slots, current labels and Ctrl+digit hints, with
empty/unavailable slots disabled. It reuses BookmarkStore order and OpenBookmark,
so navigation, relocation, persistence and revision scoping use the existing path.
No new state, storage, worker, dependency, type scale or sibling coupling.

Growing policy initially trips a cognitive-complexity warning. A shared
current_caret_symbol accessor now validates the cached source/index/caret identity
for both availability and actual Go to Definition. The latter no longer repeats
refresh/occurrence scans after command dispatch has refreshed the target. This
removes redundant work rather than merely moving policy for a metric. Final
command_enabled CCN is 6 (previously 7); cached-target validation CCN 8 and
Go to Definition CCN 4. Architecture adds 154 NLOC including two tests and the
shared accessor, with no new hotspot/cycle/max-CCN increase. Raw touched-file
clone warnings are unchanged: a full multiset audit proves all 91 baseline clone
groups identical. Duplication percentage falls only because the denominator grows.

App tests cover no source, pending publication, parsed callable/type-only folds,
filter reset, mode readiness, current bookmark toggle and occupied/empty slots.
A rendered View test changes the toggle label while open, paints all ten entries,
and reaches slot 9 with Home/Return while skipping disabled slots. Restoring the
old policy and separately removing slot disabling each fails its regression.
Final workspace passes 523 app tests (seven existing ignored), two binary, four
font, 13 reader-text and 36 doctests. Strict Clippy/all-target check/build,
warning-denied rustdoc, fmt and diff checks pass; Toolbar argument docs render.

Native empty/loaded-reader checks at 1×/2× verify disabled folds, source-mode and
bookmark keys without source, add/remove labels after actual toggles, all-slot
presentation and keyboard submenu activation. After leaving the saved line,
Home/Return in the submenu returns to it; exact copy yields `// NATIVE_SECOND`.
Captures are inspected, original fixture and final binary hashes verified, and
all app/compositor children exit zero. Failed OCR trials are retained and excluded:
a focus rectangle hides View and low-resolution OCR drops a colon in a bookmark
label. The corrected harness reuses the observed stable toolbar anchor and uses
keyboard activation plus independent exact clipboard bytes for destination proof.

Evidence: `~/.local/state/radiology-qa/view-command-state-20261031-eb0ab96`.
Other command groups, broader bookmark model/interaction, grammar/index/PR/cache
corpora and final Swift UI/standalone/reuse acceptance remain open. Full
accessibility and actual sibling integration remain deferred.
