# Final runtime acceptance after review/index coordination

Audited production source: a5fd157. The independent exported executable SHA-256 is
109b41ee2f23193de873230d332799468088821974bdaad03e74010c430e23ae.
Both native PR before manifests match that executable, and the source/stress
collectors verify unchanged binary and source hashes after their complete flows.

## Workspace and independence

The serial locked/offline workspace gate passes 943 app tests, twelve opt-in
ignores, two binary checks, thirteen font checks, twenty-three text checks and
68 doctests. The subsequent full locked/offline run with sixteen build jobs and
sixteen test threads also passes with those same counts. Its complete log is
~/.local/state/radiology-qa/final-parallel-20261109-a5fd157/workspace.log.
Strict Clippy, formatting and normal build pass for the runtime fix.
A new git archive and target-coordinated directory rebuild the committed workspace
with the configured sixteen jobs. Independent full-reader and asset/text hosts
pass locked/offline; metadata keeps local path dependencies inside the export.
The copied app and both hosts run successfully while the entire export source
directory is unavailable. No sibling product code participates in those builds.

Archive: ~/.local/state/radiology-qa/launch-index-coordination-20261109-8f48ec5/.
verification-state.json records complete/passed/exit-zero and the source/binary
identities. workspace.log and export-build.log retain the actual command output;
host-locked.log and asset-host-locked.log retain independent consumer output.

## Integrated review

The native-pr subdirectory contains native-1.0-final and native-2.0-final accepted
flows. Each runs a private real Git origin and actual scip-clang 0.4.0; gh metadata
is controlled JSON. Export sources are unavailable for the entire native run.

Cold startup performs one driver invocation for pinned After, avoiding the dirty
attached checkout. Before suppresses semantic results. Added, deleted, renamed and
binary rows, exact retained Copy after binary rejection, moving head/base,
definition navigation, same-byte empty-commit reuse and warm tool-free restart
pass. Seven successive heads pressure both normalized-index and managed-tree
retention to three. After disabling the compiler, an evicted head rematerializes
source without fabricated semantics; restoring the tool and explicit generation
restores semantic navigation. The pre-restoration counter is six driver calls/
33 total tool calls; final restoration brings it to seven/38. Before manifests
match the accepted binary; source hashes, HEAD, refs and Git status remain exact.
All owned app/restarted-app/compositor exits are zero. Fresh head-two-definition
and rematerialized-definition captures show bounded panes and exact source.

The earlier two-driver cold attempt remains explicitly unaccepted in
~/.local/state/radiology-qa/final-native-pr-20261109-8f48ec5/.
The acceptance condition was preserved and the production coordinator repaired.

## Source and stress

Archive: ~/.local/state/radiology-qa/final-source-flows-20261109-a5fd157/.
corpus/native-{1.0,2.0}-after-final and stress/native-{1.0,2.0}-after each retain
acceptance.json, independently checked accepted.json, exact clipboard hashes,
resource observations and native captures. All four complete runs pass with
export sources unavailable and every owned app/compositor exiting zero.

Representative source checks cover C/C++, malformed declaration recovery,
current-function folding/expansion, theme changes, wrapping and exact Back/Copy.
Stress covers a 5,850,053-byte Unicode/CRLF source, a 60,002-line source, modal
Code Path import during loading, exact whole-source Copy, LONG_END find,
wrap/zoom/reset preserving selected bytes, Go To Line 60001, Back and a superseded
source request retaining the accepted anchor. Five representative and eight stress
clipboard payloads per scale are verified byte-for-byte. The observed stress
resource samples are retained, without inventing a universal latency threshold.
Fresh expanded C++ and line-60001 captures confirm source, selection and panes.

QA drivers import sibling compositor helpers read-only; that is a test harness
dependency. The separate archive/build/consumer checks prove product independence.
Sources are restored and short private runtime directories removed after each run.

## Scope

These results refresh the affected runtime paths after a5fd157. They do not claim
Swift/AppKit execution on Linux, authenticated remote GitHub acceptance, every
external corpus, SDK-backed Android compilation or full accessibility. Earlier
actual compiler, seL4, IME and portal acceptance retain their documented scopes.
The complete original/upstream requirement audit remains separate.
