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

## Swift behavior still required

The authoritative implementations are `SemanticIndexService.swift`,
`SemanticIndexer.swift`, `IndexingStrategy.swift`, `IndexCache.swift`,
`IndexingWorkspace.swift`, `IndexerProcess.swift` and the six indexer adapters.
The import lifecycle above is only part of R4 in the independent parity review.

- Detect project strategies for rust-analyzer, scip-go, scip-typescript,
  scip-python, scip-clang and scip-kotlin, with nested project suppression,
  evidence, language/source coverage counts and the 32-strategy cap.
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

## Verification

Fresh workspace run: 82 application tests, 5 reader-text tests, 30 doctests;
formatting and Clippy with warnings denied pass. New tests cover corrupt-primary
fallback, file provenance/diagnostics, replace/cancel publication, missing/empty/
oversized/escaping imports, real app publication, failed reload retention and
repository-switch retirement. No new native visual acceptance was performed for
this status strip. Full indexing and Swift parity remain open.

Architecture comparison against `parity-implementation-start` retains the known
two test-only clone findings and review-preparation structural finding; this
change adds no reported product clone or complexity regression.
