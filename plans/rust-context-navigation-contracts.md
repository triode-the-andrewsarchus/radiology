# Context inspector navigation contracts

Reference: all nine annotated groups in
[ContextNavigationTests.swift](../Tests/RadiologyTests/ContextNavigationTests.swift),
compared with Rust `847e1f0` and this checkpoint. ContextModel's eighteen groups
and the reference/call-tree/palette suites remain separate reconciliation work.

## Corrected declaration-link departure

The actual enclosing-declaration link promised that Back returns to the caret,
but its inspector action requested a reading-position departure. A new real
sidebar-click regression placed the caret inside total_area, scrolled to line
120, activated the declaration, then went Back. It returned to byte 4453 (the
viewport) instead of byte 183 (the body caret).

The inspector action now uses the existing default exact-caret navigation intent.
It still reveals and flashes the declaration, records history and uses the same
atomic source transaction. This removes one incorrect flag; no new helper,
production function, job, cache, dependency or alternative history mechanism.
Other structural reading-trail owners retain their own behavior.

## Assertion mapping

Three app regressions use the real Swift Rust-analyzer fixture, existing
checkout/settling/input helpers and actual egui frames at 1× and 2×.

| Swift group | Rust evidence |
| --- | --- |
| passiveCaretMovementOnlyUpdatesContext | The exact seven-position lib caret sequence updates to Circle inside total_area; cross-file definition preview loads. After every refresh/frame, the accepted source Arc, full history, painted top and restoration position stay unchanged. Definition disclosure also leaves history untouched. Reference selection/disclosure and asynchronous source/caller preparation are independently passive below. |
| scrollingDoesNotChangeTheEnclosingDeclaration | The sidebar test scrolls the real long lib reader to line 120 while the caret remains inside total_area. The unindexed long Rust reader moves its viewport to line 150 after a genuine keyboard caret move and keeps the caret, full history and enclosing only declaration. |
| referencesPreviewWithoutNavigatingAndBackReturnsToTheCaret | Actual area references in shapes produce the lib call and shapes definition. Selecting/disclosing the call loads source containing item.area() without changing source Arc/history/caret. Reference Return through the shared keyboard handler commits the call; Back/Forward/Back retains exact destinations and shapes 8:12 departure despite a distinct restoration report. |
| nativeListActionsCommitTheNamedRowNotTheStoredSelection | Selection is deliberately on the definition while explicit Commit(call) opens the lib row. An invalid named row leaves history and pending work unchanged. Existing reference-row renderer tests require single-click selection and double-click commit. Popup owner IDs include the exact source path/range/digest, and shared context-menu tests retire missing/changed owners; actions are built from the live model rather than persisting old numeric rows. |
| callerTreeNavigationReturnsToExactDepartureCaret | Preparing the lib reference's caller tree yields total_area without navigating. Explicit caller activation reaches the exact call; Back/Forward/Back restores the body caret and caller destination. Existing changed-caller-source regression rejects stale bytes before history publication. |
| sidebarDefinitionActivationsRecordTheCaretNotTheViewport | Under-caret area activation after a real distant scroll reaches shapes.rs 7:11 and Back returns to lib 8:21. Then the actual rendered declaration link opens total_area at 5:7 and Back returns to lib body 7:10. The latter failed before the one-flag correction. Existing inspector link-renderer tests route both name and location links to these same actions. |
| fileAndRepositoryChangesResetContext | A genuinely completed shapes result is held before publication: old lib remains accepted internally, but transient enclosing Context is unavailable. Publishing the result restores the Circle declaration and no old token identity. Repository replacement clears semantic inputs/under-caret/references; opening a plain Rust function preserves structural only context with no invented semantic data. See pending ownership below. |
| unindexedFileInAnIndexedRepositoryIsLabelledByFileCoverage | Opening the fixture's real Cargo.toml retains the repository index but paints File not indexed, with no token/declaration. The plain Rust repository paints No semantic index while retaining parsed structure. Existing availability/explanation tests check the shared file-coverage wording and source/revision precedence. |
| viewerReportsCaretOnlyForSelectionChanges | In the unindexed long function, a real source ArrowRight after a rocket moves to the exact canonical byte. Subsequent rendered scroll restoration changes the reading anchor to line 150, while caret/history/enclosing declaration remain unchanged. Rust uses UTF-8 source bytes and scalar columns; shared coordinate tests separately cover exact UTF-16 producer conversions. No AppKit callback counters are claimed. |

## Pending ownership and scope

Swift clears its old model immediately on a file request. Rust preserves the
accepted reader internally until the next source is validated and atomically
published, allowing cancellation/failure recovery. `enclosing_source_context`
and source-ready gating hide transient caret/declaration actions while pending.
The held-result test verifies that boundary rather than requiring deletion of
the still-accepted source. Successful publication and repository replacement
retire the previous semantic ownership.

The long source fixtures append comments or repeat ordinary statements solely to
permit actual distant viewport movement. They preserve existing indexed source
coordinates and use the explicitly unverified importer. They do not claim new
compiler provenance or native compositor acceptance. Full accessibility and actual
sibling-app integration remain deferred.

## Verification and architecture

Archive: `~/.local/state/radiology-qa/context-navigation-20261109-847e1f0/`.
The declaration-click test failed at the viewport byte before the fix. All three
final focused tests pass at both scales, including named/keyboard/caller commits
and exact forward restoration. A draft preview assertion retained a source Arc
across Back; Back may accept a fresh validated read, so the final assertion captures
the active source for each passive-preview transaction.
Strict all-target workspace Clippy, normal build, warning-free API docs, formatting
and diff checks pass. The locked offline workspace run passes 873 application
tests (11 existing opt-in ignored), two binary tests, 13 font tests, 22 text tests
and all 66 doctests. Compilation uses -j 1 with serial tests; normal half-core
Cargo configuration remains 16 jobs. This does not close the separately recorded
parallel timing/linking investigation.

Architecture: +265 measured NLOC/three tests/one focused test file, one production
flag removed. Average/max CCN remain 3.42/132, with unchanged 36 hotspots/cycles.
Complete normalized clone multisets retain 124 instances, zero additions/removals.
The comparator's old tests.rs clone is unchanged despite its new module line;
Python chokepoint rank exchanges affect untouched equal-score functions. Existing
fixtures, source loading/publication, Context/reference/disclosure workers, input,
viewport and exact-caret transaction are reused.

Next: ContextModel, reference/call-tree/palette assertions, source/syntax,
compiler/indexing and inspector suites; final native/standalone acceptance,
parallel timing/linking limits and Android remain open.
