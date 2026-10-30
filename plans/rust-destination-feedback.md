# Transient destination feedback

Swift's contracts are `SourceViewerCoordinator.flashDestination`,
`ReaderTextView.drawLine`, the highlight revision in `SourceViewer`, and jump
activation in AppModel/semantics/bookmarks/PR navigation.

## Implemented behavior

The destination receives an 18% accent wash for 900 ms, followed by a 210 ms fade.
The band covers the shaped visual text row, across the code surface. Wrapped
continuations share canonical bytes but only the row containing the destination
flashes. Empty source and CRLF retain usable geometry and original coordinates.
Diff, selection, pin, find and caret decorations keep their existing ownership;
feedback does not move the caret, change copy text or replace selection.

The cue is bound to path/revision and SHA-256 content identity. Its clock begins
on first paint after a successful navigation publication. Pending/failed reads
cannot start destination feedback. New cue requests replace/restart the old cue;
source identity changes, ordinary committed navigation, outline previews and
review/repository retirement cancel it. Ordinary pointer/caret motion can leave
it at the landing byte until expiry, matching Swift's fixed landing indicator.

When egui's animation time is zero, the cue holds for 900 ms and clears without
fading. Native OS reduced-motion preference integration remains part of the
broader accessibility audit. Only the hold boundary and short fade schedule
repaints; the effect does not run a timer/thread or continuously repaint at rest.

## Activation and architecture

- Semantic definition/under-caret/workspace-symbol/reference destinations share
  the exact indexed-navigation path and request feedback. Validated caller
  commits request the same effect with their existing source-digest guard.
- Bookmarks request feedback after anchor relocation and fresh source loading.
- Change-block navigation and the enclosing-declaration inspector link request it.
- Outline name clicks explicitly activate feedback. Arrow previews and outline
  Return commits retain Swift's separate behavior and do not flash.
- Breadcrumbs, ordinary file/line navigation, find, history and revision switching
  keep the default navigation intent and cancel a prior cue without starting one.

NavigationIntent carries the explicit policy, and apply_destination requests the
cue only after successful resolution/publication. The existing navigate_target
uses that intent rather than adding ambiguous positional policy booleans.
`ui/source_view/destination.rs` owns small deterministic source/time state;
SourceViewport owns sampling and uses the existing fragment owner/cursor geometry
for painting. No source parser, coordinate model, dependency or sibling runtime
requirement was added. Public viewport request/cancel methods are usable by a host.

## Automated and architecture evidence

Commits `6d86935` (controller/renderer), `00d8eb5` (navigation binding) and
`078070a` (distinct path/revision identity) pass
292 app tests, 5 reader-text tests, 30 doctests, fmt, Clippy with warnings denied,
binary build and rustdoc. One external-Clang-dependent test remains ignored.
The identity regression also rejects a literal checkout `before:a.rs` against
Before-revision `a.rs`, even with identical bytes. Path and revision compare as
distinct values rather than a formatted position key.

New tests exercise first-paint clock/hold/fade/restart/disabled animation,
path/content/revision rejection and cancellation; one-band rendering within a
long wrapped/fragmented Unicode line; preservation of caret/selection; preview
cancellation; and visible empty/CRLF destinations. Native checks below cover
success/failure and different host activation paths.

Architecture baseline `before-destination-feedback` is at `82a8ece`. Comparison
has no new hotspot/cycle, clone group or dependency. Product/test NLOC increases
by 290 in one component module and existing owners. Average CCN remains 3.34,
maximum 100, clone groups 91; duplication decreases from 0.69% to 0.68%. The touched-file report labels REGRESSION
for existing short source-view test fragments; inspection found no duplicate
product mechanism. Existing shaped byte/fragment geometry supplies the effect.

## Native evidence

The extended native UI checkpoint at `00d8eb5` has SHA-256
`b91c12a617f2ce2f556a534c3eb8aa7dbe4920e6a07634aa6d0dadf54c0c75c8`.
Two private Wayland fixtures use the existing little-screen-clip test/compositor
helper; product operation remains standalone. Their controller adds bounded
capture delays to observe the short cue. Build/capture hashes and pixel
observations are retained under each fixture as `build-evidence.json` and
`pixel-evidence.json`. All app and private compositor exits are zero; pointer
helper termination is intentional.

`/tmp/radiology-destination-outline-native`, PID 4001493:

- outline-active.png / outline-expired.png: a name click flashes line 3, then
  expires. At (800,111), RGB changes from (24,52,80) to the normal caret band
  (27,34,42); a control row remains (28,28,30). Capture delays are 150 ms after
  activation and another 1.2 seconds before expiry capture.
- repeated-active.png / preview-cancelled.png / keyboard-commit.png: repeat
  activation restarts the cue, Down cancels it while preserving the source caret,
  and Return commits the selected declaration without a cue.
- bookmark-active.png / bookmark-expired.png: Ctrl+0 rereads and opens b.rs,
  flashes its resolved first line, then expires. The matching pixel pair is
  (24,52,80) → (27,34,42).
- bookmark-light-active.png / bookmark-light-expired.png: the same fresh-read
  activation works in light appearance, (198,225,255) → (241,248,255).
- bookmark-failed.png: removing only the fixture's b.rs and activating the saved
  bookmark reports the missing-file error, retains a.rs at line 8, and produces
  no destination cue.

`/tmp/radiology-destination-semantic-native`, PID 4007310:

- semantic-origin.png: exact save use after café/emoji in a.rs; SCIP coordinates
  are UTF-16 while source/caret coordinates remain canonical bytes/scalars.
- definition-active.png / definition-expired.png: F12 resolves and reads the real
  b.rs definition, flashes line 1 at its exact name, then expires. Here/declaration
  facts describe that actual destination.
- semantic-back.png: Back restores the original a.rs use without a new cue.
- definition-repeated.png / back-interrupts-feedback.png: a repeated F12 cue is
  canceled by Back during its hold, restoring the original use.

Following the path/revision hardening, the final `078070a` build is additionally
checked in `/tmp/radiology-destination-identity-native` (PID 4019090), SHA-256
`d0ae2b313505433eba845f8f7dd249e1468b6443c7b8edf06404d5894bc9b163`.
`active.png` and `expired.png` repeat the pointer landing/expiry with the same
(800,111) pixel transition. `build-evidence.json` records this checkpoint and
hashes; all app/compositor exits are zero. Extended bookmark/semantic captures
above retain their precise prior checkpoint rather than being relabeled.

## Remaining scope

This covers the implemented effect and listed fixture paths. Broad pinned-review
reference/caller, scale, reduced-motion preference, long-language-corpus and
side-by-side Swift visual/accessibility acceptance remain in the full audit.
Audit pre-existing formatted position keys during the wider cache/lifecycle work.
Enclosing-function body shade, finer folding/import presentation, request/revisit
caches, background repository work, indexing/tool/PR lifecycle and shared palette/
chrome reuse remain unfinished portions of the overall objective.

The requested swiftui-pro, macos-design and typography-designer skills remain
absent from configured roots. Swift behavior and existing Rust colors/type styles
were used; no new type scale was introduced.
