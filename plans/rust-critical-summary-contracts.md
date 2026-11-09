# Critical summary and hover contracts

Reference: four `CriticalSummaryModelTests` groups, two
`HoverPopoverInteractionTests` groups, and the first of five
`CallGraphWorkerTests` groups, compared with Rust `8441a22` plus this checkpoint.

## Runtime corrections

`CriticalAnnotations::start` now identifies exact folded callable bodies before
allocating progress state or scheduling a worker. Abstract declarations, one-line
methods and container-only folds retire old counts immediately without work.
The private selector owns the previous body-range matching logic; computation
still uses the existing bounded read pool, cancellation and six-entry batches.
`red.log` records the prior unnecessary-running state for a source with no consumer.

Hover transit now follows Swift's anchor/popup union expanded by six points.
Source geometry reuses `for_each_match_rect` and the existing emoji-aware galley,
so the anchor covers the occurrence's actual glyphs. Popup serial ownership freezes
both origin and anchor until a replacement completes dwell. This prevents a pending
different callable's anchor from capturing transit and indefinitely retaining the
old popup. Actual popup hit testing remains separate, allowing clicks on the source
or transit gap to dismiss. No new public type, dependency, worker or navigation
mechanism was added.

`transit-red.log` records the popup closing while the pointer pauses in the gap;
`transit-ownership-red.log` records pending-anchor capture. The final source-frame
and popup ownership tests pass both corrections.

## Summary model assertion mapping

| Swift group | Rust evidence |
| --- | --- |
| `modeKeepsContextReferencesStableAndCaretRequestsNoSummaries` | New both-scale app test `summary_modes_preserve_loaded_references_and_caret_moves_keep_count_inputs` imports the same Rust-analyzer fixture as Swift through the production importer. Critical/Normal/Critical preserves loaded references, selection, expanded reference content, definition expansion, declaration/definition identities, source Arc, exact caret and history. Normal clears counts; Critical produces the fixture's count of one. Actual Right-key frames preserve graph-input Arc and projection counters; committing the selected reference still navigates to shapes.rs and Back restores the exact departure caret. Existing same-owner declaration widget tests cover its independent expansion state. |
| `gutterMarksRefreshAndStaleWorkClearsOnModeAndFileChanges` | The same app flow marks area ordinary and resets it: counts change one to zero to one, clearing immediately at each request. Cargo.toml clears annotations; accepted root reopening clears annotations and graph inputs. Existing app mode/index/source transitions and annotation tests verify exact source/revision/policy binding. `late_progress_from_a_retired_request_cannot_overwrite_a_new_policy` holds an actual old batch, replaces the request, then releases its publication; only the replacement count is accepted. |
| `declarationsWithoutSourceFoldsScheduleNoAnnotationWork` | New `sources_without_folded_callable_bodies_never_schedule_annotation_work` first loads valid counts, then tries inline/multiline abstract traits, an impl with only a one-line method, and an enum. All retire old accepted counts and remain immediately idle with no pending publication or diagnostic. Existing tests retain exact body/container discrimination and six-plus-one batching. |
| `hoverOpensAfterDwellAndClosesByGraceOrEvents` | Service tests check exact byte range, 350 ms dwell, 700 ms grace, token movement, popup entry across another callable, immediate interruption, idle suppression, mode/input retirement, cancelled dwell, and normal-mode absence. Expanded external matrix checks explicitly critical core calls produce honest external/no-body state, while plumbing, locals, non-callables, unknown externals, stale indexes and Before do not open. Actual source frames verify stable reader coordinates, Escape, reopening, distant dismissal and the clicked whole-title definition action. The app reference/Back flow above uses the same indexed-range navigation path as hover destinations; this is composed evidence, not a new physical end-to-end app hover click. |

## Popup host assertion mapping

| Swift group | Rust evidence |
| --- | --- |
| `nativeHostTracksPaddingAndKeepsItsViewWhenResultsChange` | The existing real source-viewport hover test now captures the Loading area origin, waits for Ready and requires identical origin/serial. Pointer movement into actual popup padding retains it beyond grace. Rust's persistent egui Area ID and popup serial replace AppKit host-object identity and tracking areas. |
| `nativeEnterExitRoutesAndTransitCoversGapWithoutCapturingDistantText` | Service popup-enter/exit routing plus actual source frames retain the popup in its physical gap beyond grace, dismiss for a distant point and reopen. New `transit_stays_with_the_presented_anchor_until_a_replacement_finishes_dwell` draws an open popup while another callable dwells: the old transit region stays attached to the displayed call, replacement completes, then geometry transfers. The union/expansion matches Swift's six-point rule. |

These are egui input/render tests. No fresh native compositor or Swift/AppKit run
is claimed. Keyboard remains required; full assistive accessibility remains deferred.

## First worker group

`swift_fixture_resolves_cross_file_bodies_cache_and_ordinary_marks` maps
`resolvesSitesLocatesBodiesAndHonorsMarks`. It decodes the same checked-in wire
index and source as Swift. total_area resolves exactly one critical repository
area call, its shapes.rs destination and single site with complete coverage.
Cross-file area has no calls and proven zero. Removing its disk file after the
first load leaves the cached answer and read-byte counter unchanged. A fresh
immutable worker with area marked ordinary traverses that body and proves zero;
external unwrap honestly reports no body. The shared fixture copier in semantic
test support serves both app and worker tests. The existing synthetic worker
fixture is exposed only under cfg(test) to reuse it in popup ownership tests.

The other four worker groups (byte invalidation/dependent frontiers, configuration
retirement during load, viewer-byte/structure promotion, and unresolved/dynamic
coverage) still need their complete immutable-worker/app equivalent audit. The
three drawing groups also remain open. This checkpoint does not mark those suites
complete.

## Verification and architecture

Archive: `~/.local/state/radiology-qa/critical-summary-20261109-8441a22/`.
All 54 service tests, nine app/popup checks, the named transit ownership regression,
strict all-target Clippy, normal workspace build and warning-free API docs pass.
The full locked offline workspace run passes 854 app tests (11 existing ignored),
two binary/13 font/22 text tests and all 65 doctests. It uses one Cargo build job
and serial tests after the prior checkpoint's unconfirmed parallel-link SIGBUS;
normal half-core Cargo configuration remains 16 jobs. Formatting and diff checks pass.

Baseline `before-critical-summary-20261109` to
`verified-critical-summary-20261109`: +386 measured NLOC, six functions (one
production selector, one fixture helper, four tests), unchanged 674 measured files,
average/max CCN 3.41/132, 36 hotspots and unchanged dependency cycles. Complete
normalized clone multisets retain all 122 instances with zero additions/removals.
The raw comparator labels old clones touching modified files and an untouched
Python equal-score chokepoint ranking as regressions; no new clone or hotspot was
introduced. Most growth is assertion coverage; exact fold matching and source
geometry reuse existing mechanisms. No style/type hierarchy redesign was needed.

Other behavior suites, final whole-app acceptance, known parallel timing limits
and Android remain open. Standalone lifecycle and reusable reader/font/text crates
remain required; actual sibling-app integration remains deferred.
