# Same-file definition previews from prepared source

Reference: `ContextModelTests.utf16CaretOffsetsResolveOneIdentityAcrossUnicode`
and Swift ContextModel's immediate same-file preview path, compared with Rust
`1511674` and this checkpoint. Together with the prior cross-file preview group,
two of eighteen ContextModel groups now have explicit mapping. The remaining
sixteen groups and other suites/final acceptance stay open.

## Missing behavior and correction

The new both-scale app regression published only symbol metadata after moving to
the Unicode café call. It failed because the cold same-file preview remained
Loading: Rust started a second worker to parse source the reader had already
prepared (`red.log`).

The app now supplies its accepted buffer and prepared outline to the preview
owner when symbol metadata publishes. The loader chooses the memory path only
for a same-file definition whose supplied source identity matches the captured
buffer's path, revision and digest. Other uncached targets retain the existing
worker path. A mismatch in prepared ownership cannot bypass source preparation.

Both routes use the same source/index/range validation, declaration-end selection,
bounded excerpt builder and result publication/cache bookkeeping. A same-file
read uses SourceQueryInputs' existing captured-buffer precedence, so this path
performs no filesystem read, parser call or structure-mutex acquisition. The
outline is borrowed for the call; no additional AST or whole-source copy is kept.
A valid memory preview can replace an already pending request, retiring its old
publication channel before publishing the accepted result.

Metadata itself still prepares on its existing worker. The correction makes the
excerpt ready when that metadata publishes, without an additional excerpt job.
No new worker, pool, public input field, dependency, disk cache or UI option.

## Assertion mapping and ownership checks

| Boundary | Evidence |
| --- | --- |
| Complete Swift Unicode-caret group | New app test uses Swift's café/main/rocket/total source and an exact UTF-16 index. All four valid UTF-16 offsets inside café retain one Context identity, signature, main declaration and expanded state. Only metadata is polled; the cold preview is already Loaded at line 1 with exactly the café definition and no excerpt worker. The indexed byte range maps to exactly café. |
| String/surrogate and local fallback | A split surrogate is rejected by Rust's existing strict coordinate policy. Moving to the rocket's valid scalar start produces no semantic target/preview. Local total retains its local identity/name and starts no definition-preview work. These are platform equivalents of Swift's clamp-to-rocket/no-target and quiet-local assertions. |
| Accepted source changes | With the same explicitly unverified fixture index, the file changes from body value 1 to 2 and reloads through the actual app. The next cold same-file preview immediately shows value 2 and the fresh digest; old excerpt/outline data cannot leak. Source Arc/history remain unchanged during previewing, and the preview path does not write source. |
| Pending worker promotion | A real request is pending while the structure mutex is held. Matching prepared facts replace it immediately, require zero extractions and no mutex acquisition, and leave no old channel to poll afterward. |
| No filesystem dependency | The same loader test captures a checkout root inside a private temporary directory that was never created. A valid same-file preview still loads; any attempted storage read would fail. |
| Invalid coordinates/Before | Bad producer columns immediately report Unavailable, and Before never publishes After semantics. No worker is needed for these memory-path rejections. |
| Wrong prepared owner | Old three-line symbols are paired with the old buffer while current source has a five-line body. The loader rejects that prepared owner, runs the current-source worker and returns lines 1–5 after one real extraction. |
| Cross-file misses | Supplying current-file symbols for another file starts a worker; missing storage reports Unavailable instead of inventing a current-file preview. Existing cross-file structure/reuse checks retain their behavior. |

Rust continues to reject invalid UTF-8/UTF-16 producer positions rather than
clamping them, preserving the previously documented exact-coordinate policy.
The app test's metadata-only wait helper is shared with the existing result-reuse
regression, ensuring an excerpt worker cannot accidentally satisfy these checks.
A shared preview-line assertion removes duplicate test logic while retaining full
line-number checks and strengthening the stale-owner expected bounds.

## Verification and architecture

Archive: `~/.local/state/radiology-qa/memory-preview-20261109-1511674/`.
The initial app regression failed with Loading after metadata publication. The
first implementation missed a DocumentSymbol import; the compiler caught it and
it was corrected before focused checks. Thirteen preview tests and both app tests
pass, including the extended Unicode/reload and memory-only ownership checks.
Strict all-target Clippy, normal build and warning-free docs pass. The full
locked/offline serial workspace attempt reaches 884 passing app tests and eleven
existing ignored checks, with one failure in the existing Clang provisioning
interruption test: its curl descendant PID is still present immediately after
joining the installer. That test passes in isolation. The workspace stopped there,
so that initial attempt does not establish a full workspace pass. Process cleanup and the
test's direct-child-versus-descendant contract need separate follow-up; the new
preview tests passed in the full attempt. The subsequent
[installer process-contract correction](rust-installer-process-contract.md)
passes the fresh complete workspace run, including all 885 app tests and 66
doctests, and verifies this same preview implementation.

Architecture adds 289 measured NLOC/seven functions and no files. The additions
are the prepared request route, one shared publication path, reusable test waits/
assertions and three regressions. Average/max CCN remain 3.42/132 with unchanged
36 hotspots and cycles. Complete normalized clone multisets retain 124 instances,
zero additions/removals after consolidating the new test assertion duplication.
Existing source identity, captured reads, range validation, parsed declaration
lookup, excerpt bounds, cache and publication machinery are reused. The public
SourceQueryInputs fields/path and ordinary request entry point remain unchanged.

Remaining work includes the sixteen ContextModel groups, reference/call-tree/
palette assertions, source/syntax, compiler/indexing and inspector reconciliation,
then final native/standalone acceptance. Known parallel timing/linking limits and
Android remain open. Full accessibility and actual sibling integration are deferred.
