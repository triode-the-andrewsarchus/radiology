# Radiology documentation index and current state

Radiology is a native, read-only repository browser for understanding unfamiliar codebases. While the original implementation was written in Swift for macOS, the active focus is the standalone Linux-native rewrite in Rust using EGUI (`eframe` 0.27 / `egui` 0.27).

## Current accepted state

[Large-source reader performance](plans/rust-reader-performance.md) now records
exact-byte 6 MB/Back acceptance, profiling, lazy per-grammar query initialization
and shared sparse Unicode coordinates. Unwrapped destinations now reveal before
complete prefix measurement and stay anchored while it refines, with exact
provisional pointer hits and native 1×/2× evidence. Slow loading and wrapped reveal
remain open.
[Unwrapped fragment geometry](plans/rust-unwrapped-fragment-geometry.md) records
the integrated horizontal placement and deferred keyboard/reveal geometry for
bounded source fragments, with native 1×/2× copy/layout evidence.
[Ordered source gestures](plans/rust-source-gesture-order.md) now retain clicks,
drags, Ctrl-click and menu keys while prefixes measure.
[Bounded Files labels and source breadcrumbs](plans/rust-pane-labels.md) repair
long-name pane growth and add Swift-style compaction/full-location menus.
[Modal/breadcrumb focus return](plans/rust-modal-focus.md) now restores source keys
and the prior control on cancellation, with real-app and native 1×/2× evidence.
[Toolbar keyboard ownership](plans/rust-toolbar-keyboard.md) now shares traversal,
PR scrolling and focus return across all four menu groups, with native 1×/2× evidence.
Active titles now follow their popup, and focused menus preserve global reader shortcuts.
[Bounded Changes and Search results](plans/rust-result-pane-labels.md) now retain
pane widths, Swift-style preview limits and exact navigation/interaction targets.
[Inspector labels and signature disclosure](plans/rust-inspector-labels.md) now
bound long names and restore Swift's source-bound declaration disclosure, with
keyboard/copy tests and native 1×/2× evidence. Pinned summaries now include scoped
kind/category metadata, with current-index refresh and Follow Caret coverage.
[Reference/caller presentation and region bounds](plans/rust-reference-presentation.md)
now preserve fitted caller metadata, source-bound row activation and all three
regions after resizing. Ordered reference/caller keys now cross focus boundaries
within one batch, with real-app entry/exit/history and native navigation evidence.
[Inspector/global keyboard ownership](plans/rust-keyboard-owner-order.md) now
preserves modal/query handoff order and opening text through window registration.
[Inspector row identities](plans/rust-inspector-row-identity.md) now reject stale
bookmark/outline pointer releases after target replacement or movement.
[Outline status explanations](plans/rust-outline-status.md) now distinguish
loading, parser failure, unsupported files, empty languages and hidden tests.
Broader mixed-owner, mixed-language/pinned-review visual and final parity acceptance
remain open.

[Source Ctrl-click navigation](plans/rust-source-pointer-navigation.md) reuses
exact indexed targets, preserves departure history and gates source menu actions.
[Shared context-menu keyboard ownership](plans/rust-context-menu-keyboard.md)
repairs the native-proven arrow leak and adds traversal, submenus and opening-frame
key reservation with final 2× acceptance.

[Headless syntax-resource verification](plans/rust-resource-verification.md) now
checks native reader facts and both color themes, with failing-query and standalone
binary evidence. It replaces the old fallback-presence-only verifier.

