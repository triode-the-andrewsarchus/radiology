# Rust explicit semantic-tool provisioning

The reference is ToolProvisioner.swift, SemanticIndexService.install and the Swift
indexer adapters. Install remains an explicit user action, followed by regeneration.
Checking tools, opening a repository and generating an index never install software.

## Supported plans and scope

| Adapter | Installation plan | Scope |
| --- | --- | --- |
| Rust | rustup component add rust-analyzer | Active user rustup toolchain; described explicitly in the UI |
| Go | github.com/scip-code/scip-go/cmd/scip-go@v0.2.7 | Private GOBIN, module and build caches |
| TypeScript/JavaScript | @sourcegraph/scip-typescript@0.4.0 through npm | Private package prefix/cache; requires Node.js 22+ and npm |
| Python | @sourcegraph/scip-python@0.6.6 through npm | Private prefix/cache; requires Node/npm and Python 3.10+ |
| Kotlin/Java | Shared pinned scip-java installer, commit 9f288dd339c7e9fc257a7b21714d9a402fd4d837, tool 0.13.1-radiology-9f288dd339c7, Gradle 9.7.1 | Private source/build/distribution; JDK 17+, curl, tar and unzip required |
| C/C++ | Manual scip-clang installation | No automatic installer, matching Swift |

Pins match the existing Swift implementation; they are not claims about latest
upstream versions. Kotlin reuses the existing installer resource, with Linux
sha256sum/cp fallbacks while preserving the macOS shasum/ditto paths. Generation
now locates a managed or PATH Gradle fallback for projects without wrappers.

## Execution, ownership and publication

ProvisioningRequest owns tool context, protected viewed roots and one total budget.
The app uses a 30-minute budget covering writer waiting, execution and verification.
Package-manager arguments remain literal ProcessCommand fields, including paths
with spaces/shell metacharacters. The existing bounded subprocess runner supplies
progress, diagnostics, cancellation, timeouts, process-group cleanup and reaping.
A shared no-follow file lease serializes installers across app instances. Installer
children inherit that lease so a surviving writer cannot be overtaken after app exit.

Go/npm/Kotlin installs run in unique private staging directories. Verification must
find the staged managed executable, not a PATH fallback. Publication retains the
previous tool until a final probe succeeds at its permanent path, rolling back on
publication/probe failure. Failed rollback preserves the backup and reports its
location. Directory replacement is cooperative and has a brief rename interval;
this is not a filesystem-wide atomic exchange guarantee. Temporary stages and
retired backups are removed during normal completion/cancellation. Crash-left
stages may remain for later maintenance; no automatic adoption is claimed.
Rust uses rustup's own component operation instead of staged namespace publication.

Install destinations inside viewed roots are rejected before writes; Rustup's
active toolchain is also checked for overlap. Package installation never uses the
viewed repository as its working directory. Tool directories normally live under
~/.config/radiology/tools (respecting XDG_CONFIG_HOME). Project dependencies and
compiler toolchains remain prerequisites; this does not install project node_modules.

## Reader workflow

Index Projects exposes Install and Index after a failed/missing tool probe or a
missing-tool generation outcome, with explicit installation scope/prerequisites.
Clang shows manual guidance. All row installs disable during installation or index
generation; missing private storage also disables installation. Progress and Cancel
Installation are visible in the project window and status bar. Success records the
verified executable/version and regenerates the current checkout or pinned After
index. Failure/Cancel retain accepted semantic data. Repository/snapshot changes
retire installation publication; late results cannot generate in another reader.

## Automated evidence

On 2026-10-30, full gates pass 406 app tests (one ignored), eight shared reader tests,
30 doctests, formatting, strict Clippy, build and rustdoc; sh -n accepts the shared
Kotlin installer. Eight provisioning tests exercise pinned literal plans and missing
prerequisites, private staged installs, refusal of PATH fallback, viewed-root
protection, live child Cancel/timeout with successful retry, publication rollback,
writer deadline and actual Go/Rustup command/probe stand-ins. Two reader tests prove
automatic generation after success and Cancel/root-replacement isolation. Existing
five PR recovery and eleven process tests also pass after shared lease extraction.
Fixtures do not fetch packages or claim real compiler/release installation acceptance.

Architecture baseline before-tool-provisioning: no new complexity hotspot/cycle or
clone growth after sharing UI controls and test setup. Four reported clone groups
are unchanged baseline app-test fragments in a touched file. Shared file leases,
observed processes, LatestRead, tool probes and isolated generation are reused.
Logs: /tmp/radiology-provision-{tests,app-tests,process-tests,lease-tests,all-tests,clippy,build,doc}.log.

## Native acceptance — 2026-10-30

Private Wayland acceptance used a dirty TypeScript fixture and controlled local
npm/indexer executables. Five installer attempts covered live Cancel/reaping,
installer failure, successful install/automatic generation, failure with accepted
semantic data retained, and successful recovery/automatic regeneration. The final
UI reports scip-typescript 0.4.0, one document and zero missing sources. Installer
failure now displays `exited with status 7` rather than Rust's `Some(7)` formatting.
A QA-only rename of the managed executable simulated missing tooling while the
accepted index remained; this did not touch viewed checkout inputs.

Both index runs read the exact dirty checkout bytes in private workspaces. Fixture
verification confirms unchanged HEAD/status/source/manifests, no checkout build
artifacts, removed workspaces/stages/retired backups and the verified managed tool.
The driver session is terminal and both app PIDs are absent; final shutdown exit
codes were not retained, so clean-zero shutdown is not asserted here.

Evidence: `/tmp/radiology-tool-provision-native/acceptance.json` and `manifest.json`
(SHA-256 of captures, logs, fixture, driver and final binary). Key captures are
`installation-live.png`, `installation-cancelled-reader-retained.png`,
`failed-install-accepted-index-retained.png`, `recovered-installed-and-indexed.png`
and `accepted-reader.png`. The misnamed `accepted-install-failure.png` shows only a
missing tool and is not failure evidence. The driver uses the sibling capture
helper for QA only; the standalone app gains no sibling runtime dependency.

After the diagnostic wording fix, eight focused provisioning tests, strict Clippy
and build pass; the full 406-app/eight-shared/30-doctest gates above preceded that
small follow-up. Logs: `/tmp/radiology-provision-status-{tests,clippy,build}.log`.
This proves controlled native lifecycle behavior, not real package/compiler runs.

Full parity remains open: complete project/version-aware cache reuse, real-tool/corpus acceptance,
broader corrupt-store recovery and final visual/accessibility/scaling/font/sibling
reuse audit.
