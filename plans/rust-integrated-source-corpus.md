# Integrated source corpus acceptance — 2026-11-02

The checked-in Swift `SyntaxFixtures.names` contains twelve source fixtures. The
Rust native-highlight corpus still listed the original ten; C/C++ had dedicated
structural tests but were omitted from this shared path. Expanding the corpus
exposed an actual production gap: `NativeHighlights::from_tree` returned an empty
snapshot for C/C++ because its older grammar allowlist stopped before them, despite
their linked queries already being used for occurrence exclusions.

## Repair and reuse

All ten linked grammars now use the existing bounded capture/recolor pipeline.
The redundant allowlist is removed; no new parser, query, state owner, runtime,
dependency or UI setting is introduced. Existing query failures still publish the
same explicit fallback. C and C++ receive their shipped Swift query roles through
canonical byte ranges, including preprocessor/function/method/template/type roles.
Native painting and recoloring keep the established regular monospace and palette.

One test-only catalogue reads all twelve original fixture files directly. Syntax
and actual-app checks share it. Its callable column identifies a multiline body
for committed-navigation/fold checks: Kotlin script uses `collect`; its one-line
`render` expression intentionally has no collapsible multiline body. The initial
expectation of a `render` fold was a test-design correction, not a product failure.

Resource verification now requires native colors for every structural grammar,
adds a real C++ template/call probe and removes the obsolete per-probe fallback
escape flag. Its existing test compares exercised versus registered grammars so
another omission cannot silently pass. Non-structural formats retain the separate
Syntect smoke path. No native grammar claim is made for those formats.

## Evidence and its scope

| Swift reference / contract | Current evidence |
| --- | --- |
| `SyntaxFixtures.names`, `SyntaxLanguageTests`, `CSyntaxTests` | Shared twelve-fixture catalogue covers Swift, Rust, Go, Python, JavaScript/JSX, TypeScript/TSX, Kotlin/script, C and C++. Existing native syntax test checks LF/CRLF, parse success, exact declaration/call/scope/fold/capture ranges and recolored source bytes in both appearances. |
| `SyntaxQueryTests`, native color roles | Both C grammars join the existing query-compilation and Unicode/CRLF token-role cases. The expanded shared corpus fails with the original allowlist (`sample.c must retain native colors`) and passes with the repair. |
| `SourceBufferTests`, `SourceCaretTests`, function-fold/navigation behavior | One app workflow opens every fixture plus malformed-tail variants, in LF/CRLF at1×/2×:96 source cases. Real source workers publish the buffer/facts; actual app frames verify exact bytes/name ranges, parse-recovery flags and native source identity, committed function reveal, raw X fold/unfold, exact selected-name Copy, wrap/theme recolor preservation, cross-file Back destination and unchanged disk bytes. This is representative integrated behavior, not a claim that every Swift assertion was rerun. |
| Resource packaging | Locked `--verify-resources` now requires native C/C++ parse/outline/fold/call/occurrence/color probes along with all other linked grammars. |

Final locked workspace:759 app tests pass, nine opt-in ignored; two binary,13 font,
21 text and59 doctests. Strict Clippy/build, formatting/diff checks and resource
smoke pass. The explicit pinned-backend IME tests were accepted in the preceding
checkpoint; this change does not modify that package.

## Native reader observations

Private Weston/Niri sessions run the actual standalone binary at1×/2×, with the
original twelve source files and a C++ malformed-tail variant in one repository.
Accepted captures show C/C++ colors in dark/light appearance, wrap and function
fold/unfold, exact line destinations and Copy, cross-file history and the Partial
structure notice while recovered C++ remains navigable. Source and executable
hashes remain unchanged; app/Weston/Niri exit zero. Relevant dark/light, expanded,
recovered and history captures were visually inspected.

The initial native harness expected one Back to cross both the C file-open and
its later line-jump entries. Observed C-at-line1 was correct. Final checks explicitly
verify that intermediate departure, then the second Back returns to C++ at line39
and copies its exact22-byte header. All five clipboard checkpoints pass per scale;
the C header is40 bytes. The earlier diagnostic cases remain in the archive.

The accepted1× workflow samples RSS219,803,648→225,308,672 bytes;2× samples
258,105,344→262,299,648. CPU ticks, monotonic time and clock rate are retained. These
are scoped observations during interaction/software-rendered screenshots, not a
leak/performance acceptance threshold or a comparison against Swift. Large-source
frame/worker/RSS stress remains required.

## Architecture review

Against `before-integrated-source-corpus` at `fc5254f`: +134 NLOC/one test function,
with production NLOC reduced15. Native `from_tree` CCN3→2 and resource `verify_native`
CCN8→7; the integrated test is CCN11. Average/max CCN3.38/132 and36 existing above15
functions are unchanged. All125 normalized clone instances/1335 duplicated lines,
one existing cycle and eight structural findings are unchanged. The raw comparator
reports REGRESSION due existing fragments in the touched test module and an untouched
Swift Python chokepoint rank change. The complete fragment/file-pair Counter proves
no clone addition or replacement; these reports do not identify new architecture.
The shared catalogue replaces the old local list rather than copying it.

Evidence: `~/.local/state/radiology-qa/integrated-source-corpus-20261102-fc5254f` holds
the red/green regression, full checks, native driver/actions/captures/clipboard,
source/binary hashes, process statuses, observations and before/after measurements.
QA artifacts stay outside `/tmp`; the sibling capture helper is read-only.

Remaining: complete the other Swift behavior reconciliation and integrated large/
long/wrapped Unicode stress with input owners/jobs pending; moving PR head/base,
added/deleted/renamed/binary files, exact pinned semantic/cache/reopen/eviction flows;
dependency-heavy C/C++ and Android/Gradle success or explicit failure outcomes; then
the final clean standalone/resource/reuse audit. This source checkpoint closes
neither the whole integrated gate nor overall parity. Actual sibling adoption and
full assistive accessibility remain deferred.
