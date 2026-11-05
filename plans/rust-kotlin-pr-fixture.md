# Recorded Kotlin compiler fixture and PR reader acceptance — 2026-11-05

## Review-side memory repair

After choosing Before for Reader.kt, opening Store.kt and returning to Reader.kt
now keeps Before. Previously the implicit open always selected the default After.
The new app assertion fails with the original behavior and passes after repair.

One review-owned map stores each changed file's canonical After path and selected
side, so renamed Before aliases share the same choice. Implicit opens consult it;
explicit history/index locations retain their requested revision. A flag on the
existing NavigationIntent commits a choice only at accepted destination publication,
including its existing-document fast path. Revision controls, bookmarks, semantic
jumps and refresh restoration use that path; failed reads cannot modify choices.
History replay itself does not overwrite an independent stored file preference.

A refreshed snapshot keeps available choices and rekeys recognized aliases, dropping
obsolete/unavailable sides. A different PR or successful repository transition clears
the map. The immutable ReviewSnapshot stays immutable and no new worker/cache/API
is added. Choices are ephemeral for the current review, matching Swift's in-memory
scope; they are not persisted across relaunch.

A focused app regression covers rename aliases, bookmark activation, unrelated added
files, a binary failure retaining the reader and map, refresh dropping an unavailable
After choice, another PR and leaving review mode. The Kotlin workflow also sets a
stored Before preference while displaying After, as Swift's test does: F12/reference
commits reach After despite the destination's prior choice, and Back preserves the
explicit After departure while retaining the source file's separate preference.

## Reference and boundary

The checked-in Swift compiler fixture under `Tests/RadiologyTests/Fixtures/scip-kotlin`
is actual Kotlin2.4.20/scip-java output with documented plugin commit and Gradle/JDK
provenance. This checkpoint consumes its unchanged source files and `index.scip`.
It does not recompile them or simulate their overload identities in a fabricated
protobuf. Gradle and aggregate executables are controlled boundaries, as in the
Swift fixture tests. The preceding [real Gradle acceptance](rust-gradle-android-acceptance.md)
remains separate evidence for fresh compilation and partial module builds.

The new ordinary app regression builds a private real Git review with read(1) in
Before and the indexed read(2) in After. Its attached checkout contains deliberately
different Reader.kt bytes. Its base Store body is also varied so a real changed
Store destination can remember Before before an indexed After jump. It invokes the normal app generation command, personality
capture, Kotlin wrapper, source validation, normalization and publication; it never
assigns a SemanticIndex directly to the app. Existing fixture ZIP payloads exercise
plugin extraction without claiming they are compiler plugins.

Four actual-app cases cover1×/2× with either all three fixture source files or only
Reader/Store. The omitted-Noise case has a controlled Gradle failure and uses light
appearance; aggregation retains the recorded compiler data, normalization filters
Noise outside the input inventory and the report remains Partial. The complete
three-file case uses dark appearance. These are four defined cases, not a full
Cartesian product of all appearance/input/failure combinations.

## Swift assertion mapping

| Swift test | Rust app evidence |
| --- | --- |
| KotlinIndexingTests.realCompilerIndexPreservesOverloadsAndUTF16AfterEmoji | Every ranged Reader occurrence maps against UTF-16 columns; exact read-token bytes after café/emoji map correctly. Integer and string calls retain different symbol identities and reach Store.kt line4/line5 at the exact definition byte. Each query has one reference/one definition, zero unmapped/unreadable sites and no Noise hits. |
| KotlinIndexingTests.cacheAndReuseRetainUTF16ColumnsAndPartialCompilerCoverage | Actual app generation publishes three or two navigable documents as appropriate; excluded compiler-only Noise is absent. Tool-free normalized reload and exact project reuse preserve availability and the partial-build flag. An aggregate-call marker remains at one invocation across generation/reload/Update. |
| PRKotlinNavigationTests.changedBlockCaretReferencesAndDefinitionBackRemainInAfter | The changed source line has a real pinned overlay. A raw source Right key moves within the call without adding a Back step; under-caret facts retain exact occurrence bytes. Documented Ctrl+Alt+R, F12, reference-row activation and Back preserve the exact departure byte and After scope for both overloads. |
| PRKotlinNavigationTests.beforeClearsSemanticsAndAfterRemapsUsingCachedUTF16Contract | Before reads read(1), clears under-caret/query inputs, disables semantic commands and visibly explains Before not indexed while parsed changed/untouched declarations remain. Explicit caret placement after returning to After restores the integer identity, including after cache reload. A newly prepared unchanged Git snapshot preserves caret/history/identity on acceptance. |

