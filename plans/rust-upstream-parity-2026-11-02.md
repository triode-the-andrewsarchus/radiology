# Upstream parity expansion — 2026-11-02

Reference update: main `6d446b9` → `7edfc79` (one upstream publication commit,
172 feature/resource/test/documentation files plus PLAN/PROGRESS). The Rust branch
was rebased across all 271 commits. Only the initial PLAN/PROGRESS index entries
conflicted; Rust source replayed without conflicts. Rust's index remains primary;
the new Swift progress entries are preserved in PROGRESS under the imported-history
heading below. All upstream source, fixtures, scripts and detailed plans remain.

This expands the October 27 parity ledger. No completion percentage from the older
reference should be treated as covering these additions. Source presence after
rebase does not mean the Rust app implements the new Swift capabilities.

## Required work and reuse boundaries

| Priority | Capability and Swift contract | Current Rust assessment / acceptance work |
| --- | --- | --- |
| 1 | Current-function folding: unmodified source-focused x toggles the innermost named function, no repeats/IME/field interception. Committed jumps expand its body and ancestors, including late syntax; previews and refreshes preserve manual folds. | [Ported](rust-current-function-fold.md): source x, named-body targeting, hidden-parent recovery and committed destination expansion reuse syntax/fold/navigation. Full app/workspace, native 1×/2× and composition/repeat/rehighlight checks pass. [Reference](function-fold-key.md). |
| 1 | Files tracks committed navigation: expand ancestors, highlight and center current file, explicit same-file jump recenters, ordinary caret motion retains manual scroll. Temporarily reveal the active document even when filters/listing exclude it, including Before-only paths. | [Ported](rust-files-navigation.md): accepted/current-file reveal, flat query exceptions, passive scroll retention, selected-row preservation and pinned Before file-manager handoff reuse existing trees, filters, source transactions and verified revision cache. [Reference](files-sidebar-navigation.md). |
| 2 | Configurable Context panes: persistent References, Definition/Outline, Trait Implementations and Bookmarks choices; Show All recovery; enabled panes reclaim space; enabling from View reopens hidden inspector without resetting state. | [Preferences ported](rust-inspector-panes.md): app-wide model, both menus, empty recovery, stable region scrolling and data/focus preservation pass app/native restart checks. Trait preference/shared-slot policy is modeled; its actual contextual renderer remains in the next row. [Reference](inspector-pane-controls.md). |
| 2 | Rust trait implementations: semantic trait identity at caret/enclosing trait/impl, current dense source rows, positive/negative counts, coverage gaps, stable selection and guarded exact navigation. | [Ported](rust-trait-inspector.md): publication, exact context, compact rows, ordinary keys, preferences and source-verified navigation reuse accepted indexes/syntax/source transactions. App/workspace/strict and native previous/current1×/2× checks pass, including pinned After, stale/open-file edits and exact Back/Copy. [Reference](rust-trait-implementations.md). |
| 3 | Code Paths: atomic UTF-8 import ≤2 MB, title then escaped file:function:description; canonical-repository persistence shared with attached PRs, copy/delete/trace picker, numbered steps, error recovery. Dedicated left tab beside Files/Search/Changes and import/show shortcuts. | [Foundation implemented](rust-code-paths.md): current escaped text format, stable IDs, canonical app-owned storage and corruption protection pass the91-step import fixture. [Declaration/conditional/assembly resolution](rust-code-path-resolution.md) now passes91/91 real seL4 steps; [guarded app navigation](rust-code-path-app.md) passes91 steps/93 destinations and exact Back. [Standalone sidebar/import/persistence](rust-code-path-sidebar.md) now passes app/UI and native1×/2× dark/light/relaunch/export checks. Desktop-portal and actual IME-engine acceptance remain open. [Reference](code-paths.md). |
| 3 | Code Path resolution: actual current source/syntax, After during review, exact departure history, qualified declarations and overload headers; conditional alternatives; assembly labels/macros; body recovery for configuration-dependent C/C++. | [Source resolution implemented](rust-code-path-resolution.md): shared native C/C++ structure, qualified/header matching, conditional/assembly variants, partial-body recovery, pinned After and authoritative-buffer guards pass fixtures and the real91-step corpus. [App navigation/caret/trace retirement and exact history](rust-code-path-app.md) pass queued-completion, real keyboard-batch and91-step app-model probes. [Conditional menus and sidebar integration](rust-code-path-sidebar.md) now pass native1×/2× checks; environment-specific chooser/IME and broader PR acceptance remain open. |
| 4 | C/C++ syntax/compiler expansion: declaration recovery, source classification, real scip-clang fixture and no-database best-effort GNU C/C++ defaults with inferred includes and visible limitations. | [Fallback ported](rust-clang-fallback.md): private GNU defaults and inferred includes, extension mapping, real-database priority, pinned PR isolation and fallback/configured cache transitions pass unit/app/native1×/2× and actual scip-clang checks. Partial status survives complete coverage and warm cache. Declaration recovery/native grammars are accepted through Code Paths. Managed portable tool installation remains. [References](c-cpp-support.md), [fallback](clang-best-effort.md). |
| 4 | Indexing personalities: Default/seL4 per attached repository, editable validated TOML, settings reveal, selection-triggered generation, personality-aware cache/provenance, invalid-settings recovery and cancellation. seL4 runs separate configurations then unions SCIP, preserving conditional definitions. | No personality/seL4 Rust path found. Reuse indexing service, app-owned tools and isolated generation. Share packaged upstream Python/resources where compatible; adapt macOS-specific paths/tool assumptions. Preserve source/caret/history on switch. Validate the 67-profile recipe, Python provisioning, compiler jobs and union integrity against real fixtures. [References](indexing-personalities.md), [pipeline](sel4-indexing.md). |
| 5 | Updated keyboard help/tab focus: document every app shortcut, import editor Return behavior, list activation/selection, full/narrow tab titles, dedicated Help reference. | Existing Rust command routing/help and list focus are partial reuse. Reconcile against Sources/Radiology/Help/KeyboardShortcuts.md with Linux primary modifiers and explicit source focus/IME ownership. Full accessibility remains outside current scope. |

## Execution and validation

Exact-search acceptance, current-function folding and Files navigation/reveal are
complete for the documented contracts. Inspector pane preferences are also accepted;
trait results and guarded navigation, Code Paths and C/C++ fallback are accepted too.
Next implement portable Clang installation and indexing personalities/seL4,
then shortcut help and the remaining acceptance above in coherent commits, pushing each. Establish architecture baselines
and search reuse before new mechanisms; compare afterward. Keep standalone builds
and shared reader APIs suitable for later little-screen-clip adoption; no sibling
integration is authorized by this update. Use new Swift tests as behavioral oracles,
then test actual Rust frames and native keyboard/pointer flows at both scales.
Real compiler/seL4 and native IME acceptance remain separate from unit-suite success.
The older language/indexer/UI corpus audit is still required alongside this delta.

## Temporary QA cleanup

Removed 2,418 inactive owned /tmp/radiology-* entries, retaining the temporary Git
checkout. Discarded obsolete binaries/build outputs; compressed logs, scripts,
fixtures and captures to ~/.local/state/radiology-qa/tmp-cleanup-20261102/evidence.tar.gz
with manifest.json. No running process referenced those paths. /tmp free space
increased from 5.3 GiB to 17 GiB. Older temporary evidence paths in historical docs
can now be recovered from that archive; cached current search work is unchanged.
