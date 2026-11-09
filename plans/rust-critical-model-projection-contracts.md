# Critical Lens model and projection contracts

Reference: all four `CriticalLensModelTests` functions and all three
`CriticalProjectionTests` functions, inspected against Rust `8e23c03`.

## Unsupported-source transition

The extended app test `critical_mode_uses_background_calls_and_rebuilds_when_an_index_arrives`
first failed after opening Cargo.toml in Critical mode: Rust exposed an empty
projection instead of clearing it. `refresh_critical_projection` now starts work
only for an accepted buffer with supported call syntax in Critical mode; other
states use the existing stop operation. The app getter documents that unsupported
files have no projection. Status remains Call syntax unavailable; returning to
supported source uses existing cached facts. The lower-level pure projection API
continues to return an empty range set for unsupported or mismatched syntax.
No second cache, worker, navigation mechanism or public type was introduced.

## Model assertion mapping

| Swift group | Rust evidence |
| --- | --- |
| `modeTogglesProjectionWithoutTouchingNavigation` | New both-scale `noncall_marks_mode_toggles_and_caret_frames_preserve_the_reader_at_both_scales` installs a decoded UTF-16 constant occurrence, marks/resets its symbol and marks its package. Projection rebuilds while the constant remains dim. Source Arc, caret, complete history, folds and pins stay unchanged during each policy operation. The menu reports core ownership and ordinary Built-In classification. Five actual Right-key frames change the caret without projection builds/hits or range changes; Normal clears and Critical reuses cached facts. Existing app test adds unsupported-file clearing, supported return, real wire-index arrival, mode cancellation and repository/cache reset. |
| `editsBeforeLoadAreKeptAndSavesStayOrdered` | Extended MarkSession test edits a package and a new symbol before polling the persisted load, requires the package change immediately, switches roots before publication, then reopens and retains the untouched saved symbol plus newer package/new-symbol edits. A reset before load publication is replayed correctly. Rapid-save test combines 100 alternating choices, 25 distinct symbol choices and package reset; flushed disk equals the complete final map of 26 symbols, with no package choice left. Existing concurrent/revision-order store tests remain applicable. |
| `unindexedFileProjectsSyntacticCallsWithoutCreatingTargets` | The new both-scale flow opens extra.rs, which is absent from the installed index. Its actual syntactic call stays prominent, projection matches exact source and semantic occurrence lookup returns none. Existing app test separately covers no index at all, then a real wire index arriving later. |
| `projectionUsesCallSyntaxAndExactBytes` | The app flow checks source/projection identity and unchanged bytes through marking, mode toggles and repository reopening. Pure projection tests below check exact byte ownership, normalized bounded ranges and content/path/revision/syntax invalidation. |

The new app test uses existing decoded-index installation, UTF-16 occurrence,
checkout/frame/key, mark and persistence mechanisms. It does not claim a live
compiler generated that synthetic constant index; the separate fixture test below
uses the checked-in Rust-analyzer wire index. Mode and mark operations preserve
full history; ordinary caret reports update the current history coordinates
without adding Back/Forward steps, consistent with Rust's reader history contract.
The projection/cache counters stay unchanged during those reports.

Repository reopening reloads persisted package choices and clears projection at
accepted-root publication. Rust intentionally keeps its previously accepted reader
while repository discovery is pending; this audit does not change that existing
transactional root-opening behavior. Source bytes remain untouched.

## Pure projection assertion mapping

| Swift group | Rust evidence |
| --- | --- |
| `fixtureCallsStandOutWhileDeclarationsAndExpressionsRecede` | New `swift_fixture_recedes_declarations_types_receivers_and_expressions` decodes the same checked-in scip-rust/index.scip and src/lib.rs used by Swift. Repository method area stays prominent while total_area, Circle, for, += and the receiver recede. Identity, normalization and rejection of a changed source byte are asserted. |
| `onlySyntacticNamesEscapeDimmingWithIdentityBasedOrdinaryCallsExcluded` | New Unicode-rich projection test uses the Swift-shaped comment/string/generic-call/field/pattern/macro source, with exact UTF-16 semantic occurrences. Repository-marked call names, unresolved generic calls and macros remain prominent; ordinary clone and every named non-call token recede, including critical-marked fields and local bindings. Every prominent byte belongs to a syntactic call-name range, and all range boundaries are valid UTF-8 boundaries and normalized. Rust paints byte ranges directly, rather than converting them to TextKit character ranges. |
| `syntaxWithoutSemanticDataStillFocusesCallsAndMissingSyntaxDoesNothing` | Extended source-identity test asserts the exact complement around an unindexed call name. Existing tests verify unsupported syntax produces no receded bytes, same-path stale syntax is rejected, same bytes at another path/revision do not match, and no-call supported source recedes completely. Unresolved names never acquire identity-based demotion. |

Existing projection memo/cache tests also verify byte/key limits, LRU eviction,
index/policy invalidation without retaining old indexes, and cancellation while
waiting for the cache lock. Renderer/drawing, summary/hover and graph-worker suites
retain their own remaining assertion-level audit; this checkpoint does not mark
those suites reconciled.

## Verification and architecture

Archive: `~/.local/state/radiology-qa/critical-model-projection-20261109-8e23c03/`.
`red.log` records the unsupported-file mismatch. `critical-final.log` passes all
52 service tests, including final persistence changes; `model.log` passes the
both-scale constant/caret workflow; `critical-app.log` covers the fixed unsupported
flow and existing app/viewport checks. Strict all-target workspace Clippy, normal
build and warning-free API docs pass. The initial whole-workspace attempt failed
inside the linker with SIGBUS before running tests (`workspace-link-failure.log`).
The affected binary linked successfully on isolated retry (`link-retry.log`);
the workspace retry uses one Cargo build job plus serial tests. The cause of the
parallel link crash is unconfirmed; no toolchain/concurrency fix is claimed.
Normal `.cargo/config.toml` remains at the requested 16 jobs. The full locked
offline retry passes 850 app tests (11 existing ignored), two binary/13 font/22
text tests and all 65 doctests.

Baseline `before-critical-model-projection-20261109` to
`final-critical-model-projection-20261109`: +285 measured NLOC, three test functions
and one focused app-test file; no new production function. Average/max CCN remains
3.41/132, with 36 existing hotspots and unchanged cycles. Full normalized clone
multisets retain all 122 instances, zero additions/removals. Raw warnings identify
old touched test fixtures; equal-score untouched Python chokepoints may reorder.
Existing service, cache, source ownership and fixture mechanisms are reused.

These seven groups are mapped. Remaining summary/hover/worker/drawing audits,
other unreconciled suites, final native acceptance, known parallel timing limits
and Android remain open. Full accessibility and sibling-app adoption remain
deferred; keyboard support and a standalone reusable app remain required.
