# Interruptible native source analysis

Implemented in `895f5c6` on 2026-10-30. Source loading, incoming caller reads,
Critical Lens bodies and definition-preview outlines now pass their request's
cancellation/deadline predicate through native parsing and derived syntax facts.
The app remains independently runnable with the existing worker pool.

## Shared analysis contract

AnalysisControl holds one latched stop decision for the analysis call. Once a
predicate requests retirement, subsequent checks remain stopped even if that
predicate would return false later. It is a stack-local guard, without a global
flag, separate runtime or GUI dependency.

The installed Tree-sitter 0.25.10 ParseOptions progress callback interrupts native
parsing. QueryCursorOptions interrupts matching, including scans that have not
produced any result. Calls, folds, occurrences and native colors share the bounded
match visitor; text predicates, capture order, role-specific fact limits and the
65,536 cursor match limit retain their behavior. Parser/outline/test dispatch uses
one source-to-grammar parse mechanism.

Outline recursion, test-region walks, import runs, call ownership and occurrence
word scans check the same guard between nodes, members or byte blocks. Small
helper operations, sorting, query compilation and individual filesystem syscalls
still complete their current operation. This is cooperative interruption rather
than a hard real-time deadline for every instruction.

DocumentSyntax publishes one complete result or a cancellation error. A stopped
native parse never falls through to text-only symbols; a stopped query never
publishes partially collected folds, pins, colors or call facts. Independent query
failures still retain their previous recoverable coverage/error behavior. Public
synchronous parse/analysis APIs keep an uncancelled path. Each parse owns a fresh
Parser, so retry cannot resume a retired parser's partial state.

SourceLoader retains its accepted source/history until a complete replacement.
Its post-analysis guard preserves cancellation versus timeout diagnostics.
Definition previews use the outline-only cancellable path; graph/caller work uses
the shared document analysis path. Existing graph-cache retirement checks prevent
cancelled syntax from becoming a persistent unreadable body. Color dispatch and
preview declaration-end preparation are separate cohesive pipeline operations.

## Runtime and fidelity evidence

Four added tests prove native parsing stops after a scheduled guard pulse, a
no-match native query stops without producing captures, cancellation at several
derived-fact stages returns no document, and outline/test walkers stop between
parsed nodes. Fresh Rust/Swift/Kotlin analysis succeeds afterward, and ordinary
unsupported-language text fallback remains available. The no-match test distinguishes
native progress interruption from checks that only run after a match.
Existing structural, Unicode/CRLF, scope, fold, query-bound, pin and color fixtures
continue to pass. Full workspace results: 346 app tests passed, one external-Clang
fixture ignored, eight reader-text tests and 30 doctests passed. Formatting, strict
Clippy, build and rustdoc pass. Cargo still uses 16 build jobs.

Reproduce native callback checks with:

```sh
cargo test --lib syntax::cancellation_tests -- --nocapture
```

Final log: `/tmp/radiology-native-cancellation-runtime.log`. It records parser
interruption at eight guard calls and a native no-match query stopping at eight
calls with zero captures. Test executable SHA-256:
`7071d02e7affd84f62de326a64d45788dc41ed0728913a15c9a5b51af8ef6bce`.
Log SHA-256: `4f1ead2e34e03cfab310f389bff6d6b18afd47ca10c2486b1e2189d86b3287ea`.

Architecture baseline `before-native-analysis-cancellation` adds no hotspot, cycle
or clone group. The initial preparation/preview complexity findings were resolved
by separating actual coloring and declaration-end operations. Shared match traversal
and a common fact-limit fixture remove duplicate setup. Two remaining touched-file
flags (preview/hover setup and outline/test-detector fixtures) are present in the
saved baseline.

## Native reader acceptance

Private Weston/Niri acceptance uses a small accepted a.rs, four 4,440,000-byte
Rust candidates b.rs through e.rs, and a small f.rs. One uninterrupted input batch
requests b/c/d/e, then f. Captures show a.rs retained while b.rs and e.rs are pending;
f.rs with its correct declaration appears in the capture requested with a 0.5-second
delay after its click. Explicitly cancelling another b.rs request retains f.rs.
This is interaction evidence, not a general latency benchmark or a count of active
worker threads. App PID 366043, Weston and Niri exit zero; pointer helpers are
intentionally terminated. The sibling capture helper is used only for QA.

Artifacts: `/tmp/radiology-native-analysis-cancel/manifest.json` and screenshots
`accepted-before-replacements.png`, `pending-b-final.png`, `pending-e-final.png`,
`fresh-small-final.png`, `explicit-cancel-pending.png`, `explicit-cancel-retained.png`.
Driver: `/tmp/radiology-native-analysis-cancel-driver.py`. Tested app SHA-256:
`ad2fe4dea3b7fa51ea8a74498bf539a07c364d91cc028d6dedd1ca80f3558386`.
The manifest includes hashes for the driver, both executables, log and screenshots.

[Nested SCIP decoding and map construction](rust-scip-cancellation.md) now support cancellation.
Full parity remains open: PR preparation/fetch cancellation,
source/graph/context caches, complete indexing/provisioning/provenance, PR-refresh
mapping and final visual/accessibility/scaling and sibling chrome/palette acceptance.
