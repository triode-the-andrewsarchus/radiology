# Rust symbol picker parity

## Search foundation — 2026-11-02

`SymbolSearch::rank` and `SemanticIndex::search_symbols` preserve their synchronous
APIs. Ranking now keeps only the requested best candidates, preserving stable
input order for equal ranking keys and the unsorted inventory prefix for an empty
query. Cancellation is checked before work, every 128 entries and before publishing;
partial matches are discarded. `SymbolSearchMatch` now carries the entry's
`definition_count`. `SymbolSearch::VISIBLE_LIMIT` is 200, matching Swift's default;
the old picker has not been connected to that constant yet and still requests 100.

Accepted source outlines now publish `Arc<[DocumentSymbol]>` snapshots. Existing
slice readers dereference them; host assignments convert a Vec with `.into()`.
This allows a worker to retain an accepted outline without cloning every symbol
on each query or frame. Repository/review reset publishes an empty snapshot.
No new worker pool or dependency is introduced in this foundation.

The differential regression compares bounded insertion against a stable full-sort
reference across 4,096 entries, seven query patterns and limits 0/1/200/5000,
including tied names/paths, Unicode, identity, line and definition counts. It also
cancels after partial ranking and verifies that a fresh search succeeds.

## Picker integration accepted — 2026-11-02

The picker now owns a cancellable LatestRead request over either the accepted
`Arc<SemanticIndex>` or `Arc<[DocumentSymbol]>`. Query and inventory changes retire
both publication and queued activation; closing cancels pending work. Searches run
on the existing bounded read pool, with cached results between changes. Public
`GoToSymbolModal::show` now accepts `Option<&Arc<SemanticIndex>>` and a shared
outline; clones retain query/selection but start independent jobs and popup IDs.

Workspace results use Swift's 200-result limit. Body names, Small kind/definition
counts, and a bounded second path/line row replace the old single-line layout.
The 620×430 popup adds No Symbols/No Matches, loading/failure states, an accurate
shown-result count with index/document context, Cancel and Go. The document-outline
fallback retains all its navigable declarations, including the existing 1,000-row
virtualization case. This useful Rust extension is preserved. Selected secondary
text uses the primary color for readability, while font size retains hierarchy.

Both pickers now use concrete shared components for window/query/body/footer,
fixed-height measured rows, selection reveal, ordered keyboard input and closing
lifecycle. Their domain searches and destinations remain separate. Shared input
handles every repeated arrow, processes text before the next navigation key,
waits for that query before Return and drops closing suffixes. Row ownership
includes the popup and request generation, so release after index/source/query
replacement cannot activate a newly displayed destination.

Symbol-name length ranking now counts extended grapheme clusters, matching
Swift's Character count. The existing locked unicode-segmentation 1.13.3 package
is now a direct dependency; no package version changes. CJK, decomposed accents,
ZWJ families and flags rank ahead of longer ASCII names. The subsequent
[workspace Unicode repair](rust-symbol-unicode.md) now covers canonical matching,
grapheme boundaries and NFC ordering. Document substring
filtering now shares those primitives and passes its own regression/native audit;
this is not a claim of whole-application parity.

## Integration verification

Seven symbol-picker regressions preserve the former semantic keyboard/pointer,
same-frame query/Return and 1,000-symbol document cases, then add the 200-result
limit/metadata, 128 repeated arrows/closing suffixes, in-flight retirement,
empty/long Unicode rows at both scales and request-owned pointer releases. A new
semantic regression checks Swift-style grapheme lengths. Shared file-picker
regressions also pass, including selection clipping and failure/retry.

Fresh final workspace: 596 application tests passed, seven ignored; two binary,
four font, 13 text and 37 doctests passed. Strict Clippy, all-target check, binary
build, rustdoc with warnings denied, formatting and diff checks pass. A test-only
Clippy constructor warning was corrected before final verification. A copied
standalone executable also passes resource verification from a fresh directory.

Native symbol v2 runs at 1× and 2× use a real imported SCIP fixture: 200 results,
selection 128, exact destination line copy, Back, selection 150 narrowed to symbol7
plus Return, duplicate-definition metadata, Escape/Cancel source focus, No Matches,
long Unicode name/path fitting, then light-theme pointer Go. Six exact line copies
per run verify caret destinations rather than only the file path. A tight binary
OCR crop verifies two definitions; full captures are inspected. File-picker v6
regression runs at both scales pass all former flows and seven exact source copies
each. All four final runs retain unchanged binary/fixture hashes and zero child
exits. Initial symbol v1 full-row OCR misses are excluded; its capture also prompted
the selected metadata color correction before final v2 runs.

Architecture against `before-symbol-picker-ui`: default REGRESSION, +457 NLOC,
26 functions/two focused modules, max new CCN 15, no new hotspot/cycle, health +0.02.
Initial duplicate rendering/lifecycle logic was removed through actual shared
components. Full normalized audit shows three removed old test clones and three
new boilerplate groups: imports, the shared begin/Escape call-site guard and
cancellation-test assertions. Total stays 109; none duplicates ranking or row
rendering. The warning is retained rather than changing analysis scope or removing
needed guards/assertions to affect the metric.

Evidence: `~/.local/state/radiology-qa/symbol-picker-ui-20261102-a4579f2`.

Broader application parity remains open. Full accessibility and actual sibling
integration remain deferred.

## Foundation verification

Fresh workspace: 591 application tests passed, seven ignored; two binary, four
font, 13 text and 37 doctests passed. Strict Clippy, all-target check, binary
build, rustdoc with warnings denied, formatting and diff checks pass. Architecture
against `before-symbol-search-foundation`: default REGRESSION flags old clones in
an edited test file; full normalized audit shows all 109 groups unchanged. +85
NLOC/two functions, max new CCN 11, no new hotspot/cycle. No native UI improvement
is claimed for this foundation. Evidence:
`~/.local/state/radiology-qa/symbol-foundation-20261102-0652005`.
