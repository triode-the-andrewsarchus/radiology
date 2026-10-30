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
and publishes via `poll_index`. The Index Projects view shows project evidence,
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

## Tool availability and native status (2026-10-28)

`ToolDirectory` captures managed/user/PATH lookup locations explicitly. Managed
Go/npm/Clang/Kotlin tools take precedence; conventional Cargo and Go locations
remain supported. Lookup checks executable bits and regular-file targets rather
than mere existence. Version probes use the bounded process runner with a
10-second deadline. Empty/nonzero/unexpected Rust responses are unavailable,
including rustup proxies whose rust-analyzer component is missing. Rustup symlink
identity is preserved for argv[0] dispatch; Kotlin resolves its actual distribution
for plugin discovery. Child PATH matches the lookup context.

Index Projects is now a resizable window with a scrollable project list and
persistent Check Tools/Cancel Checks controls. A popup at the bottom status bar
was found to clip its controls in native testing and was replaced. Checks run on
`LatestRead`, deduplicate adapters, preserve previous results during cancellation,
and retire on repository/review transitions through shared index-state reset.
`poll_index` publishes checks; independent hosts can query `tool_availability`.
Opening a repository does not launch version probes automatically. In reviews,
checks use the pinned After workspace's directory. Tool installation and generation
are still unavailable; no successful probe is treated as a generated index.

Fresh verification: 100 application tests, 5 reader-text tests, 30 doctests; fmt,
Clippy with warnings denied and the native binary build pass. Fixture tests cover
managed precedence, symlink dispatch, missing components, empty responses,
nonexecutables, stderr versions, deadlines, app publication/cancellation and stale
repository retirement. Architecture retains only the previously documented flags.

Native acceptance reused little-screen-clip's existing private-compositor and
virtual-pointer **test helpers only**, via a temporary driver; no sibling source
or runtime dependency was added. In a private 1280×900 Weston/Niri session:
opened Index Projects, invoked Check Tools on six managed fixture executables,
verified successful versions and a Python status-7 failure, scrolled to C/C++ and
Kotlin, observed running/cancel controls, invoked Cancel Checks for a slow fixture
and confirmed its process stopped. Current source remained visible. The app and
both compositors exited with status zero. Captures are under
`/tmp/radiology-index-tools-native-window/`: `projects.png`,
`checks-complete.png`, `checks-lower-projects.png`, `checks-running.png`, and
`checks-cancelled.png`. Native evidence used fake indexers, not live generation.

## Build-input inventory and isolated workspaces (2026-10-28)

`InputInventory::discover` uses the full Git tracked/unignored inventory, including
tracked `.cargo`, build and vendor files omitted by the browser. Deleted files are
dropped; listed directories/submodules are reported as omitted, never fabricated.
Detached Git worktrees are supported. Non-Git folders use a non-following walk,
retaining `.cargo` configuration while excluding build/dependency output trees.
UTF-8 paths are validated, deduplicated and sorted. Discovery is cancellable and
bounded to 200,000 inputs and 64 MiB of path data. Unreadable/enumeration failures
are errors rather than silently complete inventories.

The process runner now supports `run_observed`: consumers receive every raw output
chunk while retained diagnostics remain bounded. Git inventory parsing streams
NUL-delimited paths through this mechanism, so large inventories do not silently
truncate at the 64 KiB diagnostic limit. Consumer errors terminate the child group.

`IndexingWorkspace` owns a unique disposable directory outside the viewed tree.
Cache-parent symlinks are resolved before writes; an internal parent is rejected.
Copies preserve source bytes and permission modes. Paths traversing symlink
directories are rejected. Regular-file opens do not follow leaf links on POSIX.
Internal symbolic links are redirected within the copy; dangling internal links
remain dangling. External Markdown links become literal link text without reading
the external target. External source/build links fail explicitly. Cleanup runs on
success, errors, cancellation and owner drop.

The copy defaults to 256 MiB per file and 2 GiB of input bytes, configurable through
`WorkspaceLimits`. Cancellation is checked between files and each 64 KiB read.
Input identities include type, mode, SHA-256 bytes and Git-compatible SHA-1 blob
IDs. The SHA-256 repository fingerprint covers original paths, bytes and modes,
independently of random workspace paths. `verify_inputs` detects build rewrites;
`verify_source` detects edits to copied inputs in the viewed tree;
`verify_snapshot` compares exact SHA-1 Git blob inventories. A fresh inventory is
also required to detect additions before claiming a current index. Snapshot
inventories using other Git object formats are not yet supported here.

