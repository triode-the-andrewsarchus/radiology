# Rust/Swift parity reconciliation — 2026-11-02

Rechecked against Rust `3add1c3` and the imported Swift main reference `7edfc79`.
The [initial review](rust-swift-parity-review-2026-10-27.md) describes `fd421e1`;
its Missing/Broken entries are historical findings. This ledger maps those findings
to current implementation and test evidence, and keeps the outstanding acceptance
work explicit. Overall parity is still unproven.

[Semantic identity corrections](rust-semantic-identity-contracts.md) now prefer
navigable/global equal-range targets, preserve cfg-aware spelling decorations, exclude
multiline module caret hits and drop invalid source documents before indexing.
The shorthand-field group is mapped; remaining semantic/Context/navigation groups
and newly identified raw SCIP tool/root metadata retention remain open.

[Code Paths contracts](rust-code-path-contracts.md) map all four suites (16 Swift
groups). Real ambiguous/binary/symlink failures retain the reader; saved traces
survive checkout/PR/Before-After/Back/relaunch/scope and shifted-source refresh.
Atomic multi-trace failures and both-scale/theme narrow sidebar/import rendering
pass. Other suites and final whole-app/native acceptance remain open.

[Critical drawing contracts](rust-critical-drawing-contracts.md) now map the three
Swift groups through actual source paint/copy/fold/replacement frames at both
scales/themes. With the preceding checkpoints, all nine Critical Lens/hover suites
(29 groups) have explicit mappings. Other suites and final acceptance remain open.

[Critical worker contracts](rust-critical-worker-contracts.md) now map all five
Swift groups through immutable worker replacement, exact callee digests, cached
request coherence and real pinned-read viewer/structure promotion with cancellation
and child reaping. No runtime correction was needed. Drawing is reconciled above;
other suites/final acceptance remain open.

[Critical summary/hover contracts](rust-critical-summary-contracts.md) now map
four summary-model and two popup-host groups, plus the first worker group. No-fold
sources schedule no annotation work; glyph-anchored transit stays attached to the
presented popup until replacement dwell finishes. Worker groups are reconciled
above; three drawing groups, other suites and final acceptance remain open.

Seven [Critical model/projection groups](rust-critical-model-projection-contracts.md)
are now reconciled. Unsupported files stop app projection work, while decoded-index
constant/caret flows and Swift's actual Rust-analyzer fixture establish the model
and byte-range contracts. Summary/hover are reconciled above; remaining worker/
drawing and final acceptance stay open.

The [frontier audit](rust-critical-frontier-contracts.md) closes all three Swift
reachability groups and repairs false proven-zero results from independently
supplied conditional summaries. Compact hover status now includes conditional
site counts; other Critical Lens workflows and final acceptance remain open.

[Critical policy and storage](rust-critical-policy-contracts.md) reconcile the
five Swift policy/override groups without runtime changes. The expanded matrix
checks classification reasons and reset precedence; actual session and filesystem
checks establish invalid-data fallback and reported deletion failures. Other
Critical Lens workflows and final acceptance remain open.

The integrated review suite now has all four assertion groups mapped. Its final
[failure/supersession audit](rust-review-retirement-contracts.md) combines actual
CLI retries, three live-child cancellations, repository/PR replacement and pinned
refresh. The newly repaired diagnostic always identifies its failed PR number.
The remaining whole-app and native acceptance scope is unchanged.

[Changed-block navigation](rust-change-block-flow-contracts.md) maps the third
integrated PR group through actual keyboard/test-region/EOF workflows and deferred
source-publication ownership. The retired-review-request group is mapped above.

[Integrated PR browsing](rust-pr-browsing-contracts.md) fixes unchanged-file Before
availability and exact-caret Back behavior after passive scrolling. The combined
head inventory/revision/filter/search/checkout workflow maps the second integrated
group; changed-block and retired loads are reconciled above.

