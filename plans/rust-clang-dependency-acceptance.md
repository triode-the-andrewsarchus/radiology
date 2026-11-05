# C/C++ dependency input acceptance — 2026-11-05

## Compiler failure and repair

The new real-Clang dependency corpus exposed an incorrect compilation-database
rewrite. A valid `-Ivendor -include dependency/stage23.hpp` command became an
absolute forced-header path under the copied project root, bypassing Clang's
include search. The same problem affected relative `-imacros` operands. Clang could
emit incomplete source documents, but the requested vendor declarations were lost.

The existing `RebaseContext::path` now receives the compiler option and preserves
relative forced-header/macro-file operands. Clang resolves them in the disposable
working directory and through the already-rebased include directories. Absolute
repository operands still rebase into the private copy; other path/output behavior
is unchanged. Separated and joined option forms share this same method. No new
wrapper, process, dependency, cache owner or UI state is introduced.

The existing rebasing regression now checks both forms of `-include`/`-imacros`,
include-search spellings and an absolute repository forced header. It fails with
the original runtime and passes after the repair. The final valid real-tool fixture
also fails with the original runtime and passes with the repaired method.

Existing cached results are not migrated or silently discarded by this repair.
Use Reindex to replace an older partial result produced by the incorrect rewrite.
Ordinary Update remains exact-input/tool-version qualified.

## Real dependency lifecycle

The new ignored acceptance test uses the existing Fixture, request and lifecycle
helpers, plus the shared exact-input immutability check. Its private Git repository
contains 33 C++17 translation units, 24 chained template headers, a vendor macro
configuration and a public API header: 59 source documents. Vendor headers are
tracked even though browser discovery hides the vendor tree. Its compilation
commands are in an ignored `build/compile_commands.json`; GNU response files are
nested and include literal spaces/punctuation. Forced headers and macro files
resolve through `-Ivendor` using the real scip-clang 0.4.0 executable.

The initial run produces all 59 documents, zero missing paths and zero compiler
errors. Cross-file definition/reference identity and exact UTF-8 byte ranges after
an emoji pass. The shared lifecycle verifies project reuse, tool-free normalized
reload, source-digest checks, unchanged viewed bytes/inventory and private workspace
cleanup. The viewed Git status remains unchanged through that initial lifecycle.

A vendor-only macro edit makes the checkout cache visibly stale and prevents
project reuse; Update recompiles with the new digest. Both original and changed
index buffer checks verify the dependency's exact bytes. The cache's stale return
is intentional: checkout readers can display a stale index while the app guards
semantic availability. It is not an absent-cache contract.

Replacing one caller with an unavailable generated-header include retains genuine
compiler data from independent successful units. The report marks Partial even
though all 59 source paths happen to have compiler documents; path coverage alone
is insufficient evidence of a successful build. The diagnostic identifies the
missing header, another caller still resolves to the exact implementation, and
both normalized reload and project reuse retain the partial-build flag.

Finally an escaping response-file input fails explicitly and publishes no new
index. Original fixture bytes remain exact except for the three deliberate test
edits. Complete inventory equality catches unexpected viewed output; the explicit
object/index checks and private-workspace check also pass.

Run with the real Linux release tool:

```sh
TMPDIR=/path/to/private/tmp \
RADIOLOGY_REAL_SCIP_CLANG=/path/to/scip-clang \
cargo test --locked --lib real_clang_vendor_dependencies_invalidate_reuse_and_preserve_partial_builds -- --ignored --nocapture
```

All three real Clang acceptance tests pass together, including the earlier missing-
header fallback and four root/nested C11/C++17 response-file cases. The dependency
workflow takes about 1.25 seconds on this host; this is a scoped observation rather
than a throughput guarantee. The tool artifact SHA-256 is
`06fd18c576f979a726c651594644ec4a35db4f471f2160b3f72eb89fa6001784`.

## Evidence and limits

Early diagnostics include a missing brace in the authored template fixture, which
was corrected, and an incorrect assertion that a changed checkout cache must be
absent, replaced by its documented stale-state check. A broad test-name filter also
selected the Rust real-tool test without its private rust-analyzer prerequisite;
that test failed tool discovery. The final explicit Clang-only three-test run is
green. No new Rust-tool compatibility claim follows from these runs.

This backend acceptance supplements the real 67-profile seL4/native evidence. It
is a bounded dependency corpus, not the original 11,513-unit tkfiasco run or real
Android/AGP acceptance. External dependencies/sysroots, arbitrary generated build
inputs, CUDA and compiler-plugin protocols retain their documented limits. No new
native UI run or Swift execution is claimed for this backend repair. Full parity
still requires the remaining behavior/compiler and standalone/reuse audit.

Full locked workspace: 761 app tests pass (ten opt-in ignored), two binary,
13 font, 22 text tests and 59 doctests. Strict workspace/all-targets Clippy,
locked binary build/resource smoke, formatting and diff checks pass. Cargo uses
16 build jobs on this 32-logical-CPU machine.

Architecture against `before-clang-dependency-acceptance`: +185 NLOC/one test,
production path CCN6→8 and new test CCN9, average/max unchanged at 3.39/132. All125
clone instances/1335 duplicated lines, one existing dependency cycle and eight
structural findings are unchanged. The comparator's Python chokepoint rank change
is in untouched Swift reference scripts. No new hotspot or architecture mechanism.

Evidence: `~/.local/state/radiology-qa/clang-dependencies-20261105-0877ee3` retains
red/green runtime and unit logs, the real-tool reports, full gates and baselines.