This is filesystem isolation, not an OS sandbox: arbitrary build scripts could
still reference absolute external paths. The generation service must control
known output directories, validate provenance and reject modified inputs. Full
dependency environments, submodule materialization, C/C++ compilation database
rebasing and pinned-snapshot inventory routing remain service integration work.
SHA-1 is used only for Git/Swift identity interoperability; mutation verification
and repository fingerprints use SHA-256. `sha1`/`sha2` are direct dependencies;
`tempfile` is promoted from test-only use for workspace lifetime management. No
pre-existing locked dependency versions changed.

Fresh verification: 110 application tests, 5 reader-text tests and 30 doctests;
fmt and Clippy with warnings denied pass. Real Git/process/filesystem fixtures
cover inventories beyond the diagnostic cap, tracked build/config inputs, ignored
and deleted files, worktrees, omitted submodules, exact Git blob identities,
permissions, internal/dangling/external links, input rewrites, source edits, cache
parent links, limits, cancellation after workspace creation and drop cleanup.
Architecture comparison retains the previously documented flags; the initial
workspace setup/materialization complexity and stream-drain duplication were
removed before acceptance. No native app generation acceptance was performed:
Generate/Reindex is not connected yet.

## Semantic source provenance boundary (2026-10-28)

`SourceBuffer::content_digest` caches the exact SHA-256 byte identity during source
preparation. `SemanticIndex::source_digests` can associate generated documents with
those bytes. Occurrence/range queries reject mismatched buffers; encoded definition
navigation carries the target digest and checks it after reading the destination.
Imports with no digest map preserve their explicitly unverified behavior. The
source footer requires coverage and matching bytes. Generation must populate the
map; this boundary alone does not create an index or complete source refresh.

Fresh verification for this step: 111 application tests, 5 reader-text tests,
30 doctests, fmt and Clippy pass. Architecture comparison now also ranks the
pre-existing workspace materialization body as a structural hotspot, in addition
to the previously recorded findings. No product clone/CCN threshold regression
was introduced by the digest boundary.

## Checkout generation service (2026-10-28)

`service::IndexGenerator` is independent of egui. Requests capture a root, private
cache parent, explicit tool lookup context, process deadline and copy limits.
It uses `LatestRead` for generation ownership and a bounded shared progress
snapshot, without an event backlog. Starting, cancelling or dropping a request
retires its publication and cancels its process group.

Generation discovers/replans against full inputs and keeps a pristine baseline.
Each project runs in its own disposable copy of that baseline, so build outputs
from another project cannot change its inputs. Probe/execution working directories
use the copy. Command resources and output are private; Cargo target output is
redirected there, with jobs capped at half available logical CPUs. Real source
bytes are never replaced with synthetic manifests. Known output controls are not
an OS sandbox for arbitrary build scripts.

After success, input type/mode/bytes are checked and the common bounded SCIP
reader decodes only regular generated output files. Project-relative documents
are rebased; column encoding is made explicit before merging. Non-source,
non-input and symbolic-link-text documents are filtered. Local IDs remain scoped
by document. Overlapping documents retain the first document and make the report
partial; they are not silently merged across unrelated local scopes. Actual
source digests populate the semantic provenance boundary. Command paths carry the
original root so Python project names remain stable across random build copies;
a subprocess argv test verifies this boundary. Missing path counts
are complete, with at most 100 names retained per project. Process diagnostics
remain bounded; Kotlin partial-build markers are detected across all streamed
stderr bytes even when omitted from retained logs.

The final pristine baseline is checked against current viewed inputs and a fresh
inventory. Changed, added or removed inputs produce a stale report. Individual
project failures preserve successful projects. The host decides whether to replace
its previous index; an all-failed run carries no replacement. This initial generation stage did not persist output; the subsequent
[persistent cache integration](rust-index-cache.md) now restores it across app
restarts. Full semantic dependency resolution remains unproven.

Current capability limits: checkout generation only; C/C++ is gated until its
compilation database can be rebased to copied paths. Kotlin requires an installed
compiler distribution and executable project wrapper (fallback Gradle selection
is not connected). Ignored dependency environments, submodules, linked source
coverage, Rust extra feature cfg injection, validated cache reuse, private tool
installation and real compiler-indexer acceptance remain open.

Six backend fixture tests cover isolation, project failures, exact source guards,
encodings/local scopes, stale source, cancellation/replacement, invalid/linked
output, timeouts and bounded partial-build diagnostics. Workspace validation with
reader changes in progress: 118 application + 5 reader-text tests, 30 doctests,
format and Clippy pass. One probe test failed once and did not reproduce in 20
parallel library-suite repetitions; enhanced failure diagnostics were retained.
Architecture reports only previously documented clone/structural findings after
removing new product duplication and excessive generation complexity.

