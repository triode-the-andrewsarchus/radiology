# Rust parity implementation

The full objective is to add the missing Swift capabilities, clean up the UI and
architecture, and keep Radiology standalone while preparing focused reuse with
little-screen-clip. The [independent review](rust-swift-parity-review-2026-10-27.md)
remains the requirement ledger; completing an initial package does not close it.

See also [indexing lifecycle](rust-indexing-parity.md) for background imports
and the remaining generation/cache requirements.

## Current scope clarification (2026-10-30)

The user explicitly excludes full accessibility for disabled users from the current
parity goal. Screen-reader support, native accessibility bridges, accessible source
text and exhaustive assistive-technology acceptance are deferred. Keyboard
shortcuts, command ownership, focus behavior and menu dismissal remain required.
Do not count deferred accessibility work as a blocker or as unfinished required
parity. Existing behavior need not be removed. The uncommitted AccessKit bridge
experiment was set aside without adding its platform dependencies.

Radiology must remain independently buildable. Shared assets, reusable reader
contracts and compatible broad UI conventions prepare for little-screen-clip;
actual sibling adoption/integration is deferred until the user requests it.

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
- Critical Lens uses background, exact-source call projections with shared Swift
  queries and semantic classification. Ordinary resolved callees recede; unknown
  and critical call names remain prominent. Renderer span splitting preserves
  original text/geometry and replaces color/name guessing. [Lens limits](rust-critical-lens.md).
- Explicit Find References/Follow Caret use exact scoped identities and replaceable
  background queries independently of source navigation. Direct rows are virtualized;
  automatic references match parsed declaration-name bytes rather than lines.
  [Reference context and remaining inspector work](rust-reference-context.md).
- Source-backed references group exact source rows by file, include definitions
  separately, report coverage and support Load More with retained selection,
  disclosure and keyboard focus. Native Return/Back acceptance passes.
  [Reference results](rust-reference-results.md).
- Source-validated incoming caller paths integrate independent background
  disclosures, visible-slice painting, branch/boundary notes, Trace More,
  digest-guarded navigation and pre-frame keyboard ownership. Native caller
  Return/Back, expansion and F12 acceptance pass. [Callers](rust-incoming-callers.md).
- Background under-caret metadata shows bounded signatures/prose, kinds, indexed
  counts, local scope/source context, Here/package/unavailable states and exact
  alternative destinations. Parsed impl/extension type links feed references.
  Native Return/Back and type F12 acceptance pass. [Inspector metadata](rust-inspector-metadata.md).
- Bookmark pane has UUID-based list selection, ordered Return/Delete navigation,
  header toggling, drag insertion and direct shortcut assignment, with scoped
  persistence and native restart acceptance. [Contract and remaining audit](rust-bookmark-pane.md).
- Outline has source-bound keyboard preview/commit, nested disclosure, active
  declaration following, hidden/stale-target rejection, virtual rows and native
  clean-session acceptance. First commits seed an otherwise empty history.
  [Contract and remaining presentation work](rust-outline-pane.md).
- Review outline callables show cached six-point markers from exact Before/After
  changed bytes. Empty-side hunk anchors preserve unchanged caret lines across
  revision toggles. Pinned Git/UI/native coverage is recorded in the
  [marker contract](rust-review-outline-changes.md).
- Source-bound destination feedback follows successful explicit jumps, with
  shaped visual-row geometry, bounded hold/fade, preview/history cancellation and
  first-paint timing. [Contract and native scope](rust-destination-feedback.md).
- Recoverable errors and review preparation/cancel feedback are visible.
- Checkout branch/change refresh uses a cancellable bounded worker snapshot, literal
  NUL paths, exact-source hunk publication and Cancel/Retry/Dismiss controls.
  [Contract and native acceptance](rust-git-refresh.md).
- Repository root/inventory/initial-tree discovery is cancellable background work;
  deferred launch destinations, current-reader retention and filter freshness
  publish as one root transaction. [Host contract](rust-repository-opening.md).

Tests include actual isolated Git repositories with dirty checkout bytes different
from the PR, renamed/spaced/added/deleted/binary files, application navigation and
bookmark scope integration, real ripgrep queries/errors, and headless egui checks
for simultaneous inspector regions and source keyboard exclusion.

## Still required before full parity acceptance

- [Real Rust/TypeScript/Go/Python/Kotlin/Java/C/C++ lifecycle acceptance](rust-real-indexer-acceptance.md) now passes through shared production-path checks, including Kotlin-to-Java navigation. All six adapter families now have small real-tool lifecycle evidence. Android/AGP, broader C/C++ build-database discovery and other corpora remain open.
- Finish semantic context/cache reuse work;
  complete index lifecycle with all Swift project strategies,
  broader project/corpus acceptance and pinned-snapshot provenance. Rust manifest
  feature cfg overrides and real-tool/native gated navigation are integrated; see
  [Rust feature acceptance](rust-feature-configuration.md). Per-project
  exact-context/version reuse and origin tracking are integrated with Update/Reindex
  controls and native recovery/restart acceptance; see [project reuse](rust-project-index-reuse.md). Explicit
  pinned provisioning and automatic regeneration are integrated with controlled
  native Cancel/failure/recovery acceptance; see [tool provisioning](rust-tool-provisioning.md). Basic
  cache persistence and stale/partial/error/cancel UI are integrated. Nested SCIP
  decoding and semantic-map construction now use [cooperative cancellation](rust-scip-cancellation.md),
  with native Cancel/retention/reload acceptance. [Normalized-cache JSON](rust-cache-json-cancellation.md)
  also interrupts bounded reads/writes and validates recovery. Native parser/
  query stages now use [latched progress cancellation](rust-native-analysis-cancellation.md).
  Preview/reference/caller/Critical Lens reads now
  propagate their owning flags and budgets; [runtime checks](rust-analysis-source-cancellation.md).
  Source loading and checkout refresh use bounded process-group cancellation/deadlines;
  see [source process acceptance](rust-source-process-cancellation.md).
