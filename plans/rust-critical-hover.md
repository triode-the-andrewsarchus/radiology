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

## Reader integration queue

The source popup implementation is in the working tree but still needs native
acceptance. It must keep stable geometry, allow entering/clicking full destination
rows, expose at most eight critical/frontier rows and coverage, dismiss on input
and all configuration transitions, and route definition actions independently of
the caret through the existing exact indexed navigation/history path. A headless
pointer replay currently verifies real title-row hit regions and source/caret/
selection preservation. Native cross-file navigation, Back, external/plumbing
behavior and mode dismissal remain before accepting the UI.

Full parity also requires progressive count publication and the broader index,
PR, inspector, source and final UI acceptance queue in rust-parity-implementation.md.
