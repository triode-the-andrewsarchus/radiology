# Indexing-personality assertion reconciliation

Reference: five IndexingPersonalityTests groups and one IndexingPersonalityUITests
group. All six are now reconciled: the later [explicit handoff](rust-index-reuse-contracts.md)
closes the cross-checkout clause left open by this document's original test-only
checkpoint. Its original nineteen-test evidence remains below; the runtime
checkpoint carries the combined verification.

| Swift group | Rust evidence and status |
| --- | --- |
| templatesParseAndTOMLRootSettingsAreTyped | Reconciled. shared_templates_capture checks both actual packaged templates, captured identity, digest and all 67 profiles/five scripts. typed_roots_accept checks literal # strings, comments, multiline arrays and nested recipe separation. invalid_root_syntax includes duplicate/unknown keys, wrong schema/identity/types and bounds. Rust uses the existing TOML parser, intentionally also validating nested syntax. |
| storesInstallWithoutReplacingUserEditsAndRejectBadSettings | Reconciled. loads_preserve checks first install, edited jobs/digest, original captured bytes and invalid-file preservation; concurrent_first_use covers races. captured_default_settings drives actual --jobs 7 and rejects incompatible settings/reuse. The type/limit matrix rejects schema 9 without replacement. |
| selectionIsPerRepositoryAndRestoresAcrossPRRoots | Reconciled. personality_selection_persists uses real SessionStore persistence, canonical alias/relaunch, independent repository default and PR enter/exit under the attached checkout choice. Reader/caret/history remain unchanged. |
| invalidSettingsFailWithoutFallingBackAndCancellationStopsThePersonality | Reconciled. settings_errors_block now exercises both Default and seL4 through the actual settings worker, error controls, file reveal, repair and regeneration request. Failed settings retire semantic data and never choose another pipeline. sel4_live_cancel interrupts the actual selected controlled runner, reaps it, preserves accepted cache bytes and cleans private inputs. Retired settings cannot publish after selection/root changes. |
| cacheNamespaceSettingsEditsAndReindexKeepTheSelectedPipeline | Reconciled by the later explicit handoff. sel4_runs_shared now proves a jobs edit forces the selected scip-sel4 runner, captures new exact bytes/digest, writes a warm-readable latest cache, rejects the old settings and leaves Default bytes intact. Default settings changes, forced generation, seL4 project reuse and cache-only loading are covered. The later explicit reuse_sources handoff now supplies the same edited seL4 settings to another root and verifies reuse without invoking its runner, plus destination cache restoration. |
| switchingPersonalityPreservesReaderAndNativePopoverLayout | Reconciled through platform equivalents. actual_picker_arrows checks the real picker Return/Escape and source/history ownership. rapid_personality_switches adds Default→seL4→Default→seL4 requests without waiting for obsolete settings, then verifies selected identity/plan and regeneration selection, reader/caret/history and actual Index Projects bounds (<600×750 logical points) at both scales/themes. Controlled service tests separately prove successful selected-runner execution; the new UI matrix disables tools and checks honest failed-tool outcomes. Existing real 67-profile/native acceptance is linked in rust-indexing-personalities.md; this checkpoint does not rerun that external corpus. |

## Cross-checkout reuse follow-up

The original checkpoint identified an unsupported explicit reuseSources handoff.
That gap is now repaired through ordered IndexReuseSource values and shared full-
input/tool/settings validation. Destination-owned publication and persisted mixed-
origin warnings are covered by [the reuse checkpoint](rust-index-reuse-contracts.md).
Different roots still never borrow automatically; a caller supplies the handoff.

## Evidence

Archive: ~/.local/state/radiology-qa/personality-20261109-fb5b7d4/.
All 19 personality settings/service/app tests and strict workspace/all-target
Clippy pass, plus formatting and diff checks. Architecture result NEUTRAL:
+94 NLOC/one app test, no runtime helper/file/API, average/max CCN 3.43/132,
36 hotspots/one cycle and all 125 normalized clones unchanged (zero added/removed).
An initially duplicated run-count assertion was simplified to exact run-log bytes.
Latest full production gate remains c20eba6; no new full-run claim for test-only
changes. Source/syntax remains 66/66. At that checkpoint compiler/personality had 30 groups remaining
(including the then-partial group); the reuse follow-up leaves 26, inspector/traits 15 and final acceptance remain.
