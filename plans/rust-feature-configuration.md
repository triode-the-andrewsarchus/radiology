# Rust manifest feature configuration and real-tool acceptance

## Behavior and Swift parity

Swift RustAnalyzerIndexer unions explicit feature names from Cargo.toml files under
an indexing strategy and supplies feature cfg overrides alongside debug_assertions
and miri. The Rust port now does the same using the isolated input inventory and
workspace. Sibling project directories do not leak overrides across boundaries.
Names are sorted/deduplicated and default is excluded, matching Swift. Parsing uses
toml_edit, already present at the same locked version; its parse feature is now a
direct dependency. Quoted keys and commented table headers also work.

Command planning remains pure: CommandPaths receives the discovered feature names
and returns JSON resources without reading or writing files. Generation reads only
private copied inputs, then materializes configuration in the private run directory.
Cargo still requests all features; build scripts and procedural macros remain
disabled as in Swift. No viewed manifests or generated configuration are rewritten.

Manifest reads reuse the existing no-follow, bounded, cancellable reader, with a
4 MiB per-manifest limit. Valid internal manifest links resolve inside the owned
workspace; escaping links fail. Cancellation is checked during reads, between
manifests/keys and after bounded TOML parsing; the parser itself has no cancellation
callback. Invalid UTF-8 or malformed unused nested manifests contribute no names,
matching Swift's best-effort discovery; Cargo diagnoses malformed files it uses.
Oversized/read failures are explicit project failures before index execution.

Normalized and project cache format 3 rejects earlier records: format 1 could
misinterpret occurrence encodings, and format 2 predates these Rust cfg overrides.
Regeneration is required because missing declarations cannot be repaired by
relabelling old semantic data. Cache-only loading continues to run no indexers.

## Automated acceptance — 2026-10-30

Two new non-network tests prove nested manifests, quoted keys/comments, duplicates,
default exclusion, sibling boundaries, retained build/proc-macro options, private
configuration materialization, oversized-file refusal and internal manifest links.
Existing cache validation also rejects format 2.

The new ignored real Rust test loads an actual two-crate workspace whose root module
is gated by miri and a feature declared only in the nested member. rust-analyzer
1.98.1 produces three documents with no missing sources; the call resolves exactly
to the declaration in src/gated.rs. Incremental reuse and asynchronous cache-only
reload without tools preserve the definition and exact input bytes/inventory.
Cargo.lock and target are absent from viewed inputs. A negative control temporarily
omitted the overrides: the same real tool omitted src/gated.rs and reported one
missing source. The accepted implementation was restored and the positive test
rerun successfully. The tool prints ConfigErrors([]) at ERROR level but returns
zero and passes coverage/navigation assertions; this message is retained in logs.

A matching minimal toolchain/component was installed only under
/tmp/radiology-real-rust-toolchain. Normal user toolchain selection is unchanged.
The opt-in test accepts RADIOLOGY_REAL_RUST_ANALYZER as an explicit binary path:

```sh
RADIOLOGY_REAL_RUST_ANALYZER=/path/to/rust-analyzer CARGO_NET_OFFLINE=true \
  cargo test --lib real_rust_analyzer_indexes_feature_gated_workspace_calls_and_reuses_the_result -- --ignored --nocapture
```

Full workspace gates pass 414 app tests (three ignored), eight shared tests and
30 doctests, fmt, strict Clippy, build and rustdoc. After sharing the real-tool
input-retention assertions, both opt-in Rust and TypeScript tests and strict Clippy
pass again. The API field/method documentation was checked in generated HTML.
Architecture adds no hotspot, cycle or clone growth; five reported test fragments
are byte-identical to baseline before-rust-feature-parity. The real-tool assertions
reuse the same fixture and input-retention helper instead of duplicating setup.

Logs: /tmp/radiology-rust-features-{tests,all-tests,clippy,final-clippy,build,doc,real-tools}.log,
/tmp/radiology-real-rust-features.log and
/tmp/radiology-real-rust-features-without-overrides.log.

## Native acceptance

The standalone reader generated a real Rust index for a dirty two-crate Git fixture,
showing three documents, zero missing sources and the actual rust-analyzer version.
Go to Line revealed the folded call; pointer selection and F12 opened src/gated.rs.
The inspector showed the exact cross-file reference. Update reused the record;
a transparent exec wrapper added after generation recorded exactly one --version
probe and no SCIP invocation. Retiring that QA-managed wrapper and restarting still
loaded the three-document cache and persisted checkout origin without tooling.
HEAD/status/all file bytes stay unchanged, no Cargo.lock/target artifacts remain,
private workspaces/stages are removed, and app/compositor exit codes are zero.

Evidence: /tmp/radiology-rust-features-native/acceptance.json and SHA-256 manifest.json.
Key captures: real-rust-generated.png, feature-call-revealed.png,
real-feature-gated-definition.png, real-rust-reused.png and
reopened-real-cache-projects.png. Tool observation/retirement are explicit QA actions
outside viewed inputs. The sibling compositor capture helper is QA-only; the app
has no sibling runtime dependency. Scope is dark 1280×900 at scale 1 on one real
workspace. Broader Rust/build-script/proc-macro/environment/corpus and desktop
scaling/accessibility acceptance remain open within the full parity objective.
