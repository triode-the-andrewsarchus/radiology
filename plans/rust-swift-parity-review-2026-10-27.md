# Rust port versus Swift: parity review, 2026-10-27

## Verdict and scope

**The Rust port is a useful standalone repository-browser foundation, but it does not have feature or interaction parity with the Swift application.** Several features described as complete are implemented only as library helpers, approximated visually, or connected to the wrong data. The earlier “full parity,” “completed,” and “exhaustive audit” statements are superseded by this review.

Reviewed snapshot: branch `rust-egui-port`, HEAD `fd421e1`. The working tree was clean before review. This review changes documentation only; it does not repair the findings.

The comparison uses the current Swift implementation and its tests as the behavioral reference, tracing Rust entry points, state transitions, UI actions, and helpers. It includes Rust builds/tests, headless egui input/render probes, and read-only inspection of `../little-screen-clip`. No live GitHub PR fetch was needed. Swift/AppKit cannot run on this Linux environment, and no side-by-side desktop screenshot acceptance was performed. Visual conclusions below concern implemented layout and interaction contracts, not certified pixel equivalence. The requested `swiftui-pro`, `typography-designer`, and `macos-design` skills were not found in local skill/plugin directories; no UI implementation was attempted.

**Product direction from the user:** keep Radiology an independent app now; prepare for eventual integration into `~/Documents/git/local/little-screen-clip`, favoring substantial reuse, useful handoffs, and compatible broad UI conventions. Preserve Swift's capabilities and workflow, while aligning Linux chrome with little-screen-clip. Exact imitation of macOS controls is not the acceptance criterion.

Status vocabulary: **Present** means a connected implementation exists, without asserting exhaustive equivalence; **Partial** means a meaningful implementation has gaps; **Missing** means the user-facing contract is absent; **Broken** means a reachable path can produce incorrect behavior.

## Feature and interaction matrix

