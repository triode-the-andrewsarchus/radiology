# Files presentation follows accepted source navigation

Ports the upstream Files behavior in `files-sidebar-navigation.md` while retaining
Rust's independent repository inventory and source navigation transactions.

## Presentation and ownership

Accepted navigation requests one centered Files reveal, including same-file jumps.
Entering Files, changing its filters or showing a previously hidden sidebar also
reveals the current row. Parent directories are unioned into existing disclosures;
passive caret movement and manual directory/scroll exploration do not recenter.
The selected row is highlighted without reopening its source when clicked, so an
exact bookmark/search caret, Before revision and history survive synchronization.

The additive `Sidebar::request_file_reveal` API lets direct hosts request explicit
same-file reveals. Per-context UI state tracks presentation changes and retires the
request once its row is laid out. The app retains one pending navigation signal
while the sidebar is hidden. It uses the actual scroll viewport even on the first
paint, before egui knows content overflow; reveal is immediate rather than animated.

`refresh_file_tree` reuses repository filters and the existing tree builder, adding
only the successfully loaded document when absent from filtered candidates. It
never changes `all_files`, saved filters, quick-picker inputs or Changes/search
inventories. Query results are flat full-path rows, while an empty query shows the
tree. Moving to another document removes the previous temporary exception. Missing
reads cannot invent rows; a loaded unlisted file and deleted Before paths can appear.
Counts include the visible document and an extra snapshot-external document; the
footer explains filter exceptions. Before uses the existing small secondary type
role, with measured space reserved before middle elision.

The current Before row's file-manager action now resolves the pinned base tree.
Other rows use After during review, and checkout rows use the attached repository.
The existing verified revision cache, writer lock, cancellation and tree lease are
reused. A handoff holds the exact revision lease until the desktop acknowledges it;
no second cache, worker pool or unpinned checkout mutation is introduced.

## Verification

Actual-app tests cover 130 nested files at 1×/2×, centered explicit/same-file jumps,
passive scroll/caret retention, manual directory collapse, returning from Search,
and selected-row clicks preserving source Arc/caret/history. The test establishes
egui's requested zoom before measuring the mounted list and lets wheel smoothing
settle. It exposed first-paint outer-panel clipping; the real viewport repair avoids
a roughly 60-point centering error. Small clip-margin differences stay within a
fraction of one row.

Further tests preserve inventory and filters while revealing a filtered test/document,
a newly loaded file outside discovery and the last accepted document after a failed
read. Real Git PR tests retain a deleted Before row, caption, byte1, revision and
history through Files clicks at both scales. The revision-tree test proves exact
base bytes, cache reuse, tamper rejection/repair, cancellation and unchanged checkout.
The earlier long-row test now clicks an unselected row to retain canonical opening
and context Copy coverage; selected-row preservation is tested through actual frames.

Final workspace: 648 app tests pass (seven existing ignored), plus two binary,
13 font and 21 text tests. Strict Clippy/check/build/rustdoc, formatting, diff checks
and 45 doctests pass. Rendered public reveal API documentation was inspected.
Native checks compare previous/current binaries in private compositor sessions at
1×/2×, including automatic nested-row reveal, a nonmatching filter, exact source
Copy and light appearance. The native fixture covers checkout paths; deleted Before
behavior is covered by the real-Git app/service tests, not claimed as native coverage.

## Architecture review

Baseline `before-files-navigation-reveal` at `7ebb233`. Existing tree/filter/source
navigation, egui scroll targets and verified revision materialization were found by
reuse lookup before implementation. Directory recursion and leaf-row layout now have
separate cohesive render methods; sidebar preparation stays outside the app's main
render dispatcher. No new complexity hotspot or dependency cycle. The normalized
125 clone groups remain identical; default offset-based reports flag shifted old
clones. Final measurements/evidence are archived with the acceptance logs.

| Dimension | Baseline | Current |
| --- | --- | --- |
| Analyzed NLOC | 93,312 | 93,718 (+406, including tests) |
| Functions / files | 3,945 / 603 | 3,956 / 604 (new test module) |
| Average / maximum CCN | 3.37 / 132 | 3.38 / 132 |
| High-complexity functions | 33 | 33 |
| Clone groups / cycles | 125 / 1 | 125 / 1 |

The existing upstream Python chokepoint rankings fluctuate without source changes;
no new Rust hotspot/cycle was introduced. Normalized clone comparison uses paths,
token counts and exact fragments, not source line offsets.

SwiftUI/macOS/typography skills are absent from installed paths. Checked-in Swift
source and existing Rust type/layout roles provide the reference; no new type scale
or visual framework was added. Full accessibility and actual sibling adoption remain
deferred. Broader parity stays in `rust-upstream-parity-2026-11-02.md`.

Evidence: ~/.local/state/radiology-qa/files-navigation-20261102-7ebb233.
