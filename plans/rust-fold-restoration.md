# Exact-document fold restoration in Rust

## Contract (2026-10-30)

Successful source publication activates fold choices for a repository-relative
path, review revision and SHA-256 digest. Leaving and returning to identical bytes
restores manual collapsed regions; reloading identical source retains choices.
Changed bytes invalidate the older choices for that path/revision, including when
the text later changes back. Unfolding a region remains effective after revisiting;
old collapsed sets cannot resurrect. Only headers present in the current parsed
fold ranges survive restoration.

Activation happens only after the source loader succeeds. Failed reads preserve
the displayed source and folds. Explicit navigation then reveals folds containing
its destination, retaining unrelated restored choices. Before, After and checkout
states are separate. Repository and changed-review-snapshot transitions reset the
store, preventing same-path choices from leaking across contexts. State is session
memory, matching the existing pin lifetime; there is no new on-disk format.

`source::document_memory::DocumentMemory<T>` extracts the existing pin activation
mechanism and is shared by pins and fold sets. It moves nondefault payloads rather
than cloning them, avoids duplicate identity/transition logic and retains existing
pin color/ephemeral behavior. Hosts use FoldStore::activate after successful source
publication and reset the store at repository/snapshot boundaries. Unfold All acts
on the current document; repository reset clears all remembered documents.

## Verification

A fold-store test covers exact bytes/revisions, invalid headers, independent files,
explicit expanded choices, changed-byte invalidation and non-resurrection. An app
test covers file return, unchanged reload, failed reads, selective destination
reveal, per-file choices, changed bytes and repository reset. Existing pin and
navigation tests continue to pass. Full gates: 195 application tests, 5 reader-text
tests, 30 doctests, fmt, Clippy, binary build and rustdoc. One external Clang
acceptance test remains ignored by default.

Private native Weston/Niri checks collapse a.rs, switch to initially expanded b.rs,
collapse b.rs, then verify each is still collapsed after returning independently.
Artifacts are under `/tmp/radiology-fold-memory-native/`: `b-initial-expanded.png`,
`a-fold-restored.png`, `b-fold-restored.png`. App and both compositors exit zero.
No drawing, type scale, default folding policy or source bytes were changed.
Architecture reports +108 NLOC and a six-line app-test fixture/assertion clone;
no new product clone/hotspot/cycle, max CCN unchanged. The shared activation
mechanism replaces the pin-specific implementation instead of duplicating it.

## Remaining scope

Default fresh-document folding, initial PR changed-block folding, exact fold/pin
mapping through PR refresh/renames and broader large-file/viewport acceptance
remain open. A changed PR snapshot intentionally resets session fold memory until
that richer refresh contract is implemented. This does not claim full source or
Rust/Swift parity.

## Parsed body defaults and initial PR visibility (2026-10-30)

Fresh documents now initially collapse parsed callable bodies and leave type/module
containers open. Callable folds use the AST body span rather than the entire
outline declaration, preserving multiline signatures and decorators. Abstract
signatures are not bodies; unsupported syntax stays expanded. Collapse Functions
uses these same facts. Count annotations match both body header and ending line,
preventing a same-line enclosing container from receiving a child function's count.

Call scopes now retain body ranges and annotated declaration ownership in the
existing shared parse. Rust attributes (including intervening comments) and Python
decorators own only their attached callable. Swift protocol requirements are
captured as abstract callables. No new parse or text-name heuristic is introduced.

For an exact pinned review source, initial defaults open changed bodies and their
ancestors, including function header/annotation edits. The other revision's hunk
side is never used. Empty-side anchors inside bodies open those bodies; boundaries
next to closed braced functions do not open unrelated neighbors. Unterminated
indentation-body EOF is owned; a trailing newline is outside it. Hide Tests stays
independent and hidden unless explicit navigation requests its source.

An initialized flag is remembered alongside the collapsed set. An explicitly
expanded empty set therefore survives file return and never looks fresh. Defaults
run once per exact document; manual changes and restored choices win. Changed
bytes/review snapshots get fresh defaults. Navigation reveals its requested target
after adopting defaults, before the first viewport layout.

Initial body/PR choices are computed in the source worker and published atomically
in PreparedSource with matching bytes/syntax/hunks. UI adoption only moves the
prepared set when the document is fresh. An ancestor stack avoids a quadratic
parent scan. Hunk-overlap work and native parsing still have cancellation checks
around the operation; finer internal interruption remains open.

Seven new syntax tests plus a pinned-review app test cover fresh body/type rules,
expanded-empty restoration, abstract Swift signatures, both revision sides,
attributes/decorators, one-time manual precedence, empty/EOF anchors, Hide Tests,
multiline signatures and same-line containers. The shared real-Git review fixture
now accepts explicit before/after source, avoiding a duplicate fixture mechanism.
The app test proves changed body visibility, manual revisit state and Before
selection against pinned objects. Existing navigation and appearance tests now
explicitly account for fresh collapsed defaults. Full gates pass 203 application
tests, 5 reader-text tests, 30 doctests, fmt, Clippy, build and rustdoc.

Private native Weston/Niri evidence under `/tmp/radiology-fold-defaults-native/`
verifies first-open multiline signatures with body/sibling collapsed (`initial.png`),
fresh b.rs defaults (`b-default-collapsed.png`) and restored manual expansion while
its sibling stays collapsed (`manual-expansion-restored.png`). App and both
compositors exit zero. Runtime PR defaults are covered by pinned-object app tests;
no live GitHub/native PR interaction is claimed here.

Architecture reports one additional short test clone, no new product hotspot or
cycle, unchanged maximum CCN, and roughly 0.7% duplication. Broader query-based
fold shapes, Fold to Definitions parity, late-data viewport handling, PR refresh/
rename mapping and wider language/runtime acceptance remain open. Earlier claims
that initial body/PR defaults remain absent are superseded by this implementation.