| Capability | Rust status | Evidence / remaining gap |
|---|---|---|
| Standalone Linux window and executable | Present | `src/main.rs`; egui/eframe 0.27, no little-screen-clip runtime dependency. |
| Git discovery and filesystem fallback | Partial | `src/repository/discovery.rs`; connected, but synchronously invoked on UI thread. |
| Files hierarchy and expansion | Present | `src/repository/file_node.rs`, `src/ui/sidebar.rs`. |
| Source Files Only / test-path filtering | Partial | Classifiers and filters are connected to Files; persistence, Changes filtering, and empty-state recovery differ. |
| Files sidebar text filter | Missing | `SourceFilters.path_query` exists, but Files has no input for it. |
| Quick file picker | Present | `src/repository/quick_search.rs`, `src/ui/modals.rs`; five-tier matcher, keyboard and mouse opening, searches all discovered files. |
| Read-only UTF-8 loading, binary/16 MiB rejection | Partial | `src/source/buffer.rs` validates; UI silently swallows failures and reads full file before size validation. |
| Syntax coloring | Partial | Syntect is connected; TS/TSX may use JS fallback and TOML an INI fallback; not Swift's query-based highlighting. |
| Rust/Python/Go/JS/TS structure | Partial | Tree-sitter extraction exists, with a smaller set of node kinds; declaration signatures are always `None`. |
| TSX/JSX outline and folding | Broken | Grammar selected, but `parse_node` has no TSX cases. Probe returns zero symbols/folds. |
| Swift/Kotlin structural support | Missing | No native grammars. Swift can get single-line fallback symbols, without actual body ranges; Kotlin `fun` is not recognized by fallback. |
| Manual folding / Collapse Functions | Partial | Connected; lacks Swift's general fold-query coverage, Fold to Definitions command, default presentation, destination reveal, and per-file preservation. |
| Hide Tests inside source | Missing | Detector is unused by UI; detector itself uses textual heuristics rather than AST analysis. |
| Caret and source navigation | Broken | Click / Up / Down exist; exact target column is lost, keyboard handling is global, and folded destinations remain hidden. |
| Text selection and Copy | Missing | Painter rows have no selection range, drag-selection handling, or selected-source copy. Copy Line exists. |
| Find in File / Next / Previous | Missing | No equivalent of Swift's NSTextFinder commands. |
| Adjustable source text size | Missing | No Increase/Decrease Source Text Size behavior. |
| Horizontal scrolling and viewport clipping | Present | Dedicated gutter and clipped code painter; existing fixes retained, without screenshot certification here. |
| Line wrapping | Broken | Galley can wrap, but virtualization and hit testing still assume one 22-point row per canonical line. |
| Back / Forward | Broken | Helper works; app discards revision, loses target column, and inserts an extra file-start entry on cross-file jumps. |
| Repository text search | Partial | Background streaming rg works; errors/cap indicator ignored, stop and glob controls absent, no live option/query refresh. |
| Existing SCIP index loading | Partial | Only `index.scip` / `.scip` at repository root; no provenance/freshness validation. |
| Automatic project detection / SCIP generation | Missing | No indexer service, provisioning, multi-project merge/cache, Reindex or Cancel UI. |
| Definition navigation and symbol search | Broken / Partial | Global-symbol search exists; local identity lost in app lookups, encoding ignored, definition column dropped. |
| Explicit Find References | Missing | No source action / command equivalent; callers tab is a different feature. |
| References and call trees | Partial | SCIP reference traversal and deterministic pill hashing exist; inaccurate textual fallback, hard depth cap, no parsed-call validation or disclosure/snippets. |
| Definition previews | Missing | Metadata/signature display exists for indexed symbols; source snippet preview/alternatives and independent under-caret region do not. |
| Three-region Context inspector | Missing | Rust has Symbol/Callers/Outline/Bookmarks tabs instead of simultaneously visible resizable regions. |
| Outline ordering and line counts | Broken | Source-order hierarchy, heuristic byte-count/30 display, no function-first size ordering or keyboard preview/commit. |
| Critical Lens policy helpers | Present | `src/critical/lens.rs`; not used to drive app rendering. |
| Critical Lens product behavior | Missing | Renderer dims by syntax color; no semantic projection, persisted user marks UI, frontier/hover summaries, or critical fold counts. |
| Bookmark creation / ordering UI | Partial | Connected in inspector and shortcuts; toolbar changes and reorder/delete are not saved. |
| Bookmark anchor relocation | Broken | `SourceBookmark::resolve` exists but is never called on app navigation. |
| Checkout versus PR bookmark scope | Broken | Store supports scope, but PR transitions do not load/swap it; shortcut save can write checkout bookmarks into PR scope. |
| PR number prompt / metadata / changed-file rows | Partial | `gh pr view` / `gh pr diff` connected, but synchronous, no useful failure/loading state. |
| Recent PR chooser and Refresh List | Missing | Listing helper unused; also lacks `--state all` for Swift's open/merged/closed list. |
| Pinned PR After / merge-base Before source | Broken | Reads attached checkout for both; no review object store/worktree or merge-base preparation. |
| Added / deleted / renamed / binary review handling | Partial | Metadata parsed; no correct revision-specific source routing or unavailable-revision UI. |
| File-specific change washes and block navigation | Broken | Whole PR diff is one document's hunk set; contextual hunk headers treated as exact changed lines; click paths do not scroll. |
| PR refresh with restoration and stale-history invalidation | Missing | Replaces metadata/diff only; no snapshot identity, line mapping, rename restoration, or transactional failure behavior. |
| Before semantic isolation / After indexing | Missing | Current optional index used irrespective of revision; no review After index. |
| Repo / PR session restoration | Partial | Repository recording works; PR setter unused, filters/wrap not saved on change, launch restoration precedes explicit CLI root. |
| Toolbar menus, side panel toggles, breadcrumbs | Partial | Controls are connected, but PR list missing; enabled states and shortcut behavior inconsistent. |
| Error, loading, parse/index degradation feedback | Missing / Partial | Most app errors discarded; footer has boolean “Index Ready” or unconditional “Ripgrep ready.” |
| Light/dark, fonts, accessibility | Partial | Hardcoded palettes and scattered text sizes; no shared host appearance/font setup or verified keyboard/accessibility equivalence. |

No completion percentage is assigned: a button, helper, and complete workflow should not count equally, and runtime acceptance remains outstanding.

## Findings requiring repair

