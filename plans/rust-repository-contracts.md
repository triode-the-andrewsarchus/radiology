# Repository discovery and checkout source contracts — 2026-11-08

All three ordinary Swift RepositoryTests functions now have explicit Rust evidence.
The symlink assertion exposed a real gap: ordinary file/bookmark reads could load
checkout bytes from outside the accepted repository. Trait and Code Path targets
already rejected such reads. The shared checkout reader now enforces the same
containment for every caller.

## Assertion mapping

| Swift function | Rust evidence |
| --- | --- |
| folderDiscoverySkipsGeneratedDirectoriesAndBuildsTree | plain_folder_discovery_prunes_generated_directories_and_builds_directory_first_tree uses the same Sources/A.swift, .build/ignored.swift and README.md fixture. It verifies the unchanged plain root, exact two-file inventory, Sources before the README leaf and the exact child path, with no Git marker present. |
| gitDiscoveryIncludesUntrackedFilesRespectsIgnoresAndFindsRoot | Existing real_git_preserves_root_whitespace_and_worktree_literal_paths opens a nested directory, verifies exact root/inventory including untracked Unicode/tab/newline names and .gitignore, excludes ignored/deleted files, then verifies a real linked worktree. This covers the Swift root/untracked/ignore/literal-newline assertions with additional path and worktree cases. Rust determines Git operations from the root rather than exposing Swift's isGit model property. |
| sourceStoreRejectsSymlinksOutsideRoot | New checkout_reads_reject_external_links_and_keep_internal_alias_coordinates checks the public direct read and asynchronous loader against external file/directory aliases, including a sibling root with the same textual prefix. Internal aliases and an aliased root still return exact Unicode/CRLF text with the requested relative path. New app regression checks ordinary file and bookmark navigation at both scales: source Arc, caret, history and bookmark identity stay accepted while the failure goes to its existing global or bookmark-owned region. Outside fixture bytes stay unchanged. |

## Shared source policy

Previously read_contained_cancellable applied the canonical root/path check only
for selected structural targets; ordinary Byte and Bookmark targets used the
uncontained read_cancellable path. Both new regressions fail on that implementation:
the public read returns the outside source, and the app replaces the accepted
reader. These are actual runtime failures, not a documentation-only discrepancy.

The canonical containment check now belongs inside read_cancellable's Checkout
branch, after repository-relative path validation and before opening the resolved
file. It uses Path component containment, so repo-other is outside repo. Reading
the resolved path retains the original relative alias in SourceBuffer. The existing
cancellation/deadline checks surround the read. Pinned Review and accepted Buffer
reads retain their existing storage semantics.

The redundant contained-read wrapper and source-target branch are removed. Code
Path and trait callers use the same shared method, as do ordinary navigation,
bookmarks, previews, critical workers and the public SourceRead::read entry point.
No second path-policy helper, dependency, public signature, cache or persistence
mechanism is added. This matches the checked-in Swift SourceStore canonical-path behavior.

## Validation and architecture

Focused checkout tests pass. The serial workspace run passes all 808 app tests
(11 opt-in ignored) in 263.52 seconds, plus two binary, 13 font and 22 text tests.
All 61 doctests also pass. Strict Clippy, warning-denied docs,
locked offline binary build, format and diff checks are recorded under
~/.local/state/radiology-qa/repository-contracts-20261108-0e464a2/.
The first default-concurrency workspace run failed two existing time-sensitive
checks: a deep call traversal exhausted its source deadline, and a large deferred
word selection did not complete within its test frame count. Both pass in isolated
rechecks (41.75 seconds for the two traversal requests, 4.05 seconds for selection).
The traversal uses the accepted-buffer shortcut rather than the modified checkout
read; the selection fixture uses an in-memory reader. A 16-thread workspace recheck hits the same two limits. A serial full-suite
run is used to separate functional verification from those unresolved concurrent
timing limits. The failed parallel logs remain in the evidence directory.
The generated public read documentation is inspected. This checkpoint uses actual
filesystem/Git and app/rendered evidence; no fresh native desktop or SwiftUI run is
claimed.

Architecture before/after-repository-contracts-20261108: +98 measured NLOC including
three regressions, one test module and net two functions after removing the old
runtime wrapper. Average/max CCN stay 3.40/132; 36 high-complexity functions, 123 clone
instances/1323 duplicate lines, one cycle and eight structural issues are unchanged.
Normalized full clone multisets have zero additions/removals. Raw comparison flags
an unchanged touched parent-test clone and equal-score ordering among untouched
Python functions. Production policy is consolidated rather than duplicated.

## Remaining work

Continue SourceFilterTests and the remaining ordinary suites. The existing Files
presentation keeps a confirmed open document visible through filters; distinguish
that presentation exception from the underlying filtered inventory when mapping
Swift model assertions. Successful SDK-backed Android indexing and machine-specific
probe limits remain open. Full accessibility and actual sibling adoption stay
deferred.
