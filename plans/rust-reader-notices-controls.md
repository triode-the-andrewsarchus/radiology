# Rust reader error notices and PR control availability

## Bounded error presentation

The reader retains its complete recoverable error string. A reusable ErrorNotice
component presents one truncated row with Details and Dismiss actions, reserving
the actions before sizing the summary. The summary takes the first nonempty line
and at most 240 Unicode scalar values, adding an ellipsis for omitted content.
Details opens a size-constrained, scrollable window with selectable full text and
Copy Details. Copy preserves the complete original string. Dismiss or clearing the
reader error closes the details window. Existing interface text roles remain.

The component owns presentation only; repository, Git, index and source error
producers retain their existing diagnostics and recovery behavior. The app's
public error state remains compatible with hosts.

## Available revisions and change navigation

SourceView receives the existing FileChange metadata. Before and After controls
use path_for_revision, including Added, Deleted, Modified and Renamed cases. An
unavailable side is disabled and explains why on hover; the selected side does not
emit another action. Keyboard revision commands also validate availability before
requesting source, so an unavailable toggle neither starts a doomed read nor
replaces the reader with an error. Explicit file-read failures remain reportable.
The header uses the actual change status instead of labeling every file Modified.

ChangeNavigation captures the existing previous/next hunk targets for a caret and
revision. The global toolbar, source breadcrumb controls and command dispatcher use
those same primitives. Controls disable at the relevant boundary or when there is
no source/change target. Zero-length addition/deletion anchor behavior is retained.
This does not change hunk navigation semantics or add another mapping algorithm.

## Evidence

Full gates on 2026-10-30 pass 396 app tests (one ignored), eight shared reader tests,
30 doctests, formatting, strict Clippy, build and rustdoc. Added tests check narrow
notice geometry across dark/light and 1×/1.5×/2× scaling, long Unicode/multiline
summaries, actual Details/Copy/Dismiss interactions, complete clipboard payload,
revision-button clicks and rejected unavailable keyboard commands preserving reader
and history. Shared pointer/label test helpers also replace an existing duplicate
caller-click fixture. Logs: `/tmp/radiology-reader-controls-{all-tests,clippy,build,doc}.log`.

Architecture baseline `before-reader-notices-controls`: source-view complexity is
reduced below its prior hotspot threshold; there are no new cycles or clone growth.
Comparison flags two unchanged baseline source-view test fragments (eight/seven
lines) in a touched file. New click-fixture duplication was consolidated into the
shared UI test support.

A full run also exposed a pre-existing timing assumption in the tree-retention
test. Separate commit b7a1b1f checks the documented next-preparation sweep instead
of requiring best-effort drop-time maintenance to finish while another writer may
be busy. Active lease, reuse, bound and LRU checks remain; production retention is
unchanged.

## Native acceptance

Core implementation `d602a49` plus the summary-alignment follow-up passed a private
1280×900 Wayland sequence. Two modified hunks navigate to source lines 10 and 40;
global/source arrows visibly disable at the corresponding boundaries. Added
new.rs keeps Before disabled; deleted b.rs opens in Before with After disabled.
Both pointer attempts and Ctrl+Alt+B preserve each accepted reader without an
error. The header correctly reports Added and Deleted.

A controlled metadata failure emits 1,200 diagnostic rows. Its notice remains one
left-aligned row with Details and Dismiss. The bounded Details window scrolls from
the initial rows to later rows. Copy Details produces the exact complete 45,793-byte
reader message, including all rows and the final marker; its SHA-256 is
`a31325811c88a6d8c96305c58295abbf324fc41febcbee0ebd9450158e9a1353`.
Dismiss closes both notice and details while keeping the Before reader; fresh PR
refresh recovers. Checkout HEAD/status/source bytes compare unchanged. Final app
and compositor exit statuses are zero.

The first native capture also exposed centered summary text; the follow-up aligns
it with other reader notices. Two focused notice tests and strict Clippy pass after
that fix and explicit dark/light context setup; build passes after the production
alignment change. The full 396/eight/30 gates and rustdoc passed before this small
layout follow-up. Broad native scaling/accessibility acceptance remains open.

Artifacts: `/tmp/radiology-reader-controls-native-accepted/manifest.json`,
`acceptance.json`, `clipboard-proof.json`, `copied-details.txt`, and hashed captures
`first-change.png`, `last-change.png`, `added-before-disabled.png`,
`added-unavailable-retained.png`, `deleted-after-disabled.png`,
`deleted-unavailable-retained.png`, `bounded-long-error.png`, `full-details.png`,
`details-scrolled.png`, `dismissed-reader-retained.png`, and `recovered-reader.png`.
Control and geometry observations are visual checks; clipboard equality and
checkout preservation are exact comparisons. Earlier QA sessions were retired by
harness errors involving clipboard MIME inference and an unsupported scroll amount.
The final session explicitly requests text, waits for publication, uses the helper's
supported range and exits cleanly. Sibling capture helpers remain QA-only; the app
has no sibling runtime dependency. Fixtures make no remote PR writes.

Full parity remains open: complete index provisioning/project/version reuse, broader store recovery,
and final visual/accessibility/scaling/font and sibling reuse acceptance.