### R1 — P1: PR review displays checkout content as review content

Rust `src/ui/app.rs:277` constructs every file path with `repository_root.join(rel_path)`. PR entry only updates summary/diff state (`app.rs:903`); it never prepares a head snapshot and does not initialize `active_revision` to After. Toolbar revision changes reread the same checkout; breadcrumb/shortcut changes only change a state field. Consequently added/deleted PR files can fail to open, and a Before/After label cannot establish which bytes are shown.

Swift's `AppModel+PullRequests.swift` routes locations through `resolveRevision` and `sourceRoot`; `PullRequest/ReviewRepository.swift` owns pinned objects and review worktrees. Implement that snapshot contract before considering the PR UI usable for review. Keep the attached checkout untouched. Gate unavailable revisions explicitly. The temporary probe confirmed that opening with `Before` reads checkout bytes.

### R2 — P1: diff data is neither file-specific nor guaranteed to represent changed lines

All PR entry/refresh routes pass a complete `gh pr diff` into `UnifiedDiffParser::parse` and assign the result to the current document (`app.rs:910`, `app.rs:970`, `src/main.rs:131`). `parse` collects every `@@` header without path association (`src/pull_request/diff.rs:145`). Any binary file makes this aggregate “binary.” Opening another file does not recompute `line_changes`; local checkout diffs can also remain from the previously opened file.

The parser assumes `-U0`, while `get_pr_diff` requests no zero-context diff (`src/pull_request/github.rs:118`). Header counts therefore include context and are painted as changes. Probe: one contextual replacement in file A plus one replacement in B yields current-document ranges `[1..4, 20..21]` and four “additions.” Store hunks by path and exact revision; obtain zero-context snapshot diffs or parse actual added/deleted lines correctly. Swift's `ReviewHighlights` additionally keys overlays by source-content identity.

### R3 — P1: repository changes retain state from the previous repository

`open_repository` (`app.rs:203`) never clears the previous index, history, pins, review/revision, search, and other document state before discovery. A root with no index retains the previous root's index; a failed/empty discovery can retain old source. Branch state can remain from a previous Git checkout. Pending search results have no repository-generation publication guard.

Probe confirmed old index, pins, Before revision, and Back history after switching between two isolated directories. Introduce one complete repository/session transition with cancellation and generation identity. Swift explicitly cancels tasks and clears state in `AppModel.swift` opening/reset paths.

### R4 — P1: indexing lifecycle is unported

`app.rs:235` only attempts to decode preexisting root files. There is no counterpart to Swift's `Semantic/Indexing/`: Rust/Go/TS/Python/C-family/Kotlin project strategies, tool availability/provisioning, background execution, bounded diagnostics, index-cache provenance, partial/stale states, Reindex, or cancellation. “Index Ready” is merely `Option::is_some` (`src/ui/source_view.rs:625`). “Ripgrep ready” does not check whether rg exists.

Port the service and observable status contract. Existing SCIP decoding is a foundation, not indexing parity. Before must remain syntax-only; After must use the exact review snapshot's index.

### R5 — P1: local semantic identities and column encodings are lost

App definition/metadata queries use raw strings (`app.rs:638`, `app.rs:1186`, `app.rs:1305`). `find_definition` and `symbol_info` synthesize `document: None` (`src/semantic/index.rs:508`), although local symbols are stored with document scope. The scoped helper succeeds in the probe, while the app's raw-string lookup fails for the same `local 1`.

SCIP document/metadata column encoding is not retained by the Rust schema; character counts go straight into indexed column comparisons. Swift preserves UTF-8/UTF-16/UTF-32 encodings and maps against exact source bytes (`SemanticIndex.swift`, `DocumentSemantics.swift`). `occurrence_at` also includes `end_col` (`index.rs:604`), confirmed to hit column 3 for range `0..3`, and returns the first rather than narrowest hit. Preserve scoped identity and encoding end to end; use half-open ranges and exact definition columns.

### R6 — P1: Critical Lens is a different feature in the current UI

`src/ui/source_view.rs:523` decides criticality from a token's color and dims other tokens. It never consumes `CriticalityPolicy`, `SourceProjection`, or `CriticalityOverrides`; those appear only in library code/tests. Syntax theme colors do not identify repository calls, FFI, system APIs, or user marks.

