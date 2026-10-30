# Pinned After semantic indexing

PR reviews can generate and reload semantic indexes for their exact pinned After
commit. Before remains source-only. This extends the existing isolated checkout
indexing pipeline rather than adding another indexer implementation.

## Inputs, isolation and lifetime

GenerationRequest carries an optional owned ReviewSnapshot. Its root must match
that snapshot's head tree. Request inventory uses raw pinned Git entries, not
mutable index flags, untracked files or browser filters. Empty gitlink directories
are recorded as omitted inputs. The existing exact tree verifier checks bytes,
symlinks, executable modes, HEAD, inventory and private Git ownership.

Generation copies a pristine baseline and verifies its original Git blob IDs with
IndexingWorkspace::verify_snapshot. Each project still runs in another disposable
workspace with existing input limits, tool discovery, timeouts, compiler adapters,
SCIP validation, coverage merging and source digests. Indexer artifacts cannot land
in the attached checkout or cached PR tree. A final tree/source proof precedes
publication; modified pinned inputs fail instead of becoming a stale PR index.
The request retains its snapshot lease through worker retirement and cancellation.

## Persistent identity and reader behavior

Normalized-cache identity uses the private object store plus pinned head commit.
The random tree path is excluded, allowing reuse after eviction or verified tree
replacement. Checkout keys remain unchanged. GenerationReport records the exact
revision; cache restore validates it, logical repository identity, inventory,
fingerprint and document provenance. A pinned fingerprint mismatch fails restore.
Existing format-1 checkout entries remain compatible via optional revision metadata.
This does not yet add per-project/version-aware incremental reuse.

Entering a review automatically attempts its cache, with no fallback to an attached
checkout import. Generate After Index and Reload Index are available in review mode;
status identifies an After index and exposes its commit. Generation always targets
After even when Before is displayed. Existing Before guards suppress semantic
navigation there. Switching scope retires older index jobs; failure or cancellation
retains accepted semantic data for the current scope.

## Evidence and remaining acceptance

On 2026-10-30, full gates pass 384 app tests (one ignored), eight shared reader
tests, 30 doctests, formatting, strict Clippy, build and rustdoc. Added tests cover
isolated pinned generation, source digests versus dirty checkout bytes, cache reuse
across tree replacement, concealed modifications, mismatched roots, live-child
cancellation with lease release, and UI commands/cache loading without checkout
import leakage. Indexer fixtures emit controlled SCIP; this is not a new real
compiler integration claim.

Architecture baseline `before-pinned-index`: no new complexity hotspot or cycle;
clone groups remain 90. Comparison labels a regression for five clone groups in
touched test files. All ten source fragments were compared against HEAD and are
exact unchanged baseline text. New duplicate indexer fixtures share one helper;
baseline preparation and verification form one cohesive request stage.
Logs: `/tmp/radiology-pinned-index-{all-tests,clippy,build,doc}.log`.

Native Generate/Reload acceptance is pending. Full parity remains open: complete
provisioning/version/project cache lifecycle, precise PR refresh restoration,
broader corrupt-store recovery, and final visual/accessibility/scaling and sibling
reuse acceptance.