[GitHub metadata contracts](rust-github-metadata-contracts.md) retain the author,
cross-repository flag and merge-commit object through list/view and refresh,
closing the first integrated PR group. Browsing and changed blocks are reconciled
above; retired review loads are mapped above and final acceptance remains open.

[Fresh PR entry](rust-pr-review-entry.md) repairs automatic source/history
selection and retained checkout Files queries. Explicit destinations and same-PR
refresh retain their intended behavior. The four-function integrated audit remains
partial; the richer metadata gap is now repaired above.

[Review fold-default contracts](rust-review-fold-contracts.md) map all seven Swift
groups, including nested/attribute ownership and held publication/manual/revision/
PR-scope outcomes. Runtime behavior is unchanged; the separate integrated-review
audit and final acceptance remain open.

[Changes presentation](rust-review-presentation-contracts.md) fixes the Before
rename selection highlight and maps all three Swift groups, including visible
counts, revision availability and rendered filter reset/empty behavior. Separate
integrated-review/fold audits and final acceptance remain open.

[Review repository contracts](rust-review-repository-contracts.md) map all four
repository groups, including missing-commit diagnostics, older-clone isolation,
three-dot Unicode/CRLF/rename outcomes and exact leased-tree cache behavior.
Separate review UI/fold suites and final acceptance remain open.

[Per-file diff isolation](rust-blob-diff-isolation.md) repairs reused-path rename
contamination through exact object comparisons while preserving Git-link modes
and pointers. One of four repository groups is mapped; the rest remain open.

[Owned review recovery](rust-review-recovery.md) implements Try Again/Choose Number
and failed-request ownership. Real private CLI, source retry, scope retirement,
bounds and keyboard checks map the last partial refresh group. All seven refresh
functions are mapped; other review suites and final acceptance remain open.

[PR refresh lifecycle](rust-pr-refresh-contracts.md) fixes misleading operation
wording and strengthens metadata/viewport/revision/retirement workflows. Six groups
were mapped here; the explicit recovery gap is now repaired above.

[Change metadata and identity](rust-change-metadata-contracts.md) close the two
remaining unified-diff groups. Pinned changes now carry actual SHA-1/SHA-256 IDs,
copy scores and shared numstat application. Together with the diff fixes below,
all five UnifiedDiffTests functions are mapped; other review suites remain open.

[Diff contracts](rust-diff-contracts.md) repair malformed-header rejection and
changed-line/gap mapping. Three of five unified-diff groups and all three change
navigation groups have explicit evidence, including both-scale real Git app
commands. Raw blob metadata/document identity are reconciled above; other suites
remain open.

[Repository search contracts](rust-search-contracts.md) reconcile all six Swift
functions and fix the shared runner's successful cancellation result. Exact Unicode
records, argument boundaries, no-match/error recovery and result caps are checked.
Other suites and integrated acceptance remain open.

[Files reveal contracts](rust-files-reveal-contracts.md) map the four Swift groups
to accepted-source app outcomes. Extended bookmark/search/nested Before workflows
pass without runtime changes; shared search-test completion removes a clone.
Other suites and integrated acceptance remain open.

[Filename/picker contracts](rust-file-order-and-picker-contracts.md) close all five
quick-file functions and replace lexical filename presentation with shared numeric
locale-aware ordering. Other suites and integrated acceptance remain open.

[Source filter contracts](rust-source-filter-contracts.md) close the four Swift
groups and repair loss of manual disclosures during same-root acceptance. Name
ordering is reconciled above; concurrent timing limits and other suites remain open.

[Repository contracts](rust-repository-contracts.md) close the three Swift
discovery/source-store functions and fix external symlink reads through the shared
checkout policy. Filtering assertions are reconciled above; other suites remain open.

