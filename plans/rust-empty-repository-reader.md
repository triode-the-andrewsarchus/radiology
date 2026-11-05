# Repository discovery waits for a file choice — 2026-11-05

Swift AppModelTests.aNewRepositoryClearsOldStateAndWinsAnOpeningRace expects the
accepted root/inventory, no selected source and empty history. RootView also opens
the parent of a dropped file as a repository, without selecting that file. Rust
instead automatically opened README, the first source, or the first inventory
entry. The new regression fails at its empty-reader assertion against that code.

Plain repository open, recent-root/session discovery and dropped folders/files
now publish the root and inventory without inventing a source visit. The existing
No File Selected view, file tree, Ctrl+K guidance and source-dependent command
availability present that state. An explicit file choice opens normally. Opening
the same root again is a fresh plain open and clears the previous reader/history.

Explicit launch file/PR destinations retain their existing handlers. Refresh uses
RestoreCurrent, resolves the current or pending destination after discovery and
restores it through the Source handler. A refresh without a source stays empty.
Removing the automatic fallback is the only runtime behavior change; no new flag,
worker, configuration, public API or dependency is needed. The public
open_repository documentation now describes the empty reader and explicit launch
option.

## Regression and native evidence

The new plain_repository_discovery test explicitly confirms a prior discovery
worker is running, holds it behind a channel, accepts a newer real root containing
both README and last.swift, and only then releases the old worker. The accepted
root/inventory remain current, source/selection/history are empty, source loading
is idle and no repository error appears. Both scales render No File Selected.
Choosing last.swift opens its exact bytes; another plain open of the same root
clears that visit. This maps the Swift opening-race contract while making the
retired operation genuinely late rather than relying on scheduler timing.

Existing drop coverage now checks no source/history visit after opening a dropped
file's parent. The bookmark empty-state matrix checks that even a populated root
has no source and cannot create a bookmark until the user chooses a file. Existing
explicit launch, saved review, failure/cancel/retry and live refresh restoration
tests remain active.

Reader fixtures now choose their source explicitly. Shared open_reader performs
repository publication followed by the requested file publication and is reused
at 20 setup sites, including isolated_source, reader_with_source and
opened_repository. A Critical Lens test reuses the existing two-file fixture and
reloads its authored source, retaining the sibling file needed for cache-return
checks. Index-import cancellation's intermediate publication checks remain explicit.
These test changes preserve their original reader/semantic assertions instead of
restoring automatic selection inside a general settle helper.

Four native runs pass: plain and explicit-file launches at 1×/2×. Plain launches
show the empty reader and open scan.rs through Ctrl+K. Explicit --file launches
open scan.rs immediately. Both workflows navigate to line 61 and use Ctrl+R to
refresh the root; exact Copy checks confirm the original 1353-byte Unicode source,
the restored call line and the full source again. Three clipboard checks per run
pass. Captures show the empty/selected/refreshed states. Prepared fixture source,
Git HEAD/refs/status and the executable hash stay unchanged throughout each run;
all accepted child exits are zero.

The first native attempt correctly copied a 17-byte checkout marker from a reused
PR fixture, while its verifier expected PR-origin bytes. Accepted runs prepare the
long checkout source before recording baseline hashes and launching the app. They
exercise checkout discovery/refresh rather than confusing a pinned PR with its
working tree. No app change was made in response to that fixture mismatch.

Workspace 795 app tests/11 ignored, two binary, 13 font,22 text and 60 doctests pass.
Strict workspace/all-target Clippy, rustdoc warnings, formatting and binary build
pass. Evidence, red regression, captures/manifests, runner and architecture counters:
~/.local/state/radiology-qa/empty-repository-20261105-a930cb2/.
Working native fixtures remain under ~/.cache/radiology-qa-empty-repository/.

## Architecture and remaining scope

Baseline before-empty-repository-reader at a930cb2 versus after-empty-reader-accepted:
+40 measured NLOC and two test functions (regression and shared fixture), same 666
files, average/max CCN 3.40/132, 36 high-complexity functions, one cycle and eight
structural issues. accept_repository now has CCN 5. Clone instances fall 125→123 and
duplicate lines 1335→1323. Full fragment/file multisets have zero additions and two
removals. The raw comparison still flags unchanged clones in modified test modules
and an equal-score ranking swap between untouched Python chokepoints; these are
not new duplication/dependency problems. Consolidating actual fixture setup removes
the accidental clones initially introduced by repeated explicit-open sequences.

This closes the initial-selection difference recorded in the structural-history
checkpoint and the ordinary opening-race assertion. The other AppModel assertions
and remaining Swift suites still need reconciliation; SDK-backed Android success
remains pending the earlier license/install decision. Full assistive accessibility
and actual little-screen-clip adoption remain deferred.
