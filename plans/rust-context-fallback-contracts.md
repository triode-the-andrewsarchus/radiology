# Context fallback and missing-definition contracts

Reference: Swift ContextModelTests implementation/outline/local/external groups,
compared with Rust `c30623b` and this checkpoint. These four groups join the four
previous mappings: eight of eighteen groups are mapped and ten remain. Other
reference/call-tree/palette suites and final acceptance remain open.

## Missing behavior and correction

The initial real-fixture test passed the implementation/outline cases but failed
on expanded external details: package metadata was present, but no visible reason
explained the absent source preview. The renderer now shows “No definition indexed
in this repository.” for a nonlocal symbol with no indexed definitions when no
preview is available. Existing package details and location help distinguish
potential external source from missing coverage. A recorded Here definition never
gets this message. Existing worker loading/errors/excerpts retain their own UI.

This intentionally avoids Swift's stronger inference that every such symbol must
be defined outside the repository. Package metadata and absent definitions do not
establish source provenance. Rust's model keeps PreviewPhase::None and starts no
excerpt worker; the unavailable-source explanation is derived in the renderer.
The user-visible contract is explanatory missing-source feedback without fabricated
navigation or source. Locals retain their source-backed single definition line.

The existing bounded three-row explanation renderer supplies presentation. A
longer draft explanation failed the full-visibility assertion by one pixel in the
default app pane; the concise final message fits completely at both scales.
No new state, query, worker, public API or dependency. Fonts/geometry/Swift source
are unchanged. Required SwiftUI/macOS/typography skills were unavailable in the
catalog/local search in the preceding checkpoint; existing Rust styles are reused.

## Complete group mapping

All four groups run through the existing real rust-analyzer fixture app matrix at
1×/2×, after the prior identity/reference workflow. The focused fallback scenario
uses that accepted source and existing metadata-only publication helper. It is
called by `context_reuses_previews_and_declaration_rows_across_identity_changes`.

| Swift group | Rust evidence |
| --- | --- |
| implBlocksFollowTheirRealSCIPTypeReference | At the impl closing brace outside both methods, the innermost declaration is Extension and the automatic reference identity is the exact indexed shapes/Circle#. The existing declaration resolver binds the occurrence at the parsed type-name byte, accepting the nondefinition only for an implementation. Under-caret facts and preview are absent at the closing brace. |
| whitespaceAndKeywordsFallBackToTheOutline | Circle at 6:39 resolves normally. The blank line at 4:0 clears under-caret/preview/declaration/reference target and visibly invites placing the caret inside a declaration. The let keyword at 8:9 clears semantic token facts while retaining area and its references. Injecting the equivalent of Swift's nil-syntax input (empty accepted symbols plus explicit structure-unavailable reason) clears the declaration/automatic target, visibly reports no parsed structure, and still resolves indexed local r. This is an absent-structure boundary injection, not a claim of a real parser crash. Actual Reload Source restores prepared structure for the following cases. |
| localsAreQuietAndDescribedFromTheIndex | At the second r in PI * r * r, the exact scoped local identity is named r, has three file occurrences and the source-backed line 8 `        let r = self.radius;`. Metadata publication leaves PreviewPhase::None with no excerpt job. Expanded details show the three-occurrence facts. Existing UI checks retain the local line and exclude the missing-repository-definition message. |
| externalSymbolsExplainMissingSourceWithoutFabricating | The real += core operator has a nonlocal identity, zero indexed definitions, no destination and core package metadata. Metadata publication starts no preview job. Expanded actual app details fully paint the missing-definition message. Source Arc/history stay unchanged. The UI regression also checks missing package metadata, excludes Here definitions and locals, and preserves existing long-text/navigation-target checks. |

The fallback phases preserve source Arc/history before recovery. The operator
case independently captures and verifies its accepted source/history after file
navigation. Existing exact-name declaration binding tests establish that equal
spelling cannot substitute for an indexed occurrence.

## Verification and architecture

Archive: `~/.local/state/radiology-qa/context-fallback-20261109-c30623b/`.
Initial independent tests recorded one pass and one expected missing-message
failure. They were consolidated into the existing two-scale app matrix to reuse
fixture/lifecycle setup, with full visible-text assertions replacing duplicate
paint searches. A consolidation typo called a nonexistent command method; the
compiler rejected it, and the existing execute_command route corrected it.
The longer explanation's clipping failure is retained in app-final.log; final
short-message app checks and all three under-caret UI checks pass. Strict all-target
Clippy, normal build and warning-free docs pass. The complete locked/offline
workspace run passes 886 app tests (eleven existing opt-in ignores), two binary/
13 font/22 text tests and all 66 doctests. The app test count stays unchanged
because the new scenarios reuse the existing fixture-matrix test. Full-run
compilation uses -j 1 with serial tests; ordinary half-core Cargo jobs=16 is
unchanged.

Architecture adds 127 measured NLOC/two functions/one focused test scenario file.
Average/max CCN remain 3.42/132 with 36 hotspots and dependency cycles unchanged.
Five actual draft clones were removed through fixture-matrix reuse, existing
full-visibility assertions and one shared no-preview disclosure check. Complete
normalized clone multisets retain 124 instances, zero additions/removals. The raw
remaining flag is an unchanged old app/tests.rs clone; equal-score Python
chokepoint exchanges are untouched code.

Next concrete source discrepancy to reproduce: Swift's nested multiline
signature group strips the declaration's own indentation and caps its text at
600 source bytes/eight lines. Rust's parser currently retains the raw header in
DocumentSymbol.detail, and the declaration renderer passes it directly to the
4096-byte generic text fitter. Preserve raw source/copy contracts while checking
this display difference with Swift's exact fixture; it is not mapped here.

Remaining: ten ContextModel groups, reference/call-tree/palette, source/syntax,
compiler/indexing and inspector reconciliation, then final native/standalone
acceptance. Known parallel timing/linking limits and Android remain open. Full
accessibility and actual little-screen-clip adoption remain deferred.