Swift drives presentation from semantic classification, parsed call sites, source projection, overrides, frontier/hover summaries, and fold annotations (`AppModel+CriticalLens.swift`, `Critical/`, `CriticalHoverView.swift`). Connect and validate the existing Rust policy before extending it; port the missing graph/summary/UI contracts. Do not mark color dimming as lens parity.

### R7 — P1: navigation fails to restore or reveal exact destinations

`navigate_to_location` (`app.rs:313`) sets `caret_col = 0` regardless of byte target; probe for byte 5 leaves effective caret at byte 0. Back/Forward callers discard `SourceLocation.revision`. Cross-file navigation first records file byte 0, then the real target, adding an intermediate history stop. Programmatic navigation does not unfold ancestors; the probe's line-2 destination remains hidden under a line-1 fold.

Toolbar/breadcrumb change buttons (`app.rs:1000`, `app.rs:1322`) update only `caret_line`, omitting viewport scroll, history, column, and semantic context; keyboard paths use another mechanism. Previous-button anchor comparison can select the current block when already inside it. Deletion-only/addition-only zero-width changes also need equivalent keyboard/button behavior. Reuse a single exact-location navigation operation for all actions; preserve the departure, revision, target column, fold reveal, and viewport.

### R8 — P1: keyboard input crosses focus boundaries and handlers conflict

`handle_shortcuts` (`app.rs:555`) tests broad modifiers without consuming events. Ctrl+O is handled twice. Theme Ctrl+Alt+T is handled twice and also matches Ctrl+T; the headless probe confirms unchanged theme plus an opened symbol modal. Ctrl+1/2 changes source mode and also activates bookmark slot 1/2. Bare Return definition handling runs globally, including when a text field owns focus.

`SourceView::show` independently reacts to Up/Down/Enter without checking focus/modifiers (`source_view.rs:189`). Probe confirmed ArrowDown moves source with a text field focused. Modal arrows can consequently affect source; Alt+Arrow can enter both ordinary caret and change navigation paths. Implement exact modifier matching, explicit focus ownership, and consumed commands shared by menus/buttons/keys. Do not interpret Return in search or a modal as a source command.

### R9 — P1: bookmark workflows bypass persistence, relocation, and scopes

Only Ctrl+D calls `bookmarks.save` (`app.rs:773`). Toolbar toggle, inspector removal, and reorder only mutate memory (`app.rs:1022`, `app.rs:1243`). `SourceBookmark::resolve` is called only in tests; navigation uses obsolete stored offsets. PR entry/exit never swaps the loaded checkout store for the PR scope; saving with Ctrl+D can write that mixed store under the PR number. Bookmark revision is discarded during navigation.

Swift's `AppModel+Bookmarks.swift` and `Bookmarks/BookmarkModel.swift` make the complete scope/resolution/save path explicit. Reuse the Rust resolver and scope helpers, invoking them from every relevant UI route. Surface ambiguous/deleted anchors instead of silently choosing stale coordinates.

### R10 — P1: syntax coverage and test hiding are materially incomplete

`src/syntax/language.rs` has no Swift/Kotlin native grammar. TSX has a grammar but no matching cases in `SymbolExtractor::parse_node` (`src/syntax/outline.rs:210`). Probes: TS function → one symbol/fold; equivalent TSX → none; Swift function → header-only fallback symbol, no fold; Kotlin function → none. Signature `detail` is always `None` (`outline.rs:271`). Rust attributes, declaration headers, and general control-flow folds need Swift fixture equivalence, not only a basic function fixture.

`TestRegionDetector` is never called by app/source view. Its claims of AST-aware detection are inaccurate: it scans lines and counts delimiters, misses Swift's cfg predicate analysis and binding-aware JS/Python/Kotlin rules, and can count braces inside strings/comments. Hide Tests currently changes file rows only. Reuse the Swift fixtures and query resources where grammar compatibility permits; port syntax-aware region detection, hidden-region presentation, navigation reveal, and outline filtering.

### R11 — P1: source reading and analysis can block the UI

