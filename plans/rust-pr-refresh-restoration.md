# Rust active PR refresh restoration

The reference contract is Sources/Radiology/AppModel+PRRefresh.swift and
[the Swift PR refresh notes](pull-request-review.md#active-pr-refresh-2026-10-06).

## Preparation and atomic publication

Same PR/head/merge-base acceptance updates metadata only and reports “PR is up to
date.” Changed head or base queues a candidate snapshot while the old reader stays
accepted. The existing LatestRead job owner prepares restoration with a shared
30-second cancellation/deadline budget. Both old and candidate snapshots retain
their tree leases. Current source loading must settle first. If the source buffer
or appearance changes during preparation, the candidate is reconciled again.
Caret, viewport and pins are read at commit time, not when Refresh was clicked.
Same-PR refresh keeps accepted-reader controls usable; another PR retains its
separate transition behavior.

Git old-to-new commit rename records choose the path. Exact source equality keeps
canonical byte offsets. Otherwise an exact blob-to-blob zero-context diff reuses
FileLineChanges::map_line; column bytes survive only when the mapped line bytes
are identical, with UTF-8 boundary clamping. Before/After preference survives when
available; fallback gets an explicit notice. A file absent from both revisions
leaves the new Changes browser with a notice and no fabricated source destination.
Git failures or unreadable refreshed source preserve the complete accepted reader.

The existing source preparation pipeline analyzes the already-read pinned buffer,
prepares syntax/highlights/default folds and exact changes before review publication.
At commit, common review installation retires old source/index/search/analysis
publication, clears obsolete history and resets manual fold/selection memory.
Current-file pins and their colors survive only identical bytes; other file pin
memory and ephemeral selection are dropped. Fresh body/review fold defaults apply.
Caret and viewport anchors reveal through new folds/test hiding. The viewport
restores its mapped source row; later explicit navigation cancels that restoration.
Source mode, font, filters, expansion and an applicable sidebar tab remain.
The refreshed After index loads through the existing commit-scoped cache pipeline.

## Automated evidence

Full gates on 2026-10-30 pass 392 app tests (one ignored), eight shared reader tests,
30 doctests, formatting, strict Clippy, build and rustdoc. Seven new reader tests
cover exact renames and latest caret/pins, inserted/changed lines and UTF-8, navigation
during preparation, Before preservation/fallback/missing source, moving merge base,
cancellation/metadata-only acceptance, and failure retaining the old reader. A
measured viewport test checks source-row restoration and superseding navigation.
The final test-fixture consolidation was followed by the seven focused tests and
strict Clippy; production code is unchanged from the full gates.

Architecture baseline `before-pr-refresh-restoration`: no new complexity hotspot
or dependency cycle; clone groups remain 90 and duplication decreases. Comparison
labels a regression for four unchanged baseline fragments in the touched app test
file. New refresh fixtures share their initial revision setup; no new clone group
remains. Shared source preparation, line mapping, pin memory and bounded job
ownership are reused. Logs: `/tmp/radiology-pr-refresh-{tests,viewport-tests,all-tests,clippy,build,doc}.log`.

Native rename/caret/pin/viewport acceptance is pending. Full parity remains open:
complete index provisioning/project/version reuse, broader corrupt-store recovery,
and the final UI/accessibility/scaling/font and sibling reuse audit.
