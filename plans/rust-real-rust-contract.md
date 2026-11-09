# Exact real Rust compiler contract

The tenth IndexingTests function, realRustAnalyzerGeneratesAnIndexFromScratch, now
has a fresh real-tool mapping. All ten IndexingTests groups and all 35 compiler/
personality groups are explicitly mapped. This does not close inspector/trait
assertions or final integrated/native/standalone/shared-host acceptance.

## Actual generation

The existing opt-in real_rust_analyzer_indexes_feature_gated_workspace_calls_and_reuses_the_result
case now also reads only Cargo.toml, src/lib.rs and src/shapes.rs from the exact Swift
scip-rust fixture. It never copies the recorded index.scip. The production generation
service uses the actual rust-analyzer 1.98.1 component to create new SCIP, producing
exactly two navigable documents and no missing sources, compiler-partial flag,
failure, stale reason or cache warning. The recorded outcome version matches the
actual tested executable's 1.98.1 version. Its bounded diagnostics retain the tool's
nonfatal empty ConfigErrors([]) log; this is not a claim of silent compiler output.

The shared real-tool lifecycle asserts that the Circle occurrence in src/lib.rs
resolves to src/shapes.rs at zero-based range (0,11,17), and that its reference
range maps back to the original source bytes with UTF-8 columns. Update reuses the
exact generated project with original provenance; asynchronous cache-only reload
works with all tools removed. Source digests reject changed bytes. The three viewed
inputs and complete inventory remain exact, and no Cargo.lock, target or index.scip
appears in the viewed checkout. Private generation workspaces are cleaned up.

The existing feature-gated workspace case also passes in the same run: three source
documents, member feature/miri configuration, cross-file double navigation, Update
reuse, tool-free reload, byte validation and source preservation. Keeping both cases
in the existing opt-in test avoids a new runner, installation policy or test fixture
abstraction. The ordinary ignore count remains twelve.

## Private prerequisite

The system rust-analyzer shim lacks its component. QA installs the exact matching
Rust 1.98.1 toolchain/component through rustup into the explicitly overridden
~/.cache/radiology-qa-real-rust/rustup, with a separate private CARGO_HOME. The normal
user toolchain/defaults and repository compiler settings are untouched. The tested
binary SHA-256 is 25da0296e9aa8e538e6ee36675f48b0cd3896c5d1764a3a71a0e2438757f5640;
this is an artifact identity, while rustup handles distribution verification.
The component reports rust-analyzer 1.98.1 (48a229c 2026-09-01).

## Evidence and limits

Archive: ~/.local/state/radiology-qa/real-rust-contract-20261109-15c6afb/.
The explicit opt-in case passes both fixtures in 4.18 seconds with offline Cargo
resolution; strict all-target Clippy and formatting pass. The preceding full serial
workspace gate passes 939 app tests (twelve opt-in ignores), binary/font/text
2/13/23 and all 68 doctests. Only an already-ignored test is extended after that
full gate; its two actual-tool paths are explicitly exercised. No production source,
resource, API or UI behavior changes in this checkpoint.

Architecture compares NEUTRAL: +25 measured NLOC, no new function/file, avg/max CCN
3.43/132, all 36 high-complexity functions/one cycle and 125 normalized clones
unchanged (zero additions/removals). The raw ranked Python alert is in an untouched
source. This exact fixture complements the prior real multi-language compiler
records; SDK-backed Android success remains unverified and requires the outstanding
license authorization. Accessibility and actual sibling integration remain deferred.