The unchanged-snapshot check exercises the normal publication path with real Git;
it does not claim a fresh GitHub request. Native PR metadata is controlled JSON.
The remembered-choice state now supplies the same explicit Before-preference
assertion as Swift. Indexed destinations update their choice to After only when
the source is accepted; history locations remain explicit even when another choice
is stored for the source path.

## Standalone native observations

Private Weston/Niri sessions run the standalone binary at1×/2× against the same
three-file recorded-index review. Real keyboard Go to Line, Ctrl+Alt+R and F12
reach the two separate Store declarations; inspected captures show lines4 and5
at both scales, each with one reference. Ctrl+[ returns to Reader. Before retains
its own exact source after an attempted F12, and After restores the pinned text.
The source-only explanation and parsed outline remain visible in Before.

Reopening Reader from Changes after leaving its Before view for Store retains Before.
Warm restart loads the normalized index without any new version/aggregate call,
and definition navigation still reaches Store. Twelve exact full-source clipboard
checks per scale cover Reader, Store, both Back returns, Before, After, remembered-side reopening and restart.
The attached source hashes, Git HEAD/refs/status and executable hash remain unchanged
through both native runs. App/restarted app/Weston/Niri exit zero. These native runs
use controlled compiler/metadata boundaries and do not claim a fresh real compiler
or live remote API. The UTF-8 footer describes source file bytes, independently of
the compiler's UTF-16 occurrence columns.

## Test corrections and scope

Initial harness failures were test API/expectation errors: the app has a shared
SemanticAvailability resolver rather than a convenience method, history.back()
returns an owned location, committed navigation already retains a departure entry,
and Find References is Ctrl+Alt+R rather than Shift+F12. The final test compares the
previous Back destination across passive input and uses the documented command.
As in Swift's test, it explicitly positions the caret after revision switches;
changed-line refresh is allowed to discard a column. The later remembered-side
assertion exposed the runtime gap described above; its red/green evidence is retained.

This checkpoint closes the named Kotlin overload/Noise and PR fixture reconciliation.
It does not establish SDK-backed Android success, every Kotlin/compiler variant,
all remaining Swift behavior fixtures or overall feature/UI parity. Full assistive
accessibility and actual little-screen-clip adoption remain deferred by user scope.
No production UI/type/style changes are made; requested UI skills remain unavailable.

## Architecture and evidence

Against `before-kotlin-pr-fixture` at5b3cdb8: +490 NLOC/three functions (two tests and their fixture), with
canonical choice state and publication integrated into existing methods. Fixture/
workflow/scope tests are CCN7/7/1; apply_destination7→10, install_review1→4 and
resolve_location3→4 remain below the hotspot threshold. Average/max remain3.39/132,
with36 pre-existing hotspots. All125 complete normalized file-pair/fragment clone
instances,1335 duplicated lines, one cycle and eight structural findings are
unchanged. The raw REGRESSION reports old self-clones in tests.rs (only the module
declaration was appended there) and an untouched Swift Python chokepoint rank change.
A complete Counter including multiplicity shows no clone addition/removal or replacement.
No new dependency or public API is introduced.

Evidence: `~/.local/state/radiology-qa/kotlin-pr-fixture-20261105-5b3cdb8` retains app
runs (including early diagnostics), native fixture/driver/actions/captures/clipboard,
cache/tool-call reports, hashes, verification logs and architecture baselines.

Final full locked workspace:763 app tests pass (eleven opt-in ignored), two binary,
13 font,22 text tests and59 doctests. Strict Clippy, locked build/resource smoke,
formatting and diff checks pass. The final native mode is `final`; earlier modes
retain the initial fixture-only and intermediate remembered-choice observations.