## Reader integration and native acceptance (2026-10-28)

Generate/Reindex/Cancel are shared reader commands in the Index Projects window.
The app drives the independent generator through `poll_index`, publishes semantic
data atomically and retains prior data when a run fails or is cancelled. Per-run
coverage, missing-file counts, tool versions and bounded diagnostics are visible.
Current report summaries distinguish generated/partial/stale/no output; they do
not claim full semantic dependency resolution. Imports and generation cannot race
to overwrite each other. Repository/review transitions retire jobs and reports.

The app's public `index_tools` context is shared by probes and generation and can
be configured by an embedded host. Repository/cache context acquisition is shared
with review preparation. Idle construction remains free of configuration writes:
path discovery is pure and saving creates parent directories when needed.

Open-buffer provenance mismatch exposes Reload Source, preserving the caret byte
request through asynchronous preparation. Inspector semantic availability now
requires current-document coverage and matching bytes. F12 requests verify the
target digest. Workspace symbol-palette and caller-row destinations now share
that encoded/digest-checked navigation path; Before uses its document outline.
[Navigation acceptance](rust-semantic-navigation.md) records tests and native
fixture checks. Per-project cache reuse and review indexing still prevent full R11 acceptance.

Native acceptance used the existing sibling private-compositor/pointer helpers
through a temporary driver, without a sibling runtime dependency or source edit.
In a private 1280×900 Weston/Niri session, a fixture indexer emitted a real small
SCIP payload. Generate published it; Reindex kept the existing reader and index;
Find produced a highlighted match during the running subprocess; Cancel stopped
the indexer and removed its workspace while preserving the previous index. Recorded
environment proved `CARGO_BUILD_JOBS=16`, isolated `CARGO_TARGET_DIR` and no generated
artifact in the viewed fixture. App, Weston and Niri exited zero. Captures under
`/tmp/radiology-generation-native/`: `before-generate.png`,
`generation-complete.png`, `reindex-responsive-find.png`,
`generation-cancelled.png`. These predate the final singular-label/status wording
cleanup; they prove the workflow, not pixel acceptance of those final strings.

Current final suite: 120 application tests, 5 reader-text tests and 30 doctests;
formatting, Clippy with warnings denied and binary build pass. Added app command,
retention, source-reload and repository retirement coverage plus an isolated idle
constructor test. One earlier stderr-version probe test failed once; it did not
reproduce in 20 parallel library-suite runs and now prints its actual result on
failure. Cause remains unproven. Architecture flags are two retained test-only
clones and structural rankings for review preparation, workspace materialization
and generation orchestration. New product duplication and excessive generation
CCN were removed; the overall result is still REGRESSION, not a clean gate.

The system rust-analyzer proxy reports its component missing. Native acceptance
therefore used fixture tools and does not prove real compiler/indexer output.
Real tool acceptance, persistent validated cache reuse, private provisioning,
C/C++ database rebasing, Kotlin fallback Gradle/dependency environments, linked
source coverage, Rust extra cfg injection, pinned After generation and automatic
open/refresh lifecycle remain required. Before never borrows After semantic data.

## Persistent cache integration (2026-10-28)

Generated data now persists atomically with its input inventory, exact source
identities and bounded reports. Repository opening/Reload Index restore it without
tools or input copies, label stale results and fall back to unverified imports if
absent/invalid. Cancellation and repository transitions retire both paths. Backend,
application and native restart checks are recorded in [cache acceptance](rust-index-cache.md).
Earlier statements that persistence is absent are superseded by this integration;
per-project/version-aware reuse, pinned After caching and retention remain open.

## C/C++ isolated generation integration (2026-10-28)

The earlier C/C++ generation gate is removed for supported repository-local
compilation databases. The service rebases directories, sources, known path
options, outputs and copied response files, and passes an absolute prepared file
under its private run to the adapter. Original database bytes remain verified.
[C/C++ acceptance and limits](rust-clang-indexing.md) records backend, real-Clang,
service/cache and native fixture acceptance. Ignored build-database discovery,
external build directories, uncommon compiler path protocols and actual
`scip-clang` acceptance remain open. Historical C/C++ gate statements above are
superseded by this integration, without claiming complete project coverage.

## Busy executable startup (2026-10-30)

