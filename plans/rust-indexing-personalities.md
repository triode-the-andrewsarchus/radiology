# Rust indexing personalities and seL4 parity

Reference: [upstream contract](indexing-personalities.md) and
[compiler matrix](sel4-indexing.md). Settings, service and standalone app workflow
are implemented. Real 67-profile Linux compiler/service acceptance now passes;
native real-matrix controls/navigation now pass at both display scales.

The [assertion reconciliation](rust-personality-contracts.md) and later
[explicit handoff](rust-index-reuse-contracts.md) now map all six personality/UI
groups, including cross-checkout seL4 cache reuse with edited captured settings.

## Settings and resource checkpoint

`semantic::indexing::personality` exposes Default/seL4 identities, immutable
Settings and a blocking PersonalityStore. It uses the exact packaged Swift TOML
and five Python resources; there is no duplicate configuration matrix or Python
pipeline. Neither loading nor parsing launches an interpreter or indexes source.
The same library remains suitable for standalone and later sibling hosts.

The existing toml_edit parser validates TOML syntax. Typed root reads enforce
schema 1, selected identity/engine, decimal project_limit 1–256, clang_jobs 1–64,
single-line quoted strings and string arrays. Arrays may span lines and contain
comments. Unknown/duplicate root values, dotted/quoted root keys, malformed types,
multiline strings, nondecimal numbers and unsupported identities fail. Default's
empty adapter allowlist means all; seL4 requires exactly scip-sel4. Bootstrap
Python candidates retain their configured order or the reference defaults.
Nested recipe semantics remain the packaged runner's responsibility. Rust parses
the whole TOML document, so malformed nested TOML fails earlier than Swift's root
scanner. It never guesses replacement settings.

Settings own captured TOML bytes and SHA-256. Default hashes its complete TOML;
seL4 additionally hashes each packaged script name, a NUL delimiter and its bytes,
in the upstream provenance order. User edits during a run cannot change a prior
Settings value. Metadata/limits and original file location remain available to
hosts through immutable accessors. Merely obtaining a store path performs no I/O
and works for invalid-file recovery.

A store loads bounded regular UTF-8 files of at most 2 MiB through the shared reader.
It resolves the app directory and rejects locations inside any supplied protected
attached/pinned root before writes. Missing templates use create-if-missing with
synced temporary files and nonclobber publication. Concurrent first-use callers or
an editor winning that race cannot have their file replaced. Invalid user files
survive. Symlink/nonregular settings fail; cancellation is checked around I/O and
after parsing. Hosts must supply all protected source roots and perform blocking
I/O on their existing workers. The shared atomic storage staging function also
continues to serve existing settings replacements.

Seven behavioral tests cover both templates and all 67 profiles, provenance,
reference root syntax, invalid identity/type/limits, user-edit preservation,
immutable prior bytes, concurrent first use, winning-editor preservation, source
protection, pre-cancellation, symlink/directory rejection and oversized files.
No UI/native workflow acceptance is claimed by this library checkpoint.
Evidence: ~/.local/state/radiology-qa/personality-settings-20261102-9ba9448.

## Integration and remaining acceptance

1. Implemented: captured settings reload on open/reload, selection and Reindex;
   SessionStore persists canonical attached-checkout choices shared with PRs. Invalid
   files fail visibly with Show Settings File and Reload Settings recovery.
2. Service integration is implemented: Default's adapter allowlist/project limit
   and Clang jobs drive actual planning/commands. Captured digest participates in
   strategy equality, normalized cache validation and project reuse. Default keeps
   its namespace; seL4 uses personalities/sel4, including revision-keyed records.
3. The distinct scip-sel4 service strategy and private shared-runner invocation are
   implemented, backed by the existing managed Clang tool. Controlled runner tests
   verify captured TOML/scripts, source-only publication, cache isolation/reuse,
   cancellation and cleanup. Real matrix/provisioning acceptance now passes after correcting shared Python
   PATH precedence on Linux.
