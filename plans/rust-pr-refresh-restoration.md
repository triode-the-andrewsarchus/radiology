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

## Native acceptance

Implementation `275d8db` passed a private 1280×900 Wayland sequence. A 202-line
function was opened at caret line 100, with content row 80 at the viewport top and
200 helper occurrences pinned. A pure rename to `moved café.rs` preserved that
caret, visible source row, pin count and teal color. Two inserted lines then moved
the caret to line 102 and the viewport anchor from source line 81 to 83 while
retaining content row 80 at the top. Changed bytes dropped the pins as specified.
A same-commit refresh preserved the reader and showed the up-to-date notice.
Controlled metadata failure retained the accepted path/line/source; a fresh retry
recovered. Six metadata view requests were observed. Checkout HEAD, dirty status
and source bytes compare unchanged. App and both compositor exit statuses are zero.

Artifacts: `/tmp/radiology-pr-refresh-native/manifest.json`, `acceptance.json`,
`heads.json`, `gh-requests.jsonl`, and hashed screenshots `pinned-before-refresh.png`,
`renamed-restored.png`, `inserted-restored.png`, `up-to-date-retained.png`,
`metadata-failure-retained.png` and `recovered-up-to-date.png`. Caret/viewport/pin
observations are visual checks, not OCR assertions. Before/base movement, revision
fallback, failure during source preparation and Cancel remain covered by automated
tests rather than this native sequence. Controlled local gh responses and Git
fixtures make no remote PR writes. Sibling capture helpers are QA only.

The metadata-failure capture exposed oversized multiline error notices and
unavailable revision controls. Both are now addressed, including native acceptance:
see [reader notices and control availability](rust-reader-notices-controls.md).

Full parity remains open:
complete index provisioning/project/version reuse, broader corrupt-store recovery,
and the final UI/accessibility/scaling/font and sibling reuse audit.
