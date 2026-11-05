# Function shading and stale-range rejection — 2026-11-05

Update: the [pending-context checkpoint](rust-pending-source-context.md) closes the
context-clearing difference recorded below at4ea88aa, while retaining accepted
source for recovery. The tests/evidence below describe that earlier checkpoint.

Swift `FunctionShadeTests.bandCoversTheFunctionsVisibleLinesAndRespectsFolds`
requires a stale range extending past the source to draw nothing. Rust previously
intersected that range with visible fragments and painted it anyway. A red
rendered-output regression reproduces the mismatch. The existing `function_band`
now rejects empty, out-of-bounds and split-UTF-8 ranges before projecting them.
`str::is_char_boundary` supplies both bound and encoding validation, without a
new range utility or cache. Valid source painting and canonical coordinates stay
unchanged.

## Three Swift assertion groups

| Swift contract | Current Rust evidence |
| --- | --- |
| `innermostCallableNotContainers` | Expanded `callable_wash_tracks_nested_declarations_and_clears_between_methods` renders Rust and Python nested functions at1×/2×. New app test `function_wash_follows_caret_and_clears_with_accepted_documents_and_repositories` uses the Swift Shape fixture, checking square/body/signature/closure/name, gaps and struct fields. Its7/3/1-row bands distinguish the outer, nested and single-line callables. |
| `followsTheCaretNotScrollingAndClearsOnFileAndRepositoryChanges` | The new app test changes caret bytes independently from viewport restoration, verifies full-width painting and then clears shading after Notes/another repository are accepted. Native passive scroll leaves the footer/context at line61 and Copy still returns that exact call line; nested/gap/container/name captures follow subsequent caret moves. |
| `bandCoversTheFunctionsVisibleLinesAndRespectsFolds` | Existing wrapped-Unicode/layer-order paint test and expanded half-open/fold test cover contiguous rows, clipping and collapsed shrinkage. New invalid-range cases preserve source/selection while painting no band. The external consumer supplies invalid and valid function metadata through public SourceView, proving rejection and a positive paint control at both scales. |

Rust retains the prior accepted reader, including its valid shade, while a new
file or repository is being prepared. Swift clears its model's current-function
range as soon as a request starts. This remains an explicit transaction/presentation
difference: Rust publishes source and structure together, and the new test verifies
accepted-buffer retention followed by correct shading/clearing on publication.
It never paints the old range against newly accepted source. The remaining parity
ledger must preserve this distinction rather than claiming identical pending UI.

The app test exercises source transitions and render output; the viewport tests
cover source-bound geometry. No new focus, fold, history or persistence state was
introduced. Caret movement derives the enclosing callable from current parsed
declarations; passive viewport movement cannot replace that byte.

## Verification

Native real-Git PR fixtures contain50 prefix rows and the nested Shape example,
with distinct checkout/After bytes. At1×/2×, area and square shade their own visible
rows; passive scroll moves into the prefix while the caret remains at61; blank and
container positions clear the wash, and a single-line function has one row. Two
exact Copy checks per scale preserve the call line and complete original source.
Source/Git/executable hashes remain unchanged and child exits are zero.

The independent reader host also reruns focus/viewport/caret/Copy/source-replacement
checks and rejects stale/split-byte shade ranges while painting a valid control.
Locked workspace:788 app tests pass (11 opt-in ignored), two binary tests,
13 font tests,22 text tests and60 doctests. Strict lint/doc/format, build and bundled
resource checks pass. AppKit execution remains outside this Linux verification.

Against `before-function-shade-audit`, measured NLOC grows132, including one new
app regression and expanded existing cases. No added clone, hotspot, dependency
cycle or structural issue: full file-pair/fragment counters retain125 instances /
1335 lines, average/max CCN3.40/132. Function-band CCN is8; the new app regression
is5. Raw duplicate warnings name unchanged test sequences. Reported function-count
reductions are a Rust test-parser limitation, not deleted application functionality.
Equal-score chokepoint rankings in untouched Python files also change between runs.

Evidence: `~/.local/state/radiology-qa/function-shade-20261105-2fa67c6/`;
working fixtures/logs: `~/.cache/radiology-qa-shade/`. Other Swift assertion groups and
SDK-backed Android validation remain open in the parity audit. The pending-context
item above is closed by the linked follow-up. Full accessibility and sibling adoption remain
deferred.
