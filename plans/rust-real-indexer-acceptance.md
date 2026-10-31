# Real indexer acceptance

## Shared lifecycle coverage — 2026-10-30

Four opt-in tests in `src/semantic/indexing/service/tests/real_tools.rs` pass
against actual tools: rust-analyzer 1.98.1, scip-typescript 0.4.0, scip-go 0.2.7,
and scip-python 0.6.6. Go, TypeScript and Python use the production pinned
installer and verify the resulting managed executable. Rust uses an explicitly
selected real component from a private QA toolchain; this does not exercise
installation into the user's active rustup toolchain.

One shared assertion path checks production generation, complete expected document
coverage, cross-file definition/reference navigation, exact encoding-aware byte
ranges, version-qualified Update reuse, asynchronous cache-only reload with no
tools configured, source digest validation, unchanged viewed files/inventory, and
cleanup of private generation workspaces. The caller fixtures contain emoji before
the call for Go, Python and TypeScript; TypeScript also uses CRLF. Rust exercises a
feature-gated module requiring the nested-workspace feature/cfg parity behavior.
Fixtures have paths containing spaces and a literal semicolon.

Go reports two documents using UTF-8 columns. Python reports two documents using
UTF-16 columns. Both have zero missing sources and no partial/stale/cache-warning
state. Python retains the real tool's nonfatal warning that pyproject.toml lacks a
`[tool.pyright]` section. All viewed input bytes and the discovered inventory stay
unchanged; no Go module artifacts or Python environment/cache directories appear
in the viewed repository. Temporary installed tools are removed with each fixture.

Run explicitly, with network access and language prerequisites available:

```sh
GOMAXPROCS=16 \
RADIOLOGY_REAL_RUST_ANALYZER=/path/to/private/toolchain/bin/rust-analyzer \
cargo test --lib semantic::indexing::service::tests::real_tools -- --ignored --nocapture
```

The four-case run passes after sharing the installer polling and lifecycle checks.
Environment: Go 1.27.1-X:nodwarf5, Python 3.14.8/pip 26.2.2, Node 26.12.1/npm
12.2.1. Logs: `/tmp/radiology-real-go.log`, `/tmp/radiology-real-python.log`, and
`/tmp/radiology-real-four-indexers.log`. Architecture comparison against
`before-real-go-acceptance` is neutral: no new hotspots, cycles or clone groups;
duplication remains 0.57% and 90 clone groups.

## Kotlin environment evidence — 2026-10-30

The pinned scip-java source builds and installs under the full local OpenJDK 27,
but generation then fails in Gradle script analysis with `Unsupported class file
major version 71`. This is a real compatibility failure, not successful acceptance.
A private Temurin JDK 17.0.21+9 subsequently passes the complete two-file Kotlin
lifecycle using Gradle 9.7.1 and Kotlin 2.4.20. The JDK archive's SHA-256 matches
Adoptium's asset metadata: `8b9b611d917e3dfd8419d3566b5949226f69b052fed7e109c1368add1dfe8711`.
JVM workers are capped for QA with `JAVA_TOOL_OPTIONS=-XX:ActiveProcessorCount=16`.
System Java selection and viewed inputs are unchanged. This private JDK is a QA
prerequisite, not a new automatic JDK installer in the app.

The runtime-only local Java 17 directory also fails installation because it lacks
javac; the shared installer now rejects this before downloading. See
[prerequisite handling](rust-tool-provisioning.md).
Logs: `/tmp/radiology-real-kotlin.log` (runtime-only),
`/tmp/radiology-real-kotlin-jdk27.log` (incompatible runtime for generation), and
`/tmp/radiology-real-kotlin-jdk17.log` (passing Kotlin-only lifecycle).

## Mixed Kotlin/Java lifecycle — 2026-10-30

The final opt-in `pinned_kotlin_java_install_navigation_unicode_reuse_and_reload`
passes with the current shared installer. It builds pinned scip-java
0.13.1-radiology-9f288dd339c7, provisions its private Gradle 9.7.1 fallback, and indexes
a real Kotlin 2.4.20 project under the private full JDK 17 above. Two Kotlin files
and one Java file produce three documents with zero missing sources and no
partial/stale/cache-warning state. A Kotlin call after an emoji resolves exactly
to the Kotlin helper; that helper's call resolves to the Java declaration's exact
line/column range. The second assertion uses the restored cache-only index, so
cross-language identity survives persistence. Both compiler plugins run. The bounded diagnostics retain the tool's nonfatal
`methodDisambiguator: fixture/JavaHelper.twice not found in sibling list` warning;
the asserted definition/reference identities still resolve. This fixture does not
prove Java overload behavior.

