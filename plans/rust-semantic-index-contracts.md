# Semantic-index contract reconciliation

Reference: all eleven annotated groups in
[SemanticIndexTests.swift](../Tests/RadiologyTests/SemanticIndexTests.swift),
compared with Rust `2f3bc46` and this checkpoint. The earlier
[identity correction](rust-semantic-identity-contracts.md) and
[metadata retention](rust-scip-metadata-contracts.md) repaired discovered runtime
gaps. This checkpoint verifies the remaining index contracts through existing
production mechanisms; it adds no runtime API, dependency, worker or cache.

## Assertion mapping

| Swift group | Rust evidence |
| --- | --- |
| decodesARealRustAnalyzerIndex | Prior metadata checkpoint's real wire fixture requires tool/root, document paths and encodings, total_area's Function kind/signature and Unicode documentation. |
| definitionsResolveAcrossFilesAndLocalsStayPerDocument | New real-fixture test requires exactly one global Circle definition at shapes.rs 0:11–17, distinct local-0 identities, items/radius names and each local definition's document. |
| caretResolutionUsesByteOffsetsAndHalfOpenRanges | Real lib occurrences all map without coordinate loss. Exact Circle/total_area/area under-caret Context requires names, kinds, definition flags and destination lines; definition activation excludes itself. Prior fixture rejects the closing bracket and multiline module caret hits. External += has known core identity and no repository destination. |
| shorthandFieldPrefersTheRepositorySymbolOverTheLocal | Prior pure-index and both-scale app checks retain exact field identity/counts/destination and Swift's deliberate cfg-aware syntactic decoration fallback. |
| occurrenceSelectionUsesIdentityWithoutRequiringARepositoryDefinition | Both-scale actual app selection retains core operator identity and nonempty ranges, and clears on whitespace. The shared Context lookup independently requires zero definitions/no destination. |
| occurrenceSelectionPreservesUnicodeByteBoundariesUntilTextKit | Actual app selection at byte 6 retains the Unicode symbol, active 4..13 and both 4..13/23..32 ranges. Shared CoordinateIndex yields UTF-16 location/length pairs (4,6)/(20,6). Both UTF-8 and UTF-16 index encodings run at 1×/2×; pinning retains exact ranges and leaves original source bytes untouched. |
| searchEntriesRankExactPrefixInitialsAndFuzzy | Real fixture includes Circle/total_area/area/new and excludes local items. New exact five-entry matrix checks circle tiers 0/1, sbg tier 2, ta prefix/initial/fuzzy order, tlar subsequence and absent zzz. Existing bounded/Unicode/stable-sort tests remain. |
| columnConversionsHonorEncodingsAndRejectOutOfBounds | New coordinate matrix requires valid UTF-8/16/32 offsets, EOL, out-of-bounds line/column, inverse units, second-line byte and multiline 4..21. Negative packed ranges fail construction; invalid scalar boundaries fail mapping. See explicit platform policy below. |
| staleAndMalformedDataDegradeWithoutFabricatingPositions | Shortened real shapes source leaves unmapped occurrences; every surviving range fits its exact bytes. Truncated protobuf returns error; empty protobuf returns no documents. Prior invalid-path matrix drops escaping/absolute/empty/dot-component documents before derived maps/search. |
| typedRangesAndUnknownFieldsDecode | Extended nested wire/cancellation fixture combines packed and typed single-line occurrence ranges, typed multiline enclosing range (field 11), definition role, UTF-16 document encoding and unknown occurrence/top-level fields. Typed range wins. All cancellation checkpoints and a fresh complete decode recover exact values. Fallback descriptors are independently checked below. |
| symbolStringFallbackNamesHandleDescriptors | Extended existing fallback test uses Swift's exact impl-method, core generic/operator, quoted TypeScript filename/name, local 12 and escaped-header-space cases. |

## Coordinate and UI boundaries

Rust intentionally rejects positions inside UTF-8 scalars or UTF-16 surrogate
pairs rather than clamping malformed producer coordinates as Swift does. This
preserves the existing exact-source contract: valid coordinates match, invalid
coordinates never fabricate a source range. Negative columns are rejected by
packed-range construction and checked signed-to-unsigned conversion.

The shared reader-text coordinate index supplies UTF-16 equivalents without
introducing TextKit into the Rust host. Selection and pin storage retain canonical
UTF-8 byte ranges. Existing cfg-aware syntactic fallback remains unchanged.
These app checks exercise actual egui state at both scales; they do not claim
new native compositor, IME-engine or full accessibility acceptance.

## Verification and architecture

Evidence archive: `~/.local/state/radiology-qa/semantic-index-audit-20261109-2f3bc46/`.
All 187 semantic tests pass (11 existing opt-in checks ignored). After the final
fixture assertions, 21 index tests, nine protobuf tests and both app tests pass;
the Unicode app matrix covers two encodings at both scales. Strict locked/offline
all-target workspace Clippy, formatting and diff checks pass. No fresh full
workspace/build/docs run was needed for this test-only checkpoint; `2f3bc46`
remains the latest full result (864 app tests and all 66 doctests).

The first coordinate-test attempt incorrectly unwrapped a negative packed range;
the decoder already rejected it correctly, and the test now requires rejection
explicitly. A final normalization assertion initially used the wrong module path
for SemanticSymbolID; the compiler caught it and the final focused runs use the
existing index-module API.

Architecture review is NEUTRAL: +304 measured NLOC/four test functions, no new
measured files or production functions, unchanged average/max CCN 3.42/132 and
36 hotspots. Complete normalized clone multisets retain 124 instances with zero
additions/removals; dependency cycles are unchanged. The comparator's reported
Python chokepoint exchange concerns untouched equal-score functions, not this
diff. Existing fixture/document/occurrence constructors, SourceBuffer conversions,
CoordinateIndex, production Context/selection/pin paths and guarded decoder are
reused. No wrappers, duplicate coordinate parsers or product abstractions added.

The remaining Context/navigation/reference/call-tree suites still need their own
assertion reconciliation. Source/syntax, compiler/indexing and inspector audits,
final native/standalone acceptance, parallel timing limits and Android remain
open. Full accessibility and actual sibling-app integration remain deferred.
