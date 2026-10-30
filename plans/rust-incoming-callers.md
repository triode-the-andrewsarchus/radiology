# Source-validated incoming call paths

## Contract and current implementation

The Swift reference disclosure traces callers from the containing callable of a
selected source hit, not just from the referenced symbol. The new public Rust
`semantic::incoming_calls::trace` implements this backend contract. Reference
disclosures now integrate the query with background ownership, virtualized
presentation, Trace More and guarded navigation. Native acceptance is recorded
below. The legacy index-only CallerTreeBuilder remains a public
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

## Disclosure integration and native acceptance — 2026-10-30

ReferenceDisclosure owns one source excerpt and one independently replaceable
caller job. Collapse drops both channels. Changed source/index inputs retire
publications; larger reference queries remap valid disclosures by exact site and
digest. Pending caller jobs rebind to newly displayed source. Caller-only reads
join the existing first-visit source-identity ledger. Tree expansion retains its
previous rows while loading. Exact digest-guarded scalar-column navigation shares
the existing source transaction, so changed bytes cannot mutate source/history.

Caller presentation reuses the reference list scroll area and variable row
geometry. It paints only the visible tree slice, uses stable identity colors,
shows filename/line and boundary notes, caps visual indentation at four levels,
and preserves complete depth/path in accessible labels and tooltips. The header
reports rows shown, distinct from the inspector's reference count. Partial
coverage and Trace More remain explicit. Right opens a reference disclosure;
Right again focuses its callers. Up/Down and Home/End move through caller controls;
Return/Space opens a site; Left collapses its parent and Escape returns list focus.
Disappearing Trace More controls also restore list focus. Tab remains available.

Caller-owned key presses use eframe's raw_input_hook and the existing shared
ordered dispatcher before egui computes automatic focus direction. A debugger
trace and consecutive-frame four-control regression demonstrate why consuming
keys only during update or applying a focus filter was insufficient in egui 0.27:
newly focused controls cannot accept a filter until a later frame. Shared filters
still express control ownership; the pre-frame hook prevents duplicate moves.
No framework fork, unsafe access or timing delay is required. Non-text caller
controls remain eligible for global reader shortcuts such as F12.

Native core acceptance at bc3a929 (PID 3766902) shows 256, 512 and 516 caller rows
with retained source state, partial notices and explicit top-level boundaries.
Consecutive Down events plus Return open other at b.rs:5, and Back restores the
original a.rs caret after the emoji. After the final expansion removes Trace More,
Down/Return operates on reference selection and opens b.rs:5. Left collapses the
tree while leaving source unchanged. Radiology and both private compositors exit
zero. The final global-focus change at deb1462 (PID 3773379) separately verifies
F12 from caller focus opening the exact b.rs:1 save declaration; all exits are zero.

Evidence lives in /tmp/radiology-caller-disclosure-native/build-evidence.json,
with accepted screenshot names and hashes, core/final commit checkpoints and the
final shortcut binary SHA-256. Core screenshots are in that directory; final
shortcut screenshots are in /tmp/radiology-caller-shortcuts-native. Rejected
filter-only screenshots are excluded from acceptance. caller-focus-gdb.log
records the observed frame-end focus reassignment. Test harness imports from the
sibling project remain test-only; the standalone product has no sibling dependency.

Full gates pass 262 app tests (one external-Clang test ignored), 5 reader-text
tests, 30 doctests, fmt, Clippy with warnings denied, build and rustdoc. Added
controller/application cases cover independent retirement, expanded-query tree
retention, caller-only first-visit invalidation, changed-source navigation refusal
and global shortcuts. UI cases cover coalesced clicks, a 1,000-row visible slice,
consecutive-frame focus and ordered commit. Architecture comparison against
before-caller-disclosure has no new product hotspot/cycle, unchanged max CCN 100,
average CCN down 0.01 and approximately unchanged duplication. It flags one added
short clone group; remaining reports concern compact fixture/assertion and UI
registration idioms rather than a duplicate caller or navigation mechanism.

## Remaining acceptance and broader parity

- Native pinned-review caller reads and a broader mixed-language/error corpus.
- More independent-disclosure and pointer/Tab flows across light/dark, scaling
  and narrow-window native acceptance, plus side-by-side Swift visual checks.
- Request/revisit cache reuse and finer native-read/parse interruption as part
  of the full reader lifecycle work.
- Richer under-caret metadata, extension/implementation links, external/local
  explanations and outline/bookmark keyboard parity.
