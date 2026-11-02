# Kotlin compact declarations and operator ranges

## Contract — 2026-11-02

The native Kotlin grammar must parse valid final class members without demanding
an explicit separator before the closing brace. Interfaces, properties, function
bodies, enums and nested classes must retain complete declarations, calls, folds,
colors and original byte ranges. Real syntax errors must still report Partial
structure. An `!is` token must include its bang directly in the native tree.

The previously recorded compact Kotlin gap is now repaired. This restores facts
that recovered error trees could lose, rather than hiding their warning. The
reader's parse-quality policy remains unchanged and uses the actual tree error
flag. No alternate parser, source rewrite, warning exemption or UI state is added.

## Implementation and dependency boundary

The exact existing tree-sitter-kotlin-ng 1.1.0 package is locally patched under
`vendor/tree-sitter-kotlin-ng`. Its generated parser, grammar specification, node
metadata and native bindings are unchanged. Two small changes in the external
scanner handle the expected separator in the existing no-newline switch and keep
`!` in the external token's range. The Rust-side highlight range adjustment is
removed. All existing consumers continue sharing the same parsed tree.

The generated C parser is about 22 MB and is stored as a deterministic 1.09 MB
gzip input. The build script expands it into OUT_DIR with the already locked
flate2 implementation and compiles it with the existing C11 configuration.
Deployment still uses a statically linked grammar: no runtime files, generator,
Node, shell extraction or sibling checkout is required. The matching upstream
parser header is retained; unused array/alloc headers were removed. The vendor
package is excluded from workspace membership as a patched dependency. No locked
package version changed.

[Vendor provenance](../vendor/tree-sitter-kotlin-ng/PROVENANCE.md), `UPSTREAM.json`,
the MIT license and `scanner.patch` identify the exact archive/revision, unchanged
input hashes, changed files and maintenance procedure. QA verifies byte-identical
decompression, matching expanded compiler inputs and a clean dry-run of the patch
against the pinned upstream scanner. This local patch must be reviewed when
updating the grammar; it is not an upstream release or submitted PR.

## Tests and native evidence

Three tests add compact declaration coverage, malformed controls and the pinned
upstream corpus. Eight compact forms run in LF/CRLF and verify clean trees,
canonical names/ranges, plus calls and folds for a multiline callable body. Five
invalid forms retain parser errors. The strengthened existing Kotlin color case
checks raw `!is`/`!in`/`as?` node ranges before recoloring, so another display-only
workaround cannot make it pass. Old grammar negatives fail on the missing class
separator and on `is_expression` owning the expected `!is` token range.

All 22 upstream corpus cases preserve their expected named trees in LF and CRLF.
Fixture hashes are pinned; test matching removes field labels and formatting
whitespace only. This is the upstream package's corpus plus the added cases, not
an exhaustive Kotlin-language claim. The standalone resource probe now includes
compact interfaces and nested classes, so lost patch packaging cannot pass simply
by parsing an uncomplicated top-level function.

Workspace passes 581 app tests (seven ignored), two binary, four font, 13 text and
37 doctests. The strengthened binary resource tests also pass a final focused
rerun. Strict Clippy/check/build/rustdoc/fmt/diff gates pass. A copied executable
passes resource verification outside the checkout.

A native 1× old-build control reproduces the warning and incomplete compact-file
outline. Two final patched native runs at 1×/2× show the valid compact file without
that warning and with its restored outline/colors. They copy exact original
CRLF/Unicode source and the `read` member name, open a genuinely malformed Kotlin
file that still reports Partial structure, copy its bytes, then return with Back
to the valid file and switch appearance. Fixture/binary hashes remain unchanged;
all child exits are zero. Representative before/after/error captures inspected.
The first control harness failed because OCR read `Reader.kt` as `Readerkt`;
final filename normalization fixes that observation. Initial v2 after runs precede
the strengthened CLI probe; final product acceptance selects v3 after runs.

## Architecture accounting

Baseline: `before-kotlin-scanner-parity` at `0e13c7a`. The default comparison is
REGRESSION because it now measures previously external third-party source:
+1,675 NLOC, +141 functions, +19 vendor clone groups, and maximum CCN 132. Lizard
measures the original upstream scanner at CCN 131; the patch adds one switch case,
raising that function to 132. The copied grammar specification supplies 125 of the
new functions. This is exposed dependency code, not a newly designed application
algorithm. Refactoring that upstream scanner solely to satisfy the metric would
obscure the small auditable patch and expand maintenance scope.

Full normalized clone audit preserves all 91 existing groups. The 19 additions
are matching upstream parser-header fragments, generated node metadata and corpus
data; none is an application clone. The header differs from the older Swift
scanner header and cannot be substituted just to remove duplicate counts.

Excluding the explicitly identified vendor inputs for a separate application
comparison gives +140 NLOC, three test functions, unchanged max CCN 100 and 24
existing hotspots. Application production code loses the highlight workaround;
no new worker/service/abstraction is introduced. Dependency cycles remain zero.
The unfiltered figures above remain the recorded default result; no analysis
exclusions or metric-oriented source moves were introduced.

## Evidence and scope

Archive: `~/.local/state/radiology-qa/kotlin-scanner-20261102-0e13c7a`.
`accepted.json` selects the v2 old control and two final v3 patched runs. It retains
fixtures, exact copies, captures, hashes/exits/actions, role counts, negative and
corpus logs, provenance/compiler-input checks, default architecture reports,
full clone audit, separate application accounting and source diff.

The compact-member defect and Rust operator-span workaround are closed. Broader
language/build corpora, reader/key/UI, indexing/PR/cache and final Swift parity
acceptance remain open. Full accessibility and actual sibling adoption remain
deferred. No UI typography or platform idiom changed in this scanner increment;
requested SwiftUI/macOS/typography skills remain absent from installed roots.
