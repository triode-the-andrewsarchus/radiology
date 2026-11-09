# Prepared SDK-backed Android acceptance

The remaining Android success gate now has an explicit ignored test in
[real_tools.rs](../src/semantic/indexing/service/tests/real_tools.rs):
real_android_debug_and_unit_test_sources_keep_utf16_navigation_and_cached_coverage.
It is compiled by the ordinary workspace gate but **has not been executed**.
No successful Android compilation is claimed by this checkpoint.

## Required output

The fixture is a real AGP 9.4.1 Android library with compileSdk 36, Build Tools
36.0.0 and main/debug/unit-test Kotlin sources. Each source uses android.content.Context;
the main declaration reads Context.packageName. The unit-test source calls a
debug-only declaration after an emoji, which in turn calls the main declaration.
No JUnit runner or emulator is needed: this tests compilation/indexing rather than
execution of an Android application. Merely compiling main while omitting debug or
unit-test shards cannot satisfy its three-document/exact-navigation assertions.

The production generation service performs the isolated build and normalization.
The existing verify_lifecycle checks exact UTF-16 original-byte call mapping,
cross-file definition and source references, complete coverage without partial/
stale/cache-warning state, Update reuse, tool-free normalized reload, unchanged
viewed bytes/inventory and removal of private build/staging directories. The new
test additionally verifies the debug-to-main destination and exact declaration
range. Existing recorded PR/native contracts remain separate evidence.

SDK preflight requires an explicitly supplied RADIOLOGY_REAL_ANDROID_SDK path,
platforms/android-36/android.jar and build-tools/36.0.0/aapt2. The fixture writes
that path into its private local.properties and disables automatic SDK downloading
with android.builder.sdkDownload=false. No test code accepts SDK terms, installs
packages or changes system Android settings. The explicit test fails without its
prerequisites; it never silently returns success for missing SDK input.

## Invocation after authorization/provisioning

Use the existing private full JDK and managed tool installation from
[real Gradle acceptance](rust-gradle-android-acceptance.md). Set JAVA_HOME/PATH,
GRADLE_USER_HOME and JAVA_TOOL_OPTIONS only for the QA process. Keep Cargo TMPDIR
at /home/vega/.cache/radiology-qa-rustdoc-tmp and cap Java CPU discovery at sixteen.
Supply RADIOLOGY_REAL_KOTLIN_TOOLS and the already licensed/provisioned
RADIOLOGY_REAL_ANDROID_SDK, then run:

```sh
cargo test --locked --offline --lib \
  real_android_debug_and_unit_test_sources_keep_utf16_navigation_and_cached_coverage \
  -- --ignored --nocapture
```

Cargo offline mode concerns Rust dependencies; Gradle may resolve its compiler/
Android dependencies over the network. SDK package download/license acceptance
remains separately pending explicit authorization. The concrete proposal is
command-line tools, Android 36 platform and Build Tools 36.0.0 (about 296 MiB),
kept under private QA storage. Approval is the remaining prerequisite, not a reason
to infer acceptance from an unattended command or elapsed time.

A passing small fixture will prove these variant hooks and lifecycle invariants;
it will not establish every dependency-heavy Android application. The original
KotlinRealDataProbe actualPR758IndexesThroughTheApplicationService names the
pinned 5d2ffdd2976c62a220118e30780edfe8fb6f2120 ContactsModule.kt → SecrecyCtx.kt
case. Its successful fresh compilation must remain distinct from the already
accepted recorded compiler navigation; retain any unavailable-corpus limit rather
than claiming this small fixture ran that application.

## Validation and architecture

Both Kotlin real-tool cases share managed_kotlin_request, which supplies the
explicit managed root, existing ToolDirectory environment capture and ten-minute
timeout. They already share verify_lifecycle; no second cache, navigation,
isolation or publication implementation is added. The first draft duplicated seven
setup lines. Consolidating the actual repeated setup removes that clone rather
than hiding it by moving fixture text or weakening assertions.

Strict all-target locked/offline Clippy and formatting pass. The real existing
multi-module JVM/partial/AGP-missing-SDK check passes after the shared setup change
(24.22 seconds). Installed Kotlin tool hashes match the prior real acceptance;
the full private JDK compiler and managed Gradle remain present. The Android SDK
success test remains opt-in/unexecuted, increasing ordinary ignored tests by one.
The final workspace result is recorded in PROGRESS.md.

Architecture NEUTRAL: +73 measured NLOC/two analyzed functions; new Android test
CCN 2, shared setup CCN 1; average/max 3.43/132, 36 existing high-complexity functions
and one existing cycle. Complete whitespace-normalized clone comparison retains
all 125 instances with zero additions/removals. The raw equal-score untouched
Python chokepoint ranking swap is unrelated; no Python or Swift source changed.
No runtime code, dependency, public API, preferences or UI type style changes.
Requested SwiftUI/macOS/type skills remain unavailable locally; this is a Rust
test-only checkpoint using the existing compiler scripts unchanged.

Archive: ~/.local/state/radiology-qa/android-success-harness-20261109-ab4fd59/.
It retains ordinary/real/strict logs, preflight identities, before/after architecture
reports, comparison and normalized clone counts. No SDK downloads or license
acceptance occurred in this checkpoint.
