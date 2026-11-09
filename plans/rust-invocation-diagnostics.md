# Actual indexer invocation diagnostics

Swift retains the attempted command with each project outcome, including failures.
Rust previously retained version, bounded stdout/stderr and failure status but no
actual host invocation. That gap is repaired with optional command_line and
working_directory fields on StrategyOutcome and a Copy Command action under the
existing Generation Details disclosure.

## Capture and compatibility

The shared ProcessCommand.diagnostic_line formatter quotes the actual executable
and literal argv through the existing shlex dependency. Valid UTF-8/non-NUL values
round-trip exactly, including spaces, quotes, empty strings, Unicode, newlines and
shell-like text. Non-UTF-8/NUL values use explicit escaped debug notation, without
lossy replacement. Environment overrides and resource contents are excluded.
Execution continues to use the original ProcessCommand, never its display string.

run_command captures the display and working directory immediately before the
attempt. Nonzero exits, missing-executable spawn failures and subsequent decoding
failures retain them. A failure during preparation or probing leaves both absent,
since no compiler invocation was attempted. Project reuse retains the originating
invocation rather than inventing one for the destination. Normalized caches retain
the same fields. Private run directories may already have been removed.

Both fields deserialize with defaults, so old format-3 project/normalized records
stay readable and show no fabricated command. The wire format is unchanged; Rust
callers constructing StrategyOutcome literals manually must add None for these
new optional fields. The shared ProcessCommand API only gains a formatter method.

## Reader UI

Generation Details shows the working directory and monospace invocation inside
its existing bounded two-axis scroll area. Copy Command writes the complete
recorded text to the clipboard; its tooltip explains that private input files may
no longer exist. It does not execute commands, navigate, move the caret, alter
history or modify source. Old records have no copy button. The summary/count/
provenance rows and detail renderer now have separate cohesive functions, keeping
the added clipboard and invocation behavior out of the summary branch logic.
No new modal, worker, display state or parser was added.

## Evidence

Archive: ~/.local/state/radiology-qa/index-diagnostics-20261109-5f729cc/.
Two formatter cases prove exact token round-trips and escaped non-UTF-8/NUL output.
Service cases cover nonzero/spawn failure capture and absent fields on malformed
preparation; old outcome records deserialize with None. The seven existing project
reuse cases now also compare original command/cwd snapshots through reuse and
normalized reload. The actual app's failure disclosure is expanded and Copy
Command is clicked at 1×/2×, checking exact clipboard contents, no extra tool run,
unchanged reader/history and absence of a button for records without a command.
All nine lifecycle cases and seven reuse cases pass with strict all-target Clippy,
normal build and warning-free docs. The generated formatter/example/outcome pages
were inspected. The combined full serial workspace gate passes 932 app tests
(twelve opt-in ignores), binary/font/text 2/13/23 and all 68 doctests (60+4+4).
This checkpoint claims real-widget frame checks,
not a new native-desktop command-disclosure run.

Architecture +222 measured NLOC/net six analyzed functions, no new source file,
dependency, hotspot or cycle; avg/max CCN 3.43/132. All 125 normalized clone instances
remain identical, zero additions/removals. The initial outcome-renderer hotspot was
removed by separating summary from details. Existing recovery assertions now share
their fixture's repaired-index contract, removing a newly detected repeated block.
Remaining raw clone/Python-rank alerts concern pre-existing normalized clones and
untouched equal-ranked functions.

The preceding [adapter validation](rust-adapter-validation.md) checkpoint also
participates in this combined verification. These close the two concrete lifecycle
gaps identified after automatic opening. Full IndexingTests assertion reconciliation
still remains, alongside Kotlin/PR, inspector/traits and final integrated/native/
standalone/shared-host acceptance. Compiler/personality remains 17/35 mapped until
that remaining proof is complete; accessibility and actual sibling adoption remain
deferred.
