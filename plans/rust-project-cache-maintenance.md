# Project cache ownership, migration and maintenance

## Layout and compatibility

Latest project records now live in projects/<repository-sha256>/<project-key>.json
under the private index cache. The repository namespace hashes the canonical
attached root; the project key retains its existing root/strategy hash. A scoped
directory must not be a symlink, and the projects parent cannot escape private
storage. Shared cache directories cannot make one repository prune another's data.

A missing scoped record still reads the previous flat projects/<project-key>.json.
After a verified generation without failed projects, maintenance migrates owned
legacy files. Ownership requires both the canonical root in the decoded metadata
and the matching filename hash. Retained records move atomically; superseded
selected records and retired project identities are removed. Skipped strategies
remain retained, including their legacy fallback when a scoped copy already exists.
Unknown, malformed, unreadable, foreign and non-record files remain untouched.
The one-time .legacy-migrated marker is created last, without following symlinks.
This layout change does not relabel old formats; normal format-3/provenance
validation still applies when loading migrated content.

## Cooperative ownership and cleanup

Generation acquires a shared .project-cache.lock using the existing non-following,
nonblocking regular-file lease primitives. The guard owns the lifetime of reads,
staging and publication. Waiting is cancellable and bounded by the smaller of the
request timeout and five seconds; contention reports an actionable retry error.
File descriptors retain CLOEXEC. Compiler children do not write project records;
private compiler workspaces are not deleted by this maintenance pass.

After publishing, the generator releases its shared lease and tries an exclusive
lease. If another generator is active, cleanup defers until a later successful
generation. Transient inherited descriptors around fork/exec can also defer it;
there is no immediate cleanup guarantee. Tests exercise the next idle maintenance
opportunity rather than assuming drop-time convergence.

Maintenance preserves selected and skipped strategy records. It removes other
recognized lowercase SHA-256 JSON records in that repository's namespace and
abandoned .radiology-index-stage-* files. It never traverses or deletes symlinks,
unknown names, other repository namespaces, normalized revision indexes or viewed
inputs. Failure, stale-source and cancelled requests do not prune accepted records.
Cancellation is checked between keys/files and within the shared bounded JSON
reader used for the one-time legacy scan. Each legacy input has the existing
256 MiB read limit; SCIP payloads are skipped while decoding identity metadata.
Normal scoped cleanup needs only names and file types, not index deserialization.

Only cooperating current generators participate in these leases. Migration and
pruning are incremental filesystem operations, not one transaction. A cancellation
can leave a partial migration, which a later generation resumes without claiming
completion. Old unknown flat files and normalized revision history are deliberately
outside this ownership-based cleanup.

## Automated evidence — 2026-10-30

Five integration tests prove rename retirement after successful retry (with failed
runs retaining old records), unknown/foreign/link preservation in shared storage,
legacy reuse/migration without compiler execution, retention beyond the 32-project
run cap, and dangling marker protection. Real separate processes hold shared and
exclusive leases: cleanup defers, writer waiting times out or cancels, and the next
idle pass removes the abandoned stage while preserving accepted bytes.

Full gates pass 419 app tests (three ignored), eight shared tests, 30 doctests,
formatting, strict Clippy, build and rustdoc. After sharing the maintenance fixture,
all five focused tests, both opt-in real Rust/TypeScript tests and strict Clippy pass
again. Tests locate normalized indexes by their filename rather than accidentally
selecting the new lock file. Architecture adds no hotspot/cycle/clone growth; five
flagged test fragments are byte-identical to baseline before-project-cache-maintenance.

Logs: /tmp/radiology-cache-maintenance-{focused,tests,all-tests,clippy,final-clippy,build,doc,real-tools}.log.
Native migration/restart evidence follows. Other real tools/corpora, broader store
recovery, normalized revision retention and final UI/accessibility/scaling/font/
sibling acceptance remain part of full parity.
