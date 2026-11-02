# Source structure quality in the reader footer

## Contract — 2026-11-01

Swift's `ReaderDetail` distinguishes failed structure analysis from a recovered
syntax tree containing errors. The Rust reader now makes the same distinction:

- `Structure unavailable` reports the parser/outline-query diagnostic on hover.
- `Partial structure` explains that the linked parser did not fully parse this
  source, so highlighting, outline and other structural results may be incomplete.
- Complete or unsupported/plain source does not receive a partial warning.

A fatal structural diagnostic takes precedence over recovered-tree quality.
Recovered declarations, calls, colors and source remain available. A parse warning
is not a load failure and does not turn an otherwise readable file into an error.

The implementation reuses `CallSites::parse_errors`, prepared by the existing
shared parse and published atomically with the displayed source. That fact is now
retained even when the call query returns no usable results; query support/error
and source identity retain their existing contracts. No parallel parser, health
state, dependency or worker is added. `SourceViewContent` adds a borrowed
`structure_error` diagnostic and a `partial_structure` flag so embedding hosts can
supply the same exact-source facts. The app supplies its existing syntax error and
recovered-tree flag carried by call analysis. Existing footer rendering, type roles and hover diagnostics
are reused; no new type scale or controls are introduced.

Recoloring and failed navigation retain the warning for the retained source.
Opening another file, reloading repaired bytes, Back navigation and repository
reset use the existing source publication/retirement boundary. The warning does
not rely on semantic index availability or Critical Lens being enabled.

## Verification

The old reader fails the real-app regression with a missing Partial structure
label. The final case opens all ten malformed language/dialect inputs represented
by Swift's `SyntaxWorkerTests`: Swift, Rust, Go, Python, JS/JSX, TS/TSX and Kotlin/
Kotlin script. It checks the recovered-error fact, intact source, visible warning,
recoverable versus failed analysis and a surviving valid Rust declaration.
Recoloring preserves the exact call-fact snapshot; an unsuccessful open retains
it. A fatal diagnostic suppresses the partial label; repaired reload and repository
reset clear stale quality state. A native-query interruption case independently
checks that the recovered-error fact survives unavailable call results without
publishing calls/scopes or claiming supported/exact call coverage.

Initial compilation missed two test imports; a repeated retained-warning assertion
was consolidated into one local assertion closure. These setup/duplication trials
are excluded from accepted checks. The initial native controller used Alt+Left,
which is not this application's Back shortcut; final checks use Ctrl+[.

The final workspace passes 572 app tests (seven ignored), two binary tests, four
font tests, 13 text tests and 37 doctests. Strict Clippy, all-target check, binary
build, rustdoc with warnings denied, formatting and diff checks pass.

Two final native 1×/2× flows on the same binary verify visible partial status,
recoloring, inspector hide/restore, exact malformed UTF-8 source copies, a valid
file transition, Back restoration, controlled repair/reload and exact repaired
source copies. The warning clears after repair and recovered declarations remain
visible before it. Dark/light captures were inspected. The private `long.rs`
fixture is deliberately repaired; its before/after SHA-256 values are recorded,
other fixture bytes remain unchanged, the binary hash remains unchanged and all
children exit zero. Fatal-status priority is covered by the real-app regression;
the native flows exercise recovered syntax and repair, not native query failure.

Architecture baseline: `before-structure-quality-status` at `9f3a8f5`. Product and
tests add 103 NLOC and two test functions, with no new code files, hotspot, maximum
CCN or cycles. Source-view rendering CCN is 9; health is effectively unchanged.
All 91 clone groups are identical after full normalized multiset comparison. The
automated regression label only flags an existing eight-line modal/source-view
test clone touching a modified file; no clone was added or removed.

Archive: `~/.local/state/radiology-qa/structure-quality-20261101-9f3a8f5`.
`accepted.json` selects `native-v2-1.0-loaded` and `native-v2-2.0-loaded`. It records
the intentional repaired-fixture hash; exact source clipboard files preserve both
versions. Controllers, captures/action logs, hashes, workspace/gate logs, negative
regression, architecture reports/full clone audit and source diff are retained.

## Remaining scope

This exposes recovered syntax quality; it does not fix grammar acceptance itself.
The linked compact Kotlin interface limitation therefore remains open and can now
be explained as partial structure. Broader corpus/grammar and full reader/index/
PR/cache/UI acceptance remain in the implementation ledger. Requested design
skills remain unavailable locally; existing typography and status roles were used.
Full accessibility and actual sibling adoption remain deferred.
