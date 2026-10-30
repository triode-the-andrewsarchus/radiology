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
