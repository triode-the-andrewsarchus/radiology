# Rust indexing parity

## Current implementation (2026-10-28)

`semantic::loader::IndexImporter` imports repository-provided `index.scip`, then
`.scip` as a fallback. File reads, protobuf decoding and semantic table building
run on the shared four-worker pool. It uses the existing `LatestRead` ownership
protocol: replacing, cancelling or dropping a job retires its publication channel.
Repository changes and acceptance of a different pinned review drop the importer.
Checkout imports are never reused as review indexes.

The app publishes a successful import atomically in `poll_index`; independent
hosts call it alongside `poll_source`. The eframe loop drives both. Reload and
Cancel are reader commands. A failed or cancelled reload retains any previous
semantic data. Status distinguishes loading, missing, failed, cancelled and
imported data. Imported files have **unknown freshness**, and the status says so;
there is no provenance claim from simply finding an index file. The source footer
reports semantic data only when the current document is present in the index.
Failures expose file-specific diagnostics. No external tool is installed or run.

Input is bounded to 256 MiB, checked both before and after reading. Empty indexes,
invalid repository-relative document paths, malformed protobuf and I/O failures
are reported; a failed first candidate still permits the second to load. Native
protobuf decoding/table building currently run to completion once entered;
cancellation prevents publication but does not interrupt those stages.

## Project strategies and commands (2026-10-28)

`semantic::indexing::IndexPlan` detects Cargo, Go, TypeScript/JavaScript, Python,
C/C++ compilation databases and Gradle Kotlin/Java projects from the unfiltered
repository inventory. Nested manifests are suppressed per adapter. Source counts
include tests regardless of reader filters. TypeScript configs take precedence
over package roots under the same configured directory. Plans sort by project
depth, registry priority and path, select at most 32 projects and retain skipped
projects explicitly. Invalid and duplicate inventory paths do not create work.
Python markers are considered together to avoid duplicate nested Python runs
when a parent uses a different manifest type from its child.

The app computes plans on `LatestRead`, retires them on repository/review changes,
and publishes via `poll_index`. The Index Projects menu shows project evidence,
languages, counts and cap omissions. Tool availability is not implied by a plan;
generation is still explicitly unavailable in the current app.

`IndexingStrategy::command` describes all six adapter invocations using argument
vectors. Paths with spaces or shell punctuation remain literal arguments. The
Kotlin command embeds the existing Swift shell/Gradle compiler integration as
per-run resources, avoiding a second implementation. Resource files are returned
as data; this planning stage writes nothing and launches no tools. Rust disables
build scripts/proc macros and requests all features, with debug/miri cfgs. Explicit
per-manifest feature cfg injection remains to be ported. Kotlin provisioning also
requires replacing the Swift installer's macOS-only `ditto` copy operation.

An isolated workspace, validated tool executable, private run directory and output
path are caller requirements. The process runner/workspace/service must enforce
them before connecting Generate/Reindex. In particular, C/C++ compilation database
absolute paths require rebasing to isolated inputs; planning alone does not solve
that. Repository discovery currently omits build directories, so a compilation
database there may require the broader build-input inventory used by Swift.

Fresh verification for this round: 89 application tests, 5 reader-text tests,
30 doctests; formatting and Clippy with warnings denied pass. New tests cover
multilingual ordering/counts/evidence, manifest nesting and sibling boundaries,
32-project overflow, invalid/duplicate paths, structured commands, and app filter/
repository isolation. No fresh native visual acceptance was performed. Architecture
comparison retains only the previously documented findings.

## Indexer process runner (2026-10-28)

`semantic::indexing::process::run` executes structured commands on a blocking
worker. stdin is closed; both output pipes are nonblocking and drained with a
per-turn budget. Each stream retains at most 32 KiB of first bytes plus 32 KiB
of last bytes, with total counts and an omission marker. No unbounded output
files or pipe-reader threads are created. Deadlines and owner cancellation are
checked every polling turn, including under continuous output.

Every child has its own POSIX process group. All completion paths kill remaining
group members and reap the leader, including a normally exiting tool that leaves
a child holding its pipes open. The leader remains waitable with `waitid` and
`WNOWAIT` until group cleanup, reserving its identity during the final signal.
Cancellation currently uses SIGKILL for prompt termination; graceful SIGTERM
escalation is a possible refinement. Children that deliberately escape the group
are not contained; this is process lifecycle management, not an OS sandbox.
Non-POSIX platforms return an explicit unsupported error.

Nonzero exits and signals are completed results; cancellation/deadline/I/O errors
carry bounded diagnostics. Resource materialization and input isolation remain
caller responsibilities. `libc` is now a direct dependency for POSIX descriptor,
wait and signal operations, reusing the existing locked version.

Fresh verification for this stage: 96 application tests, 5 reader-text tests and
30 doctests; fmt and Clippy with warnings denied pass. Real fixture subprocesses
exercise large simultaneous streams, nonzero/signal/spawn failures, prelaunch
retirement, live cancellation, timeout with TERM-ignoring descendants, and cleanup
of children after normal tool exit. Linux `/proc` evidence checks that descendant
processes stop. Architecture retains only the previously documented findings.

## Swift behavior still required

The authoritative implementations are `SemanticIndexService.swift`,
`SemanticIndexer.swift`, `IndexingStrategy.swift`, `IndexCache.swift`,
`IndexingWorkspace.swift`, `IndexerProcess.swift` and the six indexer adapters.
The import lifecycle above is only part of R4 in the independent parity review.

- Expand build-input discovery to include compilation databases omitted by the
  browser inventory, and integrate generated document coverage counts. Project
  strategy detection and the 32-strategy cap are implemented above.
- Probe actual executable/version availability, including missing rustup proxies;
  expose tool-specific installation plans and private provisioning.
- Generate indexes with cancellable bounded-diagnostic child processes, deadlines,
  isolated build inputs/workspaces where required, and no writes to viewed trees.
- Validate per-strategy fingerprints/tool versions/content digests; atomically
  cache and merge independently scoped projects with partial/stale outcomes.
- Reindex and refresh lifecycles; preserve useful prior data while reporting
  incomplete, cancelled, failed or stale strategies truthfully.
- Index pinned After snapshots and reuse only content-validated strategies.
  Before must remain isolated from After semantic data.
- Move remaining repository discovery/Git refresh/context work off the UI thread.

## Import-stage verification

Previous import-stage workspace run: 82 application tests, 5 reader-text tests, 30 doctests;
formatting and Clippy with warnings denied pass. New tests cover corrupt-primary
fallback, file provenance/diagnostics, replace/cancel publication, missing/empty/
oversized/escaping imports, real app publication, failed reload retention and
repository-switch retirement. No new native visual acceptance was performed for
this status strip. Full indexing and Swift parity remain open.

Architecture comparison against `parity-implementation-start` retains the known
two test-only clone findings and review-preparation structural finding; this
change adds no reported product clone or complexity regression.
