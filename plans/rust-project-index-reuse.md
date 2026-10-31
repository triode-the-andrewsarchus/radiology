# Rust project index reuse and provenance

## Contract

GenerationRequest now carries a stable attached_root separately from the viewed
input root, plus an explicit reuse policy. Reindex (reuse=false) always invokes
all available selected tools. Incremental requests probe each tool in its isolated
project context, then reuse only a matching record. Cache-only loading remains
independent: it runs no tools and validates source bytes/revision as before.

Each project record matches the canonical attached repository, full strategy,
actual probed tool version, normalized-cache format and entire input fingerprint.
The fingerprint includes every input path, byte digest, type and mode, including
sibling dependencies. This follows Swift ContentDigest's conservative whole-repo
rule; no unmodeled dependency independence is assumed. External dependency or
compiler-environment changes remain reasons to force Reindex. An unavailable tool
cannot qualify version-aware reuse; existing cache-only navigation remains usable.

The latest record for each repository/strategy lives under the private index cache's
projects directory. A changed version or input context replaces that record; it
never creates a new project file for each commit/version. Records are scoped by
canonical attached identity even if a host shares a cache directory. Checkout and
pinned After can reuse each other's exact inputs; Before remains source-only.
Python's project-name argument now uses the stable attached repository, preserving
its identity across private snapshot roots.

## Storage and publication

Project records share the existing bounded cancellable JSON reader/writer, 256 MiB
per-file limit, document/range validation, no-follow input reader and atomic rename.
Loaded records also validate project membership, document coverage and every source
digest against the pristine workspace. Coverage is recomputed on reuse. Corrupt
records trigger regeneration with a retained cache diagnostic, rather than an
unverified semantic publication.

Fresh records serialize to temporary files without copying another SCIP model.
Source provenance is verified before publishing staged files. Request failure, cancellation
or stale source drops pending files and preserves accepted records. A failed
project retains its own previous record while successful projects may publish. Each record is
published atomically; publishing several projects is not one filesystem transaction.
Concurrent writers can replace a latest record, but context/version qualification
prevents reuse of mismatching data. The projects directory cannot resolve outside
the private cache, including through a symlink into viewed sources. Both attached
and viewed roots are protected before workspace creation.

Outcomes record the original generating checkout or full After commit when reused.
The merged index persists those origins. A run mixing generated/reused projects or
different reused origins is partial, with the same cross-project-reference warning
as Swift and an explicit Reindex recovery. Other partial coverage/build information
is preserved. A storage failure keeps session data usable and reports a warning.

## Evidence — 2026-10-30

Five backend tests cover unchanged reuse without indexer execution; changed tool
versions, sibling bytes and executable modes; forced regeneration; partial coverage;
corrupt paths/digests; shared-cache repository isolation; missing tools; cancellation
with an already-staged project; and symlink protection. A real Git fixture exercises
checkout-to-After reuse and same-tree new-commit reuse through production review
preparation, plus persisted originating-commit provenance.

Full workspace gates pass 412 app tests (two ignored), eight shared tests and 30
doctests; fmt, strict Clippy, build and rustdoc pass. After the final test extensions, all five focused reuse tests and the opt-in real
TypeScript package test pass, including a reuse pass before cache-only reload.
Architecture baseline before-project-index-reuse: no new hotspot/cycle/clone growth;
prepare_and_run is simplified by separating execution from preparation/reuse. The
remaining reported clone fragments are unchanged baseline test setup.

Logs: /tmp/radiology-project-reuse-{focused,all-tests,clippy,build,doc}.log.

## Reader controls and native acceptance — 2026-10-30

Index Projects offers Generate/Update Index (After in a review) with exact/version
reuse and a separate Reindex button that always bypasses it. Install and Index
continues to force regeneration after installation. Project rows show the original
generating checkout or full commit; mixed-origin warnings and cache diagnostics
remain visible. Outcome rendering is shared instead of expanding row complexity.

App integration tests verify Update can succeed through reuse when the indexer
still probes correctly but any actual run fails; Reindex attempts that run and
retains the accepted reader on failure. The pinned-review command test verifies
After origins. Full UI gates again pass 412 app tests (two ignored), eight shared
tests, 30 doctests, fmt, strict Clippy, build and rustdoc. Logs use the prefix
`/tmp/radiology-project-reuse-ui-{tests,clippy,build,doc}.log`.

A private Wayland session at 1280×900, scale 1, dark appearance verifies pointer
controls: initial generation, unchanged reuse without another compiler run, actual
version-probe change causing regeneration, forced failure retaining accepted reader,
reuse recovery even while the compiler mode fails, malformed project-record recovery
with a visible warning, Reindex clearing that warning, and restart retaining the
reused checkout origin without compiler execution. Exactly five indexer attempts
occurred. All read the dirty fixture bytes in private workspaces; HEAD/status/source/
manifests remain unchanged. Temporary workspaces and stages are removed. Reported
app/compositor shutdown codes are zero.

Evidence: `/tmp/radiology-project-reuse-native/acceptance.json` and SHA-256
`manifest.json`. Key captures: `updated-reused-checkout.png`,
`changed-version-regenerated.png`, `forced-failure-reader-retained.png`,
`corrupt-record-regenerated-with-warning.png`, `cache-recovery-details.png`,
`reindex-clears-cache-warning.png`, and `reopened-persisted-reuse-origin.png`.
The misnamed `forced-recovery-fresh-index.png` precedes the successful force action
and is not evidence of warning clearance. Controlled local tooling tests lifecycle
and UI; the separate real TypeScript test covers real compiler output. The sibling
capture helper is QA-only; the app remains standalone. Native mixed-project and
pinned-origin visual acceptance remain open. The generic indexer failure text still
exposes Option-style status formatting; installer failures already use plain status.

This latest-record design avoids revision/version file growth, but does not yet
sweep records for removed/renamed project identities or qualify external build
environments beyond the indexer version. Reindex is the recovery for external
compiler/dependency changes. No full parity completion is claimed.
Broader real-tool/corpus, recovery, UI/accessibility/scaling/font and sibling reuse
acceptance remain part of the full parity goal.
