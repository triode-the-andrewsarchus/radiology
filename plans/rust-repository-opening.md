# Background repository opening

Implemented 2026-10-30. The standalone app prepares repository roots, inventories,
classification, sorting, filters and initial file trees on the existing bounded
blocking pool. Checkout Git refresh remains an independent background snapshot.

## Transaction and host contract

`open_repository` now requests work; it does not synchronously attach a root.
An embedded host drives `poll_repository`, then the existing `poll_source`,
`poll_index` and `poll_git_refresh` methods. The native frame loop does this.
`is_repository_opening` reports pending publication, and `cancel_repository_open`
retires the receiver and signals cancellation without changing accepted state.
The legacy synchronous RepositoryDiscovery APIs remain available for worker use.

LatestRead owns replacement/cancel/drop semantics. A pending root retains the
accepted repository, source, history, pins, review and session, so navigation and
bookmark edits remain usable. Success publishes the prepared root, inventory and
tree together, then starts its source/index/Git work. Existing repository reset
rules still apply at that point. Failed/cancelled requests leave the last accepted
reader/session intact. A busy row offers Cancel; failures offer Retry/Dismiss.
The pending path is available on the progress label's hover text.

A typed RepositoryDestination binds default selection, explicit launch source,
PR/file intent or checkout refresh restoration to its root request. CLI/session
restoration starts a saved review only after discovery succeeds. A newer root
retires an unaccepted review request; an explicit review action retires a pending
root. Late completions cannot cross those transitions. Accepted reviews remain
usable during discovery. Cancellation of the retired review subprocess itself is
still part of the broader PR lifecycle queue.

Checkout refresh resolves the destination when discovery publishes, preserving
caret movement or pending exact source navigation made during the wait. It does
not take a request-time caret snapshot. PR-refresh fold/pin mapping and broader
refresh-state restoration remain open parity requirements.

A prepared tree carries its SourceFilters. If filters change during discovery,
the app requests a new preparation before accepting the root; it does not build
a stale tree or rebuild that initial tree on the UI thread. Subsequent explicit
filter changes within an accepted repository retain the existing tree path.

## Discovery and shared mechanisms

Repository discovery and checkout refresh share GitRead, which uses the existing
process-group runner. Commands use literal arguments and a child-only environment,
complete stdout capped at 16 MiB per command, cancellation and a 30-second worker
deadline. Discovery checks retirement/deadline between ancestors/files and after
classification/tree preparation. Sorting/tree building and individual filesystem
syscalls are not interruptible mid-operation; retired results cannot publish.

Git's root output loses exactly its final line terminator, preserving whitespace
that belongs to the actual root. Git worktree `.git` files are supported. NUL
path records preserve tabs/newlines/Unicode and reject incomplete/invalid/escaping
paths. Missing/deleted files and directory entries do not become source files.
The file inventory remains independent of presentation filters.

An absent Git executable or nonzero discovery command uses the filesystem fallback,
including parent `.git` marker lookup and existing build/vendor exclusions.
Traversal does not follow links. A structured pre-launch OS error distinguishes
an absent tool from timeout/cancellation/permission/output failures; those failures
never silently become a successful fallback. Traversal errors reject the snapshot
rather than publishing a partial inventory. There is no new executor or sibling
runtime dependency.

## Verification and native evidence

Workspace tests pass: 335 application tests, one external-Clang fixture ignored,
eight reader-text tests and 30 doctests. Formatting, strict Clippy, build and
rustdoc pass. Tests exercise real Git roots with trailing whitespace/newlines,
worktrees, literal filenames, missing/deleted/ignored paths, no-Git traversal,
exclusions, symlinks, invalid folders, cancellation, deadlines, oversized/partial/
escaping output, failure/cancel/retry retention, request replacement, launch intent,
retired review intent, current/pending navigation and changed-filter publication.
Index tests now explicitly await root publication before asserting import state.

Architecture comparison against `before-repository-open` adds no hotspot, cycle
or clone group. Average CCN decreases; four existing fragments in the touched app
test file remain flagged. Shared Git/test support replaces duplicate mechanisms.
The configured Cargo build limit remains 16 jobs, half the host's 32 logical CPUs.

Native Weston/Niri acceptance launches explicitly into b.rs, delays root discovery
with a private child-only Git wrapper, and verifies the old source/inventory remain
visible while real arrow input moves to line 4. Cancel retains that state. An
incomplete Git inventory exposes Retry/Dismiss and retains the three-file tree.
Retry is delayed again; caret movement to line 1 during that wait survives success,
which displays the new four-file tree and still selects b.rs. App PID 230409,
Weston and Niri exit zero; pointer helpers are intentionally terminated.
The sibling capture helper is used only for QA.

Local artifacts: `/tmp/radiology-repository-open-native/manifest.json`, screenshots
`settled.png`, `opening.png`, `interactive-opening.png`, `cancelled.png`, `failed.png`,
`retry-live-caret.png`, `retried.png`, and `/tmp/radiology-repository-open-driver.py`.
Tested binary SHA-256:
`243f1e5b146d770d393b45e7855aba665bc698d926970041e3709db8dc6c8be4`.
The manifest records exact hashes for all seven screenshots and the driver.
