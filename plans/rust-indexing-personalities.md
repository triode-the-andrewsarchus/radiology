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
2. Apply Default's adapter allowlist/project limit and Clang jobs to actual planning
   and commands. Carry the settings digest through strategy identity, normalized
   cache freshness and project reuse. Keep Default's existing cache namespace and
   isolate seL4 under personalities/sel4, including pinned review caches.
3. Add a distinct scip-sel4 strategy backed by the existing managed Clang tool and
   shared Python pipeline. Stage captured TOML/scripts in private run output,
   resolve configured bootstrap Python and preserve the existing process budgets,
   source/snapshot verification, cancellation, normalized source-only publication
   and cleanup. Adapt platform messages/candidates where necessary.
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
