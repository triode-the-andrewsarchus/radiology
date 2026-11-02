# Indexing language coverage diagnostics

## Contract — 2026-11-02

Swift's `SemanticIndexService.unsupportedLanguages` reports source language
families absent from selected indexing strategies. Rust now displays the same
coarse diagnostic in Index Projects, using its existing language registry and
adapter metadata. Sorted, deduplicated names follow the selected project list;
ancillary formats (including build metadata) are excluded. Objective-C `.m` and
`.mm` receive matching language metadata, without claiming native grammar or
indexer support. TypeScript dialects and Kotlin script aliases follow their
adapter families. Projects excluded by the run limit do not count as selected.

This describes language families, not per-file coverage or tool availability.
An imported index may still contain these languages. It does not change
`GenerationReport::is_partial`; existing document/source coverage remains the
separate authoritative check. A selected project does not promise every file of
its language is indexed.

Empty plans distinguish active inspection from no detected build projects.
Initial planning uses the unfiltered browser inventory; Generate Index checks
complete build inputs. Reader filters do not alter this diagnostic. Root/review
transitions retire the existing background plan job and its display metadata.

`IndexPlan.uncovered_languages` is derived, skipped by serde, and recomputed from
current request inventory on cache publication. Persisted selected/skipped
strategies retain strict provenance validation. Cache format 3 and stored bytes
are unchanged, so existing caches remain usable without installed tools. A stale
cache reflects newly present language families while retaining original semantic
source identities and stale warnings. Existing background plan/generation/cache
paths own the calculation; no per-frame inventory scan, worker or service added.
The app-frame test renderer is shared with existing outline/status tests.

## Evidence

The plan matrix checks selected adapters, aliases, exclusions, capped projects,
sorted/deduplicated names and serialization omission. The cache roundtrip removes
tools before restore, checks unchanged cache bytes and preserved provenance, then
adds Ruby source and checks stale-cache diagnostic refresh. The existing real-app
planning case renders the exact row, preserves it through filtering and retires
it on repository switch. Removing only the new UI row makes that test fail at
its missing-row assertion. Earlier private-API/return-type compile setup failures
are excluded; formatting was corrected before final gates.

Final workspace: 574 app tests pass, seven ignored; two binary, four font, 13 text
and 37 doctests pass. Strict Clippy, all-target check, binary build, rustdoc with
warnings denied, formatting and diff checks pass.

Four native runs pass: mixed Rust/Swift/Objective-C and no-project Python/Ruby at
1×/2×. Each verifies the visible diagnostic, generation result, dark/light
presentation, exact source clipboard, unchanged fixture/binary hashes and zero
child exits. Mixed runs invoke a controlled fixture indexer exactly once, restore
the generated cache after restart without another invocation, and verify unchanged
cache bytes and omission of derived metadata. This proves the UI/cache lifecycle,
not real indexer corpus coverage. Representative dark/light captures inspected.
The first harness selector matched multiple bottom-bar “Index” words; final v2
selects the unique “Projects” token. No product change was needed for that retry.
Sibling compositor helpers are QA-only; the product remains independently built.

Architecture baseline `before-index-language-coverage` at `0b223e8`: +153 NLOC,
two functions, unchanged maximum CCN 100, 24 existing high-complexity functions,
and zero cycles. The new helper has CCN 4. Full normalized clone multiset audit
confirms all 91 groups unchanged, zero added/removed. Automated REGRESSION flags
five pre-existing clones in touched test files; no new duplication remains after
sharing the existing frame helper. Health changes approximately -0.01.

Archive: `~/.local/state/radiology-qa/index-language-coverage-20261102-0b223e8`.
`accepted.json` selects four v2 runs; logs, controller scripts, exact source copies,
fixture hashes, cache/counter evidence, captures, architecture reports, clone audit
and source diff are retained. Requested SwiftUI/macOS/typography skills remain
absent from installed roots; existing bounded text roles and type scale reused.

Broader grammar/corpus, reader/key/UI, real indexing/PR/cache and final Swift
side-by-side acceptance remain open. Full accessibility and actual sibling
integration remain deferred per user scope.
