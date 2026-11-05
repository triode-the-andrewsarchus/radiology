# Pending source context and recovery — 2026-11-05

The [function-shading audit](rust-function-shading-audit.md) recorded one remaining
context difference: a new file/root request kept the old function wash visible.
Swift clears that transient context when preparation begins. The expanded app
regression fails against the previous Rust renderer at this boundary.

`enclosing_source_context` now derives one readiness rule from existing source,
repository and Code Path owners. The source breadcrumb/function wash and automatic
inspector declaration/token details use that same rule. During destination work,
the accepted bytes remain readable but transient caret context is absent. Cached
outline navigation is disabled and the inspector prioritizes its existing
“Reading structure…” state over guidance to move the caret.

Cancellation or failure restores the accepted context automatically. Successful
publication presents the destination's own context; Back restores the original
exact caret. Appearance-only work keeps context visible. Existing accepted source,
fold choices, viewport, history, explicit references and bookmarks retain their
owners; there is no extra pending flag, persistence, public view field or host API.
Two duplicated enclosing-symbol traversals are replaced by the shared derivation.

This closes the pending **caret-context** discrepancy. Retaining the accepted
reader for recovery remains the Rust transaction behavior; it does not imitate
Swift's wholesale source teardown during loading.

## App and native evidence

Expanded `function_wash_follows_caret_and_clears_with_accepted_documents_and_repositories`
holds a source result at the existing publication boundary, then checks empty
wash/declaration/loading text, exact retained source/folds/history, cancellation
recovery, missing-file failure and successful Notes publication at1×/2×. A gated
repository result tests immediate context suppression, failure recovery and later
successful root replacement. Rehighlight retains context. The nested Code Path
visit regression checks absent context while its existing job owns preparation
and the correct Unicode target after publication.

Native verification also exposed a line-prompt focus acquisition defect: visibility
alone completed its initial-focus flag. A new inactive-window regression fails
before the fix; Go to Line now records acquisition only for a visible, enabled,
actually focused editor. Keyboard submission after activation and the reused
native prompt workflow pass without changing later Tab ownership.

The existing narrow-inspector matrix now gives loading precedence for a parsed
buffer as well as plain/error cases. Its dark/light, scale and width checks remain
active. Current-function/shade and keyboard suites continue passing.

Native private-compositor fixtures use real Git commits/objects with a controlled
wrapper around one pinned `cat-file blob` read. The harness confirms the gated
child PID is live before capturing pending UI, clicks the visible Cancel control,
and verifies child retirement. A separate controlled read failure and successful
release check both recovery paths. At1×/2×, captures show no old wash/breadcrumb /
automatic declaration during preparation, restored area context on cancel/failure,
the new destination after success, and exact Back. Four Copy checks per scale
verify the old call line after failure/Back, accepted destination and full source.
Attached files, Git HEAD/refs/status and executable hashes stay unchanged; accepted
child exits are zero. An initial OCR-harness failure is retained separately.

## Checks and architecture

Locked workspace:789 app tests pass (11 opt-in ignored), two binary tests,
13 font tests,22 text tests and60 doctests. Strict Clippy/rustdoc, formatting,
binary/resource smoke and the independent reader-host consumer pass. Host APIs,
dependencies and source-bound range validation are unchanged.

Baseline `before-pending-source-context`: one shared helper replaces duplicate
render logic; final measured NLOC grows94 with two functions (helper and focus
regression). Average/max CCN remain3.40/132. The existing renderer moves37→38 for
pending/loading presentation, while the shared context helper is6 and the line
prompt is11. No new clone, hotspot class, cycle or structural issue; full clone
file-pair/fragment counters retain125 instances/1335 lines. The existing renderer's
small complexity change reflects readiness/loading presentation; the new helper
stays small. Raw warnings identify unchanged search-test clones and equal-score
Python ranking churn. Final detailed deltas are retained with the measurements.

Evidence: `~/.local/state/radiology-qa/pending-context-20261105-4ea88aa/`;
working fixtures/logs: `~/.cache/radiology-qa-pending-context/`. Other Swift assertion
reconciliation and SDK-backed Android validation remain open. Full accessibility
and actual sibling adoption remain deferred.
