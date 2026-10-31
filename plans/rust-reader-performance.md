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


## Sparse source coordinates — 2026-10-31

`reader-text::CoordinateIndex` now stores absolute byte, UTF-16 and scalar
checkpoints approximately every 4096 bytes, plus EOF. It borrows host-owned text
for lookups and has no GUI or application dependency. Construction is one pass;
each lookup binary-searches checkpoints and scans at most 4099 bytes. Storage is
roughly 24 bytes per 4 KB on this 64-bit host. Global offsets include original
terminators; SourceBuffer subtracts the indexed line-start offset and enforces its
existing terminator-excluding line ranges. The existing ColumnEncoding paths
re-export the shared enum. Immutable text, line indexes and cached source identity
remain one snapshot; rebuild the buffer after content changes.

This replaces both UI-thread scans found in the previous profile: encoded
line/column to byte and byte to scalar column. Invalid lines, oversized columns,
overflow, split UTF-8 scalars, UTF-16 surrogate halves, clamping, empty/trailing
lines and CRLF behavior are retained. Shared tests compare every byte/unit against
independent encoding counts across multiple checkpoints; a >6 MB fixture checks
sparse storage and maximum checkpoint gaps. SourceBuffer tests separately cover
line-relative conversions and offsets on both CR and LF.

Workspace passes 514 application tests (seven ignored), two binary, four font,
13 reader-text and 36 doctests. Strict Clippy, all-target check, build, fmt and
whitespace checks pass. Warning-denied workspace rustdoc passes; rendered public
CoordinateIndex/ColumnEncoding documentation and the runnable example were
inspected. Baseline `before-coordinate-index` at `01f8519` compares NEUTRAL:
+218 NLOC including tests, 91 unchanged clone groups, no new hotspot/cycle or
maximum-CCN increase.

The unchanged native 6,291,502-byte probe passes full original copy, exact End
suffix, cross-file Back suffix and wrap-preserved selection at both 1× and 2×.
Every copy starts from a fresh private clipboard sentinel; fixtures remain
unchanged, binary hashes match the final build and child exits are zero. Observed
waits at 1× are 8.7 s initial copy, 4 ms End suffix, 9.4 s Back suffix and 2 ms
wrapped copy; at 2× they are 9.1 s, 22 ms, 9.8 s and 2 ms respectively. Prior
01f8519 1× observations were 8.5 s, 2.8 s, 12.5 s and 1.6 s. These single debug
runs include polling/repeated readiness keys and software rendering, and establish
neither release latency nor a statistical speedup guarantee.

Coordinate conversion is bounded, but initial/revisited analysis still takes
seconds. Immediate unwrapped captures retain LONG_START even after the exact End
copy succeeds. A separate delayed-capture probe reaches the tail area after
approximately 30 seconds; the complete selected suffix is not visible in that
capture. Horizontal measurement/reveal therefore still needs investigation and
visual acceptance. Wrapped scroll/reveal, many-line performance and broader
scaling/Back acceptance also remain open. Do not treat exact clipboard bytes as
proof of completed viewport reveal.

Evidence: `~/.local/state/radiology-qa/coordinate-index-20261031-01f8519`.