4. Implemented: Index Projects picker/explanation/file action uses existing menu
   controls and styles, preserves reader/caret/history on switches and reuses the
   Clang Install and Index path.
5. Service/app and native1×/2× checks cover settings/cache transitions, recovery,
   cancellation, PR identity and reader retention. The full 67-profile recipe against the pinned seL4 checkout now verifies
   original coordinates, conditional alternatives, all modes, source integrity,
   Python provisioning, forced Reindex and warm caches. Native real-matrix
   1×/2× Reindex/navigation/Back/Copy/restart now passes.

The seL4 matrix is finite and Partial remains intentional. Full assistive
accessibility and actual sibling integration retain their agreed deferred scope.

## Settings checkpoint validation

The workspace passes 741 app tests (nine opt-in tests ignored), two binary tests,
13 font tests, 21 text tests and 58 doctests. Strict lint/check/build/rustdoc,
format/diff, resource smoke and rendered Settings/store/resource pages pass.
Against baseline before-indexing-personalities at9ba9448, the architecture vector
adds574 NLOC/39 functions/four files while average/max CCN3.38/132,33 functions
above15,126 clone groups and one cycle stay unchanged. The clone-identity audit
has no additions or removals. Remaining raw comparator rank variation concerns
untouched Python functions. The initial parser hotspot was resolved by separating
pipeline provenance and root identity validation from capture/storage ownership.

## Generation and cache checkpoint

GenerationRequest now owns optional immutable Settings. None preserves existing
host behavior and wire records; the standalone app now supplies captures from its
settings worker. Captured Default filters adapters before applying the project cap and
sets actual Clang jobs. Every selected/skipped strategy retains the captured digest.
Stable project IDs stay unchanged; full strategy equality rejects incompatible
settings for project reuse. Normalized cache restore reconstructs the stored input
plan with the request's settings; a mismatched capture cannot publish as a fresh
cache. Existing unconfigured records deserialize with an absent digest.

seL4 explicitly chooses scip-sel4 and shares Clang tool discovery/provisioning.
Its bootstrap Python resolves configured candidates in order, including absolute
and ~/ paths, without requiring a search path for explicit executables. The shared
five scripts and captured TOML are staged in private run output. The run directory
sits outside the copied source tree because the Python runner inventories source;
putting its changing artifacts inside that tree would invalidate its digest. Python
bytecode is disabled and TMPDIR points to the private run. Persistent Python tools
must remain outside attached, viewed and copied source roots.

The existing source/snapshot checks, process deadlines, cancellation, normalized
inventory filtering and cleanup surround the runner. The request owns environment
assembly separately from execution. Generated headers do not become source
navigation destinations. seL4 normalized/project caches and writer lease live under
personalities/sel4; Default records are unchanged. Escaping namespace symlinks fail
before writes. Nested namespace creation occurs before opening its shared lease.

Seven new tests cover planning order/digest/coverage, actual Clang jobs, settings
edits invalidating normalized and project reuse, private seL4 resources and generated
header filtering, Default/seL4 cache isolation and tool-free warm reload, live runner
cancellation/retry, missing bootstrap/unsafe tools, escaping cache namespaces and
absolute/home program lookup. Existing managed installation tests also exercise the
seL4 alias of Clang. The runner fixtures are controlled stand-ins, not a claim that
all real compiler profiles pass.

Full workspace:748 app tests pass, nine opt-in tests ignored; two binary/13 font/
21 text tests and58 doctests pass. Strict lint/check/build/rustdoc/fmt/diff, resource
smoke and rendered API inspection pass. Both actual Clang lifecycle tests and the
five shared Python union tests pass. Architecture adds507 NLOC/15 analyzed functions/
three files; avg CCN3.38→3.37, max132,33 functions above15,126 clone groups and one
cycle unchanged. Normalized clone identities have no additions/removals. Raw
comparison flags a touched old service-test clone and unchanged Python rank ties;
request-owned command/environment assembly removed the initial execute hotspot.
Evidence: ~/.local/state/radiology-qa/personality-generation-20261102-edb74c0.

