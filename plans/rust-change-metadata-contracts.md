# File-change metadata and document identity contracts

Reference: the last two functions of
[UnifiedDiffTests.swift](../Tests/RadiologyTests/UnifiedDiffTests.swift) and
[FileChange.swift](../Sources/Radiology/PullRequest/FileChange.swift).
Reviewed from Rust `1deb012`. Together with
[diff contracts](rust-diff-contracts.md), all five ordinary UnifiedDiffTests functions
now have explicit mappings. This does not close the other review/refresh suites.

## Confirmed gap and shared implementation

An extended real Git regression failed before the repair: `a file.rs` had an empty
Before blob-ID field instead of its pinned Git object ID. Source reads already
used pinned commit/path identity; that did not populate the separate metadata
contract required by Swift and useful to another library host.

Pinned review discovery now requests `git diff --raw --no-abbrev -z -M` through the
existing bounded Git reader, with external diff/textconv disabled. The model's
new `FileChange::parse_raw` retains full SHA-1/SHA-256 object IDs and literal paths.
Absent sides retain Git's all-zero ID, matching the Swift raw fixture. Name/status
records still use the existing public entry point and have no IDs in their format.
Both entry points share one record assembly/path validation implementation.
Raw header validation checks modes, full same-format object IDs and complete fields.
Commit and raw blob metadata share the existing object-ID predicate extracted
from review validation. No per-file lookup or additional Git process was added.

Copied status now retains its similarity score, using the same bounded score
parsing as renames. The modified-rename title is `Renamed and modified`, matching
Swift. This label is used in existing tooltips; row/header geometry and type roles
are unchanged.

The existing checkout numstat parser and count application now live with
`FileChange`. `FileChange::apply_numstat` validates the entire input before updating
matching records; destination paths receive rename counts and binary records retain
zero numeric counts with `is_binary = true`. Checkout refresh calls that same model
API. Pinned reviews continue deriving counts from the exact per-file hunks they
already read; they do not add a redundant numstat request. This is real code reuse,
not a second parser added to emulate Swift's internal organization.

Document identity needs no new runtime helper: source locations already preserve
path, revision and exact caret equality, while `SourceIdentity` owns derived facts
by path/revision/content independently of the caret. The test checks that checkout
and After source do not share that identity and another path does not match.

## Two-function assertion mapping

| Swift function | Rust evidence |
| --- | --- |
| `rawRecordsCoverAddModifyDeleteRenameAndNumstat` | New `raw_records_keep_full_metadata_and_numstat_updates_matching_paths_atomically` uses all five Swift record categories/paths, both 40- and 64-digit IDs, exact side existence/aliases and R099. It checks all Before/After IDs, count tuples 206/0, 3/2, 0/55 and 4/1, binary state, Modified/Renamed titles and transactional failure. Additional C076 records preserve tab/newline paths and copy similarity. `malformed_raw_records_never_return_a_partial_change_list` rejects valid-prefix-plus-malformed-tail inputs, missing paths, parent traversal, invalid scores/modes/IDs, mixed hash formats and invalid UTF-8. |
| `locationsCarryRevisionsAndPositionKeys` | Extended existing `source::location::tests::test_position_key` checks checkout versus After inequality and exact keys, byte-only mutation preserving revision/key while changing full location equality, both `other()` directions and runtime source-fact ownership by document rather than caret. No duplicate public same-document accessor is introduced. |

Existing real Git `pinned_review_uses_merge_base_and_never_checkout_bytes` now checks
every changed file's Before/After IDs against `git rev-parse <commit>:<path>` or the
correct absent-side zero ID. It still checks source bytes, deleted/renamed sides,
attached HEAD/status and dirty checkout preservation. The existing SHA-256 tree
cache test checks both 64-digit IDs against a real SHA-256 repository, alongside
unchanged materialization/independent-workspace proof.

Cancellation tests now distinguish raw-metadata and per-file-patch subprocess
phases. Both phases independently prove live Cancel/timeout reaping, no fallback,
cleanup of incomplete work and successful retry without replacing accepted bytes.
Without this split, adding `--no-ext-diff` to the earlier metadata request would
have caused the old generic phase selector to stop testing per-file patches.

## Public API and integration notes

- Added `FileChange::parse_raw` and `FileChange::apply_numstat`, each with a runnable
  standalone-host example and explicit errors. Existing name/status parsing remains.
- `FileChangeStatus::Copied` now has `{ similarity: usize }`; exhaustive callers
  and serialized consumers must use the structured variant. The app does not
  persist this change-model enum in its session/preferences format. Existing
  rename and other status representations remain.
- New IDs populate fields that already existed. Source reads keep their existing
  pinned commit/path and containment policies. No dependency, worker, persistence
  schema, standalone executable layout or sibling-app adoption is added.

## Verification and architecture

Archive: `~/.local/state/radiology-qa/change-metadata-20261109-1deb012/`.

- `metadata-red.log`: empty Before ID fails against the real pinned object ID.
- `pull-request-final.log`: all 38 pull-request-filtered tests pass, including the
  SHA-256 workflow and separately gated raw/patch interruption cases.
- `models-final.log`: both metadata matrices pass after fixture readability cleanup.
- `checkout.log`: all seven Git-refresh tests pass using the shared count API.
- `identity.log`: extended location/ownership test passes.
- `clippy.log`, `build.log`, `rustdoc.log`: locked offline strict workspace Clippy,
  normal app build and warning-free API docs pass. Generated HTML shows both model
  entry points, error contracts and the structured Copied variant.
- `workspace.log`: full locked offline serial workspace passes 825 app tests/11
  existing ignored, two binary/13 font/22 text tests and all 64 doctests. No source
  edit or artifact replacement occurred while its self-relaunch executable was
  live. Formatting and diff checks pass.

Baselines `before-change-metadata-20261109` → `after-change-metadata-20261109`:
+253 measured NLOC, six measured functions, no file/module/dependency added.
Average CCN 3.41→3.40, max 132; all 36 existing high-complexity functions, one cycle
and eight structural findings remain in aggregate. `tracked_changes` CCN 7→5 via
shared application; the existing numstat parser retains CCN 13 at its new home.
Full normalized clone multisets retain all 122 instances/1,317 duplicate lines,
zero additions/removals. Raw comparison is NEUTRAL. Its numstat entity relocation
is not a newly introduced complexity problem, and equal-score top-ten Python
chokepoint swaps concern untouched files. Detailed vectors are in `architecture.txt`.

## Remaining scope

Review/refresh/presentation/fold/repository suites still require their own
assertion-level audits. This checkpoint does not claim a fresh native compositor
or Swift/AppKit run; prior rendered/native revision/source evidence remains linked
from the diff checkpoint. No layout or typography work was needed for tooltip
wording. Full accessibility and actual little-screen-clip adoption remain deferred.
Final integration, recorded parallel semantic/selection timing limits and SDK-backed
Android acceptance remain open. Radiology remains an independent app.
