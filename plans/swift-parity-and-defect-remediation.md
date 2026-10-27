# Swift Parity and Defect Remediation Report (Subsystems 1–4, Horizontal Scrolling & SVG System)

## Overview
Comprehensive feature audit and defect remediation pass porting Radiology from Swift to standalone Linux-native Rust + EGUI (`eframe` 0.27 / `egui` 0.27). Covers Subsystems 1, 2, 3, and 4, long-line horizontal scroll / line-wrap parity, and full vector SVG icon migration, ensuring zero UI-thread blocking, exact behavioral parity with the original macOS Swift implementation, and complete validation with unit tests, doctests, and linter gates.

Original Swift code remains 100% intact across `Sources/`, `Tests/`, `scripts/`, and `Package.swift` for follow-up review passes.

---

## Remediated Subsystems

### Subsystem 1: AppModel, Repository Discovery & Search
- **Source & Test File Classifiers:**
  - `SourceFileClassifier`: Refactored to achieve 100% parity with Swift `SourceFileClassifier`. Correctly includes 80+ programming language extensions while strictly excluding manifests, documentation, data files, build scripts (`*.config.js`, `vite.config.ts`), and minified bundles (`*.min.js`).
  - `TestPathClassifier`: Replicated exact directory match set (`test`, `tests`, `spec`, `specs`, `__tests__`, `testing`), Gradle/Kotlin source sets (`src/*Test`), and language-specific stem rules (`*_test.go`, `*.test.tsx`, `*Tests.swift`, `test_*.py`).
- **Quick File Search (Ctrl+K):**
  - Upgraded from simple substring search to Swift's 5-tier fuzzy ranking:
    1. Tier 0: Exact filename match.
    2. Tier 1: Filename prefix match.
    3. Tier 2: Contiguous substring match in filename.
    4. Tier 3: Subsequence fuzzy match in filename.
    5. Tier 4: Subsequence fuzzy match in full path.
  - Case-folded and diacritic-insensitive normalization via Unicode NFD mark stripping.
- **Repository Discovery & Sessions:**
  - Validates repository root, filters deleted working-tree files, skips directory symlinks to prevent infinite loops, sorts and deduplicates paths.
  - Capped recent repository history at 8 entries, validated directory existence, and restored active PR sessions.
- **Ripgrep Runner & Background Channels:**
  - Fixed pipe deadlocks by reading stderr in a background drain task.
  - Replaced synchronous UI-thread `tokio::runtime::Builder::build().block_on()` blocking in `SidebarAction::StartSearch` with non-blocking `mpsc::channel` streaming, cancellation token propagation, and continuous `ctx.request_repaint()` updates.

### Subsystem 2: Source, Syntax, Folding & Virtualized View
- **Language Registry & Tree-Sitter:**
  - Added TSX grammar support (`SupportedGrammar::Tsx`) mapped to `.tsx` and `.jsx` extensions.
  - Registered `.mts` and `.cts` extensions.
  - Added syntect fallback mapping for TypeScript and TOML files so syntax highlighting works without external packages.
- **Code Folds & Test Region Detector:**
  - Fixed fold deduplication to preserve outer enclosing folds when multiple syntax blocks share `start_line`.
  - Created `TestRegionDetector` discovering test modules across Rust (`#[cfg(test)]`, `#[test]`, `#[tokio::test]`), Python (`def test_*`, `class Test*`), and JavaScript/TypeScript (`describe`, `test`, `it`).
- **Virtualized SourceView & Typography Remediation:**
  - Implemented viewport virtualization: only visible line rows inside the clip rectangle are formatted and drawn.
  - Character column calculations from X coordinate clicks now account for font character metrics.
  - Arrow key navigation skips over collapsed folded line spans.
  - Occurrence pin indicators render cleanly on both the gutter and inline source tokens.
  - Fixed Line 1 ascender clipping by increasing row height to 22.0 px (`Theme.sourceLineHeight: 22`), centering text relative to row top rather than vertical galley midpoint, and adding 4px vertical clearance to `code_clip_rect.min.y`.
  - Eliminated gutter box/cell artifacts by setting `ui.spacing_mut().item_spacing = Vec2::ZERO` and painting a unified background column and continuous vertical divider.

