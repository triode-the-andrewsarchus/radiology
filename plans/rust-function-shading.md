# Enclosing callable shading

Swift's `SyntaxSnapshot+Function.swift` chooses the innermost function, method,
initializer or deinitializer whose complete half-open declaration contains the
caret. Signature and body qualify; types, properties and gaps between methods do
not. `Theme.functionShadeColor` uses a neutral label wash at 4.5% in dark mode
and 3% in light mode. This is independent of semantic index availability.

Rust reuses the exact source's existing enclosing-symbol breadcrumb chain and
`DocumentSymbol::is_callable`. No parse, query, storage or Critical Lens coupling
is added. Swift initializers/deinitializers already map to the callable kinds.
Source caret changes request the next frame so the host republishes the matching
breadcrumb context, including pointer actions that replace a CaretMoved action.

The viewport intersects the declaration with each visible bounded UTF-8 fragment
before mapping its first/last characters to shaped visual rows. Bands cover the
full code width, empty intermediate lines and line leading. Half-open ends,
wrapped partial lines, hidden folds and test-label space retain their own
geometry. Active-line tint follows one owning visual row instead of every
continuation of a logical line. Paint order is function wash, PR change wash,
destination feedback, active row, find/selection/pins, glyphs and caret.
The ordered background pass is separate from foreground decoration.

Native acceptance exposed an existing empty-line hit-test defect: the shaped
empty galley yielded only four pixels of clickable height. Source hit geometry
now uses the allocated minimum line height, so clicking the middle of a blank
line places the caret there and clears callable context between methods.

## Verification

`cargo test --workspace`: 296 app tests passed, one external-Clang fixture
ignored; five reader-text tests and 30 doctests passed. Formatting, strict Clippy,
build and rustdoc pass. New regressions cover nested parsed declarations, type
and between-method exclusion, contiguous blank lines, half-open ends, folded
bodies, wrapped Unicode bands, both palettes, layer order and empty-line clicks.

Architecture comparison against `before-function-shade` adds 216 NLOC and nine
functions, with no new hotspot, dependency cycle or clone group. Average CCN
remains 3.34, maximum 100, clone groups 91. The overall REGRESSION label refers
to two unchanged pre-existing source-view/context-inspector test fragments, which
are present in the baseline at source-view lines 567–575. An initial new
background-painting hotspot and duplicated test filter were removed.

Native artifacts are external temporary acceptance evidence, not product runtime
assets. The driver imports the sibling's compositor helper for testing only;
Radiology and reader-text remain standalone. Broader wrapping/scaling, grammar,
accessibility and full Swift visual acceptance remain open.

Final native binary SHA-256:
`c099cd67223467be8c372a730dd3d64ce3f521cd268b79833946d3a55431db3d`.
Source checkpoint is the implementation based on `6e705e8`; manifests hash each
capture and the binary. Final dark artifacts live in
`/tmp/radiology-function-shade-dark-native`: `outer-dark.png`, `nested-dark.png`,
`gap-dark.png`, `build-evidence.json`. PID 4082463 and its private Niri/Weston
exited zero. Outer scope washes blank/nested lines; nested scope leaves the outer
body clear; a direct middle-of-gap click selects line 18 with no callable wash.
At x=800, dark body pixels change from (28,28,30) to (38,38,40); nested y=190 is
shaded while y=130/230 remain base. Gap y=375 has only active-row (28,34,41).

Final light artifacts live in `/tmp/radiology-function-shade-native`:
`final-method-light.png`, `final-gap-light.png`, `final-outer-light.png`,
`final-nested-light.png`, `build-evidence.json`. Final app PID 4079205 and both
compositors exited zero. The method body at (800,211) is (248,248,248), then
returns to (255,255,255) after the gap click; the gap receives active-row
(242,248,255). Earlier captures in this directory precede the empty-line fix;
only the final-prefixed captures carry the final binary claim. Persistent
pointer helpers were intentionally terminated during cleanup.