Opening a repository runs Git/discovery/index decoding synchronously; opening a file reads it, parses it twice for outline/folds, and highlights the whole file on the UI thread (`app.rs:203`, `app.rs:275`). PR calls block the update loop. Every caret refresh can run synchronous repository-wide rg (`app.rs:404`, `app.rs:491`), even when an existing semantic lookup simply found no callers.

Source row painting is virtualized, but `visible_lines` scans every line and checks fold ranges and `max_line_len` scans source each frame (`source_view.rs:228`, `source/buffer.rs:155`). Thus the previous “O(1) visible slice” claim is too broad. Move expensive work to cancellable bounded jobs, cache immutable layout facts, and publish only matching source/repository generations. Benchmark large files, long lines, and large caller sets.

### R12 — P1: “Callers” can be ordinary text matches

`refresh_symbol_context` prefers the token under the caret, unlike Swift's independent enclosing-declaration references. It guesses an enclosing symbol through name-only workspace search and falls back to rg whenever semantic callers are empty. `find_callers_fallback` includes comments, strings, and non-call references, guesses caller names from line text, and truncates at 30 with no coverage notice. An indexed function with genuinely no incoming calls can therefore show misleading “callers.”

`CallerTreeBuilder` traverses references without parsed call-site validation, picks the first enclosing definition rather than necessarily the innermost, and silently stops at depth four. Swift's `ReferenceCallTreeWorker.swift` uses scoped identities, parsed calls, request-local cancellation, a row budget/expansion contract, and explicit boundary notices. Preserve the correct deterministic FNV palette implementation; replace the semantic assumptions. Any retained textual fallback must be labeled text matches, separate from semantic callers.

### R13 — P2: the inspector loses the defining Swift workflow

Swift `UI/ContextInspectorView.swift` has three simultaneously available regions: declaration/references; under-caret definition or outline; bookmarks. Native vertical dividers resize independently scrolling content. Rust `src/ui/context_inspector.rs:92` switches among four tabs; Symbol also contains outline, but references and bookmarks disappear when another tab is active.

Port the simultaneous-region information architecture with Linux-compatible splitters. Keep enclosing declaration and token-under-caret state independent. Add expandable reference snippets/caller trees, definition previews/alternatives, explicit references/Follow Caret, independent selection, keyboard preview/commit, and coverage/unavailable notices. This is structural UI parity work, not a cosmetic tab rename.

### R14 — P2: source selection, find, caret affordances, and wrapping need completion

Source is a custom painter with click rows, not a read-only selectable text surface. It lacks drag/Shift selection and original-byte copy, local find controls, horizontal/Home/End caret navigation, source font scaling, function-body shade, and destination feedback. The accent strip marks an active row, not Swift's insertion caret. A right-click definition action uses the old caret, without first resolving the clicked source location.

Double-click pinning searches every substring in the whole file (`source_view.rs:465`), including larger identifiers and unrelated scopes, and pins survive file switches. Swift pins are identity/byte-scoped, with semantic or function-scoped fallback. Rust stores four color choices but paints every group with the same accent (`source_view.rs:560`).

Wrapping sets galley width but still allocates fixed-height rows and maps clicks by one unwrapped character advance (`source_view.rs:261`, `source_view.rs:503`), so wrapped continuations can overlap following rows and have wrong hit testing. Reuse little-screen-clip's text viewport mechanisms before writing another selection/find/wrapping implementation.

### R15 — P2: Files, Changes, search, and commands are missing operational controls

Files has no filter field, Show All Files recovery, or open-file-filtered-out notice. Changes receives all `pr_changes` without `ReviewChangesPresentation` filtering; the helper in `src/pull_request/change.rs` is unused. Footer counts describe Files, including while viewing Changes. Rename rows omit old → new paths. File context menu only copies a relative path, lacking a Linux reveal-in-file-manager equivalent to Swift's action.

Search exposes only query/case/regex and requires Go/Return. Backend globs exist but UI fields do not; there is no Stop, capped-results indicator, error presentation, or equivalent automatic search on option changes (`sidebar.rs:338`, `app.rs:1146`). `SearchRunner` errors/cap return are discarded. The extra `try_recv` used for disconnect detection (`app.rs:825`) can consume a concurrently arriving batch without adding it to results.

