# Rust & EGUI Linux Native Port Plan

## Objective
Re-write Radiology from Swift/macOS to a standalone, native Linux application written in Rust using EGUI (`eframe` 0.27 / `egui` 0.27). The application will retain all core capabilities of Radiology as a fast, read-only repository explorer for understanding unfamiliar codebases, while adopting modern Linux desktop ergonomics and ensuring future integration readiness with `~/little-screen-clip`.

---

## Architectural Principles & Parity Ledger

| Subsystem | Swift / macOS Implementation | Rust / EGUI Linux Implementation | Parity Status |
| :--- | :--- | :--- | :--- |
| **Window & UI Framework** | SwiftUI + AppKit (`NSViewRepresentable` + TextKit) | `eframe` 0.27 + `egui` 0.27 (Wayland + X11 native) | Complete Native Parity |
| **Repository Discovery** | `git ls-files` + directory traversal fallback | Asynchronous `git ls-files -z` + `walkdir` fallback with ignore patterns | Full Parity |
| **File Tree & Filtering** | `FileNode` hierarchy, "Source Files Only", "Hide Tests", text query | `FileNode` tree with tree state, test path classifier, source file classifier | Full Parity |
| **Fuzzy File Picker** | Ordered subsequence matcher (⌘K) | Ordered subsequence fuzzy matcher with score ranking (Ctrl+K) | Full Parity |
| **Syntax Highlighting** | Tree-sitter query highlights | High-performance `syntect` engine + Tree-sitter queries | Full Parity |
| **Document Outline & Folds** | Tree-sitter symbols query & folds query | Tree-sitter AST queries for outline, folds, and call sites | Full Parity |
| **Text Search** | Streaming `ripgrep` (`rg --json`) | Asynchronous streaming `rg --json` with regex, case, and glob filters | Full Parity |
| **Semantic Navigation** | SCIP protobuf wire reader, definition/ref lookups | Pure Rust zero-copy SCIP protobuf reader & index query engine | Full Parity |
| **Call Tree & Caller Pills**| Multi-level caller trace with FNV-1a deterministic 12-color pills | Backward caller tree with cycle prevention & FNV-1a 12-color caller identity pills | Full Parity |
| **Bookmarks** | Persistent repo/PR bookmarks with line-anchor relocation | Persistent JSON bookmarks with line-anchor relocation & Ctrl+0..9 | Full Parity |
| **PR Review & Diffs** | `gh` CLI, `git diff -U0` unified diff parser, Before/After modes | `gh` CLI integration, unified diff parser, Before/After toggle, change block navigation | Full Parity |
| **Critical Lens** | Criticality policy (system API, FFI, repo def vs plumbing) | Criticality policy engine & source projection | Full Parity |
| **Session Persistence** | `UserDefaults` repo path & active PR | XDG-compliant config/cache (`~/.config/radiology/`, `~/.cache/radiology/`) | Full Parity |

---

## Phased Implementation Plan

### Phase 1: Cargo Project & Core Foundation
- Establish `Cargo.toml` with dependencies (`eframe`, `egui`, `egui_extras`, `syntect`, `tree-sitter`, `serde`, `serde_json`, `tokio`, `walkdir`, `thiserror`, `tracing`, `regex`).
- Implement `SourceBuffer` with UTF-8 byte canonical addressing, line starts indexing, line/column <-> byte offset conversions.
- Implement `NavigationHistory` with back/forward stacks, visit deduplication, branch truncation.
- Implement `SourceLocation` and caret tracking.

### Phase 2: Repository Discovery, Filtering & Fuzzy Picker
- Implement Git discovery (`git rev-parse`, `git ls-files -z --cached --others --exclude-standard`).
- Implement filesystem traversal fallback respecting standard exclusions (`.git`, `node_modules`, `target`, `build`, etc.).
- Build `FileNode` hierarchy and tree state.
- Implement `SourceFileClassifier` and `TestPathClassifier`.
- Implement `SourceFilters` ("Source Files Only", "Hide Tests").
- Implement `QuickFileSearch` with ordered subsequence fuzzy scoring.

### Phase 3: Syntax Highlighting, Outline & Folding
- Integrate syntax highlighting via `syntect` supporting all popular programming languages.
- Integrate Tree-sitter parsers and queries (symbols/outline, folds, scopes).
- Implement document outline hierarchical tree extraction.
- Implement code folding logic and collapse/expand controls.

### Phase 4: Search Subsystem
- Implement ripgrep process runner (`rg --json`) with debouncing and cancellation.
- Parse match JSON streams into `SearchMatch` records.
- Support regex toggle, case-sensitivity toggle, include/exclude glob patterns.
- Group matches by file path and provide direct jump to matched byte offsets.

### Phase 5: SCIP Semantic Indexing & Call Trees
- Implement zero-dependency / pure Rust SCIP protobuf wire reader.
- Parse `scip.proto` documents, occurrences, and symbol information.
- Provide Go to Definition, Find References, and symbol hover details.
- Build multi-level Reference Call Trees with cycle detection and FNV-1a 12-color caller identity pills.

### Phase 6: Bookmarks & Critical Lens
- Implement `SourceBookmark` with content digest and surrounding line anchors.
- Implement persistent bookmark storage scoped by repository and PR.
- Relocate bookmarks safely across edits using anchor line matching.
- Implement Critical Lens model: Critical vs Ordinary classifications.

### Phase 7: Pull Request Review & Diffs
- Implement GitHub client via `gh` CLI (list PRs, PR view, merge base determination).
- Parse unified diffs (`git diff -U0`) into `DiffHunk` and `FileLineChanges`.
- Support Before/After document viewing.
- Implement change block navigation (jump to previous/next change block).

### Phase 8: EGUI Native UI & Polish
- Implement top Toolbar: Repository picker, branch/PR selector, mode picker, filter toggles, search shortcuts.
- Implement Left Sidebar: Tabs for Files, Changes, Search with filter inputs and counters.
- Implement Center Reader: Line number gutter (5-line interval + caret line indicator), fold indicators, diff markers, syntax-colored code, occurrence highlights (teal, amber, violet, rose), caret movement.
- Implement Right Sidebar (Context Inspector): Symbol details, Definition preview, References & Call Tree, Document Outline, Bookmarks.
- Implement modal sheets: Quick File Picker (Ctrl+K), Go to Line (Ctrl+L), Go to Symbol (Ctrl+T), PR Selector (Ctrl+P).
- Verify compilation, unit tests, and interactive Linux desktop execution.