- Remaining source presentation: broader wrapping/scroll/scaling acceptance.
  [PR refresh restoration](rust-pr-refresh-restoration.md) now stages atomic source
  publication, maps Git renames/lines and latest caret/viewport state, retains only
  exact current-file pins and applies new fold defaults, with native acceptance.
  Transient occurrence
  selection, syntax-aware pinned fallback, exact byte-based fold projection, enclosing-callable
  shading and destination feedback are integrated; broader wrapping/scroll anchoring,
  scaling, and interaction acceptance remain part of the final audit. File-return
  fold restoration and initial body/PR defaults are integrated; see
  [acceptance](rust-fold-restoration.md). Typed structural folds and
  Fold to Definitions are integrated; see [query/command acceptance](rust-structural-folds.md).
- Broader Swift/Kotlin declaration/scope/grammar acceptance and grammar/fixture
  coverage beyond the implemented rules. Native syntax colors are integrated;
  see [capture/recolor contract](rust-native-highlights.md).
- Remaining outline presentation and broader bookmark interaction parity,
  mixed-language and long-name corpus, pinned-review and light/dark/scaling acceptance.
  Background metadata, local/package explanations and impl/extension type links
  are integrated; see [metadata acceptance](rust-inspector-metadata.md).
  Source excerpts, validated caller trees and reference selection/disclosure/commit
  are integrated; [preview contract and remaining scope](rust-inspector-previews.md).
- [Critical Lens projection/classification reuse](rust-critical-projection-reuse.md)
  now has exact scope invalidation, bounded LRU and native revisit/policy/index
  acceptance. Broader graph/corpus/runtime acceptance remains.
  Fold counts now publish progressively with retired-request isolation. Pointer-dwell hover summaries/navigation
  are integrated; see [hover acceptance](rust-critical-hover.md). Parsed summaries, bounded
  frontiers and snapshot workers now feed source fold counts; see
  [graph/count acceptance](rust-call-graph.md). Exact background projection and
  persisted symbol/package menus are integrated; see [marks](rust-critical-marks.md).
- Complete remaining PR workflow: broader reuse provenance and final late-data/
  refresh acceptance. [Reader notices and control availability](rust-reader-notices-controls.md)
  now provide bounded diagnostics with complete Details/Copy, disabled unavailable
  revisions, guarded keyboard toggles and change-boundary controls, with native checks.
  [Preparation processes](rust-pr-process-cancellation.md) now share cancellation/
  deadlines with native metadata/fetch Cancel and pinned-review recovery checks.
  [Recent chooser](rust-pr-chooser.md) now provides 25 repository-owned entries,
  independent list refresh/cancel, active fallback and explicit Exit PR, with native
  failure/recovery and light/dark checks. [Private-store writer leases](rust-pr-store-recovery.md)
  now cover cross-process preparation and surviving children, staged initialization,
  isolated fetches and missing-tree recovery. [Verified tree reuse and retention](rust-pr-tree-cache.md)
  now cover exact bytes, active snapshot/background-consumer leases, cross-process
  protection and bounded LRU retention, with native reuse/corruption/restart checks.
  [Pinned After indexing](rust-pinned-index.md) now uses isolated projects, exact
  Git inputs and commit-scoped cache identity, with native generation/failure/reload/
  restart checks. Before stays source-only. Broader corrupt-store recovery and
  owned retired-project cleanup and legacy migration are integrated with native
  cross-process/restart acceptance; see [cache maintenance](rust-project-cache-maintenance.md).
  [Normalized revision retention](rust-normalized-revision-retention.md) now bounds owned current-format records with shared writer protection and cross-process tests; native refresh/reuse, cross-process deferral, restart, eviction and explicit regeneration now pass. Broader corpus acceptance remains open.
- [Portable CJK/emoji fallbacks](shared-font-fallbacks.md) now use a local asset-only/optional-egui crate with exact host assets; native representative glyph/selection/copy checks now pass at 1×/1.5×/2×, with light/dark find at 1×/2×. The sibling-matching Fira Sans UI face and 12 pt metadata are now integrated with unchanged source geometry and native 1×/2× evidence. [Unified reader visuals](rust-reader-appearance.md) now align neutral surfaces/text and widget states, with contrast and native 1×/2× checks. Broader type-role/spacing alignment remains open.
- Shared lightweight palette/font/chrome/text mechanisms from little-screen-clip;
  no unconditional heavyweight workbench or sibling checkout dependency.
- [Files-row Reveal in File Manager](rust-file-manager-reveal.md) now provides the missing file/folder action with scoped background handoff and visible errors; native file/folder/Unicode paths, private PR routing, rejection/timeout and pending-scope retirement now pass through real gdbus and a private controlled receiver.
- [Keyboard command audit](rust-keyboard-audit.md) adds dedicated definition/Reindex shortcuts, the definition menu entry and repeated-generation guards. Complete menu/shortcut enabled/focus states and traversal. [Shared context-menu Escape dismissal](rust-context-menu-dismissal.md) now repairs the native-proven gap through pre-frame ownership, with component/native acceptance. Continue runtime light/dark/scaling checks,
  actual desktop visual acceptance, and requirement-by-requirement final audit.

## Commit discipline

The user explicitly requested incremental commits and pushing after each commit. Keep backend mechanisms,
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
