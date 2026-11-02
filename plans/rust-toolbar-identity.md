# Bounded toolbar identity — 2026-11-02

## Contract and implementation

Swift RepositoryMenu uses one middle-truncated line with a maximum width of 140
points. Rust passed the full repository and branch strings into ordinary widgets,
letting a long repository name push the other toolbar controls outside the window.
The actual app regression reproduces missing Pull Request at the 800-point minimum.

The repository button now uses the existing text_fit::middle mechanism with 140
points including horizontal button padding. The branch uses the same mechanism at
80 points, retaining its existing 10.5-point secondary style; repository text stays
13-point strong. Display-only CR/LF becomes spaces and the existing bounded text
helper limits shaping input. UTF-8 name ends remain visible. RichText receives the
fitted repository text so MenuBar's current-owner selection color continues to work.
Bounded metadata explanations provide hover help through the existing helper.

Original name/recent-path values still own menu scope, and commands keep original
repository paths. Copy Repository Path never copies the elided title. No new menu
owner, width engine, font, palette, dependency or sibling change is introduced.
The requested SwiftUI/macOS/type skills were searched again and remain unavailable
locally; Swift source and established Rust type roles were used directly.

## Verification

The new actual-app regression covers 1×/2×; 800/1100/1400-point widths; light/dark;
checkout and real pinned-review state. Long Unicode/newline names stay on one row,
repository/branch extents are bounded, common toolbar controls remain visible,
review/Exit controls remain present and selected-title color matches the owner.
Pointer opening and Copy retain the exact canonical repository path in every case.
A sized frame driver extracts the existing raw-input/render path for reuse by this
matrix; existing focus tests keep their original dimensions and callbacks.

Initial RED: missing Pull Request with the old title rendering. The strengthened
branch assertion initially assumed the entire feature/ prefix survived an 80-point
fit; corrected to assert the visible prefix, one row, width and retained -done end.
This was a fixture expectation, not a product defect.

Final full workspace: 629 app tests, seven ignored; two binary, 13 font, 19 text and
40 doctests pass. Strict Clippy/check/build/rustdoc/fmt/diff pass. Initial doctest
linking exhausted /tmp; rerun with TMPDIR under ~/.cache/radiology-qa-rustdoc-tmp
passes all 40. Copied standalone resources verify outside the repository.

Native private Weston/Niri v3 runs at 800×700 logical points pass old/new 1×/2×.
Old captures show the title extending across the toolbar and hide Critical and
Inspector. New dark/light captures show all controls; each new run opens the
repository menu, copies the exact original directory path and verifies clipboard
SHA-256 before another capture. Fixture (including Git) and executable hashes are
unchanged, and app/compositor exits are all zero. Captures inspected.

Native v1 closed stdin immediately and terminated the app during cleanup; v2's
acceptance reader treated the pointer tool's plain-text response as JSON and
aborted. Those attempts remain archived, and their orphaned owned child groups
were stopped. Final v3 accepts both response forms and closes the app explicitly.
They do not count as successful acceptance.

## Architecture review and remaining work

Baseline before-bounded-toolbar-identity at 23ea206: 87,010 NLOC, 3,666 functions,
average CCN 3.34, max 132, 109 clone groups. Final NEUTRAL: +115 NLOC/two functions
including regression/sized test driver; average/max CCN unchanged, no new hotspot,
cycle or clone group, health -0.01. Existing measured text and menu ownership are
reused; the apparent toolbar diff is enlarged by formatting the chained response.

Native captures expose source-footer label overlap at the 800-point minimum with
both side panes open. That is a concrete separate follow-up; this increment bounds
the toolbar identity and does not certify all narrow-pane layout. Broader toolbar/
owner transitions, Unicode/UI/type roles, corpus and full Swift acceptance remain
open. Full accessibility and actual sibling adoption remain deferred.

Evidence archive: ~/.local/state/radiology-qa/toolbar-identity-20261102-23ea206.
