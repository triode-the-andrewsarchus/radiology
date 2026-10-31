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

Native error-details/revision/navigation acceptance is pending. Full parity remains
open: complete index provisioning/project/version reuse, broader store recovery,
and final visual/accessibility/scaling/font and sibling reuse acceptance.
