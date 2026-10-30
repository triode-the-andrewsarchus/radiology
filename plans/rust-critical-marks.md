# Critical Lens decisions in Rust

## Storage and replay contract (2026-10-30)

`critical::overrides::OverrideStore` stores one bounded JSON file per canonical
repository root under an explicitly configured private settings directory. SHA-256
root keys and a root field isolate repositories; source-root aliases resolve to
the same decisions. Construction and missing-file loads create nothing. Storage
inside the viewed repository, foreign roots, unsupported formats, corrupt/oversized
JSON and nonregular/final-symlink inputs are rejected. The Rust disk schema is
private; this does not promise Swift JSON format compatibility.

Nonempty snapshots serialize to a synced temporary file and replace atomically.
Empty snapshots delete the file. Failed writes retain the previous file and report
an error. Clones share save ordering: reserve a revision when accepting an edit,
before scheduling its snapshot. A delayed lower revision cannot overwrite a newer
attempt, including after deletion or a failed newer attempt. This ordering applies
to the store and its clones, not independent processes/stores.

`MarkEdit` separates stable symbol and version-independent package identities from
decisions. Critical/Ordinary persist; None/Unknown restores built-in behavior.
Locals and empty identities never persist. Loaded snapshots are normalized to the
same rules. Edits can replay on top of a delayed load without replacing unrelated
stored decisions. `CriticalityOverrides::default` now correctly selects format 1
instead of derive(Default)'s accidental 0.

The implementation extends the existing regular-file, cancellation-aware bounded
reader and reuses workspace destination-path resolution. Index cache paths now use
that same resolver, removing their duplicated ancestor traversal. No additional
writer thread, runtime, database, configuration write or dependency was introduced.

## Verification

Seven tests cover atomic round trips, no idle writes, root/alias isolation,
normalization, obsolete and concurrent save ordering, deletion, failed newer saves,
corrupt/foreign/unsupported data, bounded/symlink input, cancellation and pending
edit replay. Full validation passes 164 application tests, 5 reader-text tests,
30 doctests, fmt, Clippy with warnings denied and binary build. The external-Clang
acceptance test remains ignored by default. Architecture comparison retains a
seven-line fixture clone; no new product clone/CCN hotspot/cycle is reported.
Maximum CCN remains 100, with about 0.7% duplication.

## Session lifecycle (2026-10-30)

`MarkSession` uses replaceable background loads and a single coalescing durable
writer on the existing worker pool. Pre-load edits replay over stored decisions;
repository switches and Drop retain accepted edits. Controlled shutdown can drain
with a deadline. Errors remain repository-scoped and failed saves retain session
decisions. Exact-source menu targets reject locals, Before and mismatched bytes.
SCIP package headers and cached repository ownership are shared with the index,
avoiding a whole-index scan during menu rendering.

Four lifecycle tests cover delayed loads/switches, rapid decisions, shutdown,
failure isolation and Drop; a semantic menu test covers identity guards and
cached ownership. The complete working tree passes 169 application tests,
5 reader-text tests, 30 doctests and Clippy. UI integration remains under
verification and is deliberately separate from this backend commit.

## Reader integration and acceptance (2026-10-30)

The app owns one mark session per attached canonical repository. Source Normal
and Critical context menus expose symbol/package Critical, Non-Critical and
Built-In choices. Targets come from exact SCIP occurrences, including encoded
range/source checks; locals, Before and mismatched sources cannot be marked.
The app revalidates the current target before accepting an action. Edits and
loaded decisions refresh the existing replaceable projection without modifying
source bytes, caret, history, folds or pins. Load/save errors appear in the source
footer with details on hover. Failed persistence retains session decisions.
Controlled eframe shutdown cancels competing readers/index jobs and drains marks
with a ten-second deadline; embedded hosts can explicitly flush the same service.

An application test imports real SCIP wire data, exercises package/symbol
precedence and both resets, checks reader-state preservation and stale-target/
Before rejection, then constructs a fresh app and verifies restored policy and
projection. Full checks pass: 170 application tests, 5 reader-text tests,
30 doctests, rustfmt, Clippy with warnings denied and binary build. One external
Clang acceptance test is ignored by default, as before.

Private native Weston/Niri acceptance uses a UTF-16 SCIP fixture with an emoji
before the selected call. Right-clicking clone selects that exact occurrence;
Normal mode offers the symbol/package menus and current Built-In explanation.
Choosing Critical makes clone prominent in Critical mode. Closing/restarting the
app restores both projection and the selected Critical menu item. Built-In resets
clone to its ordinary receded presentation; package Critical makes it prominent
again. Native artifacts are under `/tmp/radiology-mark-native-2/`, including
`choices.png`, `restored-mark-menu.png`, `built-in-restored.png`,
`package-choices.png` and `package-critical.png`. Both app runs and compositors
exit zero. The first test-driver attempt timed out on its pointer move helper;
its cleanup terminated that app. The successful run uses clicks for submenus.
No sibling source was edited and no runtime sibling dependency was introduced.

Architecture comparison against before-mark-reader-integration reports +772 NLOC,
unchanged maximum CCN (100), average CCN down 0.02, and three additional clone
groups (+0.01 percentage points duplication). Remaining reports are short test
fixtures/assertions, common footer label sequences and one required error-display
branch in SourceView::show (23 to 24). The index ownership collection adds no
branch. No new scheduling runtime or whole-index menu scan is introduced; the
reports are retained rather than claiming an architecture-neutral result.

Call graphs/frontiers, hover/count/fold annotations, broader error UI/runtime
acceptance and full Rust/Swift acceptance remain open requirements. Persisted
decisions do not establish completion of those workflows.
