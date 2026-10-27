# Full-Coverage Swift vs. Rust Parity Audit & Implementation Plan

## 1. Overview & Context

This document records the exhaustive audit of all original Swift source files (`Sources/Radiology/` ~120 files) and tests (`Tests/RadiologyTests/`) against the Rust + EGUI implementation in `src/`.

The purpose of this audit is to identify:
1. Every full UI element, view, or panel that was missing or inaccessible.
2. Every click action, context menu, keyboard shortcut, or navigation behavior that was missing or incorrectly wired.
3. Every functional gap in search, filters, bookmarks, diffs, and context inspection.

---

## 2. Audit Findings by Subsystem

### Subsystem 1: Application, Toolbar, Menus & Navigation
- **Swift Reference**: `RadiologyApp.swift`, `ReaderToolbar.swift`, `RepositoryMenu.swift`, `PullRequestMenu.swift`, `SourceSearchMenu.swift`, `SourceModePicker.swift`, `SourceRevisionPicker.swift`, `ReaderCommands.swift`, `BookmarkCommands.swift`, `RootView.swift`.
- **Gaps Identified**:
  1. *Collapsible Side Panels*: Swift's `NavigationSplitView` and `.inspector` allow toggling the left sidebar (⌃⌘S) and right context inspector (⌥⌘I) independently. The Rust app had hardcoded panels with no toggle state, buttons, or shortcuts.
  2. *Repository Menu*: In Swift, clicking the repository title in the toolbar opens a menu with "Open Repository… (⌘O)", "Open Recent" (dynamic submenu of recent repositories), and path information. Rust had only a plain "Open…" button.
  3. *Pull Request Menu*: In Swift, `PullRequestMenu` allows switching between Local Checkout and recent PRs, opening the PR number prompt (⌘P), and refreshing the active PR (⌘R). Rust had only a button opening a prompt.
  4. *Source Search Menu*: Swift groups search actions in a toolbar menu ("Go to File… ⌘K", "Go to Symbol… ⌘T", "Search Repository ⇧⌘F").
  5. *Keyboard Shortcuts*: Missing shortcuts for `⌘O` (Open Repo), `⌘P` (Open PR), `⌘R` (Refresh), `⌃⌘S` / `⌥⌘S` (Toggle Sidebar), `⌥⌘I` (Toggle Inspector), `⌥⌘1` / `⌥⌘2` (Normal / Critical Mode), `⌥⌘B` (Toggle Before / After), `⇧⌘F` (Search Repository).

### Subsystem 2: Left Sidebar & Repository Changes
- **Swift Reference**: `RepositorySidebar.swift`, `SidebarModeTabs.swift`, `ReviewChangesView.swift`, `ReviewChangeRow.swift`, `FileTreeRow.swift`, `FileTreeLabel.swift`.
- **Gaps Identified**:
  1. *Changed File Indicator in Tree*: Swift displays a 6px accent dot next to files that are modified in the active PR diff. Rust tree nodes had no changed indicator.
  2. *File Context Menu*: Right-clicking a file in the tree in Swift offers "Reveal in Finder" and path copy actions.
  3. *Change Selection Synchronization*: Selecting a modified file in the Changes tab synchronizes with the active buffer and selection indicator.

### Subsystem 3: Source Viewer & Breadcrumbs
- **Swift Reference**: `SourceBreadcrumbs.swift`, `SourceBreadcrumbPath.swift`, `SourceBreadcrumbTrail.swift`, `ChangeBlockControls.swift`, `ReaderTextView.swift`, `SourceViewer.swift`.
- **Gaps Identified**:
  1. *Interactive Symbol Breadcrumbs*: In Swift, symbol breadcrumbs in the header are clickable buttons (`Button(symbol.name) { select(symbol) }`) that scroll the editor to that declaration. Rust rendered them as static text.
  2. *Change Block Controls in Breadcrumbs*: `^` and `v` buttons in the breadcrumb bar allow jumping across modified diff hunks directly within the editor header.
  3. *Collapse Functions in Breadcrumbs*: Collapse functions button in the breadcrumb header.
  4. *Return / Enter Definition Navigation*: In `ReaderTextView.swift`, pressing plain Return/Enter when the caret is on a semantic symbol follows it directly to its definition.
  5. *Double-Click Occurrence Pinning*: In `ReaderTextView.swift`, double-clicking an identifier word pins/unpins its occurrences in the file.
  6. *Editor Context Menu*: Right-clicking in the editor provides "Go to Definition" and "Find References".

### Subsystem 4: Context Inspector & Bookmarks
- **Swift Reference**: `ContextInspectorView.swift`, `DeclarationRegionView.swift`, `UnderCaretRegionView.swift`, `BookmarksPane.swift`, `BookmarkContextMenu.swift`, `BookmarkRow.swift`.
- **Gaps Identified**:
  1. *Bookmark Context Menu*: Right-clicking a bookmark in Swift opens a menu with "Go to Bookmark", "Move Up", "Move Down", and "Remove Bookmark".
  2. *Shortcut Badges*: Displaying `[⌘0]` through `[⌘9]` on the first 10 bookmarks.
  3. *Scope Indicator*: Bookmarks pane header shows "Checkout" or "PR #num".

---

## 3. Implementation Plan

1. **Toolbar & Shell Parity (`src/ui/toolbar.rs`, `src/ui/app.rs`)**:
   - Add `show_sidebar: bool` and `show_inspector: bool` to `RadiologyApp`.
   - Add left sidebar toggle button and right inspector toggle button to `Toolbar`.
   - Transform Repository title into a native dropdown menu: "Open Repository… (⌘O)", "Open Recent" submenu, "Copy Path".
   - Transform PR control into a native dropdown menu: "Exit PR", "Refresh PR (⌘R)", "Open Pull Request… (⌘P)", and recent PR list.
   - Add `SourceSearchMenu` dropdown button in toolbar.
   - Wire all keyboard shortcuts: `Ctrl+O`, `Ctrl+P`, `Ctrl+R`, `Ctrl+Alt+S`, `Ctrl+Alt+I`, `Ctrl+Alt+1`, `Ctrl+Alt+2`, `Ctrl+Alt+B`, `Ctrl+Shift+F`.

2. **Breadcrumbs & Source View Interactivity (`src/ui/source_view.rs`, `src/ui/app.rs`)**:
   - Make symbol breadcrumbs interactive clickable links that scroll and center the target symbol.
   - Embed Change Block `^` and `v` controls and Collapse Functions button in the breadcrumbs bar.
   - Implement Return/Enter on identifier to jump to definition.
   - Implement double-click on identifier to toggle pinned occurrence group.
   - Add right-click context menu in editor: "Go to Definition", "Find References", "Copy".

3. **Sidebar Tree & Bookmarks Parity (`src/ui/sidebar.rs`, `src/ui/context_inspector.rs`)**:
   - Add changed file accent dot in file tree.
   - Add right-click context menu in file tree: "Copy Relative Path", "Copy Full Path".
   - Add right-click context menu in Bookmarks pane: "Go to Bookmark", "Move Up", "Move Down", "Remove Bookmark".

4. **Verification & Testing**:
   - Verify `cargo test --all-targets` and `cargo test --doc`.
   - Verify `cargo clippy --all-targets -- -D warnings`.
   - Verify `cargo fmt --all -- --check`.
   - Verify `cargo run -- --verify-resources`.
   - Zero emoji check across all files.
