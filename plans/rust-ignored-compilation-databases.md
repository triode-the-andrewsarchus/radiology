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
At this initial checkpoint, non-Git walks retained their existing exclusions.
The conventional non-Git lookup below now closes that discovery gap. Neither increment provides
ignored generated headers/responses/main sources, external build directories,
symbolic-link database targets excluded from the inventory, Windows/MSVC quoting,
CUDA or general compiler-option/corpus acceptance. Such inputs need further work;
this does not establish complete C++ project support or full Swift parity.

The earlier Swift reference used only the browser inventory. Updated main now
adds bounded conventional-location probes through CompilationDatabaseDiscovery,
including non-Git folders. The Rust Git glob remains broader for ignored databases;
the new non-Git checkpoint below matches the updated Swift contract.

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

## Conventional non-Git discovery — 2026-11-02

A source archive or ordinary filesystem folder now finds an existing database
under hidden build directories. Non-Git discovery retains its non-following walk
and browser exclusions, then probes the root plus the first 63 sorted directories
containing discovered CMakeLists.txt files. Each probes compile_commands.json and
the build/, out/, Debug/, Release/, cmake-build-debug/ and cmake-build-release/
locations used by Swift. At most 448 metadata probes are made, with cancellation
checks. Missing/non-directory locations and directory-valued database names do
not become files; other I/O failures surface through existing discovery errors.

The result uses existing path validation, sorting/deduplication, file/path bounds,
private copying, database rebasing, cache fingerprints and source verification.
Symbolic-link inputs/parents retain their established isolation guards. Hidden
build trees are not recursively admitted, so unrelated excluded build objects and
caches stay out of the copy. Browser contents are unchanged. Pinned review input
inspection still uses exact head blobs and never probes mutable checkout artifacts.
Inspection controls and generation share this method; removing the database
restores visible best-effort planning. No settings, worker, parser or dependency
was added.

The new inventory regression fails on the old code at the hidden root database.
It now verifies root/nested discovery, sorted directory bound, exclusion, private
copying, source/input verification and cancellation. The existing full-service
rebasing/response/cache/staleness case now covers tracked Git, ignored Git and
non-Git inputs through identical assertions. Actual app frames show the nested
evidence without the best-effort warning and retain buffer/history; removing the
database recovers fallback planning. The existing fixture helper now supports
nested source paths rather than duplicating its setup.

Actual scip-clang C11/C++17 acceptance runs both root and hidden build databases,
checking Unicode/CRLF coordinates, cross-file definitions/references, exact project
reuse, tool-free cache reload and unchanged original inputs. Native 1×/2× runs
index a nested non-Git CMake project with a required configured macro: three
source documents, no missing paths or compiler errors, no best-effort warning.
Go to Symbol opens the definition and copies its exact original bytes; dark/light
and restart load the configured cache without another compiler run. Core exits
zero; input/executable hashes stay unchanged. Initial driver Copy arrived before
asynchronous source publication; the accepted runs observe the destination first.
Generated/definition captures were visually reviewed.

Final workspace: 757 app tests pass, nine opt-in tests ignored; two binary, 13 font,
21 text and 59 doctests pass. Strict Clippy/build/rustdoc/fmt/diff and resource smoke
pass. Against before-nongit-compdb at5dba415: +151 NLOC/three functions, unchanged
analyzed file count, avg/max CCN3.37/132,33 functions above15,125 clones/1,335 duplicate
lines and one cycle. The new lookup has CCN9. All125 normalized clone instances
match the baseline; raw touched-file clone warnings are pre-existing.

Evidence: ~/.local/state/radiology-qa/nongit-compdb-20261102-5dba415.
Ignored generated headers/response files/source trees, external build inputs and
broader compiler-option/dependency corpora remain distinct acceptance work.
