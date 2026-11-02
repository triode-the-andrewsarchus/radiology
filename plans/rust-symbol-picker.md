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

## Remaining picker work

Move current workspace/outline search off rendering using the existing LatestRead
pool and shared immutable inputs. Reuse ordered keyboard handling from the file
picker; add Swift's 200-result workspace limit, kind/definition metadata, bounded
two-line rows, empty states and footer Go/Cancel. Protect pointer ownership and
retire query/index/source replacement results. Preserve the useful Rust document
outline fallback and all its navigable symbols. Check source focus and exact
semantic destinations through real app frames and native UI at both scales.

The foundation intentionally preserves the existing UTF-8 byte-length tie break.
Swift counts extended grapheme clusters; Unicode name-length ordering still needs
correction and specific acceptance before claiming symbol-search parity.

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
