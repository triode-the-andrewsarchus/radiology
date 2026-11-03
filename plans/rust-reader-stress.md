# Large reader transactions and fragment boundaries — 2026-11-02

## Failure and cause

A real source-worker/app-frame workflow opens a5,850,053-byte C++ Unicode comment
line while a file picker or Code Path import field owns input. Ownership and exact
copy already pass, but unprofiled publication frames take485–506 ms. GDB entry/return
breakpoints confirm485–496 ms inside `reader_text::projected_fragments`, reached
from the viewport's source synchronization. Almost the whole slow frame is splitting
source into display fragments, rather than parsing/highlighting on the worker.
Perf events are denied by the host; no system settings are changed. The installed
DAP CLI lacks its Rust backend, so the existing GDB supplies call-stack/timing evidence.
The first GDB probe's unsupported Rust-value inspection was corrected; the retained
successful trace and timings are the causal evidence.

`fragment_prefix_len` previously enumerated graphemes from each fragment's start
until the proposed byte cut. Ordinary Unicode therefore paid thousands of boundary
lookups to decide one cut, repeated across the complete source during publication.

## Shared text repair

The existing unicode-segmentation `GraphemeCursor` now checks the proposed boundary
and, if inside a cluster, its adjacent boundaries. It uses the same bounded prefix
context, scalar-aligned cut, four-byte lookahead and oversized-cluster policy.
Fitting graphemes stay intact; an oversized cluster can split at a scalar boundary.
Callers remove row terminators before this private check; existing public fragment
and projection tests retain LF/CRLF coverage. There is no additional worker, cache,
projection owner, dependency or public API. Independent reader hosts share the fix.

A new oracle compares boundary results with full grapheme segmentation across
budgets/limits, combining marks, modifiers, family ZWJ emoji, odd regional-indicator
runs, prepend characters and Indic conjuncts. Existing oversized-combining-sequence,
fragment/projected-span, Unicode/CRLF and source-coordinate tests remain in place.
The initial direct oracle included a CRLF pair inside a display span; callers have
already removed it. That test-domain error was replaced by an in-line tab case;
public CRLF behavior remains independently checked.

## Actual app workflow

The checked-in stress test uses the existing isolated source helper and real source
workers. Two corpora—one5,850,053-byte Unicode line and a5,520,031-byte,60,002-line
file—run with both modal owners at1×/2×. While each source is pending, the previous
reader remains accepted. On publication the authored modal prefix receives its
suffix, and Escape restores the source owner. Full Select All/Copy and wrap preserve
all bytes/history. A later large request is superseded by navigation within the
accepted anchor; its retired channel cannot replace the reader. Disk bytes stay exact.

Final dedicated app-frame observations (microseconds):

| Shape | Scale / owner | p95 | Maximum |
| --- | --- | ---: | ---: |
| Long line |1× / picker |1391 |5004 |
| Long line |1× / import |1465 |4537 |
| Long line |2× / picker |1360 |4883 |
| Long line |2× / import |1319 |4433 |
| Many lines |1× / picker |1333 |20744 |
| Many lines |1× / import |1301 |14331 |
| Many lines |2× / picker |1310 |20920 |
| Many lines |2× / import |1301 |13706 |

These are `ctx.run` app publication/UI-layout measurements before native
rasterization, in single debug runs on this host. They are not release/statistical
latency promises or timing assertions in portable CI. The initial same long-line
workflow's maximum was485–506 ms. GDB after the repair separately observes roughly
1.3–1.6 ms projection for the long source and8.3–8.9 ms for the many-line source.
All app invariants pass; the timing experiment did not replace correctness checks.

## Native observations

Fresh private Weston/Niri runs at1×/2× show the real Opening long.cpp notice while
retaining the anchor, then preserve the22-byte café/emoji import draft through source
publication. Eight exact clipboard checkpoints per scale cover the full5,850,053
source bytes, LONG_END find selection, wrap, font increase/reset, the line60001
function and the retained anchor immediately and two seconds after superseding work.
Back visits the many-file start then returns to the large wrapped source with its
tail visible. Candidate source/input states, wrapped tail, reset, Back, many-file
end and retained source captures were inspected. All source/executable hashes are
unchanged and app/Weston/Niri exit zero.

Thread snapshots observe four Tokio read workers throughout. Other threads belong
to the software-rendered desktop stack. Scoped app RSS spans216,911,872–288,686,080
bytes at1× and251,297,792–339,529,728 at2× across this workflow; CPU/thread ticks and
monotonic samples are retained. This records allocations/cache retention during
interaction; it does not establish a long-run leak bound. Cargo's half-core setting
remains16 jobs, and the app read pool remains four workers.

## Verification and architecture

Full locked workspace passes760 app tests/nine ignored, two binary/13 font/22 text
tests and59 doctests. After replacing duplicated setup with the existing fixture
helper, the targeted eight-case stress workflow passes again. Strict Clippy/build,
resource smoke, formatting and diff checks pass. The first new test fixture clone
was removed by reuse, without changing its exercised publication/input behavior.

Against `before-reader-stress-owners` at17bb068: +171 NLOC/two test functions, one
new test module. The production boundary function grows12 NLOC but CCN9→7. Average
CCN3.38→3.39; max132,36 existing above15 functions, all125 clone instances/1335
duplicated lines, one cycle and eight structural findings remain unchanged. The raw comparator's
REGRESSION flags existing shifted fragments in the touched test module and a rank
change in untouched Swift Python; the complete normalized fragment/file-pair Counter
is identical. The stress test is CCN8; no new hotspot or coupling is introduced.

Evidence: `~/.local/state/radiology-qa/reader-stress-20261102-17bb068` contains the
before/after unprofiled workflow, debugger scripts/traces, denied-perf diagnostic,
Unicode oracle/full checks, native actions/captures/bytes, hashes, thread/CPU/RSS
samples and architecture records. QA artifacts stay outside `/tmp`.

This accepts the representative large-source/pending-owner checkpoint together with
the preceding twelve-fixture source corpus and existing long-line input regressions.
Other behavior reconciliation, integrated moving PR/revision/semantic/cache flows,
dependency-heavy compiler outcomes and the final clean standalone/reuse audit remain
open. Actual sibling adoption and full assistive accessibility remain deferred.