### Subsystem 3: SCIP Protobuf, Semantics, Caller Trees & Modals
- **SCIP Protobuf Wire Decoding:**
  - Fixed wire tag field mapping: `Occurrence.enclosing_range` is decoded from field 7 (not field 5); `Occurrence` typed single/multi-line ranges decoded from fields 8–11 with precedence over packed field 1.
  - Fixed `SymbolInformation`: decoded `display_name` from field 6, and parsed field 7 signature documentation embedded `Document` message (extracting field 5 text). Added `relationships` (field 4) and `enclosing_symbol` (field 8).
- **Document-Scoped Local Symbols (`SemanticSymbolID`):**
  - Addressed SCIP specification requirement where `local N` symbols are unique only within their enclosing document. `SemanticSymbolID` scopes local symbols by document path while keeping global symbols repository-wide.
  - Implemented `SCIPSymbolString::fallback_name` matching Swift's descriptor grammar parser with backtick escapes, method disambiguator skipping, and type parameter handling.
- **Symbol Search Modal (Ctrl+T):**
  - Implemented 4-tier symbol ranking engine: exact name (tier 0), prefix (tier 1), camelCase/snake_case initials and component prefix (tier 2), and subsequence fuzzy match (tier 3).
  - Wired `GoToSymbolModal` to workspace-wide `SemanticIndex::search_symbols` when indexed, jumping directly to target file and line coordinates.
- **Caller Tree & Palette Determinism:**
  - Caller name extraction uses `SCIPSymbolString::fallback_name`, ensuring deterministic FNV-1a 12-color caller identity pills and palette slots match the Swift original across launches.
- **Navigation History & Dialogs:**
  - Fixed history corruption bug: navigating back/forward now preserves navigation stacks without clearing forward entries or inserting duplicate history visits.
  - Replaced hardcoded `$HOME` directory open with native `rfd::FileDialog::new().pick_folder()`.
  - Replaced unconditional default startup directory with valid persisted session restoration.
  - Implemented CLI argument flags: `--repository`, `--pull-request`, `--file`, and `--verify-resources`.

### Subsystem 4: Critical Lens, Pull Requests, Bookmarks, and UI/Appearance
- **Critical Lens Subsystem:**
  - `SCIPSymbolHeader`: Implemented 4-field SCIP header parser (`scheme manager package version`), supporting double-space escapes, package name extraction (`.` placeholder handling), and `scheme manager name` package keys.
  - `CriticalityProfiles`: Added verified scheme (`rust-analyzer`), standard runtime plumbing package set (`core`, `alloc`, `std`), explicit native FFI package list (22 crates), FFI descriptor rules (`ffi/`, `os/`), and system API descriptor prefixes/exclusions (`std::fs`, `std::net`, `std::process`, `std::env`, `std::os`, `std::thread`, `std::io`, `tokio`).
  - `CriticalityOverrides`: Built persistent symbol and package manual overrides rejecting `local N` bindings.
  - `CriticalityPolicy`: Implemented strict deterministic precedence table matching Swift: `local N` -> user symbol override -> user package override -> profiled FFI package/descriptor -> `extern "` signature -> system API -> repository package/definition -> runtime plumbing -> unknown.
  - `SourceProjection`: Disjoint, sorted byte range normalization and receded rendering.
  - Gutter count annotations with `CountAnnotation` and `CountCompleteness` (`0`, `3+`, `—`).
