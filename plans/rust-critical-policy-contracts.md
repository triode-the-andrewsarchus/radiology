# Critical Lens policy and override-store contracts

Reference: all three `CriticalityPolicyTests` functions and both
`CriticalityOverrideStoreTests` functions, inspected against Rust `b7b4cb8`.
This checkpoint strengthens existing tests; production behavior needed no change.

## Classification and identity

| Swift group | Rust evidence |
| --- | --- |
| `headerParsingHandlesPlaceholderAndURLVersions` | Expanded `test_scip_symbol_header_parsing` verifies core's placeholder version, std's URL version and descriptor suffix, libc's version-independent package key, local rejection, unowned package placeholder and doubled-space escaping inside the manager field. The parser is shared with the semantic index. |
| `builtInPrecedence` | Expanded `test_policy_precedence` constructs a semantic index containing a repository definition, derives exactly the app package with `for_index`, and checks the full classification/reason pairs for repository/inactive items, core plumbing, std system APIs, io/error and buffered exclusions, os FFI descriptors, known libc/SQLite bindings, unknown -sys and serde packages, pointer helpers, tokio fs versus task, other schemes and locals. An extern signature outranks repository definition. |
| `overridesWinAndSymbolBeatsPackage` | `symbol_package_overrides_and_resets_preserve_builtin_precedence` checks package versus symbol overrides, repository demotion, refused local persistence, override-free repository classification, symbol reset revealing the package decision, and Unknown package reset restoring std system/FFI defaults. |

The expanded semantic menu test also constructs indexed symbol information, first
checks RepositoryDefinition despite a manual package decision, then changes its
indexed signature to extern C and checks FfiSignature. Thus signature extraction
and override-free Built-In presentation are exercised through `MarkContext`, not
only by supplying a signature argument directly to the policy. Existing menu
guards still reject locals, mismatched source digests and Before revisions.

The existing three app-level Critical Lens tests pass as well. In particular,
`critical_marks_refresh_without_reader_changes_and_restore_after_restart`
checks actual projection changes from package/symbol choices and both resets,
identical accepted source Arc, caret/history/folds/pins, stale target/Before
rejection, durable flush and restored policy after app reconstruction. This is
render-independent app command evidence; earlier native menu acceptance remains
recorded in [Critical marks](rust-critical-marks.md).

## Persistence, invalid data and deletion failure

| Swift group | Rust evidence |
| --- | --- |
| `roundTripIsolationAndTolerance` | Existing override tests cover no idle writes, atomic symbol/package roundtrip, canonical root, another root and alias isolation, storage outside the viewed tree, corrupt/foreign rejection, removing the last choice, and older saves arriving after newer saves/deletion. Added assertions explicitly check canonical persisted root and private file location. The new `corrupt_or_foreign_saved_marks_leave_an_empty_session_without_overwriting_disk` opens both damaged JSON and a mismatched-root record through MarkSession: each yields empty effective overrides and a diagnostic, preserves original bytes and leaves the repository untouched. |
| `deletionFailuresAreReported` | The existing failed-newer-save test now also saves valid choices, makes the private parent directory read-only, attempts deletion, restores permissions before assertions, and requires an error with original bytes intact. This ran under UID 1000 on Linux. Root skips this permission-dependent branch; the existing nonempty-directory deletion failure remains unconditional. |

Swift's store returns empty data for corrupt/foreign records. Rust's low-level
store returns an error and MarkSession converts that to empty effective decisions
while exposing a diagnostic. The host outcome is therefore empty policy plus
visible failure information; invalid records are never accepted as marks or
silently overwritten merely by loading. The read-only-parent test verifies the
store error boundary; the existing MarkSession failure test separately proves
session decisions remain available and failure state does not leak across roots.
No new claim is made about automatic retries after a failed write.

## Verification and architecture

Archive: `~/.local/state/radiology-qa/critical-policy-20261109-b7b4cb8/`.
`policy.log` passes the three policy tests; `critical.log` passes all 48 Critical
Lens service tests, including the final additions; `app.log` passes the three
reader integration tests. Strict workspace all-target Clippy, formatting and diff
checks pass. No fresh whole-workspace or compositor run is claimed for this
test-only checkpoint; the previous 844 app/65 doctest run remains historical.

Baseline `before-critical-policy-contracts-20261109` to
`verified-critical-policy-20261109`: +153 measured NLOC, one test function,
no files, unchanged average/max CCN 3.41/132, 36 hotspots and existing cycles.
Complete normalized clone multisets retain all 122 instances, zero additions or
removals. The raw diff-aware report flags three old fixture clones in touched
test files and may reorder equal-score untouched Python chokepoints; neither is
a new structural dependency or clone. Existing policy, session, index, file-path
and fixture mechanisms are reused; no production API or dependency was added.

These five groups are mapped. Critical Lens model/projection, frontier, worker,
summary/hover combined assertions and broader parity still need their remaining
assertion-level reconciliation. Final native acceptance, known parallel timing
limits and Android remain open. Full accessibility and sibling-app adoption stay
deferred; keyboard support and a standalone app remain required.