Recent PR helper is unused and lacks all-state listing; menu lacks Refresh List. Fold to Definitions, Find References, source text sizing, filter shortcuts/reset, and indexing commands have no equivalent. Disabled-state/focus behavior needs a command inventory against `Sources/Radiology/UI/ReaderCommands.swift` and `BookmarkCommands.swift`.

### R16 — P2: session, reload, and failure presentation are incomplete

`SessionStore::set_active_pr` has no app call sites. Changing filters/wrapping does not save the session immediately or at shutdown; they persist only incidentally if another action saves the struct. Startup `RadiologyApp::new` restores the last root/PR before main applies explicit CLI options; old state can leak into an explicitly requested root. CLI PR opening omits changed-file list/tab initialization and revision setup. There are separate startup, modal, and refresh implementations of PR opening with different side effects.

Local Ctrl+R refreshes Git changes, not discovery/source/index state. Most `if let Ok` branches silently retain old content on failure, and PR modal closes after an unsuccessful open. No matching Swift opening/reading/error states, parse degradation, stale/partial diagnostics, or transactional refresh restoration exists. Centralize successful state transitions and return errors to visible app state.

### R17 — P2: validation claims overstate what the checks establish

The current suite has 38 unit tests and 30 doctests, with no Rust app-level parity suite. Tests establish helpers but leave many app routes above untested. Resource verification passes, but checks whether Syntect returns a syntax fallback; it does not parse/validate each Tree-sitter grammar/query or exercise outline/test/fold extraction. TSX passes resource verification while producing no symbols.

Clippy with `-D warnings` fails on the current installed toolchain at `context_inspector.rs:430`, `source_view.rs:285`, and `source_view.rs:569` (`float_literal_f32_fallback`). Historical clean checks may have used a different toolchain; they are not current evidence. Replace broad completeness claims with scenario gates and record toolchain/runtime scope explicitly.

## UI direction and future little-screen-clip integration

Read-only integration reconnaissance used `../little-screen-clip/AGENTS.md`, `ui_architecture_design.md`, workspace manifests, and the concrete sources below. That repository was not modified or built.

The target already runs a separate `clip-file-manager` application from shared library code. This supports **independent Radiology lifecycle plus reusable components**, rather than requiring a workbench tab, daemon, USB device, or media subsystems. Both desktops use egui/eframe 0.27 and Syntect 5.3. Rust editions differ (Radiology 2021, target workspace 2024); that alone does not prevent library reuse. The target root patches `egui-winit` for startup scaling; Radiology has only a commented patch pointing at an obsolete absolute path.

| Reuse area | Concrete target source | Recommended alignment |
|---|---|---|
| Neutral Light/Dark palette and semantic roles | `little-clip-core/src/palette.rs`; `clip-workbench/src/ui_appearance.rs` | Reuse/extract `AppPalette`/`WorkbenchColors` and visual mapping rather than maintain another hardcoded macOS palette. Keep syntax/data colors distinct from chrome. |
| Typography and glyph fallback | `clip-workbench/src/ui_ext.rs:250` `configure_workbench_style`; bundled render fonts | Use shared font setup and role constants. Target currently defines body 16, mono 15, heading 22, subheading 19; do not blindly copy those sizes into every compact metadata row. Source zoom remains independently adjustable. |
| Text viewport, selection, find, long-line bounds | `clip-workbench/src/text_preview/{mod,raw_code,search}.rs` | Reuse/extract source-byte selection/copy, variable row measurement, bounded 4 KiB fragments, regex find, and layout helpers. Add Radiology-specific folds/diff/semantic decorations without losing canonical-byte semantics. |
| Immutable source versus per-view state | `TextPreviewData`, `InlineTextPreview`, `TextPreviewState::from_data` | Share immutable source/layout facts through `Arc`; keep caret, viewport, history, fold/disclosure, and focus state per reader. `InlineTextPreview` is a compact preview, not a complete semantic reader replacement. |
| Syntax assets and language detection | `little-clip-core/src/syntax.rs`, `syntax/typescript.sublime-syntax`; text-preview syntax set | Reuse actual TS syntax registration and palette mapping instead of JS fallback. Keep Tree-sitter language/query support for semantic reader structure separate. |
| Viewer chrome and panels | `clip-workbench/src/shell/{studio_shell,panel,escape_hatch,shortcuts}.rs` | Align header/action/status conventions, panel split/collapse behavior, external-editor/reveal handoffs, and focus-aware commands. Shell does not mandate a fixed five-zone layout or replace Radiology's three-region inspector. |
| Cancellable work and stale completion handling | `little-clip-core/src/jobs.rs`; target view-owned receivers | Reuse `BackgroundJobs`/`JobTicket` patterns where appropriate, with repository/revision/source generation checks and bounded diagnostics. Do not copy target-specific IPC/media responsibilities. |
| Guarded source reads and host handoffs | `little-clip-core::kernel_interface`; `clip-workbench/src/file_source.rs` | For future file-manager opens, retain source identity/context through asynchronous reads; accept path plus location/revision, not just a display title. Preserve standalone checkout opening. |

