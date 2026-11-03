# Keyboard Shortcuts

Open this reference from Help > Keyboard Shortcuts or Ctrl+/. Ctrl is the primary modifier in the standalone Linux app; embedded hosts can also supply Command as their primary modifier. Actions require the relevant repository, source, selection or semantic destination. Menus show availability. Shortcuts belong to the focused sheet, menu, field or list before the background reader.

## Repository and indexing

**Ctrl+O** — Open a repository folder.

**Ctrl+P** — Open a pull request by number for the attached GitHub repository.

**Ctrl+Shift+P** — Import a Code Path from pasted text or a UTF-8 file.

**Ctrl+Alt+P** — Show Code Paths and reopen the left sidebar.

**Ctrl+R** — Reload the repository, or refresh the active pull request.

**Ctrl+Shift+I** — Reindex using the selected indexing personality. Default detects projects; seL4 runs the configured compiler matrix and union. Index Projects offers personality selection, editable settings, installation and cancellation.

**Ctrl+Alt+B** — Switch the current PR file between Before and After when both exist.

## Navigation and search

**Ctrl+K / Ctrl+T / Ctrl+L** — Go to File / Go to Symbol / Go to Line. Filter results with text, use Up/Down to select, Return to open and Escape to cancel. Line numbers are one-based; the line prompt starts at the current line.

**Ctrl+Shift+F** — Search Repository and focus the query. Return runs search; edits also update results. Search has explicit Search and Stop controls.

**Ctrl+F** — Find in the current source file.

**Ctrl+G / Ctrl+Shift+G** — Find the next / previous match. Return / Shift+Return in the local find field does the same; Escape closes local find.

**Ctrl+[ / Ctrl+]** — Navigate Back / Forward. Committed jumps retain the exact departure caret and PR revision.

**Ctrl+Alt+J / F12 / Return** — Go to the available semantic definition at the source caret. Plain Return/F12 belong to the focused reader or inspector, not an editable field. Return in a focused list activates that list's selection.

**Ctrl+Alt+R** — Find references at the caret and pin them in the inspector until Follow Caret is chosen.

**Alt+Down / Alt+Up** — Next / previous changed block in the current PR file and revision, without wrapping at the ends.

## Reader and presentation

**x** — With source focused and no modifiers, expand/collapse the innermost named function at the caret. Expanding opens folded ancestors. Holding x does not repeatedly toggle; text fields and composition keep their own input. A committed visit to a function opens its body again.

**Ctrl+Alt+0 / Ctrl+Shift+0** — Fold to Definitions / Collapse Functions.

**Ctrl+Alt+1 / Ctrl+Alt+2** — Normal Source / Critical Lens.

**Ctrl+Alt+O / Ctrl+Alt+H** — Toggle Source Files Only / Hide Tests.

**Ctrl+Alt+S / Ctrl+Alt+I** — Show/hide the left sidebar / right Context inspector. View > Context Panes controls individual panes and restores all panes.

**Ctrl+Alt+W / Ctrl+Alt+T** — Toggle source wrapping / dark and light appearance.

**Ctrl+Plus / Ctrl+Minus / Ctrl+Alt+=** — Increase / decrease / reset source text size. Ctrl+= and Ctrl+Shift+= also increase size on keyboards where plus shares the equals key. Only source text changes size.

**Arrows, Home/End, Page Up/Page Down** — Move the source caret. Home/End goes to line boundaries; Ctrl+Home/End goes to document boundaries; Ctrl+Left/Right moves by word; Ctrl+Up/Down goes to the current or adjacent line boundary. Add Shift to extend selection. Plain Left/Right collapses an existing selection to its corresponding edge.

**Ctrl+A / Ctrl+C** — Select all / copy exact source bytes, preserving original line endings. Source is read-only. Editable fields support their usual select/copy/cut/paste and undo/redo keys.

Pointer selection: drag to select text, Shift-click to extend the anchor, double-click for a word and triple-click for a line. A plain word double-click also pins its occurrences; modifier gestures and pin actions are available in the source context menu and Pins footer. Ctrl-click follows a semantic definition when available.

## Bookmarks, Code Paths and inspector lists

**Ctrl+D** — Bookmark/unbookmark the exact source caret.

**Ctrl+0 … Ctrl+9** — Open the assigned bookmark slot, zero through nine in saved order. Drag bookmarks to change that order.

**Up/Down, Home/End, Return, Escape** — In Bookmarks, Code Paths and Trait Implementations, select a row, select the first/last row, activate it, or clear the selection. Selecting a Code Path step does not navigate until Return. Code Paths resolve against After in PR review; conditional definitions offer a variant menu.

**Delete / Backspace** — Delete the selected bookmark when its list owns focus. Use the Code Path actions menu to delete or copy a saved trace.

**Up/Down, Left/Right, Return, Escape** — In References, select, collapse/expand context, open the selected reference/caller or clear selection. Caller controls support Space activation and Home/End. In Definition, Left/Right collapses/expands details and Return opens the definition.

**Up/Down, Home/End, Left/Right, Return, Escape** — In Outline, select/preview a declaration, move to the first/last row, collapse/expand hierarchy, commit the selected destination to history or clear selection.

**Left/Right** — With the sidebar tab strip focused, switch Files/Search/Code Paths/Changes, wrapping at the ends. Changes appears during PR review. Files and Changes use focused row/disclosure buttons; Tab reaches controls and Return/Space activates them.

## Sheets, menus and help

**Tab / Shift+Tab, Return / Space** — Move between eligible controls and activate the focused button. Menus and the personality selector support arrows, Return and Escape. Modifier shortcuts remain separate from plain list navigation.

**Escape** — Cancel a navigation/PR/import sheet, close this reference or close the current menu. Focus returns to the previous control. Closing input cannot activate the background reader.

**Return** — Confirm a navigation or PR prompt. In the Code Path multiline editor it inserts a newline; use Tab to reach Import and activate it to submit. Text composition remains with the editor.

**Ctrl+/** — Open this reference, including from the welcome screen. Up/Down, Page Up/Page Down and Home/End scroll it. Copy Reference copies this complete document; Done or Escape closes it.

Window-management shortcuts follow your desktop/compositor. Cancel Indexing, Exit PR, pane choices, personality selection, settings reveal, Code Path copy/delete and bookmark reordering have controls or menu actions without extra app-assigned shortcuts.