[Bookmark command parity](rust-bookmark-command-parity.md) now matches new-entry
declaration labels and ten numbered slots, retaining UUID activation beyond those
slots. The combined scope/restore and supersession flows now pass in the
[three-function navigation checkpoint](rust-bookmark-navigation-contracts.md).
The [model/storage audit](rust-bookmark-model-contracts.md) now closes the separate
three functions, including grouped reordering and deletion-selection repairs.

[App-model contracts](rust-app-model-contracts.md) reconcile all six ordinary
AppModelTests functions through explicit app assertions and the prior root/history
fixes. Other Swift suites, opt-in probe limits and Android acceptance remain open.

[Passive history trail](rust-passive-history-trail.md) fixes loss of an implicit
scrolled visit during structural navigation. Semantic/bookmark/Code Path exact
caret departures stay separately verified. This closes that app-model assertion;
the remaining groups are now reconciled in the
[app-model checkpoint](rust-app-model-contracts.md). Other Swift suites remain.

[Repository file choice](rust-empty-repository-reader.md) removes automatic source
selection after plain discovery/drop and closes that Swift opening-race assertion.
Explicit file/PR and refresh destinations remain verified; other assertions remain.

[Structural history](rust-structural-history.md) fixes caret motion erasing
committed stops and preserves exact departure Back through the existing accepted
navigation transaction. App/native 1×/2× checks pass; app-model assertions are
reconciled in the [six-group checkpoint](rust-app-model-contracts.md), and initial
repository selection is reconciled
in the [file-choice checkpoint](rust-empty-repository-reader.md).

[Scroll/history contracts](rust-scroll-history-contracts.md) reconcile one history
contract and 11 ordinary scrolling groups through expanded app/rendered tests.
The machine-specific opt-in fold probe is explicitly unrun; other suites remain.

[Dialog focus](rust-dialog-focus.md) fixes premature focus completion for File/Symbol
queries, PR entry and Code Path import. Hosted regressions cover inactive/disabled
editors and deliberate Tab ownership; the prior line-prompt rule is retained.

The reference is the checked-in Swift code, queries, fixtures and behavioral tests.
AppKit/SwiftUI are not executed on this Linux host. Broad UI acceptance follows the
user's standalone Linux/reusable-reader direction, including intentional Linux
modifiers and compositor-owned window commands. Full assistive accessibility and
actual little-screen-clip adoption remain deferred.

[Session recovery](rust-session-recovery.md) now closes all seven Swift session
contracts, including missing-root retention, failed/canceled PR retry and launch
supersession, with fresh app and native no-argument relaunch evidence. Other suite
assertions and the SDK-backed Android gate remain open.

[Caret parity](rust-caret-parity.md) closes the three Swift caret contracts for
focused empty selections, folded/empty source and typeset grapheme/EOF placement.
Other behavior assertions retain their checkpoint-specific acceptance boundaries.

[Gutter parity](rust-gutter-parity.md) adds the compact72-point width, sparse
canonical numbering, wrapped-continuation labels and logarithmic function meters.
Both Swift gutter contracts and two meter groups have rendered/native evidence.

[Function contract audit](rust-function-contracts.md) reconciles the four Swift
fold-key contracts and nested Code Path visit/repaint/revisit/Back contract, with
additional source/app/native evidence and unchanged runtime mechanisms.

[Function shading](rust-function-shading-audit.md) now rejects stale/split-byte ranges
and has app/host/native caret ownership evidence. The [pending-context update](rust-pending-source-context.md)
now clears transient context during preparation and verifies cancel/failure/success /
Back. Rust retains readable accepted source for recovery while context is absent.

## Original finding reconciliation

The latest full workspace passes 789 app tests (eleven opt-in ignored), two binary,
13 font, 22 text and 60 doctests. The examples below are bounded evidence for the
original failures; the remaining integrated acceptance is listed afterward.

