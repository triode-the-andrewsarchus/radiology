# Source multi-click selection

## Swift contract and Rust gap

`Sources/Radiology/Source/ReaderTextView.swift:645` delegates ordinary mouse input
through `NSTextView.mouseDown` before its occurrence callbacks. Native word and
paragraph selection therefore coexist with occurrence pinning. Double-click (also
word-drag) toggles the original clicked occurrence; third and later clicks pass a
nil occurrence target and do not toggle pins. Single clicks select occurrences
only when text selection remains empty. Command-click definition navigation stays
separate.

Rust previously implemented occurrence callbacks and character dragging, leaving
double/triple-click text selection collapsed. The viewport regression failed with
an empty copy instead of `café_name`. Native previous-binary checks at 1×/2× leave
a fresh clipboard sentinel unchanged after both gestures.

## Implementation and reuse

`reader_text::WordRangeCursor` extends the existing cooperative word-navigation
machinery. Word classification, projected span/grapheme iteration and fragment
traversal are shared. It selects Unicode identifier runs, whitespace runs, or an
atomic symbol without requiring syntax or semantic data. Hidden joins and visible
newlines stop words. Projection fragment limits remain authoritative, including the
existing policy for clusters larger than a fragment. Hosts must retire a cursor
when its immutable source/projection or input owner changes. Runnable public docs
show repeated bounded advances.

The viewport retains one gesture origin and a character/word/line unit. Word and
line drags extend whole units in either direction. Triple-click selects a projected
paragraph, including the original line ending; a folded paragraph's canonical
range encloses its hidden source. Wrapped continuations remain one paragraph.
The existing ordered event queue owns deferred range work: no partial selection is
published and later Copy waits behind it. Each frame budgets range work to two
milliseconds; each advance visits at most one bounded fragment per endpoint.
Navigation/projection/layout/owner retirement uses the existing shared cancellation.
The existing local six-point/300ms click tolerance remains in effect.

Occurrence callbacks use the original clicked byte, independently of the selected
range's focus. Double-drag still toggles the original occurrence, while a triple
click leaves its pin unchanged. Existing modified-click and definition-navigation
behavior remains available; exhaustive modifier/native-platform gesture acceptance
is outside this particular check.

An actual-app test exposed an additional integration defect: the first pin inserted
a row above the source, so the third click reached different UI. The existing pin
controls now render in the measured footer, without changing their actions or type
styles. The source stays under the pointer when pins appear/disappear. No dependency,
worker, alternate event router, or sibling change was added. The reusable text API
remains independent of egui and Radiology stays standalone. Requested Swift/macOS/
typography skills are unavailable in the installed skill catalog; no font roles
were changed.

## Verification

- Core tests cover Unicode identifiers, spaces/tabs, punctuation, emoji/keycaps,
  hidden joins and a huge identifier requiring thousands of bounded advances.
- Viewport tests cover 1×/2×, wrapping, the final half of an identifier glyph,
  word/line dragging forward and backward, exact original CRLF copy, folded hidden
  source, deferred Copy and focus-owner cancellation.
- Actual-app frames at both scales prove word Copy, pin creation, third-click line
  Copy without an extra toggle, and later double-click unpinning. This test failed
  before moving the pin row to the footer and passes afterward.
- Workspace: 638 app tests pass, seven ignored, zero filtered; two binary, 13 font,
  21 text and 42 doctests pass. Strict Clippy/check/build/rustdoc, shaping-feature
  checks, formatting and diff checks pass. Rendered `WordRangeCursor` HTML and a
  copied executable's resources outside the checkout pass.
- Native standalone previous/current text fixtures at 1×/2×: old collapsed
  selection reproduced; current word, paragraph, forward/reverse word and line
  drags and light-mode word Copy pass (seven fresh exact copies per scale).
- Native Rust fixtures verify triple-click line Copy while pinned, double-click
  unpin/re-pin, and pointer Clear Pins in the footer. Source/binary hashes stay
  unchanged and app/compositor exits are zero. The first 1× pin harness stopped
  because OCR joined `Clear Pins` into `ClearPins`; v2 accepts either spelling.
  This harness correction changes no product code.

## Architecture review

Baseline `before-source-click-selection` at `0d6e39a`. Final measurement: +530 NLOC,
+17 functions, one UI module; average CCN stays 3.34, maximum stays 132, no cycles,
health approximately -0.01. Initial production clones in grapheme iteration and
input reset were removed through reuse; the pointer application hotspot was
resolved by making gesture completion its own cohesive operation. The comparison
still reports REGRESSION: `WordCursor::visit` adds one necessary selection-mode
branch (CCN 15→16), and clones grow 109→110. Normalized comparison proves the only
new clone is six lines of shared long-word test setup; four reported app clones
are unchanged code shifted by the new test. Retaining two distinct ordering/owner
regressions is clearer than adding a test abstraction solely to alter the metric.
No new production clone or dependency cycle is accepted.

Evidence archive: `~/.local/state/radiology-qa/click-selection-20261102-0d6e39a`.
Broader Swift feature/UI/corpus audit and native IME-engine acceptance remain open;
this gesture fix does not establish complete parity. Full accessibility and actual
little-screen-clip integration retain their explicitly deferred scope.
