# Current-function folding and navigation

Ports the November 2 upstream reader contract in `function-fold-key.md` and
`Sources/Radiology/Syntax/SyntaxSnapshot+Function.swift`.

## Behavior and shared mechanisms

Unmodified x in the source toggles the innermost named function, method or
lifecycle routine. It ignores repeats and composed text, does not intercept
search/picker/inspector typing, and leaves siblings/nested folds unchanged.
If a collapsed ancestor hides an already expanded function, x opens the ancestor
and retains the expanded function, giving the action a visible effect. There is
also a View menu action, disabled without an available current function body.

`syntax::folds::enclosing_function_fold` reuses existing outline breadcrumbs,
callable roles and exact structural folds. It matches the declaration's own body
by identifier/end boundaries; a closure, default argument, container or nested
function cannot substitute. Missing/single-line bodies and between-method gaps
have no target. The API is independent of egui and suitable for later reader reuse.
The app continues using the existing fold store and ordered source-command queue.
Composition ownership lives with the source viewport and resets on owner retirement.
No extra parser, worker, dependency or persistent preference is introduced.

Committed navigation opens the destination's function body and ancestors, including
header/name destinations whose body does not contain the destination byte. This
applies to same-buffer and asynchronously prepared source, and history traversal.
Ordinary caret motion, outline previews and appearance rehighlighting do not reopen
a manually collapsed body. Whitespace before an indented declaration belongs to
its enclosing scope; a line-start jump there does not assert a nested target.
Tests/native acceptance use the actual function-name destination when checking
header expansion. Exact departure caret, byte selection/copy and PR revision
remain governed by the existing navigation transaction.

## Evidence

- Real Rust syntax proves nested functions and closures select the owning named
  body's exact range, with no target between methods or in a single-line body.
- Actual-app frames at 1×/2× cover x, repeats, Shift-x, composition across frames,
  search-field typing, exact caret/history preservation, hidden-parent recovery,
  sibling preservation, manual collapse through rehighlight and a committed
  signature jump. The hidden-parent case has a recorded failing run before repair.
- Five existing tests assumed file/header visits retained a collapsed destination.
  Updated expectations adopt upstream expansion; tests still prove explicit manual
  folds survive recoloring/reloads, unavailable reads preserve source, PR revisions
  remain exact and source line selection copies the currently visible source line.
- Final workspace: 644 app tests pass, seven existing ignored; two binary,
  13 font and 21 text tests pass. Strict Clippy/check/build/rustdoc, formatting,
  diff checks and 44 doctests pass; rendered shared API contract/example checked.
- Native old/current at 1×/2× confirms missing old x and collapsed old symbol-jump
  body, then current toggle/expansion, sibling retention, light appearance and
  exact 148-byte Unicode/CRLF Copy. Final fixture/binary hashes stay unchanged;
  all owned app/compositor exits are zero. Captures inspected. Initial harness
  used Go to Line before an indented declaration, which correctly resolves its
  enclosing scope; final acceptance uses Go to Symbol's exact name destination.

## Architecture review

| Dimension | Before | After | Assessment |
| --- | --- | --- | --- |
| Analyzed NLOC | 93,105 | 93,312 | +207, includes regression coverage |
| Functions | 3,939 | 3,945 | +6, no additional files |
| Average / maximum CCN | 3.37 / 132 | 3.37 / 132 | Unchanged |
| Clone groups | 125 | 125 | All normalized paths/tokens/fragments unchanged |
| High-complexity functions | 33 | 33 | No new hotspot |
| Dependency cycles | 1 | 1 | Existing upstream report unchanged |

Default comparison flags shifted old clone locations as REGRESSION; normalized
comparison verifies no new clones. The lookup adds necessary declaration/body
matching; command and composition methods own distinct operations. No new
subsystem, parallel parser, worker or coupling layer was introduced.

Requested swiftui-pro/macOS/type skills are absent from installed skill paths;
checked-in Swift behavior and existing command/menu/type roles supply the reference.
No type scale or custom visual control was added. Full accessibility remains deferred.

Architecture baseline: `before-current-function-fold` at `3252359`, after importing
upstream. Initial measurements identified growing dispatcher/input complexity;
function-fold operations and source composition capture now have focused methods,
removing those new complexity hotspots. Existing parse/fold/navigation mechanisms
were found through reuse lookup before implementation. Shifted existing clone rows
are checked by normalized fragment/path comparison rather than line offsets alone.

Evidence: ~/.local/state/radiology-qa/current-function-fold-20261102-ccffad5.
Broader upstream parity remains in `rust-upstream-parity-2026-11-02.md`, including
Files reveal, pane preferences, traits, Code Paths and indexing personalities/seL4.
