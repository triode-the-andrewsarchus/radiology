# Rust indexing personalities and seL4 parity

Reference: [upstream contract](indexing-personalities.md) and
[compiler matrix](sel4-indexing.md). This is required parity work; the settings
checkpoint below is one reusable part, not a completed app workflow.

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

## Integration still required

1. Capture/reload settings on repository open/reload, selection and Reindex; persist
   the selection by canonical attached checkout, including attached PR use. Invalid
   files need visible failure and a working Show Settings File action.
2. Service integration is implemented: Default's adapter allowlist/project limit
   and Clang jobs drive actual planning/commands. Captured digest participates in
   strategy equality, normalized cache validation and project reuse. Default keeps
   its namespace; seL4 uses personalities/sel4, including revision-keyed records.
3. The distinct scip-sel4 service strategy and private shared-runner invocation are
   implemented, backed by the existing managed Clang tool. Controlled runner tests
   verify captured TOML/scripts, source-only publication, cache isolation/reuse,
   cancellation and cleanup. Real matrix/provisioning acceptance and any required
   platform recipe adaptation remain open.
4. Add the picker/explanation/file action to Index Projects with existing styles,
   preserve reader/caret/history on switches and reuse Install and Index for Clang.
5. Exercise settings edits/cache transitions, invalid recovery, live cancellation,
   PR identity and reader retention in actual service/app paths. Run the full 67
   profile recipe against the pinned seL4 checkout and verify original coordinates,
   conditional alternatives, all modes, source integrity, Reindex and warm caches.
   Validate Python provisioning and union integrity, then native1×/2× controls.

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
host behavior and wire records; the app still passes None until picker/session
integration. Captured Default filters adapters before applying the project cap and
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

Next add repository selection/picker/reload/recovery and actual app/native checks,
then verify the full matrix at seL4 d419a119a61a9b42edf61fb79ad08de45248daa7.
The clean QA checkout is ~/.cache/radiology-qa-sel4-source. Host inspection finds
Clang23.1.4 with the required targets, CMake4.4.5 and Python3.14.8; dtc is absent
from PATH and needs private provisioning before real matrix acceptance. Current
CMake/runtime compatibility must be demonstrated by execution, not assumed.
