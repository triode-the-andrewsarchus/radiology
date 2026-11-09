# Generated-index semantic navigation contracts

Reference: all four annotated groups in
[SemanticNavigationTests.swift](../Tests/RadiologyTests/SemanticNavigationTests.swift),
compared with Rust `0f9329e` and this checkpoint. The separate ContextNavigation,
ContextModel, reference/call-tree and palette suites still require their own
assertion reconciliation.

## Corrected behavior

The new regression confirmed that the explicit Go to Definition command was
disabled for external symbols. It silently did nothing instead of explaining why
navigation was unavailable. A definition under the caret could similarly do
nothing after metadata publication, or navigate to itself before publication.

The command now accepts any exact indexed caret identity and delegates destination
selection to the same existing `DefinitionDestination::for_occurrence` used by
Context. That mechanism excludes the current occurrence and deterministically
chooses another repository definition when available. With no destination, the
command reports that the symbol is defined here or outside this repository.
Local names come from exact source bytes; global names use existing index metadata
and descriptor fallback. No source or history transaction starts on that error.
The result is independent of the asynchronous metadata job's publication timing.

Freshness, attached source, Before restrictions, index identity and caret ownership
remain guarded by `current_caret_symbol`. Valid destinations use the existing
encoded-range navigation transaction, preserving source validation and exact
history. The source canvas still offers pointer navigation only for real indexed
definitions, matching Swift's command-click and context-menu affordances.

## Assertion mapping

| Swift group | Rust evidence |
| --- | --- |
| commandClickJumpsAcrossFilesAndBackReturnsToTheExactByte | New both-scale generated fixture has no prebuilt index, installs the existing isolated fake rust-analyzer, generates the real Swift SCIP fixture, and retains exact source digests. Circle on lib.rs line 5 resolves to shapes.rs 1:11 and Context recognizes its definition. Back/Forward/Back preserve the exact departure 5:29 and destination despite a different restoration report. The new error matrix verifies external += and definition-here diagnostics, unchanged source Arc/full history/caret, both before and after metadata publication. Existing actual canvas `control_click_resolves_exact_source_and_preserves_departure_selection_and_caret` and app `indexed_commands_after_a_source_click_resolve_the_clicked_symbol_and_history` verify pointer-to-command routing and exact departure ownership. |
| referencesAndSymbolSearchUseTheGeneratedIndex | Generated Circle references require lib/shapes groups, five hits, four references, one labelled definition, lib lines 2/5 and exact use-line text. Searching circ returns Circle/Struct; canonical symbol activation reaches shapes.rs 1:11. Explicit reference activation reaches the exact lib hit. Query preparation does not change source Arc/history. Existing real picker keyboard/pointer and reference keyboard tests retain their input-specific coverage. |
| switchingRepositoriesDropsTheOldIndexAndTargets | The same generated workflow switches to a plain Swift repository and requires no semantic index, Context, captured inputs, caret identity, ephemeral selection, explicit reference target/hits or old history. The discovery plan reports Swift as uncovered. Existing latest-request retirement/generation tests cover a pending old publication. Rust uses immutable captured index ownership instead of Swift's mutable revision counter. |
| ordinarySemanticSelectionClearsAcrossWhitespaceAndFiles | The preceding both-scale shorthand app regression selects the field's exact Context identity, retains Swift's deliberate cfg-aware syntactic decoration fallback, clears on whitespace and accepted file change, and rejects closing-bracket/module hits. The independent Unicode/external selection test verifies canonical bytes and pin ownership; the complete pure-index mapping verifies exact field identity. |

Rust generation is an explicit Index Projects operation; this test exercises its
actual app/service/tool/cache pipeline rather than manually assigning an index.
The fake tool copies the same checked-in wire fixture used by Swift and writes an
artifact only inside the private work directory. This establishes navigation
through generated data; it is not fresh real compiler/SDK acceptance.

## Validation and architecture

Archive: `~/.local/state/radiology-qa/semantic-navigation-20261109-0f9329e/`.
The new generated workflow passed before the fix; the diagnostic regression failed
on command availability. Both pass after correction at both scales, with pending
and published metadata states. Strict all-target workspace Clippy, normal build
and warning-free API documentation pass. The locked offline workspace run passes
870 application tests (11 existing opt-in ignored), two binary tests, 13 font tests,
22 text tests and all 66 doctests. Compilation uses -j 1 and tests run serially;
normal half-core Cargo configuration remains 16 jobs. The previously recorded
parallel timing/link failure investigation remains open.

Architecture adds 185 measured NLOC/two tests/one focused test file; no production
function, dependency, worker, cache or alternate navigation mechanism is added.
Average/max CCN remain 3.42/132, with 36 hotspots and unchanged dependency cycles.
Complete normalized clone multisets contain the same 124 instances, zero additions
or removals. The raw comparator flags an unchanged old eight-line clone in
app/tests.rs because that file adds a module declaration; its Python chokepoint
exchange concerns untouched equal-score functions. Neither represents new debt.
Existing test-tool installation, source fixtures, app generation/settling,
reference/search APIs, guarded caret identity and destination resolver are reused.

Remaining Context/navigation/reference suites, source/syntax, compiler/indexing,
inspector, final native/standalone acceptance, parallel timing limits and Android
stay open. Full accessibility and actual sibling-app integration remain deferred.
