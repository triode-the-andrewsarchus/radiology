# Rust file picker parity

## Search foundation accepted — 2026-11-02

Repository inventories are immutable `Arc<[RepositoryFile]>` snapshots. Opening a
replacement repository publishes a fresh snapshot only on acceptance; failure or
cancellation retains the previous inventory identity. Review acceptance publishes
its inventory in the same form. Hosts assigning `all_files` convert a Vec with
`.into()`; slice readers continue to work through dereferencing.

The existing bounded file ranking implementation now supports cancellation before
normalization, after normalization, every 128 candidates and before publication.
Cancelled work returns no partial matches. The public synchronous search method
still uses the same ranking implementation with cancellation disabled. Zero limits
retain truthful totals, and the existing visible limit remains 100.

This is groundwork: the picker still searches synchronously during rendering and
shows its old 12-result interface. No native UI improvement is claimed here.

## Remaining picker acceptance

Connect immutable inventories and cancellable ranking to the existing latest-read
worker mechanism. Retire stale query/inventory results and close-time work. Match
Swift's filename/directory rows, 100-result cap and truthful totals, empty states,
Open/Cancel controls and keyboard reveal. Clamp selection before Enter, preserve
ordered keyboard ownership, bound long paths and protect row identity across
query changes. Verify rendered behavior and native keyboard/source-focus flows.

## Foundation verification

Fresh workspace run: 583 application tests passed, seven ignored; two binary,
four font, 13 text and 37 documentation tests passed. Strict Clippy, all-target
check, binary build, rustdoc with warnings denied, formatting and diff checks pass.
Cancellation regression stops after partial ranking and verifies fresh result
order/counts; the repository transaction test checks retained/replaced Arc identity.
Architecture against `before-file-picker-search`: NEUTRAL, +51 NLOC, two functions,
no new hotspot or cycle. All 110 clone groups are unchanged by normalized audit.
No additional dependency or worker pool is introduced.

Evidence: `~/.local/state/radiology-qa/file-picker-foundation-20261102-699a55e`.
Full accessibility and actual sibling integration remain deferred; broader parity
remains open.
