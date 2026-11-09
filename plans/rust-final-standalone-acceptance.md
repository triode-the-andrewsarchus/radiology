# Fresh standalone and external-host acceptance

Audited source snapshot: e49c5ecd26417335188a9cfa87bd91cda8db8065. A git archive of
that committed tree is extracted into a new QA directory outside the working
checkout. A previously empty target directory builds the complete exported
workspace locked/offline with verbose compiler output. The exported Cargo config
sets sixteen jobs for the process's thirty-two available logical CPUs. The
workspace lockfile is byte-identical to the current repository's. No sibling
source, binary or target output is used by the product build.

This closes the affected standalone/shared-host maintenance gate for this source
snapshot. It does not close the final native/integrated acceptance or complete
original/upstream ledger audit, and it does not prove SDK-backed Android success.
The artifact is a Linux debug build, not an installer or cross-platform package.

## Independent consumers

The existing external full-reader consumer is copied into the new QA directory,
with its own manifest/lock and explicit root-level pinned Kotlin grammar and
Wayland backend patches. It consumes only the exported Radiology library and local
reader-text/reader-fonts path crates. Locked/offline build and actual headless egui
frames pass at both 1× and 2×: source focus, exact line-sixty viewport restoration
without moving the line-one caret, original Unicode Select All/Copy, changed-source
anchor rejection, focus return, shared fragments/font configuration and native
Kotlin parsing. The public SourceViewContent constructor and FollowDefinition
handler compile independently of the app's construction paths.

The asset/text-only consumer uses reader-fonts with default features disabled and
reader-text directly. Its locked/offline build/runtime validates font assets and
exact CRLF/Unicode selection. Target-filtered Linux Cargo metadata contains no
Radiology, egui/eframe, Wayland, portal or file-dialog packages. Metadata for all
three workspaces confirms every path package resolves within the new export or
its two consumers, with no little-screen-clip dependency.

The initially copied full-host lock did not resolve against the current path
manifest in offline mode: Cargo selected an arboard 3.6.1/clipboard-win 5.4.1 branch
and rejected the yanked Windows version. The original diagnostic/lock are retained.
The QA consumers were seeded from the current application lock's pins, then Cargo
updated each consumer-only lock offline. Both consumers subsequently pass again
with --locked --offline. The application lock and dependency definitions remain
unchanged. This is a refreshed reproducible consumer fixture, not a product fix
or a claim that the failed initial command passed.

## Source-unavailable runtime

After all builds finish, the exported source directory is renamed away. Copies of
the app and both independent consumer executables run from an otherwise empty
run directory. The app's --verify-resources checks all eleven native parser/query
families (Rust, Python, Go, JavaScript, TypeScript, JSX, TSX, Swift, Kotlin, C and
C++) and five syntax-color fallbacks (HTML, JSON, Markdown, YAML and TOML). Both
consumers rerun their complete assertions, including the reader's two-scale cases.
All three exit zero with the export unavailable. The source is restored in finally.
No working repository or sibling directory is renamed or modified.

This demonstrates compiled resource independence for these exercised paths and
reusable library/text/font contracts. The full host remains responsible for app
lifecycle, source publication and semantic actions; actual adoption into
little-screen-clip is explicitly deferred. The intended shared-crate/API boundaries
are documented in rust-reader-host-audit.md and remain unchanged.

## Evidence

Archive: ~/.local/state/radiology-qa/final-standalone-20261109-e49c5ec/.
Scratch: ~/.cache/radiology-qa-final-standalone-20261109/.
accepted.json records commit, CPU/job counts, lock preservation and gate results.
The archive retains the complete source tar/hash, both independent consumer
manifests/sources/generated locks, original failed lock/diagnostic, verbose clean
build, locked runtime logs, target-filtered metadata/path-package audits, copied
executable identities and all three source-unavailable stdout/exit records.

The preceding serial workspace gate passes 942 app tests (twelve opt-in ignores),
2/13/23 binary/font/text tests and all 68 doctests; strict all-target Clippy passes.
No production code or dependency changes occur in this acceptance checkpoint.
New runtime/API/resource changes require relevant gate reruns before completion.
