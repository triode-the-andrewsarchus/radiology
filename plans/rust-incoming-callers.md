# Source-validated incoming call paths

## Contract and current implementation

The Swift reference disclosure traces callers from the containing callable of a
selected source hit, not just from the referenced symbol. The new public Rust
`semantic::incoming_calls::trace` implements this backend contract. Controller,
reader disclosure and native acceptance remain required before this feature is
considered integrated. The legacy index-only CallerTreeBuilder remains a public
compatibility API; it is not used by the new query.

The query captures the same `SourceQueryInputs` as references/previews and checks
the selected group's complete source digest and exact canonical range before
tracing. It primes displayed bytes, reads other paths from captured checkout/PR
storage, rejects Before source and validates generated source identities. Every
accepted file is parsed once per request and records its source digest.

Call ownership comes from shared parsed callable scopes, including innermost
nested functions and closures. Named owners require an exact indexed definition
at the parsed name's first byte. Incoming edges require a parsed callee name at
the indexed reference start and the matching scoped semantic identity. The
existing CalleeResolver moved from critical to semantic ownership; Critical Lens,
call summaries and incoming callers now share that resolver without duplication
or a reverse dependency on Critical Lens.

Callbacks/aliases, closures, top-level sites, unsupported call structure and
unindexed callables stop with explicit explanations. Unreadable files, source
identity failures, invalid coordinates and parse errors retain coverage notices.
No indexed callers is reported only when the queried branch has no missing
coverage. Coverage loss on one branch does not taint another branch's leaf note.

Traversal is iterative depth-first with source-ordered siblings. Ancestor scoped
symbols and canonical declaration locations stop recursion before rendering a
repeated callable. Repeated entry functions on distinct sibling paths remain.
Default limits are 256 rows, 512 distinct attempted files and 64 MiB accepted
source. Per-file source-read caps also apply. Pending continuations and each
incoming batch are bounded by the requested row budget; exhaustion sets has_more.
Explicit expansion doubles saturating limits. There is no arbitrary depth cutoff.
Full native-read/parse interruption remains part of the broader lifecycle queue.

## Verification and architecture

Nine new tests exercise the Swift branch fixture and exact expected depth/name
pairs; emoji/UTF-16 positions; callback, closure and nested-function boundaries;
row-budget prefix retention; mutually recursive branches and siblings; stale,
missing and invalid sites; initial digest/Before/cancellation rejection; primed
bytes with the displayed file removed; source budgets; unindexed/top-level
ownership; document-local identities at identical offsets in different files;
and a 300-deep chain with expansion beyond the legacy depth cap.

Architecture baseline: before-validated-callers at 6db3260. No new product
hotspot or cycle; maximum CCN stays 100, average moves 3.33 to 3.34. The
comparison reports two new short test clones (source-input fixture setup and
branch name/depth assertions), raising clone count 86 to 88 and duplication
0.69% to 0.70%. Parameterized rejection checks reduced earlier duplicate test
blocks. The remaining clones are assertions and required SCIP fixture setup,
not duplicated product algorithms; adding general wrappers solely to hide them
would add complexity without a useful shared mechanism. The production callee
resolver itself is reused unchanged.

The unavailable swiftui-pro, macos-design and typography-designer skills remain
absent from configured skill/plugin locations. Swift source and its caller tests
provide the behavior contract. This commit adds no typography or UI changes.

## Required integration

- Bind a replaceable background trace to each expanded reference, with retiring
  publication on source/index/configuration changes and exact retained state.
- Render names, source locations, caller identity colors, depth and boundary notes
  in the existing virtualized reference disclosure; keep navigation explicit.
- Expose partial notices and Trace More without losing selection or focus.
- Verify pinned review reads, source invalidation, independent disclosures and
  native keyboard/pointer navigation of tree rows.
- Add request/revisit cache reuse within the full reader lifecycle work.
