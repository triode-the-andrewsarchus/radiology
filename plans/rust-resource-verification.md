# Headless Rust syntax-resource verification

## Contract

`cargo run --locked -- --verify-resources` now exercises the actual reader's
DocumentSyntax analysis and highlighting paths. The earlier implementation only
checked whether Syntect could return a syntax, so a JavaScript fallback could
conceal missing TypeScript structure or broken native queries (review R17).

The standalone CLI performs no repository discovery, indexing, external-tool
installation or window creation. The checker is private to the binary and reuses
the library's existing parser, query registry and color pipeline; it adds no
parallel syntax implementation or dependency.

Nine small fixtures cover every currently registered native grammar: Rust,
Python, Go, JavaScript, TypeScript, TSX, Swift, Kotlin and C. Each requires:

- A parse without errors and the expected probe declaration in the outline.
- Supported fold extraction with a multiline fold.
- Supported call extraction with the exact target callee token.
- A usable function-local occurrence selection over the value parameter.
- Expected native colors for Rust/Swift/Kotlin/ECMAScript, rather than silently
  accepting a fallback for those fixtures.
- Complete source-byte reconstruction and at least two syntax colors in both
  appearances. Rust additionally requires its inline test region to be detected.

C++/HTML/JSON/Markdown/YAML/TOML are separately labeled color-fallback probes.
Their output makes no native grammar claim. The Python fixture has two body lines
so a valid single-body-line fold suppression is not mistaken for a missing query.
Failures identify the path, failed stages and available query diagnostics. The
command continues through other fixtures and exits 1 if any probe fails; success
exits 0. The final message explicitly describes these as syntax-resource smoke
checks.

## Evidence — 2026-10-30

The two binary tests cover the complete checker and reject missing native
structure even when TypeScript fallback colors are available. Unknown native
paths are rejected. A separate negative control temporarily inserted an invalid
node into the TypeScript/TSX highlight source: the actual command reported both
query errors and exited 1 while the six unrelated fallback probes still passed.
The valid resource was restored before the final build and gates.

The final command exits 0 with DISPLAY and WAYLAND_DISPLAY unset. Copying only the
executable into an otherwise empty temporary working directory also passes with
those variables absent; it leaves that directory containing only the executable.
Thus the smoke check uses embedded resources without a source/sibling checkout or
a running desktop. This is checked against the recorded final binary hash.

Workspace gates pass 443 library app tests (seven ignored), two executable checker
tests, four font tests, eight text tests and 33 doctests; strict Clippy, build via
cargo run, fmt and diff checks pass. Architecture comparison against
before-native-resource-verification is neutral, with no new clone group, hotspot,
cycle or maximum-CCN increase. Evidence logs, standalone proof and hashes are
retained in `~/.local/state/radiology-qa/native-resources-20261030-4409820`.

These fixtures do not prove comprehensive language semantics, indexing or visual
parity. The language/role matrix and native TSX captures are documented in
[native colors](rust-native-highlights.md); broader workflow acceptance and the
requirement-by-requirement final Swift audit remain separate. Full accessibility
is deferred by the user.


## Rust native-color requirement — 2026-10-31

The Rust probe now requires source-bound native colors, matching its shared Swift
query activation. The full binary suite and copied-executable verification pass
without DISPLAY/WAYLAND_DISPLAY or a runtime source checkout. [Rust color and
performance evidence](rust-native-highlights.md#rust-query-reuse-and-long-line-loading--2026-10-31)
records the language matrix, exact-byte large comment, retained-source recoloring
and final native 1×/2× dark/light checks.
