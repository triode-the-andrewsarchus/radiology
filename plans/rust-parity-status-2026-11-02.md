# Rust/Swift parity reconciliation — 2026-11-02

Rechecked against Rust `3add1c3` and the imported Swift main reference `7edfc79`.
The [initial review](rust-swift-parity-review-2026-10-27.md) describes `fd421e1`;
its Missing/Broken entries are historical findings. This ledger maps those findings
to current implementation and test evidence, and keeps the outstanding acceptance
work explicit. Overall parity is still unproven.

The reference is the checked-in Swift code, queries, fixtures and behavioral tests.
AppKit/SwiftUI are not executed on this Linux host. Broad UI acceptance follows the
user's standalone Linux/reusable-reader direction, including intentional Linux
modifiers and compositor-owned window commands. Full assistive accessibility and
actual little-screen-clip adoption remain deferred.

## Original finding reconciliation

The current full workspace passes 759 app tests (nine opt-in ignored), two binary,
13 font, 21 text and 59 doctests. The examples below are bounded evidence for the
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

## Remaining acceptance gates

1. **Integrated corpus and stress:** reconcile the checked-in Swift behavior fixtures
   with current Rust coverage; exercise mixed source languages, malformed/recovered
   syntax, long/wrapped/Unicode documents and cross-owner input while jobs are pending.
   Record source/caret/history invariants, stale-result retirement, frame/worker/RSS
   observations and any reproducible failures. Include current PR added/deleted/
   renamed/binary files, moving head/base, pinned semantics, cache reuse/eviction and
   reopen flows in one integrated app run. Use documented Swift limits when checking
   dependency-heavy C/C++ and Android/Gradle success or explicit failure behavior.
2. **Final standalone/reuse audit:** verify a clean locked build with half-core Cargo
   settings, packaged resource independence and library/host usage after the above
   changes. Preserve independent app lifecycle and the local shared text/font crates;
   actual sibling integration remains deferred. Re-run the relevant gates after any
   fixes and audit the complete requirement ledger before claiming parity.

New gaps found during these gates belong in the same scope and must be repaired,
not relabeled as completed. The non-Git database failure found in this pass was
fixed, tested with actual Clang/native controls and pushed as `3add1c3`.
