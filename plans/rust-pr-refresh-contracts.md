# PR refresh lifecycle contracts and recovery gap

Reference: seven functions in
[PRRefreshTests.swift](../Tests/RadiologyTests/PRRefreshTests.swift),
[AppModel+PRRefresh.swift](../Sources/Radiology/AppModel+PRRefresh.swift) and
[RootView.swift](../Sources/Radiology/UI/RootView.swift).
Reviewed from Rust `9f8a624`. Six groups were mapped at this checkpoint and the
failure/retry group was partial. The subsequent
[owned recovery checkpoint](rust-review-recovery.md) implements and verifies that
gap. Combined native/whole-app acceptance and other suites remain open.

## Current fix and UI organization

A red app regression reproduces metadata refresh failure saying `Unable to open
review: offline`. The existing accepted review remains open, so this misnames the
operation. One derived `is_review_refreshing` predicate now supplies reader
readiness, progress wording and error context. It uses the existing pending/active
PR numbers, with no new request flag or worker. Error context is captured before
pending intent clears. A failure while opening a different PR still says open.

The progress banner now says `Refreshing pull request #…` for the active review
and retains `Preparing` for a different review. Its existing panel ID, Cancel
control and placement remain. The progress banner now lives with the existing
review notice in `show_review_status`, removing review-specific layout from the
large app renderer. Existing widgets/type roles are reused; no typography change.

## Seven-group mapping

| Swift function | Current evidence and status |
| --- | --- |
| `sameHeadAndBaseIsMetadataOnlyAndCommandRouteCoalesces` | New `metadata_refresh_coalesces_commands_and_preserves_reader_fact_owners` holds a live metadata completion, sends two actual Ctrl+R events at 1×/2×, checks visible refresh wording and disabled duplicate submission while the reader stays eligible, then accepts Fresh title into review/recent list. Source, outline, native captures, calls and an explicitly seeded semantic-index Arc retain identity, as do caret/history/pins. Existing recent-list publication helper supplies the accepted list; no new runtime injection API. First CLI invocation/count is a composed service/route boundary, not a new end-to-end gh count assertion. Outside-review refresh is separately covered by existing same-root repository/position contracts. |
| `changedHeadRestoresLatestCaretAndViewportAndPreservesOptions` | Extended existing positions test holds metadata completion while changing caret to 110:1 and painted top row to 20. A real inserted-prefix snapshot accepts 111:1/top 21, retaining font 17, wrap, both source filters, a real src disclosure and Files tab, while clearing obsolete history/other-file position memory. Existing latest-rename/pin regression now explicitly retains Critical mode; other tests verify changed bytes drop pins and unchanged lines keep columns. These are complementary workflows, not one synthetic assertion of every option combination. |
| `sameHeadNewBaseRefreshesBeforeBytesAndDiffs` | Reworked existing moving-base test now proves head equality while advancing the merge base to an actual ancestor commit. It checks newer Before bytes, mapped 3:5 caret, one remaining change, Before preference and cleared history. Existing fallback/missing-file and commit-scoped semantic tests cover unavailable-side and index ownership policies. |
| `failuresRetryIntentAndProtectedWorktreeSurvivesPreparationFailure` | **Partial at this checkpoint; now repaired in [owned recovery](rust-review-recovery.md).** New error-context regression checks metadata failure for the active versus another PR at both scales, retained accepted source/history/review and re-enabled Refresh. Existing unreadable refreshed source test retains the old reader; service cancellation and cache-pressure tests protect live trees and allow retry. Swift's explicit Try Again/Choose Number actions and retained failed-number intent were outstanding here; the follow-up supplies the direct app recovery test. Do not count this group complete. |
| `pendingRevisionLoadAtPreparationRestoresLatestRequestedReader` | Extended existing moving-reader test retains navigation to an added file, then separately queues another candidate and starts an actual Before command while its source load is pending. Accepted refresh uses the latest Before request, exact base bytes/new head and a fresh history instead of an earlier prepared After reader. |
| `renamePreservesExactBytePinsAndMissingFileFallsBackHonestly` | Existing latest-rename test checks exact unchanged bytes, latest caret/pin groups, clean ephemeral state and fresh fold defaults. Existing missing/fallback test checks Before-only fallback notice and, for a former addition absent from both sides, empty source/Changes/absent notice. Positions and cache-pressure tests add exact reading restoration and retained leased old bytes. |
| `cancelDifferentPRAndRepositorySwitchDiscardRefreshCompletion` | New `retired_refresh_results_cannot_reopen_cancelled_exited_or_replaced_review_scopes` gates a confirmed live metadata worker, deliberately ignores its cancellation when attempting publication, and proves cancelled flag plus closed receiver at 1×/2× for Cancel, Exit, another PR and a real root switch. After settling frames, the intended scope survives with no stale error or pending PR. The other-PR branch explicitly stages replacement acceptance after retirement; it does not launch a real gh request. |

