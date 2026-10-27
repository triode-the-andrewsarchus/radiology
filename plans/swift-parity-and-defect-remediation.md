# Swift Parity and Defect Remediation Report (Subsystems 1–3)

## Overview
Comprehensive feature audit and defect remediation pass porting Radiology from Swift to standalone Linux-native Rust + EGUI (`eframe` 0.27 / `egui` 0.27). Covers Subsystems 1, 2, and 3, ensuring zero UI-thread blocking, exact behavioral parity with the original macOS Swift implementation, and complete validation with unit tests, doctests, and linter gates.

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
- **Virtualized SourceView:**
  - Implemented viewport virtualization: only visible line rows inside the clip rectangle are formatted and drawn.
  - Character column calculations from X coordinate clicks now account for font character metrics.
  - Arrow key navigation skips over collapsed folded line spans.
  - Occurrence pin indicators render cleanly on both the gutter and inline source tokens.

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
- **UI & Appearance:**
  - Dark/Light Theme Switching: Added toolbar `🌙 / ☀️` button, `View -> Switch to Light/Dark Mode` menu item, and `Ctrl+Alt+T` shortcut. Dynamically swaps `AppTheme`, updates `egui::Visuals`, and re-highlights open buffers.
  - Virtualized SourceView: Pre-cached Syntect highlighting and `ScrollArea::show_rows` virtualization to maintain 144 FPS.
  - Painter Viewport Clipping: Enforced `ui.painter().with_clip_rect` to prevent text overflow from bleeding into the right-hand panel.
  - Welcome Landing View: Displays "Read the Code", folder picker button, recent repositories history, and keyboard shortcut legends.
  - Breadcrumbs: Renders full enclosing symbol hierarchy (`Outer › Inner › Current`).
  - Revision Toggle: Added `Ctrl+Alt+B` shortcut to swap between Before and After revisions.

---

## Verification & Test Results
- **Unit Tests:** 38/38 passed (`cargo test --all-targets`).
- **Documentation Tests:** 29/29 passed (`cargo test --doc`).
- **Resource Verification:** `cargo run -- --verify-resources` exited 0 with all 13 grammars verified.
- **Clippy Linting:** `cargo clippy --all-targets` passed with 0 warnings.
- **Code Formatting:** `cargo fmt --all -- --check` passed with 0 diffs.
