# Review repository contracts

Reference: all four annotated functions in
[ReviewRepositoryTests.swift](../Tests/RadiologyTests/ReviewRepositoryTests.swift),
reviewed against Rust `acde386`. This continues the
[per-file diff isolation checkpoint](rust-blob-diff-isolation.md).

## Assertion mapping

| Swift function | Rust evidence and outcome |
| --- | --- |
| `fetchesPinnedCommitsByShaAndNeverWritesTheAttachedRepository` | New `fetches_pinned_commits_missing_from_the_attached_clone_and_names_unavailable_ids` creates a real single-branch, non-local clone that lacks the PR commit. Preparation fetches the exact head privately. The attached clone still lacks that object afterward; HEAD, all refs, staged index bytes, status, staged file and untracked bytes remain unchanged after success and failed base/head fetches. Both failures now include the unavailable ID. Existing request-scoped fallback and interruption tests verify moved-head rejection and cancelled/timed-out children. |
| `diffsNameStatusCountsHunksAndMaterializesBeforeBytes` | New `three_dot_changes_keep_exact_crlf_unicode_and_edited_rename_blobs_without_later_main_files` advances main independently after the PR diverges, advertises the newer base ref, and proves the pinned comparison uses their original merge base. The canonical path set excludes main's new file. Modified/added/deleted/edited-rename/binary status and exact counts, separated Unicode/CRLF hunks, renamed old path, deleted/rename Before bytes and revision tree materialization are checked together. Existing pinned metadata assertions compare every side's object ID against Git. |
| `renamedBlobDiffExcludesAChildOfTheReusedOldPath` | Prior `per_file_diffs_exclude_children_of_reused_paths_for_renames_additions_and_deletions` covers unchanged and edited renames, reused directory children, isolated counts/hunks, exact bytes, old aliases and stable mapping. See the linked checkpoint for the red regression and object-pair fix. |
| `worktreesAreExactSnapshotsWithBlobIdsAndAreEvicted` | Strengthened `exact_reuse_rejects_changes_hidden_from_the_git_index_and_preserves_leased_old_bytes` checks pinned HEAD, added/deleted membership, every materialized file's actual Git hash versus its pinned tree object, exact repository-discovery inventory and unchanged-tree reuse. Existing hidden-index tampering is repaired into a replacement while live readers retain old bytes. The mode/type/head corruption matrix now also damages the `.git` marker and verifies replacement. Existing pressure/clone test exercises five retained readers, then releases them and checks three-tree idle LRU convergence. |

## Implementation and ownership

The diagnostic fix adds the requested full object ID to the existing failed base
and failed head-fetch messages. It does not add retry, fetch or error ownership
mechanisms. Git cancellation/timeouts continue to propagate through the shared
reader; retired requests cannot continue into fallback.

Rust's private bare store owns fetched objects rather than using Swift's attached
object-directory alternate. This is a deliberate implementation difference:
attached repository state remains read-only, and private snapshots do not depend
on the attached clone retaining an object. The new old-clone test proves this
boundary rather than asserting Swift's internal alternates file layout.

Likewise, cache ownership preserves Rust's active snapshot leases. Active readers
may temporarily exceed three trees; the existing test releases owners and checks
convergence to three idle trees and eviction of the least-recent unleased tree.
Replacing a corrupted leased tree preserves the old owner rather than rewriting
its directory. This is stronger reader isolation than counting directories alone.
Before source is read by exact object ID and a separately verified base tree is
materialized for desktop handoff; Swift's individual blob-cache layout is not a
host API requirement. Public source/tree behavior is checked directly.

No runtime abstraction, worker, dependency or UI state was added. New checks
reuse the real Git fixture with tree-edit callbacks, private preparation, source
reader, tree revealer, existing cache matrix and repository discovery.

## Verification

Archive: `~/.local/state/radiology-qa/review-repository-20261109-acde386/`.

- `fetch-red.log`: the older clone is fetched successfully, then the missing-base
  assertion fails on `Unable to fetch base commit` without the requested ID.
- `pull-request.log`: 41 focused tests pass, including both new contract tests.
- `tree-cache.log`: six tests pass after the final blob/discovery/metadata checks.
- `clippy.log`, `build.log`: strict all-target workspace Clippy and build pass.
- `workspace.log`: full locked offline serial run passes 833 app tests (11
  existing ignored), two binary, 13 font, 22 text tests and all 64 doctests.

Architecture baseline `before-review-repository-20261109` to
`final-review-repository-20261109`: +229 measured NLOC/two test functions/no new
files; average/max CCN remain 3.41/132, with 36 high-complexity functions. All 122
clone instances and 1,317 duplicated lines remain unchanged, including the full
normalized file-pair/fragment multiset with multiplicity. One dependency cycle and
eight structural issues remain unchanged. Raw equal-score chokepoint ranking
movement is in untouched Python sources. The small production diagnostic change
and the contract coverage do not introduce a second Git or snapshot mechanism.

All four repository-suite groups now have explicit mappings. Remaining work
includes the separate PullRequestReview, ReviewChangesPresentation and
ReviewFoldDefaults assertion audits, final native integration acceptance, known
parallel timing limits and Android. The exact reused-path service test could also
be complemented by an app workflow; existing full-suite app navigation exercises
the shared diff path but does not reproduce that specific directory shape.
