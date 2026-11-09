# Context publication retirement and cache ownership

Swift reference: ContextModelTests.staleAsyncResultsNeverRender,
repositoryChangeAndResetDropEverything, aNewIndexRevisionRefetchesTheSameDeclaration,
and newSourceBytesWithTheSameIndexInvalidateCachedExcerpts. Comparison starts at
Rust `8cabeec`. This audit maps four further groups; fourteen of eighteen now have
mappings, with explicit-reference focus, scalar-window excerpts, availability and
expansion still requiring separate reconciliation.

## Runtime mechanisms already present

LatestRead owns cancellation plus the result receiver. Stopping/replacing it drops
the old receiver, so an old sender cannot publish into a newer request. Source
preview, metadata and reference scans already use that owner. QueryCache separates
index/repository/revision ownership and observes accepted source digests. App
repository acceptance resets the Context owners; index publication refreshes
against a new immutable index Arc. No new runtime gap was found in these paths.

The checks extend the existing real Swift rust-analyzer fixture app matrix rather
than creating a second lifecycle harness. All new functions live in a focused test
scenario module. Shared area-row loading and reset assertions avoid repeated setup;
existing read jobs, metadata-only publication, Reload Source and app frame helpers
remain the only mechanisms used. No test hook or production API is introduced.

## Assertion-level mapping

| Swift group | Current Rust evidence |
| --- | --- |
| staleAsyncResultsNeverRender | At Circle in lib.rs, actual cross-file preview and total_area reference scans have pending publication channels. Real metadata is prepared from the same captured inputs. The existing spawn_with_sender API transfers its publication endpoint out of the worker, holding completion without sleep or an occupied worker thread. After accepting shapes.rs at area's let keyword, the old sender rejects a deliberately late result. Under-caret facts/preview stay absent, area rows remain the accepted immutable Arc, and subsequent polls cannot replace them. Index/root/reset variants probe the same late-result boundary. |
| repositoryChangeAndResetDropEverything | The final scenario resets already published Circle metadata, its loaded preview, enclosing total_area and nonempty rows, then checks that Context is unavailable and empty. A separate reset retires pending channels; a late result is rejected and repeated polls keep empty state. Switching to a distinct real checkout with identical source bytes and the exact same producer index Arc creates fresh area rows, proving root ownership cannot reuse another checkout's result cache. This includes actual repository acceptance, not only direct cache-key tests. |
| aNewIndexRevisionRefetchesTheSameDeclaration | With the area declaration, source Arc and target unchanged, replacing the index with a freshly cloned immutable Arc starts new reference and metadata work. Accepted rows have a different Arc while the reader remains the same. A second case changes only Circle's index display name to Fresh Circle at the same occurrence; new metadata displays that name and rejects old Circle publication. Rust's index Arc replaces Swift's explicit indexRevision counter. |
| newSourceBytesWithTheSameIndexInvalidateCachedExcerpts | The real shapes source receives Swift's first-line // edited comment, then Reload Source accepts it while retaining the exact index Arc. Supplying the newly resolved impl caret gives the immediate memory preview with the edited header and fresh digest; independently loaded definition rows contain the edited line. Returning to another byte inside the same occurrence keeps the accepted rows and starts no query. Original bytes are restored and reloaded before repeating the matrix and the equal-source root case. |

## Scope details

The imported fixture has no captured source digests, matching Swift's legacy
unverified-index inputs; the scenario asserts that precondition. This does not
relax Rust's separate rejection of known-stale generated provenance. Actual byte
coordinates and definition text must still map before a preview/row is accepted.

The initial draft expected a preview at the old absolute caret after inserting
bytes on an earlier line. Reload Source deliberately retains an absolute byte,
whereas Swift's model test supplies a freshly resolved line/column after editing.
The test now supplies the same new caret before checking the refreshed excerpt.
That correction is a fixture-input correction, not a runtime navigation change.
The draft also imported an unused helper; using metadata-only publication for the
corrected caret removes that warning. No production behavior was changed to make
the tests pass.

The transferred endpoint probes result acceptance, not an expensive worker ignoring
cancellation or OS-child reaping. Existing source-preview/reference and process
cancellation tests retain those separate responsibilities. Pending preview/reference
jobs here are real reads/scans, retired through the actual app transitions.

## Verification and architecture

Archive: `~/.local/state/radiology-qa/context-lifecycle-20261109-8cabeec/`.
The corrected app matrix passes both scales, including original Context identity,
Unicode and fallback scenarios. Strict all-target Clippy passes. A full workspace
run passes 887 app checks (eleven existing opt-in ignores), two binary, thirteen
font, twenty-two text checks and all 66 doctests. The stronger loaded-reset
assertions were added afterward and pass in the final both-scale app check; final
strict Clippy also passes. Compilation uses -j 1 with serial tests; the ordinary
half-core jobs=16 configuration is unchanged. Product source is unchanged, so
build/docs/resource evidence from `8cabeec` still applies.

Measured addition: 224 NLOC/five test functions/one scenario file. Average/max CCN
remain 3.42/132, with 36 high-complexity sites and dependency cycles unchanged.
The actual draft setup clone was consolidated into shared fixture operations.
The complete normalized 125-instance clone multisets are identical (no additions
or removals). Raw flags concern an
unchanged existing app/tests.rs clone and equal-ranked unmodified Python items.

The four remaining ContextModel groups are not claimed complete by this audit.
In particular, expansion should be checked across index replacement: Swift's
declaration identity includes indexRevision, while Rust's current summary owner
contains source identity and selection range. This key difference needs an actual
disclosure-state check before treating it as equivalent.
Reference/call-tree/palette, source/syntax, compiler/indexing and inspector suite
reconciliation and final native/standalone gates remain indexed separately.
Android's SDK/license gate and known parallel timing/linking limitations persist;
full assistive accessibility and actual sibling-app adoption remain deferred.
