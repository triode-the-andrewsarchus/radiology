# Bookmark labels and numbered command boundaries — 2026-11-08

This checkpoint fixes two concrete differences found while reviewing the Swift
bookmark implementation. The subsequent [navigation audit](rust-bookmark-navigation-contracts.md)
closes its three combined navigation workflows. The subsequent
[model/storage review](rust-bookmark-model-contracts.md) closes its other three
functions, including deletion-selection and invalid-index repairs.

## Behavior

Swift AppModel+Bookmarks uses the current declaration's name for a new bookmark,
falling back to the last path component. Rust previously persisted path:line for
all new entries. The app now uses its existing enclosing_source_context result,
selects the innermost declaration and falls back to the filename for source outside
a declaration or without supported syntax. The existing row already displays path
and line independently, so this makes the primary label more useful without losing
coordinates. Persisted labels are retained on load and relocation; no migration
rewrites previously saved bookmarks.

Swift's slot command accepts only 0 through 9. Rust's command readiness previously
accepted any existing index, although normal menus and keys only emitted ten slots.
Command readiness now enforces that boundary. List activation still follows UUIDs
through open_bookmark and can open entries at any index. Reordering, relocation,
revision choices and persistence continue using their existing mechanisms.

No public API, storage format, dependency, worker, cache or text style changes.
The SwiftUI, macOS-design and typography skills remain unavailable in this Linux
installation as recorded in the original parity review. Swift source serves as the
behavioral reference; this checkpoint adds no Swift or typography implementation.

## Verification

Both new app regressions fail on the preceding implementation: a nested function
receives src/nested.rs:4 rather than inner; slot 10 is enabled in a twelve-entry
list. With the fixes:

- At both display scales, nested declarations and plain/unsupported source receive
  the correct label, records round-trip through the actual isolated settings store,
  and the real app renderer paints the declaration label.
- Twelve real app bookmarks survive reordering and persistence. Commands 0–9 are
  enabled; indices 10, 11 and usize::MAX leave loading/history unchanged. Slot 9
  opens its reordered destination, and UUID activation opens the twelfth entry.

Workspace passes 801 app tests/11 ignored, two binary, 13 font, 22 text and
60 doctests. Strict all-target Clippy, formatting and diff checks pass. Logs are
retained under
~/.local/state/radiology-qa/bookmark-contracts-20261108-679412b/.
This checkpoint uses app/rendered tests; it does not claim a new native desktop
run or execution of AppKit/SwiftUI on Linux.

## Architecture

Baseline before-bookmark-contracts-20261108 versus after-bookmark-contracts-20261108:
+100 measured NLOC, one test module, two test functions, no new runtime function.
The existing store toggle, context query and command readiness are reused.
Average/max CCN remain 3.40/132; 36 high-complexity functions, 123 clone instances,
1323 duplicate lines, one cycle and eight structural issues are unchanged. Full
normalized fragment/file-pair multisets contain zero added or removed clones.
The raw comparison flags an unchanged clone in the touched parent test module and
equal-score hotspot ranking changes between two untouched Python files. Neither
represents a newly introduced clone or dependency/complexity hotspot.

## Remaining work

Continue the six Swift bookmark functions against their individual assertions,
using the existing [bookmark model/native evidence](rust-bookmark-pane.md),
[revision-choice evidence](rust-kotlin-pr-fixture.md) and structural-history
checkpoints. In particular, complete the combined checkout/PR/repository restore
and superseded-jump workflows rather than treating isolated store tests as proof
of their complete app behavior. Other Swift suites and successful SDK-backed
Android indexing remain open. Accessibility and actual sibling adoption remain
deferred by the user's scope.