| Finding | Current implementation and evidence rechecked |
| --- | --- |
| R1: checkout bytes presented as a PR | `ReviewSnapshot`, `SourceRead::Review` and `RadiologyApp::resolve_location` route pinned After and merge-base Before; the app test `review_navigation_reads_pinned_revisions_and_restores_checkout_scope_on_exit` verifies distinct bytes, revision routing and scope restoration. [Pinned source contract](rust-pinned-index.md). |
| R2: whole-PR/context hunks painted as exact file changes | Per-path zero-context snapshot diffs and revision/content-bound overlays replace the aggregate path. Refresh tests exercise renamed files, inserted lines and Before mapping. [Review restoration](rust-pr-refresh-restoration.md). |
| R3: stale state after changing repositories | Repository/source/review/search jobs retire through shared owners; successful repository publication resets the scope, while failed/cancelled opening retains the accepted reader. `repository_transition_clears_prior_index_pins_review_history_and_search` and `repository_open` tests cover these distinct transitions. [Opening transaction](rust-repository-opening.md). |
| R4: no indexing lifecycle | Shared planning, tools/provisioning, isolated generation, cancellation, exact-context reuse and normalized caches are wired through Index Projects. All adapter families have real small-tool checks; real seL4 runs all 67 profiles. Current conventional non-Git discovery closes the newly found Swift delta. [Lifecycle](rust-indexing-parity.md), [personalities](rust-indexing-personalities.md), [database discovery](rust-ignored-compilation-databases.md). |
| R5: local identities and column encodings lost | Document-scoped `SemanticSymbolID`, half-open ranges and source-bound UTF-8/16/32 conversion reach app navigation. `local_utf16_definition_uses_document_identity_and_exact_column` and real compiler navigation checks cover original-byte destinations. [Semantic navigation](rust-semantic-navigation.md). |
| R6: syntax-color dimming substituted for Critical Lens | `refresh_critical_projection` uses parsed calls, exact source/index, policy and overrides; annotation/graph workers drive hover and fold counts. App tests cover delayed indexes, mark persistence, invalidation and mode transitions. [Projection reuse](rust-critical-projection-reuse.md), [marks](rust-critical-marks.md), [graph](rust-call-graph.md). |
| R7: lost columns/revisions/history and hidden destinations | One source publication transaction commits exact byte/revision destinations, departure history and fold reveal. Cross-file, failed-open, PR refresh and Code Path app tests exercise those routes. Native seL4 now proves exact Back/copy on the real compiler union. [Source transactions](rust-navigation-selection.md), [Code Paths](rust-code-path-app.md). |
| R8: shortcut collisions and focus leakage | Exact command matching, source-order input, menu ownership and shared `ModalInput` capture/replay are connected to actual app frames. Focus tests cover opening/closing batches, floating editors and pointer/key handoffs. Native Help, menus and source workflows run at both scales. [Input ownership](rust-keyboard-owner-order.md), [help](rust-keyboard-help.md). |
| R9: unsaved or incorrectly scoped bookmarks | All app mutations use durable stores; navigation resolves anchors, and checkout/PR switches load independent scopes. Existing app tests cover toolbar persistence, relocation, same-commit/different-PR identity and refresh behavior. [Bookmarks](rust-bookmark-pane.md). |
| R10: missing grammars, TSX structure and test hiding | Native grammars and shared declaration/fold/highlight queries feed the reader; AST-aware detectors drive test projections. `native_tests`, native highlight fixtures, test-detector tests and app hide/reveal tests pass. C/C++ and the pinned Kotlin scanner are included. [Outline queries](rust-native-outline-parity.md), [test regions](rust-inline-test-parity.md), [Kotlin](rust-kotlin-scanner.md). |
| R11: blocking reads/analysis and unbounded paint work | Repository/source/index/preview/graph work uses cancellable owners and bounded process/I/O paths. Shared reader-text fragments, sparse coordinates and incremental measurement drive the viewport. Cancellation and large/long-line cases pass; representative integrated performance remains an open gate. [Reader performance](rust-reader-performance.md), [source processes](rust-source-process-cancellation.md). |
| R12: ordinary text matches presented as semantic callers | Reference inputs retain exact scoped index/source identities; parsed call-site validation and bounded caller expansion replace the text-search fallback. Explicit references remain independent of caret following. [Reference context](rust-reference-context.md), [previews/callers](rust-inspector-previews.md). |
| R13: mutually exclusive inspector tabs | Simultaneous independently scrolling/resizable regions, pane preferences, independent under-caret context and trait results are connected. App/native pane, reference, outline and trait workflows have recorded acceptance. [Regions](rust-inspector-region-sizing.md), [traits](rust-trait-inspector.md). |
| R14: missing selection/find/zoom and broken wrapping | Canonical byte selections, original-byte copying, local find, source zoom, variable-height measurement, pins, caret/body shading and destination reveal use shared text primitives. Current app/source/focus tests and dedicated native long-line/emoji/wrap checks cover these routes. Integrated stress remains open. [Selection](rust-source-click-selection.md), [geometry](rust-unwrapped-fragment-geometry.md), [fonts](rust-source-emoji-shaping.md). |
| R15: incomplete operational controls | Files/query recovery, filtered Changes, search/globs/Stop/errors/caps, PR choices, file-manager handoffs, command availability, Code Paths and Help are wired. Source/list/menu actions share commands. Actual Code Path desktop-portal selection, Cancel/retry and durable restart now pass at both scales. [Search](rust-repository-search.md), [controls](rust-reader-notices-controls.md), [Code Paths](rust-code-path-sidebar.md). |
| R16: session/reload/error transitions incomplete | `launch` resolves explicit intent before saved state; repository/review/source publication is transactional. Filter/wrap/theme/font/preferences persist, refresh restores live reader state, and failures expose retained-source status. App tests cover saved PR precedence, cancelled/failed reads and refresh mapping. [Session/PR integration](rust-parity-implementation.md), [refresh](rust-contextual-refresh.md). |
| R17: verification overstated | Fresh workspace/strict/resource/rustdoc gates and scoped native/compiler archives replace the original helper-only evidence. Tests, simulated protocols, real tools and native observations are identified separately in each checkpoint. Final integrated acceptance below remains required. |