The app workflow below is now accepted. Next verify the full matrix at seL4 d419a119a61a9b42edf61fb79ad08de45248daa7.
The clean QA checkout is ~/.cache/radiology-qa-sel4-source. Host inspection finds
Clang23.1.4 with the required targets, CMake4.4.5 and Python3.14.8; dtc is absent
from PATH and needs private provisioning before real matrix acceptance. Current
CMake/runtime compatibility must be demonstrated by execution, not assumed.

## Standalone picker and recovery checkpoint

Index Projects presents a subordinate Personality selector, editable summary and
Show Settings File action beneath its existing heading. It reuses MenuBar's checked
choices and owned arrows/Return/Escape. The initial egui combo-box experiment did
not meet keyboard ownership requirements and was removed. Separate groups preserve
the active group's anchor registry, so View→Search still works while Index Projects
is open. Font sizes, colors, source styles and window structure are unchanged.
The named SwiftUI/macOS/typography skills were unavailable locally; existing app
controls and established type styles supply this Linux UI's conventions.

SessionStore saves choices by canonical attached path, including aliases and attached
PRs, using atomic session replacement. A failed selection save preserves the previous
in-memory choice. Preferences and templates are rejected inside viewed source. The
picker starts generation only after a worker captures valid settings. Repository
open/reload, index reload, inspection and Reindex all reread settings. Existing jobs
retire when selection or scope changes; a worker holds its review lease while loading.
Loading/cancellation statuses include this preparation phase. Embedded hosts can
inspect asynchronously and read the selected pipeline, settings, error and file path.

Selection retires semantic data without navigating or resetting reader history/view
settings. Invalid TOML clears semantic results and blocks import fallback; it cannot
silently run another pipeline. Recovery keeps both the selector and file action
available. Show Settings File uses the existing bounded file-manager handoff worker.
Default may use the normal repository-import fallback; seL4 never adopts that data.
Footer actions and project rows have separate presentation owners; the displayed
project limit uses the captured setting instead of a hardcoded32.

Five new app tests cover selection/relaunch/aliases/PR scope, source/caret/history
retention, failed saves and source-write rejection, invalid-file recovery, retired
loads and actual menu keys. Existing fixtures now separate source from settings;
source mutations use the accepted root. A duplicated reference fixture now reuses
the shared repository setup. Async generation tests poll preparation rather than
assuming the compiler worker starts in the command's calling frame.

Workspace:753 app tests pass, nine opt-in ignored; two binary/13 font/21 text and59
doctests pass. Strict Clippy/check/build/rustdoc/fmt/diff, resource smoke, rendered
API and actual Clang lifecycle checks pass. After fixture consolidation the157 app
checks and strict Clippy pass again. No production behavior changed in that final
fixture-only consolidation.

Native final1×/2× runs use actual scip-clang0.4.0 and the real shared Python runner.
Each generates Default data, selects seL4 by ordinary menu keys, observes the expected
failure on a deliberately non-seL4 fixture, and preserves Default's cached bytes.
Selection survives relaunch. Returning to Default regenerates; corrupting its TOML
blocks semantics, Show Settings File sends the exact URI to a private stand-in,
and restoring/reloading recovers a cached Partial index without a third compiler run.
Both themes, unchanged source/binary hashes and zero core exit statuses are verified.
Captures were inspected; held hover helpers are intentionally terminated on cleanup.
Driver diagnostics preserve earlier OCR and cache-path assumptions; final checks use
UI observations together with actual session/cache/tool-counter evidence. This does
not establish real seL4 build/profile coverage or a host desktop's file-manager setup.

Across preference/menu/picker commits against before-personality-picker at0dbf6d5:
+627 analyzed NLOC/27 functions/two files; avg/max CCN3.37/132 and33 functions above15
unchanged. Clone groups126→125, duplicated lines1341→1335; normalized identities
have zero additions/one removal. Cycles remain one. Raw comparison still flags old
touched clones and unchanged Python rank ties.