## Shared fixtures and evidence boundaries

Three new app tests share one scaled review fixture. Metadata coalescing and
viewport restoration share a confirmed-live completion gate. The existing recent
list publication helper is exposed only under cfg(test)/Unix for reuse, instead
of adding a production setter or another polling implementation.

The initial metadata fixture incorrectly assumed an automatically populated recent
list despite no controlled gh listing. It failed indexing an empty list. The final
fixture publishes an accepted list through the existing helper; this was a test
setup correction, not a product bug. A filter-field typo and single-range Clippy
lint were also corrected before final gates. The red operation diagnostic remains
the product regression. No test binary is rebuilt while the final suite is live.

## Recovery feature identified here (subsequently implemented)

Swift RootView presents Try Again, Choose Number and Cancel after a failed open or
refresh. At this checkpoint Rust offered generic diagnostics/Dismiss and the ordinary
toolbar route. Refresh could retry the accepted PR, but a failed attempt to open a
different number had cleared `pending_pr`, so `active_pr` could not recover the
failed target. Source/error presentation tests alone did not establish parity.

The follow-up needed to retain failed-request number/root/operation ownership, expose
the two recovery actions with bounded diagnostics, and retire stale recovery intent
on retry/replacement, dismissal, Exit or repository switch. Initial metadata failure
and later source-restoration failure both need coverage, including a different
failed PR while another remains accepted. A fresh successful retry must preserve
or atomically replace the accepted reader as appropriate. Reuse the existing
request entry point and error UI; avoid a second refresh state machine. These
requirements are now addressed in [owned review recovery](rust-review-recovery.md).

Other review/presentation/fold/repository suites remain separate audit work.
Full accessibility and actual sibling-app adoption remain deferred. Final combined
native acceptance, recorded parallel timing limits and SDK-backed Android acceptance
remain open. Radiology remains standalone.

## Verification and architecture

Archive: `~/.local/state/radiology-qa/pr-refresh-contracts-20261109-9f8a624/`.

- `error-red.log`: active refresh diagnostic names open instead of refresh.
- `refresh-final.log`: all 12 refresh-filtered app tests pass, including three new
  both-scale cases, unchanged-head/new-base and pending Before restoration.
- `positions-final.log`: latest caret/viewport/preferences workflow passes.
- `clippy-final.log`, `build-final.log`: locked offline strict workspace Clippy and
  normal build pass. This changes private UI code, not a public API or rustdoc.
- `workspace.log`: full locked offline serial workspace passes 828 app tests/11
  existing ignored, two binary/13 font/22 text tests and all 64 doctests. No source
  edits or artifact replacement occurred while the self-relaunch executable was
  live. Formatting and diff checks pass.

Baselines `before-pr-refresh-contracts-20261109` →
`final-pr-refresh-contracts-20261109`: +261 measured NLOC, six functions (one runtime
predicate, two test fixtures and three regressions), unchanged file count. Renderer
CCN 38→36; the existing review status renderer is CCN 6. Average/max remain 3.40/132,
with 36 existing high-complexity functions, one cycle and eight structural findings.
Full normalized clone multisets preserve all 122 instances/1,317 duplicate lines,
zero additions/removals. Raw result NEUTRAL; equal-score Python ranking churn
concerns untouched files. Initial repeated test setup was consolidated into the
shared fixture; no new clone remains. Detailed vectors are in `architecture.txt`.

The new status wording is checked in rendered app frames at 1×/2×. No fresh native
compositor or Swift/AppKit run is claimed. Earlier native reader/rename/viewport/
failure/recovery observations are historical evidence in
[rust-pr-refresh-restoration](rust-pr-refresh-restoration.md); they do not close
the recovery-control gap by themselves; the follow-up supplies direct evidence.