**Dependency constraint:** many text mechanisms are private methods or `pub(crate)`, and `clip-workbench` depends on media, ML, PDF, GPU/native libraries. They are candidates for a focused shared library extraction, not drop-in independent components. Do not add the entire workbench as an unconditional Radiology dependency or copy its private implementation into a second permanent fork. Any package extraction needs review of asset ownership, licensing, build dependencies, and both apps' tests. Do not replace exact repository source bytes with a normalized preview representation without an offset map.

Radiology already has a library target, but its central `RadiologyApp` owns UI, process execution, source/semantic state, persistence, and startup restoration together. Narrow that ownership incrementally while repairing the existing bugs:

1. Keep `main.rs` responsible for standalone launch/window/environment; make restoration explicitly lower priority than CLI intent.
2. Keep repository/revision loading, semantic indexing, bookmarks, and exact navigation usable independently of an `eframe::CreationContext`.
3. Have an embeddable reader accept source/services and emit navigation/reveal/external-open actions. The standalone host executes them now; a future file-manager/workbench host can provide the same services.
4. Keep platform chrome/preferences outside immutable code intelligence and source data. Reuse the existing library APIs before introducing generic provider/plugin layers.
5. Share actual components after a focused extraction. Preserve the independent executable and tests throughout; avoid an absolute sibling-path requirement for ordinary standalone builds.

Broad UI acceptance should preserve the three-column repository reading workspace, simultaneously usable Context regions, keyboard efficiency, calm neutral surfaces, clear hierarchy, and one purposeful source footer. Align accents/fonts/widgets with little-screen-clip. Continuous line numbers, wrapping, working-tree Changes, Linux modifiers, and different window sizes can be intentional platform improvements; record those decisions rather than calling them exact Swift parity. Swift's workbench-independent reader should remain a resizable desktop app; little-screen-clip's fixed popup constraints belong to its workbench window.

## Ordered repair plan and acceptance gates

These are proposed work packages, **not completed implementation**. Add automated scenarios for behavioral risks as they are repaired; avoid widget-existence tests as a substitute for workflow checks.

| Order | Deliverable | Exit criteria |
|---|---|---|
| 1 | Shared transition/navigation/command correctness (R3, R5, R7–R9, R16) | Switching roots clears old state/cancels jobs; local/global symbols and Unicode coordinates navigate correctly; exact Back/Forward including revisions; hidden destinations reveal; one focused action per key; all bookmark mutations persist and relocate in correct scope. |
| 2 | Reusable source viewport and UI foundations (R13–R15) | Select/copy original bytes across scrolling/folds; local find and source zoom; wrapping without overlap/hit-test errors; bounded long-line work; simultaneous Context regions; compatible host palette/font/chrome components. Preserve standalone launch. |
| 3 | Real syntax and test visibility (R10) | Swift/Rust/Go/Python/JS/TS/TSX/Kotlin fixture matrix; real body/signature ranges; TSX extraction; Rust cfg predicates, shadowed JS test functions, pytest/TestCase/Kotlin rules; reveal-on-navigation and stable manual folds. |
| 4 | Background indexing and exact context (R4, R5, R11, R12) | Fresh checkout generates index; multi-project partial/failure/cancel/stale states; no stale publication; correct local IDs/encoding; validated caller boundaries/cycles and coverage; source remains responsive. |
| 5 | Genuine PR snapshots and review workflow (R1, R2, R7, R9, R16) | Checkout deliberately differs from PR head; After equals pinned head, Before equals merge base; additions/deletions/renames/binaries; multi-file hunks stay scoped; Before has no semantics; refresh preserves mapped caret/viewport or visibly reports loss; checkout remains unchanged; PR scopes/session round-trip. |
| 6 | Semantic Critical Lens (R6) | Classifier changes visibly affect repository/system/FFI calls; marks survive restart; hover frontier and fold counts use exact snapshot identities; Normal restores source view without corrupting navigation. |
| 7 | Final parity and integration acceptance (R15–R17) | Search stop/globs/errors/caps, menu/shortcut inventory, empty/loading/failure states, light/dark/scale/keyboard tests; shared-reader source can be hosted independently; standalone clean build with no sibling checkout/daemon; current test/fmt/clippy/resource gates and desktop screenshots. |

