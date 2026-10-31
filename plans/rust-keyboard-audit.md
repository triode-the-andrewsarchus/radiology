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

## Still open

Native acceptance of the new shortcuts, complete toolbar/context-menu traversal
and focus restoration remain in the final keyboard pass. Existing context-menu
Escape evidence is recorded [separately](rust-context-menu-dismissal.md). Broader
indexing/grammar corpora, reader wrapping/scroll/scaling, PR/cache recovery and the
final Swift requirement audit remain required. Full accessibility and actual
little-screen-clip adoption remain deferred.
