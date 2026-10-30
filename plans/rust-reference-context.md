# Reference focus and inspector context

## Contract

Swift's `ContextModel` maintains an explicit reference target independently of the
current enclosing declaration and under-caret definition. Rust now follows that
contract: Find References pins the exact indexed occurrence; caret/file navigation
continues updating the definition/outline region. Follow Caret releases the pin
and queries the latest enclosing declaration without navigating source or history.

`semantic::references::ReferenceContext` owns explicit/automatic focus, captured
index/source identity, rows and replaceable background work. It reuses `LatestRead`
for cancellation and retired-publication isolation; same declaration/index/source
caret moves do not rebuild the query. Changed source/index reloads it. Repository
and review-snapshot transitions drop focus and retire workers. Before suspends
After reference data while preserving the explicit target for returning to After.

Automatic identity requires a definition at the parsed declaration name's first
canonical byte, with source digest and declared column encoding validation. It
never chooses the first definition on the same line. Explicit lookup also accepts
document-scoped locals and preserves their document identity across navigation.

Direct reference rows retain exact SCIP coordinates, sort/deduplicate sites and
reject invalid paths, negative/reversed/empty coordinates and unknown encodings.
Labels use indexed enclosing extents with full coordinate containment, falling
back to the path. Indexed references do not prove calls; these rows have no invented
recursive call relationships. Parsed/validated call-tree disclosure remains open.
Navigation reuses the existing asynchronous encoding/source-provenance path.
Reference work runs off the UI thread; the inspector paints only visible rows.

Search menu and Ctrl+Alt+R share the command dispatcher. The source menu resolves
its exact clicked caret position and disables the action without a valid target.
Explicit focus reveals the inspector and exposes Follow Caret. Loading/errors and
unavailable semantics are visible without replacing source or adding history.

## Verification

Eight new tests cover exact same-line declaration identities, UTF-16 with Unicode,
local isolation, explicit focus across files, Follow Caret, same-context idle work,
replacement indexes, retired publications, cancellation, deduplication, invalid
coordinates, stale/Before rejection, shortcut ownership, and application source/
caret/history preservation. Full workspace: 218 application tests pass, one external
Clang check ignored, 5 reader-text tests and 30 doctests pass. Formatting, Clippy
with warnings denied, build and rustdoc pass.

Architecture baseline: `before-reference-focus` at `a769d68`. The inspector's upper
region now has a cohesive renderer rather than adding branches to its main show
function. Average CCN decreases 3.36 to 3.35, maximum remains 100, and no new product
hotspot or cycle is reported. The comparison flags two additional short test clone
groups; reports touching the inspector are existing shape/pointer test sequences.
No shared product algorithm is duplicated; test setup/assertions stay local.

Native menu/Follow Caret acceptance is in progress in the private Wayland harness
at `/tmp/radiology-reference-native/`; fixture indexes are explicitly unverified
imports. No sibling runtime dependency has been introduced. Required Swift/macOS/
typography skills remain unavailable in the configured skill locations; Swift
source and the established reader typography are the UI reference.

## Remaining inspector parity

Single-click reference previews, independent Up/Down/Left/Right/Return preview/
commit behavior, full source definition previews, snippets, coverage reporting,
parsed/validated call-tree disclosure, extension implementation links, and broader
large-repository/runtime acceptance remain required. This package does not close
the full Rust/Swift parity objective.
