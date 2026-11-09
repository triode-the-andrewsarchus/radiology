# Installer cancellation: direct children and descendants

## Failure and contract

The same-file preview checkpoint's full workspace run had 884 passing app tests
and one Clang provisioning interruption failure. The shared interruption helper
required the PID recorded by fake curl to be absent immediately after joining the
installation worker. The isolated test passed; the failed PID was absent on later
inspection, but its state at the failed assertion was not captured.

The process runner starts a separate process group, sends SIGKILL to that group
on retirement and waits for its direct child. For Clang the direct child is the
installer shell, which launches curl. Curl is a descendant, so its new parent is
responsible for reaping it after the shell exits. PID existence alone does not
prove a descendant is still running: an exited zombie can remain visible until
that reaper runs. This distinction is already enforced in process group tests.

The Clang fixture now records both identities while the download is paused:
`paused.pid` names the direct installer shell, and `child.pid` names curl. Both
cancel and timeout still use the unchanged strict interruption helper to require
the shell PID to be absent with ESRCH. They separately reuse the existing Linux
process-group check to require curl to disappear or reach zombie state within two
seconds. A running or stopped descendant still fails. Existing executable-byte,
fresh successful retry and staging-directory assertions remain in the same loop.
No production cleanup or timeout changes are made.

## Reuse and verification

The existing descendant helper moves from the private process test module to
`process::test_support`, compiled only for Linux tests. Both original group-kill
checks and installer cancellation use one implementation. This is real reuse,
not an extra process policy. No public API, worker or dependency is introduced.

Focused checks pass: eleven process tests, thirteen provisioning tests (including
both Clang cancellation and timeout), and strict all-target workspace Clippy.
The fresh locked/offline full workspace run passes 885 app tests (eleven existing
opt-in ignores), two binary/13 font/22 text tests and all 66 doctests. It compiles
with -j 1 and runs tests serially; normal half-core jobs=16 remains unchanged.
The original failed workspace attempt remains archived alongside corrected logs.
Archive: `~/.local/state/radiology-qa/memory-preview-20261109-1511674/`.

Architecture against `final-memory-preview-20261109`: +14 measured NLOC/two measured
functions/one focused test-support file. Average/max CCN remain 3.42/132, with
36 hotspots and dependency cycles unchanged. Complete normalized clone multisets
remain identical: 124 instances, zero added/removed. The raw comparator's preview
clone flag predates this test-only change and is unchanged by content.

This corrects the verification contract; it does not claim that the earlier
uncaptured PID state was conclusively diagnosed, nor resolve the separate known
parallel timing/linking limits or establish full Swift parity.
