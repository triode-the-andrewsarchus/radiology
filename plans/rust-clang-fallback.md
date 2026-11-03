# Rust C/C++ best-effort indexing

Accepted checkpoint against Swift [best-effort defaults](clang-best-effort.md).
The standalone Rust app now offers Clang indexing when its source inventory has
C/C++ inputs but no real compilation database. This extends the existing
[configured Clang adapter](rust-clang-indexing.md), without running a source build.

## Contract and reuse

Real databases retain shallowest-path/lexical priority. Otherwise the strategy
uses the exact versioned evidence `Radiology best-effort C/C++ defaults v1`.
Existing strategy identity and inventory fingerprints separate generated, reused
and cached defaults from configured builds. Adding or removing a real database
invalidates the other strategy's result. No cache schema change is needed.

The public `clang_fallback::database` API generates standard argument-array JSON
from the already copied private workspace. It reuses the existing database command
format, cancellable reads, inventory validation and language registry. The service
owns the private database, output and supplementary logs through its existing run
directory and cleanup. Original sources and build inputs are never rewritten.

All translation units are included; header-only repositories substitute their
headers/fragments. GNU11 C and GNU++20 C++ defaults are explicit. Uppercase `.C`,
`.h`, `.tpp` and `.tcc` use C++, matching the reference; existing C++ extensions
remain supported. Include inference prefers nearby ancestor directories and
existing include/inc/src children, then follows literal include names through
matching repository headers. Candidate ordering is deterministic, with eight
matching headers per include, 64 include directories and 128 active recursion
entries. Cycles terminate. There is no translation-unit count cap. Leading-dash
source names are passed with `./` so they cannot become compiler options.

The app card explains GNU defaults and missing headers/defines/targets/custom
preprocessing. A defaults result is always Partial, even with complete document
coverage, after project reuse and after a warm cache reload. Configured Clang
results also retain Partial when compiler errors or errored-TU summaries accompany
a usable SCIP file. Compiler diagnostics are exposed; jobs are bounded to four.
Existing UI Small/secondary styles and the shared explanation renderer are reused.

## Validation

Seven generator tests cover deterministic commands, extension mapping, inferred
includes, cycles, bounds, malformed paths, cancellation and 2,050 translation units.
Four service tests cover private materialization/cleanup, cache and project reuse,
both real/default transitions, partial diagnostics, and a pinned PR snapshot whose
working-checkout ignored database must not influence its head inventory. An actual
app-frame test checks discovery and visible limitations.

Full workspace: 729 app tests pass, eight real-tool tests are ignored by default;
two binary, 13 font and 21 text tests pass. All strict Clippy/check/build/rustdoc,
format/diff gates and 56 doctests pass. Resource smoke checks pass. Rendered public
generator API documentation was inspected.

Explicit real-tool acceptance uses the official Linux x86_64 scip-clang 0.4.0
release, SHA-256
`06fd18c576f979a726c651594644ec4a35db4f471f2160b3f72eb89fa6001784`.
Both the existing configured C/C++ response-file lifecycle and new no-database
cross-file C++ lifecycle pass. The latter includes a missing header, Unicode/CRLF
source, exact definition coordinates, reuse and warm cache with tools absent.
Compiler errors preserve useful results and Partial status.

Native 1×/2× dark/light/relaunch checks run that actual tool against uppercase
`boot.C`. Both produce one indexed document, zero missing documents and one tool
invocation. Relaunch shows Partial cached without reinvoking the tool. Source and
binary hashes stay unchanged, and core processes exit zero. This is fixture
acceptance, not acceptance of the separate seL4 pipeline.

Evidence: `~/.local/state/radiology-qa/clang-fallback-20261102-99b7ded`.
The tool executable stays in the QA cache; the archive records its digest.

## Architecture review and remaining work

Against `before-clang-fallback` at 99b7ded: +796 NLOC, +20 analyzed functions,
four files; average CCN 3.39→3.38, maximum 132 unchanged. The 33 functions above
CCN15, 126 clone groups and one cycle are unchanged. Normalized clone identities
have zero additions/removals; raw comparison flags old clones in touched test
module declarations.

One genuine Valknut warning remains on `prepare_compilation_database`: CCN12,
cognitive30; Lizard CCN15, 39 NLOC. It now dispatches between real and generated
inputs, validates/reads real evidence and propagates filesystem/cancellation errors.
Both paths share directory creation and database writing. That added selection
is necessary; further wrapper extraction solely to lower a metric would not
simplify ownership or behavior. No new dependency, pool, workspace copier, cache
writer or cycle was introduced. Existing high-complexity functions are unchanged.

Managed portable Clang tool installation remains a separate gap: this checkpoint
requires an available scip-clang executable through existing tool discovery.
Indexing personalities, the multi-profile seL4 union, shortcut help and broader
language/UI acceptance remain required parity work. Full assistive accessibility
and actual sibling-app integration remain outside the current scope.
