# Explicit index reuse across checkout roots

Reference: all three IndexReuseTests groups and the remaining
cacheNamespaceSettingsEditsAndReindexKeepTheSelectedPipeline personality group.
The library now supports Swift's explicit cache/root/label handoff without global
cache searches. Together with the preceding personality checkpoint, compiler/
personality reconciliation reaches 9/35; 26 groups remain. Inspector/traits 15 and
final integrated/native/standalone/reuse acceptance remain separately required.

## Runtime contract

GenerationRequest.reuse_sources is an ordered Vec<IndexReuseSource>. Each source
has its owning attached root, private cache parent and display/provenance label.
Existing callers initialize an empty vector; standalone behavior remains scoped
to the current attached repository. A host can supply sources when it has an
explicit handoff, matching Swift IndexOpenOptions. Reindex (reuse=false) never
consults them. The type and request field are documented, including a compiled
configuration example; existing public struct literals must supply the new field.

The local project record is tried first. Explicit candidates follow in order,
using the same bounded reader and validation as local reuse. Candidate paths are
resolved with the destination viewed root and candidate attached root protected.
No candidate directory, lease or metadata is created or modified: reads open
complete atomic records, which remain valid if maintenance unlinks their paths.
A missing/incompatible candidate is skipped. Invalid candidates can fall through
to a later valid source; if none qualifies, generation retains the last cache
diagnostic and runs the selected tool normally.

Every match still requires the cache format, canonical owner, exact full-input
fingerprint, complete strategy/settings identity, probed tool version, valid
inventory membership and per-document source digests. These include sibling
inputs, path/type/mode changes and seL4's captured script provenance. No source
name, declaration spelling or path resemblance substitutes for those checks.

Requests with explicit sources stage reusable records into their own destination
namespace, replacing incompatible older local records. The existing generation
source recheck/cancellation gate precedes publication. Such a request refreshes
its destination record even when its own local cache supplies the match; ordinary
local reuse with no handoff leaves accepted project bytes untouched. This keeps
copy ownership on the existing staging path instead of adding an import writer.
Each record remains individually atomic, matching existing generation behavior.
The merged normalized cache also contains destination-owned SCIP/source digests.

Imported provenance is retained in the project outcome. A subsequent local reuse
keeps that origin rather than silently rewriting it to checkout, so persisted
mixed-source warnings survive. No cache wire format, dependency, UI state, second
parser or publication mechanism was added. The previous personality-only cache
load test helper is now shared at its existing parent fixture module, removing
repeated assertion plumbing across the two service suites.

## Assertion mapping

| Swift group | Rust evidence |
| --- | --- |
| digestsConservativelyCoverSiblingDependencyContext | Existing whole-input workspace identities include every path, bytes/type/mode; read_only_fingerprints and real Git snapshot verification check those identities. exact_projects_reuse changes a sibling TypeScript source and a Rust mode, invalidating both projects. explicit_sources_do_not_bypass changes shared/include.h and rejects all three Rust/TypeScript/Clang candidates. Rust uses its existing fingerprint representation rather than exposing Swift's ContentDigest helper API. |
| identicalBytesReuseAnotherRevisionsIndexWithoutRunningTheTool | Existing pinned_records_reuse uses actual Git checkout→After and same-tree new commit roots with a run counter. New explicit_checkout_reuse imports two documents from another attached root despite an indexer that would exit 9, replaces an incompatible local record, verifies the destination owner and exact buffer, then deletes the entire source cache. Both destination project reuse and normalized reload still work. The corrupt-record matrix now rejects format 2 as well as altered paths/digests and regenerates; no digestless legacy record is accepted. |
| changedDependencyContextRegeneratesAndPersistedMixedProvenanceStaysPartial | Existing sibling/mode tests count both compiler executions. New explicit multi-project case generates a changed-version Rust strategy while borrowing compatible TypeScript/Clang records, checks mixed-origin warnings after normalized reload, then reuses every record locally with the same warning intact. Shared-header changes and forced regeneration yield no reused outcomes. |
| cacheNamespaceSettingsEditsAndReindexKeepTheSelectedPipeline | The strengthened seL4 service test now hands the edited configuration to a different attached root, reuses the explicitly supplied working-tree cache without invoking that root's bootstrap runner, and restores its own normalized cache. Existing edited-settings generation, old-capture rejection, Default cache isolation and selected-pipeline cancellation remain. The fifth personality group is now reconciled. |

## Verification

Archive: ~/.local/state/radiology-qa/explicit-reuse-20261109-ae75dde/.
All 108 indexing tests pass (eleven existing opt-in ignores), including seven reuse
and nineteen personality checks. Strict workspace/all-target Clippy, normal build
and warning-free workspace docs pass. The generated IndexReuseSource page contains
the summary, field docs, configuration example and resolved reuse-field link.
The complete serial workspace gate passes 915 app tests (eleven existing opt-in
ignores), binary/font/text tests 2/13/23 and all 67 doctests (59 app, four font,
four text), including the new public configuration example. It verifies the new
runtime together with all recent source/presentation/personality additions.

Architecture +213 NLOC/three functions (one shared loader and two tests), no new
file/dependency/hotspot/cycle. Average/max CCN 3.43/132, 36 hotspots/one cycle and
all 125 normalized clone instances unchanged, zero additions/removals. Raw touched-
file warnings concern pre-existing clones and equal-ranked untouched Python ranks.
