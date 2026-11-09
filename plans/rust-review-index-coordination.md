# Coordinate automatic indexing with review preparation

Fresh native PR acceptance exposed a startup defect after automatic indexing was
added: an explicit --pr launch generated an index for the dirty attached checkout,
then generated the intended pinned After index. Actual scip-clang logs and normalized
reports showed two compiler driver calls: checkout revision None/two documents,
followed by the four-document pinned review. Existing review publication retires
prior index ownership, but the checkout compilation is unnecessary for the request.

## Fix

The existing automatic-open coordinator waits while pending_pr identifies review
preparation. Cache/inspection jobs can finish and existing data can publish; the
continuation does not import or start new generation for the superseded scope.
Accepted review publication already retires the old open and loads the pinned
After index. Failure or cancellation clears pending_pr, allowing the accepted
checkout's existing automatic open to continue. No separate deferred-intent flag,
queue, worker, cache format or startup-only path is added.

The same coordinator now reuses is_index_loading for its activity gate instead of
repeating settings/generator/plan and import checks. This also waits for ongoing
explicit tool probes/provisioning. Import completion relies on the same shared
activity predicate. The new condition initially increased coordinator CCN from
fifteen to sixteen; consolidating actual job readiness reduces it to thirteen,
without moving the branch into an arbitrary wrapper.

## Actual regression

launch_review_waits_before_indexing_and_resumes_the_resolved_scope uses the existing
real-Git before/after fixture and controlled Rust process boundary. It installs the
exact Rust source corpus into the review, dirties the attached lib, launches with
an explicit PR/file and holds the real review completion boundary. It drains
settings/cache/planning work and proves the compiler has not run while review
preparation remains pending. Against the old implementation it fails with checkout
compiled while review pending, matching the native observation.

Three actual outcomes then verify one compiler run in the resolved scope. Accepted
review generation records the exact head revision and copies pinned library bytes
into the compiler input log. Failed/cancelled preparation instead indexes the
accepted local checkout, with no review revision and the exact dirty checkout input.
The original dirty source stays unchanged. Failure retains its review error; cancel
retires it. Existing settings recovery, repository/session and real CLI recovery
cases remain passing. This checks scheduler/publication behavior and process inputs,
not just the new predicate.

## Checkpoint validation and remaining work

All twelve app lifecycle, six personality, ten repository and two review-recovery
cases pass, including actual isolated CLI recovery. Strict all-target Clippy,
formatting and normal workspace build pass. Latest complete serial workspace gate
remains the preceding snapshot's 942 app checks/twelve ignores, binary/font/text
2/13/23 and all 68 doctests. A new full gate and fresh native integrated PR run must
verify this runtime change before final acceptance. Prior standalone/native proofs
retain their audited e49c5ec source scope; affected export/window checks need refresh.

Architecture compares NEUTRAL: +73 measured NLOC/net one analyzed function, no new
file; avg/max CCN 3.43/132, all 36 high-complexity functions, one cycle and all 125
normalized clone instances unchanged (zero additions/removals). No public API,
dependency, stored setting, source styling or Swift source change is introduced.
Requested SwiftUI/macOS/type skills remain unavailable; existing Rust lifecycle
mechanisms are reused.

Regression archive: ~/.local/state/radiology-qa/launch-index-coordination-20261109-8f48ec5/.
The first fresh native observation is retained, explicitly unaccepted, under
~/.local/state/radiology-qa/final-native-pr-20261109-8f48ec5/.
Complete ledger review, integrated native rerun and SDK-backed Android success
remain open. Full accessibility and actual sibling integration remain deferred.
