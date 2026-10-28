# Rust parity implementation

The full objective is to add the missing Swift capabilities, clean up the UI and
architecture, and keep Radiology standalone while preparing focused reuse with
little-screen-clip. The [independent review](rust-swift-parity-review-2026-10-27.md)
remains the requirement ledger; completing an initial package does not close it.

## Implemented and covered by current tests

- One command type and consumed shortcut path for buttons, menus, and keys;
  exact modifier matching prevents theme/symbol and mode/bookmark collisions.
- Idle reader construction independent of eframe; explicit launch intent takes
  precedence over session restoration. Source snapshots are shared through `Arc`.
- Exact-byte navigation restores columns and unfolds hidden destinations without
  inserting file-start history stops. Failed reads retain the current document.
- Repository transitions retire search/review receivers and clear stale source,
  index, revision, pin, and history state.
- Bookmark toolbar/inspector mutations save, navigation uses anchor relocation,
  and checkout/PR transitions load independent scopes. Settings save immediately.
- Scoped local-symbol queries and UTF-8/UTF-16/UTF-32 range conversion; SCIP
  metadata/document encoding tags retained and occurrence ranges are half-open.
- Private Git review objects and detached head worktrees, pinned head/merge-base
  reads, revision path availability, per-file zero-context hunks, safe checkout
  exit, and background review preparation. Before uses no After semantic index.
- Search ownership outside egui, debounce, cancellation, stale receiver retirement,
  errors/capped-results state, live glob controls, and After-tree search in review.
- Simultaneous independently resizable declaration/references, definition/outline,
  and bookmark regions; exact function-first outline spans and working TSX
  declarations/folds. Files filter/recovery and filtered Changes counts are wired.
- Recoverable errors and review preparation/cancel feedback are visible.

Tests include actual isolated Git repositories with dirty checkout bytes different
from the PR, renamed/spaced/added/deleted/binary files, application navigation and
bookmark scope integration, real ripgrep queries/errors, and headless egui checks
for simultaneous inspector regions and source keyboard exclusion.

## Still required before full parity acceptance

- Background repository/source analysis and index lifecycle: all Swift project
  strategies, tools/provisioning, cache/provenance, stale/partial/error/cancel UI.
- Full source viewport reuse/extraction: canonical selection/copy, local find,
  source zoom, variable-height wrapping/hit testing, long-line bounds, precise
  caret and destination affordances, content-keyed folds/pins and occurrence scope.
- Swift/Kotlin grammars, broader query-based folds/scopes, syntax-aware inline
  test detection/presentation, and hidden-test navigation/outline behavior.
- Parsed/validated call trees and reference snippets/coverage/disclosure,
  explicit Find References and Follow Caret, full source definition previews,
  independent inspector keyboard preview/commit behavior.
- Semantic Critical Lens projection, persisted symbol/package marks, call graph,
  frontier/hover summaries and fold annotations.
- Complete PR workflow: recent-list chooser, cancellation/timeout of child
  processes, exact refresh caret/viewport/rename mapping and notice behavior,
  snapshot-cache retention, pinned After indexing and reuse provenance, disabled
  unavailable revision/change controls and one-time review folding defaults.
- Shared lightweight palette/font/chrome/text mechanisms from little-screen-clip;
  no unconditional heavyweight workbench or sibling checkout dependency.
- Complete menu/shortcut enabled/focus states, runtime light/dark/scaling checks,
  actual desktop visual acceptance, and requirement-by-requirement final audit.

## Commit discipline

The user explicitly requested incremental commits. Keep backend mechanisms,
integration/UI work, and documentation in reviewable commits as work progresses.
Do not merge to main. Initial commits:

- `12d51fa` — review documentation and integration boundaries.
- `8b199d2` — core coordinates, bounded reads, persistence, folds, TSX/outline data.
- `b87494d` — immutable private PR snapshots and exact per-file diffs.
- `72bba06` — cancellable repository search ownership and error handling.
- `011072d` — shared reader commands, exact app workflows, search/inspector UI.
- `b2b6134` — Cargo build jobs limited to 16 on the 32-CPU development system.

Native Wayland smoke checks now supplement the headless tests: all three Context
regions reserve their space, CRLF source renders without newline-box glyphs,
and Ctrl+Alt+T changes appearance once without also opening Go to Symbol. Broad
desktop interaction and side-by-side Swift acceptance remain required.

Architecture measurements use `.fitness/baselines/parity-implementation-start.json`
(ignored local artifacts). The original shortcut hotspot had CCN 125. Actual
duplicate commands have been removed and the inspector no longer duplicates
outline/symbol content between tabs. Continue reducing accidental duplication in
GitHub transport and sidebar/source rendering; do not treat aggregate metric
improvement as permission to leave new avoidable hotspots.
