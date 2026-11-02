# Radiology's pinned Kotlin scanner patch

Source: `tree-sitter-kotlin-ng` 1.1.0, upstream revision
`77dd60ea0a9003ce062c9728a513ffe1aaff8c82`:
https://github.com/tree-sitter-grammars/tree-sitter-kotlin/tree/77dd60ea0a9003ce062c9728a513ffe1aaff8c82

The original crates.io archive SHA-256 and unchanged-file hashes are recorded in
`UPSTREAM.json`. `Cargo.toml.orig` preserves the upstream manifest. The upstream
MIT license is retained in `LICENSE`. The root workspace patches this exact
package to this directory; it does not download a replacement during builds.

Two local scanner changes are recorded in `scanner.patch`:

- In the existing no-newline switch, permit the final class-member separator at
  `}` only when the parser expects `CLASS_MEMBER_SEMI`. This handles compact
  interfaces, properties, function bodies, enum members and nested classes without
  accepting missing operands, parameters, braces or intervening separators.
- Advance over the `!` in the semicolon lookahead path instead of skipping it.
  The external `!is` token then owns its full original byte range. Rust no longer
  needs a source-span correction in native highlight preparation.

`src/parser.c.gz` is the unchanged generated upstream parser, compressed with
`gzip.compress(data, mtime=0)`. It expands to 22,443,237 bytes, SHA-256
`9ff65161845b9e9c9d62c12e9a4e4b8d8628bdc31c681ec7e6b4bd3bd6444cb3`.
The build script expands it only into Cargo's OUT_DIR and uses the existing C11
compiler configuration. There is no Node, generator, shell-command, network or
source-checkout requirement. The compressed input is 1,090,208 bytes. flate2 was
already locked in Radiology; it is additionally used as a build dependency.

The matching upstream `parser.h` is retained. It differs from the older Swift
scanner header in this repository and must not be substituted merely to reduce
clone counts. Unused upstream alloc/array headers are omitted. Native bindings,
grammar specification and node metadata are unchanged. The three upstream corpus
files are checked against their recorded hashes; Rust tests compare all 22 expected
named trees in LF/CRLF, alongside new compact/error/range regressions.

To update, obtain a reviewed upstream package, verify its archive/revision, replace
unchanged inputs and hashes, and reapply/review `scanner.patch`. Keep the generated
parser byte-identical to that package and recompress deterministically. Retire this
patch only when upstream fixes pass the same compact, error, range, corpus and
native UI checks. No upstream changes have been submitted by this task.
