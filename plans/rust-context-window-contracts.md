# Context excerpt windows and semantic coverage

Reference: ContextModelTests.referenceTextIsWindowedOnScalarBoundariesAndBounded
and availabilityDistinguishesRepositoryIndexFromFileCoverage. Comparison begins
at `f7211aa`. These are the last two of eighteen ContextModel groups; broader
reference/call-tree, source/syntax, compiler, inspector and final acceptance remain.

## Display correction and exact source ownership

The long-Unicode Swift fixture reproduced missing visible omission markers in
Rust. The shared preview_line path previously bounded text at 2048 bytes and
exposed cropping only as a flag/tooltip. It now uses a 400-byte total display
budget, places an ellipsis at each omitted edge, clamps source boundaries to UTF-8
scalars and shifts emphasis by the leading marker's byte length. Short lines stay
whole even when their occurrence is near the end. The Rust total includes markers;
Swift permits 400 source bytes plus an ellipsis (403 bytes). Both remain within
the Swift asserted bound. This is bounded display data, not a source rewrite.

The existing shared path supplies reference rows, local definition lines and
preview snippets. No second formatter, worker, cache, public field or coordinate
mapping was introduced. Canonical indexed ranges, source locations and accepted
SourceBuffer bytes remain unchanged. Existing renderers consume display text and
its adjusted emphasis. Narrow sidebar text can still truncate at the available
width, as Swift's CodeSnippetView does; this change does not assert that every
long occurrence fits physically in a narrow pane.

The exact Swift source repeats é🚀x thirty thousand times before target on line 2.
The real reference query retains the exact target navigation slice and produces
leading omission, bounded text and exact highlighted target. Prepared reference
context shares the same line. Every context line is bounded without replacement
characters. Definition excerpt construction explicitly supplies no declaration
end, matching Swift's constructor call: it returns two lines with bounded,
scalar-safe text. The original source is unchanged. An edge matrix independently
checks leading-only, trailing-only, both-edge and uncut short-line cases.

## Coverage assertion mapping

The existing availability matrix already checks a retained nonempty index during
replacement generation, cache failure, import failure and generation error. An
uncovered file remains FileNotIndexed; a covered accepted source is Available.
Empty/absent indexes expose indexing while running and supplied failure/no-index
reasons while idle. This checkpoint adds the exact No supported projects. reason,
File not indexed / No semantic index header labels and absent Available explanation.

Swift's intermediate mapping state is represented by Rust's atomic accepted
source/index-coordinate publication plus separate metadata loading, rather than
a duplicate Mapping enum. The existing Context navigation pending-publication
checks hide old semantic facts until the new source is accepted; see
[rust-context-navigation-contracts](rust-context-navigation-contracts.md).
Known source provenance additionally rejects stale bytes and Before revisions;
legacy indexes without captured provenance retain their documented usability.

## Verification and architecture

Archive: ~/.local/state/radiology-qa/context-window-20261109-f7211aa/.
The two new preview assertions first failed against the missing marker behavior.
After repair, all 15 source-preview checks, nine reference-result checks, two
expanded availability checks and three shared preview renderer checks pass.
Strict all-target Clippy, normal workspace build and warning-free docs pass.
The complete serial workspace gate passes: 889 app tests, eleven existing opt-in
ignores, binary/font/text checks 2/13/22 and all 66 doctests. This also validates
the preceding disclosure change. Full compilation uses -j 1 and tests run with
--test-threads=1 for the documented parallel-run limitations; ordinary Cargo
configuration remains the requested half-core 16-job setting.

Architecture measurement against before-context-window-20261109 adds 97 NLOC and
two test functions, with no new file or runtime helper. Average/max CCN stays
3.42/132, high-complexity sites stay 36 and dependency cycles are unchanged.
Complete normalized clone multisets stay at 125 with zero additions/removals;
raw touched-file flags concern an existing preview fragment. The archived
architecture.json records both vectors and the complete multiset delta.

No Swift source, font scale or typography changes. Full accessibility and actual
little-screen-clip integration remain deferred. Android SDK-backed success and
previously documented parallel-run limitations remain open.
