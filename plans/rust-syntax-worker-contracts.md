# Syntax worker assertion reconciliation

Reference: all nine SyntaxWorkerTests groups. Comparison begins at c20eba6.
This checkpoint changes tests/fixtures only. Source/syntax reconciliation reaches
46/66, leaving seven test-presentation and thirteen test-region groups before
compiler/inspector and final integrated/native/standalone/reuse acceptance.

## Assertion mapping

| Swift group | Rust evidence |
| --- | --- |
| rustSymbolsPreserveActualContainmentAndDeclarationOrder | worker_rust_roots uses the exact PreparedApp/SnapshotBootstrapGlobals/create/restore source, checking all seven roots in order and the exact image versus hide/restore impl children. Existing declarations helper also checks clean parse and canonical selections. |
| swiftQueriesProduceByteBasedHighlightsSymbolsScopesAndFolds | swift_worker_unicode uses the exact Scan.swift source. Shared language metadata identifies Swift, native captures/call facts match the buffer, struct/render captures and typed declarations remain, named callable scope plus Callable/Type folds exist. |
| swiftFixtureKeepsItsFullDeclarationStructure | The preceding language fixture matrix already checks the exact sample.swift Renderer/Scan/Titles/init/deinit structure and clean parse. That evidence is reused, not duplicated. |
| everyCaptureStaysInsideCanonicalUTF8ByteRanges | The existing twelve-fixture corpus now validates every highlight, fold, callable scope, full symbol and selection range through shared CoordinateIndex. Each slice converts to UTF-16, decodes to the exact original text, and round-trips both endpoints back to their original bytes. Every fixture's exact 胸部/放射線 marker joins the same checks. Both LF/CRLF and both-theme source-preserving recolor checks remain. |
| astralCharactersDoNotShiftCaptureRanges | Exact Astral.swift keeps the leading two x-ray scalars in its comment and the function capture resolves to scan through UTF-16 units after that prefix. |
| cfgGatedFunctionLocalOccurrencesIncludeShadowingButExcludeTextAndOtherFunctions | Exact Android-gated android_log source yields four message spellings, all within the selected function and before unrelated. String/comment clicks produce no syntactic selection. This remains decoration grouping rather than semantic shadow-binding resolution. |
| unsupportedFilesGracefullyRemainPlainText | Exact README.md yields no structural grammar, symbols, folds, callable support or native capture identity, without a hard parse error; source remains unchanged. Rust returns explicit empty facts instead of Swift's nil snapshot. Existing real reader plain-text explanations remain evidence for UI behavior. |
| malformedSourceStillReturnsPartialStructure | All ten exact malformed sources recover nonempty, canonically bounded native highlights, mark parse errors and preserve text. They now share MALFORMED_SOURCE_FIXTURES with the real app warning/recovery matrix. That consumer retains its stronger Rust variant with an earlier valid declaration, then checks warning retention, repair and reset. |
| emptyAndWhitespaceOnlySourcesAreHandled | All five exact paths test the Swift newline case plus empty and mixed-whitespace/CRLF input: no symbols/folds or parse error. |

The common UTF-16 test helper uses existing reader-text conversion APIs; no new
runtime converter or TextKit-specific API is added. Every successful slice also
proves its UTF-16 bounds through checked indexing and decoding. Source path and
language ownership remain in SourceBuffer/native fact identities, rather than
adding duplicate path/language fields to DocumentSyntax. The app continues to
publish source/syntax atomically through its existing background owner.

## Verification and structure

Archive: ~/.local/state/radiology-qa/syntax-worker-20261109-c20eba6/.
All 80 syntax tests, the actual app recovery/warning test, formatting and strict
all-target Clippy pass. The latest full workspace gate belongs to c20eba6: 901 app
checks and all 66 doctests. These later test-only additions have focused evidence;
no separate build/doc/full-run result is claimed or needed for production changes,
because this checkpoint contains none.

Architecture +203 NLOC/five test functions (four cases and one shared assertion),
no new file/runtime helper. Average/max CCN stays 3.43/132, with unchanged hotspots
and cycles. All 125 clone instances remain identical by normalized multiset
comparison. The new duplicate malformed-source table was replaced with shared
fixtures; the UTF-16 assertion is also shared across corpus and worker cases.
Raw equal-ranked Python chokepoint substitutions concern untouched source.

Next: exact test-region fixtures and test-hiding presentation/interaction. Full
accessibility and actual sibling integration remain deferred; Android SDK success
and previous parallel-run limitations remain open.
