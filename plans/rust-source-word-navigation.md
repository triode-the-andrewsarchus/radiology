# Source word and paragraph navigation

## Confirmed gaps and behavior

Swift's ReaderTextView delegates ordinary text bindings to NSTextView. Rust rejected
Ctrl+arrows before its source movement queue. The native old control against
01dda3f confirms word and paragraph commands leave the selection unchanged.

The Linux source reader now accepts Ctrl+Left/Right for word movement and
Ctrl+Up/Down for paragraph movement, with Shift extension. The primary-command
modifier follows the same policy. Alt combinations retain their existing owners.
This is a portable source-reader policy, not proof of every locale-specific
inherited AppKit binding:

- Unicode letters/numbers and underscores form identifier runs, including their
  combining marks. Camel case and underscores do not introduce subword stops.
  Whitespace and ASCII punctuation are skipped between words. Other graphemes,
  including compound emoji, are individual stops; keycaps remain atomic.
- Right finishes the current/next run; Left reaches its beginning. Fragment
  continuations do not create word boundaries. Visible line breaks separate runs;
  hidden joins are hard boundaries with the existing surviving-byte ownership.
- Up/Down first reaches the current projected logical row's beginning/end. From
  that boundary, it reaches the previous/next logical row's boundary. Blank rows
  are included. Wraps and fragment limits do not create paragraphs; a folded row
  retains its joined visible paragraph. Shift keeps the original selection anchor.
- Movement and Copy use original canonical bytes, including hidden source and
  exact CRLF. Plain Right at EOF also now stays there: the old fallback incorrectly
  returned the beginning of the last fragment.

## Shared mechanism and ownership

The GUI-independent reader-text crate exports WordCursor. It owns only progress,
not source text, threads or framework objects. Each advance visits at most one
host-bounded ProjectedFragment. Identifier runs and skipped separators may span
many fragments without forcing an unbounded scan in a frame. Public docs include
an executable isolated-host example and require the same immutable source and
projection throughout an operation. Existing oversized-grapheme fragmentation
retains its bounded scalar fallback; this is not unrestricted text shaping.

The source viewport retains operation origin/direction and advances under its
existing two-millisecond work budget. No partial destination is published.
Movement, bookmark/definition commands and Copy remain ordered in the existing
input queue. Projection/source changes, layout invalidation, overriding pointer
input and input-owner retirement discard progress with the queued input. No new
worker pool, source clone, normal dependency, lockfile change or Cargo setting is
introduced. Paragraph movement reuses the existing source Home/End and adjacent
fragment navigation. Shared selection/caret/reveal publication stays unchanged.

## Appearance availability repair

Native 2× testing exposed an additional command-policy defect. Changing appearance
queues source recoloring; source_location_ready treated every source-loader job as
an incoming navigation and temporarily disabled Go to Line. Waiting for a modal
could not recover a command already rejected as unavailable.

Availability now follows pending_navigation over the accepted source. Appearance
work keeps source commands usable; an actual incoming source still blocks them
until accepted. A regression checks Go to Line, Find, bookmarks and folds during
recoloring, stable accepted buffer/caret after publication, and retirement when a
real file navigation starts. Existing loader status and cancellation remain.

## Verification

Three shared-text matrices cover Unicode identifiers/accents/CJK/emoji, forward
and backward terminal states, CRLF, hidden joins inside/across fragments and
cooperative huge identifier/separator scans. Three viewport matrices cover
1×/1.5×/2× with wrap on/off, exact selection, logical paragraphs/blank rows/folds,
EOF and trailing-newline copy, deferred bookmark-before-Copy ordering and owner
retirement. The appearance
availability regression covers the native-discovered failure.

Workspace: 611 app tests pass, seven ignored; two binary, 13 font, 19 text and 40
doctests pass. Strict Clippy/check/rustdoc, app build/fmt/diff pass. Rendered public
WordCursor docs were inspected. An independent offline CPU host exercises bounded
progress with only reader-text, no Radiology/egui dependency, and 16 build jobs.
The final copied standalone binary verifies all resources.

Native v1 old control at 1× proves both ignored bindings. Final v8 1×/2× each pass
25 fresh exact clipboard checks: Unicode identifiers and atomic symbols forward/
backward, CRLF crossings, folded/blank paragraph selection, a long-word movement
followed by bookmark and Copy in one batch, EOF, modal return, light appearance and
wrapped long-word movement. Captures are inspected. Original fixture/binary hashes
stay unchanged and every final process exits zero. Early v1/v2 after-runs are
retained separately: their 2× failure exposed the recolor availability bug. V2 adds
an observed Go to Line readiness check; v3 repeats it after the product fix.
Subsequent targeted captures verify completed long-word selection after clipboard
publication; cropped modal OCR avoids interference from dense selected text. V8
repeats the full matrix after the trailing-newline boundary fix, with completed
long-word captures and observed initial source/modal readiness.

Architecture against before-source-word-keys: +497 NLOC/13 functions/one module
including tests, no new hotspot/cycle, health -0.03. Default comparison reports
REGRESSION for four old app-test clones; the complete normalized audit confirms
all 109 clone groups unchanged, zero new/removed groups. Initial new fixture/event
boilerplate duplication was removed through shared syntax and request data.

Evidence: ~/.local/state/radiology-qa/word-keys-20261102-01dda3f.

## Remaining scope

Locale-specific AppKit word segmentation, the full inherited text-binding audit,
broader selection/focus transitions and general Unicode/bidi/UI-label rendering
remain unproven. The full feature/UI/corpus review remains open. Full accessibility
and actual little-screen-clip adoption stay deferred; the public reader-text
contract prepares reuse without requiring the sibling checkout.