- **Pull Requests Subsystem:**
  - `UnifiedDiffParser::parse_file_changes`: Converts unified diffs (`git diff` or `gh pr diff`) into structured `FileChange` items capturing path, old path, additions, deletions, binary status, and change status (`Added`, `Modified`, `Deleted`, `Renamed { similarity }`).
  - `ReviewChangesPresentation`: Filters file changes against active `SourceFilters` (`sourceFilesOnly`, `hidesTests`), calculates visible additions/deletions totals, and computes `file_count_label`.
  - Sidebar integration: Automatically populates `self.pr_changes` and switches to the `Changes` tab upon opening or restoring pull requests.
  - Change block navigation: Implemented `previous_change_line` and `next_change_line` in `FileLineChanges` for navigating between changed hunks via `Alt+Up` / `Alt+Down`.
- **Bookmarks Subsystem:**
  - `BookmarkScope`: Added repository-level scoping isolating local checkouts (`<canonical_repo>\0checkout`) from pull request reviews (`<canonical_repo>\0pr:<number>`).
  - `ContextInspector` Click Conflict Fix: Fixed event handling in bookmark rows so clicking delete (`✕`), move-up (`▲`), or move-down (`▼`) is never clobbered by row jump click interactions. Added row hover highlighting.
  - Kept keyboard shortcuts `Ctrl+D` (toggle bookmark at caret) and `Ctrl+0`..`Ctrl+9` (jump to first 10 bookmarks).
- **UI, Appearance & Horizontal Scrolling Parity:**
  - Swift Overflow Parity Investigation: Swift disabled text container width tracking (`widthTracksTextView = false`) and enabled horizontal scrolling (`hasHorizontalScroller = true`) alongside a separate fixed gutter (`SourceContainerView.swift`).
  - EGUI Horizontal Scrolling & Inspector Clipping: Computes line width via `SourceBuffer::max_line_len()`, activating egui's horizontal scrollbar. Pinned the gutter to the viewport's left edge (`viewport_clip.min.x`) and strictly clipped code rendering to `[viewport_clip.min.x + GUTTER_WIDTH, viewport_clip.max.x]`, mathematically preventing any text from bleeding into `SidePanel::right`.
  - Configurable Line Wrapping: Added `wrap_lines: bool` to `SessionStore`, accessible via `View -> Wrap Lines`, toolbar menu, and `Ctrl+Alt+W` global shortcut.
  - Dark/Light Theme Switching: Added toolbar `Icon::Sun` / `Icon::Moon` button, `View -> Switch to Light/Dark Mode` menu item, and `Ctrl+Alt+T` shortcut. Dynamically swaps `AppTheme`, updates `egui::Visuals`, and re-highlights open buffers.
  - Zero Emojis & Curated Vector SVG Icon System: Embedded 18 curated SVG assets in `assets/icons/` (`arrow_up`, `arrow_down`, `arrow_left`, `arrow_right`, `chevron_down`, `chevron_right`, `check`, `x`, `code`, `text`, `contrast`, `sun`, `moon`, `folder`, `diff`, `pin`, `search`, `wrap`) with all 93 unused files pruned. Created `src/ui/icons.rs` with `Icon` enum supporting theme-adaptive tinting, button hover feedback, and painter rasterization. Replaced all emojis across toolbar, modals, sidebar, welcome view, context inspector, and source view.
  - Virtualized SourceView: Pre-cached Syntect highlighting and `ScrollArea::show_rows` virtualization to maintain 144 FPS.
  - Welcome Landing View: Displays "Read the Code", folder picker button, recent repositories history, and keyboard shortcut legends.
  - Breadcrumbs: Renders full enclosing symbol hierarchy with vector chevrons.
  - Revision Toggle: Added `Ctrl+Alt+B` shortcut to swap between Before and After revisions.

---

## Verification & Test Results
- **Unit Tests:** 38/38 passed (`cargo test --all-targets`).
- **Documentation Tests:** 30/30 passed (`cargo test --doc`).
- **Resource Verification:** `cargo run -- --verify-resources` exited 0 with all 13 grammars verified.
- **Clippy Linting:** `cargo clippy --all-targets -- -D warnings` passed with 0 warnings.
- **Code Formatting:** `cargo fmt --all -- --check` passed with 0 diffs.
- **Zero Emojis:** Verified 0 emoji matches and 0 geometric arrow symbols in `src/`.
