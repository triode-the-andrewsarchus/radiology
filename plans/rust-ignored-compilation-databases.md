# Ignored Git compilation databases — 2026-11-02

## Contract and implementation

Checkout discovery now includes Git-ignored files named `compile_commands.json`
at any repository-relative depth. Ordinary tracked/unignored inputs are unchanged.
This covers root databases, `build/`, build variants and nested directories with
spaces. Existing pure planning still chooses the shallowest database and resolves
ties lexically. Project controls use the same discovery boundary as generation.

The existing streamed Git inventory command now accepts its selection arguments.
A second `ls-files --others --ignored --exclude-standard` query uses the explicit
Git glob `:(glob)**/compile_commands.json`. It lists matching files without opting
other ignored source, cache, object or dependency trees into the copy. Both queries
retain the existing 30-second per-process deadline, cancellation, NUL/UTF-8 parser,
64 MiB output and 200,000-file bounds. The combined inventory still goes through
normal validation, sorting/deduplication, total limits and existing deletion/type
handling. No shell evaluation, new scanner, config toggle, dependency or worker.

The same inventory flows through private copying, database rebasing, original-input
verification, cache fingerprint and current/stale reporting. A changed generated
database therefore invalidates cached context. The service never edits the original
database or writes compiler/indexer outputs into the viewed checkout.

Pinned review inspection is deliberately unchanged: it uses verified committed
head inputs rather than borrowing ignored artifacts from the attached checkout.
Non-Git walks retain their existing exclusions. This increment does not provide
ignored generated headers/responses/main sources, external build directories,
symbolic-link database targets excluded from the inventory, Windows/MSVC quoting,
CUDA or general compiler-option/corpus acceptance. Such inputs need further work;
this does not establish complete C++ project support or full Swift parity.

Swift's project detector uses the browser inventory and does not discover ignored
databases either. This extends the Rust build-discovery contract for a common
checkout workflow and keeps the standalone reusable boundary intact.

## Evidence

- RED: the new inventory regression fails before the change with a missing ignored
  root database. It passes after the change for root/nested/space-containing paths,
  shallowest selection, unrelated ignored artifact exclusion, actual private copy,
  byte/source verification and pre-cancellation.
- Existing production-service regression now runs both tracked and ignored nested
  database cases through real isolated child processes and controlled SCIP output.
  Rebasing/response expansion, semantic source digests, disposable paths, original
  database/object preservation, successful cache reload and staleness after editing
  the database pass. The indexer output is synthetic, not real scip-clang acceptance.
- Actual-app discovery regression now removes the database from tracking, ignores
  build/, reinspects and confirms the same Clang evidence before generation.
- Workspace and strict gate results are recorded in PROGRESS.md. No new native
  compositor or real compiler-indexer acceptance is claimed.

## Architecture review

Baseline `before-ignored-compilation-databases` at b41e6f2. Final code delta:
+87 NLOC/one regression function; average CCN 3.34 and maximum 132 unchanged,
no new hotspot/dependency cycle, health -0.01. Existing Git process streaming,
limits and shared repository test support are reused. The large service-test diff
mostly wraps the existing lifecycle in tracked/ignored cases rather than copying
its assertions into a new test. No parallel implementation of database parsing.

Default comparison reports REGRESSION for five existing test clone groups touching
shifted code. Normalized file-pair/whitespace-stripped-fragment Counters match all
109 groups unchanged (zero additions/removals).

Evidence archive: `~/.local/state/radiology-qa/ignored-compdb-20261102-b41e6f2`.