The shared lifecycle also proves Update reuse, source-digest validation and exact
unchanged viewed inputs/inventory. No `.gradle`, `.kotlin`, `build`, SCIP-shard or
plugin directories appear in the viewed root; temporary tool/workspace ownership
is removed with the fixture. The actual mixed-language run takes 108.82 seconds
including installation. This timing describes this small QA fixture only.

```sh
JAVA_HOME=/path/to/full/jdk17 \
JAVA_TOOL_OPTIONS=-XX:ActiveProcessorCount=16 \
cargo test --lib pinned_kotlin_java_install_navigation_unicode_reuse_and_reload -- --ignored --nocapture
```

After the harness change, all four other real-tool cases pass separately. Workspace
gates pass 425 app tests (six ignored), eight shared tests and 30 doctests; strict
Clippy, formatting and workspace build pass. Architecture remains neutral, without
new hotspots/cycles/clones. Logs: `/tmp/radiology-real-kotlin-java.log`,
`/tmp/radiology-real-kotlin-other-tools.log`,
`/tmp/radiology-kotlin-java-{workspace,clippy}.log`, and
`/tmp/radiology-kotlin-build.log`.

## Real C/C++ scip-clang lifecycle — 2026-10-30

`real_scip_clang_response_inputs_navigation_unicode_reuse_and_reload` passes
against the official v0.4.0 x86_64 Linux release asset. This is a manual QA download;
the app's manual-only Clang provisioning policy remains unchanged. Release metadata
is saved as `/tmp/radiology-scip-clang-release.json`. The downloaded binary reports
`scip-clang 0.4.0` and Clang/LLVM commit
`2078da43e25a4623cab2d0d60decddf709aaea28`. Its locally measured SHA-256 is
`06fd18c576f979a726c651594644ec4a35db4f471f2160b3f72eb89fa6001784`;
this records the tested artifact, not an upstream checksum verification claim.

One shared lifecycle runs separately in C11 and C++17 modes. Each real translation
unit includes a repository header and uses a copied GNU response file. Production
rebasing expands/repaths its include, source and output arguments before invoking
the actual compiler indexer. Both produce two documents, zero missing sources,
zero errored translation units and no partial/stale/cache-warning state. An emoji
before the call exercises UTF-8 byte columns; exact header definitions and caller
references, Update reuse, cache-only reload, source-digest rejection and private
workspace cleanup pass. Viewed source/header/response/database bytes and inventory
remain unchanged; no object or index output appears in the viewed root.

```sh
RADIOLOGY_REAL_SCIP_CLANG=/path/to/scip-clang \
cargo test --lib real_scip_clang_response_inputs_navigation_unicode_reuse_and_reload -- --ignored --nocapture
```

The Rust and Clang external-tool overrides now share their existing request setup;
the real Rust feature-gated case is rerun to check that path. Architecture comparison
against `before-real-clang-acceptance` is neutral, without new hotspots, cycles or
clone groups. Logs: `/tmp/radiology-real-clang.log` and
`/tmp/radiology-real-clang-rust-helper.log`. Workspace gates pass 425 app tests
(seven ignored), eight shared tests and 30 doctests; strict Clippy and formatting
pass. Gate logs: `/tmp/radiology-real-clang-{workspace,clippy}.log`.

## Limits and remaining acceptance

These are small real-tool projects, not evidence for dependency-heavy or broad
mixed-language corpora. They exercise backend production paths, not new native UI
interactions. Broader C/C++ databases/dependencies, Android/AGP and broader Gradle corpora remain open, as do the
broader recovery, retention and final visual/accessibility/scaling/font/sibling
reuse audits. See [implementation queue](rust-parity-implementation.md),
[provisioning](rust-tool-provisioning.md) and
[Rust feature acceptance](rust-feature-configuration.md).