The upstream additions are also connected: current-function x folding, committed
Files reveal, inspector preferences/trait implementations, Code Paths, C/C++
fallback, managed Clang, indexing personalities/seL4 and keyboard Help. See the
[upstream ledger](rust-upstream-parity-2026-11-02.md) for their individual evidence.

## Completed external handoff checkpoint

Real desktop file import now passes at both scales through the installed GTK portal:
Unicode path selection, exact editor bytes, Cancel preserving a draft, retry,
keyboard import, persistent collection and restart. Private XDG directories and
D-Bus isolate the test services. See the [portal evidence](rust-code-path-sidebar.md#actual-desktop-portal-acceptance--2026-11-02).

## Completed actual-engine checkpoint

[Real private Fcitx/Anthy acceptance](rust-native-ime.md) now passes at1×/2× after
repairs for native caret-area placement and the pinned backend's ignored preedit
clear. Import and navigation retain exact draft/query bytes after cancellation;
candidate Up/Return commits once, the next Return navigates, and active composition
survives native resize in both controls. Package provenance, backend red/green
regressions and real protocol/key/clipboard/capture evidence are retained. This is
Linux Wayland/Anthy acceptance; it does not claim every engine/platform or full
assistive accessibility. The current workspace passes758 app tests plus two
explicit pinned-backend regressions; other workspace/doctest counts are unchanged.

## Completed representative source checkpoint

[The twelve-fixture Swift source corpus](rust-integrated-source-corpus.md) now feeds
both native syntax checks and96 actual-app LF/CRLF, valid/malformed,1×/2× cases.
This exposed and repaired omitted C/C++ native colors through the existing capture
pipeline. Native C/C++ dark/light, wrapping, function folds, exact Copy and cross-file
history pass; source hashes stay unchanged. This is representative source acceptance;
other behavior fixtures, larger pending-owner stress and PR/compiler integration
remain in the next gate. Full workspace now passes759 app tests, nine ignored;
other workspace/doctest counts are unchanged.

## Completed representative large-source checkpoint

[Large source publication and input ownership](rust-reader-stress.md) now pass a
5.85 MB Unicode line and60,002-line source, both modal owners and1×/2×. A debugger-
confirmed half-second fragment scan was repaired through the existing Unicode
boundary cursor. Native full copy, find, wrapping, font/reset, Back and replacement
checks pass, with actual worker/CPU/RSS observations and explicit limits. The full
workspace now passes760 app tests and22 text tests; other counts are unchanged.

## Completed integrated PR checkpoint — 2026-11-05

[Integrated app/native PR acceptance](rust-integrated-pr-flow.md) now combines
added/deleted/renamed/binary sources, moving head/base, exact pinned real-Clang
navigation, project-cache reuse, managed-tree and normalized-index eviction,
tool-free source reopening and explicit semantic restoration at1×/2×. The app
regression additionally verifies mapped caret retention, failed-open history and
live snapshot leases. Dirty attached source/HEAD/refs/status remain unchanged.
GitHub metadata is controlled JSON; Git and Clang are real. Final native runs respect
the separate three-record retention policies and verify unavailable semantics after
eviction rather than assuming an old record survives. Full workspace: 761 app tests,
nine ignored; other counts unchanged. No production runtime change in this checkpoint.

## C/C++ dependency checkpoint — 2026-11-05

[Real tracked-vendor/response-input acceptance](rust-clang-dependency-acceptance.md)
now covers 33 translation units/59 documents and fixed relative forced-header and
macro-file include search. Complete generation, dependency-only invalidation,
Partial missing generated-header behavior, navigation, normalized reload and reuse
pass. This is a bounded corpus alongside the real seL4 matrix, not the Swift
11,513-unit tkfiasco corpus or Android/Gradle acceptance. The app suite still passes
761 tests, now with ten opt-in ignored; the three real Clang tests pass explicitly.

## Real Gradle and AGP failure checkpoint — 2026-11-05

[Real multi-module Gradle/AGP acceptance](rust-gradle-android-acceptance.md) now proves
complete four-document Kotlin/Java generation, cross-module UTF-16 navigation,
independent compilation failure retaining three compiler documents, and Partial
status/provenance through Update and tool-free cache reload. Real AGP9.4.1 with a
missing SDK produces explicit failure, no index and unchanged accepted cache/inputs.
Successful Android compilation remains unverified without SDK-backed evidence.
The original Kotlin overload/Noise and PR reader fixture checks are accepted in
the following checkpoint, which found a separate remembered-revision gap. The
Gradle/AGP checkpoint itself made no production runtime/UI change.

## Kotlin fixture and remembered side checkpoint — 2026-11-05

[Recorded Kotlin compiler/PR acceptance](rust-kotlin-pr-fixture.md) now covers both
overloads, UTF-16 bytes after emoji, Noise exclusion, exact definition/reference/Back,
Before/After availability, partial caches and unchanged review acceptance in app
frames and native1×/2×. It found and fixed forgotten per-file revision choices through
the existing accepted-navigation transaction. Alias/bookmark/refresh/failure and
scope reset checks pass. Recorded SCIP/controlled tool boundaries are explicit;
this is separate from the preceding fresh real Gradle compilation. Full workspace
passes763 app tests, eleven ignored; other counts unchanged. Further reader-state
and behavior reconciliation remains open, including per-file position restoration.

## Reading-position checkpoint — 2026-11-05

[Per-file reading positions](rust-reading-positions.md) closes the implicit-reopen
reset and wrapped-row gaps through exact document memory and the existing viewport.
Caret/scroll precedence, exact Back, content/scope invalidation and mapped refresh
pass in app frames, including wrapped Unicode across fragment boundaries at1×/2×.
Native1×/2× checks verify caret-column and passive-scroll reopen by exact Copy.
Full workspace769 app tests/11 ignored; strict Clippy/build/resources pass.
Remaining fixture reconciliation and the gates below remain open.

## Unicode quick-file checkpoint — 2026-11-05

[Unicode quick-file search](rust-unicode-file-search.md) fixes lowercase-only matching
through ICU full folding and existing candidate normalization. Sharp-S, final sigma,
ligatures, long-s and decomposed accents preserve original paths/bytes through raw
app/native1×/2× picker navigation. Ranking/cancellation tests remain green; full
workspace771 app tests/11 ignored, strict Clippy/build/resources pass. Other
behavior/compiler acceptance and final standalone/reuse gates remain open.

## Exact-byte change-block checkpoint — 2026-11-05

[Byte-aware change navigation](rust-byte-change-navigation.md) fixes empty-side
point/EOF availability, preserving inside/gap bounds and exact Back. The app,
toolbar and source header share byte targets. Unit/app checks cover synthetic EOF;
native1×/2× real Git checks cover After deletion and Before insertion points.
Full workspace774 app tests/11 ignored,60 doctests, strict Clippy/build/resources
pass. Smaller-window Before footer wrapping is repaired in the following checkpoint.

## Source status layout checkpoint — 2026-11-05

[Compact source status metadata](rust-source-status-layout.md) fixes Before's split
Read Only label through existing Small typography and metadata spacing. All details
remain available; narrower/diagnostic rows wrap safely and Line remains clickable.
The expanded app matrix and native dark/light1×/2× pass. Full workspace774 app tests,
60 doctests and strict Clippy/build/resources remain green. Other behavior/compiler
and final standalone/reuse gates remain open.

## Reader Return checkpoint — 2026-11-05

[Selection-aware Return](rust-reader-return-selection.md) fixes over-wide selection
navigation and exact departure semantics through shared non-self destinations.
App/UTF-16/native1×/2× checks pass, with full workspace777 app tests/11 ignored.
[The behavior-suite index](rust-behavior-suite-index.md) now names all66 ordinary
Swift suite files and links their Rust/evidence entry points; assertion-level
reconciliation remains explicit. SDK package installation awaits the user's Google
license authorization. Final standalone/library-host checks remain open.

## Independent build/reuse checkpoint — 2026-11-05

[Independent build/host acceptance](rust-reader-host-audit.md) now verifies a fresh
locked16-job build, isolated export, external full-reader and GUI-free shared-crate
consumers, and source-unavailable runtime resources. Three existing viewport/focus
methods are exposed for hosts. All four Swift occurrence-pin assertions have
explicit Rust evidence. Other assertion/compiler reconciliation remains open;
repeat affected standalone/reuse gates after later changes.

## Remaining acceptance gates

1. **Remaining behavior and compiler acceptance:** finish mapping the other
   checked-in Swift behavior fixtures to current Rust coverage. Mixed language,
   recovered syntax, representative long/wrapped/Unicode source, pending text owners
   and integrated PR/revision/cache workflows are accepted above. Use documented
   Swift limits for dependency-heavy C/C++ and Android/Gradle success or explicit
   failure behavior. Repair any new gaps.
2. **Standalone/reuse acceptance maintenance:** the linked host audit passes a clean
   locked build with half-core Cargo settings, embedded resource independence and
   external host consumption for its audited state. Preserve independent app lifecycle and the local shared text/font crates;
   actual sibling integration remains deferred. Re-run the relevant gates after any
   fixes and audit the complete requirement ledger before claiming parity.

New gaps found during these gates belong in the same scope and must be repaired,
not relabeled as completed. The non-Git database failure found in this pass was
fixed, tested with actual Clang/native controls and pushed as `3add1c3`.
