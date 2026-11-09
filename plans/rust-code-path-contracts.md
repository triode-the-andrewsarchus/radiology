# Code Paths behavioral contracts

Reference: all five `CodePathTests`, eight `CodePathSymbolTests`, two
`CodePathNavigationTests` and one `CodePathUITests` groups, compared with Rust
`33560ab` plus this checkpoint. All sixteen now have explicit mappings below.
No production change was needed; this pass fixes an insufficient test fixture,
adds combined saved-review/layout evidence and removes a duplicated review test.

## Format, persistence and collection mapping

| Swift group | Rust evidence |
| --- | --- |
| `parsesOrderedStepsCRLFUnicodeAndEscapedSymbolsAndRoundTrips` | `ordered_unicode_escaped_fields_and_literal_descriptions_round_trip` checks BOM/CRLF, Unicode title, escaped file/function delimiters, literal description colons/backslashes, authored order and fresh IDs on text reimport. Repeated-step/newline and real 91-step seL4 tests separately retain order and export equivalence. |
| `rejectsMalformedImports` | Parser matrix rejects missing title/steps/fields, absolute/parent paths, empty function/description and NUL. Physical-line tests reject a bad suffix after a valid record. Extended session and app imports now fail after a valid prefix and preserve the accepted collection, selected identity, reader and archive. |
| `diagnosesActualInputLineAndCapsInputSize` | Exact physical Line 4/Line 3 diagnostics survive blank lines and CRLF. UTF-8 test accepts exactly 2 MiB and rejects one additional byte. File-import tests separately bound reads and reject nonregular/invalid UTF-8 data. |
| `persistsMultipleTracesPerCanonicalRepositoryAndDeletesOnlySelection` | Existing canonical store and scope-switch tests preserve authored trace/step IDs, repeat titles/destinations, aliases and independent repositories. Extended real import/variant/Back/Copy/relaunch UI flow adds a second trace, rejects a malformed import atomically, deletes only the selected second trace, restores the first trace/step ID, then deletes the final trace without replacing source. No-root imports are rejected. The saved-review app flow below adds canonical alias relaunch and attached PR scope. |
| `corruptedStorageIsVisibleAndImportDoesNotOverwriteIt` | Extended session test exercises both a loaded collection whose archive becomes corrupt and a fresh session loading corrupt storage. Errors remain visible; subsequent imports and deletion preserve rows and exact corrupt bytes. Explicit repair/reload recovers. Existing store tests also protect foreign/versioned archives and linked storage, without rewriting malformed records on load. |

Rust uses app-owned repository-keyed files with serialized background writes;
Swift uses a UserDefaults archive. Publication follows successful persistence,
not optimistic rows. Existing accepted writes finish across scope changes and
shutdown, and a reopened load waits for the earlier write. This storage mechanism
is independent of the eventual sibling host and does not write into viewed source.

## Symbol resolution mapping

| Swift group | Rust evidence |
| --- | --- |
| `resolvesTheSameFunctionAfterLinesShiftWithoutAnIndex` | `current_source_lines_shift_without_a_semantic_index` resolves exact new bytes after Unicode/comment insertion. The combined app flow below repeats this after actual checkout refresh. |
| `qualifiedNamesDisambiguateMethodsAndOutOfClassDefinitions` | Qualified C++ method/namespace tests select A/B/C and nested N::D declarations and reject ambiguous unqualified boot. Source selections point at real name tokens. |
| `fullHeadersDisambiguateOverloadsAndDefinitionsWinOverPrototypes` | Exact-header tests tolerate header whitespace, choose the definition over its prototype and reject ordinary overload ambiguity; a unique start definition outranks its declaration. |
| `supportsRustSwiftPythonAndAssemblyLabels` | Same four language/type/label cases as Swift resolve exact terminal name bytes, with no semantic index. |
| `refusesMissingSymbolsCommentsUnsupportedTextAndAmbiguousLabels` | Comment/string/call-only, unsupported text and duplicate labels fail instead of guessing a destination. |
| `assemblyEntryMacrosAreDeclarationsAndCommentsAreIgnored` | Existing matrix checks BEGIN_FUNC, ENTRY, ENTRY_NP, SYM_FUNC_START and SYM_CODE_START at the real declaration after comments. String literals, calls and closing macros do not establish entry points. |
| `conditionalDefinitionsAreSelectableButOverloadsStayAmbiguous` | Existing C/C++/assembly tests return source-ordered opposing #if/#elif/#else alternatives and reject same-branch duplicates or independent conditionals. Added direct check retains Swift's invalid end/start position rejection. Actual app/UI tests initially open the first alternative, explicitly choose the second and preserve Back; an unavailable variant is atomic. |
| `conditionalBodyBracesDoNotLoseARecognizedFunctionHead` | Conditional-brace recovery resolves create_idle_thread on line one, refuses configureIdleThread calls and never creates call scopes/folds from recovered navigation-only declarations. |

