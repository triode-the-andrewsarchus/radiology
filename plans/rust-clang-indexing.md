# Rust C/C++ compilation database indexing

## Rebase boundary (2026-10-28)

`compilation_database::rebase` converts compilation databases to explicit argument
arrays rooted in a disposable workspace. Commands preserve order and duplicate
source configurations. Explicit arguments take precedence over quoted command
strings; command strings are decoded with POSIX quoting, no expansion, and literal
hashes. Radiology never executes a command string through a shell.

Working directories and main inputs must belong to the original repository and
are remapped component-wise, including relative `..` paths. The returned private
working directories may need creation when the original build directory was empty.
Known include, forced-header, sysroot, module-map, dependency and output options
are rebased, preserving normal system/toolchain paths and macro literals. External
output destinations are redirected under the private run. Unknown options with
embedded repository paths fail explicitly rather than retaining live source paths.

Nested response files are read only from the copied repository, expanded with GNU
quoting and rewritten through the same argument path logic. Response nesting is
limited to 16 levels, response bytes to 8 MiB per command, arguments/commands to
200,000, and input/rebased JSON to 32 MiB. Rebased JSON is accumulated one entry at
a time, bounding expansion memory. Cancellation is checked between entries and
arguments. Unsupported/outside inputs, missing operands, invalid quoting, cycles
and size errors identify the failing command.

The backend uses existing workspace path normalization and regular-file reads.
The already locked `shlex` crate is now an explicit dependency for POSIX decoding;
GNU response quoting is distinct and implemented separately, following LLVM's
rules. References: [Clang compilation database specification](https://clang.llvm.org/docs/JSONCompilationDatabase.html),
[LLVM response tokenization](https://llvm.org/docs/doxygen/namespacellvm_1_1cl.html).

Tests cover relative/absolute inputs and joined/split flags, spaces, duplicate
configurations, literal macros/hash/shell text, output relocation, response
expansion/cycles/quoting, escaping paths, symlinks, malformed data and limits.
An explicitly run Clang acceptance test moves the original repository away after
rebasing, then successfully compiles the copied source/header to a private object.
It is ignored by default because it requires Clang; this machine passed it.

## Integration and remaining requirements

Backend `574f68d` is now integrated into the generation service. Each C/C++ run
prepares a database under its existing private run directory and creates only its
mapped working directories. The adapter requires that absolute prepared path;
relative/original/parent-traversing database paths are rejected. The copied input
database stays unchanged so standard input verification still applies. Successful
C/C++ output follows normal encoding, source-digest, coverage and cache publication.
Malformed databases fail before indexer execution and retain previous data.

Integration tests cover a tracked nested `build/compile_commands.json`, response
expansion, exact private argv/path capture, unchanged original bytes, disposal,
UTF-8 semantic publication and cache reload. Native Generate acceptance used a
fixture `scip-clang` in a private Weston/Niri session. The reader showed generated
coverage and semantic availability; captured database paths referenced only the
private copy. That copy was removed, and no objects/artifacts entered the viewed
fixture. App and both compositors exited zero. Local captures:
`/tmp/radiology-clang-native/{projects,generated}.png`; the captured rebased database
is in that directory as `rebased-database.json`.

Final checks: 142 application tests, 5 reader-text tests and 30 doctests pass;
formatting, Clippy with warnings denied and binary build pass. The separate real
Clang compilation test also passes when explicitly run. It is ignored by default
because Clang is an external prerequisite. Architecture comparison finds no new
production duplication, excessive CCN or cycle; its REGRESSION verdict is a new
six-line test-fixture clone. Max CCN remains 100; duplication is about 0.7%.

No installed `scip-clang` was found on this machine. Real compiler acceptance plus
fixture semantic publication do not prove real compiler-indexer output.

Input discovery retains its existing policy: tracked/unignored files in Git;
browser exclusions in non-Git directories. Thus ignored/untracked generated build
databases are not automatically included. The nested-build integration fixture
tracks its database explicitly; broader build-database discovery is still required.

External build working directories, generated dependencies outside the copied
inventory, Windows/MSVC response quoting, uncommon path-valued compiler options
(including named module-file/plugin option protocols), CUDA strategy coverage and
actual compiler-indexer acceptance remain open. They must not be described as
completed by support for ordinary repository-local C/C++ compilation databases.
