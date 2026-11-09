# Automatic index opening and refresh lifecycle

Swift starts its semantic service on repository open, publishes cached data first,
and regenerates missing/stale work. Rust previously loaded/imported only, leaving
all generation explicit. That behavioral gap is now repaired in the standalone
app. This checkpoint advances the IndexingTests reconciliation but does not count
its whole ten-group suite as complete; invocation diagnostics and other remaining
assertions still need their own proof. Compiler/personality remains 17/35 mapped.

## Behavior

Opening or refreshing a checkout, accepting a pinned After review and Reload Index
capture the selected personality settings, inspect actual build inputs and load
the normalized cache on existing workers. An exact cache with no failed projects
stops there: no version probe or compiler executable is needed. Known Partial
coverage and compiler diagnostics remain Partial; they do not cause retry loops.

A stale cache is published before regeneration, with its original per-file source
digests. Unchanged files retain semantic navigation; changed files cannot borrow
old coordinates. A current cache with failed projects also retries those projects.
Automatic generation enables existing exact-project reuse, so successful projects
can survive a failed sibling's recovery. The continuation is consumed before
starting generation: a failure is reported once, without automatic frame-by-frame
retries. Reopen/Reload can retry. Explicit Reindex still forces regeneration.

For Default checkout opens without a usable normalized cache, the existing legacy
index.scip/.scip importer runs first. Its explicitly unverified data stays usable
if subsequent generation fails. seL4 and pinned reviews do not borrow checkout
imports. Unsupported/no-project repositories remain plain structure/import reads
rather than copying a workspace for an empty generation plan. Invalid settings
still fail visibly with no default-pipeline fallback. Missing tools produce normal
project failures and installation remains an explicit user action.

The existing personality owner keeps its Load intent through cache acceptance and
adds one Import phase. One optional cache-warning string carries a failed cache
read through import/generation into the final report instead of losing that cause
when the generator starts. Actual current inspection owns the displayed plan;
a restored older cached plan cannot overwrite it. There is no new worker, queue,
index format, source copier, parser or persistence path.

Cancellation retires the pending continuation as well as its current worker. The
cache-loading state is captured before changing cancellation status, so the actual
cache worker is stopped even when the owner now reports Cancelled. Cancellation
of generation also retires an import. Explicit installation cancels pending open
work before reserving the installation flow; successful installation still starts
its own forced generation. Repository/review/personality replacement retains the
existing worker/token retirement. Generic “Index loading cancelled” text now fits
all loading phases and identifies retained prior data when present.

## Verification

Seven new actual-app cases use the exact two-document Rust SCIP/source corpus and
the existing instrumented indexer fixture. They prove first-open generation,
current-cache reopen without an executable, intermediate stale publication,
failed refresh retaining both old in-memory data and cache bytes, successful
recovery, missing-tool recovery without installation, active-process cancellation/
root replacement, cancellation at both cache/import boundaries, failed-project
retry with successful-project reuse and explicit-install precedence.

The broad app run exposed three previous manual-only assumptions. Generated
navigation now begins from automatic publication; Kotlin review setup no longer
issues a redundant forced run; installation cancellation compares the report
that opening already produced. The existing full generation-command test now
checks retained automatic data, failed stale refresh and preserved corrupt-cache
diagnostics. Focused lifecycle, generation-command, Kotlin and provisioning cases
pass after those corrections. Strict all-target Clippy, build and warning-free
docs pass. The complete serial workspace gate passes 926 app tests (twelve
opt-in ignores), binary/font/text checks 2/13/23 and all 67 doctests (59+4+4).

Actual private Wayland runs pass at 1× dark/1280×900 and 2× light/1024×768 logical
size. Each verifies first-open two-document generation, a restart with the tool
removed and no extra invocation, then a source edit followed by failed stale
regeneration retaining the exact cache. Go to Line and Copy All preserve original
source bytes before and after failure. Images show generated and retained-data
statuses with the source visible. Authored source and executable hashes remain
unchanged except the deliberately edited dependency. All six app launches and both
compositor pairs exit zero. These use controlled tools and recorded compiler data,
not a fresh real Rust compiler acceptance claim.

The harness imports existing private-desktop helpers from little-screen-clip;
no sibling source or runtime integration was changed. Initial harness failures
were an overlong Unix socket path and a helper filter fixed to the sibling app ID.
Both terminal attempts were cleaned up; the accepted rerun uses short private
runtime directories and exact owned PIDs. Those runtime directories are removed.
Evidence: ~/.local/state/radiology-qa/automatic-index-20261109-8569474/,
including native-{1.0,2.0}/accepted.json, six captures and exit-statuses.json.

Architecture measurements remain at 125 clone instances, zero normalized clone
additions/removals, 36 high-complexity functions and one cycle; average/max CCN
3.43/132. One narrow runtime advancement method reuses the existing state owner.
Measured delta is +362 NLOC/net five analyzed functions and one test source file.
The existing runtime owner gains one advancement method; the seven new test cases
share one fixture helper. The raw touched-file clone/ranked Python alerts concern
unchanged normalized clones and untouched equal-ranked Python functions. Full
measurement and run logs are retained beside the native evidence.

## Remaining lifecycle reconciliation

The IndexingTests suite still has assertion-level work. The later
[invocation-diagnostics checkpoint](rust-invocation-diagnostics.md) now persists
actual command/cwd for failed runs and exposes a copy action.
The later [adapter-validation checkpoint](rust-adapter-validation.md) now rejects
unknown configured IDs during capture, closing the silent empty-plan gap found
here. Both concrete feature gaps are now repaired; remaining suite assertions still
need their full reconciliation.
Kotlin/PR compiler assertions, inspector/traits, final integrated/native/standalone/
shared-host acceptance and Android SDK-backed success remain separately open.
Full accessibility and actual sibling adoption stay deferred.
