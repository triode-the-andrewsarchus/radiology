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

The backend is implemented; current viewport still uses the old color heuristic.
The next integration must replace it with this exact projection, run projection
work off the UI thread, retire stale file/index/mode/policy requests, and expose
accurate loading/degradation. Source call facts must be retained from worker
publication, without re-parsing on caret movement.

Repository-scoped persisted symbol/package decisions and menus, call graph and
frontier summaries, hover models, count/fold annotations, complete query coverage
and full visual/interaction acceptance remain required. This backend alone does
not complete Critical Lens or the broader Rust/Swift parity goal.
