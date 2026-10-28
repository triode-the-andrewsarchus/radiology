# Rust inline-test presentation

The Swift requirements remain in [source-filters.md](source-filters.md). The Rust
implementation now separates conservative AST detection, immutable syntax facts,
per-document test reveal state, manual folds, and viewport byte projection.

## Detection and parsing

`DocumentSyntax::analyze` parses once for the outline, fold candidates and test
regions. `SupportedGrammar` owns grammar routing. Rust cfg uses three-valued
logic, retaining anything production-reachable or unparsed. Rust/Kotlin share an
iterative region walk. Python requires pytest module evidence for naming rules
and recognizes unittest bases; ECMAScript checks callbacks, binding provenance,
member/call chains and Vitest guards. Native Kotlin-ng node shapes are used for
annotations and test classes. Whitespace-adjacent tests merge; owned doc/trailing
comments are included without consuming production neighbors or CRLF terminators.

Swift/Kotlin grammars are linked and their initial structural outlines/folds are
covered by fixtures. This does not establish complete highlighting/query parity.

## Presentation and interaction

`TestCodePresentation` keeps its own revealed-region set and generation. Loading
another document or turning Hide Tests on retires prior reveal state. Explicitly
opened test paths and Rust test-only documents remain readable in full, with
“Test file” status. Mixed files retain dimmed first-line headers, gutter controls,
inline reveal/hide labels and a hidden-region count. Manual fold state is separate.

The viewport subtracts exact hidden byte intervals from bounded source fragments.
Canonical line numbers, visible production tails on shared lines and original
selection/copy bytes are preserved. Explicit navigation reveals the destination;
local find reveals every intersecting region before selecting it. Hidden
regions omit their declarations from the inspector outline. Full syntax remains
available to breadcrumbs, context, navigation and symbol search.

Source rows now have individual code hit regions, excluding inline controls.
The canvas retains a stable keyboard focus identity. This fixed a real reveal-label
click conflict while preserving cross-row drag selection and source shortcuts.

## Verified evidence

- Detector fixtures: Rust cfg reachability/contradictions, strings/macros,
  doc/trailing comments and CRLF; Python framework evidence; local JS shadowing,
  callbacks, TS/TSX and guards; Kotlin annotation bodies and Kotest classes.
- Projection tests: header retention, production tails, reveal/reset behavior,
  readable explicit test files, hidden-byte copy and canonical fragments.
- Reader/egui tests: manual folds survive preference changes and test navigation;
  hidden targets reveal at exact columns; inline label clicks reveal real source.
- Native Niri smoke with isolated settings: hidden body and outline omitted,
  local find reveals/centers/selects the target, clipboard exactly equals all 170
  original fixture bytes including hidden code and CRLF. Screenshots:
  `/tmp/radiology-native-inline-tests-hidden.png` and
  `/tmp/radiology-native-inline-tests-find-reveal.png`.
- Latest suite: 73 application tests, 5 reader-text tests, 30 doctests; formatting
  and Clippy with warnings denied pass.

Background source analysis remains required: this shared structural analysis still
runs synchronously on file open. Broader grammar/highlight/query coverage and the
full parity ledger remain open. A source/API fixture test assertion clone and the
pre-existing review-preparation structural flag remain documented architecture
findings; no new product traversal duplication is retained.