The structural score does decline80.73→79.57: issues7→8. A full raw analysis of the
published parent and current tree identifies the additional recommendation as a
branch reorganization of src/ui/app/tests after adding its focused personality test
module (ten analyzed files). That is a real directory-layout recommendation; it is
not a new runtime complexity/cycle issue. Moving tests into generic core/io/api
folders solely to remove the warning would not simplify this feature. Keep the
coherent feature fixture and record the warning. Initial renderer hotspots and a
new fixture clone were eliminated through presentation ownership and setup reuse.

Preferences were committed/pushed as042f8a3 and menu ownership as9677046; this app/UI
checkpoint follows separately. Evidence:
~/.local/state/radiology-qa/personality-picker-20261102-0dbf6d5.
The full matrix, private dtc provisioning, broader corpus/PR/native IME/portal checks,
shortcut help and the overall parity audit remain open.

## Linux matrix checkpoint

Real preparation and per-profile indexing now pass all67 profiles on Linux after
fixing PATH precedence in both shared Python runner stages. Upstream generators
invoke python3 by name: its validated/provisioned environment must precede a system
Clang directory such as /usr/bin. No platform fork or duplicate recipe was added.
The QA recipe changes only runtime.dtc to a privately extracted DTC1.8.1 path;
all67 bundled profile definitions remain intact. CMake4.4.5, Clang23.1.4, Ninja,
Python3.14 with exact configured pins and actual scip-clang0.4.0 pass.

The raw union has598 documents/24,934 symbols/131,126 occurrences, matching the
Swift baseline. All ranges map through the Rust decoder to original source bytes;
conditional TCB-control definitions/references survive. The manifest records all
seven modes and134 translation units. Its178 kernel path spellings resolve to176
physical kernel files; adding the library C file gives177, matching Swift. Source digest and clean Git status are unchanged. Real
Rust service generation/Reindex/cache now passes as described below; native
real-matrix acceptance remains separate. Current evidence: ~/.cache/radiology-qa-sel4-matrix.

## Real Rust service acceptance

The service's real cold run (87.18 s) and forced Reindex (86.68 s) each prepare/index
all 67 profiles inside disposable source copies. Each emits 598 documents, 24,934
symbols and 131,126 occurrences without project reuse, cache warnings or stale
source. Every occurrence range maps to the original source bytes in both outputs.
Raw union validation covers all 177 physical non-profiler C files, conditional
TCB-control source definitions/references, scheduler/IPC/interrupt symbols, SMMU and
public APIs. The pinned checkout retains its original digest and clean Git status.

Cache-only loading with no compiler/Python lookup returns the same union in 2.66 s;
a separate repeat takes 2.59 s and confirms Partial/no cache warning. Temporary
workspaces and run artifacts are removed on completion; namespaced caches persist.
Actual app-resource smoke and strict all-target Clippy pass with corrected scripts.
This supplements the app/native fixture checks; it does not substitute for native
real-matrix navigation/control acceptance at both scales.

Durable evidence and reproducible probe sources:
~/.local/state/radiology-qa/sel4-linux-matrix-20261102-b1e7f80.
The archive includes the raw union/provenance, all profile manifests and logs,
source/tool digests, service reports and coverage/service/warm Rust probes.

## Native real-matrix reader checkpoint

Both native display scales pass app-initiated cold Reindex of all 67 profiles,
with real compiler counters and honest Partial/598-document/25-missing-path status.
Subsequent navigation checks reuse those verified caches: Go to Line, cross-file
handleInterrupt definition, exact original destination copying and exact Back
caret copying. References/Definition/Outline and Files reveal remain coherent.
Dark/light and restart preserve seL4 and load Partial cached without compilation.
Final core processes exit zero; source and executable hashes remain unchanged.

The cold compiler evidence and warm navigation cases are distinct, linked artifacts.
Early driver assumptions about a definition's file, asynchronous publication and
OCR were corrected; no native product change was needed. Evidence:
~/.local/state/radiology-qa/sel4-native-20261102-fe436a8.
