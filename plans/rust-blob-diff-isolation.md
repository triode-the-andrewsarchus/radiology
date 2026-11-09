# Per-file review diff isolation

Reference: `ReviewRepositoryTests.renamedBlobDiffExcludesAChildOfTheReusedOldPath`
in [Swift's repository suite](../Tests/RadiologyTests/ReviewRepositoryTests.swift).
Reviewed against Rust `7f7670e`.

## Behavior

The real Git regression first failed: an unchanged rename acquired an added-line
highlight from a child created under its former path. Even literal Git pathspecs
can recurse into that directory. Per-file patches now compare the exact Before
and After object IDs from the raw change record. Counts, hunks, line mapping and
old-path aliases therefore describe the selected file only.

Added/deleted sides share one format-correct empty blob per change batch, written
into the existing private review object store. The existing cancellable Git reader
provides EOF on stdin. No attached checkout, index or reference is changed. The
empty-object operation has its own cancellation/timeout gate in the existing
interruption matrix, alongside raw metadata and patch execution.

Raw changes now retain Before/After Git modes. Gitlinks identify commits, so they
need their usual `Subproject commit <id>` display line materialized as a private
blob before comparison. This preserves pointer-only diffs without fetching or
traversing a submodule. Ordinary files and symbolic links use their original IDs.
Private temporary files use the existing RAII tempfile support and lossless paths.
No dependency, worker, persistence schema or sibling-app coupling was added.

## Host API

`FileChange` gains public `before_mode` and `after_mode` fields containing octal Git
mode bits (for example `0o100644` and `0o160000`). Zero denotes an absent side or
unavailable name/status metadata. Serde defaults preserve reads of older records;
Rust struct literals must supply the two new fields. Raw records retain their full
object IDs; gitlink IDs refer to commits rather than blobs. Existing examples and
in-tree callers have been migrated.

## Verification

Archive: `~/.local/state/radiology-qa/blob-diffs-20261109-7f7670e/`.

- `blob-red.log`: unchanged rename incorrectly reports After `[1..2]`.
- `per_file_diffs_exclude_children_of_reused_paths_for_renames_additions_and_deletions`
  covers pure and edited renames, Unicode/CRLF exact bytes, old-path directory
  reuse, addition replacing a directory, deletion replaced by a directory,
  isolated counts/hunks, old aliases, stable line mapping and dirty checkout/HEAD.
- Existing raw metadata tests check modes for both 40- and 64-digit IDs.
- The existing mode/link/smudge/hook test exposed the initial commit-versus-blob
  regression; its final assertions verify gitlink mode and one-line addition.
- Existing SHA-256 cache/workspace proof now includes added and deleted files.
- `pull-request-modes.log`: 39 focused tests pass before the final added mode
  assertions. Strict Clippy, normal build and warning-free API documentation pass.
- `workspace.log`: locked offline serial run passes 831 app tests (11 existing
  ignored), two binary, 13 font, 22 text tests and 64 doctests. This run preceded
  the final strict-octal validation guard and its malformed-mode assertion;
  `pull-request-final.log` verifies the affected suite after that small adjustment.
  Build, strict Clippy and warning-free docs were also rerun afterward.

## Architecture review

Baseline `before-blob-diffs-20261109` to `final-2-blob-diffs-20261109`:
+192 measured NLOC, three measured functions, no new files; average CCN 3.40→3.41,
maximum 132 and 36 high-complexity functions unchanged. All 122 clone instances
and 1,317 duplicated lines are unchanged, including the complete normalized
file-pair/fragment multiset with multiplicities. One dependency cycle and eight
structural issues remain unchanged. The reported equal-score chokepoint ranking
change is in untouched Python code, not a new dependency. New work reuses the
Git reader, object validation, private store, parser and existing fixture; no
parallel diff implementation or process runner was introduced.

This explicitly maps one of four ReviewRepositoryTests functions. The other three
repository groups, separate review/presentation/fold suites, final integration
acceptance and Android remain open. A dedicated app-level reused-path workflow
would strengthen this service-level proof; existing app diff/navigation checks
run in the full suite, but do not independently reproduce this exact path shape.
