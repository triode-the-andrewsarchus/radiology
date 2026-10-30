# Recent pull-request chooser and review controls

Implementation `99c0ae4`; tooltip follow-ups `cb106b0` and `87e31b8`, all pushed.

## Swift contract and Rust ownership

The reference is Sources/Radiology/UI/PullRequestMenu.swift and the recent-list
lifecycle in PullRequestModel.swift: 25 recent entries, state/title/number, Local
Checkout, an active review absent from the list, a number prompt, independent
Refresh PR and Refresh List actions, and retained accepted data during failures.

RecentPullRequests owns accepted entries, a repository identity and one LatestRead
request independently of review preparation. Opening a new root clears old entries
and starts a bounded list load after checking for an origin. Reattaching the same
root preserves its list, including Exit PR. Refresh, cancellation and failure retain
accepted rows; a successful empty result replaces them with an honest empty state.
Root switching retires pending work before discovery, preventing old-root results
from entering the new reader. Newer accepted review metadata updates existing rows.

The cancellable GitHub client sends --state all --limit 25 and preserves returned
order. Zero, duplicate or excess numbers reject the response. JSON/process failures
retain the previous list, with recoverable error text. Missing gh includes install/
authentication guidance. Listing shares the existing two-minute control and bounded
stdout/process-group runner; it does not block rendering or refresh a review.

## Presentation and input

PullRequestMenu borrows accepted review metadata, list entries and phase and returns
shared ReaderCommand actions. The widget owns no job or repository. The toolbar
uses a compact #number label with title/state help, explicit Exit PR, wrapped list
rows and a bounded scrolling region. The active review remains selectable when
absent from the 25 recent entries. Loading, failure, unavailable, cancelled and empty
states are distinct; retained entries are labeled as the previous list. Error text
is bounded to 400 characters with an ellipsis, avoiding an unbounded popup.

Refresh PR and Refresh List remain separate. Loading disables duplicate list refresh
and offers Cancel List Load; accepted rows stay usable. Keyboard focus reveals
scrolled rows. The number prompt pre-fills the accepted or first recent PR, rejects
empty/invalid/zero/overflow values, and lets keyboard focus reach Cancel. Selecting
a different PR with identical head/base commits changes session and bookmark scope;
metadata-only acceptance now also requires the same PR number.

Header help remains available while closed and is suppressed while popup contents
render. The first tooltip guard used egui 0.27's returned inner value, which stays
empty during an open root menu; the final guard observes actual content rendering.
A complete popup test covers that framework behavior. Existing text styles and
palette remain in use. The requested SwiftUI/macOS/typography skills are absent
from the configured roots; no substitute skill or new type scale was invented.

## Automated verification

Final full gates: 365 app tests pass (one external-Clang fixture ignored), eight
shared reader-text tests and 30 doctests, formatting, strict Clippy, build and
rustdoc. New tests cover:

- Live list-child Cancel/timeout with reaping, the exact 25-row request, ordering,
  malformed/duplicate/zero/overflow responses and fresh recovery.
- Retention on refresh/failure/cancel, accepted empty results, metadata updates,
  root replacement and rejected retired publication.
- Recent/fallback/local selection, command routing and enabled states, wrapped
  long names, bounded errors, empty states, and dark/light rendering.
- Keyboard selection and scroll reveal through the final recent row, number
  validation, Cancel focus, same-commit PR bookmark/session isolation, and popup
  tooltip suppression with closed-header help preserved.

Logs: `/tmp/radiology-pr-chooser-{runtime,all-tests,review-tests,menu-tests,prompt-tests,clippy,build,doc}.log`
and the final `/tmp/radiology-pr-chooser-tooltip-{tests,clippy,build}.log`.

Architecture baseline `before-pr-chooser`: no new high-complexity function or cycle;
clone groups fall 91→90 after sharing live-child checks, PR fixtures and frame
rendering. Comparison labels a regression from four unchanged baseline fragments
in the touched modals file (8/7/6/12 lines). Their complete text matches the baseline;
none belongs to the new chooser. The existing number/line focus clone is eliminated.
Valknut health changes 79.4→78.8; no new threshold finding accompanies that change.

## Native acceptance

A private 1280×900 Wayland session uses local Git and controlled gh responses.
It automatically loads 25 entries with Open/Merged/Closed states and a wrapped
long title. Selecting #7 opens pinned After bytes. Refresh List starts live PID
539635; Cancel reaps it while retaining the accepted list and PR reader.
Authentication failure retains both; Refresh List recovers. A successful empty
list retains #7 as the active fallback and reports no recent entries.

A bookmark made in PR #7 is absent after choosing merged PR #8 with identical
commits, with the inspector scope correctly changing to PR #8. The number prompt
opens #99 outside the list; it appears as the active fallback. Explicit Exit PR
restores checkout-only source and keeps the list. Exactly five list requests
occurred in this initial sequence, including the initial load; Exit PR issued none.
Checkout HEAD, dirty status and source bytes compare unchanged. Final restarted
captures verify light and dark menus with the pointer over the header and no
overlapping tooltip. The tracked app and compositor exit statuses are zero.

Artifacts: `/tmp/radiology-pr-chooser-native/manifest.json`, `child-results.json`,
`initial-stage.json`, and the named screenshots hashed in the manifest, including
`list-loading-retained.png`, `list-failed-retained.png`,
`empty-list-active-fallback.png`, `merged-pr-eight.png`, `outside-list-menu.png`,
`exit-preserved-list.png`, `final-light-tooltip-clear.png` and `accepted-dark-menu.png`.
Initial app SHA-256 (`99c0ae4`):
`17a7eb41a987c287bda0c16e80b923d7a6450ce17559ddf16205aa237d6b2f57`.
Final app SHA-256 (`87e31b8`):
`c0602522df75993b85a6590cee9145525d09f97188353bcd3c330504e849b358`.
Driver: `/tmp/radiology-pr-chooser-native-driver.py`; fixture:
`/tmp/radiology-pr-chooser-native-fixture.py`. Configuration/cache are isolated.
Sibling capture helpers are QA tooling only; the app remains independently built.
Temporary artifacts are local evidence, not repository assets. No network PR writes
or native two-minute timeout run were performed; timeout evidence is automated.

## Remaining scope

[Private-store writer ownership](rust-pr-store-recovery.md) now covers cooperating
preparation and surviving children, with native crash/retry acceptance.
Full parity remains open: complete cache reuse/retention and broader corrupt-store recovery,
index provisioning/version/project provenance and pinned After indexing, PR refresh
caret/viewport/fold/pin/rename mapping, broader accessibility/scaling and final
Swift comparison, and sibling palette/font/chrome reuse. Existing default fonts
still lack some emoji glyphs (the native crab fixture shows a missing-glyph box);
string preservation tests do not prove glyph coverage. The full queue is maintained
in [rust-parity-implementation.md](rust-parity-implementation.md).
