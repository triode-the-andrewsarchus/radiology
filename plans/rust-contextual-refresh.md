# Contextual repository refresh command — 2026-11-02

## Swift reference and product contract

Swift ReaderCommands exposes Reload Repository outside review and Refresh PR during
review on Command-R. AppModel.refresh relists the checkout and restores the current
document, or delegates to PR refresh. ReaderCommands and canRefreshPR disable refresh
while opening/review preparation is in progress. Rust already had the asynchronous
RestoreCurrent/review-refresh transactions, but only a shortcut for checkout reload;
its repository menu omitted reload and global refresh could restart pending work.

The repository menu now exposes Reload Repository (Ctrl+R), switching to Refresh PR
(Ctrl+R) for an accepted review. Ctrl+R and both repository/PR menu entries resolve
to ReaderCommand::Refresh. Existing RefreshPR remains a compatibility spelling and
uses the same handler/policy, preserving host calls. Reload Source remains separate.

The existing can_choose_review predicate supplies accepted repository/not-opening
readiness; pending_pr excludes PR preparation and refresh. Production review jobs
own pending_pr from start through transactional publication/failure/cancellation.
No second readiness state is added. No repository, source, review, history or worker
implementation is copied. Failed refresh retains the accepted reader as before.

## Verification

A new RED regression fails on the old command_enabled default, which permits refresh
without a repository. The green matrix covers no repository, ready checkout, pending
repository opening and pending review, including the compatibility command.

Actual app frames at 1×/2× open the repository menu and click Reload Repository,
find a newly created file, and retain displayed bytes and caret after discovery.
They then install a real pinned review and verify the contextual Refresh PR label.
With a controlled live review worker, clicking the disabled row and entering
Escape/Ctrl+R preserves that exact worker and its non-cancelled flag. Its eventual
controlled failure publishes normally while retaining the accepted snapshot.
Existing live-caret repository restoration, bookmark relocation, PR chooser and
shortcut-disambiguation tests remain in the workspace suite. Initial menu test
expected zero-based line zero; corrected to the existing one-based source contract.
No production caret defect was inferred from that fixture mistake.

The initial duplicate frame setup was replaced with focused_source_frame, reusing
an existing actual-app helper. No new native compositor captures are claimed for
this increment; scale checks are egui frames. The existing native repository and
PR transaction acceptance remains documented in their respective plans.

## Architecture review

Baseline before-contextual-refresh-command at 0999e59: 86,907 NLOC/3,664 functions,
average CCN 3.34, max 132 and 109 clone groups. Final delta +103 NLOC/two regression
functions, unchanged max/average, no new hotspot or cycle, health -0.00. An initial
redundant boolean policy pushed command_enabled across a tool threshold; reusing
can_choose_review and the owned pending state removes that issue.

Final default comparison reports REGRESSION only for existing Toolbar::show CCN
22→23: the new contextual menu label requires one branch. This is necessary UI
behavior within the existing toolbar, not a second menu implementation. No extra
helper is introduced merely to move that branch out of the metric. Normalized
file-pair/whitespace-stripped-fragment audit matches all 109 clones unchanged,
with zero additions/removals. A duplicate test setup found initially was removed
through the existing focused-source helper.

Evidence archive: ~/.local/state/radiology-qa/context-refresh-20261102-0999e59.
Full parity remains open: broader command/owner transitions, toolbar long-label
layout, text/grammar/indexer corpora and final Swift feature/UI acceptance.
Accessibility and actual sibling adoption retain their deferred scope.
