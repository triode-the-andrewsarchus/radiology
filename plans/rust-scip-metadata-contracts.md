# SCIP tool metadata retention

Reference: `SemanticIndexTests.decodesARealRustAnalyzerIndex`, Swift's
SCIPMetadata decoder and SemanticIndex source records, compared with Rust
`c798eb8` plus this checkpoint. This closes the metadata loss identified in the
[caret identity audit](rust-semantic-identity-contracts.md).

## Data and ownership

The existing decoder retained metadata text encoding while skipping ToolInfo and
project root. The real Swift fixture's serialized source record was absent
(`metadata-red.log`). It now decodes tool name, version, ordered arguments and
project-root URI through the existing guarded protobuf reader.

One public `ScipIndexSource` record represents a contributing output, including
its original document count. `ScipIndex::sources` and `SemanticIndex::sources`
share that representation. This fits Rust's existing merged-index pipeline:
individual wire outputs contribute one record; project merging appends records
in contribution order. Raw document count precedes filtering and overlap removal,
matching Swift's source-record count semantics. Failed projects contribute no
record. Missing wire metadata leaves an empty collection.

The existing `text_document_encoding` field remains the single owner of the raw
encoding value, preserving the public encoding contract. Adapter column overrides
and explicit document encodings retain their existing behavior. Source records
are informational: project-root URIs never change the attached root, source-read
storage, revision, freshness, path validation or navigation coordinates.

Metadata stays in existing project and normalized cache payloads. The optional
serde-defaulted source collection lets previous cache records load with unknown
metadata represented by an empty collection. No format bump, rewrite-on-read,
extra cache, worker or dependency was introduced. New records survive fresh,
partially successful, reused and cache-only publication paths.

## Assertions and acceptance

| Contract | Evidence |
| --- | --- |
| Real Rust-analyzer wire data | Extended fixture test requires tool name rust-analyzer, file URI, original two-document count, lib/shapes paths, UTF-8 document encodings, total_area's Function kind and exact signature, and Unicode café/rocket documentation. Normalization retains the identical records and column encoding. This maps the complete `decodesARealRustAnalyzerIndex` Swift group. |
| Complete fields and cancellation | Existing nested-message test now includes tool/version, ordered Unicode arguments, project root and unknown metadata/tool fields. Every observed retirement checkpoint still returns cancellation with no partial index, and a fresh decode recovers every field. Large-string/packed-array/malformed-message tests continue to exercise the shared guarded reader. |
| Wire ordering and malformed tool data | New test places documents before metadata, requires the final metadata message to replace the previous message, checks Unicode root preservation and original document count, and confirms metadata does not rebind document paths. Missing metadata stays empty; invalid UTF-8 ToolInfo fails decoding. |
| Multi-project generation | Existing isolated Rust/TypeScript/failed-Go test now emits metadata from the shared fake-tool fixture. Two ordered source records survive filtering, with original output counts two/one while accepted documents total two. Versions and reported private staging roots remain intact without affecting source coordinates or checkout isolation. |
| Cache-only and older records | Extended partial-coverage cache test removes tools and requires identical source records/arguments after reload. An old-format payload with the source field removed still loads documents/provenance, reports empty metadata and preserves exact cache bytes. Existing report/freshness checks remain. |
| Project reuse | Extended reuse workflow requires identical records after both projects reuse, then equality through a mixed reused/fresh update and cache reload. Tool/input invalidation and exact project-cache bytes retain their existing assertions. |
| External consumer | A separate package depends on the current public library and pinned backend/grammar patches. Locked offline build passes with 16 jobs. Its executable decodes the embedded real fixture, uses the re-exported ScipIndexSource, normalizes records, and validates exact Circle definition coordinates from an empty working directory. No sibling checkout is used or modified. |

The consumer sources/manifest/lockfile and logs are archived with this checkpoint.
It verifies this new API and metadata/source-coordinate boundary; the broader
clean-build/resource/full-reader host gate retains its
[existing acceptance](rust-reader-host-audit.md). No new compositor, SDK, compiler
installation or UI/style redesign is claimed.

## Validation and architecture

Archive: `~/.local/state/radiology-qa/scip-metadata-20261109-c798eb8/`.
The real fixture, nine protobuf checks, 42 indexing service tests (ten existing
opt-in ignored) and extended project-reuse workflow pass. The initial public-struct
addition exposed one exhaustive internal test initializer; it now supplies the
empty source collection explicitly. The independent consumer build and run pass.
The locked offline workspace run passes 864 app tests (11 existing ignored),
two binary/13 font/22 text tests and all 66 doctests. Strict all-target Clippy,
normal build and warning-free API docs pass; the generated source-record page
contains every field, the example and captured-repository ownership contract.
Workspace compilation uses -j 1 with serial tests after the prior unconfirmed
parallel-link failure; normal half-core Cargo configuration remains 16 jobs.

Baseline `before-scip-metadata-20261109` to `final-scip-metadata-20261109`:
+175 measured NLOC, three functions (one nested ToolInfo decoder and two tests),
no new measured file, average/max CCN 3.42/132, 36 hotspots and unchanged cycles.
Complete normalized clone multisets retain all 123 instances with zero additions/
removals. Raw comparator warning identifies a pre-existing touched service-test
clone; equal-score untouched Python chokepoint ranks may also move. Existing protobuf, normalization, merging, cache, fixture and host mechanisms
are reused; one shared source-record type prevents parallel metadata models.

With the earlier shorthand-field correction, two of the eleven SemanticIndexTests
groups now have complete explicit mappings. The remaining semantic/Context/
navigation groups, other behavior suites, final native/standalone acceptance,
known parallel timing limits and Android remain open. Full accessibility and actual
sibling adoption remain deferred; keyboard support and independent operation stay
required.
