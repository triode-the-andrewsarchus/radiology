# Cached caller data and keyboard control retirement

The fresh serial workspace run for 8fa696f exposed one failed keyboard test:
888 application checks passed, one failed and eleven were opt-in ignored. That
run stopped before the other crates/doctests and is not a full success. Archive:
~/.local/state/radiology-qa/caller-disclosure-20261109-b959b1f/workspace.log.

Loaded-tree reuse was correct, but the viewport retained caller control IDs after
collapse until the next paint. Reopening cached data within the same event batch
made those old IDs eligible again. Left/Right/Right could enter controls retired
by the initial Left before the reopened tree had been painted.

ReferenceViewport now retires the collapsed reference's caller controls, pending
caller focus and reveal request immediately. Both explicit-row and selected-row
collapse actions use this operation. Other references' controls remain available.
The existing paint path publishes fresh controls, so cached rows remain usable
with the next caller-entry command. This adds no new state field, revision key,
query or renderer, and does not delay or restart validated data reuse.

The old collapse regression and caller-entry navigation shared an entire command
and history sequence. One four-case matrix now covers both prepared-host and
normal event routing, each with an initially expanded or collapsed/reopened tree.
It checks immediate loaded reuse, parent focus through the coalesced retirement
batch, unchanged reader Arc, subsequent Right/Down/Enter into entry and exact Back
caret restoration. No assertion was removed to accept the failed behavior.

All five focused app reference-keyboard checks and eight reference-list component
checks pass. Strict all-target Clippy and the normal build pass. A fresh full
workspace run will combine this repair with the following source-registry
checkpoint; no later full success is claimed yet. Logs:
~/.local/state/radiology-qa/caller-keyboard-20261109-8fa696f/.

Architecture adds 23 NLOC, one small viewport retirement method and removes one
overlapping test function through the expanded matrix: net function count stays
4562. Average/max CCN stays 3.42/132, with no new hotspot/cycle. Clone count remains
125; complete normalized clone multisets have zero additions/removals, so the
raw touched-file flag concerns an existing test fragment. The UI
uses the same controls and typography. Native final acceptance remains open.

Fresh complete-workspace evidence at 2e089e9 closes this repair: see the [combined gate record](rust-buffer-outline-contracts.md#verification-and-structure).