Existing masking tests preserve Unicode/CRLF byte coordinates and cancel without
partial publication. The real 91-step seL4 corpus has its own documented resolver
and app probes; this checkpoint does not rerun a dependency-heavy seL4 build.

## Integrated navigation mapping

| Swift group | Rust evidence |
| --- | --- |
| `importJumpBackRelaunchAndPRUseAttachedRepositoryAndAfterCoordinates` | New `saved_trace_keeps_attached_scope_through_review_back_relaunch_and_shifted_checkout` runs at both scales. It durably imports, jumps and returns to the exact caret, reloads through a canonical dot alias, enters a real pinned PR with the same collection, jumps from Before to After and returns exactly to Before. Checkout exit and another repository do not change saved ownership; reopening restores the same IDs. Three inserted lines plus real refresh move the resolved function to line five. The earlier focused review regression was merged into this workflow: equal-head/different-Arc review replacement still retires a queued error. |
| `missingFilesSymbolsBinaryAndEscapingSymlinksNeverMoveReaderAndNewNavigationWins` | Corrected `missing_ambiguous_and_cancelled_steps_preserve_source_caret_and_history` now creates bad.c inside the attached root. Previously it wrote outside source and could pass on a missing-file error. It now requires the actual ambiguous diagnostic, plus distinct missing-symbol/binary/symlink errors; every failure preserves source Arc, exact caret, complete history and the absence of a successful step. Existing queued-success/error, same-ID trace replacement, root/review/source replacement, cancellation and actual Right/Left keyboard-batch tests reject stale results, even after the caret returns to its original position. |

Rust intentionally retains the attached root and reads pinned Git objects rather
than replacing the repository root with Swift's materialized head tree. Code Paths
is a left peer tab, and existing working-tree Changes remains available. Import
reveals that tab while preserving independent inspector/bookmark preferences, as
specified in the existing standalone/sidebar contract. These are documented Linux
and future-host conventions, not claims of identical macOS chrome.

## UI mapping

`nativeSidebarAndImportSheetRenderAcrossWidthsAndAppearances` maps to the new
`sidebar_variants_errors_and_import_render_within_narrow_widths_in_both_themes`
and existing native/sidebar/help/pane evidence. The new test imports Swift's
three-step boot text plus another saved trace, prepares real conditional/error
states, then paints 220/280/420-point sidebars at 1×/2× in dark/light themes.
Step descriptions remain complete, narrow text wraps, all painted labels fit
horizontally, counts/selected trace stay correct and conditional choices render.
The actual import modal renders with chooser/Cancel controls in every scale/theme.
A draft assertion incorrectly required wrapping at the widest width; it now
requires wrapping where constrained and checks bounds at every width.

Existing UI tests exercise actual row/variant/menu clicks, copy/delete/relaunch,
keyboard selection versus commit, Tab/Return/Escape, synthetic IME and source
preservation. The 10,000-row test bounds visible painting and preserves complete
descriptions and last-step selection. The shared inspector-pane and keyboard-help
tests separately render/recover all panes and narrow help while retaining focus
and source state; both suites are rerun here. Existing native 1×/2× sidebar/import/
portal/help evidence remains linked in their dedicated plans. This pass adds egui
frames, not a fresh compositor or Swift/AppKit run. Actual IME-engine acceptance
and final combined native workflows remain separate open gates.

## Verification and architecture

Archive: `~/.local/state/radiology-qa/code-path-contracts-20261109-33560ab/`.
All 42 Code Paths tests, four inspector-pane checks and two keyboard-help checks
pass, with strict all-target workspace Clippy, formatting and diff checks.
`code-paths-final.log`, `panes.log`, `help.log` and `clippy-final.log` record the
final runs. The new saved-review workflow and layout matrix also have focused logs.

Architecture baseline `before-code-path-contracts-20261109` to
`final-code-path-contracts-20261109`: +209 measured NLOC/one net test function,
unchanged 674 measured files, average CCN 3.41 to 3.42, max 132, 36 hotspots and
unchanged cycles. Full normalized clone multisets retain all 123 instances with
zero additions/removals. A draft duplicate review scenario was removed by merging
its queued-error ownership assertion into the broader saved-trace flow. Final raw
comparison is NEUTRAL. Shared import, source/review, collection and renderer fixtures
are reused without new product mechanisms.
No fresh full-workspace/build/docs run is claimed for this test-only change;
`091cd94` remains the latest full run (854 app tests, 11 existing ignored, all
65 doctests). No public API, dependency, type hierarchy or product styling changed.

All four Code Paths suites are mapped. Semantic/context/navigation, source/syntax,
compiler/indexing and inspector assertions still require remaining reconciliation,
then final whole-app/native and standalone acceptance maintenance. Known parallel
timing limits and Android remain open. Full accessibility and actual sibling
adoption remain deferred; keyboard support and independent operation stay required.
