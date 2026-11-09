# Indexing lifecycle assertion reconciliation

Nine of the ten functions in Tests/RadiologyTests/IndexingTests.swift now have
explicit Rust contract mappings. The real-rust-analyzer generation function remains
open for a fresh actual-tool run with the exact two-document Swift fixture; the
existing real feature-gated workspace case is useful prior evidence, but does not
by itself close that exact fixture contract. Compiler/personality is therefore
26/35 mapped, with nine remaining: real Rust generation, six Kotlin and two PR
Kotlin functions. Inspector/trait groups and final acceptance remain separate.

## Assertion mappings

| Swift function | Rust proof and intentional API representation |
| --- | --- |
| processCapturesStatusAndBoundedOutput | process/tests.rs checks exact literal stdout and stderr, nonzero exit status, missing-executable spawn failure and separate signal exits. A million bytes on both streams prove drain safety and bounded first/last retention. Rust exposes one bounded head/marker/tail string per stream rather than Swift's separate head/tail properties; exact 200,000-byte and exit-3 literals are not required for that representation. |
| queuedCompilerProgressCannotReplaceFinishedStatus | index_lifecycle::noisy_compiler_completion_stays_terminal_after_more_polls_and_frames emits 2,000 actual stderr progress lines, completes automatic two-document generation, then polls/renders for at least 200 ms. Every frame retains the terminal summary, index identity and no running job. The shared runner invokes its FnMut observer synchronously; Rust has no detached progress publication queue. |
| cancellationTerminatesTheWholeProcessGroup | process::tests::cancellation_reaches_a_running_child_and_preserves_diagnostics now starts a background descendant and makes parent/child ignore TERM. After the ready marker it requests actual cancellation, checks the cancellation category and exact retained diagnostics, then verifies the recorded descendant PID is gone or an exited zombie. The deadline and normal-exit descendant cases remain separate. This identifies the owned child instead of scanning unrelated global sleep processes. |
| generatesCachesReloadsAndDetectsStaleness | index_lifecycle automatic_open and automatic_stale_refresh use the exact two-document recorded Swift Rust corpus. They verify version, no Git revision, generated status, cache-only warm reload with the executable removed, no viewed output artifacts, stale publication before retry, exit-9 diagnostics and unchanged prior cache bytes/index identity on failure, then repair. Invocation diagnostics and reuse checks retain actual command/cwd snapshots across serialized reload. Formats and cache filenames remain Rust-owned. |
| missingToolsAndCancellationAreReportedNotHidden | automatic_missing_tool reports a failed outcome without installation, displays its no-version state at both scales and recovers after executable repair. automatic_generation_cancellation_and_root_replacement retires actual paused work, retains the reader and prevents late publication. Service replacement and tools probe cases additionally check ownership, bounded failures and diagnostics. Existing installation hints remain explicit. |
| nestedProjectsPrefixDocumentPaths | swift_recorded_rust_documents_and_definitions_receive_the_nested_project_prefix sends the exact recorded Swift SCIP corpus through production normalize and SemanticIndex, with engine/rust input inventory/workspace. It asserts the exact two prefixed document paths and the Circle global definition at engine/rust/src/shapes.rs. |
| inspectionPicksTopLevelProjectsAndRegistryStrategies | swift_mixed_checkout_plan_keeps_root_priority_and_exact_project_evidence uses the literal Swift path inventory and asserts all five exact selected IDs in order, tsconfig evidence, no skipped or uncovered work, plus the exact unsupported Kotlin/Swift list for orphan sources. Existing nested/sibling tests prove deterministic boundaries and reverse-input invariance. |
| substantialMonoreposDoNotSilentlyDropLaterRustProjects | swift_twelve_project_monorepo_keeps_the_last_rust_project uses all twelve literal modules/projectN roots, asserting twelve selected, no skipped and project11 present. Existing cap tests also retain overflow projects beyond the configured limit rather than silently dropping them. |
| rustAnalyzerConfigurationUsesRealManifestFeaturesAsCfgOverrides | service::tests::rust_features reads the actual private invocation config, asserting features=all and ordered debug_assertions/miri plus real root/member feature names including app-runtime and embedding_gate. Sibling features are excluded; comments/quoted names, internal links and oversized manifests have explicit cases. Source manifests remain unchanged. |

The tenth function, realRustAnalyzerGeneratesAnIndexFromScratch, still needs actual
generation of the exact Swift source fixture followed by Circle navigation, actual
version and two-document coverage. The system rust-analyzer shim currently lacks
its component, and the former private QA component is no longer present. No global toolchain has been changed to hide this missing evidence.

## Status-label cleanup

An outcome with no recorded version formerly displayed Tool availability unchecked,
even after an actual unsuccessful probe. Now only projects without an outcome and
without an explicit probe show that text; attempted outcomes without a version say
No tool version recorded. A missing version is not used to invent a precise failure
stage: private preparation can also fail before a probe. Existing outcome failures,
installation hints and explicit Check Tools results remain the authoritative detail.
The both-scale missing-tool app case verifies the new label and absence of the old
one. No new display state, worker, settings field or public API was added.

## Verification

Evidence: ~/.local/state/radiology-qa/indexing-contracts-20261109-9f1c9c7/.
All 115 indexing cases pass (twelve real-tool opt-in ignores), ten app lifecycle
cases and thirteen process cases pass. Strict all-target Clippy and the normal
workspace build pass. The latest complete serial workspace gate
remains the preceding invocation checkpoint: 932 app tests, twelve opt-in ignores,
binary/font/text 2/13/23 and all 68 doctests. Focused frame checks here are not a new
native-desktop acceptance claim. Full final integrated/standalone/reuse gates remain.

Architecture is compared to before-indexing-contracts-20261109. The normalized
clone multiset is unchanged; raw touched-test-file and ranked untouched Python
alerts are checked against the complete baseline rather than treated as new debt.
The initial temporary text-label collection repeated an existing test block; direct
iterator assertions removed that allocation and new clone. Final measured delta: +138 NLOC/net three analyzed functions, no new file;
avg/max CCN 3.43/132, 36 high-complexity functions, one cycle and all 125 normalized
clone instances unchanged (zero additions/removals).

## Real-tool follow-up

[Fresh exact-corpus generation](rust-real-rust-contract.md) now closes the tenth
function. The nine mappings above remain unchanged; all ten IndexingTests and all
35 compiler/personality functions are mapped. Final acceptance is still separate.
