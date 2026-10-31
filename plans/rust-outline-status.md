# Outline preparation and empty-state explanations

## Swift contract and Rust behavior — 2026-10-31

`Sources/Radiology/UI/DocumentOutlineView.swift` distinguishes structure loading,
parser failure, unsupported files, hidden test declarations and a parsed language
with no declarations. The standalone Rust inspector now preserves those
distinctions:

- Without source, explain how to populate the outline.
- During source/structure preparation, show Reading structure. Previously
  accepted source and rows remain visible with navigation disabled until the
  replacement publishes; an initial load does not misleadingly ask to open a file.
- For a registered language with no declarations, name that language.
- For unsupported files, explain that source text remains available and list
  structural languages from the same registry metadata used by path detection.
  Syntax coloring may support additional formats. README code examples beginning
  with `fn`, `func` or `def` do not become invented declarations.
- If Hide Tests removes every declaration, show its Ctrl+Alt+H recovery shortcut.
  Toggling that command restores the actual rows.
- If a registered native parser returns no tree, retain source and identify
  structure degradation. Existing approximate fallback rows remain available and
  are explicitly described as approximate. Recoverable errors inside a returned
  tree preserve its usable declarations without claiming a total parser failure.

`DocumentSyntax.parse_error` travels in the existing atomic PreparedSource result.
The app publishes it with the accepted source, preserves it on recoloring or a
failed replacement read, and clears it on successful analysis, review replacement
or repository reset. It is independent of a folding-query error. Embedded inspector
hosts pass `outline_loading` and the displayed source's `outline_error`.

The outline caches its language explanation by its existing exact document
identity. Three-line Small text with word-boundary prose wrapping and bounded hover text share
`text_fit::explanation`; the helper reuses measured line fitting and scalar-safe
bounds. No new font role, dependency, generic status framework or sibling runtime
requirement was introduced.

## Verification and limits

Two new real-app tests cover Rust/Kotlin/unsupported/hidden states, restoring hidden
rows and publication/recolor/failed-read/reload/reset ownership. Both fail against
the previous empty-state renderer. Adding code examples to the README independently
reproduces the previous phantom-outline bug before the fallback guard.

A parser-outcome test covers missing native trees, unsupported text, cancellation
and a recoverable malformed tree retaining a valid declaration. The existing
cancellation case now also requires unsupported text to produce no symbols through
both DocumentSyntax and the standalone SymbolExtractor. Native parser failure is
injected at its outcome and publication boundaries for these checks; no claim is
made that a statically linked grammar was made to fail in the native app.

A component test checks initial/loading/failure text at 200/300-point widths in
both themes at 1×/2×, including a long Unicode diagnostic. It primes egui's pending
scale change before asserting pane geometry, then verifies three-row/4-KiB shaping
bounds, intact prose words and the visible recovery shortcut. A saved negative
reproduces the word split before prose wrapping was separated from code/path
preview wrapping. The empty prompt uses the same bounded metadata presentation.

The workspace passes 511 app tests (seven ignored), two binary, four font,
eleven reader-text and 35 doctests. Strict Clippy/check/build, formatting,
whitespace checks and warning-denied rustdoc pass. Rendered public documentation
for parser error, registry languages and inspector preparation/error fields was
inspected.

Architecture baseline `before-outline-empty-states` is at `85d3404`. The tool's
diff-aware result is REGRESSION for unchanged touched match/test clone fragments;
the complete clone multiset has zero additions/removals (91 groups). Duplication
is 0.66% → 0.65%, average CCN remains 3.35, maximum 100, high-complexity functions
24 and dependency cycles zero. NLOC increases by 343, primarily verification.
The repeated status rendering and wrap policy share the existing measured-text
module; explanation-layout assertions are separate from the test matrix setup.

Final native 1×/2× runs show accepted empty Rust/Kotlin files, an unsupported
README containing code-like examples, and hidden/restored Rust test declarations.
Narrow dark/light captures preserve whole prose words and the recovery shortcut.
Original source copy is exact, fixture hashes are unchanged, the binary matches
the final build, and all application/compositor exits are zero. Earlier pre-wrap
captures are investigation evidence, not the final acceptance set.

Native captures and complete reports are recorded in the evidence archive:
`~/.local/state/radiology-qa/outline-status-20261031-85d3404`.

Broader grammar/corpus and pinned-review visuals, mixed-owner/menu/source ordering,
reader/large-file performance, real index/PR/cache acceptance and the final Swift/
reuse audit remain open. Full assistive accessibility and actual sibling adoption
remain deferred; keyboard shortcuts and focus/navigation remain required.
