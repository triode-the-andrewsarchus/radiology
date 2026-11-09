# Source-buffer coordinates and document outline contracts

Reference: four SourceBufferTests groups and three DocumentOutlineTests groups.
Comparison begins at 2e089e9. This is test-only reconciliation using existing
shared coordinates, symbols and outline owners. Together with registry/C mapping,
16 of 66 source/syntax groups have explicit mappings; 50 remain to reconcile.
Compiler/inspector and final integrated/native/standalone acceptance remain open.

## Source-buffer assertion mapping

swift_unicode_buffer_contracts uses the exact a🙂é/CRLF/漢字/combining-mark source
through SourceBuffer's existing CoordinateIndex. It checks byte line starts
[0,9,20], derived UTF-16 line starts [0,6,12], three lines, exact byte 5/unit 3
round trips, byte-inside-emoji clamping, byte column 3 of line 2, and every scalar
boundary plus EOF in both directions. This is stronger boundary coverage than
Swift's grapheme-start loop. Original CR/LF bytes remain included in global units.

| Swift group | Rust evidence and API conventions |
| --- | --- |
| coordinatesHandleEmojiCombiningMarksAndCRLF | Exact fixture assertions above, including clamping byte 3 to the emoji start before conversion. Shared typed UTF-16 lookup deliberately returns None for a split surrogate; egui uses scalar/canonical-byte positions, with no TextKit surrogate caret. |
| emptySourceAndOutOfBoundsLocations | Empty validated bytes yield one line and byte 0 even for line 100. Oversized columns clamp to line end. Existing real app signed-line command checks clamp negative/zero/99 input to the first/last valid line and preserve invalid edits. Rust's low-level byte_offset returns EOF for an out-of-document line, whereas Swift's byte-column helper clamps the line; the UI bounds the line first. Exact encoded imports reject oversized coordinates rather than silently relocating them. |
| binaryAndInvalidUTF8AreRejected | The exact NUL-containing and 0xff byte vectors both fail the real from_bytes constructor with BinaryOrInvalidUtf8. Trusted from_str test/host construction is not substituted as evidence for file validation. |
| convertsCanonicalByteRangesToTextKitRanges | The exact a🩻胸z selection converts through shared global UTF-16 offsets to start 1/length 3. Rust does not construct NSRange; canonical source bytes remain the UI/navigation coordinates and the reusable reader-text API supplies encoded units. |

Rust's imported-coordinate rejection policy was already established in semantic
index reconciliation. This checkpoint does not weaken it or add a duplicate
permissive converter simply to emulate Swift's private TextKit interface. The
existing [caret contract](rust-caret-parity.md) and line-command test prove the
corresponding user navigation rather than claiming the two low-level APIs share
identical invalid-input behavior.

## Outline assertion mapping

swift_outline_fixture uses the exact Shape/impl/short/larger/nested/tied source.
It checks function order [larger,short,nested,tied], line spans [7,1,1,1] and flat
function depth. LOCAL remains in the non-callable declarations. One global
selection-range uniqueness assertion covers callable/declaration identities and
cross-group collisions, consolidating two duplicate checks. Original symbol
pointer identity proves navigation references the actual immutable parse forest,
including nested functions. The exact multiline café/woman-technologist source
with its endpoint at the following line start spans three lines, proving half-open
line counting across multibyte text.

| Swift group | Rust evidence |
| --- | --- |
| methodsAndNestedFunctionsAreFlatAndLargestFirst | Exact fixture ordering, counts, depth, global identity uniqueness, retained LOCAL and original symbol references above. Rust's hierarchical parse forest is flattened by the existing outline_rows projection, not reparsed into another model. |
| halfOpenLineSpanUsesBytesAcrossMultilineUnicode | Exact supplied half-open range ends at line 4's start and counts only the preceding three source lines. |
| identicalFilesStillInvalidateOnEitherPathChange | Existing hidden_and_stale_targets rejects targets against equal bytes under other.rs, changed bytes under the same path, and a Before revision. The strengthened case resolves new targets and asserts exact replacement path/revision key and digest. The app's existing outline navigation test also rejects an action captured before opening another file. Rust publishes source and syntax atomically in PreparedSource rather than exposing Swift's separate syntaxPath/loading InputKey; old packets retire at the existing source owner, as covered by Context navigation publication checks. |

The viewport's positive replacement check prevents a false pass caused by disabling
all navigation after invalidation. Every new current target must resolve while
the old source target stays invalid. No new path key, revision counter, UI state
or query mechanism was introduced.

## Verification and structure

The full serial workspace gate at 2e089e9 passes 891 application tests, eleven
existing opt-in ignores, binary/font/text checks 2/13/22 and all 66 doctests.
That gate verifies the source-language and preceding caller-keyboard production
changes. Archive: ~/.local/state/radiology-qa/source-contracts-20261109-51aaa44/.

These later test-only additions pass seven source-buffer checks, all eighteen
outline checks, six viewport checks and the real signed-line command test.
After consolidating uniqueness, the final outline fixture check and strict
all-target Clippy pass again. No production source or rustdoc changed, so no
additional build/doc/full-run result is claimed for this test-only checkpoint.
Archive: ~/.local/state/radiology-qa/buffer-outline-20261109-2e089e9/.

Architecture +137 NLOC/two test functions, no new file/runtime helper. Average/max
CCN remains 3.42/132, with unchanged hotspots/cycles. Complete normalized clone
multisets stay 125 with zero additions/removals; the raw outline touched-file
flag concerns an existing fragment. The initial duplicate uniqueness assertion
was consolidated rather than hidden in a helper. Next: call-site and remaining
syntax/test-region groups, then compiler/inspector and final acceptance.