Full-suite tool-probe failures now include concrete Linux ETXTBSY (`Text file
busy`) evidence. A deterministic regression holds an executable write handle open,
verifies that the kernel rejects exec with ETXTBSY, then releases it. The original
runner fails immediately; the repaired runner waits and launches successfully.
Persistent-busy and cancellation variants verify the original timeout/cancel
contract before any tool starts. The precise writer responsible for the earlier
parallel-suite race was not traced; inherited descriptors during concurrent
fork/exec are a plausible explanation, not a proven attribution.

Only ETXTBSY is retried, before process launch, within the existing wall-clock
deadline and cancellation token. Nonzero tool exits and other spawn errors retain
their previous behavior. No command is re-executed after launch, no arbitrary
probe timeout is enlarged, and process-group ownership/draining remain unchanged.
The shared runner covers generation, tool probes and Git input enumeration.
Architecture comparison against `before-busy-executable-start` is neutral.
Final validation passes 157 application tests, 5 reader-text tests, 30 doctests,
fmt, Clippy with warnings denied and binary build. Twenty additional parallel
library-suite runs pass, including the deterministic busy-executable regression.

## Generation acceptance contract and remaining integration

Generate/Reindex is connected for checkout work. Preserve these publication rules
while completing caches, refresh, pinned snapshots and semantic destinations:

- Discover/replan on the worker using the full input inventory. Keep one pristine
  baseline copy alive for the run; create an independent build copy per strategy
  so generated files from one tool do not influence another tool's inputs.
- Reuse `LatestRead` for replacement/cancellation and the existing process runner
  for both probes and execution. Live progress needs bounded shared state, not
  another retirement protocol or unbounded event queue.
- Materialize each command's resources in a private run directory. Override known
  build output paths (especially Cargo target directories); retain the half-core
  Cargo build limit. Do not create shadow manifests or missing submodule contents.
- Share the importer's bounded SCIP read/decode path. Rebase project-relative
  documents, set explicit per-document encodings before merging, filter documents
  to real input paths, preserve local symbol scopes, and report overlapping or
  missing document coverage instead of claiming a complete workspace.
- Verify each build copy after its tool exits. Before publication, compare the
  pristine baseline to the viewed inputs and a fresh inventory, including newly
  added/deleted paths. Keep source changes and partial builds visible in status.
- Carry source-byte provenance into generated semantic data. A loaded immutable
  source buffer may predate a run even when disk inputs were current at indexing
  time. Caret lookups and navigation must reject mismatched buffer bytes; caching
  source digests avoids hashing whole files on each UI interaction.
- Keep useful previously published data during failure/cancellation, with explicit
  stale/unverified status. Do not let imports and generated results race to
  overwrite each other. Repository/review transitions retire every indexing job.
- Keep Generate/Reindex/Cancel controls and per-strategy outcomes/diagnostics
  responsive while completing open/refresh and cache lifecycle integration.
- Pinned review generation must verify the worktree against exact Git object
  inventories. Checkout imports must never be applied to Before/After by accident.
- Persist only validated outputs with tool/config/content provenance and atomic
  publication. Cache reuse must be measured against the full repository context,
  including sibling dependencies. Generation alone does not finish cache parity.

## Swift behavior still required

The authoritative implementations are `SemanticIndexService.swift`,
`SemanticIndexer.swift`, `IndexingStrategy.swift`, `IndexCache.swift`,
`IndexingWorkspace.swift`, `IndexerProcess.swift` and the six indexer adapters.
The import lifecycle above is only part of R4 in the independent parity review.

- Connect full build-input discovery and project replanning to generation, then
  integrate generated document coverage counts. Strategy detection, full input
  inventory and the 32-strategy cap are implemented above.
- Add tool-specific installation plans, prerequisite checks and private
  provisioning. Executable/version availability probes are implemented above.
- Connect generated indexes to the cancellable process runner and isolated
  workspaces; control known output paths and verify inputs before publication.
- Validate per-strategy fingerprints/tool versions/content digests; atomically
  cache and merge independently scoped projects with partial/stale outcomes.
- Complete automatic open/refresh and persistent-cache lifecycles. Explicit
  Generate/Reindex/Cancel now retain prior data with per-run outcomes.
- Index pinned After snapshots and reuse only content-validated strategies.
  Before must remain isolated from After semantic data.
- Finish context/cache responsiveness, finer decode interruption and cancellation
  propagation through remaining source consumers. Source-loader Git reads now
  have [bounded cancellation](rust-source-process-cancellation.md).
  [Repository discovery](rust-repository-opening.md) and
  [checkout Git refresh](rust-git-refresh.md) are now worker-owned.

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
