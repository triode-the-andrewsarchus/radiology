# Owned review failure recovery

Reference: Swift RootView's failed-open/refresh alert, PullRequestModel's
`failedNumber`/`retryFailedReview`, and the fourth function of
[PRRefreshTests.swift](../Tests/RadiologyTests/PRRefreshTests.swift).
Reviewed from Rust `6e39cbe`. This repairs the explicit recovery gap recorded in
[PR refresh contracts](rust-pr-refresh-contracts.md).

## Behavior and ownership

A rendered-app regression first fails because Try Again is absent. Rust now
retains one failed review request with canonical repository root, requested PR
number and complete diagnostic. It lives with the existing review lifecycle,
not in the generic reader error string. Metadata errors capture the pending number
before clearing it; source-restoration errors capture it before retiring candidate
preparation. Worker disconnection also retains recoverable intent when known.

The bounded review error notice reuses ErrorNotice's summary, Details/Copy Details
and dismissal presentation. A wrapping action row adds Try Again and Choose Number.
Retry calls the existing `open_pull_request` with the failed number, including a
failed PR 8 while PR 7 remains accepted. Choose Number reuses the existing prompt,
preferring the failed number over the accepted/recent number. Ordinary prompt
commands retain their existing active/session/recent fallback order.

The accepted source/review remains usable while a request fails. A same-commit
successful retry remains metadata-only; changed source publishes through the
existing prepared-refresh transaction. Recovery introduces no second fetch or
source-preparation worker. A binary refreshed source can fail after snapshot
preparation, retain the old reader/tree and recover to a later readable head.

Retry/replacement, successful acceptance, dismissal, Exit and repository changes
retire failed intent. Recovery verifies the repository owner again before acting;
a stale click after retirement does nothing. Reader errors can coexist with a
review failure without altering its target. Each ErrorNotice owns its own detail
window state and UI ID. No string-prefix inspection is used to decide ownership.
The nonmodal bounded notice is the Rust presentation of Swift's alert actions;
Dismiss provides its Cancel behavior.

## Host API migration

`RadiologyApp::error` remains the generic reader/command error field. Review
preparation failures now use the separate `review_error() -> Option<&str>` getter.
Hosts that previously read `error` for PR preparation must read this getter too.
The field rustdoc links directly to it; generated documentation was checked with
warnings denied. The getter exposes only the current repository's failure.
Failure intent is transient, with no new persisted schema or dependency.

## Verification

Archive: `~/.local/state/radiology-qa/review-recovery-20261109-6e39cbe/`.

- `recovery-red.log`: missing painted Try Again before implementation. Its small
  regression was subsequently consolidated into the broader dismissal/scope matrix
  rather than retaining duplicate setup/assertions.
- `recovery-final.log`: two final workflow tests pass. The first uses real app
  frames at 1×/2× for failed-other-PR controls, unchanged accepted reader, prompt
  prefill, independent reader error ownership, dismissal, Exit, root change and
  stale-action rejection.
- The second relaunches exactly its own test in a child process with a private gh
  executable and private PATH/XDG roots. Parent process environment and user tool
  configuration are untouched. It exercises real `open_pull_request`, Git snapshot
  preparation, rendered Try Again clicks and subsequent publication at both scales.
  First open, same PR and another PR each assert exactly one failed view and one
  retry of the same number; successful readers have exact pinned bytes. Binary
  restoration failure followed by a readable new head likewise asserts two views
  of PR 7, retained old buffer/tree and correct fresh Unicode source/history.
- The child also checks all four controls within actual clips at widths 320/480/900
  and both scales with a long multiline Unicode diagnostic. Status-widget Tab/Enter
  chooses the failed number, clears the failure and makes no additional gh view.
  This keyboard check starts within the status widget context; existing app reader
  shortcut ownership and global Ctrl+R coalescing remain separately tested.
- `review-all.log`: the 24 review-prefixed app tests passed before consolidating
  the redundant small recovery test. Final two-test recovery and full workspace
  below are the authoritative final counts.
- `saved-review.log`: startup restoration failure retains its saved PR identity
  and reads the new owned error getter.
- `clippy-final.log`, `build.log`, `rustdoc-final.log`: locked offline strict
  workspace Clippy, normal build and warning-free API docs pass. The final field
  documentation clarification does not alter runtime code.
- Initial `workspace.log`: 829 app tests passed and one older repository-menu
  test still unwrapped the generic error field for a PR failure. Its migration to
  `review_error()` passes in `menu-final.log`. `workspace-final.log` passes the
  complete locked offline serial rerun: 830 app tests/11 existing ignored, two
  binary/13 font/22 text tests and all 64 doctests. No source edits or artifact
  replacement occurred while its self-relaunch executable was live. Formatting
  and diff checks pass.

The fake executable helper and the older configuration test's self-relaunch
pattern are shared rather than copied. CLI response and target/count assertions
are shared within the private driver. Source fixtures and private XDG/cache/tool
trees are temporary and drop after their owner/child completes. The isolated retry
driver makes no real GitHub request or remote PR write. No Android SDK/license
action is part of this change.

## Architecture and remaining scope

Baselines `before-review-recovery-20261109` → `final-2-review-recovery-20261109`:
+352 measured NLOC, eight measured functions, two files (one cohesive recovery
module and its tests). Runtime recovery functions have CCNs 1/2/4/6. Average/max
remain 3.40/132; the 36 existing high-complexity functions, one cycle and eight
structural findings remain. Complete normalized clone multisets retain all 122
instances/1,317 duplicate lines, zero additions/removals. The raw comparison flags
an unchanged old clone in touched app tests; a genuinely new repeated fixture was
removed by consolidating the regression and sharing CLI responses. Detailed
vectors are in `architecture.txt`.

Together with the preceding lifecycle checkpoint, all seven ordinary PRRefreshTests
groups now have explicit app/service mappings. This does not complete other
review/presentation/fold/repository suites or broader parity. Next source-inspected
priority: `ReviewRepositoryTests.renamedBlobDiffExcludesAChildOfTheReusedOldPath`.
Rust `ReviewSnapshot::add_change` still diffs tree/pathspec pairs; an old file path
reused as a directory can pull unrelated child hunks into a rename's parsed patch.
Verify with a real Git regression and use pinned blob identities for per-file
isolation. This recovery checkpoint does not claim that diff case is fixed.
No fresh native
compositor or Swift/AppKit run is claimed: UI evidence here is egui app/status
frames plus real process/Git requests. Shared fonts and visual roles are reused;
full accessibility and actual sibling-app adoption remain deferred. Final combined
native acceptance, the recorded concurrent semantic/selection timing limits and
SDK-backed Android acceptance remain open. Radiology remains standalone.

The next-priority reused-path defect is now fixed and mapped in
[per-file diff isolation](rust-blob-diff-isolation.md). Other repository assertions
remain open.
