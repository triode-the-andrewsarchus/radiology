# Exact fold identities and overlapping headers

Swift keeps each distinct SourceFold byte range, sorts outer ranges before their
children, and selects the first eligible fold for each physical gutter row. Its
folded IDs do not collapse separate declarations just because their headers share
a source line. Rust previously discarded all but one candidate on a line and
stored collapsed state by that line, so a method could replace its type's fold.

Follow-up: [canonical-byte projection](rust-fold-projection.md) now implements the
partial-line and non-delimited header work left open at this checkpoint.

## Implementation

Fold extraction still canonicalizes exact duplicate spans by semantic role, but
retains overlapping candidates and sorts by start byte, then descending end byte.
`FoldStore.folded_ranges` is the sole collapsed-state collection. Manual toggles,
callable defaults, Fold to Definitions, prepared source results, exact-document
memory and navigation reveal carry the original half-open byte spans. Restored
state is intersected with current parsed identities. No second line-keyed state,
parser, runtime dependency or persistence format is added.

The measured viewport invalidates visibility from the exact collapsed set. The
single disclosure on a row chooses the first parsed range, with an interaction ID
and action carrying that range. Collapsing/expanding an outer type preserves its
child callable choices. Initial function defaults can collapse a method sharing
a row with an expanded type, while later methods remain visible.

PR expansion marks exact changed bodies/owners and propagates through ancestors
using byte containment. Same-row ancestors no longer alias children. Critical
Lens count batches and scope matching also use exact body spans, so separate
nested callables on one row retain independent counts and container folds cannot
borrow their counts. Exact source/revision validation and job retirement remain.

Existing broad navigation fixtures use test-only header assertions; those helpers
compute rows from the canonical state and store no compatibility state. New
identity regressions assert exact spans directly. Repeated app fold assertions
are consolidated into one fixture assertion.

## Verification

`cargo test --workspace`: 301 application tests pass, one external-Clang fixture
ignored; five reader-text tests and 30 doctests pass. Formatting, strict Clippy,
binary build and rustdoc pass. Coverage includes independent shared-header type/
method state, fresh callable defaults, manual outer toggles, exact file return,
Fold to Definitions, PR changed-child ancestry with a folded sibling, actual
pointer gutter toggles, and separate nested callable count publications. Existing
source/navigation/test-projection/review/cache/history tests remain green.

Architecture comparison against `before-fold-identities` adds 257 NLOC and eight
functions, lowers average CCN from 3.34 to 3.33, leaves maximum CCN 100, duplication
0.68%, clone groups 91, and adds no dependency cycle or product hotspot. The tool
still labels REGRESSION because six existing test-clone fragments touch modified
files. All six are present in the baseline: four app test pairs at 102/1579,
197/1150, 877/954, 1644/1682 and the two source-view/context-inspector fragments.
Two newly detected assertion clones during implementation were removed.

## Remaining fold rendering acceptance

This change establishes range identity and ownership. The viewport still projects
whole physical rows, so partial-line bodies and non-delimited header placement
(Python/Kotlin imports) do not yet match Swift's glyph projection. Initial PR
visibility, navigation and tests must be preserved when implementing that next
step. Broader grammar, scaling, accessibility, pinned-refresh mapping and visual
acceptance remain open. Full Swift/Rust parity is not complete.


Native evidence lives in `/tmp/radiology-fold-identity-native/`: final-prefixed
`defaults`, `outer-collapsed`, `outer-reexpanded`, `second-expanded`, `other-file`
and `restored` PNGs, plus `build-evidence.json` with capture hashes. The final
binary SHA-256 is `83e9bd3958b72a8db416e2ddd4e2254a635f275ba5591885bbc0e62b02faa799`,
based on `0511756` plus this implementation. Final PID 4117906 and both private
compositors exited zero. Pointer helpers were intentionally terminated. The
shared-header fixture preserves method defaults when toggling the outer type,
then restores the first method collapsed and second expanded after file return.
Earlier unprefixed captures use a pre-cleanup binary and carry no final-build
claim. The external driver uses the sibling compositor helper only for testing;
Radiology remains independently buildable and runnable.
