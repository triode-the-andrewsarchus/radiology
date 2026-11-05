# Selection-aware reader Return — 2026-11-05

## Reconciled Swift contract

`ReaderReturnKeyTests` and `ReaderTextView.keyboardDefinitionRoute` distinguish a
plain reader Return from the general definition command. A nonempty selection must
fit within one indexed occurrence. Its departure is the selection's lower byte,
independent of selection direction. An empty caret just after an identifier may
resolve the preceding occurrence, while keeping the caret's exact departure byte.
Modified Return, absent/external/self-only definitions and Before source fall
through without navigation or selection mutation.

A new actual-app red regression proves Rust previously followed the caret through
a selection spanning leading whitespace and an occurrence. SourceInput now retains
a distinct plain-Return event and evaluates it after preceding reader input. The
existing canonical ByteSelection range, indexed byte conversion and queued input
owner determine one query/departure. A successful route emits the new additive
`SourceViewAction::FollowDefinition(DefinitionDestination)`; library hosts should
handle that exact destination through their existing indexed-range navigation.
The app does so through its existing accepted-publication/history transaction.
The ordinary definition shortcuts retain their command route.

DefinitionDestination now shares the inspector's validated, canonical non-self
choice between the inspector and Return. This supports multiple definition facts
without sending Return back to the same declaration. The route retains the live
selection until accepted navigation clears it, and failed/invalid target lookup
leaves the selection intact. Existing input retirement prevents later coalesced
keys from running against the old source after a successful Return.

## Verification

Temporary execution: `~/.cache/radiology-qa-reader-return`.
Durable evidence: `~/.local/state/radiology-qa/reader-return-20261105-fbf6872`.

- The app red regression navigates from b.rs to a.rs despite an over-wide selection;
  the final route retains b.rs, exact selection Copy and history. F12 still follows
  the same current caret after that rejected Return.
- An app matrix at1×/2× follows exact word selections and end carets, then Back
  restores the word's lower byte or exact end-caret departure respectively.
- A measured-reader UTF-16 fixture after emoji/accented text covers inside/end
  carets, exact and over-wide selections, leading whitespace, external and self-only
  symbols, plain whitespace, modified Return and Before. Selection remains unchanged
  when the view emits an action and when it rejects the event.
- Native1×/2× uses the recorded Swift Kotlin compiler SCIP with controlled tools/PR
  metadata and real Git. Raw Shift+arrows selects `store.read`; Return preserves it.
  End-caret and reversed `read` selections follow the exact Store overload; Back
  restores their distinct bytes, verified by Copy. Before retains its original
  source. Seven exact Copy checks per scale, inspected captures, unchanged attached
  file/HEAD/ref/status and executable hashes; core processes exit zero. This replays
  recorded compiler facts and does not claim fresh Android compilation.
- Full locked workspace:777 app tests/11 opt-in ignored, two binary,13 font,22 text
  and60 doctests. Strict workspace/all-target Clippy, locked build, resource smoke
  and format/diff checks pass.

Architecture baseline `before-reader-return-selection`: +211 NLOC/five functions
(two production routing/target helpers, three tests). A pre-existing UTF-16 document
fixture constructor moves into existing semantic test_support and is reused by the
context and viewport tests. All125 clone instances/1335 duplicate lines,36 CCN>15
hotspots, one cycle and eight structural findings remain unchanged; the full clone
Counter has zero additions/removals. Shared destination helper CCN2; Return router
CCN12 reflects required target/selection/EOF/revision guards. Raw diff-aware reports
flag unchanged old self-clones and Swift Python rank noise; there is no added
CCN>15 hotspot or duplicated mechanism. No new dependency, worker or persistence.

## Remaining audit

The [behavior-suite evidence index](rust-behavior-suite-index.md) inventories81
root Swift files and334 annotated functions, separating ordinary suites, external
probes and support. It maps all66 ordinary suite files to Rust entry points and
existing evidence; those links alone do not prove every assertion has been reconciled.
Continue the remaining assertions and final standalone/library-host audit, including
handling the new SourceViewAction variant. Full accessibility and actual sibling
adoption remain deferred by user scope.

No Android SDK was found. A private Platform36/Build Tools36.0.0/command-line setup
is prepared from Google's repository metadata (about296 MiB download). The user's
license authorization is pending; no SDK package was downloaded or license accepted.
Google requires agreement before downloading; see the
[SDK terms](https://developer.android.com/studio#terms-and-conditions).
