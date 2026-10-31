# Rust outline selection, preview and disclosure

The authoritative Swift contracts are `DocumentOutlineView.swift`,
`DocumentOutline.swift`, `OutlineRow.swift`, and the preview/commit path in
`AppModel.swift` and `SourceViewer.swift`. Swift preview scrolls/reveals without
selecting text; explicit activation navigates. Rust keeps that distinction.

## Implemented contract

- Functions retain source-span line counts and descending-size/source-order
  sorting. Non-callable declarations form a collapsible, indented Declarations
  section. Removed callable containers promote nested non-callable declarations.
- Up/Down and Home/End select and preview. Left collapses a declaration or selects
  its parent; Right expands or selects its first visible child. Return and name
  clicks commit exact declaration-name bytes. Escape clears list selection.
- Preview reveals manual folds and scrolls to exact original bytes, retaining
  source caret, source selection, under-caret context and navigation history.
  The outline therefore remains usable during previews. Committed navigation
  retains the existing reveal/history/context behavior.
- Source caret changes follow the current innermost declaration. Unchanged active
  declarations preserve independent list selection/disclosure. Moving into a
  collapsed descendant reveals its ancestors. Hidden tests are removed from the
  projection; an action targeting a hidden row cannot navigate.
- Targets carry path/revision, SHA-256 content identity and declaration selection
  range. Dispatch revalidates them against current visible source-bound rows.
  Changed bytes, another path/revision, or a retired scope reject stale actions.
- One focus target owns unmodified list navigation before egui focus movement.
  Tab, modified keys, releases and global reader commands retain their routes.
  Source/review preparation disables outline navigation.
- Only visible fixed-height rows paint. Metadata/order are prepared once per
  exact document rather than sorting full syntax every repaint; test projection
  updates by generation. Names truncate within the available row width. Existing
  Body/Small styles, colors and SVG chevrons provide hierarchy/disclosure.
- Rust constants, statics and enum variants now participate in the parsed outline,
  with exact name ranges and native enum-child nesting.

## Ownership and shared mechanisms

`ui/outline_list.rs` owns document-bound rows, projection, disclosure and selection.
`ui/app/outline.rs` validates and binds preview/commit to the displayed reader.
`SourceViewport::preview_byte` is a separate, content-guarded reveal request;
committed source navigation takes precedence over a queued preview.

Bookmarks and outline share focused raw-key dispatch, standard list movement,
row reveal geometry and deterministic test painting. The inspector delegates
outline presentation. Common repository/source readiness checks serve both
regions. No dependency or sibling runtime requirement was added.

Native work exposed a pre-existing empty-history edge: the first committed jump
could have no recorded departure. `record_departure` now seeds a missing current
entry only at a successful navigation commit. Preview still leaves an empty
history empty. A regression exercises both empty and seeded history with a
Unicode departure and exact Return/Back restoration.

Actual source-context changes now synchronize outline state immediately, including
commit followed by Back before another paint. Returning to whitespace clears the
old declaration selection; Return cannot reopen it accidentally. The application
regression failed before this binding and now verifies the sequence without a UI
frame between commands.

## Automated and architecture evidence

At `7d351ac`, checks pass 283 app tests, 5 reader-text tests, 30 doctests, formatting,
Clippy with warnings denied, binary build and rustdoc. One external-Clang-dependent
test remains intentionally ignored. New tests cover native declaration kinds,
preview caret/selection preservation, stale source rejection, hierarchy controls,
active-caret following, hidden-test filtering, 1,000-row virtualization/reveal,
and application preview/commit/history and stale-target isolation.

Baseline `before-outline-controls` is at `92158fa`. Final product comparison has
no new hotspot or dependency cycle; average CCN remains 3.34 and maximum 100.
Clone groups remain 91, duplication decreases from 0.70% to 0.69%, health remains
approximately 82.69; feature and test NLOC increases by 779. The tool labels its
touched-file report REGRESSION because it also flags an extended declarative
language-kind match table and existing short test/UI fragments. Inspection found
no new duplicate product mechanism. Shared key/reveal/test helpers removed the
initial accidental list duplication; source target resolution is a cohesive
operation outside the viewport's geometry preparation.

## Native acceptance

The final clean-session fixture is `/tmp/radiology-outline-final`, driven by
`/tmp/radiology-outline-final-driver.py`. PID 3935485 executes the binary at
`7d351ac`, SHA-256
`6d81bc397b3e60a0dbbad07b5304d407975d0fc5b15816a0a2928a8823bd361c`.
The corpus contains 120 functions, nested modules/enum variants, Unicode origin
text, and a test declaration. Product operation needs no sibling checkout; the
external private compositor/input helper is reused from little-screen-clip.

- `origin.png`: clicking the first function and Ctrl+[ restores line 1.
- `preview.png`: End selects/reveals the last nested enum variant while source
  status remains Line 1 and the declaration context stays at the original caret.
- `collapse.png`: ordered Left/Left selects the parent and hides enum variants.
  Existing vector chevrons display correctly without font-dependent glyphs.
- `commit.png`: Right/Right/Down/Return reveals and commits Second on line 605,
  with declaration and breadcrumbs identifying the enum member.
- `back.png`: Ctrl+[ restores line 1, clears the departed active declaration and
  clears outline selection. This is the final post-binding-fix acceptance.
- `hide-tests.png`: Ctrl+Alt+H removes hidden_case from the outline and projects
  the source test region without changing the current caret.
- `light.png`: the same selection/disclosure/hidden-test presentation remains
  readable in light appearance after Ctrl+Alt+T. This is a single-scale fixture,
  not completion of the application's general theme/scaling audit.

The binary and capture hashes are recorded in `build-evidence.json` under that
fixture. Application and private compositor exits are zero; pointer-helper
termination is intentional.

For the history investigation, `/tmp/radiology-outline-debug-2/radiology.log`
records GDB stops in navigate_history on the preceding `49ef722` build: Ctrl+[
reaches command dispatch with two entries at index 1, and Back restores the
origin after the preview/commit sequence. The native final build above includes
subsequent immediate outline/caret binding. DAP's lldb backend was unavailable;
GDB was launched as the parent of the private app because direct attach is
restricted. All debugging processes have exited.

Earlier `/tmp/radiology-outline-native` captures include intermediate builds and
inconsistent restart/input sequences; they are not final acceptance. Repeat
broader native restart/input acceptance in the final lifecycle audit.

## Remaining outline and broader parity work

PR changed-callable markers are now integrated; see the
[review marker contract](rust-review-outline-changes.md). Transient destination
feedback is also integrated; see its [contract](rust-destination-feedback.md). Broader mixed-language/long-name, pinned-revision,
scaling, keyboard interaction and side-by-side Swift visual acceptance remains.
Language/parser-error, preparation and hidden-test explanations now follow the
[Swift status contract](rust-outline-status.md), including no phantom declarations
in unsupported text and native narrow-theme/scale evidence. Revisit/document caches,
indexing/PR lifecycle, background repository work, source rendering and shared
palette/chrome integration remain part of the full objective.

The requested swiftui-pro, macos-design and typography-designer skills are absent
from the configured skill roots. This work follows checked-in Swift behavior and
existing Rust styles. No new type scale was introduced.
