# Critical frontier and conditional-count contracts

Reference: all three ordinary `CriticalFrontierTests` functions, inspected against
Rust `5059aa2`. Existing reachability traversal already handled cycles, bounds,
unknown leaves and no-body results. This audit exposed two conditional-summary gaps.

## Confirmed gap and correction

A new test supplies a parsed closure scope containing two sites with Complete
summary coverage, matching Swift's synthetic frontier test. Rust previously
returned CountAnnotation { count: 0, completeness: Proven }. Source-produced Rust
summaries normally already include conditional gaps, so that path was partial;
the independent frontier API did not enforce the same rule for supplied summaries.

The existing traversal merge now consumes the summary and takes the larger of
its prepared conditional coverage and immediate nested-scope site total before
merging. This makes conditional-only results partial without double counting
normal summaries. It also preserves larger counts for deeper nested scopes that
are absent from the immediate conditional list. No conditional callable becomes
an execution edge. Unknown identities still deduplicate separately across graphs.

The compact status now includes the root summary's conditional site count, as
Swift does. This uses the existing status method consumed by the hover card;
there is no new widget or formatter. A helper body with one direct system call
and two closure calls now paints `1 direct · 2 conditional · partial`. The status
shows immediate root-owned conditional sites; traversal coverage can include
additional/deeper sites and remains available in the detail explanation.

## Assertion mapping

| Swift group | Rust evidence |
| --- | --- |
| `viaPathsDedupCyclesAndDiamonds` | Expanded shortest-path test combines root recursion, self-recursion, diamonds, repeated direct/indirect critical identities, ordinary external plumbing and repeated unknown leaves. Exactly three bodies load, paths retain shortest stable via chains, three distinct critical operations form a lower bound, and status reads `1 direct · 3 reachable · partial`. Removing the root's unknown still finds the nested unknown. |
| `unknownOrConditionalOnlyFunctionsAreNeverProvenZero` | Unknown leaves are never expanded or counted and produce `0+` plus partial explanation. New conditional metadata test checks Complete, already-counted two-site and deeper three-site coverage at both root and through an ordinary body. All remain partial, immediate root status names two conditional sites, and no direct/indirect critical edges appear. |
| `capsMissingBodiesMacrosAndNoBodyAreExplicit` | Existing tests check depth-one/zero-body boundaries, abstract missing-body gaps, conditional partial results, empty proven zero and explicit external no-body annotation. New real-chain matrix checks depth 2, body limit 1 and full depth 6/body 48: capped paths stay empty/truncated; full expansion reaches z via a/b/c exactly once, retains macro/unresolved gaps and reports `1+`. A macro with a repository definition is never expanded. The Rust body-provider API represents absent/abstract/unreadable bodies with explicit no-body summaries rather than Swift's optional result. |

The graph test now takes a real parsed/semantically resolved nested source through
frontier building. Its coverage already includes three conditional sites across
deeper scopes while immediate metadata totals two; the frontier must preserve
that exact coverage and display two immediate conditional sites. This checks the
production preparation path as well as independently supplied summary data.

The existing source viewport hover test now uses a helper containing a closure.
It paints the conditional status and still checks dwell, Escape suppression,
reopening, actual definition-row hit testing, immutable source, unchanged caret
and no selection. This is an egui frame test, not new compositor acceptance.

## Verification and architecture

Archive: `~/.local/state/radiology-qa/critical-frontier-20261109-5059aa2/`.
`red.log` captures false Proven versus expected Partial. `critical-final.log`
passes 50 service tests; `popup.log` passes the final rendered hover workflow.
Strict workspace all-target Clippy, normal build and warning-free API docs pass.
The full locked offline serial run passes 847 app tests (11 existing ignored),
two binary/13 font/22 text tests and all 65 doctests.

Baseline `before-critical-frontier-20261109` to `final-critical-frontier-20261109`:
+193 measured NLOC, two test functions, no files or production functions added.
Average/max CCN remains 3.41/132 with 36 existing hotspots and unchanged cycles.
Complete normalized clone multisets retain all 122 instances with zero additions
or removals. Raw touched-file fixture/viewport clone warnings predate this change;
equal-score untouched Python chokepoints may reorder. The existing traversal,
summary metadata, coverage accumulator, graph fixtures and hover renderer are
reused; no new dependency or public type was introduced.

All three frontier groups are mapped. Critical Lens model/projection, graph-worker,
summary/hover combined assertions and broader parity still require their remaining
reconciliation. Final native acceptance, known parallel timing limits and Android
remain open. Full accessibility and sibling-app adoption remain deferred.
