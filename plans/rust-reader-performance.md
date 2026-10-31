# Large-source reader acceptance and performance

## Verified starting point — 2026-10-31

At `50d0493`, a fresh private Wayland run opens a 6,291,502-byte Rust source with a
single Unicode line and original CRLF endings. Each clipboard assertion first
publishes a unique marker on the private display, then waits for the expected
payload hash. This prevents a prior text offer or screenshot from passing a copy
check. Copy All matches the entire original source; End/Shift-Left/Copy yields
`LONG_END`; navigating to peer.rs and Back, then Shift-End/Copy, yields that same
exact eight-byte suffix. A wrap toggle preserves the selected original bytes.
Fixtures are unchanged and app/compositors exit zero.

Observed debug-build probe waits are approximately 10.0 seconds for fresh full
source copy, 3.0 seconds for End/suffix selection, 12.7 seconds for Back/suffix and
1.8 seconds for wrapped copy. These are observations in a software-rendered private
compositor, including clipboard polling and repeated readiness input, not general
latency guarantees or a release-build benchmark. Exact copy does not prove that
all deferred horizontal geometry or viewport reveal has completed. Broader many-
line, wrap/scroll/scaling and visual Back acceptance remain open.

## Profile findings

The installed DAP adapter was unavailable in the earlier reader investigation.
GDB sampling reuses the private QA child's process-scoped debugger opt-in wrapper;
no product code or system ptrace policy changes. Full stacks are retained.

- During cold Rust loading, the syntax worker is inside Tree-sitter query analysis
  compiling **Swift** call patterns. The role cache eagerly compiles all nine
  languages on first use, even when only Rust was requested.
- At End and after Back, the UI thread is inside
  `SourceBuffer::byte_offset_encoded`, traversing more than three million Unicode
  scalars while applying caret reveal or source key movement. This is independent
  of query initialization and still requires correction.

The sampled loading frame alone does not establish that all initial loading delay
is query compilation. Remaining analysis/highlighting, projection and clipboard
costs must be measured separately. Profiled-run timings include debugger overhead
and are not used as the performance comparison.

## Per-grammar query initialization

The existing four role registries now hold an independent `OnceLock<Result<Query, String>>` for each language. Creating a registry only allocates empty cells. The
requested grammar's cell compiles and retains its query/error, with no mutex held
over another language's compilation. Existing query sources, role separation,
static references, callers, predicates and cancellation checks remain unchanged.
No new dependency or public API is introduced.

Two tests use isolated registries to verify lazy initialization across Calls,
Folds, Occurrences and Highlights, pointer reuse and eight concurrent requests
sharing one compiled query. Reintroducing equivalent eager construction fails the
unrelated-language test on Swift. This does not claim cancellation inside the
native Query constructor itself; an actively requested grammar still compiles on
the background worker.

The same unprofiled 6 MB native script after this change preserves all exact-byte,
Back, wrap and fixture/exit checks. Its observed initial copy wait is about 8.5
seconds; End remains about 2.8 seconds and Back about 12.5 seconds. One before/after
pair establishes a useful observation, not a statistical speedup claim. The long-
column scan remains the next confirmed defect.

Workspace passes 513 application tests (seven ignored), two binary, four font,
eleven reader-text and 35 doctests. Strict Clippy, all-target check, build,
formatting and whitespace checks pass. The native binary hash matches that build.

Architecture baseline `before-large-reader-acceptance` is at `50d0493`. Query-cache
comparison is NEUTRAL: +60 NLOC including tests, no added clone, hotspot, cycle or
maximum-CCN increase. Full objective acceptance remains open. Full assistive
accessibility and actual sibling adoption remain deferred; keyboard behavior and
standalone reuse preparation remain required.

Evidence: `~/.local/state/radiology-qa/query-lazy-20261031-50d0493`.
