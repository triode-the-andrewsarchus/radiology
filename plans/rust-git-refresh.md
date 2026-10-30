# Background checkout Git refresh

Implemented 2026-10-30 on `rust-egui-port`. The app remains standalone.
This increment removes branch/change refresh subprocesses from the GUI thread;
repository root/file discovery is also background work now; see
[repository opening](rust-repository-opening.md).

## Ownership and publication

`repository::git_refresh` prepares one owned GitSnapshot on the existing four-worker
blocking pool. The app owns it through LatestRead, alongside independent source,
index and review jobs. Replacing, cancelling, switching repositories, entering a
review or exiting retires the receiver and signals the subprocess cancellation
flag. There is no extra executor, reader thread or background service.

Successful publication replaces branch and file changes together. It never opens
a file or moves the caret, selection, scroll position or history. A captured
SourceIdentity gates current-file hunks by path, typed revision and content digest.
The worker also checks current disk bytes before and after obtaining those hunks;
changed/missing disk bytes yield no hunks for the stale displayed document.
Navigating to a different document while refreshing still allows project changes
to publish, while retaining that document's independently prepared hunks.

The Changes-tab transition still triggers refresh. During work, a progress row
provides Cancel; accepted changes remain visible and source interaction continues.
Errors retain accepted state and provide Retry/Dismiss. Pending/active PR sessions
never publish checkout results over pinned review data.

## Shared mechanisms and bounds

The existing indexer process-group runner moved to `process`; IndexCommand stays
an alias and the old indexing module reexports the same runner. It retains bounded
first/last diagnostics, deadline/cancel checks, ETXTBSY retry, pipe draining and
kill-group/reap behavior, including descendants holding pipes open.

Checkout Git commands use literal argument arrays, child-only GIT_OPTIONAL_LOCKS=0,
disabled external diff/text conversion and a 30-second total worker deadline.
A stdout observer retains complete records up to 16 MiB per command. Overflow is
an error that kills/reaps the subprocess, rather than silently parsing truncated
diagnostic text. Status/count queries avoid obtaining the full repository patch.
Only the captured displayed file needs a zero-context patch.

PR review and checkout refresh share FileChange's NUL name/status parser. Shared
record validation rejects missing terminators/fields, invalid UTF-8 and invalid
repository paths. Tabs, newlines, spaces and Unicode remain literal paths; renamed
paths and NUL numstat rename records retain their identity. Binary counts stay
zero. Untracked paths are enumerated with `ls-files --others -z`; unborn repos
also include staged paths. Added displayed files receive full-file added hunks.
Untracked text reads obey the source file-size limit; oversized/binary files show
zero text counts. Symlinks count the link without following its target for counts.
Plain folders and bare repositories yield an empty checkout snapshot.

HEAD is verified again before publication; moving commits reject the snapshot
and offer Retry. These sequential read-only commands do not constitute an atomic
filesystem snapshot if arbitrary working-tree files change during the scan.
Individual filesystem reads/native parsing still need finer interruption.
[Source-loader Git processes](rust-source-process-cancellation.md) now use the
same bounded cancellation path. Other cache/index lifecycle work remains open.

## Verification

Full workspace tests, 30 doctests, strict Clippy, formatting, build and rustdoc
pass. New fixtures cover literal Unicode/tab/newline paths, deletes, binary and
untracked files, staged/untracked unborn files, plain/bare folders, malformed
NUL records, current disk identity, added-file hunks, cancelled-before-launch
work, running deadlines with child reaping, oversized stdout and a moving HEAD.
App tests cover error/cancel retention, independent project/document publication,
foreign-root rejection and repository-transition retirement. Existing process
fixtures continue to cover running cancellation and descendant cleanup.

Architecture baseline `before-git-refresh` found an initial overly complex scan;
separating tracked status/count assembly, added files and selected-source hunks
removed it. Final comparison adds no hotspot, dependency cycle or clone group.
The tool still labels existing fragments inside the touched app test file as
regressions; that file only adds Git polling to its common settle helper and the
new test module declaration. The existing fragments were not expanded.

Native acceptance used a private Weston/Niri session, real Git fixture and a
child-only Git wrapper to delay/fail refresh. The wrapper does not affect the host
or production app. The existing sibling capture helper was used only for testing.
Observed: busy progress and Cancel with accepted changes/source visible; real
caret movement to line 5 during the delay; cancellation retaining that state;
a failed diff retaining +2/-1 and exposing Retry/Dismiss; Retry updating to +3/-1
after a private fixture edit while retaining the source/caret. App PID 181371,
Weston and Niri exited zero; pointer helpers were intentionally terminated.

Local artifacts: `/tmp/radiology-git-refresh-native/manifest.json`, screenshots
`settled.png`, `refreshing.png`, `cancelled.png`, `failed.png`, `retried.png` and
`/tmp/radiology-git-refresh-driver.py`. SHA-256 of the tested binary:
`8280e3fe0ed4eb6389ecc28d3c480be151ce218d9ae575d061dba147fd7d4a80`.
Screenshot SHA-256 values, respectively:

- `32564836329ff4a3827350a17f20d67468dee90306e068facb9e2c02d673e5e2`
- `9dc558140f9cbb7d68a2efcfbb32ff821a7abbc73937bcc0e40a08d4adfd8933`
- `30e6923759af3a7beea7c26a7fa68680e168ba595d275cbfba7028841552824b`
- `127e8f6f281c654902bfc34de6cec746ec9a10d2000f88ac8bb0695739f4e771`
- `3424bf58422cd83044e2966aa999d7711c1ae813f5a1bfde4e51d45439df08f6`
