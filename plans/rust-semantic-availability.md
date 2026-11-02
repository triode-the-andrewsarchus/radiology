# Source-specific semantic availability

## Contract — 2026-11-01

Swift's `SemanticAvailability` distinguishes a missing repository index from a
file outside coverage and the source-only PR Before revision. Rust previously
used a generic revision message or “No exact indexed declaration here.” The
Outline header also omitted Swift's quiet semantic status.

`semantic::availability::SemanticAvailability` now provides one borrowed, pure
availability result for both regions. It resolves no file, Before, nonempty index
coverage and exact source digests before consulting replacement work or failures.
A retained usable index remains usable during reindexing and after failures.
Imports without digests keep their existing unknown-freshness behavior; availability
does not certify freshness. Generated data with changed source bytes gets a
separate Source changed explanation. Empty indexes retain their no-documents
reason. Cache, generation and import failures retain their existing diagnostic
priority; cancellation and missing-index states explain how to retry.

`from_services` binds the existing generator/importer lifecycle to the pure
resolver. It does not probe tools, read files, spawn work or add state. Mere tool
probing is not labeled Indexing. Rust does not have Swift's whole-file semantic
mapping phase; existing under-caret/reference jobs retain their own loading state.

`InspectorContent.semantic_availability` replaces its coarse
`references_available` boolean. Standalone and future embedded callers supply the
same source/index/lifecycle facts. Existing query ownership, pins, workers,
canonical coordinates and source navigation remain unchanged. The old private
reference-input availability accessor is now test-only.

The References status and lower Outline hint share the same reason. The hint
uses existing fitted Small typography and a bounded full-reason tooltip. Status
explanations reuse the existing three-row bounded prose mechanism. Available
sources with an unindexed declaration get Not in the index with Swift's
configuration/indexer explanation. Initial reference loading says Finding
references. Unavailable/loading/error headings omit an unproven numeric zero;
failed queries no longer also render zero-reference coverage. A loaded zero still
uses the normal coverage path. Worker failures retain a References unavailable
heading and their complete scrollable diagnostic. More extensive failure-detail
presentation remains a separate UI audit item.

No dependency, cache, persistent model, application service or sibling checkout
requirement was added. Requested SwiftUI/macOS/typography skills remain unavailable
in this environment; this change reuses the established type roles and fitters.

## Evidence

- Pure resolver tests exercise missing/empty indexes, lifecycle reason priority,
  cancellation, loading, Before, absent document coverage, exact generated digests,
  unknown-provenance imports and retained usable indexes during replacement/failure.
- Real app frames transition through missing/empty/uncovered/stale/Before and back
  to usable indexed context. Component frames verify matching region headings,
  bounded explanations and absence of false zero coverage at 1×/1.5×/2× in light
  and dark themes. Worker diagnostics and indexed/no-symbol/loading status are
  checked separately.
- Workspace passes **553 app tests, seven ignored**, two binary, four font,
  13 reader-text and 36 doctests with eight test threads. The first default-thread
  run timed out in the pre-existing source-loader five-second wait; the unchanged
  test passes in the full rerun. Initial scale measurement needed a warm frame
  because egui applies the pending pixels-per-point change at frame start.
- Strict workspace/all-target Clippy, check, binary build, rustdoc, fmt and diff
  checks pass. Architecture: +451 NLOC including tests, nine functions; resolver
  CCN 8, service binding 3, reference render 11, status selection 5. App render
  remains CCN 36; lower region remains 15. No new hotspot, max-CCN increase or
  cycle. All 91 clone groups match the baseline normalized multiset exactly;
  the comparator's touched old reference-test clone warning adds no duplication.
  Health changes -0.01.
- Eight native cases cover absent index, empty index, uncovered source and malformed
  import at 1× and 2×. Each verifies matching headings, retained source structure,
  no generic/false-zero status, exact source copy, fixture/binary hashes and zero
  child exits. The 2× uncovered capture was visually inspected. These are controlled
  fixture checks, not broad language/indexer/PR corpus acceptance.

Durable evidence: `~/.local/state/radiology-qa/semantic-availability-20261101-08d1c49`.
Broader inspector/input/corpus, reader/language/index/PR/cache and final Swift UI/
standalone/reuse acceptance remain open. Full accessibility and actual sibling
adoption remain deferred.
