# Rust Critical Lens parity

## Exact call facts and projection backend (2026-10-30)

Rust source analysis now extracts named/macro/dynamic call sites from its existing
native parse. It reuses Swift's call queries for Rust, Python, Go, ECMAScript and
Swift. Kotlin adapts those rules to the linked kotlin-ng node names; C has a small
native call query. Query compilation and active-match-limit failures are explicit,
and unsupported languages carry no call coverage. Queries compile once and are
shared across source workers. Call facts carry exact path/revision/SHA-256 identity.

Critical projection follows the Swift call-focused contract: recede the complement
of syntactic call-name ranges, retaining critical and unresolved/unknown callees.
Only resolved ordinary identities demote a call name. Same-start exact occurrences
win over broader candidates, single-line ranges use document encoding, and scoped
local identities are retained. No callee-spelling heuristics are introduced.
Missing/stale/unsupported semantic encodings and Before leave names unresolved;
unsupported/mismatched syntax produces no dimming. Supported syntax with no calls
recedes all bytes, matching Swift.

The existing policy is reused with indexed package ownership and signature data,
including symbol/package overrides. Descriptor extraction now shares the semantic
SCIP parser rather than splitting on the last space. SourceProjection binds to
exact bytes and revision, validates scalar boundaries and supports binary range
lookup. Text, coordinates and source layout remain unchanged.

Tests cover all nine linked grammar variants, comments/strings/declarations,
computed calls/macros, UTF-16 after emoji, semantic-vs-spelling classification,
override precedence, stale/Before/unsupported encodings, exact occurrence
selection, changed syntax/content/path/revision, normalization and cancellation.
Validation: 154 application tests, 5 reader-text tests, 30 doctests, fmt and Clippy
pass (the existing external-Clang acceptance test remains ignored by default).

## Remaining integration and full parity requirements

Backend `e3ee241` is integrated. `CriticalLens` uses the existing replaceable
read-worker job, retains only exact-source publications and retires work on mode,
source, index and repository transitions. Semantic indexes are shared immutable
`Arc` snapshots instead of cloned whole indexes. Source call facts are retained
from the background parse; caret movement does not re-parse or request projection.

The viewport splits syntax-colored spans at projection boundaries, including
unhighlighted fallback text. It checks source identity and mode, preserving bytes,
font metrics, selection/copy/navigation and wrapping. While pending, source remains
undimmed; the source footer reports preparation, call-focused presentation or
unsupported call syntax. The old color heuristic and unused callee-name heuristic
API were removed. Actual criticality still uses semantic identity and metadata.

Application tests cover syntax-only projection, asynchronous import arrival,
retiring Normal-mode requests, source changes and empty-repository transitions.
A renderer test checks one broad colored span across Unicode/projection boundaries,
exact original text/geometry, Normal restoration and rejecting stale projections.
Native private Weston/Niri acceptance uses an imported UTF-16 fixture: runtime
clone recedes, indexed system_open and unresolved mystery remain prominent, and
Normal restores syntax colors. Captures:
`/tmp/radiology-critical-native-final/{initial,critical,normal-restored}.png`.
The app and both compositors exited zero.

Critical-specific tests, 30 doctests, fmt, Clippy with warnings denied and binary
build pass. A full run passed 156 application tests plus 5 reader-text tests, but
repeated later full runs exposed the existing probe test's Linux `Text file busy`
startup race (including its timeout stage). It remains a separate runner repair;
these observations did not establish a reliably clean full-suite gate at the
integration commit. The subsequent deadline-bound startup repair now passes
157 application tests, 5 reader-text tests, 30 doctests, fmt/Clippy/build and
20 additional parallel library-suite runs. The deterministic regression covers
real ETXTBSY, release, timeout and cancellation; the specific original writer
remains untraced.
Architecture comparison has no new product clone or CCN threshold spike; it still
reports two GUI test clones and structural flags for projection orchestration and
the existing source show function. Maximum CCN remains 100. This is not a clean
architecture or full parity verdict.

[Repository decision storage](rust-critical-marks.md) is now implemented and
verified; reader load/save/menu/projection integration is pending. Marking UI, call graph and
frontier summaries, hover models, count/fold annotations, complete query coverage
and full visual/interaction acceptance remain required. This backend alone does
not complete Critical Lens or the broader Rust/Swift parity goal.

## Persisted source decisions (2026-10-30)

Repository-scoped symbol/package menus now feed the background projection in
Normal/Critical mode, with exact semantic targeting, durable lifecycle and restart
acceptance. See [mark contract and verification](rust-critical-marks.md). Earlier
statements that reader overrides are empty or menus remain absent are superseded.
Call graph/frontier and hover/fold annotations remain open.
