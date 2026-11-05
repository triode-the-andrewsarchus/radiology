# Real Gradle modules and Android failure acceptance — 2026-11-05

## Scope

The new opt-in `real_gradle_modules_keep_partial_shards_and_report_missing_android_sdk`
uses the production generation service, isolated workspace, embedded Swift compiler
scripts, real Gradle 9.7.1, Kotlin 2.4.20, the pinned scip-java compiler plugins and a
private full JDK 17.0.21+9. Google AGP 9.4.1 is resolved for the Android phase. Tool
provisioning also uses the production ToolProvisioner; this is not simulated Gradle
or fixture SCIP output. No production runtime or UI code changes in this checkpoint.

The system Java 17 directory contains only runtime tools, so a private Temurin JDK
is downloaded and verified against its release API digest before extraction:
`8b9b611d917e3dfd8419d3566b5949226f69b052fed7e109c1368add1dfe8711`.
The existing production installer verifies the Gradle download and builds pinned
scip-java `0.13.1-radiology-9f288dd339c7` from commit
`9f288dd339c7e9fc257a7b21714d9a402fd4d837`. JAVA_HOME, Gradle user home and temporary
paths are child-only QA settings. JVM CPU discovery is capped at16; Cargo jobs stay16.
System Java and Android settings are not changed. The verified JDK download archive
is removed after extraction; private installed tools remain available for follow-up.

## Complete and partial module builds

The private three-module build contains an app depending on a Kotlin/Java core,
plus an independent Kotlin module. The complete build emits four source documents,
zero missing paths and no partial-build/cache-warning/stale state. An app call after
an emoji resolves across module boundaries to the exact UTF-16 core definition;
the core's Kotlin call resolves to the Java source declaration.

Changing only the independent module to refer to an undefined symbol makes its
compilation fail. Gradle's real --continue workflow preserves compiler shards from
the successful app/core modules: three documents remain, one source is uncovered,
and `partial_build` is true. Exact definition/reference/byte checks remain usable.
Update reuses the project record, and tool-free normalized reload retains both
Partial status and the compiler-failure flag. Each phase checks exact viewed input
bytes/inventory and absence of build/plugin output in the viewed tree.

The existing real-tool lifecycle helper now accepts an expected partial-build flag
and compares status/provenance through generation, reuse and cache reload. All
other real-tool fixtures keep their existing expectations. This avoids a separate
Kotlin-only implementation of cache, navigation and immutability assertions.

## Actual Android SDK failure

The independent module is then converted to a real AGP 9.4.1 Android library, with
compileSdk36 and a deliberately nonexistent SDK path in local.properties. No SDK
or SDK licenses are installed/accepted by this check. AGP evaluates the real build
and fails with `SDK location not found`, including the warning that sdk.dir does
not exist. With no compiler shards, aggregation fails and generation publishes no
index. The previous accepted normalized cache remains byte-for-byte intact.
All viewed inputs match the intentionally edited fixture, and no build/plugin
artifacts appear there.

This is verified Android failure behavior, not successful Android compilation.
There is no real Android SDK on this host in this checkpoint. Successful debug/
unit-test variant hooks, Android symbol navigation, a dependency-heavy application
such as tkos and other SDK/AGP/compiler versions remain unverified. Shared scripts
and real AGP configuration are stronger evidence than the earlier controlled
SDK-environment test, but do not substitute for that missing success evidence.

## Swift behavior reconciliation

| Swift contract | Current evidence / remaining limit |
| --- | --- |
| KotlinIndexingTests cacheAndReuseRetainUTF16ColumnsAndPartialCompilerCoverage | New real multi-module complete/partial compiler workflow verifies generation, cross-module UTF-16 definition/reference coordinates, Update and normalized reload with truthful partial-build provenance. |
| KotlinIndexingTests isolatedInputsPreserveBytesModesAndNeverWriteThroughSymlinks; directoryLinksAreRebasedAndTrackedBuildInputsSurviveBrowserFiltering | Existing Rust workspace/input inventory regressions remain authoritative for modes/links/tracked dependencies. This real run additionally proves exact bytes/inventory and absence of viewed Gradle output. |
| KotlinIndexingTests realCompilerIndexPreservesOverloadsAndUTF16AfterEmoji | New compiler run verifies Unicode and cross-language identities, but not the original overload/Noise fixture assertions. Reconcile that checked-in SCIP fixture next. |
| PRKotlinNavigationTests changedBlockCaretReferencesAndDefinitionBackRemainInAfter; beforeClearsSemanticsAndAfterRemapsUsingCachedUTF16Contract | Generic Rust UTF-16/pinned navigation and real Clang PR acceptance exist; a direct integrated Rust app run with this Kotlin compiler fixture is still required for the remaining behavior audit. |

Run after provisioning the managed Kotlin root, with full JDK17 on JAVA_HOME/PATH
and a private GRADLE_USER_HOME/TMPDIR:

```sh
RADIOLOGY_REAL_KOTLIN_TOOLS=/path/to/private/managed/tools \
cargo test --locked --lib real_gradle_modules_keep_partial_shards_and_report_missing_android_sdk -- --ignored --nocapture
```

Evidence: `~/.local/state/radiology-qa/gradle-android-20261105-877e26d` contains release
metadata, installer/probe logs, the actual compiler reports, verification commands,
tool identities, architecture baselines and full check logs. Large downloaded JDK,
Gradle and tool caches are not copied into the durable evidence archive. Private
QA setup remains under `~/.cache/radiology-qa-android-gradle`.

## Verification and architecture

Full locked workspace passes761 app tests (eleven opt-in ignored), two binary,
13 font,22 text tests and59 doctests. Strict Clippy, locked build/resource smoke,
formatting and diff checks pass. Final explicit Gradle and Clang checks enforce
exact compiler-failure flags, including the existing Clang missing-header case.
No native UI run is needed for this test-only change; no UI appearance claim is made.

Against `before-android-gradle-acceptance` at877e26d: +139 NLOC/one test, new testCCN7,
shared helperCCN8→9, average/max unchanged at3.39/132. All125 normalized clone instances
and1335 duplicated lines, one existing cycle, eight structural findings and36
hotspots remain unchanged. The complete clone Counter includes multiplicity. No
new runtime mechanism, dependency, helper or architecture hotspot is introduced.

The raw architecture comparison can rank an untouched Swift Python chokepoint as
new/resolved; no reference-script code changed in this checkpoint. Full clone and
structural counts remain unchanged despite that ranking noise.
