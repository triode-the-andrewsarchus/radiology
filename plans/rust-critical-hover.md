# Critical Lens source hover in Rust

## Interaction controller (2026-10-30)

`CriticalHover` is independent of EGUI. It consumes an immutable Arc<GraphInputs>
configuration and a monotonic clock, waits 350 ms on an exact SCIP occurrence,
then schedules a snapshot-bound frontier on the existing worker pool. Moving
within one occurrence does not restart dwell. Repository callables and critical
external callables qualify; locals, external plumbing/unclassified symbols,
Before, stale source and mismatched syntax do not. Callability shares the Swift
SCIP descriptor/kind rules through SemanticSymbolID::is_callable.

One popup serial survives loading/result changes. Leaving the anchor starts a
700 ms grace period; popup entry or anchor return cancels closure. Click/key/scroll
interruption cancels work and suppresses reopening at the same idle byte. Source,
index, storage or policy configuration replacement retires requests and current
presentation using LatestRead. No hover method changes source, caret, selection,
folds or history. External critical calls publish an explicit external/no-body
result without a false definition link. There is no idle polling; hosts request
only the next dwell/grace deadline or pending-worker completion poll.

Four controller tests cover deterministic dwell/grace, within-token movement,
background completion identity, pointer interruption, popup entry/return, early
leave, configuration retirement, external/plumbing/local/stale/Before guards.
Current working-tree checks (including pending reader wiring) pass 190 application
tests, 5 reader-text tests, 30 doctests, fmt, Clippy, build and rustdoc. One external
Clang test remains ignored by default. Architecture retains short test/footer
clones with no new product hotspot/cycle; maximum CCN remains 100.

## Reader integration and acceptance (2026-10-30)

The app now retains one immutable Arc<GraphInputs> per source/index/storage/policy
configuration and shares it with the reader hover adapter. Caret changes do not
replace that configuration. The source viewport tracks measured original-byte
pointer hits, exposes no hover in Normal/modal/Before contexts, and owns popup
interaction state. Repository source transitions retire old presentation before
painting a different document. The popup position is frozen per presentation and
its title hit region remains stable when loading completes.

The interactive 380-point popup uses the existing body/meta type scale and colors.
The title's full row navigates to Definition when available; external rows are
honestly unavailable. Loaded content shows compact direct/reachable/completeness
status, coverage detail, at most eight critical/frontier rows, via chains and a
remaining-row count. Destination actions carry exact SemanticSymbolID independently
of the source caret and enter the existing indexed range/source-provenance/history
navigation path. Source/outside clicks, keys, scrolling, mode/modal/source/index/
policy/repository changes dismiss or retire the popup. Pointer entry cancels the
grace timer, allowing real button clicks. Background completion explicitly requests
a repaint even though it keeps the same popup serial; otherwise a loading frame
could remain painted until another pointer event.

A headless widget replay exercises actual measured token hits, dwell, background
completion, Escape dismissal, idle suppression, renewed hover, popup entry and a
click on the trailing part of the whole Definition row. It verifies navigation
intent without modifying source, caret or selection. Controller tests verify that
completion reports a visual change for repaint scheduling. Full checks pass
190 application tests, 5 reader-text tests, 30 doctests, rustfmt, Clippy, binary
build and rustdoc. Architecture comparison reports one additional seven-line
interaction-test clone and existing footer label sequences, with no new product
hotspot or dependency cycle; maximum CCN remains 100. UI/native code performs no
body reads; all graph I/O/parsing stays on the bounded existing worker pool.

Private native Weston/Niri acceptance uses a two-document UTF-16 SCIP fixture with
non-ASCII and emoji before the hovered occurrence. Normal mode stays quiet.
Critical hover on save loads b.rs's body and paints a complete one-operation
summary without further pointer input. Moving into the popup retains it beyond
the grace interval; clicking the far-right part of the title opens b.rs at save.
Back restores a.rs's original caret at café on line 2. External system_open shows
an unavailable definition and external/no-body status; runtime clone remains
quiet after the grace period. Changing to Normal dismisses a visible popup.
Artifacts under `/tmp/radiology-hover-native/` include `repository-hover.png`,
`entered-popup.png`, `hover-definition.png`, `back-restored.png`,
`external-hover.png`, `plumbing-no-popup.png` and `mode-dismissed.png`.
Both app runs and both compositors exit zero. Persistent pointer helper clients
are deliberately terminated between movements and report -15. No sibling source
was changed or runtime dependency introduced. The unavailable design skills were
rechecked; UI follows the local Swift reference and existing Rust type scale.

Remaining acceptance includes broader languages, long names/eight-row overflow,
edge-of-screen placement, scaling/light mode, source changes while a popup is
loading, and finer interruption inside native reads/parses. Annotation publication is now progressive; see rust-call-graph.md. Cache reuse
across independent requests/file revisits remains open.
Full parity also retains the broader index, PR, inspector, source and final UI
acceptance queue in rust-parity-implementation.md.