Review this sequence as dependencies emerge; viewport extraction must not delay correctness fixes or become a broad unrelated workbench refactor. Existing Rust policy, scoped-index APIs, bookmark resolver, classifiers, search worker, and source models should be reused where sound.

## Validation and architecture evidence

| Check | Result |
|---|---|
| `cargo test --workspace --locked` | PASS: 38 unit tests, 30 doctests; three compiler warnings. |
| `cargo fmt --all --check` | PASS. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | FAIL: three float-literal fallback errors listed in R17. |
| `cargo run --locked -- --verify-resources` | PASS for its Syntect lookup checks; not Tree-sitter/behavioral verification. |
| Temporary Rust/headless egui probes | Confirmed R1/R3/R5/R7/R8/R10 and contextual multi-file diff defect; output below. |
| Swift/AppKit test suite and side-by-side desktop acceptance | Not run on Linux. |
| little-screen-clip tests/build | Not run; read-only architectural inspection only. |

Toolchain: `rustc 1.98.1 (48a229cea 2026-09-01)`, Cargo 1.98.1. Logs for this run: `/tmp/radiology-parity-{tests,clippy,fmt,resources,probe}.log`; temporary probe `/tmp/radiology-parity-probe.rs` linked against this repository's compiled library, with configuration isolated under `/tmp/radiology-parity-config`. These temporary artifacts are not durable test-suite additions.

```text
Ctrl+Alt+T: theme dark=true, symbol modal=true
ts: outline symbols=1, folds=1
tsx: outline symbols=0, folds=0
swift: outline symbols=1, folds=0
kt: outline symbols=0, folds=0
local definition: scoped=true, UI lookup=false
half-open occurrence 0..3 matches column 3=true
Before loads checkout bytes=true
repository switch: old index=true, old pins=true, old revision=Some(Before), back history=true
jump byte 5: caret col=0, effective caret byte=0
jump into fold remains hidden=true
multi-file contextual diff: rendered After ranges=[1..4, 20..21], additions=4
ArrowDown with text input focus also moves source to line=2
```

Architecture Fitness comparison to HEAD reports neutral, as expected for an initially clean tree: 1.39% duplication, average CCN 3.69, maximum CCN 125, 95 clone groups, zero detected dependency cycles, 31,721 measured source lines across the mixed-language repository. This is **not a comparison of Rust to Swift behavior or proof of parity**. Existing Rust hotspots are `RadiologyApp::handle_shortcuts` CCN 125, `RadiologyApp::update` CCN 76, and `SourceView::show` CCN 80 (16 parameters). These align with observed duplicated/conflicting handlers and scattered transitions. Consolidate actual command/transition responsibilities while fixing behavior; do not split functions merely to improve scores.

Reuse search also located the existing bookmark resolver; source tracing identified the unused scoped-index, critical projection/policy, test detector, and review presentation helpers. The seven architecture-review dimensions are therefore: no review-induced code complexity increase; no new code duplication; no introduced dependency relationships/cycles; no implementation growth to assess in this documentation-only review; substantial preexisting hotspots; no hotspot remediation performed; concrete unconnected/reusable mechanisms identified. The measured neutral diff does not excuse the product gaps above.
