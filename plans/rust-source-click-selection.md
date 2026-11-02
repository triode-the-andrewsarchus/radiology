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
click leaves its pin unchanged. The initial plain-gesture change retained existing modified-click behavior; the
follow-up modifier audit and repair are recorded below. Exhaustive native-platform
gesture acceptance remains outside these representative checks.

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

## Modified source gestures — 2026-11-02

A follow-up audit found that Rust reset click counts for every modified press and
release. Swift's mouse handler delegates those presses to native text selection:
its command branch suppresses occurrence callbacks, while Shift/Option gestures
retain them. Rust's modifier restriction prevented word/paragraph selection, and
it evaluated occurrence policy using mouse-up flags rather than the owning press.

All source presses now participate in the existing local click sequence. The
existing gesture stores its captured press modifiers. Shift keeps the existing
selection anchor when extending by words/paragraphs; releasing the modifier before
mouse-up does not change the gesture. Alt and Shift double-clicks retain pinning.
Primary-command gestures select text without pinning. Definition navigation is
limited to the first primary click, including when semantic data arrives between
clicks; subsequent clicks select text instead. The established Linux Ctrl/primary
mapping and Alt/Shift exclusion for definition navigation remain unchanged.
There is no new input mechanism, public API, helper, dependency or UI/type change.

The initial modifier regression failed on Shift's missing double-click pin. It was
then folded into the existing multi-click matrix to reuse the same fixture and
right-half-of-final-glyph probe. The matrix now covers seven modifier combinations,
1×/2×, wrap on/off, exact Unicode/CRLF copying and modifier release before mouse-up.
The existing indexed-click matrix additionally checks late definition publication
for Ctrl and primary-command presses, with a positive CaretMoved assertion after
release and exact `helper` Copy. Existing single-click navigation cases still pass.

Workspace: 638 app tests pass, seven ignored/zero filtered; two binary, 13 font,
21 text and 42 doctests pass. The strengthened late-definition case passes after
the full run. Strict Clippy/check/build/rustdoc with shaping, fmt/diff and copied
standalone resources pass. Native previous/current binaries at 1×/2× reproduce
seven old incorrect selections and verify seven new exact fresh clipboard results
per scale. Separate Rust fixtures verify six exact copies and the expected pin
presence/absence through Shift/Alt/Ctrl word/paragraph gestures at each scale.
Captures are inspected, fixture/binary hashes stay unchanged, and all final child
processes exit zero.

Architecture baseline `before-modified-source-clicks` at `3842ed1`: +47 NLOC,
unchanged files/functions/average/max complexity and dependency cycles; health
approximately -0.01. Initial new test duplication was removed by extending the
existing matrix and strengthening the late-definition action assertion. Default
comparison still labels the shifted old long-word fixture clone a REGRESSION;
complete normalized comparison proves all 110 clone groups unchanged, zero new or
removed groups. No production hotspot was introduced.

Evidence: `~/.local/state/radiology-qa/modified-clicks-20261102-3842ed1`.
This proves the documented portable modifier policy, not every platform's inherited
text interaction. Broader feature/UI/corpus and native IME-engine acceptance remain
open; full accessibility and actual sibling adoption remain deferred.
