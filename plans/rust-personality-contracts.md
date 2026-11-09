# Indexing-personality assertion reconciliation

Reference: five IndexingPersonalityTests groups and one IndexingPersonalityUITests
group. Four settings/model groups and the UI group are reconciled. The final
cache/pipeline group is partially verified and remains open for explicit
cross-checkout reuse. Five of 35 compiler/personality groups are mapped.
This checkpoint changes tests only, using existing stores, runners and app owners.

| Swift group | Rust evidence and status |
| --- | --- |
| templatesParseAndTOMLRootSettingsAreTyped | Reconciled. shared_templates_capture checks both actual packaged templates, captured identity, digest and all 67 profiles/five scripts. typed_roots_accept checks literal # strings, comments, multiline arrays and nested recipe separation. invalid_root_syntax includes duplicate/unknown keys, wrong schema/identity/types and bounds. Rust uses the existing TOML parser, intentionally also validating nested syntax. |
| storesInstallWithoutReplacingUserEditsAndRejectBadSettings | Reconciled. loads_preserve checks first install, edited jobs/digest, original captured bytes and invalid-file preservation; concurrent_first_use covers races. captured_default_settings drives actual --jobs 7 and rejects incompatible settings/reuse. The type/limit matrix rejects schema 9 without replacement. |
| selectionIsPerRepositoryAndRestoresAcrossPRRoots | Reconciled. personality_selection_persists uses real SessionStore persistence, canonical alias/relaunch, independent repository default and PR enter/exit under the attached checkout choice. Reader/caret/history remain unchanged. |
| invalidSettingsFailWithoutFallingBackAndCancellationStopsThePersonality | Reconciled. settings_errors_block now exercises both Default and seL4 through the actual settings worker, error controls, file reveal, repair and regeneration request. Failed settings retire semantic data and never choose another pipeline. sel4_live_cancel interrupts the actual selected controlled runner, reaps it, preserves accepted cache bytes and cleans private inputs. Retired settings cannot publish after selection/root changes. |
| cacheNamespaceSettingsEditsAndReindexKeepTheSelectedPipeline | Partially verified, OPEN. sel4_runs_shared now proves a jobs edit forces the selected scip-sel4 runner, captures new exact bytes/digest, writes a warm-readable latest cache, rejects the old settings and leaves Default bytes intact. Default settings changes, forced generation, seL4 project reuse and cache-only loading are covered. Swift's explicit reuseSources from another checkout root is not supported by Rust's current attached-root-scoped cache; do not count that clause as complete. |
| switchingPersonalityPreservesReaderAndNativePopoverLayout | Reconciled through platform equivalents. actual_picker_arrows checks the real picker Return/Escape and source/history ownership. rapid_personality_switches adds Default→seL4→Default→seL4 requests without waiting for obsolete settings, then verifies selected identity/plan and regeneration selection, reader/caret/history and actual Index Projects bounds (<600×750 logical points) at both scales/themes. Controlled service tests separately prove successful selected-runner execution; the new UI matrix disables tools and checks honest failed-tool outcomes. Existing real 67-profile/native acceptance is linked in rust-indexing-personalities.md; this checkpoint does not rerun that external corpus. |

## Remaining cross-checkout reuse contract

Swift accepts an explicit cache/root/label reuse source, validates content digest,
and copies a matching strategy into the new root's own cache. Rust currently
requires the same canonical attached identity for project reuse, while checkout
and pinned After roots under that identity already share exact records. Different
attached repositories intentionally do not borrow one another automatically.
Completing the reference behavior requires an explicit source handoff with the
same captured settings/tool/strategy/full-input checks and destination-owned
publication; an arbitrary global cache scan would not satisfy this contract.
This is required remaining work, not a newly accepted scope exclusion.

## Evidence

Archive: ~/.local/state/radiology-qa/personality-20261109-fb5b7d4/.
All 19 personality settings/service/app tests and strict workspace/all-target
Clippy pass, plus formatting and diff checks. Architecture result NEUTRAL:
+94 NLOC/one app test, no runtime helper/file/API, average/max CCN 3.43/132,
36 hotspots/one cycle and all 125 normalized clones unchanged (zero added/removed).
An initially duplicated run-count assertion was simplified to exact run-log bytes.
Latest full production gate remains c20eba6; no new full-run claim for test-only
changes. Source/syntax remains 66/66. Compiler/personality has 30 groups remaining
(including the partial group), inspector/traits 15 and final acceptance remain.
