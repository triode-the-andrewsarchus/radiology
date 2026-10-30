# Cancellation through semantic source consumers

Implemented 2026-10-30. Definition/reference previews, reference collection,
incoming-caller traces and Critical Lens graph reads pass the owning job's flag
and deadline through SourceQueryInputs/SourceRead into pinned Git blob reads.
The existing shared worker pool and process-group runner remain the execution path.

## Behavior and ownership

SourceQueryInputs::read now requires cancellation and deadline arguments, including
its displayed-buffer fast path. Valid displayed bytes still win over storage and
return the same Arc. Other paths use the captured checkout or pinned review with
the existing source-size, path, encoding and digest validation.

The source-read timeout policy is shared with SourceLoader. A reference query and
incoming trace keep a 30-second budget across their reads; each preview and graph
frontier captures its own budget. An annotation sweep retains its existing
per-frontier evaluation and progressive publication. Budget checks are cooperative:
Git subprocesses are interrupted and reaped. [Native parse/query stages](rust-native-analysis-cancellation.md)
now have progress callbacks; filesystem syscalls and bounded helper operations
finish their current operation before the next check.

Reference collection checks retirement/expiration before turning an ordinary I/O
error into an unreadable-file coverage entry, and checks again before returning
success. A cancelled final file therefore cannot complete as successful coverage.
Genuine unreadable source still reports the existing partial-coverage explanation.

Incoming trace caches check the request around reads/parses. Critical graph reads
skip the file cache when the request retires or expires; the enclosing summary
also checks the guard before storing a result. Cancellation cannot create a
persistent unreadable/no-body cache entry. Facts already accepted from the same
immutable configuration stay reusable. A fresh evaluation of a cancelled file
can load its real body normally.

No user-facing controls changed. Existing preview/reference/caller/annotation/hover
job replacement now retires the underlying pinned read as well as its publication.
Synchronous SourceRead::read remains available with its bounded default; all
current application analysis consumers use the cancellation-aware path.

## Runtime verification

A Linux subprocess test uses a real local ReviewSnapshot and semantic coordinates
bound to its exact After bytes. Only the isolated test child's PATH contains a
private Git wrapper. The wrapper holds `cat-file blob` with a real sleep process;
repository preparation and other Git operations execute the installed Git.
The parent test's environment is untouched and no network is needed.

For preview, references, incoming callers and Critical Lens, the fixture confirms
that the pinned-blob PID is live, signals the same AtomicBool supplied to the
semantic API, requires a cancellation error within two seconds, and checks that
the PID no longer exists (ESRCH). Graph recovery then loads a real body with a
fresh flag. The reference fixture has a single indexed file, covering cancellation
of the final source read. Separate tests cover expired/cancelled cache misses,
displayed-buffer guards and Arc reuse.

Reproduce with:

```sh
cargo test --lib source::preview::cancellation_tests -- --nocapture
```

Final runtime log: `/tmp/radiology-analysis-live-cancellation.log`; manifest:
`/tmp/radiology-analysis-source-manifest.json`. Confirmed live-then-reaped PIDs:
preview 298656, references 298661, callers 298666, Critical Lens 298671.
Unit-test executable SHA-256:
`b12cbd3cfd673ab4e142f2b7c048ea78f2e48cf84117730d2f389557be88b051`.
Runtime-log SHA-256:
`cf96f311795171ea36a0430de2ff071b69a030d1aa021e5ba04b08de03ab1001`.
This is semantic API/process verification; broader native PR UI acceptance and
pinned-After index provisioning remain on the parity queue.

Full workspace checks pass 342 app tests (one external-Clang fixture ignored),
eight reader-text tests, 30 doctests, formatting, strict Clippy, build and rustdoc.
Cargo still uses 16 jobs on the confirmed 32-logical-CPU host.
Architecture baseline `before-analysis-source-cancellation` adds no hotspot, cycle
or clone group. Duplication decreases. The tool flags three touched-file fragments;
all three are present verbatim in the saved baseline: frontier/worker tests,
hover/preview job setup and incoming/reference test fixtures. They were not added
by this change.

[Nested SCIP decoding and map construction](rust-scip-cancellation.md) now support cancellation.
Remaining work includes normalized-cache JSON interruption, PR preparation/fetch
cancellation and retention, index provisioning/provenance, source/graph/context
revisit caches, PR-refresh mapping and final visual/accessibility/scaling acceptance.
