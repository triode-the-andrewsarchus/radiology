# Android SDK context for Gradle indexing

## Confirmed Swift/Rust gap

Swift ScipKotlinIndexer.command supplies an existing ~/Library/Android/sdk as
ANDROID_HOME when that variable is absent. The shared compiler wrapper expects
Gradle to resolve the actual Android SDK; it does not discover one itself. Rust
previously added only PATH to generation commands, leaving conventional SDKs
undiscovered when the launching desktop did not provide SDK variables.

The new isolated-generation regression fails before the repair with no SDK
variables configured: the controlled Gradle child exits before its invocation
marker and the existing wrapper records a partial build. The repaired path reaches
Gradle with the exact SDK context and completes generation.

## Child-only lookup and explicit settings

ToolDirectory.indexing_environment reuses child_environment for PATH and adds a
fallback only to the Kotlin/Java Gradle strategy. When both ANDROID_HOME and
ANDROID_SDK_ROOT are absent, it checks existing directories under the context's
home in this order: Android/Sdk, then the Swift Library/Android/sdk convention.
Missing homes and files pretending to be SDK directories produce no override.
Paths remain OsStrings and are passed through the existing process environment,
including spaces and shell punctuation. No shell source is built from SDK paths.

If either SDK variable exists, including an empty or invalid value, the lookup
adds no SDK override. The subprocess inherits explicit values as before. It does
not reconcile conflicting user settings: [Android's environment contract](https://developer.android.com/tools/variables)
identifies ANDROID_HOME as the SDK location and ANDROID_SDK_ROOT as deprecated,
with consistency checks when both are used. ANDROID_USER_HOME and ANDROID_SDK_HOME preference locations
are not treated as SDK roots. Project local.properties remains untouched.

Generation uses the new method at its existing command-environment boundary.
Version checks, provisioning and non-Gradle adapters retain their original PATH
handling. The compiler scripts are still embedded directly from the Swift source;
no duplicate SDK shell script, worker, dependency or Cargo setting is introduced.
The public method supports standalone/embedding hosts without UI dependencies.
The lookup installs nothing and does not mutate the parent environment or viewed
repository. Ordinary Gradle dependency/build behavior is unchanged.

## Verification and limits

One unit matrix covers missing home/directories, a non-directory candidate, the
Swift fallback, Linux-first precedence and unchanged environments for all five
other adapter families. The service regression runs the real isolated generation
path, shared shell wrapper/plugin extraction and process runner with a controlled
Gradle executable and fixture SCIP output. It checks complete report/coverage,
private working directory, exact original source bytes/inventory, absence of source
build/plugin output, an unchanged SDK sentinel and unchanged parent SDK variables.
The deterministic test ZIP contains fixture payloads, not compiler plugins.

Separate child-process runs pass with both variables absent, explicit HOME, legacy
ROOT only, conflicting HOME/ROOT, empty HOME and empty ROOT. No test changes process-
global environment state. This is environment handoff evidence, not a real Android
SDK, compiler, AGP, dependency or license-acceptance test. Real Android/AGP corpus
acceptance remains open; no Android SDK was found at the checked local locations.

Workspace: 623 app tests pass, seven ignored; two binary, 13 font, 19 text and
40 doctests pass. Final targeted source/SDK immutability checks and strict Clippy
pass after strengthening the fixture. Strict workspace check/build/rustdoc/fmt/diff
and copied standalone resource checks pass. The rendered public method docs are
inspected. No new native visual run is needed for this backend-only change.

Architecture against before-android-sdk-context: +146 NLOC/four functions including
two tests, no new hotspot/cycle, health unchanged at displayed precision. The default
comparison flags one old service-test clone as REGRESSION; a complete normalized
fragment/file-pair audit matches all 109 groups, zero new or removed. Production
CCN is 5 for generation overrides and 2 for directory lookup.

Reference page was fetched through Pulpie on 2026-11-02; the SDK contract summary and tool
stats are retained with the evidence. Archive:
~/.local/state/radiology-qa/android-sdk-20261102-08bd674.

## Remaining parity work

SDK discovery/handoff is implemented. Real Android/AGP builds, broader C/C++ build
databases/dependencies, Gradle/compiler corpora and final Swift feature/UI coverage
remain open in the master audit. SDK/dependency-version-aware cache behavior is not
newly certified by this change. Full accessibility and actual sibling adoption
remain deferred by user scope.

## Actual Gradle/AGP follow-up — 2026-11-05

[Real Gradle module and Android failure acceptance](rust-gradle-android-acceptance.md)
now supplements the controlled environment test above. Actual Kotlin2.4.20/Gradle9.7.1
compilation proves complete/partial multi-module data, cross-language navigation and
cache provenance. Real AGP9.4.1 reaches an explicit missing-SDK error, publishes no
index and preserves the accepted cache. Successful Android variant compilation and
SDK-backed navigation remain unverified; no SDK installation or new support-version
claim follows from this failure check.