**Parity implementation underway (2026-10-30):** initial correctness, private PR
snapshots, cancellable repository/local source search, inspector integration, and
a measured selectable source viewport with zoom are implemented with regression
tests. [Implementation status and remaining acceptance queue](plans/rust-parity-implementation.md).
Async source loading, inline Hide Tests and isolated checkout index generation
are also integrated, including persistent index reload, supported C/C++ command
rebasing, unlimited content-bound pins and background call-focused Critical Lens.
The full parity objective remains open; the review below describes the pre-fix
snapshot. The user requests incremental commits and pushing after every commit.
Full accessibility is explicitly deferred by the user (2026-10-30); keyboard
shortcuts, focus behavior and menu dismissal remain required. Actual sibling
integration is also deferred; standalone operation and reuse preparation remain
in scope. See the [current scope](plans/rust-parity-implementation.md#current-scope-clarification-2026-10-30).

[Reader appearance](plans/rust-reader-appearance.md) records host-aligned neutral roles,
unified widget visuals, contrast checks, native 1×/2× evidence and the original Escape menu gap.
[Context-menu dismissal](plans/rust-context-menu-dismissal.md) records its shared repair,
pre-frame Escape ownership and native/component regression evidence.
[Keyboard command audit](plans/rust-keyboard-audit.md) records Linux mappings,
dedicated definition/Reindex bindings, shared menu dispatch, repeat-job guards,
native acceptance and same-frame modal ownership.

[Ordered source input](plans/rust-source-input-order.md) records coalesced navigation,
selection and copy ordering, all-scale regression coverage and native 2× acceptance.

[Private PR-store recovery](plans/rust-pr-store-recovery.md) records child-held
writer leases, staged initialization, isolated fetches and native crash/retry evidence.

[Recent PR chooser](plans/rust-pr-chooser.md) records the repository-scoped 25-entry
list, compact switcher/Exit PR, keyboard behavior and native failure/recovery checks.

[PR process cancellation](plans/rust-pr-process-cancellation.md) records shared
metadata/fetch/snapshot deadlines, tree ownership and native Cancel/recovery checks.

[Normalized-cache JSON cancellation](plans/rust-cache-json-cancellation.md) records
latched bounded decoding/writes, atomic retention and native cache recovery.

[SCIP cancellation](plans/rust-scip-cancellation.md) records nested decoding and
semantic-map retirement, malformed-input checks and native Cancel/reload acceptance.

[Native analysis cancellation](plans/rust-native-analysis-cancellation.md) records
latched parser/query interruption, fact-stage retirement and large-file native acceptance.

[Semantic source cancellation](plans/rust-analysis-source-cancellation.md) records
preview/reference/caller/graph propagation, cache safety and live-child verification.

[Bounded source Git reads](plans/rust-source-process-cancellation.md) records request
cancellation, shared exact-source hunks, raw pinned blobs and native live-child checks.

[Background repository opening](plans/rust-repository-opening.md) records atomic
root/inventory/tree publication, launch intent, filter freshness and native acceptance.

[Background checkout Git refresh](plans/rust-git-refresh.md) records snapshot
ownership, shared process/record reuse, bounds and native acceptance.

[Native Swift/Kotlin/ECMAScript source colors](plans/rust-native-highlights.md) records shared
capture rules, dialect overlays, precedence, exact-source recoloring, retained theme
facts and native TypeScript/TSX acceptance.

[Transient occurrence selection and pin styling](plans/rust-occurrence-selection.md) records
click-owned groups, shared resolution, shaped outlines and selection independence.

[Syntax-aware occurrence pins](plans/rust-syntactic-pins.md) records exact parsed scopes,
excluded syntax, partial-SCIP preference and source ownership constraints.

[Canonical-byte fold projection](plans/rust-fold-projection.md) records shared bounded
visible-span fragments, delimiter/header semantics and exact input/copy mapping.

[Exact fold identities and overlapping headers](plans/rust-fold-identities.md) records
independent range state, defaults/restoration, PR ancestry and Critical Lens count ownership.

[Enclosing callable shading](plans/rust-function-shading.md) records parsed scope reuse,
shaped bands, background layer order, empty-line hit testing and native acceptance.

[Transient destination feedback](plans/rust-destination-feedback.md) records the
source-bound hold/fade, shaped-row rendering, explicit navigation policy and
native success/failure/cancellation acceptance.

[Changed callable markers in review outlines](plans/rust-review-outline-changes.md)
records exact Before/After byte overlap, cached presentation, PR-only ownership,
zero-length hunk anchor mapping and pinned Git/native acceptance.

[Outline selection, preview and native acceptance](plans/rust-outline-pane.md)
records source-bound keyboard navigation, hierarchical disclosure, exact preview/
commit semantics, shared list mechanisms and remaining outline/inspector work.

[Bookmark pane behavior and native acceptance](plans/rust-bookmark-pane.md)
records identity-based keyboard selection, shortcut assignment, drag reordering,
scoped persistence and remaining bookmark/outline acceptance.

[Shared source viewport primitives and integration contract](plans/shared-text-viewport.md)
records the standalone `reader-text` package extracted from existing host mechanisms.

[Rust inline-test detection, presentation, and verification](plans/rust-inline-test-parity.md)
records the native grammar and Hide Tests implementation, including remaining
background-analysis and highlighting work.

[Background source preparation and integration contract](plans/rust-source-loading.md)
records the integrated source worker, atomic navigation/history publication, and
remaining repository/index/background cancellation work.

[Rust indexing lifecycle and remaining Swift requirements](plans/rust-indexing-parity.md)
records asynchronous SCIP imports, project strategies, tool probes, cancellable
processes, real-input workspaces, Generate/Reindex controls, native fixture
acceptance and remaining cache/provisioning/snapshot integration.

[Critical Lens decision storage and reader integration](plans/rust-critical-marks.md)
records atomic ordered storage, durable session edits, source menus and native
restart/projection acceptance.

[Critical Lens pointer-dwell hover](plans/rust-critical-hover.md) records the
controller, reader integration and acceptance scope.

[Parsed call summaries and bounded critical frontiers](plans/rust-call-graph.md)
records lexical/semantic summaries, snapshot workers, progressive source fold
counts and remaining graph/cache acceptance.

[Rust Critical Lens implementation and acceptance](plans/rust-critical-lens.md)
records shared call queries, exact semantic projection and remaining reader/graph work.

[Source-validated incoming call paths](plans/rust-incoming-callers.md) records the
shared callee resolver, retained background disclosures, virtualized caller rows,
Trace More, guarded navigation and native acceptance.

[Source-backed reference results](plans/rust-reference-results.md) records validated
source-line hits, definition/reference counts, coverage, expandable queries and
native keyboard/navigation acceptance.

[Inspector metadata and implementation context](plans/rust-inspector-metadata.md)
records background under-caret facts, scoped local/package explanations, parsed
implementation type links and native definition/navigation acceptance.

[Inspector source previews and navigation](plans/rust-inspector-previews.md) records
bounded source excerpts, independent reference selection/disclosure/commit,
keyboard focus and remaining inspector presentation/call-tree work.

[Reference focus and inspector context](plans/rust-reference-context.md) records
explicit Find References, Follow Caret, exact declaration identities and background
reference loading, with the remaining preview/call-tree work.

[Typed structural folds and Fold to Definitions](plans/rust-structural-folds.md)
records shared query roles, declaration containment, native command acceptance and limits.

[Exact-document fold restoration](plans/rust-fold-restoration.md) records shared
fold/pin memory, parsed body/PR defaults, source-publication integration and native acceptance.

[Occurrence pin storage and controls](plans/rust-occurrence-pins.md)
records unlimited groups, stable colors and content/revision-bound restoration.

[C/C++ compilation database rebasing](plans/rust-clang-indexing.md)
records private command/response/input/output mapping and compiler acceptance.

[Persistent semantic index cache](plans/rust-index-cache.md)
records atomic storage, source provenance, background loading and remaining
reader/cache lifecycle integration.

[Semantic coordinate validation and shared navigation](plans/rust-semantic-navigation.md)
records canonical definitions, exact reference coordinates, source identity and
palette/inspector navigation acceptance.

**Independent Rust/Swift parity review (2026-10-27): parity is incomplete.**
The source audit and headless probes at `fd421e1` found 17 groups of functional,
interaction, UI, and validation gaps. In particular, PR review reads checkout
bytes rather than pinned revisions; indexing generation is absent; Critical Lens
policy is disconnected from rendering; navigation, keyboard focus, bookmarks,
repository transitions, syntax coverage, and the inspector require repair.
Earlier full-parity/completion claims below are historical implementation reports,
not the current acceptance verdict.

- [Current parity matrix, prioritized findings, evidence, and repair gates](plans/rust-swift-parity-review-2026-10-27.md)
- Validation: 38 unit tests and 30 doctests pass; rustfmt and the current resource
  lookup check pass. Clippy with `-D warnings` fails on three float-literal warnings
  with Rust 1.98.1. Swift runtime and side-by-side visual acceptance remain unverified.
- User direction: Radiology remains independently buildable/runnable now, while
  preparing for integration with `~/Documents/git/local/little-screen-clip`.
  Favor shared source-view, palette, typography, shell, and job mechanisms; retain
  Swift's functional workflow and align broad Linux UI conventions with that host.
  The review identifies concrete reuse candidates and dependency boundaries.
- This review changed documentation only. The repair packages are proposed work,
  not implemented fixes.

## Historical implementation reports (parity claims superseded by review above)

**Deduplicated Status Bar Remediation (2026-10-27):**
Eliminated duplicated status bar at the bottom of the application:
- Removed extraneous window-level `TopBottomPanel::bottom("status_bar")` from [`RadiologyApp::update`](file:///home/vcarlson/radiology/src/ui/app.rs).
- Retained the single, authentic macOS-parity status bar in [`SourceView::show`](file:///home/vcarlson/radiology/src/ui/source_view.rs) located directly within the central editor beneath the divider (matching `ReaderDetail.swift`).
- Added clickable borderless `Line N` button to the `SourceView` footer wired to `SourceViewAction::OpenGoToLine` (opening `GoToLineModal` on click, with tooltip `Go to Line (⌘L)`).
- Omitted revision label when browsing local checkouts (matching macOS Swift, which only renders `Before` or `After` during PR reviews).
- 38 unit tests and 30 doctests passing, clippy clean with -D warnings, rustfmt clean. Swift code intact.
[Full parity audit and defect remediation plan](plans/swift-parity-and-defect-remediation.md).

**Full Swift Parity Audit, Toolbar Menus, Collapsible Panels & Interaction Parity (2026-10-27):**
Completed full-coverage audit of original macOS Swift code against the Rust + EGUI port and implemented parity across all missing UI controls, menus, and click actions:
- Collapsible Side Panels: Added `show_sidebar` and `show_inspector` with native toolbar toggle buttons and keyboard shortcuts (`Ctrl+Alt+S` / `⌃⌘S` and `Ctrl+Alt+I` / `⌥⌘I`).
- Repository Menu: Replicated `RepositoryMenu.swift` with native dropdown menu showing "Open Repository… (⌘O)", "Open Recent" submenu with interactive history list, and "Copy Repository Path".
- Pull Request Menu: Replicated `PullRequestMenu.swift` with native dropdown menu showing `#num · Title`, "Refresh PR (⌘R)", "Open Pull Request… (⌘P)", and "Exit PR Review".
- Source Search Menu: Replicated `SourceSearchMenu.swift` with native toolbar dropdown for "Go to File… (⌘K)", "Go to Symbol… (⌘T)", "Go to Line… (⌘L)", and "Search Repository (⇧⌘F)".
- Interactive Symbol Breadcrumbs: Made enclosing symbol breadcrumb segments interactive clickable links that scroll and center the editor viewport on the clicked declaration.
- File Tree Context Menu & Changed Dot: Replicated `FileTreeLabel.swift` with 5px accent dot on modified PR files and right-click context menu ("Copy Relative Path").
- Bookmarks Pane Context Menu: Replicated `BookmarkContextMenu.swift` with right-click context menu ("Go to Bookmark", "Move Up", "Move Down", "Remove Bookmark").
- Editor Interaction: Replicated `ReaderTextView.swift` with double-click word occurrence pinning, Return/Enter definition navigation, and right-click context menu ("Go to Definition", "Copy Line").
- Full Keyboard Shortcut Suite: Bound `Ctrl+O`, `Ctrl+P`, `Ctrl+R`, `Ctrl+Alt+S`, `Ctrl+Alt+I`, `Ctrl+Alt+1`, `Ctrl+Alt+2`, `Ctrl+Shift+F`, and `Return`.
- 38 unit tests and 30 doctests passing, clippy clean with -D warnings, rustfmt clean, zero emojis. Swift code intact.
[Full parity audit and defect remediation plan](plans/swift-parity-and-defect-remediation.md).

**Full Gutter Line Number Rendering (2026-10-27):**
Restored continuous line number rendering across all lines in the editor gutter:
- Removed `is_multiple_of(5)` filter in [`SourceView::show`](file:///home/vcarlson/radiology/src/ui/source_view.rs) so every line displays its line number.
- 38 unit tests and 30 doctests passing, clippy clean with -D warnings, rustfmt clean. Swift code intact.
[Parity audit & defect remediation plan](plans/swift-parity-and-defect-remediation.md).

**Non-Selectable Inspector & Sidebar Navigation Labels (2026-10-27):**
Disabled text selection on navigation labels in the right inspector (Symbols, Callers, Outline, Bookmarks) and left sidebar:
- Configured `ui.style_mut().interaction.selectable_labels = false;` so label text no longer intercepts mouse clicks for text selection.
- Clicking anywhere on a label or row immediately triggers navigation actions.
- 38 unit tests and 30 doctests passing, clippy clean with -D warnings, rustfmt clean. Swift code intact.
[Parity audit & defect remediation plan](plans/swift-parity-and-defect-remediation.md).

**Horizontal Scroll Centering Defect Remediation (2026-10-27):**
Resolved defect where opening a file with long lines automatically centered the horizontal scroll bar:
- Eliminated per-frame `line_response.scroll_to_me(Some(Align::Center))` which operated on the full unclipped `row_width` and centered the horizontal scroll bar.
- Attached `ScrollArea::both().id_source(&buffer.path)` so each file independently tracks scroll position.
- Programmatic line navigation sets only `vertical_scroll_offset(scroll_y)`; horizontal scroll defaults to `0.0` (fully left) on file open and is never automatically modified.
- 38 unit tests and 30 doctests passing, clippy clean with -D warnings, rustfmt clean. Swift code intact.
[Parity audit & defect remediation plan](plans/swift-parity-and-defect-remediation.md).

**Git Changes Tab, Clickable Outline Navigation & Ripgrep Caller Fallback (2026-10-27):**
Resolved four core user defects and missing macOS functional parity features:
- Changes Tab in Sidebar: Made the `Changes` tab permanently selectable alongside `Files` and `Search`. Displays real Git working tree diffs (`git diff HEAD` and untracked `??` files) with line additions and deletions, or active PR changes when reviewing a PR. Rendered with clean status badges (`M`, `A`, `D`, `R`, `C` with zero emojis/unicode symbols), bold filename, directory, and `+A −D` counters. Added native sidebar footer (`{visible} of {total} files` + `Icon::Diff`).
- Outline Clickable Navigation & Editor Centering: Outline symbol rows are interactive with click handling and pointing hand cursor. Programmatically scrolls the editor and centers the target line via `scroll_area.vertical_scroll_offset` and `line_response.scroll_to_me(Some(Align::Center))`.
- Caller List Population & Declaration Region Wiring: Context Inspector Declaration card displays enclosing symbol and line link. Implemented `App::find_callers_fallback` using high-speed synchronous `rg --json -w <symbol> .` (~5ms) to discover callers across the repo with deterministic 12-color caller capsule pills.
- Bottom Status Bar Parity: Replicated macOS `ReaderDetail.swift` status bar (language, line count, index status, Line N button, UTF-8 · Read Only).
- 38 unit tests and 30 doctests passing, clippy clean with -D warnings, rustfmt clean. Swift code intact.
[Parity audit & defect remediation plan](plans/swift-parity-and-defect-remediation.md).

**Gutter Viewport Anchor & File Switch Displacement Remediation (2026-10-27):**
Resolved defect where selecting a new file displaced line numbers and gutter to `x = 0.0`, covering the sidebar file tree:
- Explicitly captured `editor_viewport = ui.available_rect_before_wrap()` outside `ScrollArea`. In EGUI, child `ui.clip_rect().min.x` could fall back to `0.0` when switching files before `content_is_too_large` was computed.
- Directly anchored gutter column, row line numbers, and code scissor bounds to `editor_viewport.min.x` (fixed at `CentralPanel` left boundary).
- Set `ui.set_clip_rect(ui.max_rect())` on `CentralPanel`.
- 38 unit tests and 30 doctests passing, clippy clean with -D warnings, rustfmt clean. Swift code intact.
[Parity audit & defect remediation plan](plans/swift-parity-and-defect-remediation.md).

**Dark Mode Premultiplied Alpha & Contrast Remediation (2026-10-27):**
Resolved active line and wash contrast issues in dark mode:
- Migrated translucent color definitions from `from_rgba_premultiplied` to `from_rgba_unmultiplied`. Previously, passing unscaled RGB (`blue = 255`) caused premultiplied blending to add full-intensity blue directly to the dark background, producing an overpowering neon bar.
- Dark mode active line now uses `from_rgba_unmultiplied(10, 132, 255, 14)` (~5% opacity), matching macOS Swift `Theme.swift` reference.
- Corrected diff washes, occurrence pins, and selection row highlights.
- 38 unit tests and 30 doctests passing, clippy clean with -D warnings, rustfmt clean. Swift code intact.
[Parity audit & defect remediation plan](plans/swift-parity-and-defect-remediation.md).

**Typographic Baseline & Unified Line LayoutJob Remediation (2026-10-27):**
Resolved inconsistent vertical token jumping and text baseline misalignment within rendered source lines:
- Migrated code line formatting to `egui::text::LayoutJob` with section-based `TextFormat`.
- Eliminated per-token galley layouts where varying glyph bounding boxes caused individual tokens (`-`, `` `code` ``, `(`, `.`, `**`) to receive differing vertical offsets.
- Entire line is laid out as a single `Galley` via `code_painter.layout_job(job)`, locking all tokens across the line onto an identical typographic font baseline.
- 38 unit tests and 30 doctests passing, clippy clean with -D warnings, rustfmt clean. Swift code intact.
[Parity audit & defect remediation plan](plans/swift-parity-and-defect-remediation.md).

**Horizontal Scroll Viewport Clipping Remediation (2026-10-27):**
Resolved horizontal scroll blank area / cut-off text defect when scrolling right on long lines:
- Eliminated `.intersect(ui.max_rect())` on `viewport_clip`. In EGUI `ScrollArea`, `ui.max_rect()` translates with scroll offset (`-state.offset.x`), which caused the clip rect's right edge to retract leftwards and blank out newly visible text. Direct `ui.clip_rect()` strictly bounds against `SidePanel::right` while preserving full visible canvas width.
- Expanded `full_code_width` buffer for tab expansion and trailing comments.
- 38 unit tests and 30 doctests passing, clippy clean with -D warnings, rustfmt clean. Swift code intact.
[Parity audit & defect remediation plan](plans/swift-parity-and-defect-remediation.md).

**Line Rendering & Gutter Alignment Remediation (2026-10-27):**
Resolved line rendering alignment defects when opening files such as `AGENTS.md`:
- Line 1 Ascender Clipping: Increased `row_height` to 22.0 px (`Theme.sourceLineHeight: 22`). Replaced center-offset text positioning with baseline/top-relative row centering (`text_y = rect.min.y + ((row_height - galley.size().y) / 2.0).max(0.0)`). Added 4px vertical clearance to `code_clip_rect.min.y` to eliminate top scissor clipping.
- Gutter Row Box Artifacts: Fixed fragmented gutter cells by zeroing item spacing (`ui.spacing_mut().item_spacing = Vec2::ZERO`) inside `ScrollArea::show_rows` and rendering a single contiguous gutter column background fill and single continuous hairline right divider. Applied seamless canvas frame to `CentralPanel`.
- Dynamic Font Metrics: Monospace character width is measured dynamically via font metrics. Code text leading inset matches `Theme.sourceInset = 12`.
- 38 unit tests and 30 doctests passing, clippy clean with -D warnings, rustfmt clean. Swift code intact.
[Parity audit & defect remediation plan](plans/swift-parity-and-defect-remediation.md).

**macOS High-Fidelity Visual Parity Alignment (2026-10-27):**
Aligned the Rust + EGUI interface directly against the original macOS Swift screenshot:
- Theme Palette: Light mode refined to crisp `#FFFFFF` editor & gutter surfaces, `#F6F6F8` macOS sidebar,
  `#E5E5EA` hairline dividers, soft teal active line wash (`rgba(0, 122, 255, 0.05)`), and 2.5px vertical
  accent indicator bar at the caret line.
- Changes Sidebar: Replicated `ReviewChangeRow` two-line presentation (bold filename + muted directory +
  colored `+add −del` counters), status badges (`⊕`, `⊘`, `⊖`, `⤹`), and top PR summary card (`#num · Open · X files`
  with overall `+A −D` totals).
- Top Toolbar: Reorganized into grouped navigation (`< >` history, `↑ ↓` change block jump, `↶ Exit PR`),
  centered repository title & PR subtitle, segmented `[Normal | Critical]` pill, and right-aligned modal buttons.
- Breadcrumbs & Revision: Added `Modified` status indicator and segmented `Revision [Before | After]` pill.
- Gutter Folds & Diffs: Rendered collapsed folds as inline capsule badges with hidden line count (`▶ 8`, `▶ 5`),
  and 3px vertical color bars on the right edge of the gutter for modified lines.
- Context Inspector: Replicated macOS stacked view displaying Declaration (symbol, line link, signature) and
  Document Outline (with `ƒ(x)` icon, active dot `●`, and line span `X L`) together.
- Status Bar: Replicated left (`Rust · 815 lines`) and right (`ⓘ Index Ready · Line 564 · After · UTF-8 · Read Only`).
- 38 unit tests and 30 doctests passing, clippy clean with -D warnings, rustfmt clean. Swift code intact.
[Parity audit & defect remediation plan](plans/swift-parity-and-defect-remediation.md).

**Horizontal Scrolling, Line Wrap Toggle & SVG Vector System (2026-10-27):**
Resolved long-line overflow bleeding into the right inspector and completed 100% vector SVG
migration with zero emoji usage:
- Swift parity investigation: Swift disabled text container width tracking (`widthTracksTextView = false`)
  and enabled horizontal scrolling (`hasHorizontalScroller = true`) alongside a separate fixed
  gutter (`SourceContainerView.swift`).
- EGUI horizontal scrolling & clipping: Compute line width via `SourceBuffer::max_line_len()`,
  enabling egui's horizontal scrollbar. Pinned the gutter to the viewport's left edge (`viewport_clip.min.x`)
  and strictly clipped code rendering to `[viewport_clip.min.x + GUTTER_WIDTH, viewport_clip.max.x]`,
  mathematically preventing any text from bleeding over `SidePanel::right`.
- Configurable line wrapping: Added `wrap_lines: bool` to `SessionStore`, accessible via `View -> Wrap Lines`,
  toolbar menu, and `Ctrl+Alt+W` global shortcut.
- Vector SVG System: Embedded 18 curated SVG assets in `assets/icons/` with zero unused files.
  Created `src/ui/icons.rs` providing theme-adaptive tinting, button hover feedback, and direct painter
  rasterization. Replaced all emojis across toolbar, modals, sidebar, welcome view, inspector, and tests.
- 38 unit tests and 30 doctests passing, clippy clean with -D warnings, rustfmt clean. Swift code intact.
[Parity audit & defect remediation plan](plans/swift-parity-and-defect-remediation.md).

**SourceView Virtualization & Rendering Performance Fix (2026-10-27):**
Fixed severe UI performance degradation and text overflow bleeding into the right
context inspector. Replaced per-frame syntax highlighting and widget allocation over
all document lines with pre-cached syntax highlighting and row virtualization
(`ScrollArea::show_rows`), reducing render cost from hundreds of thousands of syntect
calls per second to $O(1)$ visible slice draws (~40 lines). Added strict painter
viewport clipping (`ui.painter().with_clip_rect`) preventing code lines from ever
overflowing into `SidePanel::right`. Implemented Welcome View with recent repository
links and keyboard shortcut hints.

**Swift Parity & Defect Remediation (Subsystems 1–4 Completed, 2026-10-27):**
Exhaustive feature audit and defect remediation pass validating every feature
from the original Swift codebase against the Rust + EGUI port across all 4 subsystems:
- Subsystem 1: Exact source/test classifiers, 5-tier fuzzy search with diacritic normalization, background streaming ripgrep search, folder picker dialog, and validated recent sessions.
- Subsystem 2: TSX and `.mts`/`.cts` grammar support, fold deduplication preserving outer blocks, multi-language test region detector, and virtualized SourceView rendering.
- Subsystem 3: SCIP wire tag fixes (enclosing range field 7, display name field 6, signature extraction), document-scoped local symbols (`SemanticSymbolID`), 4-tier workspace symbol ranking (Ctrl+T), deterministic 12-color caller palette, navigation history corruption fixes, and CLI flags (`--repository`, `--pull-request`, `--file`, `--verify-resources`).
- Subsystem 4: Critical Lens (`SCIPSymbolHeader`, `CriticalityProfiles`, `CriticalityOverrides`, exact precedence `CriticalityPolicy`, `SourceProjection` and receded rendering in `SourceMode::Critical`, `CountAnnotation`), Pull Requests (`UnifiedDiffParser::parse_file_changes`, `ReviewChangesPresentation` source filtering, change block navigation `previous_change_line`/`next_change_line` via Alt+Up/Down, PR sidebar changes tab), Bookmarks (`BookmarkScope` isolating checkout vs PR number, ContextInspector bookmark row click conflict fix and hover styling), and UI theme switching (dark/light mode toggle with dynamic re-highlighting).
38 unit tests and 29 doctests passing, clean clippy with zero warnings, rustfmt clean.
Original Swift source code is 100% preserved in place as requested.
[Parity audit & defect remediation plan](plans/swift-parity-and-defect-remediation.md).

**Rust & EGUI Linux Native Rewrite (Completed):** Standalone Linux-native
rewrite of Radiology in Rust using EGUI (`eframe` 0.27 / `egui` 0.27) on branch
`rust-egui-port`. Features complete architectural parity with the Swift original:
fast Git/filesystem discovery, `FileNode` tree with "Source Files Only" and
"Hide Tests" filters, fuzzy subsequence file search (Ctrl+K), multi-language syntax
highlighting with `syntect` and Tree-sitter outline/folding, asynchronous streaming
`ripgrep` search, pure-Rust zero-copy SCIP protobuf decoding with definition/reference
navigation and caller call trees with FNV-1a 12-color identity pills, persistent
repo/PR bookmarks with line-anchor relocation, and GitHub PR review with unified
diff parsing and Before/After viewing. Comprehensive RFC 1574 rust-doc-formatting
applied across all modules, types, and methods with 28 passing doctests and 0 doc
warnings. 26 unit tests passing, clean clippy, fully standalone, and ready for future
integration into `~/little-screen-clip`.
[Architecture and parity plan](plans/rust-egui-linux-port.md).

**Published to GitHub main (2026-10-11):** all accumulated work merged through
[PR #1](https://github.com/tqbf/radiology/pull/1), explicitly authorized by the
user after initially requesting a push. Feature commit `3b8f7f5`, merge `dfc877a`
at **14:53:25 CDT**; local main fast-forwarded to the confirmed GitHub merge.
The preceding 269-test/signed-build gates cover this snapshot. Publication docs
supersede earlier rounds' local/uncommitted statements.
[Scope, commits and validation](plans/github-publication.md).

**Repository/PR bookmarks (2026-10-11):** third independently scrolling,
resizable Context pane; ⌘D toggles the exact caret, ⌘0–⌘9 use the first ten
ordered bookmarks. Native drag and context-menu reordering persist per canonical
attached checkout/PR number, with checkout separate. Before/After and exact Back
are preserved; changed source uses bounded anchors or an explicit error.
Normal/Critical now use ⌥⌘1/⌥⌘2. Nested native splits pass both divider checks.
**269 tests /57 suites**, light/dark renders, check/build/strict signing/resources
pass; built **14:41:15 CDT**. Physical drag/menu/shortcut checks remain unverified;
desktop automation is unavailable. [Contract and acceptance](plans/bookmarks.md).

**Caller identity pills and cycle repair (2026-10-11):** each caller is a
clickable capsule colored from a fixed 12-color palette by a deterministic
method-name/full-file-path hash. Colors persist across branches/relaunches;
light/dark text hierarchy and native navigation remain. Recursive branches now
stop before rendering a repeated callable, checked by semantic identity and
canonical declaration location; distinct same-named functions remain separate.
**263 tests / 55 suites**, three-width light/dark renders, check/build/strict
signing/packaged resources pass; built **13:47:46 CDT**. Retried live automation
still reports missing `@oai/sky`; mouse checks remain unverified.
[Palette and cycle evidence](plans/caller-pills.md).

**Reference caller trees and resizable inspector (2026-10-11):** reference
disclosure traces the containing callable backward through exact SCIP identities
and parsed calls, preserving branches and marking recursion/unknown boundaries.
No depth cutoff; larger trees continue with Trace more callers. Native VSplitView
resizes independently scrolling References and Definition/Outline panes.
**260 tests / 54 suites** pass, including 270-level continuation, caller/Back
navigation and native divider resizing. Three-width light/dark component renders
pass; check/build/strict signing/packaged resources pass, built **13:31:01 CDT**.
Physical UI remains unverified because desktop automation failed to start.
[Contract and evidence](plans/reference-call-trees.md). This supersedes the
2026-10-08 outer scrolling layout below.

**Inspector refinement (2026-10-08):** stronger symbol title and
small section headings, compact top-stacked layout, exact-identity definition
deduplication, stronger snippet contrast and faint appearance-aware section
tints. Native PR758 verifies stacked layout, matching-name deduplication,
reference disclosure/Return/Back; three-width light/dark renders pass.
**253 tests / 52 suites**, check/build/sign/resources pass; built **22:59:05 CDT**.
One preexisting highlight timer test failed initially, then passed alone and in
the unchanged full rerun. Different-target native check remains unverified.
[Inspector contract and evidence](plans/inspector-refinement.md).

**Toolbar and Changes sidebar refinement (2026-10-08):** selected-
value Normal/Critical dropdown, Before/After beside it, menu-only source filters;
tab-like Changes/Files/Search without Browse; clearer filenames/status/counts,
selection, summary spacing and filter-aware footer. Native PR758 verifies layout,
mode/revision/filter controls, tab clicks and keyboard Files→Search. Three-width
light/dark row renders pass. **253 tests / 52 suites**, compile/build/sign/resources
pass; built **17:35:01 CDT**. Separate review app closed; user reader preserved.
[Current task and evidence](plans/sidebar-refinement.md).

**Source breadcrumbs (2026-10-08):** source header
reserves a fixed trailing area for change/collapse controls and compacts paths
from the beginning, retaining nearby directories and enclosing declarations
one at a time as space allows. File/final symbol get priority at tight widths;
full location/declaration jumps remain in the ellipsis menu. Live PR758 now shows
crates/harness-core/src/file/Nomex/admit_call; Before/After/menu and five-width
light/dark renders pass. **253 tests / 52 suites**, compile/build/sign/resources
pass; built **17:02:10 CDT**. User reader preserved; review app closed.
[Layout and validation](plans/source-breadcrumbs.md).

**Reader session restoration (2026-10-08):** normal relaunch reopens the attached
repository and last confirmed PR, resolving fresh metadata. Exit PR remembers
local checkout; CLI/manual choices win, and failed restores retain their saved
PR for retry. Real separate-instance quit/relaunch restored tkos #758 without
arguments; the current user's #758 record is saved for the first upgraded launch.
**253 tests / 52 suites**, check/build/sign/resources pass; built **16:36:48 CDT**.
[Lifecycle and acceptance](plans/session-restoration.md).

**Compact source gutter (2026-10-08):** user steered toolbar work
to narrow the existing 104-point gutter to **72 points** and print only every
fifth canonical line plus the actual caret line. Native draw/fold/scroll tests
pass; live PR758 Go to Line7 / Down to8 confirms sole non-fifth label updates.
Combined gates **246 tests / 51 suites**, build/sign/resources; signed
**16:10:23 CDT**. Source font unchanged; counts defer to tooltips when cramped.
[Rendering contract](plans/source-gutter.md).

**Toolbar grouping (2026-10-08):** native navigation/repository,
labeled PR context with compact refresh and Exit PR inside its menu, and viewing
tools with grouped filter/search actions. Duplicate PR subtitle removed; source
Before/After remains in the breadcrumb row. Native review verified grouping,
menus, filter binding, left-sidebar button/Ctrl-Cmd-S and Exit PR. Detail-scoped
toolbar avoids misplaced sidebar tools/duplicate toggle. Narrow resizing remains
unverified after two native drag errors. Same 246-test/signed gates above.
[Contract and review](plans/toolbar-layout.md).

**Kotlin workspace repair (2026-10-08):** user's real app exposed
a directory-symlink copy/hash failure missed by earlier manual compiler probes.
Directory/internal-file links, dangling external Markdown links and complete
tracked build inputs are repaired. The actual PR758 SemanticIndexService path
passes: **2,088 source documents, 18 references / 4 files**, generated with honest
Partial coverage, cache and cleanup verified. Gates **244 tests / 50 suites**,
compile/build/strict signing/resources; signed **01:10:35 CDT**. No physical UI
check/restart; relaunch and reopen PR758. See PROGRESS.md and
[the Kotlin contract](plans/kotlin-semantic-indexing.md).

**Kotlin semantic support (2026-10-07):** compiler-backed Gradle indexing now
supports AGP9 built-in Kotlin; the managed Kotlin2.4.20-compatible tool is
installed here. Disposable real-input builds, pre/post byte verification,
snapshot-only document publication, UTF-16 adapter defaults on generation/cache/
reuse, and persisted Partial build coverage preserve PR identity. Real PR758
After `addSecrecyTag` resolves to SecrecyCtx.kt:18 with 18 references in 4 files;
broader build emits 2,379 compiler documents with two native setup failures,
honestly Partial. Native-component PR caret/reference/definition/Back tests pass.
Gates **241 tests / 50 suites**, compile/build/strict signing/resources; signed
**18:36:00 PDT**. Physical UI remains unverified; relaunch is required to use
the new registry. [Current contract and exact evidence](plans/kotlin-semantic-indexing.md).

**Context inspector recovery (2026-10-07):** native drag-to-collapse now has a
scene-bound **View > Show Context Inspector / Hide Context Inspector** command
and **⌥⌘I** shortcut. Toolbar labels match the current presentation state;
recovery stays available without a source file. ⌃⌘I remains Reindex. Gates:
**234 tests / 47 suites**, compile/build, strict signing and packaged resources;
signed **17:47:35 PDT**. Physical drag/reopen remains unverified. See
[the current sidebar contract](plans/context-sidebar.md) and the newest
PROGRESS.md entry for validation and the manual drag/reopen checks.

The user confirmed on 2026-10-04 that the latest large-function folding fix works: “yeah you fixed it, you can settle at this point.” The recurring status heartbeat was deleted and must not be recreated unless the user asks. A subsequent request now authorizes the caret-driven Context sidebar described below.

The caret-driven **Context inspector** is implemented by Opus 5.5 and replaces the Outline/Symbol toggle. Its upper region tracks the enclosing declaration and references; its lower region shows the selected semantic definition or the outline. Entries are **one line by default**, with disclosure for signatures, documentation, and source context. The lower heading is **Definition**; “Under Caret” was removed. Selecting references stays compact; Return commits navigation. Source Return/Enter goes to definition, and Cmd-[ restores the exact departure caret even after scrolling. A subtle, steady source caret makes character-level keyboard movement visible. Gates after the caret round passed with **129 tests in 21 suites**. A faint shade under the caret's innermost function or method followed. Its gates pass: **132 tests in 22 suites**, `make check`, signed build, strict codesign, and packaged resource verification. GPT Sol verified the faint shade live in light mode on untouched tkos: correct function boundaries, outside-function clearing, fold collapse/expansion, passive-scroll stability, and readable caret/highlight layering. Dark-mode shading remains unverified; see the newest PROGRESS.md entry. GPT Sol verified the caret live on untouched tkos: arrows update Context without navigation, word selection and sidebar focus hide the caret, pinned tint does not obscure it, and Return/Back still works. The same pass confirmed concise default rows and the Definition heading. Full disclosure and narrow-width acceptance, the README availability-label check, and dark-mode caret appearance remain manual; see [the UI review](plans/context-ui-review.md) for evidence and exact checks. See [the implementation plan](plans/context-sidebar.md) and the newest PROGRESS.md entry. `/Users/tqbf/codebase/tk/tkos` remains read-only. Changes are local and uncommitted.

**Critical Lens (2026-10-04, Fable 5.1):** a `Normal | Critical` source-mode control (toolbar segmented control, View menu ⌘1/⌘2), a draw-time projection that recedes known runtime plumbing (stage one, verified live by GPT Sol; see [Sol's review](plans/critical-lens-ui-review.md)), and now function summaries, a bounded critical frontier with via paths and honest coverage, an automatic Critical section in the Context inspector, an automatic hover popover on callables, and reachable-critical counts in the fold gutter and outline (stages two and three). Call edges come only from parsed call sites resolved to SCIP identities; closures and nested functions own their own sites; unknown callees and closure-owned calls make counts partial, never a proven zero. Gates: `make check`, **161 tests in 32 suites**, signed build (2026-10-04 16:19:59), strict codesign, and packaged resource verification. Sol attempted the final native review, but attachment failed with `-10005: cgWindowNotFound` on both the initial attempt and the single permitted recovery. UI work stopped within the user's tooling limit. Implementation and model/build gates are complete; summaries, hover, and counts remain visually unverified. [Sol's review](plans/critical-lens-ui-review.md) contains the exact manual checks and screenshot requests; the final build is left available. See [the Critical Lens design](plans/critical-lens.md) and the newest PROGRESS.md entry.

**Critical rendering refinement (GPT Sol):** the user superseded the earlier protect-all-syntax behavior. Critical now aggressively dims non-call source and ordinary runtime calls; only actual parsed critical/unknown call-name tokens remain prominent. Unresolved syntactic call names stay visible without invented identities/edges. Alpha is 30% light / 38% dark; no layout/storage/navigation changes. Native same-viewport `app.rs:115–139 execute_future` comparison succeeded; the user independently accepted it (“this is much better”). Existing pins/line tint and concise upper call names/independent lower Definition were observed. Dark mode, new mark toggles, hover navigation and folded-count semantics remain unverified; shared-reader activity ended live work. Gates pass: **163 tests / 32 suites**, check/build/strict codesign/empty-env 8-grammar resources; signed binary **16:53:36**. See [the review](plans/critical-lens-ui-review.md) and [current rendering contract](plans/critical-lens.md). Earlier projection/contrast descriptions are historical and superseded.

**Latest Critical Lens change:** the user removed its sidebar depiction. The sidebar now behaves identically across Normal/Critical: concise declaration/references and Definition/outline, with existing disclosures/navigation. No Critical call/frontier/status section, lower summary or outline density counts remain. Sidebar-only work/state/components are removed; source hover and fold-gutter computation remain. Call-focused rendering, toolbar and marks are preserved. Actual gates: **164 tests / 32 suites**, check/build/strict codesign/empty-env resources (8 grammars), signed **17:05:05**. Live check not attempted to avoid disrupting the user's shared reader. Earlier sidebar feature descriptions above are historical and superseded; see [current contract](plans/critical-lens.md) and [evidence](plans/critical-lens-ui-review.md).

**Hover interaction repair:** navigable popup titles and whole rows are now Buttons instead of passive names with small trailing links. Stable content hosting, native full-popover tracking and a source-to-popup transit region preserve entry/loading. External targets stay unavailable; existing exact-caret navigation/Back is retained. Gated **166 tests / 33 suites**, check/build/strict codesign/resources (8 grammars), signed **17:21:39**. Physical click verification remains unverified: native tooling attached to the old shared reader even after one full-path recovery, so live work stopped. No sidebar lens content restored. See [native review and manual steps](plans/critical-lens-ui-review.md).

**Latest ordinary outline change:** all file callables, including nested functions/methods, appear first sorted largest-to-smallest by signature-to-end line span (source-order ties), with quiet one-line `n L` counts. Non-callables remain in a native hierarchy without duplicate functions. Cache keys include both paths as well as bytes/symbols. Original declaration navigation and Normal/Critical parity remain. Gates **169 tests / 34 suites**, check/build/strict signing/empty-env resources; signed **17:35:04**. Live outline verification remains manual because native attachment selected the user's older reader; no interaction was attempted. The previous hover physical-click limitation remains explicit. See [outline architecture](plans/structural-navigation.md), [handoff](plans/agent-handoff.md) and [review](plans/critical-lens-ui-review.md).

**Pull request review (2026-10-04, Fable 5.1; gated at 184 tests / 39 suites, signed 19:56:31 CDT; UI unverified, Sol acceptance pending):** the toolbar **Pull Request** menu lists the last 25 PRs (open/merged/closed) of the attached repository through read-only `gh`; ⌘P opens one by number (the old ⌘P file-picker alias is gone; ⌘K remains). After is the pinned head SHA checked out into an app-owned worktree; Before is GitHub's merge base, materialized per changed file. A **Changes** sidebar segment, file-tree and outline dots, green/red gutter bars and line washes, a per-file **Before | After** control (⌥⌘B), and ⌥⌘↓/↑ change navigation make up the UI. The After snapshot is indexed with per-strategy exact-bytes reuse (git blob digests, metadata format 3); mixed reused/regenerated runs are Partial; Before is syntax-only and never receives mapped semantics. Design, measured facts, identities, and acceptance cases: [plans/pull-request-review.md](plans/pull-request-review.md).

**PR acceptance repairs (GPT Sol, latest):** full pinned blob hunks prevent rename-path contamination; whole-repository dependency-context digests reject unproven sibling reuse; mixed reuse Partial survives cache reload/refresh; refresh preserves revision options; PR failures have native recovery alerts and file guidance uses Cmd-K. Gates **186 tests / 39 suites**, check/build/strict signing/empty-env resources; signed **20:07:43 CDT**. Real #724/#723/#725 backend probes and actual phone-provisioner After Rust generation (one document, 3,711 valid occurrences), matching reuse and context-change regeneration passed. Native attachment selected the user's older pinned reader; no UI actions/screenshots. Full PR UI/GUI cancellation and previous hover/outline limits remain open. [Acceptance evidence/manual checklist](plans/pull-request-ui-review.md); [current contract](plans/pull-request-review.md). Earlier project-local 15/16 reuse estimate is superseded by conservative repository-context policy.

**Latest PR affordance:** visible **Exit PR** (title and return icon) beside the selector while active/loading, using existing exit/cancel to local checkout, with pending error cleanup. No Git branch switch or misleading main label. Gates **186/39**, check/build/strict signing/env-i resources; signed **20:18:12 CDT**. No native attempt; manual check recorded in [PR UI review](plans/pull-request-ui-review.md).

**Source filters (2026-10-04, Opus 5.5; model/build gates passed; UI unverified, GPT Sol acceptance pending):** two independent, persisted toggles beside the Files filter field and in the View menu. **Hide Tests** (⌃⌘T) hides recognized test files/directories from the Files tree. It also hides syntax-detected test regions inside ordinary source as a fold-union projection: a dimmed header line, a gutter disclosure with a hidden-line count, and a quiet clickable label. Detection covers Rust `cfg(test)`/test attributes (three-valued cfg logic, mid-`impl` helpers), Kotlin annotations/test classes/Kotest, TS/TSX/JS framework calls with binding checks and Vitest in-source blocks, and best-effort Python `TestCase`/pytest. Explicit navigation reveals the region; Back stays exact; hidden declarations leave the outline. **Source Files Only** (⌃⌘O) prunes the tree to code/headers outside artifacts, dependencies and generated output. ⌘K, Search, Changes, Go to Symbol and indexing still cover every file. Gates: **210 tests / 43 suites** (including 3 opt-in tkos probes that skip by default; run separately and passing), check/build/strict codesign/empty-env resources; signed **23:16:01 CDT**. Real tkos probe: app.rs 35–43, 803–833 (834 production visible), 3424–EOF; the whole-repository sweep found no false positives among the non-Rust hits. Design and limits: [plans/source-filters.md](plans/source-filters.md); Sol's acceptance: [plans/source-filters-ui-review.md](plans/source-filters-ui-review.md).

**Source-filter acceptance repairs (latest, GPT Sol):** narrow pytest test_ detection, preserve Vitest production else, refresh label hit identities at unchanged geometry, reject hidden headers inside outer folds. Actual AppKit/detector regressions pass; **213 tests / 43 suites**, check/build/strict signing/env-i resources, signed **23:23:22 CDT**. Two targeted real-tkos probes rerun successfully. Native attachment/recovery selected older #691 reader; no interaction/screenshots, visual acceptance still open. [Review/manual checks](plans/source-filters-ui-review.md), [current contract](plans/source-filters.md).

**Latest source-filter scope fix (2026-10-05, GPT Sol):** PR Changes now shares both SourceFilters options with Files; canonical diffs remain full. Rename/copy retained if either available path passes; deleted uses Before. Compact controls, visible counts/totals and filtered-empty reset; selection/filter changes preserve current reader/revision/history. Gates **216/44**, check/build/strict signing/resources/diff check; signed **2026-10-05 19:00:13 PDT**. No native retry; [scope/review](plans/source-filters-ui-review.md) and [PR contract](plans/pull-request-review.md). Earlier Changes-unfiltered descriptions are superseded.

**Latest PR source affordance (2026-10-06, GPT Sol):** previous/next changed-block icon buttons beside Before/After, shared directional availability with ⌥⌘↑/↓. Skip current hunk, nearest from gaps, no wrap; exact revision/content guards, reachable empty-side EOF anchors, observable genuine-caret updates and exact Back departure. Breadcrumbs scroll without consuming reader height. Gates **220/45**, check/build/sign/resources/diff; signed **14:22:51 PDT**. Native attachment/recovery selected older #753 reader; no interaction/screenshots. [PR contract](plans/pull-request-review.md), [UI/manual evidence](plans/pull-request-ui-review.md).

**Latest PR default visibility (2026-10-06, GPT Sol):** changed function/block folds and ancestors initially open once matching exact-document syntax/hunks are available. Header/attribute/decorator ownership is parsed; sibling closures stay folded. Manual choices (including pending diff and new bulk commands) win; fresh coordinators don't replay old commands. Hide Tests stays hidden; late layout preserves viewport/selection, discards obsolete navigation markers and yields to newer jumps. Gates **227/46**, check/build/strict signing/resources/diff; signed **14:47:45 PDT**. No native launch/attempt; component proof and exact manual PR anchors in [review](plans/pull-request-ui-review.md), [fold contract](plans/structural-navigation.md), [PR contract](plans/pull-request-review.md).

**Latest PR refresh (2026-10-06, GPT Sol):** visible Refresh PR / ⌘R fetches current metadata/head/merge-base while old reader stays usable; Refresh List separate, repository reload outside PR. Same head+base is metadata-only. Changed snapshot prepared atomically with protected worktrees, latest reader (including pending source load), exact-byte/Git-line restoration, proven renames and honest revision/missing fallback. SHA-old trails/data cleared, byte-proven current pins/options retained; failures retry refresh intent, cancellation/stale requests drop. Gates **234/47**, check/build/strict signing/resources/diff; signed **20:52:31 PDT**. No live attempt/remote refresh; [PR contract](plans/pull-request-review.md), [UI/manual evidence](plans/pull-request-ui-review.md).

The app icon now uses the supplied root `RADIO.png`; Make regenerates all macOS sizes when that image or the icon script changes. The icon-stage signed bundle was rebuilt at **16:42:51** (preserved in the subsequent call-focused build) and registered with LaunchServices. Compile, 161 tests/32 suites, signing, packaged resources, and decoded icon variants pass; application source and the final Critical Lens UI acceptance status are unchanged. See [the build guide](plans/build-system.md) and the newest PROGRESS.md entry.

Delivered behavior includes repository opening/recent/drop/CLI launch, Git-aware discovery, read-only TextKit source, ripgrep search, exact history, Tree-sitter structure for eight grammars, structural navigation and folding, fuzzy file lookup, and app-owned SCIP generation/navigation. The current fold gutter is 104 points wide with separate 28-point disclosure, exact hidden-line count, and canonical line-number lanes; the earlier 68-point/faint 2-point meter was superseded. Large-fold membership is binary-searched and glyph/layout invalidation is limited to changed ranges. Ordinary and locked occurrence highlighting coexist with native selection; a function-local Tree-sitter fallback covers parsed occurrences absent from SCIP without becoming a semantic guess.

The fresh checkout/build gate on SHEFFIELD (2026-10-04, macOS 26.5 / Swift 6.3.3) passed `make check`, 97 tests in 16 suites, a signed build, strict code-sign verification, and packaged verification of eight grammars/31 query files. Bundled builds now explicitly select SwiftPM's `swiftbuild` backend, matching the successful Swift 6.4 build on ELSTON; see [the build guide](plans/build-system.md). The user’s earlier acceptance is separate evidence that the reported fold interaction is fixed. See PROGRESS.md and the development history for the exact boundary.

## Required reading order

1. [README.md](README.md) — product usage, requirements, commands, shortcuts, supported languages/indexers, caches, and troubleshooting.
2. [PROGRESS.md](PROGRESS.md) — newest-first evidence log, including user acceptance and historical validation counts.
3. [plans/agent-handoff.md](plans/agent-handoff.md) — architecture map, invariants, key files, validation, gaps, and working constraints.
4. [plans/development-history.md](plans/development-history.md) — chronological development rounds and superseded behavior.
5. The subsystem document relevant to the next task.

## Subsystem documentation

- [plans/shared-font-fallbacks.md](plans/shared-font-fallbacks.md) — portable host-matching Fira Sans/CJK/emoji assets, optional egui registration, compact standalone typography and native selection/copy/scaling evidence.

- [plans/rust-file-manager-reveal.md](plans/rust-file-manager-reveal.md) — shared file/folder context action, scoped desktop handoff, bounded RPC ownership and validation.

- [plans/rust-normalized-revision-retention.md](plans/rust-normalized-revision-retention.md) — bounded recognized PR semantic records, shared writer ownership, safe deferred cleanup and open-reader behavior.

- [plans/rust-real-indexer-acceptance.md](plans/rust-real-indexer-acceptance.md) — actual Rust, TypeScript, Go, Python, Kotlin/Java and C/C++ tools through shared generation/navigation/reuse/reload acceptance.

- [plans/rust-critical-projection-reuse.md](plans/rust-critical-projection-reuse.md) — exact-source LRU projections, scoped classification reuse, bounds and retirement.

- [plans/rust-project-cache-maintenance.md](plans/rust-project-cache-maintenance.md) — repository namespaces, cooperative leases, legacy migration and safe retired-record cleanup.

- [plans/rust-feature-configuration.md](plans/rust-feature-configuration.md) — manifest feature cfg parity, real rust-analyzer coverage/navigation, negative control and native acceptance.

- [plans/rust-project-index-reuse.md](plans/rust-project-index-reuse.md) — version-qualified whole-context project reuse, staged cache publication and origin provenance.

- [plans/rust-tool-provisioning.md](plans/rust-tool-provisioning.md) — explicit pinned tool installation, staged verification/rollback, cancellation, automatic indexing and real TypeScript coordinate/cache acceptance.
- [plans/rust-reader-notices-controls.md](plans/rust-reader-notices-controls.md) — compact error/details presentation and revision/change control availability.
- [plans/rust-pr-refresh-restoration.md](plans/rust-pr-refresh-restoration.md) — atomic changed-commit refresh, rename/line mapping, latest reader state and acceptance.
- [plans/rust-pinned-index.md](plans/rust-pinned-index.md) — pinned After generation, cache identity, isolated inputs and revision-scoped publication.
- [plans/rust-pr-tree-cache.md](plans/rust-pr-tree-cache.md) — exact PR tree reuse, snapshot leases, bounded retention, automated evidence and pending native acceptance.
- [plans/source-breadcrumbs.md](plans/source-breadcrumbs.md) — adaptive path/declaration retention, fixed revision area, type hierarchy, and visual acceptance.
- [plans/session-restoration.md](plans/session-restoration.md) — last repository/PR persistence, startup precedence, failure recovery, and real relaunch acceptance.
- [plans/source-gutter.md](plans/source-gutter.md) — compact gutter, fifth-line/caret numbering, count fitting, typography and native draw verification.
- [plans/toolbar-layout.md](plans/toolbar-layout.md) — native toolbar groups, repository/PR/search menus, type hierarchy and visual acceptance.
- [plans/kotlin-semantic-indexing.md](plans/kotlin-semantic-indexing.md) — requested Kotlin compiler-backed indexing, Android Gradle compatibility, read-only isolation, and validation.
- [plans/architecture.md](plans/architecture.md) — layers, actors, canonical coordinates, and lifecycle.
- [plans/design-system.md](plans/design-system.md) — native macOS layout, type roles, color, source and gutter presentation.
- [plans/build-system.md](plans/build-system.md) — SwiftPM, bundle assembly, signing, optional release pipeline, and external tools.
- [plans/tree-sitter.md](plans/tree-sitter.md) — grammar versions, query resources, parsing contract, fallbacks, and syntax limits.
- [plans/structural-navigation.md](plans/structural-navigation.md) — outline synchronization, breadcrumbs, folding, reveal behavior, gutter geometry, and performance.
- [plans/context-sidebar.md](plans/context-sidebar.md) — caret-driven Context inspector: identities, reference preview/commit, caret-vs-viewport contract, caches, validation, real tkos results, and limits.
- [plans/context-ui-review.md](plans/context-ui-review.md) — GPT Sol's native UI/design evidence, initial defects, final repaired-build recheck, screenshots, and remaining manual checks.
- [plans/critical-lens.md](plans/critical-lens.md) — Critical Lens: measured SCIP/Tree-sitter facts, architecture and API boundaries, classification precedence and profiles, override persistence, draw-time projection, implemented summaries/call graph/frontier/hover/gutter contracts, validation, and limits. See [plans/critical-lens-ui-review.md](plans/critical-lens-ui-review.md) for native stage-one evidence, screenshots, contrast judgment and coverage limits.
- [plans/scip-semantic-navigation.md](plans/scip-semantic-navigation.md) — SCIP decoder/model, strategy discovery, adapters, provisioning, cache/freshness, revision indexes and exact-bytes reuse, navigation, occurrence highlighting, and limits.
- [plans/pull-request-review.md](plans/pull-request-review.md) — pull request review: measured tkos/gh/git facts, the strategy-level index reuse assessment, identities (attached root vs. snapshot roots, revisions on locations), components, minimal UI, behavior contract, sequence, and acceptance cases.
- [plans/scip-ui-handoff.md](plans/scip-ui-handoff.md) — historical first-cut UI test handoff; consult current PROGRESS/handoff before treating its open items as current.
- [plans/source-filters.md](plans/source-filters.md) — Hide Tests and Source Files Only: options/persistence/scope, path classifiers, per-language Tree-sitter test detection, fold-union presentation, reveal/navigation/identity contract, tests and limits. Sol's live acceptance: [plans/source-filters-ui-review.md](plans/source-filters-ui-review.md).
- [plans/quick-file-picker.md](plans/quick-file-picker.md) — Cmd-K/Cmd-P search/ranking/lifecycle.
- [Sources/GrammarScanners/PROVENANCE.md](Sources/GrammarScanners/PROVENANCE.md) — vendored JavaScript/Python scanner sources.
- [radiology.plan.md](radiology.plan.md) — original long-form product specification; useful direction, not current completion status.
- [treesitter.plan.md](treesitter.plan.md) — original Swift-first Tree-sitter request, preserved as history.
- [plans/roadmap.md](plans/roadmap.md) — delivered scope and possible future slices.
- [SWIFTUI-RULES.md](SWIFTUI-RULES.md), [PROBLEMS.md](PROBLEMS.md) — inherited implementation lessons and pitfalls.
- [plans/renaming.md](plans/renaming.md) — historical template rename notes; not an onboarding step.

## Maintenance rules

- Keep this file the master index; put deep documentation under `plans/` and update PROGRESS.md after work.
- Read and follow AGENTS.md. Use `swiftui-pro` before and after code decisions, `macos-design` for UI decisions, and `typography-designer` whenever setting type. A documentation-only description of existing behavior does not authorize redesign.
- Preserve canonical UTF-8 byte locations and convert only at boundaries. Do not make semantic navigation guess from spelling.
- Never edit the viewed repository during indexing tests. In particular, do not vendor dependencies into tkos or synthesize shadow manifests to conceal real project failures.
- Do not present old test totals as current. Label every recorded count/timing by its development round and say when no fresh test run was performed.
