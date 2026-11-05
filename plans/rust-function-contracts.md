# Function-fold keys and Code Path visits — 2026-11-05

Reconciled all four Swift `ReaderFunctionFoldKeyTests` contracts and the single
`SourceFunctionNavigationTests` contract. Existing runtime mechanisms satisfy this
group; this checkpoint strengthens their regression evidence and records the
publication boundary rather than adding another fold/navigation mechanism.

## Assertion evidence

| Swift contract | Rust evidence |
| --- | --- |
| `xTogglesTheInnermostFunctionWithoutChangingSelectionOrSource` | Expanded `current_function_toggle_owns_only_source_presses_and_committed_jumps_expand_bodies` uses nested `café`, an emoji body and an exact selected `é`. Copy preserves that range after X; caret/history remain exact, repeats do not toggle, ancestors open and sibling folds remain closed. Native ordinary-X/Caps-Lock-X captures toggle only the nested body and preserve selection. |
| `hiddenAncestorsOpenAndOutsideFunctionsDoesNothing` | Existing app regression folds the ancestor over an open target, then X opens the ancestor without closing the target. `named_function_body_excludes_nested_closures_and_container_gaps` rejects gaps, half-open declaration ends and bodyless declarations. Native X on the blank line between functions preserves source. |
| `modifiedKeysAndEditableFieldsKeepTheirNativeHandling` | The app matrix now checks Shift/Ctrl/Alt/Command combinations, plus existing composition and editable-search ownership. Native Shift+X retains the fold; Find accepts `x` and returns the exact reader selection on close. Native Caps Lock remains an ordinary fold key. |
| `parsedBodiesMatchAcrossLanguagesAndIgnoreParameterClosures` | The existing innermost-body test now includes the exact Swift/Go/Python/JavaScript/TypeScript/Kotlin examples, checks body/declaration end identity and Kotlin body start line6 after the default lambda. Rust trait methods and empty single-line bodies have no toggleable function fold. |
| `codePathVisitsExpandBodiesWithLateSyntaxAndRespectManualRecollapse` | New `nested_function_visits_expand_only_the_destination_and_manual_recollapse_survives_repaint` checks prepared visits at1×/2×: accepted source/folds persist while work is pending; target and ancestors expand on publication, sibling stays folded; source-font rehighlight preserves manual collapse; a repeated same-file visit reopens only its target without adding history; another-file visit and Back restore the exact nested byte and fold choices. |

Rust publishes a prepared source and its syntax atomically, using the existing
source transaction. It does not expose Swift's intermediate mounted-reader /
late-syntax state. The equivalent cold destination has structure before acceptance;
appearance work only publishes colors for the same exact buffer and cannot replay
default folds over a manual choice. Code Path visits and Back share the same
`apply_destination` body/ancestor reveal path. No new pending-intent cache is needed.

Tests are in `src/ui/app/tests/focus.rs`, `src/ui/app/code_paths/tests.rs` and
`src/syntax/folds.rs`. Existing command filtering, parsed body ownership and
transaction logic were inspected alongside the Swift assertions; the new checks
pass without production changes.

## Native and architecture verification

Private-compositor runs use a real Git PR fixture with nested Unicode functions,
a separate callback and sibling, and distinct checkout/After bytes. At1×/2×,
eight clipboard checks per run verify the selected accent before/after ordinary
and Caps Lock toggles, editable Find text, selection return and complete source.
Captures confirm collapsed/expanded target bodies, modified-key retention and
gap behavior. Source/Git/executable hashes stay unchanged and child exits are zero.
Go to Line is allowed to paint before caret keys are injected. Early harness
timing/socket-path failures remain separate from the accepted `native-*-final`
captures. The fixture is syntax/UI evidence, not a live compiler or GitHub service.

Locked workspace:787 app tests pass (11 opt-in ignored), two binary tests,
13 font tests,22 text tests and60 doctests. Strict Clippy and formatting pass.
Runtime, public APIs, dependencies and bundled resources are unchanged from the
[gutter checkpoint](rust-gutter-parity.md), whose native executable is reused here.

Architecture baseline `before-function-contract-audit`: +108 measured NLOC and one
test function. Average/max CCN remain3.40/132, with no added hotspot, dependency
cycle or structural issue. Full file-pair/fragment counters retain125 clone
instances/1335 lines with zero additions/removals. The raw duplicate warning names
two unchanged repository-search test sequences in the modified focus test file.
The other raw warning swaps equal-score chokepoints in untouched Python scripts.

Evidence: `~/.local/state/radiology-qa/function-contracts-20261105-a2addb8/`;
working fixtures/logs: `~/.cache/radiology-qa-function-contracts/`. Other behavior
suites and SDK-backed Android success remain open. Full accessibility and actual
sibling app adoption remain deferred.
