# Rust parity implementation

The full objective is to add the missing Swift capabilities, clean up the UI and
architecture, and keep Radiology standalone while preparing focused reuse with
little-screen-clip. The [independent review](rust-swift-parity-review-2026-10-27.md)
remains the requirement ledger; completing an initial package does not close it.

See also [indexing lifecycle](rust-indexing-parity.md) for background imports
and the remaining generation/cache requirements.

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
- Local source find uses complete spillable results, literal/regex and case
  controls, original-byte navigation, shared menu/keyboard commands, visible
  errors, document-content invalidation, and shaped-glyph match highlights.
  Repeated Return keeps query focus; closed find does not navigate new documents.
- Measured source viewport uses bounded UTF-8 fragments and variable row heights;
  shaped hit testing/visual arrows, Shift/drag selection, exact original-byte copy,
  caret strokes, long-line destination reveal, and independently persisted source
  zoom are integrated. Source focus is separate from other controls. Indexed pins
  use exact semantic occurrences; lexical fallback is scoped to enclosing functions.
- Native Swift/Kotlin grammars, nested declaration outlines and source-span folds
  are registered and tested; detailed highlighting/query coverage remains open.
- Conservative AST test detection and independent Hide Tests projection are
  integrated: owned headers/controls/status, exact source copy, manual-fold
  preservation, filtered inspector outline, readable explicit test documents,
  and automatic reveal through navigation/local find. Structural facts share one
  native parse. [Evidence and boundaries](rust-inline-test-parity.md).
- Asynchronous source reads/analysis/highlights/diffs publish atomically with
  exact navigation and history. Pending/cancel feedback preserves the current
  reader; bookmark/definition/revision pre-reads are removed. Theme recoloring
  shares immutable source; repository/review transitions retire stale reads.
  [Source integration and limits](rust-source-loading.md).
- Isolated checkout Generate/Reindex/Cancel and bounded coverage/diagnostic UI
  share configured tool paths and retire competing jobs. Generated destinations
  require matching source digests. Workspace symbols, caller rows and F12 share
  exact encoding-aware asynchronous navigation; the symbol picker is virtualized.
  [Index lifecycle](rust-indexing-parity.md) and [navigation acceptance](rust-semantic-navigation.md).
- Generated semantic data and bounded reports persist atomically outside the
  checkout. Opening/Reload Index restore current or explicitly stale cache data
  without tools and fall back to unverified imports for missing/invalid entries.
  Source identity guards survive reload. [Cache acceptance](rust-index-cache.md).
- C/C++ generation uses rebased private compilation databases and copied response
  files, preserving source input verification, UTF-8 coordinates and cache reports.
  Real Clang compiles the rebased fixture with original sources unavailable;
  native generation uses fixture SCIP output. [C/C++ limits](rust-clang-indexing.md).
- Occurrence pins have unlimited groups with stable cyclic colors. Exact
  document/revision/content activation restores them after navigation and rejects
  changed bytes. Scrollable group controls remove individual/current-document
  pins. [Pin acceptance and remaining scope](rust-occurrence-pins.md).
- Recoverable errors and review preparation/cancel feedback are visible.

Tests include actual isolated Git repositories with dirty checkout bytes different
from the PR, renamed/spaced/added/deleted/binary files, application navigation and
bookmark scope integration, real ripgrep queries/errors, and headless egui checks
for simultaneous inspector regions and source keyboard exclusion.

## Still required before full parity acceptance

- Background repository discovery/Git refresh/SCIP decode and semantic context;
  complete index lifecycle with all Swift project strategies, provisioning,
  per-project/version-aware cache reuse and pinned-snapshot provenance. Basic
  cache persistence and stale/partial/error/cancel UI are integrated. Source/native stage and Git
  subprocess cancellation still need finer interruption/deadlines.
- Remaining source presentation: content-keyed fold and exact PR-refresh pin
  restoration, AST-aware fallback occurrence scope, enclosing-function body
  shade and transient destination feedback. Broader wrapping/scroll anchoring,
  scaling, and interaction acceptance remain part of the final audit.
- Swift/Kotlin highlighting and broader declaration/query-based folds/scopes;
  broader grammar/fixture acceptance beyond the implemented inline-test rules.
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
