# Repository search request presentation

Repository search now retires prior results and errors as soon as options change,
shows debounced work as active, and explains empty, working and failed states.
Explicit submission and Stop take precedence over automatic typing debounce.

## Reference and confirmed gaps

Swift `AppModel.performSearch` cancels the previous request, clears matches/errors/
limits immediately and marks the debounce as searching. `RepositorySearchView`
provides Search the Source, Searching, No Matches and Search Unavailable states.
Rust previously left the old request's presentation visible during its 150ms
schedule, used worker-only `running` for Stop, and had no empty-state messages.
The existing pinned-review lifecycle regression fails before the repair because
old matches survive a replacement schedule. Source inspection also exposed app
ordering that scheduled after handling explicit Go/Stop, replacing those actions
when options changed in the same frame.

## Shared lifecycle and UI

`RepositorySearch::is_searching` reports both scheduled and running work to any host.
`clear` cancels and clears matches/error/limit, while existing `stop` retains published
matches. Immediate and scheduled starts reuse `clear`; empty queries
leave no pending request. Existing LatestRead ownership, review leases, cancellation
and generation/channel retirement remain the mechanism. No new worker, queue,
dependency or framework coupling was added. The additive public status/reset API
includes a runnable isolated-host example.

The app now schedules option changes before dispatching explicit sidebar actions.
This lets Enter/Go run immediately and lets Stop/Clear cancel a newly scheduled
request. Query, checkbox and glob widgets report edits before rendering results;
old rows are neither painted nor activated in that editing frame. This also works
for direct sidebar hosts without the app's pointer/key frame staging.

The footer reports Searching with the current count while pending/running. Empty
states follow the Swift workflow. Failed requests hide results, use the existing
bounded three-row explanation and offer Copy Error for the complete diagnostic.
Existing type roles/colors and widgets are reused. SwiftUI/macOS/type skills remain
unavailable in the installed catalog; the checked-in Swift view and existing UI
mechanisms provide the reference. Full accessibility remains deferred.

## Verification and evidence

- Extended real-rg and pinned-review service tests prove immediate retirement of
  old matches/errors/limits, blank-query clearing, pending progress, Stop, snapshot
  lease release and retired-channel rejection. The old-state regression has an
  authoritative failing run before the production change.
- Actual-app frames at 1×/2× edit a query then deliver a staged click at the old row.
  Exact source caret/history remain unchanged and the old row is absent. Editing
  and Enter in one batch starts the worker immediately. The test uses actual
  primary-command field bindings and flushes the app's pointer frame boundary.
- Sidebar frames prove Stop during debounce without polling it into a worker,
  No Matches after cancellation, and bounded long errors with complete copying.
  Existing result fixtures now provide their real `needle` query. The first full
  run exposed a preview selector that also matched this populated query; it now
  identifies the fixture's `needle café` previews. The truncation/count assertions
  and canonical activation expectations are preserved.
- Final workspace: 641 app tests pass, seven ignored/zero filtered; two binary,
  13 font, 21 text and 43 doctests pass. Strict Clippy/check/build/rustdoc with
  shaping, formatting and diff checks pass. Rendered RepositorySearch API docs and
  copied standalone resources outside the checkout pass.
- Native previous/current binaries at 1×/2× reproduce missing old empty guidance.
  Current runs verify blank guidance, active progress/Stop, termination of an owned
  controlled rg process, No Matches, fresh result activation and exact Unicode/CRLF
  source copy. An invalid long regex yields bounded error UI and a fresh complete
  10,098-byte diagnostic copy matching real rg stderr. New queries recover, and
  light appearance plus clearing restore the correct messages. Captures inspected;
  fixture/binary hashes unchanged and all app/compositor exits zero. Native checks
  do not claim timing-sensitive sub-150ms observation of debounce; service/sidebar
  tests prove that phase directly.

Architecture baseline `before-search-request-presentation` at `f26c8ff`: +170 NLOC,
five functions (two public service methods, one private status renderer, two
regressions), unchanged files/average/max complexity/cycles; health changes by less
than 0.01. Default comparison reports REGRESSION for a shifted old row-allocation
clone. Complete normalized comparison proves all 110 clone groups unchanged, no
new/removed groups or production hotspot. Reset and label mechanisms are reused.

Archive: `~/.local/state/radiology-qa/search-presentation-20261102-f26c8ff`.
Broader Swift feature/UI/input/indexer corpus and native IME-engine acceptance remain
open. Actual little-screen-clip adoption stays deferred; Radiology remains standalone.

## Exact search inputs checkpoint

Swift uses isEmpty and passes query/glob strings verbatim. Rust now follows that
contract instead of trimming. Whitespace-only queries run; genuinely empty queries
clear the request. Include/exclude patterns retain leading spaces and escaped
trailing spaces. Ripgrep itself interprets gitignore-style globs: a literal trailing
space requires escaping, so the app must neither trim nor invent escaping.
SearchMatch.column documentation now correctly identifies a UTF-8 byte column.

Four service tests pass, including twenty real-rg combinations covering immediate/
scheduled whitespace, tabs, Unicode whitespace, exact offsets and space-bearing
filenames/globs. The corrected escaped-glob regression fails with old trimming.
App search tests pass with a source-only include fixture. Evidence is currently
under ~/.cache/radiology-qa-exact-search-inputs (service-green-v3.log and glob-red.log).
Full workspace/native acceptance and complete architecture comparison remain pending
while the requested upstream rebase takes priority. Earlier failed runs include
incorrect unescaped-glob fixture assumptions and /tmp exhaustion; they are not
acceptance evidence.

Post-rebase acceptance (2026-11-02): 642 app tests pass (seven existing ignored),
two binary/13 font/21 text tests and 43 doctests pass. All six strict gates pass.
Native current binaries at 1×/2× validate indentation, tabs, Unicode whitespace,
escaped include/exclude globs and a single-space filename, with exact CRLF copies,
empty-query recovery and light appearance. Old 1× reproduces ignored indentation
and the trimmed escaped-glob error. Binary/fixture hashes stay unchanged and all
owned child exits are zero. Driver corrections wait for Search before clicking
glob fields; earlier attempts are retained as harness failures, not product evidence.

Architecture comparison across the rebase includes upstream's 85 new analyzed
files, 15 additional clone groups and one reported Python dependency cycle. Its
new complexity hotspots are in imported Swift/Python; these must be distinguished
from the five changed Rust search files. No production Rust mechanism was added
by the search fix. A new post-rebase baseline isolates subsequent feature work.
Evidence archive: ~/.local/state/radiology-qa/exact-search-inputs-20261102.
